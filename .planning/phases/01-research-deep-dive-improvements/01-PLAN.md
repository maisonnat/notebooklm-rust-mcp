# Phase 1: Research Deep Dive Improvements — Plan

**Status:** Ready for execution
**Created:** 2026-04-11

## Goal

Split the blocking `research_deep_dive` MCP tool into `research_deep_dive_start` (non-blocking) + `research_deep_dive_status` (non-blocking check), extract report markdown from deep research results, implement exponential backoff polling, and keep the old tool as a deprecated wrapper.

## Requirements

- REQ-01: `research_deep_dive_start` returns `task_id` immediately without blocking
- REQ-02: `research_deep_dive_status` returns current status as single non-blocking poll
- REQ-03: Deep research report markdown is extracted and returned in status response
- REQ-04: Research polling uses exponential backoff with jitter (2s → 4s → 8s → 10s cap)
- REQ-05: Existing `research_deep_dive` tool continues to work as deprecated wrapper
- REQ-06: Status response includes: status, task_id, query, sources_found, sources, report, elapsed_seconds
- REQ-07: All existing tests pass, new tests cover the new behavior

## Tasks

### T-01: Create `ResearchDeepDivePoller` struct with exponential backoff
**File:** `src/research_poller.rs` (new file)
**Estimated:** 30min
**Depends on:** nothing

- Create `ResearchDeepDivePoller` struct following `ArtifactPoller` pattern
- Use `BackoffConfig { base_secs: 2, cap_secs: 10, max_retries: None }`
- Implement `Poller` trait: `create_request()` using RPC `e3bVqc`, `parse_response()`, `is_complete()`
- Parse ALL THREE source formats from Python SDK:
  - Fast research: `[url, title, desc, type, ...]`
  - Deep research (current): `[None, [title, report_markdown], None, type, ...]`
  - Deep research (legacy): `[None, title, None, type, ..., [chunk1, chunk2, ...]]`
- Extract report markdown into `ResearchTaskStatus.report` field
- Add `report: Option<String>` to `ResearchTaskStatus` struct

### T-02: Add `research_deep_dive_start` tool
**Files:** `src/main.rs`, `src/notebooklm_client.rs`
**Estimated:** 20min
**Depends on:** nothing

- Create `research_deep_dive_start` tool definition in main.rs
- Reuse existing `start_deep_research()` client method (already works, already returns task_id)
- Return JSON: `{ "task_id": "...", "notebook_id": "...", "query": "...", "status": "started" }`
- Register in tool router

### T-03: Add `research_deep_dive_status` tool
**Files:** `src/main.rs`, `src/notebooklm_client.rs`
**Estimated:** 30min
**Depends on:** T-01

- Create `research_deep_dive_status` tool definition in main.rs
- Call poll RPC once (non-blocking), use new `ResearchDeepDivePoller` for parsing
- Accept optional `task_id` parameter (omit = latest task)
- Return JSON with full status including:
  - `status`: "in_progress" | "completed" | "no_research"
  - `task_id`: research task identifier
  - `query`: original query text
  - `sources_found`: count of sources discovered
  - `sources`: array of source objects (url, title, type)
  - `report`: full markdown report (deep research only)
  - `elapsed_seconds`: time since research started
- Register in tool router

### T-04: Create deprecated wrapper for old `research_deep_dive`
**File:** `src/main.rs`
**Estimated:** 15min
**Depends on:** T-02, T-03

- Keep existing `research_deep_dive` tool name and schema
- Update description to mark as deprecated: "DEPRECATED: Use research_deep_dive_start + research_deep_dive_status instead. This tool will be removed in the next major release."
- Internal implementation: call `start` then loop `status` with exponential backoff until complete or timeout
- Return same response format as before (backward compatible)

### T-05: Update tool descriptions and schemas
**File:** `src/main.rs`
**Estimated:** 10min
**Depends on:** T-02, T-03

- Add tool descriptions that reference each other: "Use research_deep_dive_status to poll progress"
- Ensure `timeout_secs` parameter in old tool uses same backoff logic

### T-06: Add module declarations and imports
**File:** `src/main.rs`
**Estimated:** 5min
**Depends on:** T-01

- Add `mod research_poller;` to main.rs
- Add `use crate::research_poller::ResearchDeepDivePoller;`

### T-07: Write unit tests
**Files:** `src/research_poller.rs`, inline in `src/main.rs`
**Estimated:** 30min
**Depends on:** T-01, T-02, T-03, T-04

- Test `ResearchDeepDivePoller` parsing all 3 source formats
- Test report markdown extraction (current format)
- Test report markdown extraction (legacy format with chunks)
- Test exponential backoff progression
- Test `is_complete()` for status codes 1, 2, 6
- Test deprecated wrapper produces same output as before
- Test start tool returns immediately
- Test status tool for in_progress and completed states

### T-08: Run full test suite and clippy
**Estimated:** 10min
**Depends on:** T-07

- `cargo test` — all 329+ existing tests pass
- `cargo clippy` — zero warnings
- New tests for research_poller pass

## File Changes Summary

| File | Action | Description |
|------|--------|-------------|
| `src/research_poller.rs` | CREATE | ResearchDeepDivePoller with backoff, report extraction |
| `src/main.rs` | MODIFY | Add 2 new tools, deprecated wrapper, module declaration |
| `src/notebooklm_client.rs` | MODIFY | Add report field to ResearchTaskStatus, minor changes |
| `src/rpc/notes.rs` | MODIFY (maybe) | Add report field to ResearchTaskStatus struct |

## Risk Assessment

| Risk | Severity | Mitigation |
|------|----------|------------|
| RPC response format change by Google | Medium | Defensive parsing, fallback to empty, log warnings |
| Breaking old tool behavior | Low | Deprecated wrapper preserves exact behavior |
| Backoff too aggressive for rate limits | Low | Cap at 10s, jitter prevents thundering herd |

## Success Criteria

1. `research_deep_dive_start` returns task_id in < 2 seconds
2. `research_deep_dive_status` returns current status without blocking
3. Deep research report markdown is extracted and non-empty when research completes
4. Old `research_deep_dive` tool works exactly as before (backward compat)
5. All tests pass, zero clippy warnings
6. Response format matches competitor pattern (m4yk3ldev/notebooklm-mcp)
