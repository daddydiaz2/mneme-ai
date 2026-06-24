/// SDD Profile management — named model configurations for different phases.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// SDD phases that can have model assignments
pub const SDD_PHASES: &[&str] = &[
    "sdd-explore",
    "sdd-propose",
    "sdd-spec",
    "sdd-design",
    "sdd-tasks",
    "sdd-apply",
    "sdd-verify",
    "sdd-archive",
];

/// A model assignment for a specific SDD phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelAssignment {
    pub provider: String,
    pub model: String,
    pub reasoning_effort: Option<String>,
}

/// An SDD profile — named set of model assignments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SddProfile {
    pub name: String,
    pub orchestrator: ModelAssignment,
    pub phases: HashMap<String, ModelAssignment>,
}

/// Profile store manages saving/loading profiles
pub struct ProfileStore {
    profiles_dir: PathBuf,
}

impl ProfileStore {
    pub fn new() -> Self {
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            profiles_dir: base.join("mneme-ai").join("profiles"),
        }
    }

    pub fn init(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(&self.profiles_dir)?;
        // Create default profile if none exist
        if !self.list()?.iter().any(|p| p.name == "default") {
            let default = SddProfile {
                name: "default".to_string(),
                orchestrator: ModelAssignment {
                    provider: "opencode".to_string(),
                    model: "default".to_string(),
                    reasoning_effort: None,
                },
                phases: HashMap::new(),
            };
            self.save(&default)?;
        }
        Ok(())
    }

    pub fn list(&self) -> anyhow::Result<Vec<SddProfile>> {
        let mut profiles = Vec::new();
        if !self.profiles_dir.exists() {
            return Ok(profiles);
        }
        for entry in std::fs::read_dir(&self.profiles_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "toml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(profile) = toml::from_str::<SddProfile>(&content) {
                        profiles.push(profile);
                    }
                }
            }
        }
        profiles.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(profiles)
    }

    pub fn get(&self, name: &str) -> anyhow::Result<Option<SddProfile>> {
        let path = self.profiles_dir.join(format!("{}.toml", name));
        if !path.exists() {
            return Ok(None);
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(Some(toml::from_str(&content)?))
    }

    pub fn save(&self, profile: &SddProfile) -> anyhow::Result<()> {
        let path = self.profiles_dir.join(format!("{}.toml", profile.name));
        let content = toml::to_string_pretty(profile)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn delete(&self, name: &str) -> anyhow::Result<()> {
        let path = self.profiles_dir.join(format!("{}.toml", name));
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Export profile as OpenCode agent config (JSON snippet for opencode.json)
    pub fn to_opencode_agents(&self, profile: &SddProfile) -> serde_json::Value {
        let mut agents = serde_json::json!({});

        // Orchestrator agent
        agents[format!("sdd-orchestrator-{}", profile.name)] = serde_json::json!({
            "description": format!("SDD orchestrator ({})", profile.name),
            "mode": "primary",
            "model": format!("{}/{}", profile.orchestrator.provider, profile.orchestrator.model),
            "prompt": "{file:~/.config/opencode/prompts/sdd/sdd-orchestrator.md}",
            "tools": {
                "bash": true,
                "edit": true,
                "question": true,
                "read": true,
                "task": true,
                "write": true
            }
        });

        // Phase sub-agents
        for phase_name in SDD_PHASES {
            let assignment = profile
                .phases
                .get(*phase_name)
                .unwrap_or(&profile.orchestrator);
            let agent_key = format!("{}-{}", phase_name, profile.name);

            agents[&agent_key] = serde_json::json!({
                "description": format!("{} ({})", phase_name, profile.name),
                "hidden": true,
                "mode": "subagent",
                "model": format!("{}/{}", assignment.provider, assignment.model),
                "prompt": format!("{{file:~/.config/opencode/prompts/sdd/{}.md}}", phase_name),
                "tools": {
                    "bash": true,
                    "edit": true,
                    "read": true,
                    "write": true
                },
                "variant": "high"
            });
        }

        agents
    }
}
