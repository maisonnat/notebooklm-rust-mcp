# Design: Module 6 — Anti-Detection Hardening

## Architecture Overview

The new components plug into the existing request pipeline between the tool handler and Google's batchexecute endpoint. The current flow is:

```
tool handler → batchexecute() → batchexecute_with_retry() → batchexecute_no_retry() → Google
```

After this module, the flow becomes:

```
tool handler → batchexecute()
  → batchexecute_with_retry()
    → [circuit breaker check]
    → batchexecute_no_retry()
      → [rate limiter] → [jitter 800-2000ms] → [browser headers sent via reqwest::Client]
      → Google POST
    → [parse response]
    → [detect auth error?]
      → YES → [acquire refresh lock] → refresh_tokens() → [retry once with new csrf/sid]
              → refresh failed → [increment circuit breaker counter]
      → NO → [reset circuit breaker counter]
    → [429? parse Retry-After → override backoff]
    → [apply exponential backoff (fixed 2^x + jitter 800-2000ms)]
```

All changes are confined to 4 files + 1 new module. No new crate dependencies — `AtomicU32` comes from `std::sync::atomic`, `tokio::sync::Mutex` is already available via the existing `tokio = { features = ["full"] }` in `Cargo.toml:8`.

---

## Component Design

### 1. `src/browser_headers.rs` (New Module)

**Purpose**: Centralize all Chrome-like HTTP headers in one place for easy maintenance.

```rust
// src/browser_headers.rs

use reqwest::header::{HeaderMap, HeaderValue, COOKIE, CONTENT_TYPE, USER_AGENT};

/// Build a HeaderMap with Chrome-like headers for batchexecute requests.
/// Returns headers WITHOUT Cookie and Content-Type — those are caller-specific.
pub fn browser_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    // --- User-Agent ---
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36",
        ),
    );

    // --- Sec-Fetch-* (XHR from same-origin) ---
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));

    // --- Sec-CH-UA Client Hints ---
    headers.insert(
        "sec-ch-ua",
        HeaderValue::from_static(
            "\"Chromium\";v=\"136\", \"Google Chrome\";v=\"136\", \"Not?A_Brand\";v=\"99\"",
        ),
    );
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert("sec-ch-ua-platform", HeaderValue::from_static("\"Windows\""));

    // --- Origin & Referer ---
    headers.insert("origin", HeaderValue::from_static("https://notebooklm.google.com"));
    headers.insert(
        "referer",
        HeaderValue::from_static("https://notebooklm.google.com/"),
    );

    // --- Accept ---
    headers.insert("accept", HeaderValue::from_static("*/*"));
    headers.insert(
        "accept-language",
        HeaderValue::from_static("en-US,en;q=0.9"),
    );

    // --- X-Client-Data (Chrome sends this, low entropy) ---
    headers.insert(
        "x-client-data",
        HeaderValue::from_static("CIi2yQEIpLbJAQjEtskBCKmdygEIqLPKAQ=="),
    );

    headers
}
```

**Integration point**: `NotebookLmClient::new()` at `src/notebooklm_client.rs:183`.

Currently the constructor builds headers manually:
```rust
// lines 188-190 (current)
let mut headers = header::HeaderMap::new();
headers.insert(header::COOKIE, header::HeaderValue::from_str(&cookie).unwrap());
headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/x-www-form-urlencoded;charset=utf-8"));
```

After change:
```rust
// lines 188-191 (new)
let mut headers = crate::browser_headers::browser_headers();
headers.insert(header::COOKIE, header::HeaderValue::from_str(&cookie).unwrap());
headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/x-www-form-urlencoded;charset=utf-8"));
```

Same pattern for `upload_http` at line 201 — merge browser headers then add Cookie only (no Content-Type).

**Module declaration**: Add `pub mod browser_headers;` at `src/main.rs:460` (after `auth_helper`).

**Header count**: 12 headers injected (User-Agent, 3x Sec-Fetch-*, 3x Sec-CH-UA, Origin, Referer, Accept, Accept-Language, X-Client-Data) + 2 caller-specific (Cookie, Content-Type) = 14 total.

---

### 2. Circuit Breaker

**Purpose**: Stop hammering Google with expired credentials. After 3 consecutive auth errors, open the circuit and fail fast.

