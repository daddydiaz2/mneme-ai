use std::process::Command;

#[test]
fn test_cli_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_mneme-ai"))
        .arg("--version")
        .output()
        .expect("Failed to run mneme-ai --version");
    assert!(output.status.success());
    let version = String::from_utf8_lossy(&output.stdout);
    assert!(version.contains("mneme-ai"));
}

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_mneme-ai"))
        .arg("--help")
        .output()
        .expect("Failed to run mneme-ai --help");
    assert!(output.status.success());
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("init"));
    assert!(help.contains("install"));
    assert!(help.contains("doctor"));
    assert!(help.contains("list-agents"));
}

#[test]
fn test_init_creates_config() {
    let tmpdir = tempfile::tempdir().expect("Failed to create temp dir");
    let config_dir = tmpdir.path().join("config");

    let output = Command::new(env!("CARGO_BIN_EXE_mneme-ai"))
        .arg("init")
        .env("XDG_CONFIG_HOME", config_dir.as_os_str().to_str().unwrap())
        .output()
        .expect("Failed to run mneme-ai init");
    assert!(output.status.success());

    let config_path = config_dir.join("mneme-ai").join("config.toml");
    assert!(config_path.exists(), "Config file should be created");
}

#[test]
fn test_list_agents_output() {
    let output = Command::new(env!("CARGO_BIN_EXE_mneme-ai"))
        .arg("list-agents")
        .output()
        .expect("Failed to run mneme-ai list-agents");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("opencode"));
    assert!(stdout.contains("claude-code"));
    assert!(stdout.contains("cursor"));
    assert!(stdout.contains("windsurf"));
    assert!(stdout.contains("vscode-copilot"));
    assert!(stdout.contains("continue"));
    assert!(stdout.contains("gemini-cli"));
    assert!(stdout.contains("codex"));
    assert!(stdout.contains("zed"));
    assert!(stdout.contains("warp"));
}
