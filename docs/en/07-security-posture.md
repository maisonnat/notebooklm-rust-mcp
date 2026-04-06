---
title: "Security Posture — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: en
scan_type: full
---

# Security Posture

## Executive Summary

| Area | Status | Details |
|------|--------|---------|
| Memory Safety | **CLEAN** | Zero `unsafe` blocks |
| Supply Chain | **CLEAN** | 0 vulnerabilities (334 deps) |
| Credential Storage | **SECURE** | OS keyring + DPAPI fallback |
| TLS | **SECURE** | rustls (no C dependencies) |
| Authentication | **MODERATE** | Reverse-engineered Google API |

## Authentication & Authorization

### Cookie Handling

- Extracted via Chrome DevTools Protocol (CDP)
- Targets `__Secure-1PSID` and `__Secure-1PSIDTS` (HttpOnly cookies)
- Never visible in JavaScript — requires browser automation
- **Critical fix**: CDP `Network.getCookies` returns name+value but NOT SameSite/Secure/Path/Domain attributes. Google's server rejects requests without proper cookie attributes. Solution: direct header injection from CDP cookies with attribute reconstruction.

### CSRF Token

- `SNlM0e` token extracted from NotebookLM HTML via regex
- Not static — requires refresh on 400 errors
- Expiry detection built into error handling

### Credential Storage

```
Primary:   OS Keyring (Windows Credential Manager / macOS Keychain / Linux Secret Service)
           Service: "notebooklm-mcp" | Entry: "google-credentials"
Fallback:  DPAPI encrypted file (Windows only)
```

> Credentials are **never** stored in environment variables, config files, or logs.

## Memory Safety

- **Zero `unsafe` blocks** in the entire codebase
- All array access uses defensive helpers (`get_string_at`, `get_uuid_at`)
- `cargo-audit` scan: **0 vulnerabilities** across 334 crate dependencies
- No `unwrap()` on external data (RPC responses)

## Sensitive Data Handling

| Data | Handling | Storage |
|------|----------|---------|
| Google cookies | Extracted via CDP, never logged | OS keyring |
| CSRF token | Extracted from HTML, validated per request | OS keyring |
| Session ID | Extracted from HTML | OS keyring |
| Notebook content | Processed via RPC, never stored locally | Google's servers |
| User queries | Sent to Google RPC, not logged | Not persisted |

## Rate Limiting & Retry

| Mechanism | Value | Purpose |
|-----------|-------|---------|
| Token bucket quota | 2s period (~30 req/min) | Prevent API abuse |
| Exponential backoff | Jitter 150-600ms | Avoid thundering herd |
| Max retries | 3 attempts | Resilience |
| Backoff cap | 30 seconds | Prevent infinite waits |

## Supply Chain

- All dependencies from **crates.io** (official Rust package registry)
- TLS via **rustls** (pure Rust, no OpenSSL/C dependencies)
- Chrome automation via **headless_chrome** (CDP protocol, no Selenium)
- No build scripts that download external binaries

## Download Security

Artifact downloads validate:

- **Domain whitelist**: Only `googleapis.com` and `googleusercontent.com`
- **Scheme enforcement**: HTTPS only (no HTTP downloads)
- **Streaming**: No temporary file writes for in-memory content

## Known Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Google API changes | High | Defensive parsing, structured errors, modular RPC layer |
| Cookie expiration | Medium | Auto-detection, easy re-auth via `auth-browser` |
| No official API | Medium | Modular design for easy adaptation to changes |
| Reverse-engineered protocol | Medium | All parsing is defensive — unexpected formats return errors |

> **[Español](../es/07-security-posture.md)** · **[Português](../pt/07-security-posture.md)**