**Data structure** — lives inside `NotebookLmClient`:

```rust
// New fields in NotebookLmClient struct (line 166)
pub struct NotebookLmClient {
    http: Client,
    upload_http: Client,
    csrf: String,
    sid: String,
    limiter: Limiter,
    conversation_cache: SharedConversationCache,
    upload_semaphore: Semaphore,

    // --- NEW: Module 6 circuit breaker ---
    /// Consecutive auth error count. AtomicU32 for lock-free concurrent access.
    auth_error_count: std::sync::atomic::AtomicU32,
    /// Instant when circuit opened. Used for half-open probe after 60s.
    circuit_opened_at: std::sync::Mutex<Option<std::time::Instant>>,
    /// Cookie string stored for CSRF refresh (needed by refresh flow).
    cookie: String,
    /// Mutex to serialize CSRF refresh attempts across concurrent requests.
    refresh_lock: tokio::sync::Mutex<()>,
}
```

**Constants**:
```rust
const CIRCUIT_BREAKER_THRESHOLD: u32 = 3;
const CIRCUIT_BREAKER_COOLDOWN: std::time::Duration = std::time::Duration::from_secs(60);
```

**State machine**:

```
CLOSED ──(3 auth errors)──→ OPEN ──(60s elapsed)──→ HALF-OPEN
  ↑                                                    │
  └──────────(probe succeeds)──────────────────────────┘
                                                         │
                                    (probe fails)───────→ OPEN
```

**Check logic** — called at the top of `batchexecute_with_retry()` before any HTTP call:

```rust
fn check_circuit_breaker(&self) -> Result<(), String> {
    let count = self.auth_error_count.load(std::sync::atomic::Ordering::Relaxed);

    if count >= CIRCUIT_BREAKER_THRESHOLD {
        // Check if half-open (60s elapsed since open)
        if let Ok(guard) = self.circuit_opened_at.lock() {
            if let Some(opened_at) = *guard {
                if opened_at.elapsed() < CIRCUIT_BREAKER_COOLDOWN {
                    return Err(NotebookLmError::CircuitOpen(
                        format!(
                            "Circuit breaker OPEN after {} consecutive auth errors. \
                             Run `notebooklm-mcp auth-browser` to re-authenticate. \
                             Cooldown: {}s remaining.",
                            count,
                            CIRCUIT_BREAKER_COOLDOWN.as_secs()
                                - opened_at.elapsed().as_secs()
                        ),
                    )
                    .to_string());
                }
                // 60s elapsed → half-open: allow one probe request
                // (fall through, don't return Err)
            }
        }
    }
    Ok(())
}
```

**Counter manipulation** — called from `batchexecute_with_retry()` after each attempt:

```rust
fn record_auth_success(&self) {
    self.auth_error_count.store(0, std::sync::atomic::Ordering::Relaxed);
}

fn record_auth_failure(&self) {
    let count = self.auth_error_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    if count >= CIRCUIT_BREAKER_THRESHOLD {
        // Record when circuit opened (first time crossing threshold)
        if let Ok(mut guard) = self.circuit_opened_at.lock() {
            if guard.is_none() {
                *guard = Some(std::time::Instant::now());
                tracing::warn!(
                    "Circuit breaker OPENED after {} consecutive auth errors",
                    count
                );
            }
        }
    }
}
```

**Constructor change** — `NotebookLmClient::new()` at line 183:

```rust
Self {
    http,
    upload_http,
    csrf,
    sid,
    limiter,
    conversation_cache: new_conversation_cache(),
    upload_semaphore,
    // NEW
    auth_error_count: std::sync::atomic::AtomicU32::new(0),
    circuit_opened_at: std::sync::Mutex::new(None),
    cookie,  // store for CSRF refresh
    refresh_lock: tokio::sync::Mutex::new(()),
}
```

**Signature change**: `new()` now takes `cookie: String` which it stores for refresh. The existing callers all pass cookie already — no signature change needed at call sites.

**Auth error detection** — inside `batchexecute_no_retry()` at line 302, when `!res.status().is_success()`:

```rust
if !res.status().is_success() {
    let status = res.status();
    if status == reqwest::StatusCode::UNAUTHORIZED
        || status == reqwest::StatusCode::FORBIDDEN
        || status.as_u16() == 400
    {
        // Signal auth error to caller (don't increment here — caller handles it)
        return Err(format!("AUTH_ERROR:{}", status));  // marker prefix
    }
    return Err(format!("Error HTTP {}", status));
}
```

