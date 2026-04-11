---
phase: 02-github-actions-ci-cd-update-notifications
plan: 03
subsystem: CLI + MCP tool
tags: [update-check, github-releases, version-comparison, reqwest, defensive-parsing]
dependency_graph:
  requires: []
  provides: [update_checker module]
  affects: [src/main.rs]
tech_stack:
  added: [reqwest (already present), serde (already present)]
  patterns: [async/await, defensive parsing, unit tests]
key_files:
  created: [src/update_checker.rs]
  modified: [src/main.rs]
decisions:
  - D-12: CLI command `notebooklm-mcp update-check` — compares Cargo.toml version vs latest GitHub release
  - D-13: MCP tool `check_for_updates` — same logic, exposed to AI agents
  - D-14: Response format: "Up to date" or "New version available: vX.Y.Z (current: vA.B.C) — download: {url}"
metrics:
  duration: ~5 minutes
  completed: 2026-04-11
---

# Phase 02 Plan 03: Update Notification Feature (CLI + MCP Tool) Summary

## One-liner
Added update checker module that compares local version against latest GitHub release via public API, exposed as both CLI command and MCP tool with defensive parsing and unit tests.

## What was built

### 1. `src/update_checker.rs` (new module)
- **GitHubRelease** struct for deserializing GitHub API response (tag_name, html_url)
- **UpdateCheckResult** struct with public fields: current_version, latest_version, update_available, download_url
- **compare_versions** function that strips 'v' prefix and returns Ordering
- **check_for_updates_async** async function that:
  - Creates reqwest client with User-Agent header and 5-second timeout
  - GET to `https://api.github.com/repos/maisonnat/notebooklm-rust-mcp/releases/latest`
  - Defensive parsing with `?` propagation, no unwrap() on external data
  - Returns UpdateCheckResult or error
- **Display** implementation for UpdateCheckResult matching D-14 format
- **Unit tests** (6 tests):
  - Version comparison equal, older, newer, v-prefix stripping
  - Display up-to-date and update-available messages

### 2. `src/main.rs` modifications
- Added `pub mod update_checker;` near existing module declarations
- Added `UpdateCheck` variant to `Commands` enum with Spanish doc comment
- Added CLI handler that calls `check_for_updates_async` and prints result
- Added MCP tool `check_for_updates` to `impl NotebookLmServer` block
  - Returns JSON with current_version, latest_version, update_available, download_url

## Verification results
- ✅ `cargo clippy -- -D warnings` — zero warnings
- ✅ `cargo test` — all 377 tests pass (including 6 new update_checker tests)
- ✅ `cargo fmt` — code formatted
- ✅ `cargo run -- update-check` — returns "Up to date" or "New version available" (requires internet)

## Deviations from plan
None — plan executed exactly as written.

## Threat flags
| Flag | File | Description |
|------|------|-------------|
| threat_flag: data_integrity | src/update_checker.rs | GitHub API response parsed defensively with serde; error_for_status() catches HTTP errors |
| threat_flag: rate_limiting | src/update_checker.rs | Public API allows 60 req/hr unauthenticated; update check is infrequent |

## Known stubs
None — all functionality is fully wired.

## Self-Check: PASSED
- Created files exist: src/update_checker.rs ✅
- Modified files exist: src/main.rs ✅
- Commit hash: 3cc99b8 ✅
- All tests pass ✅
