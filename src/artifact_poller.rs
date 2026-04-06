//! Artifact Poller — Blocking polling for artifact generation status.
//!
//! Lecciones del reverse engineering (notebooklm-py):
//! - NO hay endpoint poll-by-ID — se lista TODO y se escanea por task_id
//! - task_id == artifact_id (son el mismo identificador)
//! - Media ready gate: status=COMPLETED no significa URL disponible
//! - Audio/Video/Infographic/SlideDeck necesitan verificación extra
//!
//! Design Decision (AD-2): Blocking polling, no channels.
//! MCP tools son inherentemente request-response. No hay forma de
//! enviar notificaciones async con el framework actual.
//!
//! Design Decision (AD-4): Archivo separado, NO extender SourcePoller.
//! SourcePoller es específico para fuentes con lógica distinta.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

use crate::errors::{NotebookLmError, NotebookResult};
use crate::parser::Artifact;
use crate::rpc::artifacts::ArtifactType;
#[cfg(test)]
use crate::rpc::artifacts::ArtifactStatus;

/// Reference to NotebookLmClient behind Arc<RwLock<>>.
pub type ArtifactClientRef = Arc<RwLock<crate::NotebookLmClient>>;

/// Configuration for artifact generation polling.
///
/// Uses exponential backoff: each iteration doubles the interval
/// up to `max_interval`. This prevents hammering the API while
/// still responding quickly when generation finishes.
#[derive(Debug, Clone)]
pub struct ArtifactPollerConfig {
    /// Initial interval between polls (default: 2s).
    pub initial_interval: Duration,
    /// Maximum interval cap for exponential backoff (default: 10s).
    pub max_interval: Duration,
    /// Maximum time to wait before timing out (default: 300s / 5 min).
    pub timeout: Duration,
}

impl Default for ArtifactPollerConfig {
    fn default() -> Self {
        Self {
            initial_interval: Duration::from_secs(2),
            max_interval: Duration::from_secs(10),
            timeout: Duration::from_secs(300),
        }
    }
}

impl ArtifactPollerConfig {
    /// Create config with a specific timeout (useful for cinematic videos).
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Self::default()
        }
    }
}

/// Poller for artifact generation status.
///
/// Monitors artifact generation by listing all artifacts in a notebook
/// and scanning for the target `task_id`. Includes a "media ready gate"
/// for media types (audio, video, infographic, slide_deck) where the API
/// reports COMPLETED before CDN URLs are populated.
///
/// # Usage
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use tokio::sync::RwLock;
/// use crate::artifact_poller::ArtifactPoller;
///
/// let poller = ArtifactPoller::new(client_ref);
/// let artifact = poller.wait_for_completion("nb-id", "task-id", None).await?;
/// ```
pub struct ArtifactPoller {
    client: ArtifactClientRef,
    config: ArtifactPollerConfig,
}

impl ArtifactPoller {
    /// Create a new poller with default configuration.
    pub fn new(client: ArtifactClientRef) -> Self {
        Self::with_config(client, ArtifactPollerConfig::default())
    }

    /// Create a new poller with custom configuration.
    pub fn with_config(client: ArtifactClientRef, config: ArtifactPollerConfig) -> Self {
        Self { client, config }
    }

    /// Get a reference to the current configuration.
    pub fn config(&self) -> &ArtifactPollerConfig {
        &self.config
    }

    // =====================================================================
    // Task 5.2 — poll_status
    // =====================================================================

    /// Poll the current status of an artifact by task_id.
    ///
    /// There is NO poll-by-ID endpoint in NotebookLM. This method:
    /// 1. Lists ALL artifacts in the notebook via `LIST_ARTIFACTS` (gArtLc)
    /// 2. Scans for the artifact whose `id == task_id`
    /// 3. Returns the found artifact (with status, kind, raw_data)
    ///
    /// # Errors
    ///
    /// Returns `ArtifactNotFound` if the artifact is not in the list.
    /// This can happen if the artifact was deleted or the task_id is wrong.
    pub async fn poll_status(
        &self,
        notebook_id: &str,
        task_id: &str,
    ) -> NotebookResult<Artifact> {
        let client = self.client.read().await;

        let artifacts = client
            .list_artifacts(notebook_id, None)
            .await
            .map_err(NotebookLmError::from_string)?;

        // Scan for the artifact with matching task_id
        let found = artifacts
            .into_iter()
            .find(|a| a.matches_task_id(task_id));

        match found {
            Some(artifact) => {
                info!(
                    "Poll: artifact {} status={} kind={}",
                    task_id, artifact.status, artifact.kind
                );
                Ok(artifact)
            }
            None => Err(NotebookLmError::ArtifactNotFound(format!(
                "Artifact {} not found in notebook {}",
                task_id, notebook_id
            ))),
        }
    }

