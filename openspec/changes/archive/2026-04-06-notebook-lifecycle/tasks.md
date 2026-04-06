# Tasks: Notebook Lifecycle Management (CRUD + Sharing)

## Phase 1: Foundation — Types & Parsers

- [x] 1.1 Crear `src/rpc/notebooks.rs` con: RPC IDs (`WWINqb`, `VfAZjd`, `JFMDGd`, `QDyure`), `ShareAccess` enum (Restricted=0, AnyoneWithLink=1) con `code()`/`from_code()`, structs `SharedUser`, `ShareStatus`, `SuggestedTopic`, `NotebookSummary`. Todos con `Serialize, Deserialize`. Incluir unit tests para `ShareAccess` roundtrip.
- [x] 1.2 Modificar `src/rpc/mod.rs` — agregar `pub mod notebooks;`
- [x] 1.3 Enriquecer struct `Notebook` en `src/notebooklm_client.rs` — agregar `sources_count: usize` (default 0), `is_owner: bool` (default true), `created_at: Option<String>` (default None) con `#[serde(default)]`. Actualizar `list_notebooks()` para seguir usando solo `{id, title}` (no populate nuevos campos). Crear `Notebook::minimal(id, title)` helper si needed.
- [x] 1.4 Agregar parsers en `src/rpc/notebooks.rs`: `parse_notebook_details(inner: &Value) -> Option<Notebook>` (extrae de rLM1Ne response: title[0], id[2], sources_count, is_owner[5][1], created_at[5][5][0]), `parse_share_status(inner: &Value, notebook_id: &str) -> ShareStatus` (formato `[[[users]], [is_public], 1000]`), `parse_summary(inner: &Value) -> NotebookSummary` (formato `[[[summary]], [[topics]]]`). Unit tests con fixtures para cada parser.

## Phase 2: Client Methods

- [x] 2.1 Implementar `delete_notebook(&self, notebook_id: &str) -> Result<(), String>` en `notebooklm_client.rs` — RPC `WWINqb`, payload `[[id], [2]]`, idempotente (Ok(()) si no hay error HTTP).
- [x] 2.2 Implementar `get_notebook(&self, notebook_id: &str) -> Result<Notebook, String>` — RPC `rLM1Ne`, payload `[id, null, [2], null, 0]`, reutiliza `extract_by_rpc_id` + `parse_notebook_details`.
- [x] 2.3 Implementar `rename_notebook(&self, notebook_id: &str, new_title: &str) -> Result<Notebook, String>` — RPC `s0tc2d`, payload `[id, [[null, null, null, [null, title]]]]`, luego invoca `get_notebook()` internamente para devolver datos actualizados.
- [x] 2.4 Implementar `get_summary(&self, notebook_id: &str) -> Result<NotebookSummary, String>` — RPC `VfAZjd`, payload `[id, [2]]`, parser `parse_summary`.
- [x] 2.5 Implementar `get_share_status(&self, notebook_id: &str) -> Result<ShareStatus, String>` — RPC `JFMDGd`, payload `[id, [2]]`, parser `parse_share_status`. Defaults seguros si response null/empty.
- [x] 2.6 Implementar `set_sharing_public(&self, notebook_id: &str, public: bool) -> Result<ShareStatus, String>` — RPC `QDyure`, payload con access code (0=Restricted, 1=AnyoneWithLink), luego invoca `get_share_status()` internamente.

## Phase 3: MCP Tools & CLI

- [x] 3.1 Agregar 6 request structs en `src/main.rs`: `NotebookDeleteRequest { notebook_id }`, `NotebookRenameRequest { notebook_id, new_title }`, `NotebookGetRequest { notebook_id }`, `NotebookSummaryRequest { notebook_id }`, `NotebookShareStatusRequest { notebook_id }`, `NotebookShareSetRequest { notebook_id, public }`. Todas con `Serialize, Deserialize, JsonSchema`.
- [x] 3.2 Agregar 6 métodos `#[tool]` en `NotebookLmServer` (dentro de `#[tool_router] impl`): `notebook_delete`, `notebook_rename`, `notebook_get`, `notebook_summary`, `notebook_share_status`, `notebook_share_set`. Formato output consistente con tools existentes.
- [x] 3.3 Agregar 6 variantes al enum `Commands` en `src/main.rs`: `Delete { notebook_id }`, `Rename { notebook_id, title }`, `Get { notebook_id }`, `Summary { notebook_id }`, `ShareStatus { notebook_id }`, `ShareSet { notebook_id, public: bool }` (con flag `--public`/`--private` mutually exclusive).
- [x] 3.4 Agregar 6 bloques de manejo CLI en `main()` (uno por comando). Patrón: crear client → invocar método → imprimir resultado → `return Ok(())`.

## Phase 4: Verification

- [x] 4.1 Ejecutar `cargo test` — verificar que todos los tests existentes pasan + los nuevos tests de parsers y enums.
- [x] 4.2 Ejecutar `cargo clippy` — sin warnings nuevos. Corregir si hay.
