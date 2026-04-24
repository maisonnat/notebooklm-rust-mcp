# Tasks: Module 5 — Advanced Features

## Phase 1: Source Management (Delete / Rename)

### Task 1.1: Implement delete_source (RPC: tGMBJ)
- [x] 1.1.1: En `notebooklm_client.rs`, crear `delete_source(&self, notebook_id: &str, source_id: &str) -> Result<(), String>`
- [x] 1.1.2: Construir payload triple-nested: `[[[source_id]]]`
- [x] 1.1.3: Enviar petición POST batchexecute con RPC ID `tGMBJ`
- [x] 1.1.4: Unit test: parsear respuesta de éxito
- [x] 1.1.5: Unit test: manejo de source no existente (idempotente)

### Task 1.2: Implement rename_source (RPC: b7Wfje)
- [x] 1.2.1: En `notebooklm_client.rs`, crear `rename_source(&self, notebook_id: &str, source_id: &str, new_title: &str) -> Result<(), String>`
- [x] 1.2.2: Construir payload asimétrico: `[null, [source_id], [[[new_title]]]]`
- [x] 1.2.3: Enviar petición POST batchexecute con RPC ID `b7Wfje`
- [x] 1.2.4: Unit test: payload construction

### Task 1.3: MCP + CLI exposure for source management
- [x] 1.3.1: Crear `SourceDeleteRequest` y `SourceRenameRequest` structs en `main.rs`
- [x] 1.3.2: Añadir `#[tool] source_delete` en `NotebookLmServer`
- [x] 1.3.3: Añadir `#[tool] source_rename` en `NotebookLmServer`
- [x] 1.3.4: Añadir variantes `SourceDelete` y `SourceRename` en enum `Commands`
- [x] 1.3.5: Implementar handlers CLI en `match commands`

---

## Phase 2: Source Fulltext Extraction

### Task 2.1: RPC call and payload (RPC: hizoJc)
- [x] 2.1.1: En `notebooklm_client.rs`, crear `get_source_fulltext(&self, notebook_id: &str, source_id: &str) -> Result<String, String>`
- [x] 2.1.2: Construir payload: `[[source_id], [2], [2]]`
- [x] 2.1.3: Enviar petición POST batchexecute con RPC ID `hizoJc`
- [x] 2.1.4: Unit test: payload construction

### Task 2.2: Recursive defensive parser in Rust
- [x] 2.2.1: En `parser.rs`, crear `extract_all_text(val: &serde_json::Value, current_depth: u32, max_depth: u32) -> Vec<String>`
- [x] 2.2.2: Implementar lógica: si `val` es String → añadir al Vec. Si es Array → iterar recursivamente
- [x] 2.2.3: Implementar max_depth enforcement (default 10, hard cap 20)
- [x] 2.2.4: Retornar vector aplanado con `.join("\n")`
- [x] 2.2.5: Unit test: strings simples → retornan como están
- [x] 2.2.6: Unit test: arrays anidados de 3 niveles → aplanados correctamente
- [x] 2.2.7: Unit test: max_depth alcanzado → detiene recursión
- [x] 2.2.8: Unit test: mix de strings y arrays → todos los strings extraídos

### Task 2.3: MCP + CLI exposure for fulltext
- [x] 2.3.1: Crear `SourceGetFulltextRequest` struct en `main.rs`
- [x] 2.3.2: Añadir `#[tool] source_get_fulltext` en `NotebookLmServer`
- [x] 2.3.3: Añadir variante `SourceGetFulltext` en enum `Commands`
- [x] 2.3.4: Implementar handler CLI

---

## Phase 3: Notes CRUD

### Task 3.1: Two-step note creation
- [x] 3.1.1: (RPC `CYK0Xb`) Crear `create_note_empty(&self, notebook_id: &str) -> Result<String, String>`. Payload: `[notebook_id, "", [1], null, "New Note"]`. Extraer note_id
- [x] 3.1.2: (RPC `cYAfTb`) Crear `update_note(&self, notebook_id: &str, note_id: &str, title: &str, content: &str) -> Result<(), String>`. Payload: `[notebook_id, note_id, [[[content, title, [], 0]]]]`
- [x] 3.1.3: Crear método público `create_note(&self, notebook_id, title, content)` que llame a 3.1.1 y luego a 3.1.2 secuencialmente
- [x] 3.1.4: Unit tests para ambos payloads

