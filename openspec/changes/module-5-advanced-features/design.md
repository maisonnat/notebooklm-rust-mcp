# Design: Module 5 — Advanced Features

## Architecture Overview

Five independent features following the established Batch Execute pattern. Each maps to specific Google RPC IDs discovered from the Python implementation. All features are additive — no existing behavior changes.

## RPC Mapping

| Feature | RPC ID(s) | Direction | Pattern |
|---------|-----------|-----------|---------|
| Delete Source | `tGMBJ` | Mutation | Single call, triple-nested payload |
| Rename Source | `b7Wfje` | Mutation | Single call, asymmetric payload |
| Get Fulltext | `hizoJc` | Read | Single call, recursive parse |
| Create Note | `CYK0Xb` → `cYAfTb` | Two-step | Create empty, then update |
| List Notes | `cFji9` | Read | Single call, filter status=2 |
| Delete Note | `AH0mwd` | Mutation (soft) | Single call |
| Get Conversation ID | `hPTbtc` | Read | Single call, triple-nested extraction |
| Get Conversation Turns | `khqZz` | Read | Single call, reverse order |
| Start Research | `QA9ei` | Async start | Returns task_id |
| Poll Research | `e3bVqc` | Async poll | Reuse artifact_poller pattern |
| Import Research | `LBwxtb` | Mutation | Import sources + report |

## Module Changes

### `src/notebooklm_client.rs`

9 new methods added following the existing `batchexecute()` pattern:

```rust
// Source management
pub async fn delete_source(&self, notebook_id: &str, source_id: &str) -> Result<(), String>
pub async fn rename_source(&self, notebook_id: &str, source_id: &str, new_title: &str) -> Result<(), String>
pub async fn get_source_fulltext(&self, notebook_id: &str, source_id: &str) -> Result<String, String>

// Notes
pub async fn create_note(&self, notebook_id: &str, title: &str, content: &str) -> Result<String, String>
pub async fn list_notes(&self, notebook_id: &str) -> Result<Vec<Note>, String>
pub async fn delete_note(&self, notebook_id: &str, note_id: &str) -> Result<(), String>

// Chat history
pub async fn get_last_conversation_id(&self, notebook_id: &str) -> Result<Option<String>, String>
pub async fn get_conversation_turns(&self, notebook_id: &str, conversation_id: &str, limit: u32) -> Result<Vec<ChatTurn>, String>

// Deep research
pub async fn start_deep_research(&self, notebook_id: &str, query: &str) -> Result<String, String>
pub async fn poll_research_status(&self, notebook_id: &str, task_id: &str) -> Result<ResearchStatus, String>
pub async fn import_research_sources(&self, notebook_id: &str, task_id: &str, sources: Value) -> Result<(), String>
```

### `src/rpc/notes.rs` (New)

```rust
pub struct Note { pub id: String, pub title: String, pub content: String }
pub struct ChatTurn { pub role: String, pub text: String }
pub struct ResearchStatus { pub status_code: u32, pub sources: Vec<Value>, pub is_complete: bool }
```

### `src/parser.rs`

New parsing functions:

```rust
pub fn extract_all_text(val: &Value, current_depth: u32, max_depth: u32) -> Vec<String>
pub fn parse_conversation_turns(data: &Value) -> Vec<ChatTurn>
pub fn parse_notes(data: &Value) -> Vec<Note>
pub fn parse_research_status(data: &Value, task_id: &str) -> Option<ResearchStatus>
```

### `src/main.rs`

8 new `#[tool]` methods + 8 new CLI Commands enum variants + request structs.

## Design Decisions

### DD-1: Notes two-step process

**Decision**: Wrap the two-step (create empty → update) in a single public method `create_note()`.

**Rationale**: MCP tools and CLI commands should expose a single operation. The internal two-step is an implementation detail.

### DD-2: Chat history integration with ask_question

**Decision**: Make chat history sync opt-in rather than automatic in `ask_question`.

**Rationale**: Automatic sync on every question adds latency. Better to provide a separate `chat_history` tool that the AI agent can call when needed, and optionally integrate with `ask_question` via a flag.

### DD-3: Deep research as blocking tool

**Decision**: The `research_deep_dive` MCP tool blocks until completion (using tokio::sleep polling loop).

**Rationale**: AI agents expect a single response. Returning a task_id would require the agent to poll manually — the blocking pattern provides a better UX for MCP consumers.

### DD-4: Recursive text extractor depth limit

**Decision**: Default max_depth of 10 with a hard cap at 20.

**Rationale**: Google's responses are deeply nested but finite. 10 levels covers all observed cases. The hard cap prevents runaway recursion on malformed responses.

### DD-5: Notes soft-delete

**Decision**: `delete_note` performs a soft-delete (status=2), not a hard delete.

**Rationale**: This matches Google's behavior. The list parser filters out status=2 entries, providing the appearance of deletion.

## Sequence Diagrams

### Source Delete Flow

```
MCP Tool → delete_source() → batchexecute(tGMBJ, [[[source_id]]]) → Google → Confirmation
```

### Note Create Flow (Two-Step)

```
MCP Tool → create_note()
  → batchexecute(CYK0Xb, [notebook_id, "", [1], null, "New Note"]) → Google → note_id
  → batchexecute(cYAfTb, [notebook_id, note_id, [[[content, title, [], 0]]]]) → Google → Updated
  → Return note_id
```

### Deep Research Flow

```
MCP Tool → start_deep_research() → task_id
  → loop { poll_research_status() → check status_code }
  → if complete { import_research_sources() }
  → Return summary
```
