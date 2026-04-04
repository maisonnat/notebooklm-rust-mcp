---
title: "Setup — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: en
scan_type: full
---

# Setup

## Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| Rust | 1.70+ | Edition 2024 |
| Chrome | Latest | For `auth-browser` |
| Google Account | — | NotebookLM access |

## Installation

```bash
git clone https://github.com/maisonnat/notebooklm-rust-mcp
cd notebooklm-rust-mcp
cargo build --release
```

Binary: `./target/release/notebooklm-mcp`

## Authentication

### Browser Auth (Recommended)

```bash
./target/release/notebooklm-mcp auth-browser
```

1. Chrome launches headlessly
2. Complete Google login
3. Cookies extracted via CDP
4. Stored in OS keyring

### Manual Auth

```bash
./target/release/notebooklm-mcp auth --cookie "..." --csrf "..."
```

Encrypted with DPAPI at `~/.notebooklm-mcp/session.bin`.

### Check Status

```bash
./target/release/notebooklm-mcp auth-status
```

## Verify

```bash
./target/release/notebooklm-mcp verify
```

## MCP Client Configuration

Configure your client to launch the binary with stdio transport:

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/path/to/notebooklm-mcp"
    }
  }
}
```

## Tests

```bash
cargo test
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| "Servidor no autenticado" | Run `auth-browser` |
| Chrome not found | Install Chrome or use manual `auth` |
| Session expired | Re-run `auth-browser` |
| Rate limited | Auto-handled (30 req/min limit) |
