# Proposal: Notebook Lifecycle Management (CRUD + Sharing)

## Intent

El servidor MCP puede crear notebooks pero NO puede eliminarlos, renombrarlos, ni obtener sus metadatos completos. Tampoco puede gestionar la visibilidad (público/privado) ni obtener resúmenes AI. Este cambio completa el ciclo de vida básico del notebook, desbloqueando workflows de automatización donde un agente IA gestiona notebooks de forma autónoma (crear → poblar → compartir → limpiar).

## Scope

### In Scope
- `delete_notebook(id)` — eliminación con idempotencia (404/sin error = success)
- `rename_notebook(id, new_title)` — renombrar y devolver Notebook actualizado
- `get_notebook(id)` — detalles completos (sources_count, is_owner, created_at)
- `get_summary(id)` — resumen AI + suggested topics
- `get_share_status(id)` — leer estado de sharing (público/privado, usuarios compartidos)
- `set_sharing_public(id, public)` — toggle público/privado
- MCP tools correspondientes + CLI subcommands

### Out of Scope
- User-level sharing (add_user, remove_user, update_permission)
- View level configuration (full notebook vs chat only)
- Remove from recently viewed
- Notebook-level export

## Capabilities

### New Capabilities
- `notebook-lifecycle`: Operaciones CRUD sobre notebooks — delete, rename, get (detalles completos), get_summary. RPCs: `WWINqb`, `s0tc2d`, `rLM1Ne` (reutilizado), `VfAZjd`.
- `notebook-sharing`: Gestión de visibilidad de notebooks — get_share_status, set_sharing_public. RPCs: `JFMDGd`, `QDyure`.

### Modified Capabilities
None. Las capacidades existentes no cambian a nivel de requisitos.

## Approach

**Extensión del patrón establecido** — mismo estilo que módulos anteriores:

1. **`src/rpc/notebooks.rs`** (new): RPC IDs (`WWINqb`, `VfAZjd`, `JFMDGd`, `QDyure`), enums de sharing (`ShareAccess`), structs de respuesta (`ShareStatus`, `SharedUser`). Parser helpers para responses posicionales.

2. **`src/notebooklm_client.rs`** (modified): +6 métodos públicos. Cada uno sigue el patrón `batchexecute()` → `extract_by_rpc_id()` → parse defensivo. `delete_notebook` con idempotencia. `rename_notebook` con `get_notebook` post-rename para devolver datos actualizados. `get_share_status` con parser de la respuesta `[[[users]], [is_public], 1000]`.

3. **`src/main.rs`** (modified): +6 MCP tools (`notebook_delete`, `notebook_rename`, `notebook_get`, `notebook_summary`, `notebook_share_status`, `notebook_share_set`). +6 CLI subcommands.

4. **`src/parser.rs`** (modified): +parsers para `get_notebook` response (sources_count, is_owner, created_at), `ShareStatus` response.

5. **Struct `Notebook` enriquecida**: Agregar `sources_count: usize`, `is_owner: bool`, `created_at: Option<String>`. Backward-compatible — los campos nuevos son opcionales en la construcción desde `list_notebooks`.

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/rpc/notebooks.rs` | **New** | RPC IDs, ShareAccess enum, ShareStatus struct, parser helpers |
| `src/rpc/mod.rs` | Modified | Add `pub mod notebooks` |
| `src/notebooklm_client.rs` | Modified | +6 métodos: delete, rename, get, get_summary, get_share_status, set_sharing_public |
| `src/main.rs` | Modified | +6 MCP tools, +6 CLI subcommands, +6 request structs |
| `src/parser.rs` | Modified | +parse_notebook_details(), +parse_share_status() |
| `src/errors.rs` | Modified | +NotebookNotFound variant (opcional) |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| `source_path` no implementado en batchexecute | Medium | Testear empíricamente; si falla, agregar param URL a batchexecute |
| `s0tc2d` dual-use (rename + view_level) | Low | Solo implementar rename; view_level es out-of-scope |
| ShareStatus response null/vacía | Medium | Parsing defensivo con defaults (is_public=false, users=[]) |
| DELETE sobre notebook inexistente | Medium | Idempotencia: tratar "sin error" como success, no verificar existencia previa |
| Notebook struct breaking change | Low | Campos nuevos con Option/defaults; list_notebooks sigue devolviendo { id, title } |

## Rollback Plan

Métodos nuevos son aditivos — no modifican existentes. Si falla:
1. Remover las 6 nuevas MCP tools del router
2. Remover los métodos del cliente (compilation garantiza referencia limpia)
3. Remover `src/rpc/notebooks.rs` y `pub mod notebooks` de mod.rs
4. Ningún dato persistente se modifica

## Dependencies

- Ninguna dependencia externa nueva
- RPC IDs extraídos de `teng-lin/notebooklm-py` (reference, no runtime dependency)
- Patrones de parsing defensivo ya establecidos en `src/parser.rs`

## Success Criteria

- [ ] `notebook_delete` MCP tool elimina notebook con idempotencia
- [ ] `notebook_rename` MCP tool renombra y devuelve datos actualizados
- [ ] `notebook_get` MCP tool devuelve details (sources_count, is_owner, created_at)
- [ ] `notebook_summary` MCP tool devuelve resumen AI + suggested topics
- [ ] `notebook_share_status` MCP tool devuelve estado de sharing
- [ ] `notebook_share_set` MCP tool toggle público/privado
- [ ] CLI subcommands funcionan para las 6 operaciones
- [ ] `cargo test` pasa sin errores
- [ ] `cargo clippy` sin warnings nuevos
