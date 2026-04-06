# Proposal: Multi-Source Support

## Intent

El servidor MCP solo acepta fuentes de texto. Los agentes IA no pueden ingerir URLs web, videos de YouTube, archivos PDF/Word ni documentos de Google Drive. Este cambio desbloquea el uso real de NotebookLM como herramienta de investigación.

## Scope

### In Scope
- `add_url_source`: Ingestar URLs web con auto-detección YouTube
- `add_file_source`: Subida de archivos (PDF, TXT, MD, EPUB, DOCX) con streaming disco→red
- `add_drive_source`: Añadir documentos de Google Drive por file_id
- 3 nuevas MCP tools + CLI commands correspondientes
- Typed structs para los nuevos payloads RPC (no refactor del existente)
- SourcePoller mejorado con estados de procesamiento reales

### Out of Scope
- Módulo 2 (Artifacts/Generation)
- Módulo 3 (Notebook CRUD)
- Refresh/freshness de fuentes existentes
- Source guide / fulltext
- Migración del `add_source` de texto al sistema tipado

## Capabilities

### New Capabilities
- `multi-source-ingestion`: Ingesta de URLs, archivos binarios y documentos Drive como fuentes de notebook. Cubre los RPC `izAoDd` (URL/YouTube/Drive), `o4cbdc` (file register), y el protocolo de resumable upload.

### Modified Capabilities
- `source-polling`: Actualmente solo verifica presencia de source_id. Se amplía para detectar estados de procesamiento (Ready/Processing/Error) desde la respuesta de `rLM1Ne`.

## Approach

**Hybrid** — Typed structs con `serde::Serialize` para payloads NUEVOS, sin tocar el existente.

1. Crear `src/rpc/sources.rs` con structs para cada payload posicional (URL, YouTube, Drive, FileRegister)
2. Implementar file upload en 3 pasos (register RPC → start resumable POST → stream POST) usando `tokio::fs::File` + `reqwest::Body::wrap_stream()` con chunks de 64KB
3. Crear un segundo `reqwest::Client` sin `Content-Type` global para los upload endpoints (el actual tiene `application/x-www-form-urlencoded` hardcodeado que interfiere)
4. Detección YouTube por hostname parsing con `url::Url` crate
5. Registrar 3 nuevas MCP tools en `main.rs`

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/rpc/sources.rs` | **New** | Structs tipados para payloads RPC de sources |
| `src/notebooklm_client.rs` | Modified | +3 métodos públicos, +3 helpers privados de upload |
| `src/main.rs` | Modified | +3 MCP tools, +3 CLI subcommands |
| `src/errors.rs` | Modified | +3 variantes: FileNotFound, UploadFailed, ValidationError |
| `src/source_poller.rs` | Modified | Polling con estados reales (no solo presence) |
| `src/parser.rs` | Modified | +1 parser para SOURCE_ID nested de file registration |
| `Cargo.toml` | Modified | +`url` crate para URL parsing |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Google cambia RPC IDs | Low | Mismo riesgo que código actual; IDs estables hace meses |
| Upload endpoint requiere auth distinta | Med | Reuse cookies del cliente existente; testear con header injection |
| Content-Type global interfiere con upload | Med | Segundo cliente HTTP sin Content-Type default |
| Streaming falla en archivos grandes | Low | reqwest stream ya testeado; chunks de 64KB probados en producción Python |

## Rollback Plan

Los métodos nuevos son aditivos — no modifican `add_source` existente. Si algo falla:
1. Remover las 3 nuevas MCP tools del router
2. Remover los métodos del cliente (compilation garantiza que nada los referencia)
3. Ningún dato persistente se modifica (no hay migraciones)

## Dependencies

- Crate `url` para parsing robusto de URLs y detección YouTube
- Cargo.toml ya tiene `reqwest` con feature `stream`

## Success Criteria

- [ ] `source_add_url` MCP tool ingesta URLs web correctamente
- [ ] YouTube URLs se detectan y usan payload correcto automáticamente
- [ ] `source_add_file` sube PDF/TXT/DOCX con streaming (sin cargar archivo completo en RAM)
- [ ] `source_add_drive` acepta file_id + mime_type + title
- [ ] SourcePoller detecta estados Ready/Processing/Error
- [ ] `cargo test` pasa sin errores
- [ ] `cargo clippy` sin warnings nuevos
