---
title: "Overview — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
last_commit: "b467e15"
scan_type: full
tags: [rust, mcp, documentation]
audience: both
---

# Modelos de Dados (Português)

## Estruturas do Cliente

### NotebookLmClient

```rust
pub struct NotebookLmClient {
    http: Client,                    // reqwest HTTP client
    csrf: String,                    // Token CSRF
    limiter: Limiter,                // Rate limiter (governor)
    conversation_cache: SharedConversationCache,
    upload_semaphore: Semaphore,     // Max 2 uploads simultâneos
}
```

### Notebook

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: String,      // UUID de 36 caracteres
    pub title: String,  // Título do caderno
}
```

Exemplo:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "Notas de Reunião Q1"
}
```

## Autenticação

### BrowserCredentials

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserCredentials {
    pub cookie: String,   // "__Secure-1PSID=...; __Secure-1PSIDTS=..."
    pub csrf: String,    // Token CSRF (SNlM0e)
}
```

### AuthResult

```rust
pub enum AuthResult {
    Success(BrowserCredentials),
    FallbackRequired(String),  // Chrome não disponível
    Failed(String),            // Erro de autenticação
}
```

### AuthStatus

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub chrome_available: bool,           // Chrome instalado
    pub has_stored_credentials: bool,     // Credenciais salvas
}
```

## Erros

### NotebookLmError

```rust
pub enum NotebookLmError {
    SessionExpired(String),    // Sessão do Google expirou
    CsrfExpired(String),       // Token CSRF inválido
    SourceNotReady(String),    // Fonte em processamento
    RateLimited(String),       // Rate limit atingido
    ParseError(String),        // Erro de parsing
    NetworkError(String),      // Erro de rede
    Unknown(String),           // Erro genérico
}
```

## Parser

### RpcResponse

```rust
pub struct RpcResponse {
    pub rpc_id: String,      // ID do método RPC (ex., "wXbhsf")
    pub inner_json: Value,   // Payload parseado
}
```

## Polling

### PollerConfig

```rust
#[derive(Debug, Clone)]
pub struct PollerConfig {
    pub check_interval: Duration,  // Intervalo entre verificações (default: 2s)
    pub timeout: Duration,       // Timeout total (default: 60s)
    pub max_retries: usize,       // Max tentativas (default: 30)
}
```

### SourceState

```rust
pub enum SourceState {
    Ready,        // Fonte indexada e pronta
    Processing,  // Ainda processando
    Error(String), // Erro no processamento
    Unknown,      // Estado não reconhecido
}
```

## Cache

### ConversationHistory

```rust
pub struct ConversationHistory {
    pub messages: Vec<ConversationMessage>,
}

pub struct ConversationMessage {
    pub question: String,
    pub answer: String,
}
```

### ConversationCache

```rust
pub struct ConversationCache {
    conversations: RwLock<HashMap<String, (String, ConversationHistory)>>,
    // notebook_id -> (conversation_id, histórico)
}
```

## Request/Response DTOs

### NotebookCreateRequest

```rust
pub struct NotebookCreateRequest {
    pub title: String,
}
```

### SourceAddRequest

```rust
pub struct SourceAddRequest {
    pub notebook_id: String,
    pub title: String,
    pub content: String,
}
```

### AskQuestionRequest

```rust
pub struct AskQuestionRequest {
    pub notebook_id: String,
    pub question: String,
}
```

## Sessão

### SessionData

```rust
#[derive(Serialize, Deserialize)]
struct SessionData {
    cookie: String,   // Cookies encriptadas
    csrf: String,     // Token CSRF
}
```

Localização: `~/.notebooklm-mcp/session.bin` (encriptado com DPAPI)