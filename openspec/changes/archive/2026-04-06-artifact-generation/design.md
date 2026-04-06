# Design: Artifact Generation & Download

## Architecture Decisions

### AD-1: Hybrid Enum + Dispatcher Pattern

**Decision**: Usar un enum `ArtifactConfig` con variantes por tipo, cada una con solo sus campos válidos. Un dispatcher mapea la variante al array posicional correcto.

**Rationale**: 
- Compile-time safety: Quiz requiere `difficulty`, Audio no. El compilador rechaza configs inválidos.
- Single generation entry point: un método `generate_artifact()` para todos los tipos excepto mind map.
- Testable: cada variante produce un JSON array predecible y testeable.
- Follows user instruction: "Aprovecha el sistema de tipos de Rust para que cada variante del Enum contenga solo los parámetros de configuración válidos".

**Alternative rejected**: Dynamic payload builder — runtime errors possible, harder to test.

### AD-2: Blocking Polling (no channels) for MCP

**Decision**: Implementar polling blocking con exponential backoff. No usar `tokio::sync::mpsc` canales.

**Rationale**:
- MCP tools son inherentemente request-response. Un tool call no puede "devolver" un resultado después de que el caller se desconecta.
- El MCP server usa `rmcp` que espera una respuesta síncrona del tool handler.
- El user puede elegir: `wait=true` (bloquea hasta completion) o `wait=false` (retorna task_id inmediatamente para polling manual).
- El Python reference también usa blocking polling con `asyncio.sleep` en el mismo runtime.

**Tradeoff**: Si un LLM llama `artifact_generate` con `wait=true` y el video tarda 5 minutos, el MCP tool está bloqueado. Esto es aceptable porque:
1. El LLM puede usar `wait=false` si no quiere esperar
2. Los timeouts son configurables (default 5 min, cinematic 30 min)
3. No hay forma de enviar notificaciones MCP async con el framework actual

### AD-3: Reuse upload_http Client for Downloads

**Decision**: Reusar el `upload_http` client (sin Content-Type global) para descargas HTTP de artefactos media.

**Rationale**:
- Ya tiene las cookies de auth configuradas.
- No tiene Content-Type global que podría interferir con la descarga.
- Las descargas son GET requests que no necesitan Content-Type body header.
- Evita crear un tercer `reqwest::Client`.

### AD-4: New File for ArtifactPoller

**Decision**: Crear `src/artifact_poller.rs` como módulo separado, NO extender `SourcePoller`.

**Rationale**:
- SourcePoller es específico para fuentes (status codes diferentes, no tiene media ready gate).
- ArtifactPoller necesita lógica distinta: scan por task_id, media URL verification, type-specific URL extraction.
- Separation of concerns: cada poller maneja su dominio.
- Ambos pueden coexistir como módulos independientes.

### AD-5: HTML Parsing with scraper Crate

**Decision**: Usar `scraper` crate (CSS selectors) para parsear quiz/flashcard HTML y extraer `data-app-data`.

**Rationale**:
- `scraper` es el crate Rust estándar para HTML parsing (wrapper sobre `html5ever` + `selectors`).
- CSS selectors son más robustos que regex para HTML.
- El atributo `data-app-data` se puede seleccionar con `[data-app-data]`.
- HTML entity unescaping viene incluido.

**Alternative rejected**: `regex` — frágil para HTML, no maneja entities correctamente.

### AD-6: Atomic File Writes for Downloads

**Decision**: Escribir a archivo temporal `{path}.tmp`, rename atómico al completar.

**Rationale**:
- Si la descarga falla a mitad, no queda un archivo corrupto.
- `std::fs::rename` es atómico en el mismo filesystem.
- Mismo patrón que el Python reference.

### AD-7: Error Variants for Artifact Operations

**Decision**: Agregar 4 variantes nuevas a `NotebookLmError`:

```rust
ArtifactNotReady(String),    // Download attempted on non-completed artifact
ArtifactNotFound(String),    // artifact_id not found in notebook
DownloadFailed(String),      // HTTP download or parsing failed
GenerationFailed(String),    // Artifact generation failed (non-rate-limit)
```

