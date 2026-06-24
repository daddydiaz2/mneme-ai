use clap::Parser;
use mneme_ai::agents;
use mneme_ai::cli::{
    self, BackupCommands, Commands, MemoryCommands, ProfileCommands, SkillsCommands,
};
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
        Commands::Install {
            agent,
            with_agents,
            agents_only,
        } => {
            if agent == "all" {
                for a in agents::SUPPORTED_AGENTS {
                    if a.supported {
                        install::install_agent(a.name)?;
                    }
                }
            } else {
                install::install_agent(&agent)?;
            }
            if with_agents {
                install::install_opencode_agents()?;
            }
        }
        Commands::Setup { agent } => install::setup_agent(&agent)?,
        Commands::Doctor => doctor::run_doctor()?,
        Commands::Tui => mneme_ai::tui::run_tui()?,
        Commands::Profile { command } => handle_profile(command)?,
        Commands::Sync {
            opencode,
            profile,
            profile_phase,
        } => {
            if let Some(p) = profile {
                // Parse name:provider/model
                if let Some((name, rest)) = p.split_once(':') {
                    let (provider, model) = rest.split_once('/').unwrap_or((rest, "default"));
                    let store = ProfileStore::new();
                    store.init()?;
                    let profile_obj = mneme_ai::profile::SddProfile {
                        name: name.to_lowercase(),
                        orchestrator: mneme_ai::profile::ModelAssignment {
                            provider: provider.to_string(),
                            model: model.to_string(),
                            reasoning_effort: None,
                        },
                        phases: std::collections::HashMap::new(),
                    };
                    store.save(&profile_obj)?;
                    println!("✓ Profile '{}' created: {}/{}", name, provider, model);
                }
            }
            if let Some(_pp) = profile_phase {
                println!("ℹ Phase overrides not yet implemented in CLI.");
            }
            if opencode {
                install::sync_to_opencode()?;
            }
        }
        Commands::Backup { command } => match command {
            BackupCommands::Create => {
                let backup_dir = mneme_ai::config_dir().join("backups");
                std::fs::create_dir_all(&backup_dir)?;
                let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
                let backup_file = backup_dir.join(format!("mneme-ai-backup-{}.tar.gz", ts));
                // Simple backup: copy config dirs
                let configs = mneme_ai::config_dir();
                let output = std::fs::File::create(&backup_file)?;
                let mut tar = tar::Builder::new(output);
                tar.append_dir_all("mneme-ai", &configs)?;
                tar.finish()?;
                println!("✓ Backup created: {}", backup_file.display());
            }
            BackupCommands::List => {
                let backup_dir = mneme_ai::config_dir().join("backups");
                if backup_dir.exists() {
                    for entry in std::fs::read_dir(&backup_dir)? {
                        let e = entry?;
                        println!(
                            "  {} ({} bytes)",
                            e.file_name().to_string_lossy(),
                            e.metadata()?.len()
                        );
                    }
                } else {
                    println!("No backups found.");
                }
            }
            BackupCommands::Restore { name } => {
                println!("ℹ Restore not yet implemented. File: {}", name);
            }
        },
        Commands::Memory { command } => match command {
            MemoryCommands::Search { query, project } => {
                mneme_ai::mneme::search(&query, project.as_deref())?;
            }
            MemoryCommands::List { project, limit } => {
                let project = project.unwrap_or_else(|| "default".to_string());
                println!("ℹ List memories via: mneme list --project {}", project);
            }
            MemoryCommands::Stats { project } => {
                let project = project.unwrap_or_else(|| "default".to_string());
                println!("ℹ View stats via: mneme stats --project {}", project);
            }
        },
        Commands::Skills { command } => match command {
            SkillsCommands::List => {
                println!("Installed skills:");
                let skills_dir = dirs::config_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("opencode")
                    .join("skills");
                if skills_dir.exists() {
                    for entry in std::fs::read_dir(&skills_dir)? {
                        let e = entry?;
                        if e.file_type()?.is_dir() {
                            println!("  - {}", e.file_name().to_string_lossy());
                        }
                    }
                } else {
                    println!("  No skills directory found.");
                }
            }
            SkillsCommands::Refresh => {
                println!("ℹ Skill refresh requires the skill-registry tool.");
            }
        },
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
                println!(
                    "Create one: mneme-ai profile create --name cheap --model opencode/default"
                );
                return Ok(());
            }
            println!(
                "{:<20} {:<25} {:<10} {:<15}",
                "NAME", "ORCHESTRATOR", "PHASES", "MODEL"
            );
            println!("{}", "-".repeat(70));
            for p in &profiles {
                let phases = if p.phases.is_empty() {
                    "default".to_string()
                } else {
                    format!("{}", p.phases.len())
                };
                println!(
                    "{:<20} {:<25} {:<10} {:<15}",
                    p.name,
                    format!("{}/{}", p.orchestrator.provider, p.orchestrator.model),
                    phases,
                    &p.orchestrator.model
                );
            }
        }
        ProfileCommands::Create { name, model, phase } => {
            let (provider, mdl) = model
                .as_deref()
                .and_then(|m| m.split_once('/'))
                .unwrap_or(("opencode", "default"));

            let mut phases = std::collections::HashMap::new();
            for p in &phase {
                if let Some((phase_name, model_str)) = p.split_once('=') {
                    if let Some((prov, mdl)) = model_str.split_once('/') {
                        phases.insert(
                            phase_name.to_string(),
                            mneme_ai::profile::ModelAssignment {
                                provider: prov.to_string(),
                                model: mdl.to_string(),
                                reasoning_effort: None,
                            },
                        );
                    }
                }
            }

            let profile = mneme_ai::profile::SddProfile {
                name: name.to_lowercase().replace(' ', "-"),
                orchestrator: mneme_ai::profile::ModelAssignment {
                    provider: provider.to_string(),
                    model: mdl.to_string(),
                    reasoning_effort: None,
                },
                phases,
            };
            store.save(&profile)?;
            println!("✓ Profile '{}' created.", profile.name);
            if !phase.is_empty() {
                println!("  Phase overrides: {}", phase.len());
            }
        }
        ProfileCommands::Show { name } => match store.get(&name)? {
            Some(profile) => {
                println!("Profile: {}", profile.name);
                println!(
                    "Orchestrator: {}/{}",
                    profile.orchestrator.provider, profile.orchestrator.model
                );
                if profile.phases.is_empty() {
                    println!("Phases: all use orchestrator model");
                } else {
                    println!("\nPhase assignments:");
                    for (phase, assignment) in &profile.phases {
                        let known = mneme_ai::profile::SDD_PHASES.contains(&phase.as_str());
                        let marker = if known { "●" } else { "○" };
                        println!(
                            "  {} {:<20} → {}/{}",
                            marker, phase, assignment.provider, assignment.model
                        );
                    }
                }
            }
            None => println!("Profile '{}' not found.", name),
        },
        ProfileCommands::Edit { name, model, phase } => {
            let mut profile = store
                .get(&name)?
                .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found.", name))?;
            if let Some(m) = model {
                if let Some((provider, model)) = m.split_once('/') {
                    profile.orchestrator.provider = provider.to_string();
                    profile.orchestrator.model = model.to_string();
                }
            }
            if let Some(phase_name) = phase {
                // Remove phase override (revert to orchestrator model)
                profile.phases.remove(&phase_name);
                println!("  Removed phase override for '{}'", phase_name);
            }
            store.save(&profile)?;
            println!("✓ Profile '{}' updated.", name);
        }
        ProfileCommands::Delete { name, force } => {
            if name == "default" && !force {
                println!("Cannot delete default profile. Use --force to override.");
                return Ok(());
            }
            store.delete(&name)?;
            println!("✓ Profile '{}' deleted.", name);
        }
        ProfileCommands::Export { name } => {
            let profile = store
                .get(&name)?
                .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found.", name))?;
            let config = mneme_ai::opencode::OpenCodeConfig::default();
            let agents_json = mneme_ai::opencode::generate_agents(&config);
            println!("{}", serde_json::to_string_pretty(&agents_json)?);
        }
        ProfileCommands::Clone { source, dest } => {
            let profile = store
                .get(&source)?
                .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found.", source))?;
            let mut cloned = profile.clone();
            cloned.name = dest.to_lowercase().replace(' ', "-");
            store.save(&cloned)?;
            println!("✓ Profile '{}' cloned from '{}'.", cloned.name, source);
        }
    }

    Ok(())
}
