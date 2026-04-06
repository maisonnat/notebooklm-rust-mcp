# Design: Notebook Lifecycle Management (CRUD + Sharing)

## Technical Approach

Extensión directa del patrón establecido en los 3 módulos anteriores. Cada operación sigue el flujo `batchexecute()` → `extract_by_rpc_id()` → parse defensivo. Se crea un nuevo archivo `src/rpc/notebooks.rs` para los RPC IDs y tipos de sharing, manteniendo la separación por dominio que ya tienen `rpc/artifacts.rs` y `rpc/sources.rs`.

## Architecture Decisions

### AD-1: Notebook struct enrichment (no new struct)

**Choice**: Agregar campos opcionales al struct `Notebook` existente (`sources_count`, `is_owner`, `created_at`) en vez de crear un `NotebookDetails` separado.

**Alternatives**: (A) Crear `NotebookDetails` con todos los campos — obliga a mapear entre structs. (B) Usar `HashMap<String, Value>` — pierde type safety.

**Rationale**: Un solo struct simplifica la API. `list_notebooks()` continúa usando solo `{id, title}`. `get_notebook()` popula los campos extra. Los campos nuevos usan `Default` trait para backward compat.

### AD-2: delete_notebook idempotency via "no error = success"

**Choice**: NO verificar existencia previa. Si `batchexecute(WWINqb)` retorna sin error HTTP, consideramos success. Google ya no retorna error para notebooks inexistentes.

**Alternatives**: (A) `get_notebook()` previo → 2 RPC calls por delete. (B) Intentar parse de la respuesta → frágil.

**Rationale**: El reference Python no hace pre-check. Menos RPCs = menos rate limiting. El spec define idempotencia explícitamente.

### AD-3: Sharing structs en rpc/notebooks.rs (not errors.rs)

**Choice**: `ShareAccess`, `ShareStatus`, `SharedUser`, `NotebookSummary`, `SuggestedTopic` viven en `src/rpc/notebooks.rs`.

**Alternatives**: (A) Ponerlos en `parser.rs` — mixing parsing con domain types. (B) Archivo separado `src/sharing.rs` — overhead para 4 structs pequeños.

**Rationale**: Consistente con `rpc/artifacts.rs` que contiene `ArtifactTypeCode`, `ArtifactStatus`, `GenerationStatus`. Cada dominio tiene sus tipos en su archivo RPC.

### AD-4: set_sharing_public usa post-toggle read pattern

**Choice**: `set_sharing_public()` ejecuta `QDyure` + luego `get_share_status()` para devolver estado confirmado. Igual que `rename_notebook()` hace `s0tc2d` + `get_notebook()`.

**Alternatives**: (A) Devolver bool sin confirmar — no le da feedback al LLM. (B) Parsear la respuesta de QDyure — el Python no lo hace (usa `allow_null=True`).

**Rationale**: Patrones consistentes. El LLM necesita datos confirmados, no suposiciones.

## Data Flow

```
MCP Tool / CLI Command
         │
         ▼
NotebookLmServer (main.rs)
         │  #[tool] macro dispatch
         ▼
NotebookLmClient (notebooklm_client.rs)
         │  batchexecute(rpc_id, payload)
         ▼
batchexecute() ──→ Google batchexecute API
         │
         ▼
extract_by_rpc_id() (parser.rs)
         │
         ▼
parse_notebook_details() / parse_share_status() (parser.rs)
         │
         ▼
Notebook / ShareStatus / NotebookSummary (rpc/notebooks.rs)
```

### Sequence: delete_notebook
```
Client → batchexecute("WWINqb", [[id], [2]])
        → Google returns success (even if not found)
        → Ok(())
```

### Sequence: rename_notebook
```
Client → batchexecute("s0tc2d", [id, [[null,null,null,[null,title]]]])
        → Google returns success
        → get_notebook(id) internally
        → Ok(Notebook { id, title: new_title, ... })
```