### Task 3.2: List and soft-delete
- [x] 3.2.1: (RPC `cFji9`) Crear `list_notes(&self, notebook_id: &str) -> Result<Vec<Note>, String>`. Payload: `[notebook_id]`
- [x] 3.2.2: En parser, filtrar notas con status == 2 (soft-deleted). Regla: si item tiene `["id", null, 2]`, omitir
- [x] 3.2.3: (RPC `AH0mwd`) Crear `delete_note(&self, notebook_id: &str, note_id: &str) -> Result<(), String>`. Payload: `[notebook_id, null, [note_id]]`
- [x] 3.2.4: Unit test: parser filtra notas borradas correctamente
- [x] 3.2.5: Unit test: parser retorna solo notas activas

### Task 3.3: Notes types
- [x] 3.3.1: Crear `src/rpc/notes.rs` con structs `Note { id, title, content }`
- [x] 3.3.2: Añadir `pub mod notes;` en `src/rpc/mod.rs`

### Task 3.4: MCP + CLI exposure for notes
- [x] 3.4.1: Crear `NoteCreateRequest`, `NoteListRequest`, `NoteDeleteRequest` structs en `main.rs`
- [x] 3.4.2: Añadir `#[tool] note_create`, `#[tool] note_list`, `#[tool] note_delete` en `NotebookLmServer`
- [x] 3.4.3: Añadir variantes `NoteCreate`, `NoteList`, `NoteDelete` en enum `Commands`
- [x] 3.4.4: Implementar handlers CLI

---

## Phase 4: Chat History Sync

### Task 4.1: Get active conversation ID (RPC: hPTbtc)
- [x] 4.1.1: Implementar `get_last_conversation_id(&self, notebook_id: &str) -> Result<Option<String>, String>`
- [x] 4.1.2: Payload: `[[], null, notebook_id, 1]`
- [x] 4.1.3: Parsear triple-nested response: `[[[conv_id]]]`
- [x] 4.1.4: Retornar `None` si no hay conversación
- [x] 4.1.5: Unit test: extracción de conversation_id
- [x] 4.1.6: Unit test: respuesta vacía → retorna None

### Task 4.2: Get conversation turns (RPC: khqZz)
- [x] 4.2.1: Implementar `get_conversation_turns(&self, notebook_id: &str, conversation_id: &str, limit: u32) -> Result<Vec<ChatTurn>, String>`
- [x] 4.2.2: Payload: `[[], null, null, conversation_id, limit]`
- [x] 4.2.3: Unit test: payload construction

### Task 4.3: Chronological parser
- [x] 4.3.1: Crear `ChatTurn { role: String, text: String }` en `src/rpc/notes.rs`
- [x] 4.3.2: En parser.rs, crear `parse_conversation_turns(data: &Value) -> Vec<ChatTurn>`
- [x] 4.3.3: Detectar tipo: si `turn[2] == 1` → usuario (texto en `turn[3]`). Si `turn[2] == 2` → IA (texto en `turn[4][0][0]`)
- [x] 4.3.4: Invertir array para orden cronológico (Google devuelve newest-first)
- [x] 4.3.5: Unit test: parsear turno de usuario
- [x] 4.3.6: Unit test: parsear turno de IA
- [x] 4.3.7: Unit test: inversión de orden cronológico

### Task 4.4: Integration with ask_question
- [x] 4.4.1: Modificar `ask_question` para intentar recuperar `last_conversation_id` desde Google si no hay cache local
- [x] 4.4.2: Si se encuentra, usar el conversation_id oficial en el parámetro `history_json`
- [x] 4.4.3: Si falla la recuperación, fallback al comportamiento actual (nueva conversación)

### Task 4.5: MCP + CLI exposure for chat history
- [x] 4.5.1: Crear `ChatHistoryRequest` struct en `main.rs`
- [x] 4.5.2: Añadir `#[tool] chat_history` en `NotebookLmServer`
- [x] 4.5.3: Añadir variante `ChatHistory` en enum `Commands`
- [x] 4.5.4: Implementar handler CLI