    // =====================================================================
    // Task 5.4 — wait_for_completion
    // =====================================================================

    /// Wait for artifact generation to complete.
    ///
    /// Implements a blocking polling loop with exponential backoff:
    /// - Initial interval: 2s
    /// - Doubles each iteration, capped at 10s
    /// - Respects the configured timeout (default: 300s / 5 min)
    ///
    /// # Media Ready Gate
    ///
    /// For media types (Audio, Video, Infographic, SlideDeck), the API
    /// reports COMPLETED before CDN URLs are populated. This method
    /// continues polling until the media URL is actually available.
    ///
    /// # Errors
    ///
    /// - `ArtifactNotFound` — artifact deleted or wrong task_id
    /// - `GenerationFailed` — artifact status reached FAILED
    /// - `ArtifactNotReady` — timeout exceeded while still processing
    pub async fn wait_for_completion(
        &self,
        notebook_id: &str,
        task_id: &str,
    ) -> NotebookResult<Artifact> {
        use tracing::warn;

        let start = std::time::Instant::now();
        let mut interval = self.config.initial_interval;

        info!(
            "Waiting for artifact {} completion (timeout: {:?})",
            task_id, self.config.timeout
        );

        loop {
            // Check timeout
            if start.elapsed() >= self.config.timeout {
                return Err(NotebookLmError::ArtifactNotReady(format!(
                    "Timeout after {:?} waiting for artifact {}",
                    start.elapsed(),
                    task_id
                )));
            }

            match self.poll_status(notebook_id, task_id).await {
                Ok(artifact) => {
                    if artifact.is_failed() {
                        // Extract error reason from raw_data if available
                        let error_reason = artifact
                            .raw_data
                            .as_array()
                            .and_then(|arr| arr.get(3))
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown generation error");

                        return Err(NotebookLmError::GenerationFailed(format!(
                            "Artifact {} failed: {}",
                            task_id, error_reason
                        )));
                    }

                    if artifact.is_completed() {
                        // Apply media ready gate for media types
                        if !is_media_ready(&artifact) {
                            warn!(
                                "Artifact {} status=COMPLETED but media not ready yet, continuing to poll...",
                                task_id
                            );
                            // Don't return — keep polling for the URL
                        } else {
                            info!(
                                "Artifact {} completed successfully (kind: {}, elapsed: {:?})",
                                task_id, artifact.kind, start.elapsed()
                            );
                            return Ok(artifact);
                        }
                    }

                    // Still processing or media not ready — wait and retry
                    info!(
                        "Artifact {} status={} (kind: {}), polling in {:?}...",
                        task_id, artifact.status, artifact.kind, interval
                    );
                }
                Err(NotebookLmError::ArtifactNotFound(_)) => {
                    // Artifact not yet visible in listing — keep waiting
                    info!(
                        "Artifact {} not yet visible in listing, polling in {:?}...",
                        task_id, interval
                    );
                }
                Err(e) => return Err(e),
            }

            tokio::time::sleep(interval).await;

            // Exponential backoff: double interval, cap at max_interval
            interval = (interval * 2).min(self.config.max_interval);
        }
    }
}

// =========================================================================
// Media Ready Gate — Phase 5.3
// =========================================================================

