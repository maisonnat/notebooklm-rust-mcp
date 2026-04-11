# Roadmap — NotebookLM MCP Server

## Phase 1: Research Deep Dive Improvements

**Goal:** Split `research_deep_dive` into `start` + `status` tools, improve polling with exponential backoff, extract report markdown.

### Scope
- Split current blocking `research_deep_dive` into two tools:
  - `research_deep_dive_start`: Start research, return task_id immediately
  - `research_deep_dive_status`: Check status (non-blocking single check OR blocking wait)
- Add exponential backoff to polling (like ArtifactPoller already does)
- Extract report markdown from deep research results (Python does this, we don't)
- Return source count and summary during polling for progress visibility

### References
- `src/notebooklm/_research.py` — Python reference implementation
- `src/notebooklm/cli/research.py` — Python CLI `status` and `wait` commands
- `docs/rpc-reference.md` — RPC method documentation (e3bVqc status codes)
- `src/artifact_poller.rs` — Existing exponential backoff pattern to reuse
- MCP specification — Tools are request-response, no native async
