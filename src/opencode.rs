/// OpenCode agent configuration generator.
/// Creates the mneme-orchestrator agent with SDD phases, review agents,
/// judgment day agents, and skill integration — similar to gentle-ai.
use std::collections::HashMap;
use std::path::PathBuf;

/// SDD phases that get sub-agents
pub const SDD_PHASES: &[&str] = &[
    "explore", "propose", "spec", "design", "tasks", "apply", "verify", "archive", "init",
    "onboard",
];

/// Review agent types
pub const REVIEW_AGENTS: &[&str] = &[
    "review-readability",
    "review-reliability",
    "review-resilience",
    "review-risk",
];

/// Dual-review agents (renamed from 'jd' to avoid gentle-ai naming)
pub const JUDGMENT_AGENTS: &[&str] = &["mneme-judge-alpha", "mneme-judge-beta", "mneme-fix"];

/// Model assignment for a specific phase
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub provider: String,
    pub model: String,
    pub variant: String,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            provider: "opencode-go".to_string(),
            model: "deepseek-v4-flash".to_string(),
            variant: "high".to_string(),
        }
    }
}

/// Full agent configuration for OpenCode
pub struct OpenCodeConfig {
    pub orchestrator_model: ModelConfig,
    pub sdd_models: HashMap<String, ModelConfig>,
    pub review_model: ModelConfig,
    pub judgment_model: ModelConfig,
}

impl Default for OpenCodeConfig {
    fn default() -> Self {
        let default_model = ModelConfig::default();
        Self {
            orchestrator_model: default_model.clone(),
            sdd_models: HashMap::new(),
            review_model: default_model.clone(),
            judgment_model: default_model,
        }
    }
}

/// Generate all agents for OpenCode
pub fn generate_agents(config: &OpenCodeConfig) -> serde_json::Value {
    let mut agents = serde_json::json!({});

    // 1. mneme-orchestrator (primary, delegator)
    let mut task_permissions = serde_json::Map::new();
    // SDD phases
    for phase in SDD_PHASES {
        task_permissions.insert(
            format!("sdd-{}", phase),
            serde_json::Value::String("allow".to_string()),
        );
    }
    // Review agents
    for agent in REVIEW_AGENTS {
        task_permissions.insert(
            agent.to_string(),
            serde_json::Value::String("allow".to_string()),
        );
    }
    // Judgment agents
    for agent in JUDGMENT_AGENTS {
        task_permissions.insert(
            agent.to_string(),
            serde_json::Value::String("allow".to_string()),
        );
    }

    agents["mneme-orchestrator"] = serde_json::json!({
        "description": "mneme-ai SDD orchestrator — coordinates sub-agents, delegates work",
        "mode": "primary",
        "model": format!("{}/{}", config.orchestrator_model.provider, config.orchestrator_model.model),
        "permission": {
            "task": task_permissions
        },
        "prompt": "{file:~/.config/opencode/prompts/mneme/mneme-orchestrator.md}",
        "tools": {
            "bash": true,
            "edit": true,
            "question": true,
            "read": true,
            "task": true,
            "write": true
        },
        "variant": config.orchestrator_model.variant
    });

    // 2. SDD phase sub-agents
    for phase in SDD_PHASES {
        let model = config
            .sdd_models
            .get(*phase)
            .unwrap_or(&config.orchestrator_model);
        let agent_key = format!("sdd-{}", phase);

        agents[&agent_key] = serde_json::json!({
            "description": format!("SDD {} phase", phase),
            "hidden": true,
            "mode": "subagent",
            "model": format!("{}/{}", model.provider, model.model),
            "prompt": format!("{{file:~/.config/opencode/prompts/sdd/sdd-{}.md}}", phase),
            "tools": {
                "bash": true,
                "edit": true,
                "read": true,
                "write": true
            },
            "variant": model.variant
        });
    }

    // 3. Review agents
    for agent in REVIEW_AGENTS {
        agents[agent] = serde_json::json!({
            "description": format!("{} — read-only code review", agent),
            "hidden": true,
            "mode": "subagent",
            "model": format!("{}/{}", config.review_model.provider, config.review_model.model),
            "prompt": format!("{{file:~/.config/opencode/prompts/review/{}.md}}", agent),
            "tools": {
                "bash": true,
                "read": true
            }
        });
    }

    // 4. Judgment day agents
    for agent in JUDGMENT_AGENTS {
        let is_judge = agent.contains("judge");
        let tools = if is_judge {
            serde_json::json!({"bash": true, "read": true})
        } else {
            serde_json::json!({"bash": true, "edit": true, "read": true, "write": true})
        };

        agents[agent] = serde_json::json!({
            "description": format!("{} — judgment day protocol", agent),
            "hidden": true,
            "mode": "subagent",
            "model": format!("{}/{}", config.judgment_model.provider, config.judgment_model.model),
            "prompt": format!("{{file:~/.config/opencode/prompts/judgment-day/{}.md}}", agent),
            "tools": tools,
            "variant": "max"
        });
    }

    agents
}

