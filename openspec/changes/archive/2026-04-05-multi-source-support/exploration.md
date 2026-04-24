# Exploration: Multi-Source Support (URLs + File Upload + Drive)

## Current State

El servidor MCP actual soporta UNICAMENTE fuentes de texto (`add_source` via RPC `izAoDd`). El cliente Rust tiene:

- `NotebookLmClient` con métodos: `list_notebooks`, `create_notebook`, `add_source` (text only), `get_notebook_sources`, `ask_question`
- `SourcePoller` con polling básico (solo verifica si source_id aparece en la lista)
- `parser.rs` con funciones defensivas para arrays posicionales de Google
- `errors.rs` con enum `NotebookLmError`
- 4 MCP tools: `notebook_list`, `notebook_create`, `source_add`, `ask_question`
- `reqwest` con features `rustls-tls`, `json`, `stream` (ya tiene streaming habilitado)

### Limitaciones Actuales

1. **Solo texto**: No se pueden añadir URLs web, YouTube, ni archivos binarios (PDF, Word, TXT)
2. **No hay file upload**: Falta completamente el protocolo de subida resumable de Google
3. **No hay Google Drive**: No se pueden añadir documentos de Drive como fuentes
4. **El `add_source` actual** usa un payload diferente al de URL (posición `[1]` para texto vs `[2]` para URL)

## Findings from Reverse Engineering (teng-lin/notebooklm-py)

### 1. URL Source (RPC: `izAoDd` - ADD_SOURCE)

**Payload para URL regular:**
```python
params = [
    [[None, None, [url], None, None, None, None, None]],  # URL en posición [2] del array interno (8 elementos)
    notebook_id,
    [2],
    None,
    None,
]
```

**Payload para YouTube (detección automática):**
```python
params = [
    [[None, None, None, None, None, None, None, [url], None, None, 1]],  # URL en posición [7], array de 11 elementos
    notebook_id,
    [2],
    [1, None, None, None, None, None, None, None, None, None, [1]],  # Config extra
]
```

**Payload para Google Drive:**
```python
source_data = [
    [file_id, mime_type, 1, title],  # File info
    None, None, None, None, None,    # Padding
    None, None, None, None,          # Padding
    1,                               # Trailing flag
]
params = [
    [source_data],  # SINGLE wrap (no double!)
    notebook_id,
    [2],
    [1, None, None, None, None, None, None, None, None, None, [1]],
]
```

### 2. File Upload (3-step protocol)

**El flujo es:**

1. **Register intent** (RPC: `o4cbdc` - ADD_SOURCE_FILE):
   ```python
   params = [
       [[filename]],  # Double-nested filename
       notebook_id,
       [2],
       [1, None, None, None, None, None, None, None, None, None, [1]],
   ]
   # → Returns SOURCE_ID (nested: [[[[source_id]]]])
   ```

2. **Start resumable upload session** (HTTP POST directo, NO batchexecute):
   ```
   POST https://notebooklm.google.com/upload/_/?authuser=0
   
   Headers:
     Content-Type: application/x-www-form-urlencoded;charset=UTF-8
     x-goog-upload-command: start
     x-goog-upload-header-content-length: <file_size>
     x-goog-upload-protocol: resumable
   
   Body: {"PROJECT_ID": notebook_id, "SOURCE_NAME": filename, "SOURCE_ID": source_id}
   
   → Returns header: x-goog-upload-url
   ```

3. **Stream upload file** (HTTP POST directo):
   ```
   POST <x-goog-upload-url>
   
   Headers:
     x-goog-upload-command: upload, finalize
     x-goog-upload-offset: 0
   
   Body: raw file bytes (streamed in 64KB chunks)
   ```

**Tipos de archivo soportados:** PDF, TXT, Markdown, EPUB, Word (.docx)

### 3. YouTube Detection

Se detecta por hostname (`youtube.com`, `youtu.be`, etc.) y se extrae video ID de:
- `youtube.com/watch?v=ID`
- `youtu.be/ID`
- `youtube.com/shorts/ID`
- `youtube.com/embed/ID`
- `youtube.com/live/ID`

### 4. Upload URL

```
const UPLOAD_URL = "https://notebooklm.google.com/upload/_/"
```

