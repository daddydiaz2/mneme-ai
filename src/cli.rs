use clap::{Parser, Subcommand};

/// mneme-ai v0.4.0 — Ecosystem configurator for AI coding agents.
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
    /// Initialize mneme-ai config.
    Init,

    /// Install mneme integration for one or all agents.
    Install {
        /// Agent name: opencode, claude-code, cursor, windsurf, or "all".
        agent: String,
        /// Also create the mneme-orchestrator agent in OpenCode.
        #[arg(long)]
        with_agents: bool,
        /// Skip MCP config (only create agents).
        #[arg(long)]
        agents_only: bool,
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

    /// Manage SDD profiles (model configurations per phase).
    Profile {
        #[command(subcommand)]
        command: ProfileCommands,
    },

    /// Sync profiles and configurations.
    Sync {
        /// Sync to opencode.json (write agents).
        #[arg(long)]
        opencode: bool,

        /// Create a profile during sync: name:provider/model.
        #[arg(long)]
        profile: Option<String>,

        /// Override a specific phase: name:phase:provider/model.
        #[arg(long)]
        profile_phase: Option<String>,
    },

    /// Backup or restore configurations.
    Backup {
        #[command(subcommand)]
        command: BackupCommands,
    },

    /// Browse mneme memories (requires mneme MCP).
    Memory {
        #[command(subcommand)]
        command: MemoryCommands,
    },

    /// Manage skills registry.
    Skills {
        #[command(subcommand)]
        command: SkillsCommands,
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
        /// Profile name (lowercase, hyphens allowed).
        name: String,
        /// Provider/model (e.g. opencode/default, anthropic/claude-sonnet-4).
        #[arg(short, long)]
        model: Option<String>,
        /// Phase-specific override: phase=provider/model
        #[arg(short, long)]
        phase: Vec<String>,
    },

    /// Show profile details.
    Show { name: String },

    /// Edit a profile's model.
    Edit {
        name: String,
        /// New orchestrator model.
        #[arg(short, long)]
        model: Option<String>,
        /// Phase to update.
        #[arg(short, long)]
        phase: Option<String>,
    },

    /// Delete a profile.
    Delete {
        name: String,
        /// Skip confirmation.
        #[arg(long)]
        force: bool,
    },

    /// Export profile as opencode.json snippet.
    Export { name: String },

    /// Duplicate a profile.
    Clone { source: String, dest: String },
}

#[derive(Subcommand)]
pub enum BackupCommands {
    /// Create a backup of all configurations.
    Create,

    /// List available backups.
    List,

    /// Restore from a backup.
    Restore {
        /// Backup ID or name.
        name: String,
    },
}

#[derive(Subcommand)]
pub enum MemoryCommands {
    /// Search mneme memories.
    Search {
        query: String,
        /// Project filter.
        #[arg(short, long)]
        project: Option<String>,
    },
    /// List recent memories.
    List {
        /// Project filter.
        #[arg(short, long)]
        project: Option<String>,
        #[arg(short, long, default_value = "10")]
        limit: u32,
    },
    /// Show memory statistics.
    Stats {
        /// Project filter.
        #[arg(short, long)]
        project: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum SkillsCommands {
    /// List installed skills.
    List,
    /// Refresh skill registry.
    Refresh,
}
