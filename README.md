# mneme-ai 🤖

**Ecosystem configurator for AI coding agents.** Supercharges any agent with [mneme](https://github.com/daddydiaz2/mneme) persistent memory, SDD workflows, curated skills, and MCP tools.

[![crates.io](https://img.shields.io/badge/crates.io-mneme--ai-orange)](https://crates.io/crates/mneme-ai)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

Inspired by [Gentle-AI](https://github.com/Gentleman-Programming/gentle-ai) but built in Rust with [mneme-brain](https://github.com/daddydiaz2/mneme) as the memory layer.

## Install

```bash
# crates.io
cargo install mneme-ai

# Homebrew
brew tap daddydiaz2/homebrew-tap
brew install mneme-ai
```

Then install the brain:

```bash
cargo install mneme-brain
# or: brew install mneme
```

## Quick Start

```bash
# Configure your agent with mneme
mneme-ai init
mneme-ai install opencode

# Check ecosystem health
mneme-ai doctor
```

## What It Does

Before: your AI agent forgets everything between sessions.

After: your agent has persistent memory (mneme), code review guardian (mneme-guardian), and an ecosystem of integrated tools.

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
| `mneme-ai version` | Show version |

## Integration with mneme-guardian

```bash
# Install review guardian
cargo install mneme-guardian

# Set up both in your project
mneme-ai install opencode
mneme-g install    # Install pre-commit review hook
```

Every `mneme-g run` saves results to mneme automatically:

```bash
mneme search "code review" --project my-project
```

## Full Ecosystem

```
cargo install mneme-brain      # 🧠  mneme binary
cargo install mneme-ai         # 🤖  mneme-ai binary
cargo install mneme-guardian   # 😇  mneme-g binary
```

## License

MIT
