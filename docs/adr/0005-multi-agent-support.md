# ADR 0005: Multi-Agent Support

## Status

Accepted

## Context

The [Agent Skills Specification](https://agentskills.io/specification) is designed to be agent-agnostic. Multiple AI coding agents now support skills:

| Agent       | Skills Directory          | Notes                      |
| ----------- | ------------------------- | -------------------------- |
| Claude Code | `.claude/skills/<name>/`  | Full specification support |
| Cursor      | `.cursor/skills/<name>/`  | Basic skill support        |
| Codex       | `.codex/skills/<name>/`   | Basic skill support        |
| OpenCode    | `.opencode/skill/<name>/` | Note: singular "skill"     |
| Antigravity | `.agent/skills/<name>/`   | Basic skill support        |

Skilo currently assumes Claude Code as the target agent. The [add-skill](https://github.com/vercel-labs/add-skill) CLI demonstrates automatic detection of installed agents and installation to multiple targets simultaneously.

To maximize skill reuse, skilo should support installing and managing skills across all compatible agents.

## Decision

We will add multi-agent support to skilo, enabling skill operations across all specification-compliant agents.

### Agent Registry

Define supported agents with their configuration:

```rust
pub struct AgentConfig {
    pub name: &'static str,
    pub display_name: &'static str,
    pub project_dir: &'static str,    // Relative to project root
    pub global_dir: &'static str,     // Relative to home directory
    pub features: AgentFeatures,
}

pub struct AgentFeatures {
    pub context_fork: bool,    // Supports context: fork
    pub hooks: bool,           // Supports hooks
    pub allowed_tools: bool,   // Supports allowed-tools field
}
```

### Agent Detection

Skilo will detect installed agents by checking for their configuration directories:

| Agent       | Detection Path                 |
| ----------- | ------------------------------ |
| Claude Code | `.claude/` or `~/.claude/`     |
| Cursor      | `.cursor/` or `~/.cursor/`     |
| Codex       | `.codex/` or `~/.codex/`       |
| OpenCode    | `.opencode/` or `~/.opencode/` |
| Antigravity | `.agent/` or `~/.agent/`       |

### Command-Line Option

Add `--agent` option to relevant commands:

```bash
# Install skill to specific agent(s)
skilo add anthropics/skills --agent claude --agent cursor

# Install to all detected agents
skilo add anthropics/skills --agent all

# Use default (claude) if not specified
skilo add anthropics/skills
```

The `--agent` option accepts:
- Agent name: `claude`, `cursor`, `codex`, `opencode`, `antigravity`
- Special value: `all` (all detected agents)
- Multiple values: `--agent claude --agent cursor`

### Affected Commands

| Command       | Multi-Agent Behavior                   |
| ------------- | -------------------------------------- |
| `skilo add`   | Install to specified agent directories |
| `skilo new`   | Create in specified agent directory    |
| `skilo list`  | List skills for specified agent        |

Note: `lint`, `fmt`, and `check` operate on the provided path directly and do not have agent-specific flags.

### Default Behavior

When `--agent` is not specified:
- `add`, `new`, `list`: Use Claude Code (`.claude/skills/`)

### Compatibility Warnings

Some specification features are agent-specific:

```
Installing code-review to cursor...
Warning: Skill uses 'context: fork' which is only supported by Claude Code
Installing... done (some features may not work)
```

### Configuration

Add to `.skilorc.toml`:

```toml
[agents]
default = "claude"              # Default agent for add/new
auto_detect = true              # Auto-detect installed agents
enabled = ["claude", "cursor"]  # Limit to specific agents

[agents.claude]
enabled = true
project_dir = ".claude/skills"
global_dir = "~/.claude/skills"

[agents.cursor]
enabled = true
project_dir = ".cursor/skills"
global_dir = "~/.cursor/skills"
```

### Example Usage

```bash
# List detected agents
skilo agents

# Output:
# Detected agents:
#   claude    .claude/skills/     (3 skills)
#   cursor    .cursor/skills/     (1 skill)

# Install to all detected agents
skilo add anthropics/skills --agent all

# Install to specific agents
skilo add anthropics/skills --agent claude --agent cursor

# Create new skill for cursor
skilo new my-skill --agent cursor
```

### New Command: `skilo agents`

List detected agents and their skill counts:

```bash
skilo agents [--verbose]
```

Options:
- `--verbose`: Show feature support matrix and skill details

## Consequences

### Positive

- Skills become truly portable across agents
- Single tool for all agent skill management
- Automatic agent detection reduces configuration
- Warns about incompatible features before installation

### Negative

- Increased complexity in installation logic
- Must track agent-specific directory conventions
- Agent feature sets may diverge over time
- Testing matrix grows with each agent

### Neutral

- Claude Code remains the default target
- Existing behavior unchanged when `--agent` not specified
- Agent configurations can be customized

## Implementation Notes

### Dependencies

- `dirs` - Home directory detection
- `once_cell` - Lazy agent registry initialization

### Module Structure

```
src/
├── agents/
│   ├── mod.rs          # Agent registry and detection
│   ├── config.rs       # Agent configuration types
│   ├── detect.rs       # Agent detection logic
│   └── features.rs     # Feature compatibility checking
└── ...
```

### Agent Feature Matrix

| Feature         | Claude | Cursor  | Codex   | OpenCode | Antigravity |
| --------------- | ------ | ------- | ------- | -------- | ----------- |
| Basic skills    | Yes    | Yes     | Yes     | Yes      | Yes         |
| `context: fork` | Yes    | No      | No      | No       | No          |
| Hooks           | Yes    | No      | No      | No       | No          |
| `allowed-tools` | Yes    | Partial | Partial | Partial  | Partial     |
| Scripts         | Yes    | Yes     | Yes     | Yes      | Yes         |

## References

- [add-skill CLI](https://github.com/vercel-labs/add-skill)
- [Agent Skills Specification](https://agentskills.io/specification)
- [Claude Code Skills Documentation](https://docs.anthropic.com/en/docs/claude-code/skills)
