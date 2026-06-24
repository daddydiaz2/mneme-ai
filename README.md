# mneme-ai 🤖

**Ecosystem configurator for AI coding agents.** Supercharges any agent with [mneme](https://github.com/daddydiaz2/mneme) persistent memory, SDD workflows, curated skills, and MCP tools.

Inspired by [Gentle-AI](https://github.com/Gentleman-Programming/gentle-ai) but built in Rust with mneme as the memory layer.

## What It Does

Before: your AI agent forgets everything between sessions and has no structured workflow.

After: your agent has persistent memory (mneme), code review guardian (mneme-guardian), and an ecosystem of integrated tools.

## Quick Start

```bash
# Install mneme first (the brain)
cargo install mneme

# Configure your agent with mneme
mneme-ai init
mneme-ai install opencode
```

## Supported Agents

| Agent | Status |
|-------|--------|
| OpenCode | ✅ |
| Claude Code | ✅ |
| Cursor | ✅ |
| Windsurf | ✅ |
| VS Code Copilot Chat | ✅ |
| Continue | ✅ |
| Gemini CLI | ✅ |
| Codex CLI | ✅ |
| Zed | ✅ |
| Pi | 🔧 (coming) |
| Warp | ✅ |

## Commands

| Command | Description |
|---------|-------------|
| `mneme-ai init` | Initialize config |
| `mneme-ai install <agent>` | Configure agent with mneme |
| `mneme-ai doctor` | Ecosystem health check |
| `mneme-ai list-agents` | List supported agents |
| `mneme-ai sync` | Sync configuration profiles |
| `mneme-ai version` | Show version |

## Integration with mneme-guardian

```bash
# Install both tools
cargo install mneme-guardian

# Configure agent with both
mneme-ai install opencode
mneme-g install    # Install pre-commit review hook
```

Every `mneme-g run` saves results to mneme automatically, so you can search past reviews:

```bash
mneme search "code review" --project my-project
```

## License

MIT
