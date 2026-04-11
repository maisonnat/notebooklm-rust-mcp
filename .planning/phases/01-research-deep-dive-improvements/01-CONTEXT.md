# Phase 1: Research Deep Dive Improvements - Context

**Gathered:** 2026-04-11
**Status:** Ready for planning

## Phase Boundary

Mejorar la API de research deep dive del MCP server para que el agente pueda iniciar una investigación sin bloquearse, verificar su progreso cuando quiera, y obtener el reporte markdown completo cuando termine. Mantener backward compatibility con la tool existente.

## Implementation Decisions

### D-01: Tool Architecture — Split
- **Decision:** Split `research_deep_dive` into two tools:
  - `research_deep_dive_start` — Fires research, returns `task_id` immediately
  - `research_deep_dive_status` — Non-blocking single poll check
- **Rationale:** ALL competitors use split. MCP protocol is request-response, no native async. Blocking tool freezes the agent for minutes.
- **Competitor evidence:** m4yk3ldev/notebooklm-mcp (32 tools), teng-lin/notebooklm-py (500+ stars), claude-world/notebooklm-skill — ALL use split pattern.

### D-02: Status Mode — Non-blocking Only
- **Decision:** `research_deep_dive_status` is non-blocking (single poll). No built-in "wait" mode.
- **Rationale:** Agent MCP can loop externally when needed. Prevents long-running tool calls.
- **Competitor evidence:** m4yk3ldev uses non-blocking only. Python SDK offers `wait` but it's a client-side loop, not a server-side block.

### D-03: Report Markdown Extraction
- **Decision:** Extract and return full report markdown from deep research results.
- **Rationale:** Deep research generates a comprehensive markdown report. We currently discard it (`sources: Vec::new()` always empty). Python SDK extracts it. This is a direct competitive advantage.
- **Parsing formats to support (from Python SDK):**
  - Fast research sources: `[url, title, desc, type, ...]`
  - Deep research (current): `[None, [title, report_markdown], None, type, ...]`
  - Deep research (legacy): `[None, title, None, type, ..., [chunk1, chunk2, ...]]`

### D-04: Polling Strategy — Exponential Backoff
- **Decision:** Exponential backoff with jitter, reusing ArtifactPoller pattern.
- **Pattern:** 2s → 4s → 8s → 10s max (cap)
- **Rationale:** Already implemented in `src/artifact_poller.rs`. No competitor does this (all use fixed 5s). Better UX — faster initial detection, less server load on long waits.
- **Competitor evidence:** Python SDK uses fixed 5s. m4yk3ldev uses fixed interval. We're the only ones with backoff.

### D-05: Backward Compatibility — Deprecated Wrapper
- **Decision:** Keep existing `research_deep_dive` as deprecated wrapper.
- **Implementation:** Old tool internally calls `start` + `status` loop. New tools coexist. Old tool marked deprecated in description, to be removed in next major release.
- **Rationale:** Professional approach. No breaking changes for existing MCP clients. Clean migration path.

## Canonical References

### Core Implementation Files
- `src/main.rs` lines 918-970 — Current `research_deep_dive` tool definition
- `src/notebooklm_client.rs` lines 2085-2194 — `start_deep_research()` and `poll_research_status()` methods
- `src/artifact_poller.rs` — Exponential backoff pattern to reuse (line 55-56: backoff config)

### Reference Implementation (Python)
- `teng-lin/notebooklm-py/src/notebooklm/_research.py` — `start()` and `poll()` methods, source parsing
- `teng-lin/notebooklm-py/src/notebooklm/cli/research.py` — CLI `status` and `wait` commands
- `teng-lin/notebooklm-py/docs/rpc-reference.md` — RPC method docs, status code semantics

### Competitor MCP Implementations
- `m4yk3ldev/notebooklm-mcp` — TypeScript MCP, 32 tools, split pattern `research_start` / `research_status` / `research_import`
- `claude-world/notebooklm-skill` — Python MCP, pipeline pattern

### RPC Method Details
- RPC `e3bVqc` — Poll research status
- Status codes: `1` = in_progress, `2` = completed, `6` = completed (deep research)
- RPC `LBwxtb` — Import research results

## Existing Code Insights

### Reusable Assets
- `ArtifactPoller` struct — Exponential backoff pattern with `BackoffConfig` (base 2s, cap 10s, jitter)
- `PollStatus` enum — Standardized status reporting pattern
- `ResearchTaskStatus` struct — Already has status code parsing, just needs report extraction

### Established Patterns
- Tool registration: `ToolRouter` in main.rs, `ToolHandler` trait
- Polling: `Poller` trait with `create_request()` + `parse_response()` + `is_complete()`
- Response formatting: `CallToolResult::success()` with JSON content

### Integration Points
- Tool router at `src/main.rs` line 625-661 — where new tools get registered
- `NotebookLmClient` at `src/notebooklm_client.rs` — where RPC methods live
- JSON-RPC response parsing at lines 2109-2143 — where poll response is parsed

## Specific Ideas

- Competitor `m4yk3ldev` returns: `{ task_id, message: "Research started. Use research_status to poll progress." }` from start
- Competitor status returns: `{ results }` with all tasks or specific task
- Python `poll()` returns: `{ task_id, status, query, sources, summary, report, tasks[] }`
- Our status should include: `status`, `task_id`, `query`, `sources_found` (count), `sources` (list), `report` (markdown), `elapsed_seconds`

## Deferred Ideas

None — all decisions are within the phase scope.

---

*Phase: 01-research-deep-dive-improvements*
*Context gathered: 2026-04-11*
