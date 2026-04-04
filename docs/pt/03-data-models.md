---
title: "Modelos de Dados — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: pt
scan_type: full
---

# Modelos de Dados

## Entidades Principais

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
    pub csrf: String,    // Valor do token SNlM0e
}
```

### `SessionData` (criptografado com DPAPI em disco)

```rust
struct SessionData {
    cookie: String,
    csrf: String,
}
```

## Tipos de Requisicao MCP

| Tipo | Campos |
|------|--------|
| `NotebookCreateRequest` | `title: String` |
| `SourceAddRequest` | `notebook_id, title, content: String` |
| `AskQuestionRequest` | `notebook_id, question: String` |

## Tipos de Erro

### `NotebookLmError`

| Variante | Dispara | Recuperacao |
|----------|---------|-------------|
| `SessionExpired` | 401, nao autorizado | Reautenticar |
| `CsrfExpired` | 400, proibido | Atualizar CSRF automaticamente |
| `SourceNotReady` | Indexacao de fonte | Fazer polling para verificar prontidao |
| `RateLimited` | 429, excesso de requisicoes | Reduzir o ritmo |
| `ParseError` | Erros de JSON | Registrar log e retentar |
| `NetworkError` | Conexao/timeout | Retentar com backoff |
| `Unknown` | Nao classificado | Registrar log e investigar |

Deteccao automatica via `from_string()` examina o texto de erro em busca de palavras-chave de status HTTP.

## Tipos Internos

| Tipo | Finalidade |
|------|------------|
| `ConversationMessage` | `{ question, answer: String }` |
| `ConversationHistory` | `Vec<ConversationMessage>` |
| `SourceState` | `Ready \| Processing \| Error \| Unknown` |
| `PollerConfig` | `check_interval: 2s, timeout: 60s, max_retries: 30` |
| `AuthResult` | `Success \| FallbackRequired \| Failed` |
| `AuthStatus` | `{ chrome_available, has_stored_credentials: bool }` |
| `RpcResponse` | `{ rpc_id, inner_json: Value }` |

## Armazenamento

- **Sem banco de dados** — `HashMap` em memoria + `RwLock`
- **Credenciais:** Keyring do SO (primario) ou arquivo DPAPI em `~/.notebooklm-mcp/session.bin`