The `AUTH_ERROR:` prefix is the contract between `batchexecute_no_retry()` and `batchexecute_with_retry()`. The retry loop checks for this prefix to decide whether to attempt CSRF refresh and whether to increment the circuit breaker counter.

---

### 3. Auto CSRF Refresh

**Purpose**: When a request fails with auth error, silently refresh CSRF+SID and retry once.

**Integration point**: Inside `batchexecute_with_retry()` at line 243.

**Flow**:

```
batchexecute_with_retry(rpc_id, payload, max_retries)
  for attempt in 0..=max_retries {
    1. check_circuit_breaker()?   ← NEW
    2. result = batchexecute_no_retry(rpc_id, payload)
    3. if Ok → record_auth_success(); return Ok(result)
    4. if Err and contains "AUTH_ERROR:" and attempt == 0:
         a. Acquire refresh_lock (tokio::sync::Mutex)
         b. Call self.refresh_csrf_internal()
         c. Retry batchexecute_no_retry() with new token (ONE more call, not a loop)
         d. If retry Ok → record_auth_success(); return Ok
         e. If retry Err → record_auth_failure(); continue normal retry loop
    5. If Err and not auth error → continue normal retry loop
    6. Apply backoff (with Retry-After support)
  }
```

**Refresh method** — new private method on `NotebookLmClient`:

```rust
/// Refresh CSRF token and Session ID using stored cookie.
/// Must be called while holding refresh_lock.
async fn refresh_csrf_internal(&self) -> Result<(String, String), String> {
    let auth_helper = crate::auth_helper::AuthHelper::new();
    let (new_csrf, new_sid) = auth_helper.refresh_tokens(&self.cookie).await?;

    // Update mutable fields via interior mutability
    // csrf and sid need to be interior-mutable for this to work
    // See "Interior Mutability" section below
    Ok((new_csrf, new_sid))
}
```

**Interior mutability problem**: `csrf` and `sid` are `String` fields. Multiple `&self` references need to update them. Solution: wrap in `tokio::sync::RwLock<String>`.

```rust
// Revised struct fields
pub struct NotebookLmClient {
    // ... existing fields ...
    csrf: tokio::sync::RwLock<String>,
    sid: tokio::sync::RwLock<String>,
    // ... new fields ...
}
```

This requires updating all read sites (currently `self.csrf.clone()` at line 283 in `batchexecute_no_retry()`) to `self.csrf.read().await.clone()`. There are ~15 references to `self.csrf` and `self.sid` across the file — all in async contexts, so `.await` is fine.

**Constructor change**:
```rust
csrf: tokio::sync::RwLock::new(csrf),
sid: tokio::sync::RwLock::new(sid),
```

**batchexecute_no_retry() update** — line 275:

```rust
let csrf = self.csrf.read().await.clone();
let sid = self.sid.read().await.clone();

let form_data = [
    ("f.req", req_array),
    ("at", csrf)  // was self.csrf.clone()
];
```

And URL construction at line 292:
```rust
if !sid.is_empty() {
    url.push_str(&format!("&source-path=/&f.sid={}", sid));
}
```

**Refresh lock interaction with circuit breaker**:

```
Request fails with AUTH_ERROR
  → acquire refresh_lock
    → refresh_tokens() succeeds
      → update csrf/sid
      → retry once
        → succeeds → record_auth_success() → counter resets to 0
        → fails → record_auth_failure() → counter increments
    → refresh_tokens() fails
      → record_auth_failure() → counter increments
      → propagate original error
```

The lock ensures that if 5 concurrent requests all get AUTH_ERROR, only 1 refresh call happens to Google. The others wait on the Mutex, then use the already-refreshed token.

**`ask_question()` at line 889** also uses `self.csrf.clone()` at line 943 — must update to `self.csrf.read().await.clone()`.

---

### 4. Backoff Fix

**Current bug** at `src/notebooklm_client.rs:230`:

```rust
let base_delay = 1u64.pow(attempt.min(6)); // ALWAYS 1 — 1^anything = 1
```

**Fix**:

