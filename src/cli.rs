use clap::{Parser, Subcommand};

/// mneme-ai — Ecosystem configurator for AI coding agents.
///
/// Supercharges your AI agent with mneme persistent memory, SDD workflows,
/// curated skills, and MCP tools — regardless of which agent you use.
#[derive(Parser)]
#[command(name = "mneme-ai", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize mneme-ai config in the current project.
    Init,

    /// Install mneme integration for a specific agent.
    Install {
        /// Agent name: opencode, claude-code, cursor, windsurf, etc.
        agent: String,
    },

    /// Setup/configure a specific agent with mneme.
    Setup {
        /// Agent name.
        agent: String,
    },

    /// Run ecosystem health checks.
    Doctor,

    /// Sync configuration profiles.
    Sync {
        /// Profile name (e.g. "cheap", "premium").
        #[arg(long)]
        profile: Option<String>,

        /// Per-phase model override (e.g. "cheap:sdd-design:model-x").
        #[arg(long)]
        profile_phase: Option<String>,
    },

    /// List all supported agents.
    ListAgents,

    /// Show version.
    Version,
}
