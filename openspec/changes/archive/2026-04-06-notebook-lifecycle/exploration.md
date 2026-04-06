# Exploration: Notebook Lifecycle Management (CRUD + Sharing)

## Current State

El servidor MCP NotebookLM tiene 3 módulos archivados que cubren:
- **Auth** (browser + DPAPI)
- **Multi-Source** (text, URL, YouTube, Drive, file upload)
- **Artifact Generation & Download** (9 tipos, polling, streaming download)

Actualmente el servidor puede:
- `list_notebooks()` via RPC `wXbhsf` — devuelve `Vec<Notebook { id, title }>`
- `create_notebook(title)` via RPC `CCqFvf` — devuelve `String` (UUID)
- `get_notebook_sources(notebook_id)` via RPC `rLM1Ne` — usa el mismo RPC que `GET_NOTEBOOK`
- `delete_artifact(notebook_id, artifact_id)` via RPC `V5N4be`

**Lo que NO puede hacer (gap del Módulo 3):**
- Eliminar un notebook
- Renombrar un notebook
- Obtener detalles completos de un notebook (sources_count, is_owner, created_at)
- Obtener resumen AI de un notebook
- Gestionar sharing (público/privado, usuarios, permisos)

## Reference: Endpoints del Python (teng-lin/notebooklm-py)

### Notebook Operations
| Operación | RPC ID | Payload Python | Estado en Rust |
|-----------|--------|----------------|----------------|
| LIST_NOTEBOOKS | `wXbhsf` | `[None, 1, None, [2]]` | ✅ Implementado |
| CREATE_NOTEBOOK | `CCqFvf` | `[title, None, None, [2], [1]]` | ✅ Implementado |
| GET_NOTEBOOK | `rLM1Ne` | `[notebook_id, None, [2], None, 0]` | ⚠️ Parcial — se usa para sources, no para detalles del notebook |
| RENAME_NOTEBOOK | `s0tc2d` | `[notebook_id, [[None, None, None, [None, new_title]]]]` | ❌ No existe |
| DELETE_NOTEBOOK | `WWINqb` | `[[notebook_id], [2]]` | ❌ No existe |
| SUMMARIZE | `VfAZjd` | `[notebook_id, [2]]` | ❌ No existe |
| REMOVE_RECENTLY_VIEWED | `fejl7e` | `[notebook_id]` | ❌ No existe |

### Sharing Operations
| Operación | RPC ID | Payload Python | Estado en Rust |
|-----------|--------|----------------|----------------|
| GET_SHARE_STATUS | `JFMDGd` | `[notebook_id, [2]]` | ❌ No existe |
| SHARE_NOTEBOOK (set public) | `QDyure` | `[[[notebook_id, None, [access], [access, ""]]], 1, None, [2]]` | ❌ No existe |
| SHARE_NOTEBOOK (add user) | `QDyure` | `[[[notebook_id, [[email, None, perm]], None, [msg_flag, welcome]]], notify, None, [2]]` | ❌ No existe |
| SHARE_NOTEBOOK (remove user) | `QDyure` | `[[[notebook_id, [[email, None, 4]], None, [0, ""]]], 0, None, [2]]` | ❌ No existe |
| SET_VIEW_LEVEL | `s0tc2d` | `[notebook_id, [[None, None, None, None, None, None, None, None, [[level]]]]]` | ❌ No existe |

### Notas clave del reference
1. **RENAME_NOTEBOOK (s0tc2d)** es dual-use: también se usa para `set_view_level` con payload diferente
2. **SHARE_NOTEBOOK (QDyure)** es triple-use: set public, add user, remove user — payload diferente para cada caso
3. **GET_NOTEBOOK (rLM1Ne)** ya lo tenemos para sources — el Python lo usa con el mismo payload para obtener detalles del notebook (title, sources_count, is_owner, created_at)
4. **source_path** en el Python es un parámetro URL (`/notebook/{id}` o `/`) — en Rust nuestro `batchexecute` no lo usa (siempre manda a la misma URL). Esto puede requerir investigación.
5. **allow_null=True** en el Python indica que la respuesta puede ser null/empty — hay que manejarlo con parsing defensivo.

## Affected Areas

- `src/notebooklm_client.rs` — +5 métodos públicos (delete, rename, get_notebook, get_summary, sharing)
- `src/rpc/notebooks.rs` — **New**: RPC IDs para notebook + sharing, enums de sharing
- `src/rpc/mod.rs` — Modified: agregar `pub mod notebooks`
- `src/main.rs` — Modified: +3-5 MCP tools, +3-5 CLI subcommands
- `src/parser.rs` — Modified: +parsers para GET_NOTEBOOK response (sources_count, is_owner, created_at), ShareStatus
- `src/errors.rs` — Modified: +variantes si aplica

