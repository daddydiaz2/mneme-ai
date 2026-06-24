/// Skills management — scan, install, list, and refresh skills.
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub path: PathBuf,
    pub skill_type: SkillType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SkillType {
    Sdd,
    Review,
    Judgment,
    Workflow,
    Other,
}

/// Get the skills directory
pub fn skills_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("opencode")
        .join("skills")
}

/// Get the shared skills dir
pub fn shared_dir() -> PathBuf {
    skills_dir().join("_shared")
}

/// Scan installed skills
pub fn scan_skills() -> Vec<Skill> {
    let dir = skills_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut skills = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if name == "_shared" {
                    continue;
                }

                let skill_file = path.join("SKILL.md");
                let description = if skill_file.exists() {
                    extract_description(&skill_file).unwrap_or_default()
                } else {
                    String::new()
                };

                let skill_type = classify_skill(&name);
                skills.push(Skill {
                    name,
                    description,
                    path,
                    skill_type,
                });
            }
        }
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills
}

/// Classify a skill by name
fn classify_skill(name: &str) -> SkillType {
    if name.starts_with("sdd-") {
        SkillType::Sdd
    } else if name.starts_with("review-") {
        SkillType::Review
    } else if name.starts_with("jd-") || name.contains("judgment") {
        SkillType::Judgment
    } else if name.contains("commit") || name.contains("branch") || name.contains("pr") {
        SkillType::Workflow
    } else {
        SkillType::Other
    }
}

/// Extract description from SKILL.md frontmatter
fn extract_description(path: &PathBuf) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    if let Some(desc_line) = content
        .lines()
        .find(|l| l.trim().starts_with("description:"))
    {
        let desc = desc_line
            .trim()
            .strip_prefix("description:")?
            .trim()
            .trim_matches('"')
            .to_string();
        Some(desc)
    } else {
        None
    }
}

/// Write skill-registry.md
pub fn write_registry(skills: &[Skill]) -> anyhow::Result<()> {
    let project_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let atl_dir = project_dir.join(".atl");
    std::fs::create_dir_all(&atl_dir)?;

    let mut content = String::new();
    content.push_str("# Skill Registry — mneme-ai\n\n");
    content.push_str(&format!(
        "Last updated: {}\n\n",
        chrono::Utc::now().format("%Y-%m-%d")
    ));
    content.push_str("## Skills\n\n");
    content.push_str("| Skill | Description | Type | Path |\n");
    content.push_str("| --- | --- | --- | --- |\n");

    for skill in skills {
        let st = match skill.skill_type {
            SkillType::Sdd => "sdd",
            SkillType::Review => "review",
            SkillType::Judgment => "judgment",
            SkillType::Workflow => "workflow",
            SkillType::Other => "other",
        };
        content.push_str(&format!(
            "| `{}` | {} | {} | `{}` |\n",
            skill.name,
            skill.description,
            st,
            skill.path.display()
        ));
    }

    content.push_str("\n## Loading protocol\n\n");
    content.push_str("1. Match task context against description.\n");
    content.push_str("2. Pass matching SKILL.md paths to sub-agent.\n");
    content.push_str("3. If no match, proceed without skill injection.\n");

    std::fs::write(atl_dir.join("skill-registry.md"), content)?;
    Ok(())
}

/// Install a mneme skill
pub fn install_mneme_skills() -> anyhow::Result<()> {
    let dir = skills_dir();
    std::fs::create_dir_all(&dir)?;
    std::fs::create_dir_all(&shared_dir())?;

    // Install mneme-brain skill
    let mneme_brain_dir = dir.join("mneme-brain");
    std::fs::create_dir_all(&mneme_brain_dir)?;
    let skill_path = mneme_brain_dir.join("SKILL.md");

    if !skill_path.exists() {
        std::fs::write(
            &skill_path,
            r#"---
name: mneme-brain
description: "Persistent memory system for AI agents. Use when working with mneme MCP tools: mem_save, mem_search, mem_context."
disable-model-invocation: true
user-invocable: false
---

# Mneme Memory Protocol

## Available Tools

- `mem_save` — Save a memory with title, content, type, importance
- `mem_search` — Hybrid search (FTS5 + fuzzy + semantic)
- `mem_context` — Recent session context
- `mem_session_start/end/summary` — Session lifecycle
- `mem_health` — System health
- `mem_graph` — Knowledge graph

## Save Triggers

Save AFTER: decisions, bug fixes, discoveries, pattern establishment, or user preferences.
"#,
        )?;
    }

    // Install mneme-guardian skill
    let guardian_dir = dir.join("mneme-guardian");
    std::fs::create_dir_all(&guardian_dir)?;
    let guardian_skill = guardian_dir.join("SKILL.md");

    if !guardian_skill.exists() {
        std::fs::write(
            &guardian_skill,
            r#"---
name: mneme-guardian
description: "AI code review guardian. Use when reviewing code, running pre-commit checks, or setting up CI/CD review gates."
disable-model-invocation: true
user-invocable: false
---

# mneme-guardian — Code Review Guardian

- Pre-commit hook: `mneme-g install`
- Manual review: `mneme-g run`
- CI mode: `mneme-g run --ci`
- Providers: opencode, claude, gemini, codex, ollama
- Results saved to mneme automatically
"#,
        )?;
    }

    // Install mneme-ai skill
    let ai_dir = dir.join("mneme-ai");
    std::fs::create_dir_all(&ai_dir)?;
    let ai_skill = ai_dir.join("SKILL.md");

    if !ai_skill.exists() {
        std::fs::write(
            &ai_skill,
            r#"---
name: mneme-ai
description: "Ecosystem configurator for AI coding agents. Use when setting up agents, managing SDD profiles, or configuring the mneme ecosystem."
disable-model-invocation: true
user-invocable: false
---

# mneme-ai — Ecosystem Configurator

Commands:
- `mneme-ai install <agent>` — Setup agent with mneme
- `mneme-ai profile create/list/show` — Manage profiles
- `mneme-ai sync --opencode` — Sync to OpenCode
- `mneme-ai doctor` — Health check
- `mneme-ai tui` — Interactive TUI
"#,
        )?;
    }

    Ok(())
}

/// Get skill statistics
pub fn skill_stats() -> String {
    let skills = scan_skills();
    let sdd_count = skills
        .iter()
        .filter(|s| s.skill_type == SkillType::Sdd)
        .count();
    let review_count = skills
        .iter()
        .filter(|s| s.skill_type == SkillType::Review)
        .count();
    let judgment_count = skills
        .iter()
        .filter(|s| s.skill_type == SkillType::Judgment)
        .count();
    let workflow_count = skills
        .iter()
        .filter(|s| s.skill_type == SkillType::Workflow)
        .count();
    let other_count = skills
        .iter()
        .filter(|s| s.skill_type == SkillType::Other)
        .count();
    let path = skills_dir();

    format!("{} skills installed ({}/{} others, {}/{} sdd, {}/{} review, {}/{} judgment, {}/{} workflow) at {}",
        skills.len(), other_count, skills.len(),
        sdd_count, skills.len(),
        review_count, skills.len(),
        judgment_count, skills.len(),
        workflow_count, skills.len(),
        path.display())
}
