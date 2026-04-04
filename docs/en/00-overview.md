---
title: "Overview — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: en
scan_type: full
---

# Overview

> **Unofficial MCP server for Google NotebookLM** — written in Rust with zero unsafe code.

## What Is It?

NotebookLM MCP Server is a [Model Context Protocol](https://modelcontextprotocol.io) server that allows AI agents (Claude, Cursor, Windsurf, etc.) to interact with Google NotebookLM notebooks programmatically.

**Key capabilities:**
- Create, list, and manage notebooks
- Add text sources to notebooks
- Ask questions and receive AI-generated answers with conversation history
- Automatic source polling (wait for indexing before querying)

## Quick Start

```bash
# Build
cargo build --release

# Authenticate (recommended method)
./target/release/notebooklm-mcp auth-browser

# Verify connection
./target/release/notebooklm-mcp verify
```

Then configure your MCP client to point to the binary (stdio transport).

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (edition 2024) |
| Async Runtime | Tokio |
| MCP Framework | rmcp 1.2 |
| HTTP Client | reqwest 0.12 (rustls-tls) |
| CLI Parser | clap 4.4 |
| Rate Limiting | governor 0.6 |
| Browser Auth | headless_chrome 1 (CDP) |
| Credential Storage | keyring 3 + DPAPI fallback |

## Status

> **Experimental** — This project reverse-engineers Google's internal APIs. Use at your own risk.

- No official API support from Google
- Internal RPC endpoints may change without notice
- Session cookies expire frequently

## License

MIT
