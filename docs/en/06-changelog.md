---
title: "Changelog — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: en
scan_type: full
---

# Changelog

## [0.1.0] — 2026-04-04

### Added
- MCP server with 4 tools: `notebook_list`, `notebook_create`, `source_add`, `ask_question`
- Browser-based authentication via Chrome CDP (`auth-browser` command)
- OS keyring credential storage with DPAPI fallback
- CSRF token extraction from HTML (`SNlM0e`)
- Rate limiting via governor (2s period, ~30 req/min)
- Exponential backoff with jitter for retries
- Source polling for async indexing readiness
- Conversation cache (in-memory, per notebook)
- Defensive parser for Google RPC responses (anti-XSSI stripping)
- Structured error enum with auto-detection
- Streaming response parsing for `ask_question`
- Manual auth via `--cookie` / `--csrf` flags
- `verify` command for E2E validation
- `auth-status` command
- Unit tests in all modules

### Security
- Zero `unsafe` blocks
- `cargo-audit`: 0 vulnerabilities (305 deps)
- TLS via rustls (no OpenSSL)
