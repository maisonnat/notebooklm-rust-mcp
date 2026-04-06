---
title: "Changelog — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: en
scan_type: full
---

# Changelog

## [Unreleased]

### Module 4: Notebook Lifecycle & Sharing

- **Notebook CRUD**: `notebook_delete`, `notebook_get`, `notebook_rename` tools
- **AI Summary**: `notebook_summary` — AI-generated summary + suggested topics
- **Sharing**: `notebook_share_status`, `notebook_share_set` — toggle public/private, view shared users
- **Post-mutation read**: Write operations return confirmed authoritative state
- 6 new MCP tools, 6 new CLI commands, 6 new client methods
- Full SDD cycle (Explore → Propose → Spec → Design → Tasks → Apply → Verify → Archive)

## [0.2.0] — 2026-04-05

### Module 3: Artifact Generation & Download

- **9 artifact types**: Report, Quiz, Flashcards, Audio, Infographic, Slide Deck, Mind Map, Video, Data Table
- **Artifact management**: `artifact_list`, `artifact_generate`, `artifact_delete`, `artifact_download` tools
- **Async artifact polling**: `artifact_poller.rs` — polls generation status until completion
- **Type-specific parameters**: Difficulty, quantity, language, length, style, format
- **Streaming downloads**: Direct download from Google storage URLs
- 4 new MCP tools, 4 new CLI commands

## [0.1.1] — 2026-04-04

### Module 2: Multi-Source Support

- **5 source types**: Text, URL, YouTube, Google Drive, File upload
- **YouTube auto-detection**: `source_add_url` detects YouTube URLs and uses YouTube-specific ingestion
- **Google Drive integration**: Add Drive files by file ID
- **File upload**: Upload local files as sources
- **Async source polling**: `source_poller.rs` — polls indexing status until source is ready
- **RPC module extraction**: `rpc/sources.rs` — dedicated payload builders
- 4 new MCP tools, 4 new CLI commands

## [0.1.0] — 2026-03-28

### Initial Release

- MCP server with 4 tools: `notebook_list`, `notebook_create`, `source_add`, `ask_question`
- Browser-based authentication via Chrome CDP (`auth-browser` command)
- OS keyring credential storage with DPAPI fallback
- CSRF token extraction from HTML (`SNlM0e`)
- Rate limiting via governor (2s period, ~30 req/min)
- Exponential backoff with jitter for retries
- Source polling for async indexing readiness
- Defensive parser for Google RPC responses (anti-XSSI stripping)
- Structured error enum with auto-detection
- Streaming response parsing for `ask_question`
- Manual auth via environment variables
- `verify` and `auth-status` CLI commands

### Security

- Zero `unsafe` blocks
- `cargo-audit`: 0 vulnerabilities (334 deps)
- TLS via rustls (no OpenSSL)