```rust
let base_delay = 2u64.pow(attempt.min(6)); // 1, 2, 4, 8, 16, 32, 64 seconds
```

**Resulting delays** (base + jitter):

| attempt | base_delay (seconds) | old jitter (ms) | new jitter (ms) | old total | new total |
|---------|---------------------|-----------------|-----------------|-----------|-----------|
| 0 | 0 (skip) | — | — | 0 | 0 |
| 1 | 2 | 100-1000 | 800-2000 | ~2.5s | ~3.4s |
| 2 | 4 | 100-1000 | 800-2000 | ~4.5s | ~5.4s |
| 3 | 8 | 100-1000 | 800-2000 | ~8.5s | ~9.4s |
| 4 | 16 | 100-1000 | 800-2000 | ~16.5s | ~17.4s |
| 5 | 32 | 100-1000 | 800-2000 | ~32.5s | ~33.4s → capped 30s |
| 6 | 64 | 100-1000 | 800-2000 | ~64.5s → capped 30s | ~65.4s → capped 30s |

**Jitter increase** — `apply_jitter()` at line 262:

```rust
// Before (line 265)
rng.gen_range(150..=600)

// After
rng.gen_range(800..=2000)
```

**Backoff cap** remains at 30s (line 237) — no change needed.

**Retry-After interaction** (see Component 5): when `Retry-After` header is present, it replaces the entire `base_delay + jitter` calculation. The jitter is NOT added on top of Retry-After to respect Google's timing exactly.

---

### 5. Retry-After Support

**Purpose**: When Google returns 429 with `Retry-After`, use their delay instead of our backoff.

**Parse function** — new function in `notebooklm_client.rs`:

```rust
/// Parse Retry-After header value. Returns delay in milliseconds.
/// Supports both integer seconds ("5") and HTTP-date formats.
fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    let value = headers.get("retry-after")?.to_str().ok()?;

    // Try integer seconds first: "5" → 5000ms
    if let Ok(secs) = value.parse::<u64>() {
        let ms = secs * 1000;
        // Cap at 120 seconds to prevent excessive waits
        return Some(ms.min(120_000));
    }

    // Try HTTP-date format: "Wed, 21 Oct 2015 07:28:00 GMT"
    // Use httpdate crate or manual parsing
    // For simplicity, parse with chrono-like logic:
    if let Ok(datetime) = httpdate::parse_http_date(value) {
        let now = std::time::SystemTime::now();
        let delay = datetime.duration_since(now).unwrap_or_default();
        return Some(delay.as_millis().min(120_000) as u64);
    }

    None
}
```

**Dependency note**: `httpdate` crate is lightweight (no deps). Add to `Cargo.toml`:
```toml
httpdate = "1"
```

**Integration** — inside `batchexecute_no_retry()` at line 296, when we get the response:

```rust
let res = self.http.post(&url)
    .form(&form_data)
    .send()
    .await
    .map_err(|e| format!("HTTP request failed: {}", e))?;

let status = res.status();

// Check for 429 with Retry-Before parsing
if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
    if let Some(retry_ms) = Self::parse_retry_after(res.headers()) {
        info!("429 Rate limited. Retry-After: {}ms", retry_ms);
        // Return special error that retry loop can use
        return Err(format!("RATE_LIMITED_RETRY_AFTER:{}", retry_ms));
    }
}
```

**In the retry loop** (`batchexecute_with_retry`), when deciding delay:

```rust
// Inside the retry loop at line 252
if attempt < max_retries {
    if let Some(retry_ms) = extract_retry_after_ms(&last_error) {
        info!("Using Retry-After delay: {}ms", retry_ms);
        tokio::time::sleep(Duration::from_millis(retry_ms)).await;
    } else {
        Self::apply_exponential_backoff(attempt).await;
    }
}
```

Helper to extract:
```rust
fn extract_retry_after_ms(error: &str) -> Option<u64> {
    error.strip_prefix("RATE_LIMITED_RETRY_AFTER:")?.parse().ok()
}
```

**`ask_question()` at line 948** also does a direct POST to the streaming endpoint — it should also parse Retry-After but it doesn't go through `batchexecute_with_retry()`. This is a known limitation: streaming endpoint retries are not in scope. The streaming endpoint is less likely to 429 since it's rate-limited by the same `self.limiter`.

