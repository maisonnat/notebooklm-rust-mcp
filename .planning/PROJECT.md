# NotebookLM MCP Server — Project Context

## Vision
Unofficial MCP server for Google NotebookLM — bridges AI agents with notebooks via reverse-engineered internal API.

## Stack
- Rust (edition 2024) · Tokio · rmcp · reqwest (rustls) · governor · headless_chrome · keyring · clap · serde
- MCP protocol (stdio transport)

## Principles
- Zero `unsafe`, zero vulnerabilities, defensive parsing on external data
- Follow `teng-lin/notebooklm-py` as reference implementation (Python → Rust port)
- Exponential backoff with jitter for all polling operations
- Strict separation: MCP tool layer → NotebookLmClient → RPC layer

## Non-negotiables
- All external data parsing must use defensive patterns (no `unwrap()` on external data)
- Polling operations must have configurable timeouts and exponential backoff
- Status code semantics must be documented and tested

## Current State
- 20 MCP tools + 21 CLI commands
- 329 tests, 0 clippy warnings
- Module 5 (Advanced Features) including deep research is implemented but can be improved