---

## Phase 5: Deep Research

### Task 5.1: Start research (RPC: QA9ei)
- [x] 5.1.1: Implementar `start_deep_research(&self, notebook_id: &str, query: &str) -> Result<String, String>`
- [x] 5.1.2: Payload: `[null, [1], [query, 1], 5, notebook_id]`
- [x] 5.1.3: Extraer task_id de la respuesta
- [x] 5.1.4: Unit test: payload construction

### Task 5.2: Poll research status (RPC: e3bVqc)
- [x] 5.2.1: Crear `ResearchStatus { status_code: u32, sources: Vec<Value>, is_complete: bool }` en `src/rpc/notes.rs`
- [x] 5.2.2: Crear `src/rpc/research.rs` con tipos de research
- [x] 5.2.3: Añadir `pub mod research;` en `src/rpc/mod.rs`
- [x] 5.2.4: Implementar `poll_research_status(&self, notebook_id: &str, task_id: &str) -> Result<ResearchStatus, String>`. Payload: `[null, null, notebook_id]`
- [x] 5.2.5: Parsear respuesta buscando task_id, extraer status_code de `task_info[4]`
- [x] 5.2.6: Detectar completion: status_code == 2 o 6
- [x] 5.2.7: Extraer array de fuentes descubiertas
- [x] 5.2.8: Unit test: parser de status
- [x] 5.2.9: Unit test: detección de completion

### Task 5.3: Import research sources (RPC: LBwxtb)
- [x] 5.3.1: Implementar `import_research_sources(&self, notebook_id: &str, task_id: &str, sources: Value) -> Result<(), String>`
- [x] 5.3.2: Para fuentes web (type 2): construir payload de 11 elementos
- [x] 5.3.3: Para reporte principal (type 3): construir payload de 11 elementos
- [x] 5.3.4: Enviar todo a `LBwxtb`
- [x] 5.3.5: Unit test: payload construction

### Task 5.4: Blocking MCP tool with polling loop
- [x] 5.4.1: Crear `ResearchDeepDiveRequest` struct en `main.rs`
- [x] 5.4.2: Añadir `#[tool] research_deep_dive` en `NotebookLmServer`
- [x] 5.4.3: Implementar loop con `tokio::time::sleep` que llame al poller hasta completion
- [x] 5.4.4: Timeout configurable (default 300s). Si excede, retornar resultado parcial con fuentes descubiertas
- [x] 5.4.5: Al completar, llamar `import_research_sources` y retornar resumen
- [x] 5.4.6: Añadir variante `Research` en enum `Commands`
- [x] 5.4.7: Implementar handler CLI

---

## Phase 6: Integration & Verification

### Task 6.1: Full test suite
- [x] 6.1.1: `cargo test` — todos los tests pasan
- [x] 6.1.2: `cargo clippy` — 0 warnings
- [x] 6.1.3: Verificar que los 20 tools existentes siguen funcionando (no regressions)
- [x] 6.1.4: Contar: 28 MCP tools totales, 29 CLI commands totales

### Task 6.2: Documentation update
- [x] 6.2.1: Actualizar `docs/en/02-api-reference.md` con los 8 nuevos tools
- [x] 6.2.2: Actualizar `docs/en/05-user-guide.md` con los nuevos workflows
- [x] 6.2.3: Actualizar `docs/en/06-changelog.md` con la versión 0.3.0

---

## Summary

| Phase | Feature | New Methods | New Tools | New CLI | Tasks |
|-------|---------|-------------|-----------|---------|-------|
| 1 | Source Management | 2 | 2 | 2 | 15 |
| 2 | Source Fulltext | 1 | 1 | 1 | 12 |
| 3 | Notes CRUD | 3 | 3 | 3 | 16 |
| 4 | Chat History | 2 | 1 | 1 | 15 |
| 5 | Deep Research | 3 | 1 | 1 | 16 |
| 6 | Integration | — | — | — | 4 |
| **Total** | | **11** | **8** | **8** | **78** |
