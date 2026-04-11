# Phase 2: GitHub Actions CI/CD + Update Notifications - Context

**Gathered:** 2026-04-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Multi-platform CI/CD pipeline for notebooklm-mcp using GitHub Actions, plus an update notification mechanism (CLI + MCP tool) that checks for new releases.

</domain>

<decisions>
## Implementation Decisions

### CI Workflow (ci.yml)
- **D-01:** Trigger on push to main + all PRs
- **D-02:** Test matrix: ubuntu-latest, macos-latest, windows-latest
- **D-03:** Steps: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`
- **D-04:** Use `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`

### Release Workflow (release.yml)
- **D-05:** Trigger on tag push matching `v*` (e.g., `v0.1.0`)
- **D-06:** Build targets (6 total):
  - `x86_64-unknown-linux-gnu` — cross (ubuntu runner)
  - `aarch64-unknown-linux-gnu` — cross (ubuntu runner)
  - `x86_64-apple-darwin` — cargo (macos runner)
  - `aarch64-apple-darwin` — cargo (macos runner, cross-compile)
  - `x86_64-pc-windows-msvc` — cargo (windows runner)
  - `aarch64-pc-windows-msvc` — cargo (windows-11-arm runner)
- **D-07:** Use `cross` for ALL Linux targets (pattern from starship/ripgrep/fd — engram obs #7)
- **D-08:** Use `softprops/action-gh-release@v2` for release creation
- **D-09:** Archive naming: `notebooklm-mcp-v{version}-{target}.tar.gz` (standard pattern)
- **D-10:** `fail-fast: false` on build matrix
- **D-11:** No .rpm generation (nobody does it — engram obs #7)

### Update Notification
- **D-12:** CLI command `notebooklm-mcp update-check` — compares `Cargo.toml` version vs latest GitHub release via `https://api.github.com/repos/maisonnat/notebooklm-rust-mcp/releases/latest`
- **D-13:** MCP tool `check_for_updates` — same logic, exposed to AI agents
- **D-14:** Response: "Up to date" or "New version available: vX.Y.Z (current: vA.B.C) — download: {url}"

### Platform-Specific Notes
- **D-15:** `headless_chrome` and `keyring` dependencies may need special handling in CI (browser binary for headless_chrome tests)
- **D-16:** `windows-dpapi` and `windows` crate: only relevant on Windows runners, skip on other OS
- **D-17:** `rustls-tls` (not openssl) — avoids cross-platform TLS issues in CI

### OpenCode's Discretion
- Exact clippy flags beyond `-D warnings`
- Whether to add a `cargo audit` step (nice-to-have, not blocking)
- Whether to cache `cross` Docker images

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### CI/CD Patterns
- Engram observation #7 — Rust CLI CI/CD patterns: cross for Linux ARM, native runners for macOS/Windows, softprops for releases
- Engram observation #3 — Build/deploy workflow for engram-rust (reference implementation)

### Project Files
- `Cargo.toml` — Version, dependencies, features (version source for update check)
- `src/main.rs` — CLI entry point (where to add `update-check` subcommand)
- `src/notebooklm_client.rs` — HTTP client patterns (reuse reqwest for GitHub API calls)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `reqwest` with `rustls-tls` + `json` — already in deps, perfect for GitHub API calls
- `clap` with `derive` — existing CLI framework, add `update-check` subcommand
- `serde` + `serde_json` — parse GitHub API response
- `rmcp` — MCP framework, add `check_for_updates` tool

### Established Patterns
- CLI commands follow `clap::Subcommand` derive pattern (see `src/main.rs`)
- MCP tools use `#[tool]` macro from rmcp
- HTTP calls use `reqwest::Client` with proper error handling

### Integration Points
- `main.rs` — add `UpdateCheck` variant to CLI enum
- New module `update_checker.rs` — GitHub API version check logic
- MCP server tool registration in `src/rpc/` — add `check_for_updates` tool

</code_context>

<specifics>
## Specific Ideas

- User wants to be notified when there's a new version available
- "Nuestro MCP nos avise de alguna forma que hay actualizaciones"
- Build should be fully automated on GitHub — no local build step needed for releases
- Pattern should follow what starship/ripgrep/fd do (proven, ecosystem-tested)

</specifics>

<deferred>
## Deferred Ideas

- Auto-update mechanism (download + replace binary) — future phase
- Homebrew/Scoop/AUR packaging — future phase
- GitHub Actions for automated dependency updates (Dependabot/Renovate)

</deferred>

---

*Phase: 02-github-actions-ci-cd-update-notifications*
*Context gathered: 2026-04-11*
