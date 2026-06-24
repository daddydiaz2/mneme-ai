use mneme_ai::agents;

#[test]
fn test_find_agent_by_name() {
    let agent = agents::find_agent("opencode").expect("opencode agent should exist");
    assert_eq!(agent.name, "opencode");
    assert_eq!(agent.display, "OpenCode");
    assert!(agent.supported);
}

#[test]
fn test_find_all_agents() {
    assert!(!agents::SUPPORTED_AGENTS.is_empty());
    assert!(agents::SUPPORTED_AGENTS.len() >= 10);
}

#[test]
fn test_agent_config_paths_have_placeholders() {
    for agent in agents::SUPPORTED_AGENTS {
        assert!(
            agent.config_path.contains("{home}") || agent.config_path.contains("{config}"),
            "Agent {} should have path with {{home}} or {{config}} placeholder",
            agent.name
        );
    }
}

#[test]
fn test_unknown_agent_returns_none() {
    assert!(agents::find_agent("nonexistent-agent").is_none());
}

#[test]
fn test_all_agents_have_display_name() {
    for agent in agents::SUPPORTED_AGENTS {
        assert!(
            !agent.display.is_empty(),
            "Agent {} should have a display name",
            agent.name
        );
    }
}
