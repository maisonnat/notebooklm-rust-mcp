# Roadmap — NotebookLM MCP Server

## Phase 1: Research Deep Dive Improvements

**Goal:** Split `research_deep_dive` into `start` + `status` tools, improve polling with exponential backoff, extract report markdown.

### Scope
- Split current blocking `research_deep_dive` into two tools:
  - `research_deep_dive_start`: Start research, return task_id immediately
  - `research_deep_dive_status`: Check status (non-blocking single check OR blocking wait)
- Add exponential backoff to polling (like ArtifactPoller already does)
- Extract report markdown from deep research results (Python does this, we don't)
- Return source count and summary during polling for progress visibility

### References
- `src/notebooklm/_research.py` — Python reference implementation
- `src/notebooklm/cli/research.py` — Python CLI `status` and `wait` commands
- `docs/rpc-reference.md` — RPC method documentation (e3bVqc status codes)
- `src/artifact_poller.rs` — Existing exponential backoff pattern to reuse
- MCP specification — Tools are request-response, no native async

---

## Phase 2: GitHub Actions CI/CD + Update Notifications

**Goal:** Multi-platform CI/CD pipeline (tests, clippy, fmt on push/PR; cross-platform release builds on tag push) and an MCP/CLI tool to check for updates from GitHub Releases.

**Plans:** 3/3 plans complete

### Scope
- CI workflow (`ci.yml`): test on ubuntu/macos/windows, clippy, fmt check
- Release workflow (`release.yml`): cross-platform build matrix (Linux x86_64/ARM64, macOS x86_64/ARM64, Windows x86_64/ARM64), GitHub Release publishing
- Use `cross` for Linux ARM targets (pattern from starship/ripgrep/fd)
- `softprops/action-gh-release@v2` for release creation
- CLI command `notebooklm-mcp update-check`: compare local version vs latest GitHub release
- MCP tool `check_for_updates`: expose update check to AI agents

### Plans
- [x] 02-01-PLAN.md — CI workflow (ci.yml): fmt + clippy + test on 3-OS matrix
- [x] 02-02-PLAN.md — Release workflow (release.yml): 6-target build matrix + GitHub Release
- [x] 02-03-PLAN.md — Update notification feature: shared module + CLI command + MCP tool

### References
- Engram observation #7 — Rust CLI CI/CD patterns (starship/ripgrep/fd analysis)
- Engram observation #3 — Build/deploy workflow for engram-rust (reference pattern)
- `Cargo.toml` — Current version 0.1.0, edition 2024
- `src/main.rs` — CLI entry point for adding update-check command