---

## Error Handling

### New Error Variant: `CircuitOpen`

**File**: `src/errors.rs:12` — add to `NotebookLmError` enum:

```rust
/// Circuit breaker abierto — demasiados errores de auth consecutivos.
/// El usuario debe re-autenticarse con auth-browser.
CircuitOpen(String),
```

**Display impl** — add to `fmt()` at line 43:

```rust
NotebookLmError::CircuitOpen(msg) => write!(
    f,
    "CIRCUIT BREAKER ABIERTO: {}. Ejecuta `notebooklm-mcp auth-browser` para re-autenticar.",
    msg
),
```

**`from_string()` detection** — add at line 148 (before the final `Unknown` arm):

```rust
} else if lower.contains("circuit") || lower.contains("breaker") {
    NotebookLmError::CircuitOpen(s)
}
```

**`requires_new_credentials()`** — update at line 157:

```rust
pub fn requires_new_credentials(&self) -> bool {
    matches!(
        self,
        NotebookLmError::SessionExpired(_) | NotebookLmError::CircuitOpen(_)
    )
}
```

### Error Classification for Circuit Breaker

Not all errors should increment the counter. The retry loop must classify errors:

| Error pattern | Action | Circuit breaker |
|---------------|--------|-----------------|
| `AUTH_ERROR:401` | Try CSRF refresh | Increment on refresh failure |
| `AUTH_ERROR:400` | Try CSRF refresh | Increment on refresh failure |
| `AUTH_ERROR:403` | Try CSRF refresh | Increment on refresh failure |
| `RATE_LIMITED_RETRY_AFTER:N` | Wait N ms, retry | No change |
| Network timeout / connection refused | Retry with backoff | No change |
| JSON parse error | Propagate | No change |
| `CircuitOpen` | Propagate immediately | No change (already open) |

---

## Data Flow

### Flow 1: Normal Request (Happy Path)

```
1. batchexecute("wXbhsf", payload) called
2. → batchexecute_with_retry("wXbhsf", payload, 3)
3. check_circuit_breaker() → Ok (counter=0)
4. batchexecute_no_retry("wXbhsf", payload)
   → limiter.until_ready() → apply_jitter(800-2000ms)
   → POST with 14 headers (12 browser + cookie + content-type)
   → Google returns 200 OK
5. record_auth_success() → counter stays 0
6. Return parsed response
```

### Flow 2: Auth Failure with Successful Refresh

```
1. batchexecute("wXbhsf", payload) called
2. → batchexecute_with_retry("wXbhsf", payload, 3)
3. check_circuit_breaker() → Ok (counter=0)
4. batchexecute_no_retry("wXbhsf", payload)
   → Google returns 400 Bad Request
   → Returns Err("AUTH_ERROR:400")
5. Error starts with "AUTH_ERROR:" and attempt==0
6. Acquire refresh_lock (tokio::sync::Mutex)
7. refresh_csrf_internal()
   → AuthHelper::refresh_tokens(cookie)
   → GET https://notebooklm.google.com/ with browser headers
   → Extract SNlM0e and FdrFJe from HTML
   → Returns (new_csrf, new_sid)
8. Update self.csrf and self.sid (via RwLock write)
9. Retry: batchexecute_no_retry("wXbhsf", payload)
   → Uses new csrf in form data ("at" field)
   → Uses new sid in URL (f.sid param)
   → Google returns 200 OK
10. record_auth_success() → counter stays 0
11. Release refresh_lock
12. Return parsed response
```

### Flow 3: Auth Failure with Circuit Breaker Opening

```
1. batchexecute("wXbhsf", payload) called
2. → batchexecute_with_retry("wXbhsf", payload, 3)
3. check_circuit_breaker() → Ok (counter=2, below threshold)

[Attempt 0]
4. batchexecute_no_retry() → Err("AUTH_ERROR:401")
5. Attempt CSRF refresh → refresh_tokens() fails (cookies expired)
6. record_auth_failure() → counter=3
7. circuit_opened_at = Some(Instant::now())
8. Log: "Circuit breaker OPENED after 3 consecutive auth errors"

[Attempt 1]
9. check_circuit_breaker() → counter=3, opened_at=2s ago (< 60s)
10. Return Err(NotebookLmError::CircuitOpen(...))
11. User sees: "CIRCUIT BREAKER ABIERTO: ... Ejecuta `notebooklm-mcp auth-browser`"

... 60 seconds later ...

12. New request comes in
13. check_circuit_breaker() → counter=3, opened_at=65s ago (> 60s)
14. Half-open: allow ONE probe request
15. batchexecute_no_retry() → still fails
16. record_auth_failure() → counter=4 (stays open)
17. Circuit remains open

OR:

15. batchexecute_no_retry() → succeeds (user ran auth-browser in between)
16. record_auth_success() → counter=0
17. circuit_opened_at = None
18. Circuit CLOSED, normal operation resumes
```

