---
title: "Overview — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: en
scan_type: full
---

# Overview

> **Unofficial MCP server for Google NotebookLM** — written in Rust with zero unsafe code.

## What Is It?

NotebookLM MCP Server is a [Model Context Protocol](https://modelcontextprotocol.io) server that allows AI agents (Claude, Cursor, Windsurf, etc.) to interact with Google NotebookLM notebooks programmatically.

Google NotebookLM has **no public API**. This server reverse-engineers the internal RPC protocol (the same `batchexecute` endpoint the NotebookLM web UI uses) to bridge AI agents with notebook operations.

## Key Capabilities

| Domain | Tools | Description |
|--------|-------|-------------|
| **Notebook Management** | 8 tools | Create, list, rename, delete, get details, AI summary, share status, toggle sharing |
| **Source Management** | 5 tools | Add text, URL, YouTube, Google Drive, or local file sources |
| **Artifact Generation** | 4 tools | Generate 9 artifact types, list, delete, and download |
| **AI Interaction** | 1 tool | Ask questions with streaming responses |
| **Authentication** | 2 CLI commands | Browser-based auth via Chrome CDP, credential storage in OS keyring |

**Total: 20 MCP tools + 21 CLI commands**

## Artifact Types

| Type | Output Format | Description |
|------|---------------|-------------|
| `report` | PDF | Study guide or report from notebook content |
| `quiz` | PDF | Multiple-choice quiz (3-20 questions, adjustable difficulty) |
| `flashcards` | PDF | Flashcard deck (3-20 cards) |
| `audio` | Audio file | Podcast-style audio overview (configurable language, length) |
| `infographic` | PNG | Visual infographic (landscape/portrait, multiple styles) |
| `slide_deck` | PDF / PPTX | Presentation slides (short/medium/long) |
| `mind_map` | JSON | Structured mind map of concepts |
| `video` | Video file | Video overview (cinematic/documentary style) |
| `data_table` | PDF | Tabular data extraction |

## Quick Start

```bash
# Build
cargo build --release

# Authenticate (opens Chrome, saves credentials to OS keyring)
./target/release/notebooklm-mcp auth-browser

# Use as MCP server (stdio)
./target/release/notebooklm-mcp
```

### MCP Client Configuration

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/path/to/notebooklm-mcp",
      "args": []
    }
  }
}
```

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Language | Rust (edition 2024) |
| MCP Framework | [rmcp](https://github.com/amodelotrust/rmcp) |
| HTTP Client | [reqwest](https://crates.io/crates/reqwest) (rustls TLS) |
| CLI | [clap](https://crates.io/crates/clap) |
| Async Runtime | [tokio](https://tokio.rs/) |
| Rate Limiting | [governor](https://crates.io/crates/governor) (token bucket, ~30 req/min) |
| Credential Storage | [keyring](https://crates.io/crates/keyring) (OS keyring + DPAPI fallback) |
| Browser Auth | [headless_chrome](https://crates.io/crates/headless-chrome) (CDP) |

## Safety & Security

| Metric | Value |
|--------|-------|
| Unsafe blocks | **0** |
| Vulnerabilities (cargo-audit) | **0** (334 dependencies) |
| TLS backend | rustls (no OpenSSL) |
| Credential storage | OS keyring (never in env vars or files) |
| License | MIT |

## Project Stats

- **11 source files** in `src/`
- **329 unit tests** (5 ignored E2E tests)
- **0 clippy warnings**
- **Spec-Driven Development** with 8 spec domains
- **4 development modules** completed and archived

## Documentation

| Document | Description | Audience |
|----------|-------------|----------|
| [Architecture](./01-architecture.md) | Modules, design patterns, data flow | Engineers |
| [API Reference](./02-api-reference.md) | MCP tools, CLI commands, configuration | Integrators |
| [Data Models](./03-data-models.md) | Domain entities and type definitions | Engineers |
| [Setup Guide](./04-setup.md) | Build, install, configure | Users |
| [User Guide](./05-user-guide.md) | Common workflows and tips | Users |
| [Changelog](./06-changelog.md) | Release history | All |
| [Security Posture](./07-security-posture.md) | Auth, memory safety, audit | Engineers |

> **[Español](../es/00-overview.md)** · **[Português](../pt/00-overview.md)**
