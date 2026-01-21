# skilo

[![CI](https://github.com/manuelmauro/skilo/actions/workflows/ci.yml/badge.svg)](https://github.com/manuelmauro/skilo/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/skilo.svg)](https://crates.io/crates/skilo)

A CLI tool for [Agent Skills](https://agentskills.io/specification) development.

## Installation

```bash
# Quick install
curl -sSfL https://raw.githubusercontent.com/manuelmauro/skilo/main/install.sh | sh

# Or from crates.io
cargo install skilo
```

## Quick Start

```bash
skilo new my-skill                    # Create a skill from template
skilo add owner/repo                  # Install skills from git
skilo remove my-skill                 # Remove a skill
skilo list                            # List installed skills
skilo lint .                          # Validate skills
skilo fmt .                           # Format SKILL.md files
```

Run `skilo -h` for all commands and options.

## Using the Skilo Skill

Install the use-skilo skill to teach your AI assistant how to use skilo:

```bash
skilo add manuelmauro/skilo --skill use-skilo
```

## Commands

| Command            | Description                        |
| ------------------ | ---------------------------------- |
| `new`              | Create a skill from template       |
| `add`              | Install skills from git/local path |
| `remove`           | Remove installed skills            |
| `list`             | List installed skills              |
| `agents`           | List detected AI coding agents     |
| `cache`            | Manage git repository cache        |
| `lint`             | Validate skills against spec       |
| `fmt`              | Format SKILL.md files              |
| `check`            | Run lint + format check            |
| `read-properties`  | Output skill metadata as JSON      |
| `to-prompt`        | Generate XML for agent prompts     |
| `self update`      | Update skilo to latest version     |
| `self completions` | Generate shell completions         |

## Skill Structure

```
my-skill/
├── SKILL.md        # Required: manifest with YAML frontmatter
├── scripts/        # Optional: executable scripts
├── references/     # Optional: additional docs
└── assets/         # Optional: static resources
```

### SKILL.md Format

```markdown
---
name: my-skill
description: What the skill does and when to use it.
license: MIT
---

# My Skill

Instructions for the AI agent.
```

## Configuration

Create `.skilorc.toml` for project settings:

```toml
[lint]
strict = true

[lint.rules]
name_format = true
name_length = 64
body_length = 500

[new]
default_license = "MIT"
default_template = "hello-world"

[add]
# default_agent = "claude"  # Optional: defaults to ./skills/
confirm = true
```

See `skilo lint --help` for all available rules.

## Multi-Agent Support

Skilo supports 14 AI coding agents. By default, skills install to `./skills/`. Use `--agent` to target specific agents:

```bash
skilo add owner/repo --agent claude           # Install to .claude/skills/
skilo add owner/repo --agent all              # Install to all detected agents
skilo new my-skill --global --agent claude    # Create global skill
skilo agents                                  # List detected agents
```

## Environment Variables

| Variable        | Description                                |
| --------------- | ------------------------------------------ |
| `SKILO_CONFIG`  | Path to configuration file                 |
| `SKILO_HOME`    | Override skilo home (default: `~/.skilo/`) |
| `SKILO_CACHE`   | Override git cache directory               |
| `SKILO_OFFLINE` | Set to `1` for offline mode                |

## Shell Completions

```bash
# Bash (add to ~/.bashrc)
eval "$(skilo self completions bash)"

# Zsh (add to ~/.zshrc)
eval "$(skilo self completions zsh)"

# Fish (add to ~/.config/fish/config.fish)
skilo self completions fish | source
```

## CI Integration

```yaml
- name: Validate skills
  run: |
    curl -sSfL https://raw.githubusercontent.com/manuelmauro/skilo/main/install.sh | sh
    skilo check --strict .
```

## License

MIT OR Apache-2.0
