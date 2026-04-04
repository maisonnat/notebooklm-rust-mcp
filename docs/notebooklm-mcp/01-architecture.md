# Arquitectura de NotebookLM MCP Server

## Visión General

```
┌─────────────────────────────────────────────────────────────────┐
│                      Cliente MCP                                │
│  (Cursor, Windsurf, Claude Desktop, etc.)                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ stdio
┌─────────────────────────────────────────────────────────────────┐
│                    notebooklm-mcp                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ CLI Handler  │  │ MCP Server  │  │ Session Manager     │  │
│  │ (clap)       │  │ (rmcp)      │  │ (DPAPI/Keyring)     │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                 NotebookLmClient                          │  │
│  │  ┌─────────────┐  ┌──────────────┐  ┌─────────────────┐  │  │
│  │  │ Rate Limit  │  │ Parser RPC   │  │ Conversation    │  │  │
│  │  │ (governor)  │  │ (defensivo)  │  │ Cache           │  │  │
│  │  └─────────────┘  └──────────────┘  └─────────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ HTTPS
┌─────────────────────────────────────────────────────────────────┐
│              Google NotebookLM API (internal)                  │
│  - batchexecute (RPC)                                           │
│  - GenerateFreeFormStreamed (streaming)                        │
└─────────────────────────────────────────────────────────────────┘
```

## Módulos

### `main.rs` — Punto de entrada

- **CLI**: Parsea comandos con `clap`
- **MCP Server**: Servidor rmcp con tools y resources
- **Session Management**: Carga/guarda credenciales encriptadas

### `notebooklm_client.rs` — Cliente HTTP

```rust
pub struct NotebookLmClient {
    http: Client,           // reqwest
    csrf: String,          // Token CSRF
    limiter: Limiter,      // Rate limiting
    conversation_cache: SharedConversationCache,
}
```

**Responsabilidades:**
- HTTP requests a la API de NotebookLM
- Rate limiting (2 req/segundo)
- Retry con exponential backoff
- Parsing de respuestas RPC
- Cache de conversaciones

### `auth_browser.rs` — Autenticación browser

```rust
pub struct BrowserAuthenticator {}
```

**Flujo:**
1. Lanza Chrome headless via CDP
2. Navega a accounts.google.com
3. Usuario completa login manualmente
4. Extrae cookies `__Secure-1PSID` y `__Secure-1PSIDTS`
5. Guarda en OS Keyring (Windows Credential Manager)

### `auth_helper.rs` — Helper de autenticación

- Extrae token CSRF (`SNlM0e`) del HTML
- Valida sesión con GET a notebooklm.google.com
- Refresco automático de CSRF cuando expira

### `parser.rs` — Parser defensivo

Funciones utilitarias para parsear respuestas RPC de Google:

```rust
extract_by_rpc_id(&response, "wXbhsf")  // Extrae por RPC ID
strip_antixssi_prefix(text)             // Limpia prefijo )]}'
get_string_at(array, index)            // Acceso seguro a arrays
get_uuid_at(array, index)               // Extrae UUID válido
```

### `source_poller.rs` — Polling de fuentes

Espera hasta que una fuente esté indexada antes de permitir preguntas.

```rust
pub async fn wait_for_source_ready(&self, notebook_id, source_id)
```

### `conversation_cache.rs` — Cache de conversaciones

Mantiene historial de preguntas/respuestas por notebook para que el chatbot tenga contexto.

### `errors.rs` — Errores estructurados

```rust
pub enum NotebookLmError {
    SessionExpired,    // Cookies expiraron
    CsrfExpired,       // Token CSRF expiró
    SourceNotReady,   // Fuente indexándose
    RateLimited,       // Límite de requests
    ParseError,        // Falló parseo
    NetworkError,      // Error de red
}
```

## Autenticación

### Método 1: Chrome headless (recomendado)

```bash
notebooklm-mcp auth-browser
```

1. Abre ventana de Chrome para login manual
2. Extrae cookies HttpOnly via CDP
3. Guarda en Windows Credential Manager

### Método 2: Manual

```bash
notebooklm-mcp auth --cookie "..." --csrf "..."
```

1. Copiar cookies manualmente desde DevTools
2. Extraer CSRF token del HTML
3. Encriptar con DPAPI y guardar

### Método 3: keyring

```rust
BrowserAuthenticator::store_in_keyring(&creds)
BrowserAuthenticator::load_from_keyring()
```

## Rate Limiting

```rust
let quota = Quota::with_period(Duration::from_secs(2)).unwrap();
```

- 2 requests por segundo (por cliente)
- Exponential backoff en retries: 1s, 2s, 4s, 8s... (max 30s)
- Jitter: 100-1000ms aleatorio para evitar thundering herd

## Protocolo MCP

### Tools

| Tool | Descripción |
|------|-------------|
| `notebook_list` | Lista todas las libretas |
| `notebook_create` | Crea una libreta nueva |
| `source_add` | Añade una fuente de texto |
| `ask_question` | Pregunta al chatbot |

### Resources

- `notebook://{uuid}` — Notebooks como recursos

## Flags de Compilación

```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = ["Win32_Security_Credentials", "Win32_Foundation"] }
```

Solo Windows tiene acceso a DPAPI para encriptación de credenciales.