## Affected Areas

- `src/notebooklm_client.rs` — Añadir métodos: `add_url_source`, `add_file_source`, `add_drive_source`, `_register_file_source`, `_start_resumable_upload`, `_upload_file_streaming`
- `src/main.rs` — Registrar nuevas MCP tools: `source_add_url`, `source_add_file`, `source_add_drive`
- `src/errors.rs` — Añadir variantes: `FileNotFound`, `ValidationError`, `UploadFailed`
- `src/source_poller.rs` — Mejorar polling para soportar estados de procesamiento de archivos (no solo presence check)
- `src/parser.rs` — Añadir parsers para respuestas de file registration (SOURCE_ID nested) y URL source
- **NUEVO** `src/rpc/sources.rs` — Structs tipados para los payloads RPC de sources (URL, File, Drive, YouTube)

## Approaches

### Approach 1: Typed RPC Payloads (Recommended)

Crear structs Rust con `serde::Serialize` que representen los payloads posicionales de Google. Usar enums para distinguir tipos de fuente.

- **Pros**: Tipado seguro en compile-time, autocompletado, documentación viviente, zero-cost abstractions
- **Cons**: Más código upfront, requiere entender los arrays posicionales profundamente
- **Effort**: Medium

### Approach 2: String-based (Current Pattern)

Seguir el patrón actual de construir JSON strings con `format!()`.

- **Pros**: Rápido de implementar, similar a lo existente
- **Cons**: Frágil, difícil de mantener, no hay validación en compile-time, propenso a errores de encoding
- **Effort**: Low

### Approach 3: Hybrid

Typed structs para los payloads nuevos (URL, File, Drive) mientras se migra gradualmente el existente.

- **Pros**: Balance entre velocidad y seguridad, incremental
- **Cons**: Dos patrones coexistiendo temporalmente
- **Effort**: Medium

## Recommendation

**Approach 3 (Hybrid)** con tendencia a Approach 1.

Razón: El módulo 1 es el momento perfecto para introducir typed structs para los NUEVOS payloads (URL, File, Drive) sin romper lo existente. Los payloads posicionales son complejos y un solo error de índice rompe todo. El enum `SourceType` nos da pattern matching exhaustivo en compile-time.

Para file upload, usar `tokio::fs::File` con `reqwest::Body::wrap_stream()` para streaming directo disco→red (zero-copy cuando sea posible), con chunks de 64KB. Esto es SUPERIOR a Python que usa `open()` + `yield` — Rust tiene async I/O real.

## Risks

1. **Google puede cambiar los RPC IDs en cualquier momento** — Mitigación: Los IDs ya están documentados y son estables desde hace meses. Mismo riesgo que el código actual.
2. **Upload URL puede requerir auth cookies adicionales** — Mitigación: Ya tenemos las cookies en el cliente HTTP. Probablemente funcione con el mismo `reqwest::Client`.
3. **Resumable upload puede ser más complejo de lo documentado** — Mitigación: Seguir el protocolo exacto de notebooklm-py que ya funciona en producción.
4. **El cliente actual usa `default_headers` globales** que pueden interferir con el upload endpoint — Mitigación: Crear un segundo cliente HTTP sin Content-Type global para el upload, o usar `header::HeaderMap` por-request.
5. **Drive sources requieren file_id que el usuario debe obtener externamente** — Mitigación: Documentar claramente que `add_drive_source` necesita el file_id de Google Drive.

## Ready for Proposal

**Yes.** Hay suficiente información para crear una propuesta detallada. El cambio es bien acotado: añadir 3 nuevos tipos de fuente al MCP server con typed RPC payloads y file streaming upload.

### Scope Definition

**In Scope:**
- `add_url_source` (web URLs + auto-detección YouTube)
- `add_file_source` (PDF, TXT, MD, EPUB, DOCX con streaming upload)
- `add_drive_source` (Google Drive docs con file_id)
- MCP tools correspondientes
- Mejora del SourcePoller para estados de procesamiento

**Out of Scope:**
- Módulo 2 (Artifacts/Generation)
- Módulo 3 (Notebook CRUD)
- Refresh de fuentes
- Source guide / fulltext
