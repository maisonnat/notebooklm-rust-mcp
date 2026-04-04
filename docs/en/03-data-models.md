---
title: "Data Models — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: en
scan_type: full
---

# Data Models

## Core Entities

### `Notebook`

```rust
pub struct Notebook {
    pub id: String,    // UUID (36 chars)
    pub title: String,
}
```

### `BrowserCredentials`

```rust
pub struct BrowserCredentials {
    pub cookie: String,  // "__Secure-1PSID=...; __Secure-1PSIDTS=..."
    pub csrf: String,    // SNlM0e token value
}
```

### `SessionData` (DPAPI-encrypted on disk)

```rust
struct SessionData {
    cookie: String,
    csrf: String,
}
```

## MCP Request Types

| Type | Fields |
|------|--------|
| `NotebookCreateRequest` | `title: String` |
| `SourceAddRequest` | `notebook_id, title, content: String` |
| `AskQuestionRequest` | `notebook_id, question: String` |

## Error Types

### `NotebookLmError`

| Variant | Triggers | Recovery |
|---------|----------|----------|
| `SessionExpired` | 401, unauthorized | Re-authenticate |
| `CsrfExpired` | 400, forbidden | Auto-refresh CSRF |
| `SourceNotReady` | Source indexing | Poll for readiness |
| `RateLimited` | 429, too many | Back off |
| `ParseError` | JSON errors | Log and retry |
| `NetworkError` | Connection/timeout | Retry with backoff |
| `Unknown` | Unclassified | Log and investigate |

Auto-detection via `from_string()` examines error text for HTTP status keywords.

## Internal Types

| Type | Purpose |
|------|---------|
| `ConversationMessage` | `{ question, answer: String }` |
| `ConversationHistory` | `Vec<ConversationMessage>` |
| `SourceState` | `Ready \| Processing \| Error \| Unknown` |
| `PollerConfig` | `check_interval: 2s, timeout: 60s, max_retries: 30` |
| `AuthResult` | `Success \| FallbackRequired \| Failed` |
| `AuthStatus` | `{ chrome_available, has_stored_credentials: bool }` |
| `RpcResponse` | `{ rpc_id, inner_json: Value }` |

## Storage

- **No database** — in-memory `HashMap` + `RwLock`
- **Credentials:** OS keyring (primary) or DPAPI file at `~/.notebooklm-mcp/session.bin`
