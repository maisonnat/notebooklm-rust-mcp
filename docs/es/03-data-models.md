---
title: "Modelos de Datos — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: es
scan_type: full
---

# Modelos de Datos

## Entidades Principales

### `Notebook`

```rust
pub struct Notebook {
    pub id: String,    // UUID (36 caracteres)
    pub title: String,
}
```

### `BrowserCredentials`

```rust
pub struct BrowserCredentials {
    pub cookie: String,  // "__Secure-1PSID=...; __Secure-1PSIDTS=..."
    pub csrf: String,    // Valor del token SNlM0e
}
```

### `SessionData` (Encriptado con DPAPI en disco)

```rust
struct SessionData {
    cookie: String,
    csrf: String,
}
```

## Tipos de Solicitud MCP

| Tipo | Campos |
|------|--------|
| `NotebookCreateRequest` | `title: String` |
| `SourceAddRequest` | `notebook_id, title, content: String` |
| `AskQuestionRequest` | `notebook_id, question: String` |

## Tipos de Error

### `NotebookLmError`

| Variante | Disparadores | Recuperacion |
|----------|-------------|-------------|
| `SessionExpired` | 401, no autorizado | Re-autenticarse |
| `CsrfExpired` | 400, prohibido | Auto-refrescar CSRF |
| `SourceNotReady` | Indexacion de fuente | Sondear disponibilidad |
| `RateLimited` | 429, demasiadas | Retroceder |
| `ParseError` | Errores JSON | Registrar y reintentar |
| `NetworkError` | Conexion/timeout | Reintentar con backoff |
| `Unknown` | Sin clasificar | Registrar e investigar |

Auto-deteccion via `from_string()` examina el texto de error buscando palabras clave de estado HTTP.

## Tipos Internos

| Tipo | Proposito |
|------|-----------|
| `ConversationMessage` | `{ question, answer: String }` |
| `ConversationHistory` | `Vec<ConversationMessage>` |
| `SourceState` | `Ready \| Processing \| Error \| Unknown` |
| `PollerConfig` | `check_interval: 2s, timeout: 60s, max_retries: 30` |
| `AuthResult` | `Success \| FallbackRequired \| Failed` |
| `AuthStatus` | `{ chrome_available, has_stored_credentials: bool }` |
| `RpcResponse` | `{ rpc_id, inner_json: Value }` |

## Almacenamiento

- **Sin base de datos** — `HashMap` en memoria + `RwLock`
- **Credenciales:** keyring del SO (primario) o archivo DPAPI en `~/.notebooklm-mcp/session.bin`
