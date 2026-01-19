//! Supported AI coding agents and their skill directories.

use serde::Deserialize;

/// Supported AI coding agents.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
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
}
