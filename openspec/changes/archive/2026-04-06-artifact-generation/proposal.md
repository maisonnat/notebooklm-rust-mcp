# Proposal: Artifact Generation & Download

## Intent

El servidor MCP puede ingerir fuentes pero NO puede generar contenido. Los agentes IA no pueden crear podcasts, videos, reportes, quizzes ni ningún otro artefacto de NotebookLM. Tampoco pueden descubrir qué artefactos ya existen ni descargarlos. Este cambio desbloquea la capacidad de automatización completa del ciclo de vida de contenido en NotebookLM.

## Scope

### In Scope
- `list_artifacts`: Descubrir todos los artefactos de un notebook (tipo, estado, título, ID)
- `generate_artifact`: Generar los 10 tipos de artefacto (audio, video, cinematic video, report, quiz, flashcards, infographic, slide deck, data table, mind map)
- `wait_for_completion`: Polling con exponential backoff + media ready gate
- `download_artifact`: Descargar cualquier artefacto completado (streaming para media, inline para texto, HTML parse para quiz/flashcard)
- `delete_artifact`: Eliminar artefactos
- MCP tools correspondientes + CLI subcommands

### Out of Scope
- Rename artifact
- Export artifact to Google Docs/Sheets
- Share artifact publicly
- Revise individual slides
- Get suggested report formats
- Module 3 (Notebook CRUD)

## Capabilities

### New Capabilities
- `artifact-generation`: Generación de los 10 tipos de artefacto via RPC `CREATE_ARTIFACT` (R7cb6c) y `GENERATE_MIND_MAP` (yyryJe). Cubre el armado de payloads posicionales por tipo, parsing de respuestas, y detección de rate limiting (USER_DISPLAYABLE_ERROR).
- `artifact-discovery`: Listado de artefactos existentes via RPC `LIST_ARTIFACTS` (gArtLc). Cubre parsing de la respuesta posicional, mapeo de type codes a enums, y extracción de metadata (URLs de descarga, contenido inline, timestamps).
- `artifact-download`: Descarga de artefactos completados. 3 estrategias: streaming HTTP (audio/video/infographic/slide_deck), inline extraction (report/data_table), HTML parse (quiz/flashcard). Streaming con chunks de 64KB, escritura atómica (tmp → rename).

### Modified Capabilities
- `source-polling`: El patrón de polling existente se extiende para artefactos. No se modifica el SourcePoller, pero se crea un ArtifactPoller con el mismo patrón de exponential backoff.

## Approach

**Hybrid** — Enum `ArtifactConfig` con variantes por tipo (cada variante tiene solo sus campos válidos), más un dispatcher que mapea cada variante al array posicional correcto.

1. Crear `src/rpc/artifacts.rs` con:
   - `ArtifactTypeCode` enum (int codes: AUDIO=1..DATA_TABLE=9)
   - `ArtifactStatus` enum (PROCESSING=1..FAILED=4)
   - Enums de config: `AudioFormat`, `AudioLength`, `VideoFormat`, `VideoStyle`, `QuizDifficulty`, `QuizQuantity`, `InfographicOrientation`, `InfographicDetail`, `InfographicStyle`, `SlideDeckFormat`, `SlideDeckLength`
   - `ArtifactConfig` enum con variantes por tipo (ej. `ArtifactConfig::Audio { format, length, instructions, language, source_ids }`)
   - Cada variante tiene su método `to_params_array()` que genera el JSON posicional correcto
   - Unit tests que validan que cada variante produce el array exacto esperado

2. Implementar `ArtifactPoller` en `src/artifact_poller.rs`:
   - `poll_status(notebook_id, task_id) -> GenerationStatus` — lista todos los artefactos y escanea por task_id
   - `wait_for_completion(notebook_id, task_id, timeout) -> GenerationStatus` — exponential backoff con media ready gate
   - Media ready gate: para tipos media (audio, video, infographic, slide_deck), verifica que la URL de descarga esté poblada ANTES de declarar completed

