use crate::agents::{self, ConfigFormat};
use crate::opencode;
use std::path::PathBuf;

const CONFIG_DIR: &str = "mneme-ai";
const CONFIG_FILE: &str = "config.toml";

/// Get config directory (XDG_CONFIG_HOME/mneme-ai or ~/.config/mneme-ai)
fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join(CONFIG_DIR)
}

/// Get home directory
fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Resolve {home} and {config} placeholders in paths
fn resolve_path(template: &str) -> PathBuf {
    template
        .replace("{home}", &home_dir().to_string_lossy())
        .replace(
            "{config}",
            &dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .to_string_lossy(),
        )
        .into()
}

/// Initialize default config for mneme-ai
pub fn init_config() -> anyhow::Result<()> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let config_path = dir.join(CONFIG_FILE);

    if config_path.exists() {
        println!("✓ Config already exists: {}", config_path.display());
        return Ok(());
    }

    let config = format!(
        r#"# mneme-ai configuration
# Created: {}

[defaults]
provider = "opencode"
mneme_bin = "mneme"
auto_sync = true

[agents]
# List of agents to auto-configure on install
enabled = ["opencode", "claude-code", "cursor", "windsurf"]
"#,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")
    );

    std::fs::write(&config_path, &config)?;
    println!("✓ Created config: {}", config_path.display());
    Ok(())
}

/// Install mneme integration for a specific agent
pub fn install_agent(agent_name: &str) -> anyhow::Result<()> {
    let agent = agents::find_agent(agent_name).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown agent: {}. Use 'mneme-ai list-agents' to see supported agents.",
            agent_name
        )
    })?;

    if !agent.supported {
        println!(
            "⚠  Agent '{}' is not yet fully supported (coming soon).",
            agent_name
        );
        return Ok(());
    }

    println!("Installing mneme for {}...", agent.display);
    setup_mcp_config(agent)?;

    // For OpenCode: also generate agents and prompt files
    if agent_name == "opencode" {
        println!("  Generating mneme-orchestrator agents...");
        opencode::write_prompt_files()?;
        let config = opencode::OpenCodeConfig::default();
        let agents_json = opencode::generate_agents(&config);
        opencode::write_to_opencode(&agents_json)?;
    }

    println!("✓ {} configured with mneme.", agent.display);
    Ok(())
}

/// Setup agent as alias for install
pub fn setup_agent(agent_name: &str) -> anyhow::Result<()> {
    install_agent(agent_name)
}

/// Write MCP config for a specific agent
fn setup_mcp_config(agent: &agents::AgentInfo) -> anyhow::Result<()> {
    let config_path = resolve_path(agent.config_path);

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Read existing config or create empty
    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !config.is_object() {
        config = serde_json::json!({});
    }

    match agent.config_format {
        ConfigFormat::McpServers | ConfigFormat::ClaudeCode => {
            ensure_object(&mut config, "mcpServers");
            config["mcpServers"]["mneme"] = serde_json::json!({
                "command": "mneme",
                "args": ["mcp"]
            });
        }
        ConfigFormat::McpServersUnderscore => {
            ensure_object(&mut config, "mcp_servers");
            config["mcp_servers"]["mneme"] = serde_json::json!({
                "command": "mneme",
                "args": ["mcp"]
            });
        }
        ConfigFormat::Servers => {
            ensure_object(&mut config, "servers");
            config["servers"]["mneme"] = serde_json::json!({
                "type": "stdio",
                "command": "mneme",
                "args": ["mcp"]
            });
        }
        ConfigFormat::Opencode => {
            if config.get("mcp").map_or(true, |v| !v.is_object()) {
                config["mcp"] = serde_json::json!({});
            }
            config["mcp"]["mneme"] = serde_json::json!({
                "command": ["mneme", "mcp"],
                "type": "local",
                "enabled": true
            });
        }
        ConfigFormat::Continue => {
            let servers = config
                .get("models")
                .and_then(|m| m.as_array())
                .cloned()
                .unwrap_or_default();

            let mut new_servers = servers;
            new_servers.push(serde_json::json!({
                "title": "Mneme MCP",
                "provider": "mcp",
                "model": "mneme",
                "apiBase": "",
                "server": {
                    "command": "mneme",
                    "args": ["mcp"]
                }
            }));

            config["models"] = serde_json::json!(new_servers);
        }
    }

    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&config_path, content)?;
    println!("  ✓ Config written: {}", config_path.display());
    Ok(())
}

fn ensure_object(config: &mut serde_json::Value, key: &str) {
    if config.get(key).map_or(true, |v| !v.is_object()) {
        config[key] = serde_json::json!({});
    }
}
