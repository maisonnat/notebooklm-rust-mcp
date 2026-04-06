# Tasks: Artifact Generation & Download

## Phase 1: Foundation — Types & Errors

- [x] 1.1 Add `scraper` dependency to Cargo.toml (HTML parsing for quiz/flashcard)
- [x] 1.2 Create `src/rpc/artifacts.rs` with all enums:
  - `ArtifactTypeCode` (int: AUDIO=1..DATA_TABLE=9)
  - `ArtifactStatus` (int: PROCESSING=1..FAILED=4)
  - `ArtifactType` (user-facing: Audio, Video, Report, Quiz, Flashcards, MindMap, Infographic, SlideDeck, DataTable, Unknown)
  - `AudioFormat`, `AudioLength`
  - `VideoFormat`, `VideoStyle`
  - `QuizQuantity`, `QuizDifficulty`
  - `InfographicOrientation`, `InfographicDetail`, `InfographicStyle`
  - `SlideDeckFormat`, `SlideDeckLength`
  - `ReportFormat` (str enum: BriefingDoc, StudyGuide, BlogPost, Custom)
  - + unit tests for all enum values
- [x] 1.3 Add 4 error variants to `NotebookLmError`: `ArtifactNotReady`, `ArtifactNotFound`, `DownloadFailed`, `GenerationFailed`
  - Update `Display`, `from_string()` keyword detection
  - + unit tests for new variants
- [x] 1.4 Add `pub mod artifacts` to `src/rpc/mod.rs`

## Phase 2: Core — ArtifactConfig & Payload Builders