---

## File Change Summary

| File | Change | Lines affected |
|------|--------|----------------|
| `src/browser_headers.rs` | **NEW** — `browser_headers()` function | ~60 lines |
| `src/main.rs:460` | Add `pub mod browser_headers;` | 1 line |
| `src/notebooklm_client.rs:166` | Add 5 new fields to struct | +8 lines |
| `src/notebooklm_client.rs:183` | Update constructor (headers + new fields) | ~15 lines modified |
| `src/notebooklm_client.rs:224` | Fix backoff `1^x` → `2^x` | 1 line |
| `src/notebooklm_client.rs:262` | Increase jitter 150-600 → 800-2000 | 1 line |
| `src/notebooklm_client.rs:243` | Rewrite `batchexecute_with_retry()` | ~40 lines rewritten |
| `src/notebooklm_client.rs:275` | Update `batchexecute_no_retry()` (auth detection, Retry-After) | ~20 lines modified |
| `src/notebooklm_client.rs` | New methods: `check_circuit_breaker`, `record_auth_*`, `refresh_csrf_internal`, `parse_retry_after` | ~60 lines |
| `src/notebooklm_client.rs` | Update `self.csrf`/`self.sid` reads to `.read().await` | ~15 sites |
| `src/notebooklm_client.rs:889` | Update `ask_question()` csrf read | 1 line |
| `src/errors.rs:12` | Add `CircuitOpen` variant | +2 lines |
| `src/errors.rs:43` | Add Display arm | +4 lines |
| `src/errors.rs:148` | Add `from_string()` detection | +2 lines |
| `src/errors.rs:157` | Update `requires_new_credentials()` | +1 line |
| `src/auth_helper.rs:93` | No signature change (already returns `(String, String)`) | 0 lines |
| `Cargo.toml` | Add `httpdate = "1"` | +1 line |

---

## Testing Strategy

### Unit Tests — Backoff Fix (`src/notebooklm_client.rs` tests module)

```rust
#[test]
fn test_exponential_backoff_grows_exponentially() {
    // Verify that attempt 1 < attempt 2 < attempt 3
    // We can't easily test timing, but we can test the base_delay formula
    assert_eq!(2u64.pow(1.min(6)), 2);   // attempt 1 → 2s base
    assert_eq!(2u64.pow(2.min(6)), 4);   // attempt 2 → 4s base
    assert_eq!(2u64.pow(3.min(6)), 8);   // attempt 3 → 8s base
    assert_eq!(2u64.pow(6.min(6)), 64);  // attempt 6 → 64s base (cap)
    assert_eq!(2u64.pow(7.min(6)), 64);  // attempt 7 → still 64s (cap)
}
```

### Unit Tests — Circuit Breaker State Transitions

```rust
#[tokio::test]
async fn test_circuit_breaker_closed_below_threshold() {
    let client = make_test_client();
    client.record_auth_failure();
    client.record_auth_failure();
    assert_eq!(client.auth_error_count.load(Ordering::Relaxed), 2);
    assert!(client.check_circuit_breaker().is_ok());
}

#[tokio::test]
async fn test_circuit_breaker_opens_at_threshold() {
    let client = make_test_client();
    client.record_auth_failure();
    client.record_auth_failure();
    client.record_auth_failure();
    assert_eq!(client.auth_error_count.load(Ordering::Relaxed), 3);
    assert!(client.check_circuit_breaker().is_err());
}

#[tokio::test]
async fn test_circuit_breaker_resets_on_success() {
    let client = make_test_client();
    client.record_auth_failure();
    client.record_auth_failure();
    client.record_auth_success();
    assert_eq!(client.auth_error_count.load(Ordering::Relaxed), 0);
    assert!(client.check_circuit_breaker().is_ok());
}
```

