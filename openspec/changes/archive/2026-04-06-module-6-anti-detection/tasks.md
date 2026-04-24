# Tasks: Module 6 — Anti-Detection Hardening

## Phase 1: Infrastructure (browser_headers.rs + dependencies)
- [x] 1.1: Create `src/browser_headers.rs` with `browser_headers() -> HeaderMap` function (12 Chrome-like headers: User-Agent, 3x Sec-Fetch-*, 3x Sec-CH-UA, Origin, Referer, Accept, Accept-Language, X-Client-Data)
- [x] 1.2: Add `pub mod browser_headers;` to `src/main.rs:460` (after `auth_helper`)
- [x] 1.3: Add `httpdate = "1"` to `Cargo.toml` dependencies
- [x] 1.4: Write 4 unit tests for browser_headers (User-Agent contains Chrome, Sec-Fetch-* values, Origin/Referer, no Cookie/Content-Type)

## Phase 2: Struct Changes (RwLock + new fields)
- [x] 2.1: Change `csrf: String` → `csrf: tokio::sync::RwLock<String>` in `NotebookLmClient` struct at `src/notebooklm_client.rs:166`
- [x] 2.2: Change `sid: String` → `sid: tokio::sync::RwLock<String>` in `NotebookLmClient` struct at `src/notebooklm_client.rs:166`
- [x] 2.3: Add 4 new fields to struct: `auth_error_count: AtomicU32`, `circuit_opened_at: Mutex<Option<Instant>>`, `cookie: String`, `refresh_lock: tokio::sync::Mutex<()>`
- [x] 2.4: Update constructor `NotebookLmClient::new()` at line 183 — wrap csrf/sid in RwLock, initialize all new fields (AtomicU32::new(0), Mutex::new(None), store cookie, Mutex::new(()))
- [x] 2.5: Update all ~15 `self.csrf` / `self.sid` read sites to `.read().await` (batchexecute_no_retry at line 275, ask_question at line 943, and ~13 other references)
- [x] 2.6: Run `cargo check` to verify all RwLock migrations compile

## Phase 3: Headers Integration
- [x] 3.1: Replace manual header construction in `NotebookLmClient::new()` at lines 188-190 with `crate::browser_headers::browser_headers()` + Cookie + Content-Type
- [x] 3.2: Replace manual header construction in `upload_http` at line 201 with `browser_headers()` + Cookie (no Content-Type)
- [x] 3.3: Run `cargo check` to verify headers compile

## Phase 4: Error Handling
- [x] 4.1: Add `CircuitOpen(String)` variant to `NotebookLmError` enum at `src/errors.rs:12`
- [x] 4.2: Add Display arm for CircuitOpen at `src/errors.rs:43` (Spanish message mentioning `auth-browser`)
- [x] 4.3: Add `from_string()` detection for circuit/breaker keywords at `src/errors.rs:148` (before the final `Unknown` arm)
- [x] 4.4: Update `requires_new_credentials()` at `src/errors.rs:157` to include `CircuitOpen(_)`
- [x] 4.5: Write 3 unit tests for CircuitOpen error variant (Display contains "CIRCUIT BREAKER ABIERTO", requires_new_credentials returns true, from_string detects circuit/breaker keywords)

## Phase 5: Circuit Breaker
- [x] 5.1: Add constants `CIRCUIT_BREAKER_THRESHOLD: u32 = 3` and `CIRCUIT_BREAKER_COOLDOWN = Duration::from_secs(60)` in `src/notebooklm_client.rs`
- [x] 5.2: Implement `check_circuit_breaker(&self) -> Result<(), String>` method (closed → open at threshold → half-open after 60s)
- [x] 5.3: Implement `record_auth_success(&self)` (resets counter to 0) and `record_auth_failure(&self)` (increments counter, sets circuit_opened_at on threshold crossing) methods
- [x] 5.4: Write 3 unit tests for circuit breaker state transitions (closed below threshold, opens at threshold, resets on success)

## Phase 6: Auto CSRF Refresh
- [x] 6.1: Implement `refresh_csrf_internal(&self) -> Result<(String, String), String>` private method using `AuthHelper::refresh_tokens(&self.cookie).await`
- [x] 6.2: Add `AUTH_ERROR:` prefix detection in `batchexecute_no_retry()` at line 302 for 400/401/403 status codes (return `Err(format!("AUTH_ERROR:{}", status))`)
- [x] 6.3: Rewrite `batchexecute_with_retry()` loop at line 243: check_circuit_breaker → attempt → detect AUTH_ERROR prefix on attempt==0 → acquire refresh_lock → refresh_csrf_internal → write new csrf/sid via RwLock → retry once → record success/failure

## Phase 7: Backoff Fix + Retry-After
- [x] 7.1: Fix exponential backoff at `src/notebooklm_client.rs:230`: `1u64.pow()` → `2u64.pow()` (currently always returns 1)
- [x] 7.2: Increase jitter range at line 265: `rng.gen_range(150..=600)` → `rng.gen_range(800..=2000)`
- [x] 7.3: Implement `parse_retry_after(headers: &HeaderMap) -> Option<u64>` function (integer seconds + HTTP-date via httpdate crate, capped at 120s)
- [x] 7.4: Add `RATE_LIMITED_RETRY_AFTER:N` error pattern in `batchexecute_no_retry()` for 429 responses at line 296
- [x] 7.5: Implement `extract_retry_after_ms(error: &str) -> Option<u64>` helper; update retry loop at line 252 to use Retry-After delay when present, fallback to exponential backoff
- [x] 7.6: Write 4 unit tests for parse_retry_after (integer seconds "5" → 5000ms, missing header → None, capped at 120_000 for "999", invalid "not-a-number" → None)
- [x] 7.7: Write 1 unit test for exponential backoff formula (2^1=2, 2^2=4, 2^3=8, 2^6=64, 2^7 capped at 64)

## Phase 8: Validation
- [x] 8.1: Run `cargo clippy` — 0 warnings
- [x] 8.2: Run `cargo test` — all pass (existing + new ~15 tests)
- [x] 8.3: Verify `make_test_client()` at line 2128 still works with new fields (no change needed since defaults are in constructor)

## Phase 9: Documentation
- [x] 9.1: Update `docs/en/02-api-reference.md` (note anti-detection improvements: browser headers, circuit breaker, auto CSRF refresh, Retry-After)
- [x] 9.2: Update `docs/en/06-changelog.md` with v0.3.1 entry (anti-detection hardening)
- [x] 9.3: Translate doc updates to ES (`docs/es/`) and PT (`docs/pt/`)
