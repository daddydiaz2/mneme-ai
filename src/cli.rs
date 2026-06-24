use clap::{Parser, Subcommand};

/// mneme-ai — Ecosystem configurator for AI coding agents.
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

    /// Launch interactive TUI.
    Tui,

    /// Manage SDD profiles.
    Profile {
        #[command(subcommand)]
        profile_cmd: ProfileCommands,
    },

    /// List all supported agents.
    ListAgents,

    /// Show version.
    Version,
}

#[derive(Subcommand)]
pub enum ProfileCommands {
    /// List all SDD profiles.
    List,

    /// Create a new SDD profile.
    Create {
        /// Profile name.
        name: String,
        /// Provider/model string (e.g. opencode/default).
        #[arg(short = 'm')]
        model: Option<String>,
    },

    /// Show profile details.
    Show {
        /// Profile name.
        name: String,
    },

    /// Delete a profile.
    Delete {
        /// Profile name.
        name: String,
    },
}