### Sequence: set_sharing_public
```
Client → batchexecute("QDyure", [[[id,null,[access],[access,""]]],1,null,[2]])
        → Google returns success
        → get_share_status(id) internally
        → Ok(ShareStatus { is_public, share_url, ... })
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `src/rpc/notebooks.rs` | **Create** | RPC IDs (`WWINqb`, `VfAZjd`, `JFMDGd`, `QDyure`), `ShareAccess` enum, `ShareStatus`/`SharedUser`/`NotebookSummary`/`SuggestedTopic` structs, `parse_notebook_details()`/`parse_share_status()`/`parse_summary()` parsers |
| `src/rpc/mod.rs` | Modify | Add `pub mod notebooks;` |
| `src/notebooklm_client.rs` | Modify | Enrich `Notebook` struct (+3 fields). Add 6 methods: `delete_notebook`, `rename_notebook`, `get_notebook`, `get_summary`, `get_share_status`, `set_sharing_public` |
| `src/main.rs` | Modify | Add 6 request structs + 6 `#[tool]` methods + 6 CLI `Commands` variants |
| `src/parser.rs` | Modify | Add `parse_notebook_details`, `parse_share_status`, `parse_summary` (or put in rpc/notebooks.rs per AD-3) |

## Interfaces / Contracts

### Enriched Notebook struct
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notebook {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub sources_count: usize,
    #[serde(default = "default_true")]
    pub is_owner: bool,
    #[serde(default)]
    pub created_at: Option<String>,
}
```

### ShareAccess enum
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareAccess { Restricted = 0, AnyoneWithLink = 1 }
```

### ShareStatus struct
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareStatus {
    pub notebook_id: String,
    pub is_public: bool,
    pub access: ShareAccess,
    pub shared_users: Vec<SharedUser>,
    pub share_url: Option<String>,
}
```

### NotebookSummary struct
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookSummary {
    pub summary: String,
    pub suggested_topics: Vec<SuggestedTopic>,
}
```

### New client methods (signatures)
```rust
pub async fn delete_notebook(&self, notebook_id: &str) -> Result<(), String>
pub async fn rename_notebook(&self, notebook_id: &str, new_title: &str) -> Result<Notebook, String>
pub async fn get_notebook(&self, notebook_id: &str) -> Result<Notebook, String>
pub async fn get_summary(&self, notebook_id: &str) -> Result<NotebookSummary, String>
pub async fn get_share_status(&self, notebook_id: &str) -> Result<ShareStatus, String>
pub async fn set_sharing_public(&self, notebook_id: &str, public: bool) -> Result<ShareStatus, String>
```

### MCP tools (6 new)
`notebook_delete`, `notebook_rename`, `notebook_get`, `notebook_summary`, `notebook_share_status`, `notebook_share_set`

### CLI commands (6 new)
`delete`, `rename`, `get`, `summary`, `share-status`, `share-set`

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit (rpc/notebooks.rs) | `ShareAccess` code/from_code roundtrip, `ShareStatus` default construction | `#[cfg(test)] mod tests` |
| Unit (parser) | `parse_notebook_details` with various array lengths, `parse_share_status` with null/empty data, `parse_summary` with partial data | Fixture-based with `serde_json::json!` |
| Unit (client) | Payload construction for each RPC (exact JSON output) | String matching on serialized params |
| Integration | `cargo test` + `cargo clippy` | Existing CI |

## Migration / Rollout

No migration required. Todos los cambios son aditivos:
- `Notebook` struct enrichment es backward-compatible (defaults)
- Los 6 métodos nuevos no modifican existentes
- Las 6 MCP tools son nuevas — no rompen tools existentes
- Las 6 CLI commands son nuevas — no rompen commands existentes

## Open Questions

None. Los endpoints y payloads están documentados del reference Python. Los patrones de implementación están establecidos en los módulos archivados. La única incógnita técnica (source_path) se resuelve con testing empírico en la fase Apply.
