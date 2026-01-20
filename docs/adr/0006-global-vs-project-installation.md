# ADR 0006: Global vs Project-Level Installation

## Status

Accepted

## Context

Skills can be useful at two scopes:

1. **Project-level**: Skills specific to a project, shared with the team via version control
2. **Global**: Personal skills available across all projects

The [add-skill](https://github.com/vercel-labs/add-skill) CLI supports both modes with a `-g/--global` flag. This pattern is familiar from package managers like npm (`npm install -g`).

Currently, skilo only operates at the project level. Users working on multiple projects may want personal skills (e.g., coding style preferences, custom workflows) available everywhere without duplicating them.

## Decision

We will add global installation support to skilo via a `--global` flag.

### Installation Locations

| Mode              | Directory                  | Example                               |
| ----------------- | -------------------------- | ------------------------------------- |
| Project (default) | `.claude/skills/<name>/`   | `./project/.claude/skills/my-skill/`  |
| Global            | `~/.claude/skills/<name>/` | `/home/user/.claude/skills/my-skill/` |

For other agents (per ADR 0005):

| Agent       | Project            | Global               |
| ----------- | ------------------ | -------------------- |
| Claude Code | `.claude/skills/`  | `~/.claude/skills/`  |
| Cursor      | `.cursor/skills/`  | `~/.cursor/skills/`  |
| Codex       | `.codex/skills/`   | `~/.codex/skills/`   |
| OpenCode    | `.opencode/skill/` | `~/.opencode/skill/` |
| Antigravity | `.agent/skills/`   | `~/.agent/skills/`   |

### Command-Line Option

Add `--global` flag to relevant commands:

```bash
# Install skill globally
skilo add anthropics/skills --global

# Create new skill globally
skilo new my-personal-skill --global

# List global skills
skilo list --global

# List project + global skills
skilo list --all

# List skills from specific project + global
skilo list /path/to/project --all

# Lint/format global skills (pass path directly)
skilo lint ~/.claude/skills/
skilo fmt ~/.claude/skills/
```

Short form: `-g`

```bash
skilo add anthropics/skills -g
```

### Affected Commands

| Command      | Global Behavior                            |
| ------------ | ------------------------------------------ |
| `skilo add`  | Install to `~/.claude/skills/`             |
| `skilo new`  | Create in `~/.claude/skills/`              |
| `skilo list` | List skills in global directory            |

Note: `lint`, `fmt`, and `check` operate on the provided path directly. To lint/format global skills, pass the path explicitly: `skilo lint ~/.claude/skills/`

### New Command: `skilo list`

List installed skills at project or global level:

```bash
# List project skills (default)
skilo list

# List project skills from specific directory
skilo list /path/to/project

# List global skills only
skilo list --global

# List all skills (project + global)
skilo list --all

# List skills from specific project + global
skilo list /path/to/project --all
```

Output:

```
Project skills (.claude/skills/):
  code-review      Review code changes
  run-tests        Run test suite

Global skills (~/.claude/skills/):
  my-workflow      Personal workflow automation
  style-guide      My coding style preferences
```

### Precedence

When both project and global skills exist with the same name:
- Project skills take precedence (closer scope wins)
- `skilo list --all` shows both with indicators
- Installing a skill that exists at the other scope triggers a warning

```
Warning: Skill 'code-review' exists globally.
Project installation will shadow the global version.
Continue? [y/N]
```

### Configuration

Add to `.skilorc.toml` (project-level):

```toml
[install]
scope = "project"  # default scope: project | global
```

Add to `~/.config/skilo/config.toml` (user-level):

```toml
[install]
default_scope = "project"  # project | global

[global]
skills_dir = "~/.claude/skills"  # Override global directory
```

### Environment Variable

Support `SKILO_GLOBAL_DIR` to override the global skills directory:

```bash
SKILO_GLOBAL_DIR=~/my-skills skilo add anthropics/skills -g
```

### Example Workflows

**Personal productivity skills:**

```bash
# Install your preferred skills globally
skilo add my-org/personal-skills -g

# Available in every project automatically
cd ~/projects/new-project
# Skills from ~/.claude/skills/ are available
```

**Team project with shared skills:**

```bash
cd ~/projects/team-project

# Install team skills at project level (committed to git)
skilo add team-org/standard-skills

# .claude/skills/ is gitignored by default, so add to git
git add .claude/skills/
git commit -m "Add team skills"
```

**Mixed setup:**

```bash
# Global: personal preferences
skilo add personal/my-skills -g

# Project: team standards
skilo add team/standards

# Both are available, project takes precedence on conflicts
```

### Directory Creation

When `--global` is used and the global skills directory doesn't exist, skilo creates it:

```bash
$ skilo add anthropics/skills -g
Creating ~/.claude/skills/...
Installing code-review... done
```

## Consequences

### Positive

- Personal skills available across all projects
- Familiar pattern from npm, cargo, pip
- Clear separation between personal and team skills
- No duplication of common personal skills

### Negative

- Two locations to manage and remember
- Potential confusion about which skills are active
- Global skills not version-controlled by default
- Name conflicts between scopes require resolution

### Neutral

- Project-level remains the default
- Existing behavior unchanged without `--global`
- Integrates with multi-agent support (ADR 0005)

## Implementation Notes

### Dependencies

- `dirs` - Cross-platform home directory detection

### Module Structure

```
src/
├── scope/
│   ├── mod.rs          # Scope enum and resolution
│   ├── project.rs      # Project directory detection
│   └── global.rs       # Global directory handling
└── ...
```

### Platform Considerations

| Platform | Global Base Path                |
| -------- | ------------------------------- |
| Linux    | `~/.claude/skills/`             |
| macOS    | `~/.claude/skills/`             |
| Windows  | `%USERPROFILE%\.claude\skills\` |

### XDG Compliance (Linux)

Consider XDG Base Directory specification:

```
~/.local/share/claude/skills/  # XDG_DATA_HOME
```

However, for consistency with agent conventions, use `~/.claude/skills/` as agents expect this location.

## References

- [add-skill CLI](https://github.com/vercel-labs/add-skill)
- [npm global install](https://docs.npmjs.com/cli/commands/npm-install#global)
- [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
