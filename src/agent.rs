//! Supported AI coding agents and their skill directories.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Supported AI coding agents.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Agent {
    /// OpenCode.
    OpenCode,
    /// Claude Code by Anthropic.
    #[default]
    Claude,
    /// Codex by OpenAI.
    Codex,
    /// Cursor.
    Cursor,
    /// Amp by Sourcegraph.
    Amp,
    /// Kilo Code.
    KiloCode,
    /// Roo Code.
    RooCode,
    /// Goose by Block.
    Goose,
    /// Gemini CLI by Google.
    Gemini,
    /// Antigravity.
    Antigravity,
    /// GitHub Copilot.
    Copilot,
    /// Clawdbot.
    Clawdbot,
    /// Droid.
    Droid,
    /// Windsurf by Codeium.
    Windsurf,
}

/// Agent feature support flags.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AgentFeatures {
    /// Supports `context: fork` in SKILL.md.
    pub context_fork: bool,
    /// Supports hooks.
    pub hooks: bool,
    /// Supports `allowed-tools` field.
    pub allowed_tools: bool,
    /// Supports scripts.
    pub scripts: bool,
}

/// Information about a detected agent.
#[derive(Debug, Clone)]
pub struct DetectedAgent {
    /// The agent type.
    pub agent: Agent,
    /// Path to the skills directory (project or global).
    pub skills_path: PathBuf,
    /// Number of skills found in this location.
    pub skill_count: usize,
    /// Whether this is a global installation.
    pub is_global: bool,
}

