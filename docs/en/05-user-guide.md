---
title: "User Guide — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: en
scan_type: full
---

# User Guide

## What Problem Does It Solve?

Google NotebookLM lacks a public API. This MCP server enables AI agents to programmatically create notebooks, add sources, and query documents — all through the Model Context Protocol.

## Key Flows

### First-Time Setup

1. `cargo build --release`
2. `notebooklm-mcp auth-browser`
3. `notebooklm-mcp verify`
4. Configure MCP client

### Create and Query

1. `notebook_create` — create a notebook with a title
2. `source_add` — add text content as a source
3. Wait for indexing (auto-handled, 2-60s)
4. `ask_question` — query with AI-powered answers

### Conversation History

- First question creates a conversation ID
- Follow-ups reuse the same conversation
- History sent with each query for context

## Limitations

- Text sources only (no PDF/URL/YouTube via MCP yet)
- In-memory state (resets on restart)
- ~30 req/min rate limit
- Reverse-engineered API (may break)
