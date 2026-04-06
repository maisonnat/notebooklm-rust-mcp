# Exploration: Artifact Generation & Download (Module 2)

## Current State

Post-Module 1, el servidor MCP tiene:

- `NotebookLmClient` con métodos: `list_notebooks`, `create_notebook`, `add_source` (text), `add_url_source`, `add_file_source`, `add_drive_source`, `get_notebook_sources`, `ask_question`
- `SourcePoller` con polling basado en estados reales (PROCESSING=1, READY=2, ERROR=3)
- `parser.rs` con funciones defensivas + `extract_nested_source_id`
- `errors.rs` con enum `NotebookLmError` (10 variantes)
- `src/rpc/sources.rs` con typed structs para payloads de fuentes
- 2 `reqwest::Client`: uno con Content-Type global (RPC), otro sin Content-Type (uploads)
- 7 MCP tools: `notebook_list`, `notebook_create`, `source_add`, `source_add_url`, `source_add_file`, `source_add_drive`, `ask_question`
- 64 tests pasando, 0 clippy warnings

### Limitaciones Actuales

1. **No se pueden generar artefactos**: No hay forma de crear audio, video, reportes, quizzes, etc.
2. **No se pueden listar artefactos existentes**: No hay discovery de artefactos en un notebook
3. **No se pueden descargar artefactos**: No hay mecanismo de descarga (streaming o inline)
4. **No hay polling de generación**: El SourcePoller es solo para fuentes, no para artefactos

## Findings from Reverse Engineering (teng-lin/notebooklm-py)

### 1. Artifact Types — 10 Tipos

| Type | Code | Output | Download Method | Generation RPC |
|------|------|--------|-----------------|----------------|
| Audio Overview | 1 | MP4/MP3 | Streaming HTTP | `CREATE_ARTIFACT` |
| Report | 2 | Markdown | Inline text extraction | `CREATE_ARTIFACT` |
| Video Overview | 3 | MP4 | Streaming HTTP | `CREATE_ARTIFACT` |
| Quiz / Flashcards | 4 | JSON/MD/HTML | HTML parse → transform | `CREATE_ARTIFACT` |
| Mind Map | 5 | JSON | Notes system (SPECIAL) | `GENERATE_MIND_MAP` |
| Infographic | 7 | PNG | Direct URL download | `CREATE_ARTIFACT` |
| Slide Deck | 8 | PDF/PPTX | Direct URL download | `CREATE_ARTIFACT` |
| Data Table | 9 | CSV | Structured data parse | `CREATE_ARTIFACT` |

**CRITICAL**: Type 4 (Quiz/Flashcards) comparte código interno. Se distinguen por **variant**:
- Variant 1 = Flashcards
- Variant 2 = Quiz

**CRITICAL**: Type 6 appears UNUSED in the reference.

### 2. RPC Endpoints — Artifact Operations

| RPC ID | Name | Operation | source_path |
|--------|------|-----------|-------------|
| `R7cb6c` | CREATE_ARTIFACT | Generate any artifact type (except mind map) | `/notebook/{id}` |
| `gArtLc` | LIST_ARTIFACTS | List all artifacts in a notebook | `/notebook/{id}` |
| `V5N4be` | DELETE_ARTIFACT | Delete an artifact | `/notebook/{id}` |
| `rc3d8d` | RENAME_ARTIFACT | Rename an artifact | `/notebook/{id}` |
| `Krh3pd` | EXPORT_ARTIFACT | Export to Google Docs/Sheets | `/notebook/{id}` |
| `RGP97b` | SHARE_ARTIFACT | Share artifact publicly | `/notebook/{id}` |
| `v9rmvd` | GET_INTERACTIVE_HTML | Get quiz/flashcard HTML content | `/notebook/{id}` |
| `KmcKPe` | REVISE_SLIDE | Revise individual slide with prompt | `/notebook/{id}` |
| `yyryJe` | GENERATE_MIND_MAP | Generate mind map (DIFFERENT RPC!) | `/notebook/{id}` |
| `CYK0Xb` | CREATE_NOTE | Persist mind map as note | `/notebook/{id}` |
| `cFji9` | GET_NOTES_AND_MIND_MAPS | List notes + mind maps | `/notebook/{id}` |
| `ciyUvf` | GET_SUGGESTED_REPORTS | AI-suggested report formats | `/notebook/{id}` |

### 3. Artifact Status Codes