impl Agent {
    /// Returns all supported agents.
    pub fn all() -> &'static [Agent] {
        &[
            Agent::OpenCode,
            Agent::Claude,
            Agent::Codex,
            Agent::Cursor,
            Agent::Amp,
            Agent::KiloCode,
            Agent::RooCode,
            Agent::Goose,
            Agent::Gemini,
            Agent::Antigravity,
            Agent::Copilot,
            Agent::Clawdbot,
            Agent::Droid,
            Agent::Windsurf,
        ]
    }

    /// Returns the project-level skills directory for this agent.
    pub fn skills_dir(&self) -> &'static str {
        match self {
            Agent::OpenCode => ".opencode/skill",
            Agent::Claude => ".claude/skills",
            Agent::Codex => ".codex/skills",
            Agent::Cursor => ".cursor/skills",
            Agent::Amp => ".agents/skills",
            Agent::KiloCode => ".kilocode/skills",
            Agent::RooCode => ".roo/skills",
            Agent::Goose => ".goose/skills",
            Agent::Gemini => ".gemini/skills",
            Agent::Antigravity => ".agent/skills",
            Agent::Copilot => ".github/skills",
            Agent::Clawdbot => "skills",
            Agent::Droid => ".factory/skills",
            Agent::Windsurf => ".windsurf/skills",
        }
    }

    /// Returns the global (user-level) skills directory for this agent.
    pub fn global_skills_dir(&self) -> &'static str {
        match self {
            Agent::OpenCode => "~/.config/opencode/skill",
            Agent::Claude => "~/.claude/skills",
            Agent::Codex => "~/.codex/skills",
            Agent::Cursor => "~/.cursor/skills",
            Agent::Amp => "~/.config/agents/skills",
            Agent::KiloCode => "~/.kilocode/skills",
            Agent::RooCode => "~/.roo/skills",
            Agent::Goose => "~/.config/goose/skills",
            Agent::Gemini => "~/.gemini/skills",
            Agent::Antigravity => "~/.gemini/antigravity/skills",
            Agent::Copilot => "~/.copilot/skills",
            Agent::Clawdbot => "~/.clawdbot/skills",
            Agent::Droid => "~/.factory/skills",
            Agent::Windsurf => "~/.codeium/windsurf/skills",
        }
    }

    /// Returns the display name for this agent.
    pub fn display_name(&self) -> &'static str {
        match self {
            Agent::OpenCode => "OpenCode",
            Agent::Claude => "Claude Code",
            Agent::Codex => "Codex",
            Agent::Cursor => "Cursor",
            Agent::Amp => "Amp",
            Agent::KiloCode => "Kilo Code",
            Agent::RooCode => "Roo Code",
            Agent::Goose => "Goose",
            Agent::Gemini => "Gemini CLI",
            Agent::Antigravity => "Antigravity",
            Agent::Copilot => "GitHub Copilot",
            Agent::Clawdbot => "Clawdbot",
            Agent::Droid => "Droid",
            Agent::Windsurf => "Windsurf",
        }
    }

    /// Returns the CLI name for this agent (used in --agent flag).
    pub fn cli_name(&self) -> &'static str {
        match self {
            Agent::OpenCode => "opencode",
            Agent::Claude => "claude",
            Agent::Codex => "codex",
            Agent::Cursor => "cursor",
            Agent::Amp => "amp",
            Agent::KiloCode => "kilocode",
            Agent::RooCode => "roocode",
            Agent::Goose => "goose",
            Agent::Gemini => "gemini",
            Agent::Antigravity => "antigravity",
            Agent::Copilot => "copilot",
            Agent::Clawdbot => "clawdbot",
            Agent::Droid => "droid",
            Agent::Windsurf => "windsurf",
        }
    }

    /// Returns the features supported by this agent.
    pub fn features(&self) -> AgentFeatures {
        match self {
            Agent::Claude => AgentFeatures {
                context_fork: true,
                hooks: true,
                allowed_tools: true,
                scripts: true,
            },
            Agent::Cursor | Agent::Codex | Agent::OpenCode | Agent::Antigravity => AgentFeatures {
                context_fork: false,
                hooks: false,
                allowed_tools: true, // Partial support
                scripts: true,
            },
            _ => AgentFeatures {
                context_fork: false,
                hooks: false,
                allowed_tools: false,
                scripts: true,
            },
        }
    }

    /// Returns the detection path for this agent (config directory).
    pub fn detection_dir(&self) -> &'static str {
        match self {
            Agent::OpenCode => ".opencode",
            Agent::Claude => ".claude",
            Agent::Codex => ".codex",
            Agent::Cursor => ".cursor",
            Agent::Amp => ".agents",
            Agent::KiloCode => ".kilocode",
            Agent::RooCode => ".roo",
            Agent::Goose => ".goose",
            Agent::Gemini => ".gemini",
            Agent::Antigravity => ".agent",
            Agent::Copilot => ".github",
            Agent::Clawdbot => "skills", // Special case: no dot prefix
            Agent::Droid => ".factory",
            Agent::Windsurf => ".windsurf",
        }
    }

    /// Returns the global detection path for this agent.
    pub fn global_detection_dir(&self) -> &'static str {
        match self {
            Agent::OpenCode => "~/.config/opencode",
            Agent::Claude => "~/.claude",
            Agent::Codex => "~/.codex",
            Agent::Cursor => "~/.cursor",
            Agent::Amp => "~/.config/agents",
            Agent::KiloCode => "~/.kilocode",
            Agent::RooCode => "~/.roo",
            Agent::Goose => "~/.config/goose",
            Agent::Gemini => "~/.gemini",
            Agent::Antigravity => "~/.gemini/antigravity",
            Agent::Copilot => "~/.copilot",
            Agent::Clawdbot => "~/.clawdbot",
            Agent::Droid => "~/.factory",
            Agent::Windsurf => "~/.codeium/windsurf",
        }
    }

    /// Resolve the project-level skills directory to an absolute path.
    pub fn resolve_project_skills_dir(&self, project_root: &Path) -> PathBuf {
        project_root.join(self.skills_dir())
    }

    /// Resolve the global skills directory to an absolute path.
    pub fn resolve_global_skills_dir(&self) -> Option<PathBuf> {
        let dir = self.global_skills_dir();
        expand_tilde(dir)
    }

    /// Check if this agent is detected at the project level.
    pub fn is_detected_project(&self, project_root: &Path) -> bool {
        let detection_path = project_root.join(self.detection_dir());
        detection_path.exists()
    }

    /// Check if this agent is detected at the global level.
    pub fn is_detected_global(&self) -> bool {
        let dir = self.global_detection_dir();
        expand_tilde(dir).map(|p| p.exists()).unwrap_or(false)
    }

    /// Detect all agents installed at the project level.
    pub fn detect_project(project_root: &Path) -> Vec<Agent> {
        Agent::all()
            .iter()
            .filter(|a| a.is_detected_project(project_root))
            .copied()
            .collect()
    }

    /// Detect all agents installed at the global level.
    pub fn detect_global() -> Vec<Agent> {
        Agent::all()
            .iter()
            .filter(|a| a.is_detected_global())
            .copied()
            .collect()
    }

    /// Detect all agents (project and global).
    pub fn detect_all(project_root: &Path) -> Vec<DetectedAgent> {
        let mut detected = Vec::new();

        for agent in Agent::all() {
            // Check project level
            let project_path = agent.resolve_project_skills_dir(project_root);
            if agent.is_detected_project(project_root) {
                let skill_count = count_skills(&project_path);
                detected.push(DetectedAgent {
                    agent: *agent,
                    skills_path: project_path,
                    skill_count,
                    is_global: false,
                });
            }

            // Check global level
            if let Some(global_path) = agent.resolve_global_skills_dir() {
                if agent.is_detected_global() {
                    let skill_count = count_skills(&global_path);
                    detected.push(DetectedAgent {
                        agent: *agent,
                        skills_path: global_path,
                        skill_count,
                        is_global: true,
                    });
                }
            }
        }

        detected
    }
}

impl std::fmt::Display for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cli_name())
    }
}

/// Expand tilde in a path to the home directory.
pub fn expand_tilde(path: &str) -> Option<PathBuf> {
    if path.starts_with("~/") {
        dirs::home_dir().map(|home| home.join(&path[2..]))
    } else if path == "~" {
        dirs::home_dir()
    } else {
        Some(PathBuf::from(path))
    }
}

/// Count the number of skills in a directory.
fn count_skills(path: &Path) -> usize {
    if !path.exists() {
        return 0;
    }

    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir() && e.path().join("SKILL.md").exists())
                .count()
        })
        .unwrap_or(0)
}