3. Implementar generación en `src/notebooklm_client.rs`:
   - `generate_artifact(notebook_id, config) -> GenerationStatus` — dispatcher único que llama CREATE_ARTIFACT
   - `generate_mind_map(notebook_id, source_ids) -> MindMapResult` — caso especial con GENERATE_MIND_MAP + CREATE_NOTE
   - Rate limiting: `USER_DISPLAYABLE_ERROR` se captura y devuelve como `GenerationStatus { status: "failed", error_code: "USER_DISPLAYABLE_ERROR" }` en vez de exception

4. Implementar descarga en `src/notebooklm_client.rs`:
   - `download_artifact(notebook_id, artifact_id, output_path) -> String` — dispatcher por tipo
   - Streaming HTTP con chunks de 64KB para media types
   - Validación de dominio Google antes de descargar (security gate)
   - Escritura atómica: `{path}.tmp` → rename on success
   - Reusar `upload_http` client (sin Content-Type global) para downloads

5. Registrar MCP tools en `src/main.rs`:
   - `artifact_list` — lista artefactos de un notebook
   - `artifact_generate` — genera artefacto (type + config como params)
   - `artifact_download` — descarga artefacto completado
   - `artifact_delete` — elimina artefacto

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/rpc/artifacts.rs` | **New** | Enums, ArtifactConfig, payload builders, unit tests |
| `src/rpc/mod.rs` | Modified | Add `pub mod artifacts` |
| `src/artifact_poller.rs` | **New** | ArtifactPoller con polling + media ready gate |
| `src/notebooklm_client.rs` | Modified | +6 métodos públicos (list, generate, generate_mind_map, wait_for_completion, download, delete), +helpers privados |
| `src/main.rs` | Modified | +4 MCP tools, +4 CLI subcommands |
| `src/errors.rs` | Modified | +4 variantes: ArtifactNotReady, ArtifactNotFound, DownloadFailed, GenerationFailed |
| `src/parser.rs` | Modified | +parsers para artifact list response, generation result, quiz HTML |
| `Cargo.toml` | Modified | +`scraper` crate (para HTML parsing de quiz/flashcard `data-app-data`) |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Google cambia RPC IDs | Low | Mismo riesgo que Module 1; IDs estables hace meses |
| Quiz/Flashcard config order reversal | High | Unit tests que validan posiciones exactas del array |
| Media ready gate omitido | Medium | Obligatorio en ArtifactPoller; tests que simulan URL vacía |
| HTML parsing de quiz/flashcard frágil | Medium | Usar `scraper` crate (selectores CSS robustos); fallback a regex |
| Infographic URL scan sin posición fija | Medium | Forward-scan algorithm con validación de prefijo `http` |
| Mind Map two-step process olvidado | Low | Método dedicado `generate_mind_map` que hace ambos pasos |
| Video files muy grandes (>100MB) | Low | Streaming con 64KB chunks; nunca carga entero en RAM |
| Rate limiting durante generación | Medium | Capturar USER_DISPLAYABLE_ERROR; devolver GenerationStatus retryable |

## Rollback Plan

Los métodos nuevos son aditivos — no modifican existentes. Si algo falla:
1. Remover las 4 nuevas MCP tools del router
2. Remover los métodos del cliente (compilation garantiza que nada los referencia)
3. Ningún dato persistente se modifica

## Dependencies

- Crate `scraper` para HTML parsing robusto de quiz/flashcard (CSS selectors)
- Cargo.toml ya tiene `reqwest` con feature `stream` (necesario para streaming downloads)
- Cargo.toml ya tiene `tokio` con features necesarias

## Success Criteria

- [ ] `artifact_list` MCP tool lista artefactos con tipo, estado, título, ID
- [ ] `artifact_generate` genera los 10 tipos de artefacto correctamente
- [ ] Mind map usa GENERATE_MIND_MAP + CREATE_NOTE (two-step)
- [ ] `wait_for_completion` espera con exponential backoff + media ready gate
- [ ] `artifact_download` descarga media via streaming (64KB chunks)
- [ ] Quiz/flashcard se descargan via HTML parse + transform
- [ ] Rate limiting devuelve GenerationStatus retryable (no exception)
- [ ] URL validation restringe descargas a dominios Google
- [ ] `artifact_delete` elimina artefactos
- [ ] `cargo test` pasa sin errores
- [ ] `cargo clippy` sin warnings nuevos