```python
PROCESSING = 1    # Being generated
PENDING = 2       # Queued
COMPLETED = 3     # Ready
FAILED = 4        # Generation failed
```

### 4. Generation Flow — CREATE_ARTIFACT

**ALL artifact types (except Mind Map)** use the **SAME** RPC endpoint `R7cb6c`. The params differ by type code and config array position.

#### Common Pattern
```python
params = [
    [2],
    notebook_id,
    [
        None, None,
        <type_code>,           # ArtifactTypeCode integer
        source_ids_triple,      # [[[sid1]], [[sid2]], ...]
        None, None,
        # ... padding varies by type ...
        [config_wrapper],       # Position varies: 6=audio, 7=report, 8=video, 9=quiz, 14=infographic, 16=slide_deck, 18=data_table
    ],
]
```

**Response parsing**:
```python
result[0][0]  # = artifact_id (also used as task_id for polling)
result[0][4]  # = initial status code (ArtifactStatus)
```

**KEY INSIGHT**: `task_id` and `artifact_id` are the SAME identifier.

#### Payload per Type

**Audio (type 1)** — config at index 6:
```python
params = [[2], notebook_id, [
    None, None, 1, source_ids_triple, None, None,
    [None, [instructions, length_code, None, source_ids_double, language, None, format_code]],
]]
```
- `AudioFormat`: DEEP_DIVE=1, BRIEF=2, CRITIQUE=3, DEBATE=4
- `AudioLength`: SHORT=1, DEFAULT=2, LONG=3

**Report (type 2)** — config at index 7:
```python
params = [[2], notebook_id, [
    None, None, 2, source_ids_triple, None, None, None,
    [None, [title, description, None, source_ids_double, language, prompt, None, True]],
]]
```
- Built-in templates: BRIEFING_DOC, STUDY_GUIDE, BLOG_POST, CUSTOM

**Video (type 3)** — config at index 8:
```python
params = [[2], notebook_id, [
    None, None, 3, source_ids_triple, None, None, None, None,
    [None, None, [source_ids_double, language, instructions, None, format_code, style_code]],
]]
```
- `VideoFormat`: EXPLAINER=1, BRIEF=2, CINEMATIC=3
- `VideoStyle`: AUTO_SELECT=1..PAPER_CRAFT=10
- **Cinematic** sets format=3, style=None. Uses Veo 3 AI. Requires Ultra subscription. ~30-40 min.

**Quiz (type 4, variant 2)** — config at index 9:
```python
params = [[2], notebook_id, [
    None, None, 4, source_ids_triple, None, None, None, None, None, None,
    [None, [2, None, instructions, None, None, None, None, [quantity_code, difficulty_code]]],
]]
```
- `QuizQuantity`: FEWER=1, STANDARD=2
- `QuizDifficulty`: EASY=1, MEDIUM=2, HARD=3

**Flashcards (type 4, variant 1)** — config at index 9:
```python
params = [[2], notebook_id, [
    None, None, 4, source_ids_triple, None, None, None, None, None, None,
    [None, [1, None, instructions, None, None, None, [difficulty_code, quantity_code]]],
]]
```
**CRITICAL**: Quiz has `[quantity, difficulty]` at position [7]. Flashcards has `[difficulty, quantity]` at position [6] — **REVERSED ORDER AND DIFFERENT POSITION!**

**Infographic (type 7)** — config at index 14:
```python
params = [[2], notebook_id, [
    None, None, 7, source_ids_triple, None, None, None, None, None, None, None, None, None, None,
    [[instructions, language, None, orientation_code, detail_code, style_code]],
]]
```
- `InfographicOrientation`: LANDSCAPE=1, PORTRAIT=2, SQUARE=3
- `InfographicDetail`: CONCISE=1, STANDARD=2, DETAILED=3
- `InfographicStyle`: AUTO_SELECT=1..SCIENTIFIC=11

**Slide Deck (type 8)** — config at index 16:
```python
params = [[2], notebook_id, [
    None, None, 8, source_ids_triple, None, None, None, None, None, None, None, None, None, None, None, None,
    [[instructions, language, format_code, length_code]],
]]
```
- `SlideDeckFormat`: DETAILED_DECK=1, PRESENTER_SLIDES=2
- `SlideDeckLength`: DEFAULT=1, SHORT=2

**Data Table (type 9)** — config at index 18:
```python
params = [[2], notebook_id, [
    None, None, 9, source_ids_triple, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
    [None, [instructions, language]],
]]
```