## Approaches

### Approach 1: Core CRUD + Sharing (Recomendado)

**Scope mínimo pero completo**: delete, rename, get_notebook (detalles), get_summary, sharing completo.

Operaciones nuevas:
1. `delete_notebook(id)` — RPC `WWINqb`, idempotente (404 = success)
2. `rename_notebook(id, new_title)` — RPC `s0tc2d`, devuelve Notebook actualizado
3. `get_notebook(id)` — RPC `rLM1Ne`, devuelve Notebook con metadata completa
4. `get_summary(id)` — RPC `VfAZjd`, devuelve resumen AI + suggested topics
5. `get_share_status(id)` — RPC `JFMDGd`, devuelve estado de sharing
6. `set_sharing_public(id, public)` — RPC `QDyure`, toggle público/privado
7. `share_with_user(id, email, permission)` — RPC `QDyure`, compartir con usuario
8. `remove_user_share(id, email)` — RPC `QDyure`, remover acceso

- Pros: Cobertura completa del ciclo de vida del notebook. MCP tools ricas para agentes IA.
- Cons: Más trabajo (8 operaciones). Sharing con user management tiene payload complejo.
- Effort: Medium

### Approach 2: Core CRUD Only (Mínimo viable)

Solo las 3 operaciones del roadmap del usuario:
1. `delete_notebook(id)` — RPC `WWINqb`
2. `rename_notebook(id, new_title)` — RPC `s0tc2d`
3. `get_notebook(id)` — RPC `rLM1Ne` (enriquecer struct Notebook existente)

- Pros: Rápido, directo, cumple el roadmap exacto.
- Cons: Sin sharing, sin resumen AI. El usuario pidió sharing como "opcional".
- Effort: Low

### Approach 3: Core CRUD + Sharing Básico (Público/Privado)

3 operaciones core + toggle público/privado:
1. `delete_notebook(id)`
2. `rename_notebook(id, new_title)`
3. `get_notebook(id)`
4. `set_sharing_public(id, public)` — solo toggle, sin user management
5. `get_share_status(id)` — solo lectura

- Pros: Buen balance entre utilidad y esfuerzo. MCP tools suficientes para automatización básica.
- Cons: Sin user-level sharing (add_user, remove_user, set_view_level).
- Effort: Low-Medium

## Recommendation

**Appro 3** (Core CRUD + Sharing Básico). Razones:

1. Las 3 operaciones core (delete, rename, get_notebook) son straightforward — payloads simples, parsing defensivo ya establecido.
2. Sharing básico (público/privado + get_status) agrega MUCHO valor para automatización MCP con poco esfuerzo extra — un solo RPC (`QDyure`) con payload simple.
3. User-level sharing (add_user, remove_user, set_view_level) puede ser un módulo separado (Módulo 3b) si se necesita. El payload es más complejo y el caso de uso para un MCP server es menos claro.
4. `get_summary` se puede incluir como bonus — payload trivial (`[id, [2]]`), parsing simple.

**Operaciones finales propuestas:**
1. `delete_notebook(id)` — idempotente
2. `rename_notebook(id, new_title)` — devuelve Notebook
3. `get_notebook(id)` — Notebook enriquecida (sources_count, is_owner, created_at)
4. `get_summary(id)` — resumen AI + suggested topics (bonus)
5. `get_share_status(id)` — lectura del estado de sharing
6. `set_sharing_public(id, public)` — toggle público/privado

## Risks

- **source_path no implementado**: El Python usa `source_path` para cambiar la URL del batchexecute. Nuestro `batchexecute()` siempre va a la misma URL. Puede que algunos RPCs necesiten un path diferente — requiere testing empírico.
- **RENAME_NOTEBOOK dual-use**: El RPC `s0tc2d` sirve tanto para rename como para set_view_level. Payloads completamente diferentes. Si en el futuro queremos view_level, hay que tener cuidado de no romper rename.
- **SHARE_NOTEBOOK null response**: El Python marca `allow_null=True` para sharing. Puede que Google devuelva null/empty — parsing defensivo obligatorio.
- **DELETE idempotencia**: Google puede no devolver error para notebook inexistente. Necesitamos decidir si tratamos "no error" como success idempotente, o si hacemos un get previo para verificar existencia.
- **Notebook struct enriquecida**: Actualmente `Notebook { id, title }`. Enriquecerla con `sources_count`, `is_owner`, `created_at` requiere parser nuevo. Backward-compatible si los campos nuevos son `Option`.

## Ready for Proposal
Yes. El scope está claro: 6 operaciones (3 core + summary + sharing básico). Los endpoints y payloads están documentados del reference. Los patrones de implementación están bien establecidos en los módulos anteriores.
