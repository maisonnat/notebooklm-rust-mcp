# Proposal: Module 5 — Advanced Features (Source CRUD, Notes, Chat History, Deep Research)

## Intent

Complete the NotebookLM MCP Server with the remaining 5 missing features that prevent full parity with the Python competitor. Currently, sources can only be added (not deleted or renamed), there's no way to read extracted fulltext, notes are unsupported, chat history is local-only (not synced with Google), and deep research is absent.

## Scope

### In Scope
- Source management: delete and rename existing sources
- Fulltext extraction: retrieve Google-indexed text from any source
- Notes CRUD: create, list, and delete notes within notebooks
- Chat history sync: read official conversation history from Google servers
- Deep research: trigger Google's autonomous web research and import results

### Out of Scope
- Deep Research result formatting/parsing beyond source import
- Notes update/edit (only create, list, delete)
- Source move between notebooks
- UI changes

## Capabilities

### New Capabilities
- `source-management`: Delete and rename existing sources in a notebook
- `source-fulltext`: Extract the full indexed text from any source
- `notes`: Create, list, and delete notes within notebooks
- `chat-history`: Sync and read official chat history from Google servers
- `deep-research`: Trigger autonomous web research and import sources

### Modified Capabilities
- `multi-source-ingestion`: Adding fulltext extraction as a downstream capability of source operations

## Approach

Follow the established Batch Execute pattern. Each feature maps to one or more Google RPC IDs discovered from the Python implementation. Implementation proceeds in order of complexity: source management (simplest) → fulltext → notes (two-step) → chat history → deep research (most complex, async polling).

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/notebooklm_client.rs` | Modified | 9 new methods (delete/rename source, get fulltext, notes CRUD, chat history, deep research) |
| `src/main.rs` | Modified | 8 new MCP tools + 8 new CLI commands |
| `src/parser.rs` | Modified | Recursive text extractor for fulltext, conversation turn parser, note filter |
| `src/rpc/notes.rs` | New | Note types and payload builders |
| `src/rpc/research.rs` | New | Deep research types and payload builders |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| RPC IDs may have changed | Medium | Defensive parsing, test each RPC individually |
| Deep research polling timeout | Medium | Configurable timeout, graceful failure |
| Notes two-step process may race | Low | Sequential await, no parallelism in create flow |
| Chat history format undocumented | High | Sample real responses, defensive parsing with fallback |

## Rollback Plan

Each feature is independent. If any RPC fails validation, comment out the MCP tool and client method. No database or schema changes — purely additive.

## Dependencies

- Existing `batchexecute()` infrastructure
- Existing `artifact_poller` pattern (reuse for deep research polling)
- Real NotebookLM account for E2E validation

## Success Criteria

- [ ] All 8 new MCP tools return correct results
- [ ] All 8 new CLI commands work end-to-end
- [ ] `cargo test` passes with new unit tests
- [ ] `cargo clippy` has 0 warnings
- [ ] Existing tools unaffected (no regressions)
