use crate::mneme;
use std::path::PathBuf;

/// Ecosystem health check
pub fn run_doctor() -> anyhow::Result<()> {
    println!("🔍 mneme-ai ecosystem doctor");
    println!("{}", "-".repeat(40));
    println!();

    // Check mneme binary
    check_mneme()?;

    // Check config
    check_config()?;

    // Check agents
    check_agents()?;

    println!();
    println!("{}", "-".repeat(40));
    println!("✅ Doctor check complete.");

    Ok(())
}

fn check_mneme() -> anyhow::Result<()> {
    print!("🧠 mneme binary... ");
    match mneme::find_mneme() {
        Some(path) => {
            let version = mneme::get_version(&path).unwrap_or_else(|| "unknown".to_string());
            println!("✅ found at {} (v{})", path.display(), version);
        }
        None => {
            println!("⚠  not found on PATH");
            println!("   Install with: cargo install mneme");
        }
    }
    Ok(())
}

fn check_config() -> anyhow::Result<()> {
    let config_path = config_dir().join("config.toml");
    print!("📋 config... ");
    if config_path.exists() {
        println!("✅ {}", config_path.display());
    } else {
        println!("⚠  not found (run 'mneme-ai init' to create)");
    }
    Ok(())
}

fn check_agents() -> anyhow::Result<()> {
    println!();
    println!("🤖 agent integrations:");
    for agent in crate::agents::SUPPORTED_AGENTS {
        if !agent.supported {
            continue;
        }
        let path: PathBuf = agent
            .config_path
            .replace(
                "{home}",
                &dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .to_string_lossy(),
            )
            .replace(
                "{config}",
                &dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .to_string_lossy(),
            )
            .into();

        let config_dir = path.parent().unwrap_or(&path);
        print!("  {}... ", agent.display);
        if path.exists() {
            println!("✅ configured");
        } else if config_dir.exists() {
            println!(
                "⚠  agent dir exists but needs setup (run 'mneme-ai install {}')",
                agent.name
            );
        } else {
            println!("  not detected");
        }
    }
    Ok(())
}

fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("mneme-ai")
}
