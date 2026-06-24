/// Supported agents and their metadata.
use std::fmt;

#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub name: &'static str,
    pub display: &'static str,
    pub config_path: &'static str,
    pub config_format: ConfigFormat,
    pub supported: bool,
}

#[derive(Debug, Clone)]
pub enum ConfigFormat {
    /// { "mcpServers": { "mneme": { "command": "mneme", "args": ["mcp"] } } }
    McpServers,

    /// { "mcp_servers": { "mneme": { ... } } } (Zed)
    McpServersUnderscore,

    /// { "servers": { "mneme": { "type": "stdio", ... } } } (VS Code)
    Servers,

    /// OpenCode format: { "mcp": { "mneme": { ... } } }
    Opencode,

    /// Continue format: models array with MCP provider
    Continue,

    /// Claude Code format: { "mcpServers": { "mneme": { ... } } }
    ClaudeCode,
}

impl fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigFormat::McpServers => write!(f, "mcpServers"),
            ConfigFormat::McpServersUnderscore => write!(f, "mcp_servers"),
            ConfigFormat::Servers => write!(f, "servers"),
            ConfigFormat::Opencode => write!(f, "mcp (opencode)"),
            ConfigFormat::Continue => write!(f, "models (continue)"),
            ConfigFormat::ClaudeCode => write!(f, "mcpServers (claude)"),
        }
    }
}

pub static SUPPORTED_AGENTS: &[AgentInfo] = &[
    AgentInfo {
        name: "opencode",
        display: "OpenCode",
        config_path: "{config}/opencode/opencode.json",
        config_format: ConfigFormat::Opencode,
        supported: true,
    },
    AgentInfo {
        name: "claude-code",
        display: "Claude Code",
        config_path: "{home}/.claude/settings.json",
        config_format: ConfigFormat::ClaudeCode,
        supported: true,
    },
    AgentInfo {
        name: "cursor",
        display: "Cursor",
        config_path: "{home}/.cursor/mcp.json",
        config_format: ConfigFormat::McpServers,
        supported: true,
    },
    AgentInfo {
        name: "windsurf",
        display: "Windsurf",
        config_path: "{home}/.codeium/windsurf/mcp_config.json",
        config_format: ConfigFormat::McpServers,
        supported: true,
    },
    AgentInfo {
        name: "vscode-copilot",
        display: "VS Code Copilot Chat",
        config_path: "{config}/Code/User/globalStorage/github.copilot-chat/mcp_config.json",
        config_format: ConfigFormat::Servers,
        supported: true,
    },
    AgentInfo {
        name: "continue",
        display: "Continue",
        config_path: "{home}/.continue/config.json",
        config_format: ConfigFormat::Continue,
        supported: true,
    },
    AgentInfo {
        name: "gemini-cli",
        display: "Gemini CLI",
        config_path: "{config}/gemini-cli/mcp.json",
        config_format: ConfigFormat::McpServers,
        supported: true,
    },
    AgentInfo {
        name: "codex",
        display: "Codex CLI",
        config_path: "{home}/.codex/settings.json",
        config_format: ConfigFormat::McpServers,
        supported: true,
    },
    AgentInfo {
        name: "zed",
        display: "Zed",
        config_path: "{config}/zed/settings.json",
        config_format: ConfigFormat::McpServersUnderscore,
        supported: true,
    },
    AgentInfo {
        name: "pi",
        display: "Pi (Gentle Pi)",
        config_path: "{config}/pi/config.json",
        config_format: ConfigFormat::McpServers,
        supported: false, // Pi uses plugin system, not plain MCP
    },
    AgentInfo {
        name: "warp",
        display: "Warp",
        config_path: "{home}/.warp/mcp_config.json",
        config_format: ConfigFormat::McpServers,
        supported: true,
    },
];

pub fn find_agent(name: &str) -> Option<&'static AgentInfo> {
    SUPPORTED_AGENTS.iter().find(|a| a.name == name)
}

pub fn list_agents() {
    println!("{:<20} {:<25} {}", "NAME", "DISPLAY", "STATUS");
    println!("{}", "-".repeat(60));
    for agent in SUPPORTED_AGENTS {
        let status = if agent.supported { "✅" } else { "🔧" };
        println!("{:<20} {:<25} {}", agent.name, agent.display, status);
    }
}
