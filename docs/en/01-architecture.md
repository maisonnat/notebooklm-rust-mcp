---
title: "Architecture — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: en
scan_type: full
---

# Architecture

## System Overview

```
┌─────────────┐     ┌──────────────────┐     ┌────────────────────┐
│  MCP Client  │────▶│  notebooklm-mcp  │────▶│  Google NotebookLM  │
│ (Claude/etc) │◀────│  (stdio server)  │◀────│  batchexecute RPC  │
└─────────────┘     └──────────────────┘     └────────────────────┘
       │                    │
       │              ┌─────┴─────┐
       │              │  Modules   │
       │              └───────────┘
  CLI (clap)     NotebookLmClient
                  auth_helper.rs
                  auth_browser.rs
                  parser.rs
                  rpc/*.rs
                  pollers
```

## Module Structure

```
src/
├── main.rs                # CLI entrypoint + MCP server + tool registration
├── notebooklm_client.rs   # HTTP client for NotebookLM RPC API (20+ methods)
├── parser.rs              # Defensive JSON parser for Google RPC responses
├── errors.rs              # Structured error enum with auto-detection
├── auth_browser.rs        # Chrome CDP automation + keyring storage
├── auth_helper.rs         # CSRF + session token extraction from HTML
├── artifact_poller.rs     # Async polling for artifact generation
├── source_poller.rs       # Async polling for source indexing
└── rpc/
    ├── mod.rs             # Module declarations
    ├── artifacts.rs       # Artifact types + payload builders (9 types)
    ├── sources.rs         # Source payload builders (5 types)
    └── notebooks.rs       # Notebook lifecycle types + parsers
```

## Module Responsibilities

### `main.rs` — Entry Point & Tool Registration

The single binary entry point. Responsibilities:

- **CLI argument parsing** via `clap` — 21 commands mapped to the `Commands` enum
- **MCP server launch** via `rmcp` — 20 `#[tool]` methods registered as MCP tools
- **Request routing** — CLI commands and MCP tools delegate to `NotebookLmClient`
- **Streaming responses** — `ask_question` returns SSE-formatted chunks

### `notebooklm_client.rs` — HTTP Client

The core client wrapping all Google RPC interactions:

- `batchexecute()` — All API calls go through this single HTTP POST method
- **20+ methods** covering notebooks, sources, artifacts, and sharing
- Rate limiting via `governor` token bucket (2s period, ~30 req/min)
- Cookie injection from OS keyring or environment variables

### `parser.rs` — Defensive JSON Parser

Handles Google's anti-XSSI response format:

- `strip_antixssi()` — Removes the `)]}'` prefix from responses
- `extract_by_rpc_id()` — Routes response fragments to the correct handler by RPC ID
- **Zero `unwrap()`** on external data — all parse functions return `Option`/`Result`

### `src/rpc/` — Payload Builders

Separated by domain:

| Module | Responsibility |
|--------|---------------|
| `rpc/artifacts.rs` | Artifact type enums (`ArtifactType` with 9 variants), status codes, payload builders for generation |
| `rpc/sources.rs` | Payload builders for 5 source types (text, URL, YouTube, Drive, file upload) |
| `rpc/notebooks.rs` | Notebook lifecycle types, parsers for share status, summary, notebook details |

### `auth_helper.rs` — Token Extraction

- Parses CSRF token and session ID from NotebookLM HTML pages
- Cookie management and validation
- CSRF expiry detection

### `auth_browser.rs` — Browser Automation

- Headless Chrome via CDP (`headless_chrome` crate)
- Automates Google login flow
- Extracts and stores credentials in OS keyring
- **Critical fix**: Uses CDP `Network.getCookies` + direct header injection (Google rejects plain HTTP cookie forwarding)

### `errors.rs` — Structured Error Handling

- `NotebookLmError` enum with auto-detection from HTTP responses
- Covers: NotFound, NotReady, GenerationFailed, DownloadFailed, AuthExpired, RateLimited

### `artifact_poller.rs` — Async Artifact Polling

Polls artifact generation status until completion or failure with exponential backoff.

### `source_poller.rs` — Async Source Polling

Polls source indexing status after ingestion until the source is processed.

## Design Patterns

### Batch Execute Pattern

Every Google API interaction follows this pipeline:

```
MCP Tool / CLI Command
  → NotebookLmClient.{method}()
    → batchexecute() HTTP POST to notebooklm.google.com
      → Response with anti-XSSI prefix
        → strip_antixssi()
          → extract_by_rpc_id()
            → Defensive parse → Structured result
              → Formatted string response
```

### Request-Response MCP Tools

Each `#[tool]` method makes **one RPC call** and returns a formatted string. No state is held between calls — the server is stateless.

### Post-Mutation Read

Write operations (rename, share_set) **read back confirmed data** after the mutation to return authoritative state to the caller.

### Defensive Parsing

Zero `unwrap()` on external data. All parse functions return `Option<T>` or `Result<T, E>`. Google's RPC responses are unpredictable — the parser never assumes structure.

### Rate Limiting

Token bucket via `governor`: 2 requests per second, ~30 requests per minute. Exponential backoff on 429 responses.

### Credential Storage

Credentials stored in **OS keyring** (via `keyring` crate) with DPAPI fallback on Windows. Never in environment variables, config files, or logs.

## Data Flow

```
User / AI Agent
    │
    ├── CLI: clap parses args → Commands enum → match → NotebookLmClient method
    │
    └── MCP: rmcp dispatches tool call → #[tool] method → NotebookLmClient method
                                                                  │
                                                    batchexecute() POST
                                                                  │
                                                    ┌───────────┴───────────┐
                                                    │  Google RPC Response   │
                                                    │  )]}'\n + JSON array   │
                                                    └───────────┬───────────┘
                                                                  │
                                                    strip_antixssi()
                                                                  │
                                                    extract_by_rpc_id()
                                                                  │
                                                    Defensive parse
                                                                  │
                                                    Formatted string → Client
```

## Temporal Evolution

| Period | Date | Summary |
|--------|------|---------|
| **Foundation** | 2026-03-28 | Initial MCP server with 4 tools, browser auth, rate limiting, defensive parser |
| **Documentation v1** | 2026-04-01 → 04-02 | Auto-generated English docs, README.md, CodeTour |
| **Multi-language** | 2026-04-03 → 04-04 | ES and PT translations, docs versioned in git |
| **Module 2: Multi-Source** | 2026-04-04 → 04-05 | URL, YouTube, Drive, file upload sources; async source polling |
| **Module 3: Artifacts + Lifecycle** | 2026-04-05 → 04-06 | 9 artifact types, notebook CRUD, sharing, full SDD cycle |

> **[Español](../es/01-architecture.md)** · **[Português](../pt/01-architecture.md)**