**Mind Map (type 5)** — COMPLETELY DIFFERENT:
```python
params = [
    source_ids_nested,    # [[[sid1]], [[sid2]], ...]
    None, None, None, None,
    ["interactive_mindmap", [["[CONTEXT]", ""]], ""],  # Hardcoded
    None,
    [2, None, [1]],       # Hardcoded
]
# RPC: GENERATE_MIND_MAP ("yyryJe")
# After generation, must call CREATE_NOTE ("CYK0Xb") to persist!
```

### 5. Polling — LIST_ARTIFACTS + Scan

**There is NO poll-by-ID endpoint.** Polling works by listing ALL artifacts and scanning for the `task_id`.

```python
# LIST_ARTIFACTS ("gArtLc")
params = [[2], notebook_id, 'NOT artifact.status = "ARTIFACT_STATUS_SUGGESTED"']
```

Response structure per artifact:
```
art[0]  = artifact_id (string)
art[1]  = title (string)
art[2]  = artifact_type (int: ArtifactTypeCode)
art[3]  = error_reason (string, only when failed)
art[4]  = status_code (int: ArtifactStatus)
art[5]  = nested error payload (list, only when failed)
art[6]  = metadata (audio: [6][5] = media URL list)
art[7]  = content (report: markdown at [7][0])
art[8]  = metadata (video: media URL list)
art[9]  = options ([[variant_code], ...] — variant at [9][1][0])
art[15] = timestamps ([[unix_ts], ...])
art[16] = metadata (slide deck: [config, title, slides, pdf_url, pptx_url])
art[18] = data (data table: deeply nested rich-text)
```

**Media Ready Gate**: For media types (audio, video, infographic, slide_deck), even if `status=COMPLETED`, the URL may not be populated yet. Must verify URL availability before declaring complete:
- Audio: `art[6][5]` must be non-empty list
- Video: `art[8]` must contain URL starting with "http"
- Infographic: forward-scan for nested URL
- Slide Deck: `art[16][3]` must be valid URL

**Polling config**:
```
initial_interval: 2.0s
max_interval: 10.0s
timeout: 300s (5 min default)
backoff: interval = min(interval * 2, max_interval)
```

### 6. Download Flow — 3 Strategies

#### Strategy 1: Streaming HTTP (Audio, Video, Infographic, Slide Deck)
- Extract URL from artifact metadata
- Validate URL is on trusted Google domain (*.google.com, *.googleusercontent.com, *.googleapis.com)
- Stream with chunk_size=65536 to avoid loading entire file in RAM
- Write to `.tmp` file, rename on success (atomic)