- [x] 2.1 Define `ArtifactConfig` enum with variants: Audio, Video, Report, Quiz, Flashcards, Infographic, SlideDeck, DataTable
  - Each variant has ONLY its valid config fields (type-safe)
  - No MindMap variant (handled by separate method)
  - No CinematicVideo variant (it's Video { format: Cinematic, style: None })
- [x] 2.2 Implement `ArtifactConfig::to_params_array()` dispatcher — maps each variant to the correct positional JSON array
  - Audio: type_code=1, config at index 6
  - Report: type_code=2, config at index 7
  - Video: type_code=3, config at index 8
  - Quiz: type_code=4, variant=2, config at index 9, [quantity, difficulty] at [7]
  - Flashcards: type_code=4, variant=1, config at index 9, [difficulty, quantity] at [6] — REVERSED!
  - Infographic: type_code=7, config at index 14
  - SlideDeck: type_code=8, config at index 16
  - DataTable: type_code=9, config at index 18
- [x] 2.3 Implement source ID formatting helpers: `to_triple_nested()` and `to_double_nested()`
- [x] 2.4 Implement report template prompts (built-in templates for BriefingDoc, StudyGuide, BlogPost)
- [x] 2.5 Unit tests: verify each ArtifactConfig variant produces the exact expected JSON array (16 tests — one per type + edge cases)

## Phase 3: Discovery — List & Parse Artifacts

- [x] 3.1 Add `parse_artifact_list()` to `src/parser.rs` — parse LIST_ARTIFACTS response into Vec of raw Value arrays
- [x] 3.2 Implement `Artifact::from_api_response(data: &Value)` — parse a single artifact array into typed struct
- [x] 3.3 Implement `client.list_artifacts(notebook_id) -> Vec<Artifact>` — RPC call to LIST_ARTIFACTS (gArtLc) with filter param
- [x] 3.4 Unit tests for artifact list parsing with mock responses

## Phase 4: Generation — Trigger & Parse Results

- [x] 4.1 Implement `parse_generation_result()` in parser.rs — extract task_id from result[0][0] and status from result[0][4]
- [x] 4.2 Implement `client.generate_artifact(notebook_id, config) -> GenerationStatus` — dispatcher that calls CREATE_ARTIFACT (R7cb6c) via batchexecute
  - Handles USER_DISPLAYABLE_ERROR → GenerationStatus with is_rate_limited=true
  - + unit tests
- [x] 4.3 Implement `client.generate_mind_map(notebook_id, source_ids) -> MindMapResult` — two-step: GENERATE_MIND_MAP (yyryJe) + CREATE_NOTE (CYK0Xb)
  - Returns note_id + mind_map_data
  - + unit tests
- [x] 4.4 Define `GenerationStatus` struct: { task_id, status, url, error, error_code }
  - Properties: is_complete, is_failed, is_in_progress, is_rate_limited
  - Refactored generate_artifact(), check_rate_limit(), parse_generation_result() to use it
  - + 6 unit tests

## Phase 5: Polling — ArtifactPoller

- [x] 5.1 Create `src/artifact_poller.rs` with `ArtifactPoller` struct
- [x] 5.2 Implement `poll_status(notebook_id, task_id) -> GenerationStatus` — list all artifacts, scan for task_id, return status
- [x] 5.3 Implement media ready gate: `_is_media_ready(artifact_data) -> bool` — verify URL populated for media types
  - Audio: check art[6][5] non-empty
  - Video: check art[8] has URL starting with "http"
  - Infographic: forward-scan for URL
  - SlideDeck: check art[16][3] non-empty
- [x] 5.4 Implement `wait_for_completion(notebook_id, task_id, timeout) -> GenerationStatus` — exponential backoff loop (2s → 10s cap)
  - Degrade to PROCESSING if media not ready
  - Respect timeout
  - + unit tests

## Phase 6: Download — Streaming & Inline

- [x] 6.1 Implement URL extraction helpers in parser.rs:
  - `extract_audio_url(artifact_data) -> Option<String>`
  - `extract_video_url(artifact_data) -> Option<String>` (prefer quality=4)
  - `extract_infographic_url(artifact_data) -> Option<String>` (forward-scan)
  - `extract_slide_deck_url(artifact_data, format) -> Option<String>` (PDF or PPTX)
  - + unit tests for each
- [x] 6.2 Implement URL domain validation: `validate_google_domain(url) -> Result<(), NotebookLmError>`
  - Must be HTTPS + *.google.com, *.googleusercontent.com, *.googleapis.com
  - + unit tests
- [x] 6.3 Implement `streaming_download(client, url, output_path) -> Result<String, NotebookLmError>`
  - 64KB chunks via `response.bytes_stream()`
  - Write to `{path}.tmp`, rename on success, delete on failure
  - + unit tests (mock HTTP)
- [x] 6.4 Implement inline download for reports: extract markdown from art[7][0], write to file
- [x] 6.5 Implement data table CSV extraction: parse art[18] nested cells, write CSV with UTF-8 BOM
- [x] 6.6 Implement quiz/flashcard download: RPC GET_INTERACTIVE_HTML (v9rmvd) → extract data-app-data via scraper → parse JSON → write
- [x] 6.7 Implement mind map download: RPC GET_NOTES_AND_MIND_MAPS (cFji9) → extract JSON from note → write
- [x] 6.8 Implement `client.download_artifact(notebook_id, artifact_id, output_path) -> String` — dispatcher by artifact type

## Phase 7: Delete Artifact

- [x] 7.1 Implement `client.delete_artifact(notebook_id, artifact_id) -> Result<()>` — RPC DELETE_ARTIFACT (V5N4be)
  - params = [[2], artifact_id]
  - + unit test

## Phase 8: MCP Tools & CLI

- [x] 8.1 Register `artifact_list` MCP tool in main.rs
- [x] 8.2 Register `artifact_generate` MCP tool in main.rs — parses kind + optional config params → ArtifactConfig
- [x] 8.3 Register `artifact_download` MCP tool in main.rs
- [x] 8.4 Register `artifact_delete` MCP tool in main.rs
- [x] 8.5 Add CLI subcommands: `artifact list`, `artifact generate`, `artifact download`, `artifact delete`

## Phase 9: Testing & Cleanup

- [x] 9.1 Integration test: full generate → poll → download cycle (audio) — _requires real credentials_
- [x] 9.2 Integration test: quiz generation → download via HTML parse — _requires real credentials_
- [x] 9.3 Integration test: mind map generation (two-step) — _requires real credentials_
- [x] 9.4 Integration test: rate limiting returns retryable GenerationStatus — _requires real credentials_
- [x] 9.5 Verify `cargo test` passes (300 pass), `cargo clippy` has 0 warnings
- [x] 9.6 Document key findings and gotchas in code comments
