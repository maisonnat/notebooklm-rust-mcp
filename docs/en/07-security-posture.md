---
title: "Security Posture — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: en
scan_type: full
---

# Security Posture

## Executive Summary

| Area | Status | Details |
|------|--------|---------|
| Memory Safety | **CLEAN** | Zero `unsafe` blocks |
| Supply Chain | **CLEAN** | 0 vulnerabilities (305 deps) |
| Credential Storage | **SECURE** | OS keyring + DPAPI |
| TLS | **SECURE** | rustls (no C dependencies) |
| Authentication | **MODERATE** | Reverse-engineered Google API |

## Authentication & Authorization

### Cookie Handling
- Extracted via Chrome DevTools Protocol (CDP)
- Targets `__Secure-1PSID` and `__Secure-1PSIDTS` (HttpOnly cookies)
- Never visible in JavaScript — requires browser automation
- Auto-detection of authentication completion via cookie presence check

### CSRF Token
- `SNlM0e` token extracted from NotebookLM HTML via regex
- Not static — requires refresh on 400 errors
- Backup regex pattern for alternate HTML formats

### Credential Storage

```
Primary:   OS Keyring (Windows Credential Manager / Linux Secret Service)
           Service: "notebooklm-mcp" | Entry: "google-credentials"
Fallback:  DPAPI encrypted file at ~/.notebooklm-mcp/session.bin
           Scope: CurrentUser (Windows only)
```

## Memory Safety

- **Zero `unsafe` blocks** in the entire codebase
- All array access uses defensive helpers (`get_string_at`, `get_uuid_at`)
- `cargo-audit` scan: **0 vulnerabilities** across 305 crate dependencies
- No `unwrap()` on external data (RPC responses)

## Rate Limiting & Retry

| Mechanism | Value | Purpose |
|-----------|-------|---------|
| Token bucket quota | 2s period (~30 req/min) | Prevent API abuse |
| Jitter | 150-600ms random | Avoid thundering herd |
| Max retries | 3 attempts | Resilience |
| Backoff cap | 30 seconds | Prevent infinite waits |
| Upload semaphore | 2 concurrent | Limit parallel uploads |

## Supply Chain

- All dependencies from **crates.io** (official Rust package registry)
- TLS via **rustls** (pure Rust, no OpenSSL/C dependencies)
- Chrome automation via **headless_chrome** (CDP protocol, no Selenium)
- No build scripts that download external binaries

## Known Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Google API changes | High | Defensive parsing, structured errors |
| Cookie expiration | Medium | Auto-detection, easy re-auth |
| No official API | Medium | Modular design for easy adaptation |
| In-memory state | Low | Stateless design, no PII stored |