URL extraction per type:
| Type | Location | MIME Filter |
|------|----------|-------------|
| Audio | `art[6][5]` — iterate for `item[2] == "audio/mp4"` | audio/mp4 |
| Video | `art[8]` — iterate for `item[2] == "video/mp4"`, prefer quality=4 | video/mp4 |
| Infographic | Forward-scan `art[i][2][0][1][0]` for http URL | image/* |
| Slide Deck PDF | `art[16][3]` | direct URL |
| Slide Deck PPTX | `art[16][4]` | direct URL |

**Auth**: Cookies from storage attached to download client. Cross-domain redirects to CDN are authenticated.

#### Strategy 2: Inline Content (Report, Data Table)
- **Report**: Content at `art[7][0]` — markdown string, written directly to file
- **Data Table**: Content at `art[18]` — deeply nested rich-text, parsed recursively into CSV

#### Strategy 3: HTML Parse (Quiz, Flashcards)
1. Fetch HTML via `GET_INTERACTIVE_HTML` (`v9rmvd`): `params = [artifact_id]`
2. Extract `data-app-data="..."` attribute from HTML
3. Parse JSON from attribute value
4. Format as JSON, Markdown, or raw HTML

#### Mind Map Download
- Content stored in notes system via `GET_NOTES_AND_MIND_MAPS` (`cFji9`)
- JSON string at `mind_map_data[1][1]`
- Written as formatted JSON

### 7. Rate Limiting in Generation

When generation is rate-limited, the API returns `rpc_code == "USER_DISPLAYABLE_ERROR"`. Instead of throwing an exception, the reference wraps this into a `GenerationStatus` with:
```python
status = "failed"
error_code = "USER_DISPLAYABLE_ERROR"
```

This allows callers to distinguish retryable rate limits from real errors via `is_rate_limited`.

### 8. Delete/Rename Artifacts

```python
# Delete
params = [[2], artifact_id]
# RPC: DELETE_ARTIFACT ("V5N4be")

# Rename
params = [[artifact_id, new_title], [["title"]]]
# RPC: RENAME_ARTIFACT ("rc3d8d")
```

## Affected Areas

- `src/rpc/artifacts.rs` — **NEW**: Typed structs for all artifact generation payloads
- `src/rpc/mod.rs` — **MODIFIED**: Add `pub mod artifacts`
- `src/notebooklm_client.rs` — **MODIFIED**: +artifact methods (list, generate_*, download_*, delete, wait_for_completion)
- `src/errors.rs` — **MODIFIED**: +error variants (ArtifactNotReady, ArtifactNotFound, DownloadFailed, GenerationFailed)
- `src/parser.rs` — **MODIFIED**: +artifact response parsers
- `src/main.rs` — **MODIFIED**: +MCP tools (artifact_list, artifact_generate, artifact_download, artifact_delete)
- `Cargo.toml` — **MODIFIED**: potentially +`html-parser` or similar for quiz/flashcard HTML extraction

## Approaches

### Approach 1: Typed RPC Payloads with Builder Pattern (Recommended)

Crear structs Rust con builders para cada tipo de artefacto. Cada struct tiene su método `to_json_array()` que genera el payload posicional correcto.

- **Pros**: Compile-time safety, exhaustive pattern matching, impossible to mix up quiz vs flashcard config order
- **Cons**: More code upfront
- **Effort**: Medium-High

### Approach 2: Dynamic Payload Builder

Un solo builder genérico que acepta `ArtifactTypeCode` y config options, y construye el array correcto en runtime.

- **Pros**: Less code, single code path
- **Cons**: Runtime errors possible, harder to test all edge cases
- **Effort**: Medium

### Approach 3: Hybrid — Typed Dispatch + Dynamic Config

Enum `ArtifactConfig` con variantes por tipo (cada variante tiene solo sus campos válidos), más un dispatcher que arma el array correcto.

- **Pros**: Type-safe config per artifact, single dispatch logic
- **Cons**: More complex enum definition
- **Effort**: Medium

## Recommendation

**Approach 3 (Hybrid)**. Razón:
1. The user's original instruction says "Aprovecha el sistema de tipos de Rust para que cada variante del Enum contenga solo los parámetros de configuración válidos para ese artefacto" — this is EXACTLY Approach 3
2. Compile-time safety for config (Quiz has difficulty, Audio doesn't)
3. Single dispatch function that maps enum variant → positional array
4. Easy to test: each variant produces a known-good JSON array

For downloads, use **streaming with chunks of 64KB** (same as Module 1 uploads, but in reverse). The `upload_http` client (no global Content-Type) can be reused for downloads.

For polling, **blocking polling is acceptable for MCP** (the user's draft suggests channels, but MCP tools are inherently request-response). We can add a `wait` parameter: if true, block until completion; if false, return task_id immediately for later polling.

## Risks

1. **Google can change RPC IDs** — Same risk as Module 1. IDs are stable.
2. **Media ready gate is critical** — Without it, downloads fail immediately after "completion". Must implement.
3. **Quiz/Flashcard config order reversal** — EASY to get wrong. Must have tests that validate exact payload positions.
4. **HTML parsing for quiz/flashcard** — Fragile. `data-app-data` attribute format may change.
5. **Infographic URL scanning** — No fixed position. Must forward-scan the artifact array.
6. **Mind map is a two-step process** — Generate + persist as note. Easy to forget step 2.
7. **Large video files** — Cinematic videos can be very large. Must stream to disk.

## Ready for Proposal

**Yes.** Hay suficiente información para crear una propuesta detallada.

### Scope Definition

**In Scope:**
- List artifacts in a notebook
- Generate all 10 artifact types (audio, video, report, quiz, flashcards, infographic, slide deck, data table, mind map, cinematic video)
- Wait for generation completion (polling with media ready gate)
- Download all artifact types (streaming for media, inline for reports, HTML parse for quiz/flashcard)
- Delete artifacts
- MCP tools for all operations

**Out of Scope:**
- Rename artifact
- Export artifact to Google Docs/Sheets
- Share artifact publicly
- Revise individual slides
- Get suggested report formats
- Module 3 (Notebook CRUD)