/// Check if a media artifact has its download URL populated.
///
/// The NotebookLM API reports status=COMPLETED before CDN URLs are
/// ready for media types. Downloading immediately after "completion"
/// will fail. This gate verifies the URL is actually available.
///
/// Returns `true` if the artifact is ready for download, `false` if
/// we should keep polling.
///
/// # Media type checks (from exploration.md):
///
/// | Type        | Check                                      |
/// |-------------|--------------------------------------------|
/// | Audio       | `art[6][5]` non-empty list                 |
/// | Video       | `art[8]` has URL starting with "http"      |
/// | Infographic | Forward-scan for nested URL                |
/// | SlideDeck   | `art[16][3]` non-empty string (PDF URL)    |
///
/// Non-media types (Report, DataTable, Quiz, Flashcards, MindMap)
/// don't need this gate — their content is inline or fetched separately.
pub fn is_media_ready(artifact: &Artifact) -> bool {
    // Non-media types are always "ready" once status=COMPLETED
    if !matches!(
        artifact.kind,
        ArtifactType::Audio
            | ArtifactType::Video
            | ArtifactType::Infographic
            | ArtifactType::SlideDeck
    ) {
        return true;
    }

    let raw = &artifact.raw_data;
    let arr = match raw.as_array() {
        Some(a) => a,
        None => return false,
    };

    match artifact.kind {
        ArtifactType::Audio => is_audio_media_ready(arr),
        ArtifactType::Video => is_video_media_ready(arr),
        ArtifactType::Infographic => is_infographic_media_ready(arr),
        ArtifactType::SlideDeck => is_slide_deck_media_ready(arr),
        _ => true, // Should not reach here due to the filter above
    }
}

/// Audio: check `art[6][5]` is a non-empty list.
///
/// art[6] is audio metadata, art[6][5] is the media URL list.
/// Each entry in the list contains: [url, mime_type, ...]
fn is_audio_media_ready(arr: &[serde_json::Value]) -> bool {
    let media_list = arr
        .get(6)
        .and_then(|v| v.as_array())
        .and_then(|v| v.get(5))
        .and_then(|v| v.as_array());

    match media_list {
        Some(list) => !list.is_empty(),
        None => false,
    }
}

/// Video: check `art[8]` has a URL starting with "http".
///
/// art[8] is video metadata containing a list of media entries.
/// Each entry: [url, quality, mime_type, ...]. Prefer quality=4.
fn is_video_media_ready(arr: &[serde_json::Value]) -> bool {
    let video_meta = arr.get(8).and_then(|v| v.as_array());

    match video_meta {
        Some(entries) => entries.iter().any(|entry| {
            entry
                .as_array()
                .and_then(|e| e.first())
                .and_then(|v| v.as_str())
                .map(|url| url.starts_with("http"))
                .unwrap_or(false)
        }),
        None => false,
    }
}

/// Infographic: forward-scan for nested URL.
///
/// Infographic URLs don't have a fixed position. We scan the artifact
/// array looking for nested structures like `art[i][2][0][1][0]` that
/// contain an HTTP URL.
fn is_infographic_media_ready(arr: &[serde_json::Value]) -> bool {
    // Forward-scan: look for any nested URL in the artifact data
    for item in arr.iter() {
        if let Some(url) = scan_for_http_url(item)
            && url.starts_with("http") && !url.contains("notebooklm.google.com") {
                return true;
            }
    }
    false
}

/// Recursively scan a Value for HTTP URLs in nested structures.
///
/// Looks for string values starting with "http" at any nesting depth.
fn scan_for_http_url(value: &serde_json::Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        if s.starts_with("http") {
            return Some(s.to_string());
        }
        return None;
    }

    if let Some(arr) = value.as_array() {
        // Check the specific infographic pattern: [i][2][0][1][0]
        if arr.len() > 2
            && let Some(nested) = arr
                .get(2)
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
                .and_then(|v| v.as_str())
                && nested.starts_with("http") {
                    return Some(nested.to_string());
                }
        // Also scan children
        for item in arr {
            if let Some(url) = scan_for_http_url(item) {
                return Some(url);
            }
        }
    }

    None
}

