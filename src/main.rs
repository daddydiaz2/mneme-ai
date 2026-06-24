use clap::Parser;
use mneme_ai::agents;
use mneme_ai::cli::{self, Commands, ProfileCommands};
use mneme_ai::doctor;
use mneme_ai::install;
use mneme_ai::profile::ProfileStore;

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
        Commands::Tui => mneme_ai::tui::run_tui()?,
        Commands::Profile { profile_cmd } => handle_profile(profile_cmd)?,
        Commands::ListAgents => agents::list_agents(),
        Commands::Version => {
            println!("mneme-ai v{}", env!("CARGO_PKG_VERSION"));
        }
    }

    Ok(())
}

fn handle_profile(cmd: ProfileCommands) -> anyhow::Result<()> {
    let store = ProfileStore::new();
    store.init()?;

    match cmd {
        ProfileCommands::List => {
            let profiles = store.list()?;
            if profiles.is_empty() {
                println!("No profiles configured.");
                return Ok(());
            }
            println!("{:<20} {:<20} {:<15}", "NAME", "ORCHESTRATOR", "PHASES");
            println!("{}", "-".repeat(60));
            for p in &profiles {
                println!(
                    "{:<20} {:<20} {:<15}",
                    p.name,
                    format!("{}/{}", p.orchestrator.provider, p.orchestrator.model),
                    p.phases.len()
                );
            }
        }
        ProfileCommands::Create { name, model } => {
            let (provider, mdl) = model
                .as_deref()
                .and_then(|m| m.split_once('/'))
                .unwrap_or(("opencode", "default"));

            let profile = mneme_ai::profile::SddProfile {
                name: name.to_lowercase().replace(' ', "-"),
                orchestrator: mneme_ai::profile::ModelAssignment {
                    provider: provider.to_string(),
                    model: mdl.to_string(),
                    reasoning_effort: None,
                },
                phases: std::collections::HashMap::new(),
            };
            store.save(&profile)?;
            println!("✓ Profile '{}' created.", profile.name);
        }
        ProfileCommands::Show { name } => match store.get(&name)? {
            Some(profile) => {
                println!("Profile: {}", profile.name);
                println!(
                    "Orchestrator: {}/{}",
                    profile.orchestrator.provider, profile.orchestrator.model
                );
                println!("Phases configured: {}", profile.phases.len());
                if !profile.phases.is_empty() {
                    println!("\nPhase assignments:");
                    for (phase, assignment) in &profile.phases {
                        println!("  {} → {}/{}", phase, assignment.provider, assignment.model);
                    }
                }
            }
            None => println!("Profile '{}' not found.", name),
        },
        ProfileCommands::Delete { name } => {
            if name == "default" {
                println!("Cannot delete the default profile.");
                return Ok(());
            }
            store.delete(&name)?;
            println!("✓ Profile '{}' deleted.", name);
        }
    }

    Ok(())
}