/// Write agent config to opencode.json
pub fn write_to_opencode(agents: &serde_json::Value) -> anyhow::Result<()> {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let config_path = config_dir.join("opencode").join("opencode.json");

    let mut config: serde_json::Value = if config_path.exists() {
        let content = std::fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Merge agents into existing config
    if !config.is_object() {
        config = serde_json::json!({});
    }
    if config.get("agent").map_or(true, |v| !v.is_object()) {
        config["agent"] = serde_json::json!({});
    }

    // Add/overwrite each agent
    if let Some(obj) = agents.as_object() {
        for (key, value) in obj {
            config["agent"][key] = value.clone();
        }
    }

    let content = serde_json::to_string_pretty(&config)?;
    std::fs::write(&config_path, content)?;

    println!("✓ Agents written to {}", config_path.display());
    Ok(())
}

/// Write prompt files for all agents
pub fn write_prompt_files() -> anyhow::Result<()> {
    let prompts_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opencode")
        .join("prompts");

    // SDD prompts
    let sdd_dir = prompts_dir.join("sdd");
    std::fs::create_dir_all(&sdd_dir)?;

    for phase in SDD_PHASES {
        let path = sdd_dir.join(format!("sdd-{}.md", phase));
        if !path.exists() {
            let content = format!(
                "---\nname: sdd-{phase}\ndescription: \"SDD {phase} phase — delegated by mneme-orchestrator\"\ndisable-model-invocation: true\nuser-invocable: false\n---\n\n# SDD {phase} phase\n\nExecute this phase for the mneme-ai SDD workflow.\n"
            );
            std::fs::write(&path, content)?;
        }
    }

    // Orchestrator prompt
    let mneme_dir = prompts_dir.join("mneme");
    std::fs::create_dir_all(&mneme_dir)?;
    let orch_path = mneme_dir.join("mneme-orchestrator.md");
    if !orch_path.exists() {
        std::fs::write(&orch_path, include_str!("../prompts/mneme-orchestrator.md"))?;
    }

    // Review prompts
    let review_dir = prompts_dir.join("review");
    std::fs::create_dir_all(&review_dir)?;
    for agent in REVIEW_AGENTS {
        let path = review_dir.join(format!("{}.md", agent));
        if !path.exists() {
            std::fs::write(&path, format!("# {agent}\n\nRead-only review agent.\n"))?;
        }
    }

    // Judgment day prompts
    let jd_dir = prompts_dir.join("judgment-day");
    std::fs::create_dir_all(&jd_dir)?;
    for agent in JUDGMENT_AGENTS {
        let path = jd_dir.join(format!("{}.md", agent));
        if !path.exists() {
            std::fs::write(
                &path,
                format!("# {agent}\n\nJudgment day protocol agent.\n"),
            )?;
        }
    }

    Ok(())
}

/// Customize OpenCode with mneme branding: update AGENTS.md, add plugins, set author
pub fn customize_opencode() -> anyhow::Result<()> {
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    let opencode_dir = config_dir.join("opencode");

    // 1. Update AGENTS.md with mneme branding
    let agents_path = opencode_dir.join("AGENTS.md");
    let content = if agents_path.exists() {
        std::fs::read_to_string(&agents_path)?
    } else {
        String::new()
    };

    if !content.contains("mneme tools") {
        let branding = "<!-- mneme-ai:branding -->\n## mneme tools\n\n**Powered by mneme-ai** — Ecosystem configurator for AI coding agents.\nAuthor: Daniel Diaz\n\n";
        std::fs::write(&agents_path, branding.to_string() + &content)?;
    }

    // 2. Update tui.json with mneme plugins
    let tui_path = opencode_dir.join("tui.json");
    let mut tui_config: serde_json::Value = if tui_path.exists() {
        let c = std::fs::read_to_string(&tui_path)?;
        serde_json::from_str(&c).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !tui_config.is_object() {
        tui_config = serde_json::json!({});
    }
    if tui_config.get("plugin").map_or(true, |p| !p.is_array()) {
        tui_config["plugin"] = serde_json::json!([]);
    }

    if let Some(plugins) = tui_config["plugin"].as_array_mut() {
        if !plugins
            .iter()
            .any(|p| p.as_str() == Some("opencode-subagent-statusline"))
        {
            plugins.push(serde_json::json!("opencode-subagent-statusline"));
        }
    }

    std::fs::write(&tui_path, serde_json::to_string_pretty(&tui_config)?)?;

    // 3. Update agent descriptions in opencode.json with mneme-ai branding
    let opencode_json_path = opencode_dir.join("opencode.json");
    if opencode_json_path.exists() {
        let mut oc_config: serde_json::Value = {
            let c = std::fs::read_to_string(&opencode_json_path)?;
            serde_json::from_str(&c).unwrap_or(serde_json::json!({}))
        };

        if let Some(agents) = oc_config.get_mut("agent").and_then(|a| a.as_object_mut()) {
            for (_key, agent) in agents.iter_mut() {
                if let Some(desc) = agent.get_mut("description") {
                    if let Some(d) = desc.as_str() {
                        if !d.contains("mneme") && !d.contains("gentle") {
                            *desc = serde_json::json!(format!("{} — mneme-ai", d));
                        }
                    }
                }
            }
            std::fs::write(
                &opencode_json_path,
                serde_json::to_string_pretty(&oc_config)?,
            )?;
        }
    }

    println!("✓ OpenCode customized with mneme tools branding");
    Ok(())
}