/// SlideDeck: check `art[16][3]` is a non-empty string (PDF URL).
///
/// art[16] is slide deck metadata: [config, title, slides, pdf_url, pptx_url]
fn is_slide_deck_media_ready(arr: &[serde_json::Value]) -> bool {
    arr.get(16)
        .and_then(|v| v.as_array())
        .and_then(|v| v.get(3))
        .and_then(|v| v.as_str())
        .map(|url| !url.is_empty() && url.starts_with("http"))
        .unwrap_or(false)
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    // -----------------------------------------------------------------------
    // ArtifactPollerConfig tests — Task 5.1
    // -----------------------------------------------------------------------

    #[test]
    fn test_default_config() {
        let config = ArtifactPollerConfig::default();
        assert_eq!(config.initial_interval, Duration::from_secs(2));
        assert_eq!(config.max_interval, Duration::from_secs(10));
        assert_eq!(config.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_config_with_timeout() {
        let config = ArtifactPollerConfig::with_timeout(Duration::from_secs(1800));
        assert_eq!(config.timeout, Duration::from_secs(1800));
        // Other values remain default
        assert_eq!(config.initial_interval, Duration::from_secs(2));
        assert_eq!(config.max_interval, Duration::from_secs(10));
    }

    // -----------------------------------------------------------------------
    // Media ready gate tests — Task 5.3
    // -----------------------------------------------------------------------

    fn make_test_artifact(kind: ArtifactType, raw_data: Value) -> Artifact {
        Artifact {
            id: "test-artifact-id".to_string(),
            title: "Test".to_string(),
            kind,
            status: ArtifactStatus::Completed,
            raw_data,
        }
    }

    // --- Non-media types: always ready ---

    #[test]
    fn test_media_ready_report() {
        let art = make_test_artifact(ArtifactType::Report, Value::Null);
        assert!(is_media_ready(&art), "Report should always be media-ready");
    }

    #[test]
    fn test_media_ready_data_table() {
        let art = make_test_artifact(ArtifactType::DataTable, Value::Null);
        assert!(
            is_media_ready(&art),
            "DataTable should always be media-ready"
        );
    }

    #[test]
    fn test_media_ready_quiz() {
        let art = make_test_artifact(ArtifactType::Quiz, Value::Null);
        assert!(is_media_ready(&art), "Quiz should always be media-ready");
    }

    #[test]
    fn test_media_ready_flashcards() {
        let art = make_test_artifact(ArtifactType::Flashcards, Value::Null);
        assert!(
            is_media_ready(&art),
            "Flashcards should always be media-ready"
        );
    }

    #[test]
    fn test_media_ready_mind_map() {
        let art = make_test_artifact(ArtifactType::MindMap, Value::Null);
        assert!(is_media_ready(&art), "MindMap should always be media-ready");
    }

    // --- Audio media ready ---

    #[test]
    fn test_audio_media_ready_with_url() {
        // art[6][5] = non-empty list
        let raw = serde_json::json!([
            "audio-id",
            "Audio Title",
            1,
            null,
            3,
            null,
            [
                null,
                null,
                null,
                null,
                null,
                [["https://cdn.google.com/audio.mp4", "audio/mp4"]]
            ]
        ]);
        let art = make_test_artifact(ArtifactType::Audio, raw);
        assert!(is_media_ready(&art));
    }

    #[test]
    fn test_audio_media_ready_empty_list() {
        // art[6][5] = empty list
        let raw = serde_json::json!([
            "audio-id",
            "Audio Title",
            1,
            null,
            3,
            null,
            [null, null, null, null, null, []]
        ]);
        let art = make_test_artifact(ArtifactType::Audio, raw);
        assert!(
            !is_media_ready(&art),
            "Empty media list should not be ready"
        );
    }

    #[test]
    fn test_audio_media_ready_no_metadata() {
        // art[6] = null (not yet populated)
        let raw = serde_json::json!(["audio-id", "Audio Title", 1, null, 3, null, null]);
        let art = make_test_artifact(ArtifactType::Audio, raw);
        assert!(
            !is_media_ready(&art),
            "Missing audio metadata should not be ready"
        );
    }

    // --- Video media ready ---

    #[test]
    fn test_video_media_ready_with_url() {
        // art[8] = list with entries containing HTTP URLs
        let raw = serde_json::json!([
            "video-id",
            "Video Title",
            3,
            null,
            3,
            null,
            null,
            null,
            [["https://cdn.google.com/video.mp4", 4, "video/mp4"]]
        ]);
        let art = make_test_artifact(ArtifactType::Video, raw);
        assert!(is_media_ready(&art));
    }

    #[test]
    fn test_video_media_ready_empty() {
        // art[8] = empty list
        let raw = serde_json::json!(["video-id", "Video Title", 3, null, 3, null, null, null, []]);
        let art = make_test_artifact(ArtifactType::Video, raw);
        assert!(
            !is_media_ready(&art),
            "Empty video metadata should not be ready"
        );
    }

    #[test]
    fn test_video_media_ready_no_metadata() {
        // art[8] missing (array too short)
        let raw = serde_json::json!(["video-id", "Video Title", 3, null, 3]);
        let art = make_test_artifact(ArtifactType::Video, raw);
        assert!(
            !is_media_ready(&art),
            "Missing video metadata should not be ready"
        );
    }

    // --- Infographic media ready ---

    #[test]
    fn test_infographic_media_ready_with_url() {
        // Simulate the nested URL pattern: art[i][2][0][1][0]
        let raw = serde_json::json!([
            "infographic-id",
            "Infographic Title",
            7,
            null,
            3,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            [[null, [null, [["https://cdn.google.com/infographic.png"]]]]]
        ]);
        let art = make_test_artifact(ArtifactType::Infographic, raw);
        assert!(is_media_ready(&art));
    }

    #[test]
    fn test_infographic_media_ready_no_url() {
        // No HTTP URL found in any nested structure
        let raw = serde_json::json!([
            "infographic-id",
            "Infographic Title",
            7,
            null,
            3,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            [[null, [null, []]]]
        ]);
        let art = make_test_artifact(ArtifactType::Infographic, raw);
        assert!(!is_media_ready(&art), "No URL should not be ready");
    }

    #[test]
    fn test_infographic_media_ready_minimal() {
        // Array too short for any URL
        let raw = serde_json::json!(["infographic-id", "Infographic Title", 7, null, 3]);
        let art = make_test_artifact(ArtifactType::Infographic, raw);
        assert!(!is_media_ready(&art));
    }

    // --- SlideDeck media ready ---

    #[test]
    fn test_slide_deck_media_ready_with_pdf_url() {
        // art[16][3] = PDF URL
        let raw = serde_json::json!([
            "deck-id",
            "Deck Title",
            8,
            null,
            3,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            [
                "config",
                "Title",
                [],
                "https://cdn.google.com/deck.pdf",
                null
            ]
        ]);
        let art = make_test_artifact(ArtifactType::SlideDeck, raw);
        assert!(is_media_ready(&art));
    }

    #[test]
    fn test_slide_deck_media_ready_empty_url() {
        // art[16][3] = empty string
        let raw = serde_json::json!([
            "deck-id",
            "Deck Title",
            8,
            null,
            3,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            null,
            ["config", "Title", [], "", null]
        ]);
        let art = make_test_artifact(ArtifactType::SlideDeck, raw);
        assert!(!is_media_ready(&art), "Empty URL should not be ready");
    }

    #[test]
    fn test_slide_deck_media_ready_no_metadata() {
        // art[16] missing
        let raw = serde_json::json!(["deck-id", "Deck Title", 8, null, 3]);
        let art = make_test_artifact(ArtifactType::SlideDeck, raw);
        assert!(
            !is_media_ready(&art),
            "Missing deck metadata should not be ready"
        );
    }

    // --- scan_for_http_url helper ---

    #[test]
    fn test_scan_for_http_url_string() {
        let val = Value::String("https://example.com/file.mp4".to_string());
        assert_eq!(
            scan_for_http_url(&val),
            Some("https://example.com/file.mp4".to_string())
        );
    }

    #[test]
    fn test_scan_for_http_url_non_http() {
        let val = Value::String("not-a-url".to_string());
        assert!(scan_for_http_url(&val).is_none());
    }

    #[test]
    fn test_scan_for_http_url_null() {
        assert!(scan_for_http_url(&Value::Null).is_none());
    }

    #[test]
    fn test_scan_for_http_url_nested_array() {
        let val = serde_json::json!([null, [null, [["https://cdn.example.com/img.png"]]]]);
        assert_eq!(
            scan_for_http_url(&val),
            Some("https://cdn.example.com/img.png".to_string())
        );
    }

    // --- Edge cases ---

    #[test]
    fn test_media_ready_raw_not_array() {
        // Artifact with raw_data that's not an array
        let art = Artifact {
            id: "test".to_string(),
            title: "Test".to_string(),
            kind: ArtifactType::Audio,
            status: ArtifactStatus::Completed,
            raw_data: Value::String("not an array".to_string()),
        };
        assert!(!is_media_ready(&art));
    }
}
