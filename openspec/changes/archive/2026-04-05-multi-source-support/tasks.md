# Tasks: Multi-Source Support

## Phase 1: Foundation (deps + types)

- [x] 1.1 Add `url = "2"` and `tokio-util = { version = "0.7", features = ["io"] }` to `Cargo.toml` dependencies
- [x] 1.2 Create `src/rpc/mod.rs` with `pub mod sources;`
- [x] 1.3 Create `src/rpc/sources.rs`: define `UrlSourceInner`, `YoutubeSourceInner`, `DriveSourceInner`, `UploadSessionBody` + builder functions + 10 unit tests
- [x] 1.4 Add `FileNotFound(String)`, `UploadFailed(String)`, `ValidationError(String)` variants to `NotebookLmError` in `src/errors.rs` + update `Display`, `from_string()`
- [x] 1.5 Add `pub mod rpc;` declaration in `src/main.rs`

## Phase 2: URL & YouTube Sources

- [x] 2.1 Add `is_youtube_url(url: &str) -> bool` helper in `src/notebooklm_client.rs` using `url::Url` hostname matching against `{youtube.com, www.youtube.com, m.youtube.com, music.youtube.com, youtu.be}`
- [x] 2.2 Implement `add_url_source(&self, notebook_id: &str, url: &str) -> Result<String, String>` in `NotebookLmClient`: branch YouTube vs regular payload, call `batchexecute("izAoDd", ...)`, extract source_id via parser
- [x] 2.3 Add `SourceAddUrlRequest { notebook_id, url }` struct + `source_add_url` MCP tool in `main.rs`
- [x] 2.4 Add `AddUrl` CLI subcommand (args: `--notebook-id`, `--url`) in `main.rs`

## Phase 3: Google Drive Source

- [x] 3.1 Implement `add_drive_source(&self, notebook_id: &str, file_id: &str, title: &str, mime_type: &str) -> Result<String, String>` in `NotebookLmClient`: single-wrapped source_data, RPC `izAoDd`, default mime_type `application/vnd.google-apps.document`
- [x] 3.2 Add `SourceAddDriveRequest { notebook_id, file_id, title, mime_type: Option<String> }` + `source_add_drive` MCP tool
- [x] 3.3 Add `AddDrive` CLI subcommand (args: `--notebook-id`, `--file-id`, `--title`, `--mime-type`)

## Phase 4: File Upload (3-step protocol)

- [x] 4.1 Add `upload_http: Client` field to `NotebookLmClient` (no global Content-Type) — construct in `new()` with cookie header only
- [x] 4.2 Implement `_register_file_source(&self, notebook_id: &str, filename: &str) -> Result<String, String>`: RPC `o4cbdc` with `[[filename]]` params, extract nested SOURCE_ID via new parser fn
- [x] 4.3 Add `extract_nested_source_id(response: &Value) -> Option<String>` to `src/parser.rs`: recursively unwrap `[[[[id]]]]` nesting
- [x] 4.4 Implement `_start_resumable_upload(&self, notebook_id: &str, filename: &str, file_size: u64, source_id: &str) -> Result<String, String>`: POST to `/upload/_/?authuser=0`, extract `x-goog-upload-url` header
- [x] 4.5 Implement `_stream_upload_file(&self, upload_url: &str, file_path: &Path) -> Result<(), String>`: `tokio::fs::File` → `ReaderStream` → `Body::wrap_stream()` in 64KB chunks, acquire `upload_semaphore` permit
- [x] 4.6 Implement `add_file_source(&self, notebook_id: &str, file_path: &str) -> Result<String, String>`: validate path exists + is_file, get file_size, orchestrate steps 1→2→3, map errors to `FileNotFound`/`ValidationError`/`UploadFailed`
- [x] 4.7 Add `SourceAddFileRequest { notebook_id, file_path }` + `source_add_file` MCP tool
- [x] 4.8 Add `AddFile` CLI subcommand (args: `--notebook-id`, `--file-path`)

## Phase 5: Source Polling Enhancement

- [x] 5.1 Update `SourceState::from_response()` in `src/source_poller.rs` to parse status code from `src[3][1]` (1=Processing, 2=Ready, 3=Error) instead of presence-only check
- [x] 5.2 Add `SourceState::Error(String)` variant usage in `wait_for_source_ready()` — return `NotebookLmError::SourceNotReady` with error message on Error state

## Phase 6: Testing

- [x] 6.1 Unit tests for `is_youtube_url()`: watch, short, embed, live, non-youtube, invalid
- [x] 6.2 Unit tests for RPC payload serialization: verify JSON output of each `SourcePayload` variant matches rpc-reference.md format
- [x] 6.3 Unit tests for `extract_nested_source_id()`: `[[[[id]]]]`, `[[[id]]]`, `[[id]]`, null, empty
- [x] 6.4 Unit tests for `SourceState::from_response()` with status codes 1, 2, 3 and edge cases
- [x] 6.5 Unit tests for new error variants in `NotebookLmError::from_string()` (file, upload, validation patterns)
- [x] 6.6 Run `cargo test` and `cargo clippy` — zero errors, zero new warnings
