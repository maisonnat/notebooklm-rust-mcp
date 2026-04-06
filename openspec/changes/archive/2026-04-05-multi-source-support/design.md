# Design: Multi-Source Support

## Technical Approach

Extender `NotebookLmClient` con 3 métodos públicos (`add_url_source`, `add_file_source`, `add_drive_source`) que mapean a los RPCs `izAoDd` y `o4cbdc` + protocolo de resumable upload. Los payloads se tipan con structs `serde::Serialize` en un nuevo módulo `src/rpc/`. El file upload usa streaming async con `tokio::fs::File` → `ReaderStream` → `reqwest::Body::wrap_stream()`.

## Architecture Decisions

### AD-1: Segundo reqwest::Client para uploads

**Choice**: Añadir campo `upload_http: Client` a `NotebookLmClient` sin Content-Type global
**Alternatives**: (A) Header override por-request, (B) quitar Content-Type global del cliente existente
**Rationale**: El cliente actual hardcodea `Content-Type: application/x-www-form-urlencoded` como default header. Los uploads necesitan JSON (step 2) y raw bytes (step 3). Override por-request con reqwest requiere usar `.header()` que se suma (no reemplaza) a defaults. Un segundo cliente es limpio y no rompe el existente.

### AD-2: Módulo `src/rpc/` para typed structs

**Choice**: Crear `src/rpc/mod.rs` + `src/rpc/sources.rs`
**Alternatives**: (A) `src/source_types.rs` flat, (B) structs inline en `notebooklm_client.rs`
**Rationale**: Module 2 (`rpc/artifacts.rs`) y Module 3 (`rpc/notebooks.rs`) extenderán este módulo. Un directorio `rpc/` escala mejor que archivos flat. Separa "protocolo" de "lógica de aplicación".

### AD-3: Streaming via tokio-util::ReaderStream

**Choice**: `tokio::fs::File` → `ReaderStream` → `reqwest::Body::wrap_stream()`
**Alternatives**: (A) `reqwest::Body::from_reader()` (bloquea hasta EOF), (B) leer archivo entero en `Bytes`
**Rationale**: `wrap_stream` permite control de chunk size y es lazy — no carga el archivo en RAM. `from_reader` no expone chunks. Lectura entera viola el requisito de memoria para archivos grandes.

### AD-4: YouTube detection con crate `url`

**Choice**: `url::Url` para parseo + hostname matching
**Alternatives**: (A) regex (ya en deps), (B) string manipulation manual
**Rationale**: `url` parsea correctamente authority, scheme, query params. Regex es frágil para URLs con edge cases. El crate es tiny (< 100KB) y zero-dependency.

### AD-5: SourceState desde rLM1Ne (no solo presence)

**Choice**: Parsear `src[3][1]` de la respuesta GET_NOTEBOOK para obtener status code
**Alternatives**: (A) seguir con presence-only check
**Rationale**: El spec requiere distinguir Ready/Processing/Error. La respuesta de `rLM1Ne` incluye `SourceStatus` en `src[3][1]`: 1=Processing, 2=Ready, 3=Error. Presence check no detecta errores.

## Data Flow

### URL Source Flow

```
MCP Tool / CLI
    │
    ▼
add_url_source(nb_id, url)
    │
    ├─ is_youtube_url? ──Yes──▶ YouTube payload (11-elem array)
    │                          RPC: izAoDd
    ▼ No
Regular URL payload (8-elem array)
RPC: izAoDd
    │
    ▼
batchexecute() ──▶ parser ──▶ extract source_id ──▶ return
```

### File Upload Flow (3-step)

```
add_file_source(nb_id, path)
    │
    ▼
Step 1: RPC o4cbdc (register)
    │  params: [[filename]], nb_id, [2], config
    ▼
    ├── extract SOURCE_ID from nested response
    ▼
Step 2: POST /upload/_/?authuser=0 (start session)
    │  Headers: x-goog-upload-command=start
    │           x-goog-upload-protocol=resumable
    │  Body: {PROJECT_ID, SOURCE_NAME, SOURCE_ID}
    ▼
    ├── extract x-goog-upload-url from response headers
    ▼
Step 3: POST <upload_url> (stream file)
    │  Headers: x-goog-upload-command=upload, finalize
    │  Body: tokio::fs::File → ReaderStream → Body::wrap_stream
    ▼
    └── return source_id
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/rpc/mod.rs` | Create | Declara submódulo `pub mod sources` |
| `src/rpc/sources.rs` | Create | Structs tipados para payloads RPC: `UrlSourcePayload`, `YoutubeSourcePayload`, `DriveSourcePayload`, `FileRegisterPayload` + helper `UploadSessionBody` |
| `src/notebooklm_client.rs` | Modify | +field `upload_http: Client`, +3 métodos públicos, +3 helpers privados (`_register_file_source`, `_start_resumable_upload`, `_stream_upload_file`) |
| `src/main.rs` | Modify | +`pub mod rpc`, +3 request structs (JsonSchema), +3 MCP tools, +3 CLI subcommands |
| `src/errors.rs` | Modify | +3 variantes: `FileNotFound(String)`, `UploadFailed(String)`, `ValidationError(String)` |
| `src/parser.rs` | Modify | +fn `extract_nested_source_id()` para SOURCE_ID de o4cbdc |
| `src/source_poller.rs` | Modify | `SourceState::from_response()` parsea status code real desde rLM1Ne |
| `Cargo.toml` | Modify | +`url = "2"`, +`tokio-util = { version = "0.7", features = ["io"] }` |

## Interfaces / Contracts

### RPC Payload Structs (`src/rpc/sources.rs`)

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum SourcePayload {
    Url(UrlSourcePayload),
    Youtube(YoutubeSourcePayload),
    Drive(DriveSourcePayload),
}
```

### NotebookLmClient — New Public Methods

```rust
impl NotebookLmClient {
    pub async fn add_url_source(&self, notebook_id: &str, url: &str) -> Result<String, String>;
    pub async fn add_file_source(&self, notebook_id: &str, file_path: &str) -> Result<String, String>;
    pub async fn add_drive_source(&self, notebook_id: &str, file_id: &str, title: &str, mime_type: &str) -> Result<String, String>;
}
```

### New Error Variants

```rust
pub enum NotebookLmError {
    // ... existing variants ...
    FileNotFound(String),      // Path does not exist
    UploadFailed(String),      // Upload session or stream failed
    ValidationError(String),   // Path is directory, invalid input, etc.
}
```

### New MCP Request Structs

```rust
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SourceAddUrlRequest { pub notebook_id: String, pub url: String }

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SourceAddFileRequest { pub notebook_id: String, pub file_path: String }

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SourceAddDriveRequest { pub notebook_id: String, pub file_id: String, pub title: String, pub mime_type: Option<String> }
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | YouTube detection, URL parsing, payload serialization | `#[test]` con fixtures de URLs, assert JSON output |
| Unit | SourceState from response | `#[test]` con mock Values de rLM1Ne |
| Unit | Error variant classification | `#[test]` para `from_string()` con nuevos patrones |
| Integration | RPC payload roundtrip | `#[test]` serialize → deserialize para cada SourcePayload |
| Manual | File upload end-to-end | `cargo run -- add-file --notebook-id X --file-path Y` |

## Migration / Rollout

No migration required. Cambios son 100% aditivos. El `add_source` de texto existente no se toca.

## Open Questions

None — todos los endpoints, payloads y protocolos están documentados en `docs/rpc-reference.md` del repo de referencia.
