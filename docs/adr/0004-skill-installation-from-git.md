# ADR 0004: Skill Installation from Git Repositories

## Status

Accepted

## Context

Skilo currently focuses on skill creation, validation, and formatting. The [add-skill](https://github.com/vercel-labs/add-skill) CLI tool demonstrates a complementary workflow: installing skills from remote git repositories.

The skill ecosystem benefits from sharing and reuse. Developers need a way to:

- Discover and install skills from public repositories
- Share skills across projects without copy-pasting
- Pull skills from curated skill collections (e.g., `vercel-labs/agent-skills`, `anthropics/skills`)
- Install specific skills from a repository containing multiple skills

Currently, users must manually clone repositories and copy skill directories, which is error-prone and lacks version tracking.

## Decision

We will add a `skilo add` command to install skills from git repositories.

### Command Syntax

```bash
skilo add <source> [options]
```

### Source Formats

| Format | Example |
|--------|---------|
| GitHub shorthand | `owner/repo` |
| Full GitHub URL | `https://github.com/owner/repo` |
| GitLab URL | `https://gitlab.com/owner/repo` |
| SSH URL | `git@github.com:owner/repo.git` |
| Direct skill path | `https://github.com/owner/repo/tree/main/skills/my-skill` |
| Local path | `./path/to/skills` or `/absolute/path` |

### Options

| Option | Short | Description |
|--------|-------|-------------|
| `--skill <name>` | `-s` | Install specific skill(s) by name |
| `--list` | `-l` | List available skills without installing |
| `--yes` | `-y` | Skip confirmation prompts (CI-friendly) |
| `--branch <branch>` | `-b` | Specify git branch (default: default branch) |
| `--tag <tag>` | `-t` | Specify git tag |

### Skill Discovery

When given a repository, skilo searches for skills in these locations (in order):

1. Root directory (if contains `SKILL.md`)
2. `skills/` directory
3. `.claude/skills/` directory
4. Recursive search for `SKILL.md` files

Each discovered skill is validated against the Agent Skills Specification before installation.

### Installation Behavior

1. **Clone**: Fetch repository to temporary directory
2. **Discover**: Find all valid skills in the repository
3. **Select**: Filter by `--skill` option if provided
4. **Validate**: Run `skilo lint` on each skill
5. **Confirm**: Show skills to install and prompt for confirmation (unless `--yes`)
6. **Install**: Copy skill directories to target location
7. **Cleanup**: Remove temporary directory

### Default Installation Location

Skills are installed to `.claude/skills/<skill-name>/` by default (project-level).

### Example Usage

```bash
# Install all skills from a repository
skilo add anthropics/skills

# Install a specific skill
skilo add vercel-labs/agent-skills --skill code-review

# List available skills without installing
skilo add anthropics/skills --list

# Install from a specific branch
skilo add owner/repo --branch develop

# Non-interactive installation for CI
skilo add anthropics/skills --skill lint-fix --yes
```

### Output

```
Fetching skills from github.com/anthropics/skills...

Found 5 skills:
  code-review      Review code changes for quality and correctness
  conventional-commits  Guide for writing conventional commits
  lint-fix         Automatically fix linting issues
  test-runner      Run tests and report results
  pr-review        Review pull requests

Install all 5 skills to .claude/skills/? [y/N] y

Installing code-review... done
Installing conventional-commits... done
Installing lint-fix... done
Installing test-runner... done
Installing pr-review... done

Installed 5 skills to .claude/skills/
```

### Error Handling

| Scenario | Behavior |
|----------|----------|
| Invalid git URL | Exit with error code 2 |
| Repository not found | Exit with error code 3 |
| No skills found | Exit with error code 1, suggest using `--list` |
| Skill validation fails | Skip skill, warn user, continue with others |
| Skill already exists | Prompt for overwrite (skip if `--yes`) |
| Network error | Exit with error code 3 |

### Configuration

Add to `.skilorc.toml`:

```toml
[add]
default_agent = "claude"  # Target agent for installation
confirm = true            # Prompt before installing (false for CI)
validate = true           # Validate skills before installing
```

## Consequences

### Positive

- Enables skill sharing and reuse across projects
- Integrates with existing git workflows
- Maintains validation before installation
- Supports CI/CD with non-interactive mode
- Leverages familiar git repository patterns

### Negative

- Requires git to be installed
- Network-dependent operation
- Must handle various git hosting platforms
- Repository structure assumptions may not fit all layouts

### Neutral

- Complements existing `new`, `lint`, `fmt` commands
- Does not replace manual skill authoring
- Does not include a skill registry (relies on git repositories)

## Implementation Notes

### Dependencies

- `git2` - Git operations (clone, checkout)
- `url` - URL parsing and validation
- `tempfile` - Temporary directory management
- `dialoguer` - Interactive prompts

### Module Structure

```
src/
├── commands/
│   ├── mod.rs
│   ├── add.rs          # New: add command
│   └── ...
├── git/
│   ├── mod.rs          # New: git operations
│   ├── source.rs       # Source URL parsing
│   └── fetch.rs        # Repository fetching
└── ...
```

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | No skills found or all skills failed validation |
| 2 | Invalid arguments or source format |
| 3 | I/O or network error |

## References

- [add-skill CLI](https://github.com/vercel-labs/add-skill)
- [Agent Skills Specification](https://agentskills.io/specification)
- [git2 crate](https://docs.rs/git2/latest/git2/)
