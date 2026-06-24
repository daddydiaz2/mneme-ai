use clap::Parser;
use mneme_ai::agents;
use mneme_ai::cli::{self, Commands};
use mneme_ai::doctor;
use mneme_ai::install;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = cli::Cli::parse();

    match cli.command {
        Commands::Init => install::init_config()?,
        Commands::Install { agent } => install::install_agent(&agent)?,
        Commands::Setup { agent } => install::setup_agent(&agent)?,
        Commands::Doctor => doctor::run_doctor()?,
        Commands::Sync {
            profile,
            profile_phase,
        } => {
            let _ = (profile, profile_phase);
            tracing::info!("Sync not yet implemented in v0.1.0");
        }
        Commands::ListAgents => agents::list_agents(),
        Commands::Version => {
            println!("mneme-ai v{}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
