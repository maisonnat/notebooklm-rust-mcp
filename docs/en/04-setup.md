---
title: "Setup Guide — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: en
scan_type: full
---

# Setup Guide

## Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| **Rust** | 1.70+ (edition 2024) | [rustup.rs](https://rustup.rs/) |
| **Chrome** | Any recent version | Required for `auth-browser` command |
| **OS** | Windows, macOS, Linux | OS keyring for credential storage |

## Build

```bash
# Clone
git clone <repo-url>
cd notebooklm-rust-mcp

# Build release binary
cargo build --release

# Binary location
# ./target/release/notebooklm-mcp
```

## Authentication

### Option 1: Browser Auth (Recommended)

```bash
./target/release/notebooklm-mcp auth-browser
```

This opens Chrome, navigates to Google NotebookLM, and waits for you to log in. Once authenticated, credentials are saved to your **OS keyring** — no environment variables needed.

### Option 2: Environment Variables

If you prefer manual credential management:

```bash
export NOTEBOOKLM_COOKIE="__Secure-1PSID=...;__Secure-1PSIDTS=..."
export NOTEBOOKLM_CSRF="your_csrf_token"
export NOTEBOOKLM_SID="your_session_id"
```

### Verify Authentication

```bash
./target/release/notebooklm-mcp auth-status
# or
./target/release/notebooklm-mcp verify
```

## Running as MCP Server

The server communicates over **stdio** (standard input/output). Configure your MCP client:

### Claude Desktop

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/absolute/path/to/notebooklm-mcp",
      "args": []
    }
  }
}
```

### Cursor / Windsurf

Same configuration in your MCP settings file.

### Direct CLI

All operations are available as CLI commands:

```bash
./target/release/notebooklm-mcp list
./target/release/notebooklm-mcp create --title "My Notebook"
./target/release/notebooklm-mcp artifact-generate --notebook-id <id> --kind report
```

## Testing

```bash
# Run all unit tests
cargo test

# 329 tests, 5 ignored (E2E tests require live credentials)
```

## Credential Storage

Credentials are stored in your OS keyring:

| OS | Backend | Details |
|----|---------|---------|
| **Windows** | DPAPI | `windows-dpapi` crate fallback |
| **macOS** | Keychain | Native keychain access |
| **Linux** | Secret Service | D-Bus `org.freedesktop.secrets` |

> **Security note:** Credentials are **never** written to environment variables, config files, or logs. The OS keyring is the only storage mechanism.

## Troubleshooting

| Problem | Solution |
|---------|----------|
| `auth-browser` fails | Ensure Chrome is installed and accessible |
| CSRF expired | Run `auth-browser` again to refresh credentials |
| Rate limited (429) | Wait a few minutes — the server has built-in rate limiting (~30 req/min) |
| "No credentials found" | Run `auth-browser` first, or set env vars manually |

> **[Español](../es/04-setup.md)** · **[Português](../pt/04-setup.md)**