**Rationale**: Cada tipo de error permite al caller (MCP tool o CLI) dar un mensaje específico y accionable al usuario.

## File Changes

### New Files

| File | Purpose |
|------|---------|
| `src/rpc/artifacts.rs` | Enums (ArtifactTypeCode, ArtifactStatus, AudioFormat, etc.), ArtifactConfig enum, `to_params_array()` builders, unit tests |
| `src/artifact_poller.rs` | ArtifactPoller: `poll_status()`, `wait_for_completion()`, media ready gate, URL extraction helpers |

### Modified Files

| File | Changes |
|------|---------|
| `src/rpc/mod.rs` | Add `pub mod artifacts` |
| `src/errors.rs` | +4 error variants: ArtifactNotReady, ArtifactNotFound, DownloadFailed, GenerationFailed + Display + from_string detection |
| `src/notebooklm_client.rs` | +`list_artifacts()`, +`generate_artifact()`, +`generate_mind_map()`, +`wait_for_completion()`, +`download_artifact()`, +`delete_artifact()`, +private helpers |
| `src/parser.rs` | +`parse_artifact_list()`, +`parse_generation_result()`, +`extract_artifact_url()` |
| `src/main.rs` | +4 MCP tools (artifact_list, artifact_generate, artifact_download, artifact_delete), +4 CLI subcommands |
| `Cargo.toml` | +`scraper` crate |

## Data Flow

### Generation Flow

```
MCP Tool: artifact_generate
  → ArtifactConfig::Audio { ... }  (type-safe enum variant)
  → config.to_params_array()        (generates positional JSON array)
  → client.generate_artifact(nb_id, config)
    → batchexecute("R7cb6c", params)
    → parse_generation_result()
    → GenerationStatus { task_id, status }
  → if wait=true:
    → poll_status(nb_id, task_id)  [LIST_ARTIFACTS + scan]
    → media_ready_gate()           [verify URL populated]
    → exponential_backoff loop
    → return final GenerationStatus
```

### Download Flow

```
MCP Tool: artifact_download
  → client.download_artifact(nb_id, artifact_id, output_path)
    → list_artifacts(nb_id)         [find target artifact]
    → verify is_completed
    → match artifact.kind:
      Audio/Video/Infographic/SlideDeck:
        → extract_url_from_artifact()  [type-specific position]
        → validate_url_domain()         [Google domains only]
        → streaming_download()          [64KB chunks → tmp → rename]
      Report:
        → extract inline markdown from art[7][0]
        → write to file
      DataTable:
        → extract nested data from art[18]
        → parse cells recursively
        → write CSV with BOM
      Quiz/Flashcard:
        → RPC GET_INTERACTIVE_HTML (v9rmvd)
        → extract data-app-data from HTML (scraper)
        → parse JSON
        → write JSON
      MindMap:
        → RPC GET_NOTES_AND_MIND_MAPS (cFji9)
        → extract JSON from note
        → write formatted JSON
```

## ArtifactConfig Enum Design

```rust
pub enum ArtifactConfig {
    Audio {
        format: AudioFormat,      // DeepDive, Brief, Critique, Debate
        length: AudioLength,      // Short, Default, Long
        instructions: Option<String>,
        language: String,
        source_ids: Vec<String>,
    },
    Video {
        format: VideoFormat,      // Explainer, Brief, Cinematic
        style: Option<VideoStyle>, // None for Cinematic
        instructions: Option<String>,
        language: String,
        source_ids: Vec<String>,
    },
    Report {
        format: ReportFormat,     // BriefingDoc, StudyGuide, BlogPost, Custom { prompt }
        language: String,
        source_ids: Vec<String>,
        extra_instructions: Option<String>,
    },
    Quiz {
        difficulty: QuizDifficulty,
        quantity: QuizQuantity,
        instructions: Option<String>,
        source_ids: Vec<String>,
    },
    Flashcards {
        difficulty: QuizDifficulty,
        quantity: QuizQuantity,
        instructions: Option<String>,
        source_ids: Vec<String>,
    },
    Infographic {
        orientation: InfographicOrientation,
        detail: InfographicDetail,
        style: InfographicStyle,
        instructions: Option<String>,
        language: String,
        source_ids: Vec<String>,
    },
    SlideDeck {
        format: SlideDeckFormat,
        length: SlideDeckLength,
        instructions: Option<String>,
        language: String,
        source_ids: Vec<String>,
    },
    DataTable {
        instructions: String,
        language: String,
        source_ids: Vec<String>,
    },
}
```

