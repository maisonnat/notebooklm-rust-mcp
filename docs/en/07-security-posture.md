---
title: "Security Posture — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.3.1"
last_updated: "2026-04-06"
lang: en
scan_type: full
---

# Security Posture

## Executive Summary

| Area | Status | Details |
|------|--------|---------|
| Memory Safety | **CLEAN** | Zero `unsafe` blocks |
| Supply Chain | **CLEAN** | 0 vulnerabilities (335 deps) |
| Credential Storage | **SECURE** | OS keyring + DPAPI fallback |
| TLS | **SECURE** | rustls (no C dependencies) |
| Authentication | **MODERATE** | Reverse-engineered Google API |
| Anti-Detection | **HARDENED** | Chrome-like headers, circuit breaker, auto-refresh |

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
- **Module 6**: Auto-refresh on auth error — silent CSRF+SID refresh before user notices

### Credential Storage

```
Primary:   OS Keyring (Windows Credential Manager / macOS Keychain / Linux Secret Service)
           Service: "notebooklm-mcp" | Entry: "google-credentials"
Fallback:  DPAPI encrypted file (Windows only)
```

> Credentials are **never** stored in environment variables, config files, or logs.

## Anti-Detection Hardening (Module 6)

### Browser Fingerprint Spoofing

All requests to Google's `batchexecute` endpoint include Chrome-like HTTP headers to avoid WAF detection:

| Header | Value | Purpose |
|--------|-------|---------|
| `User-Agent` | Chrome/136 on Windows | Browser identification |
| `sec-fetch-dest` | `empty` | XHR request type |
| `sec-fetch-mode` | `cors` | CORS mode |
| `sec-fetch-site` | `same-origin` | Origin check |
| `sec-ch-ua` | Chromium;v=136 | Client hint |
| `sec-ch-ua-mobile` | `?0` | Desktop mode |
| `sec-ch-ua-platform` | "Windows" | OS hint |
| `origin` | https://notebooklm.google.com | Origin header |
| `referer` | https://notebooklm.google.com/ | Referer header |
| `accept` | `*/*` | Content type |
| `accept-language` | en-US,en;q=0.9 | Language preference |

### Circuit Breaker

Prevents hammering Google with expired credentials:

```
CLOSED ──(3 auth errors)──→ OPEN ──(60s cooldown)──→ HALF-OPEN
  ↑                                                    │
  └──────────(probe succeeds)──────────────────────────┘
                                                          │
                                     (probe fails)───────→ OPEN
```

- **Threshold**: 3 consecutive auth errors (401/400/403)
- **Cooldown**: 60 seconds before allowing a probe request
- **User action**: "Run `notebooklm-mcp auth-browser` to re-authenticate"
- **Implementation**: `AtomicU32` for lock-free counter, `Mutex<Option<Instant>>` for timestamp

### Auto CSRF Refresh

When an auth error is detected, the server automatically:

1. Acquires a `tokio::sync::Mutex` lock (prevents concurrent refreshes)
2. Calls `refresh_tokens()` to get a new CSRF + Session ID from Google
3. Retries the original request exactly once with the new tokens
4. If refresh fails → increments circuit breaker counter → propagates error

### Retry-After Support

When Google returns HTTP 429 with a `Retry-After` header:

- Parses integer seconds (`"5"` → 5000ms) and HTTP-date formats
- Uses server-specified delay instead of calculated backoff
- Caps at 120 seconds to prevent excessive waits
- Falls back to exponential backoff if header is absent

## Rate Limiting & Retry

| Mechanism | Value | Purpose |
|-----------|-------|---------|
| Token bucket quota | 2s period (~30 req/min) | Prevent API abuse |
| Pre-request jitter | 800-2000ms | Simulate human timing |
| Exponential backoff | 2^x seconds (2, 4, 8, 16...) | Retry spacing |
| Backoff jitter | 800-2000ms | Avoid thundering herd |
| Max retries | 3 attempts | Resilience |
| Backoff cap | 30 seconds | Prevent infinite waits |
| Retry-After | Server-specified (up to 120s) | Respect Google's guidance |

## Memory Safety

- **Zero `unsafe` blocks** in the entire codebase
- All array access uses defensive helpers (`get_string_at`, `get_uuid_at`)
- `cargo-audit` scan: **0 vulnerabilities** across 335 crate dependencies
- No `unwrap()` on external data (RPC responses)

## Sensitive Data Handling

| Data | Handling | Storage |
|------|----------|---------|
| Google cookies | Extracted via CDP, never logged | OS keyring |
| CSRF token | Extracted from HTML, auto-refreshed on expiry | OS keyring |
| Session ID | Extracted from HTML, auto-refreshed on expiry | OS keyring |
| Notebook content | Processed via RPC, never stored locally | Google's servers |
| User queries | Sent to Google RPC, not logged | Not persisted |

## Supply Chain

- All dependencies from **crates.io** (official Rust package registry)
- TLS via **rustls** (pure Rust, no OpenSSL/C dependencies)
- Chrome automation via **headless_chrome** (CDP protocol, no Selenium)
- No build scripts that download external binaries
- **httpdate** crate for Retry-After date parsing (zero dependencies)

## Download Security

Artifact downloads validate:

- **Domain whitelist**: Only `googleapis.com` and `googleusercontent.com`
- **Scheme enforcement**: HTTPS only (no HTTP downloads)
- **Streaming**: No temporary file writes for in-memory content

## Known Risks

| Risk | Severity | Mitigation |
|------|----------|------------|
| Google API changes | High | Defensive parsing, structured errors, modular RPC layer |
| Cookie expiration | Medium | Auto-detection, auto-refresh, easy re-auth via `auth-browser` |
| No official API | Medium | Modular design for easy adaptation to changes |
| Reverse-engineered protocol | Medium | All parsing is defensive — unexpected formats return errors |
| WAF detection | Low | Chrome-like headers, human jitter, circuit breaker |

> **[Español](../es/07-security-posture.md)** · **[Português](../pt/07-security-posture.md)**