### Unit Tests — Retry-After Parsing

```rust
#[test]
fn test_parse_retry_after_seconds() {
    let mut headers = HeaderMap::new();
    headers.insert("retry-after", HeaderValue::from_static("5"));
    assert_eq!(parse_retry_after(&headers), Some(5000));
}

#[test]
fn test_parse_retry_after_missing() {
    let headers = HeaderMap::new();
    assert_eq!(parse_retry_after(&headers), None);
}

#[test]
fn test_parse_retry_after_capped_at_120s() {
    let mut headers = HeaderMap::new();
    headers.insert("retry-after", HeaderValue::from_static("999"));
    assert_eq!(parse_retry_after(&headers), Some(120_000));
}

#[test]
fn test_parse_retry_after_invalid() {
    let mut headers = HeaderMap::new();
    headers.insert("retry-after", HeaderValue::from_static("not-a-number"));
    assert_eq!(parse_retry_after(&headers), None);
}
```

### Unit Tests — New Error Variant

```rust
#[test]
fn test_circuit_open_display() {
    let err = NotebookLmError::CircuitOpen("3 errors".to_string());
    assert!(err.to_string().contains("CIRCUIT BREAKER ABIERTO"));
    assert!(err.to_string().contains("auth-browser"));
}

#[test]
fn test_circuit_open_requires_new_credentials() {
    let err = NotebookLmError::CircuitOpen("test".to_string());
    assert!(err.requires_new_credentials());
}

#[test]
fn test_from_string_detects_circuit_open() {
    let err = NotebookLmError::from_string("Circuit breaker is open".to_string());
    assert!(matches!(err, NotebookLmError::CircuitOpen(_)));
}
```

### Unit Tests — Browser Headers

```rust
#[test]
fn test_browser_headers_has_user_agent() {
    let headers = crate::browser_headers::browser_headers();
    let ua = headers.get("user-agent").unwrap().to_str().unwrap();
    assert!(ua.contains("Chrome/"));
    assert!(ua.contains("Mozilla/5.0"));
}

#[test]
fn test_browser_headers_has_sec_fetch() {
    let headers = crate::browser_headers::browser_headers();
    assert_eq!(headers.get("sec-fetch-dest").unwrap(), "empty");
    assert_eq!(headers.get("sec-fetch-mode").unwrap(), "cors");
    assert_eq!(headers.get("sec-fetch-site").unwrap(), "same-origin");
}

#[test]
fn test_browser_headers_has_origin_referer() {
    let headers = crate::browser_headers::browser_headers();
    assert_eq!(headers.get("origin").unwrap(), "https://notebooklm.google.com");
    assert_eq!(headers.get("referer").unwrap(), "https://notebooklm.google.com/");
}

#[test]
fn test_browser_headers_no_cookie_or_content_type() {
    let headers = crate::browser_headers::browser_headers();
    assert!(headers.get("cookie").is_none());
    assert!(headers.get("content-type").is_none());
}
```

### Integration Test — Auto-Refresh Flow

```rust
#[tokio::test]
async fn test_auto_csrf_refresh_on_auth_error() {
    // Uses a mock HTTP server (wiremock or manual TcpListener) that:
    // 1st request: returns 400 Bad Request (trigger auth error)
    // 2nd request (refresh GET to /): returns HTML with SNlM0e and FdrFJe
    // 3rd request: returns 200 OK with valid batchexecute response
    //
    // Verifies:
    // - Only 1 refresh call made
    // - Final response is the successful 200 response
    // - Circuit breaker counter is 0 after success
}
```

This test requires either `wiremock` (new dev-dependency) or the manual `TcpListener` pattern already used in `src/notebooklm_client.rs:2178`. Prefer the manual pattern for consistency — no new dependency.

---

## `make_test_client()` Update

The existing helper at line 2128 must initialize the new fields:

```rust
fn make_test_client() -> NotebookLmClient {
    NotebookLmClient::new("test_cookie=1".to_string(), "test_csrf".to_string(), String::new())
    // new() already handles all field initialization
}
```

Since all new fields have defaults (AtomicU32::new(0), Mutex::new(None), etc.), no change needed here.
