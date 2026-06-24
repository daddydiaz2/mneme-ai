mod cli;
mod install;
mod doctor;
mod agents;
mod mneme;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::Init => install::init_config()?,
        cli::Commands::Install { agent } => install::install_agent(&agent)?,
        cli::Commands::Setup { agent } => install::setup_agent(&agent)?,
        cli::Commands::Doctor => doctor::run_doctor()?,
        cli::Commands::Sync { profile, profile_phase } => {
            // TODO: full sync implementation
            tracing::info!("Sync not yet implemented in v0.1.0");
        }
        cli::Commands::ListAgents => agents::list_agents(),
        cli::Commands::Version => {
            println!("mneme-ai v{}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}
