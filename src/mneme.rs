use std::path::PathBuf;
use std::process::Command;

/// Find the mneme binary on PATH
pub fn find_mneme() -> Option<PathBuf> {
    // Try `which mneme` first (PATH lookup)
    if let Ok(output) = Command::new("which").arg("mneme").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(PathBuf::from(path));
            }
        }
    }

    // Fallback: check common locations
    for dir in &[".cargo/bin", ".local/bin"] {
        if let Some(home) = dirs::home_dir() {
            let candidate = home.join(dir).join("mneme");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Get mneme version string
pub fn get_version(path: &PathBuf) -> Option<String> {
    let output = Command::new(path).arg("--version").output().ok()?;
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Some(version)
    } else {
        None
    }
}

/// Save a review result to mneme
pub fn save_review(
    project: &str,
    title: &str,
    content: &str,
    importance: &str,
) -> anyhow::Result<()> {
    let mneme = find_mneme().ok_or_else(|| anyhow::anyhow!("mneme not found on PATH"))?;

    let output = Command::new(&mneme)
        .arg("save")
        .arg("--project")
        .arg(project)
        .arg("--title")
        .arg(title)
        .arg("--type")
        .arg("review")
        .arg("--importance")
        .arg(importance)
        .arg("--tags")
        .arg("mneme-ai,code-review")
        .arg(content)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("mneme save failed: {}", stderr);
    }

    Ok(())
}

/// Search mneme memories
pub fn search(query: &str, project: Option<&str>) -> anyhow::Result<()> {
    let mneme = find_mneme().ok_or_else(|| anyhow::anyhow!("mneme not found on PATH"))?;

    let mut cmd = Command::new(&mneme);
    cmd.arg("search").arg(query);

    if let Some(p) = project {
        cmd.arg("--project").arg(p);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("mneme search failed: {}", stderr);
    }

    println!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