Note: `CinematicVideo` is NOT a separate variant — it's `ArtifactConfig::Video { format: Cinematic, style: None }`. Mind Map is handled by a separate method `generate_mind_map()`.

## MCP Tools Design

### artifact_list
```json
{
  "name": "artifact_list",
  "description": "List all artifacts in a notebook with type, status, and metadata",
  "inputSchema": {
    "type": "object",
    "properties": {
      "notebook_id": { "type": "string", "description": "Notebook UUID" },
      "kind": { "type": "string", "enum": ["audio","video","report","quiz","flashcards","mind_map","infographic","slide_deck","data_table"], "description": "Optional filter by type" }
    },
    "required": ["notebook_id"]
  }
}
```

### artifact_generate
```json
{
  "name": "artifact_generate",
  "description": "Generate an artifact (audio, video, report, quiz, etc.)",
  "inputSchema": {
    "type": "object",
    "properties": {
      "notebook_id": { "type": "string" },
      "kind": { "type": "string", "enum": ["audio","video","report","quiz","flashcards","infographic","slide_deck","data_table","mind_map"] },
      "instructions": { "type": "string", "description": "Natural language instructions for generation" },
      "language": { "type": "string", "default": "en" },
      "wait": { "type": "boolean", "default": false, "description": "Wait for generation to complete" },
      "timeout": { "type": "number", "default": 300, "description": "Max wait time in seconds" },
      "audio_format": { "type": "string", "enum": ["deep_dive","brief","critique","debate"] },
      "audio_length": { "type": "string", "enum": ["short","default","long"] },
      "video_format": { "type": "string", "enum": ["explainer","brief","cinematic"] },
      "video_style": { "type": "string", "enum": ["auto","classic","whiteboard","kawaii","anime","watercolor","retro_print","heritage","paper_craft"] },
      "quiz_difficulty": { "type": "string", "enum": ["easy","medium","hard"] },
      "quiz_quantity": { "type": "string", "enum": ["fewer","standard","more"] },
      "report_format": { "type": "string", "enum": ["briefing_doc","study_guide","blog_post","custom"] },
      "infographic_orientation": { "type": "string", "enum": ["landscape","portrait","square"] },
      "infographic_detail": { "type": "string", "enum": ["concise","standard","detailed"] },
      "slide_deck_format": { "type": "string", "enum": ["detailed","presenter"] },
      "slide_deck_length": { "type": "string", "enum": ["default","short"] }
    },
    "required": ["notebook_id", "kind"]
  }
}
```

### artifact_download
```json
{
  "name": "artifact_download",
  "description": "Download a completed artifact to a local file",
  "inputSchema": {
    "type": "object",
    "properties": {
      "notebook_id": { "type": "string" },
      "artifact_id": { "type": "string" },
      "output_path": { "type": "string", "description": "Local file path for the download" },
      "format": { "type": "string", "enum": ["pdf","pptx"], "description": "For slide decks only" }
    },
    "required": ["notebook_id", "artifact_id", "output_path"]
  }
}
```

### artifact_delete
```json
{
  "name": "artifact_delete",
  "description": "Delete an artifact from a notebook",
  "inputSchema": {
    "type": "object",
    "properties": {
      "notebook_id": { "type": "string" },
      "artifact_id": { "type": "string" }
    },
    "required": ["notebook_id", "artifact_id"]
  }
}
```
