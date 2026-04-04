# Modelos de Datos

## Estructuras del Cliente

### NotebookLmClient

```rust
pub struct NotebookLmClient {
    http: Client,                    // reqwest HTTP client
    csrf: String,                    // Token CSRF
    limiter: Limiter,                // Rate limiter (governor)
    conversation_cache: SharedConversationCache,
    upload_semaphore: Semaphore,     // Max 2 uploads simultáneos
}
```

### Notebook

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: String,      // UUID de 36 caracteres
    pub title: String,  // Título de la libreta
}
```

Ejemplo:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "Notas de Reunión Q1"
}
```

## Autenticación

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
    FallbackRequired(String),  // Chrome no disponible
    Failed(String),            // Error de autenticación
}
```

### AuthStatus

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    pub chrome_available: bool,           // Chrome instalado
    pub has_stored_credentials: bool,     // Credenciales guardadas
}
```

## Errores

### NotebookLmError

```rust
pub enum NotebookLmError {
    SessionExpired(String),    // Sesión de Google expiró
    CsrfExpired(String),       // Token CSRF inválido
    SourceNotReady(String),    // Fuente en procesamiento
    RateLimited(String),       // Rate limit alcanzado
    ParseError(String),        // Error de parseo
    NetworkError(String),      // Error de red
    Unknown(String),           // Error genérico
}
```

## Parser

### RpcResponse

```rust
pub struct RpcResponse {
    pub rpc_id: String,      // ID del método RPC (e.g., "wXbhsf")
    pub inner_json: Value,   // Payload parseado
}
```

## Polling

### PollerConfig

```rust
#[derive(Debug, Clone)]
pub struct PollerConfig {
    pub check_interval: Duration,  // Intervalo entre checks (default: 2s)
    pub timeout: Duration,       // Timeout total (default: 60s)
    pub max_retries: usize,       // Max attempts (default: 30)
}
```

### SourceState

```rust
pub enum SourceState {
    Ready,        // Fuente indexada y lista
    Processing,  // Aún procesándose
    Error(String), // Error en procesamiento
    Unknown,      // Estado no reconocido
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
    // notebook_id -> (conversation_id, historial)
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

## Sesión

### SessionData

```rust
#[derive(Serialize, Deserialize)]
struct SessionData {
    cookie: String,   // Cookies encriptadas
    csrf: String,     // Token CSRF
}
```

Ubicación: `~/.notebooklm-mcp/session.bin` (encriptado con DPAPI)
