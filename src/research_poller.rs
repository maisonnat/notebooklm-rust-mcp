//! Research Deep Dive Poller — Non-blocking single poll + blocking loop with exponential backoff.
//!
//! Parsing rules from notebooklm-py reference:
//! - Fast research source: `[url, title, desc, type, ...]` (url at index 0)
//! - Deep research (current): `[None, [title, report_markdown], None, type, ...]`
//! - Deep research (legacy): `[None, title, None, type, ..., [chunk1, chunk2, ...]]`
//!
//! Status codes: 1 = in_progress, 2 = completed, 6 = completed (deep research)

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::errors::{NotebookLmError, NotebookResult};
use crate::rpc::notes::{ResearchSource, ResearchStatus};

/// Reference to NotebookLmClient behind Arc<RwLock<>>.
pub type ResearchClientRef = Arc<RwLock<crate::NotebookLmClient>>;

/// Configuration for research deep dive polling.
///
/// Uses exponential backoff: each iteration doubles the interval
/// up to `max_interval`. Mirrors `ArtifactPollerConfig` pattern.
#[derive(Debug, Clone)]
pub struct ResearchPollerConfig {
    /// Initial interval between polls (default: 2s).
    pub initial_interval: Duration,
    /// Maximum interval cap for exponential backoff (default: 10s).
    pub max_interval: Duration,
    /// Maximum time to wait before timing out (default: 300s / 5 min).
    pub timeout: Duration,
}

impl Default for ResearchPollerConfig {
    fn default() -> Self {
        Self {
            initial_interval: Duration::from_secs(2),
            max_interval: Duration::from_secs(10),
            timeout: Duration::from_secs(300),
        }
    }
}

impl ResearchPollerConfig {
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            timeout,
            ..Self::default()
        }
    }
}

/// Research deep dive poller.
///
/// Provides two modes:
/// - `poll_once()` — non-blocking single check (for `research_deep_dive_status`)
/// - `wait_for_completion()` — blocking loop with exponential backoff (for deprecated wrapper)
pub struct ResearchDeepDivePoller {
    client: ResearchClientRef,
    config: ResearchPollerConfig,
}

impl ResearchDeepDivePoller {
    pub fn new(client: ResearchClientRef) -> Self {
        Self::with_config(client, ResearchPollerConfig::default())
    }

    pub fn with_config(client: ResearchClientRef, config: ResearchPollerConfig) -> Self {
        Self { client, config }
    }

    /// Non-blocking single poll. Returns current research status.
    ///
    /// Calls the client's `poll_research_status` method which handles RPC.
    pub async fn poll_once(
        &self,
        notebook_id: &str,
        task_id: Option<&str>,
    ) -> NotebookResult<ResearchStatus> {
        let client = self.client.read().await;
        let tid = task_id.unwrap_or("");
        client
            .poll_research_status(notebook_id, tid)
            .await
            .map_err(NotebookLmError::from_string)
    }

    /// Blocking wait with exponential backoff.
    ///
    /// Polls until research completes or timeout is reached.
    /// Uses backoff: 2s → 4s → 8s → 10s (cap).
    pub async fn wait_for_completion(
        &self,
        notebook_id: &str,
        task_id: &str,
    ) -> NotebookResult<ResearchStatus> {
        let start = std::time::Instant::now();
        let mut interval = self.config.initial_interval;

        tracing::info!(
            "Waiting for research {} completion (timeout: {:?})",
            task_id,
            self.config.timeout
        );

        loop {
            if start.elapsed() >= self.config.timeout {
                return Err(NotebookLmError::ArtifactNotReady(format!(
                    "Timeout after {:?} waiting for research {}",
                    start.elapsed(),
                    task_id
                )));
            }

            let status = self.poll_once(notebook_id, Some(task_id)).await?;

            if status.is_complete {
                tracing::info!(
                    "Research {} completed (code: {}, elapsed: {:?})",
                    task_id,
                    status.status_code,
                    start.elapsed()
                );
                return Ok(status);
            }

            tracing::info!(
                "Research {} status={} (code: {}), polling in {:?}...",
                task_id,
                if status.status_code == 0 {
                    "not_found"
                } else {
                    "in_progress"
                },
                status.status_code,
                interval
            );

            tokio::time::sleep(interval).await;
            interval = (interval * 2).min(self.config.max_interval);
        }
    }
}

// =========================================================================
// Parsing helpers
// =========================================================================

/// Parse all research tasks from the e3bVqc response.
///
/// Returns a Vec of (task_id, ResearchStatus) pairs, ordered by recency
/// (most recent first based on array position).
pub fn parse_all_research_tasks(value: &serde_json::Value) -> Vec<(String, ResearchStatus)> {
    let arr = match value.as_array() {
        Some(a) => a,
        None => return Vec::new(),
    };

    let mut tasks = Vec::new();

    for item in arr {
        if let Some(item_arr) = item.as_array()
            && let Some(task_id) = item_arr.first().and_then(|v| v.as_str())
        {
            let task_id = task_id.to_string();

            // item[1] contains research data array
            let data = match item_arr.get(1).and_then(|v| v.as_array()) {
                Some(d) => d,
                None => continue,
            };

            // Query at data[1] — can be [query_text, ...] or just the query
            let query = data
                .get(1)
                .and_then(|v| v.as_array())
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Sources + summary at data[3]: [[sources], summary_text]
            let (sources, report) =
                if let Some(sources_summary) = data.get(3).and_then(|v| v.as_array()) {
                    let sources_arr = sources_summary
                        .first()
                        .and_then(|v| v.as_array())
                        .cloned()
                        .unwrap_or_default();

                    let summary_text = sources_summary
                        .get(1)
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let parsed_sources = parse_sources(&sources_arr);
                    let report = extract_report(&sources_arr, summary_text);

                    (parsed_sources, report)
                } else {
                    (Vec::new(), None)
                };

            // Status code at data[4]
            let code = data.get(4).and_then(|v| v.as_i64()).unwrap_or(0) as u32;

            tasks.push((
                task_id,
                ResearchStatus {
                    status_code: code,
                    sources,
                    report,
                    is_complete: code == 2 || code == 6,
                    query,
                },
            ));
        }
    }

    tasks
}

/// Parse source array entries into ResearchSource structs.
///
/// Handles three formats from NotebookLM:
/// - Fast research: `[url, title, desc, type, ...]`
/// - Deep research (current): `[None, [title, report_markdown], None, type, ...]`
/// - Deep research (legacy): `[None, title, None, type, ..., [chunk1, chunk2, ...]]`
fn parse_sources(sources: &[serde_json::Value]) -> Vec<ResearchSource> {
    sources.iter().filter_map(parse_single_source).collect()
}

fn parse_single_source(src: &serde_json::Value) -> Option<ResearchSource> {
    let arr = src.as_array()?;

    // Result type — could be at index 3 or 10
    let result_type = arr
        .get(3)
        .and_then(|v| v.as_u64())
        .or_else(|| arr.get(10).and_then(|v| v.as_u64()))
        .unwrap_or(1) as u32;

    // Fast research: url at index 0 (string starting with http)
    if let Some(url) = arr.first().and_then(|v| v.as_str())
        && url.starts_with("http")
    {
        return Some(ResearchSource {
            url: Some(url.to_string()),
            title: arr.get(1).and_then(|v| v.as_str()).map(|s| s.to_string()),
            description: arr.get(2).and_then(|v| v.as_str()).map(|s| s.to_string()),
            result_type,
        });
    }

    // Check if index 0 is null (for deep research formats)
    let index0_is_null = arr.first().map(|v| v.is_null()).unwrap_or(false);

    if index0_is_null {
        // Deep research (current format): [null, [title, report_markdown], null, type, ...]
        if let Some(title_report) = arr.get(1).and_then(|v| v.as_array())
            && title_report.len() >= 2
        {
            return Some(ResearchSource {
                url: None,
                title: title_report
                    .first()
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                description: title_report
                    .get(1)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                result_type: 5, // report type
            });
        }

        // Deep research (legacy format): [null, "title", null, type, ..., [chunk1, chunk2, ...]]
        if let Some(title) = arr.get(1).and_then(|v| v.as_str()) {
            return Some(ResearchSource {
                url: None,
                title: Some(title.to_string()),
                description: None,
                result_type: 5,
            });
        }
    }

    None
}

/// Extract report markdown from sources array.
///
/// Priority:
/// 1. Deep research current format: source[1][1] contains report_markdown
/// 2. Deep research legacy format: source[6] contains chunk array
/// 3. Summary text from poll response (fallback)
fn extract_report(sources: &[serde_json::Value], summary: Option<String>) -> Option<String> {
    // Look for deep research report in sources
    for src in sources {
        if let Some(arr) = src.as_array() {
            // Current format: [null, [title, report_markdown], null, type, ...]
            if let Some(title_report) = arr.get(1).and_then(|v| v.as_array())
                && let Some(report_md) = title_report.get(1).and_then(|v| v.as_str())
                && !report_md.is_empty()
            {
                return Some(report_md.to_string());
            }

            // Legacy format: [null, "title", null, type, ..., [chunk1, chunk2, ...]]
            if arr.first().map(|v| v.is_null()).unwrap_or(false)
                && arr.get(1).and_then(|v| v.as_str()).is_some()
                && let Some(chunks) = arr.get(6).and_then(|v| v.as_array())
            {
                let text: String = chunks
                    .iter()
                    .filter_map(|c| c.as_str())
                    .collect::<Vec<_>>()
                    .join("\n\n");
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
    }

    // Fallback to summary if available
    summary.filter(|s| !s.is_empty())
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_config_defaults() {
        let config = ResearchPollerConfig::default();
        assert_eq!(config.initial_interval, Duration::from_secs(2));
        assert_eq!(config.max_interval, Duration::from_secs(10));
        assert_eq!(config.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_config_with_timeout() {
        let config = ResearchPollerConfig::with_timeout(Duration::from_secs(600));
        assert_eq!(config.timeout, Duration::from_secs(600));
        assert_eq!(config.initial_interval, Duration::from_secs(2));
    }

    // --- Source parsing ---

    #[test]
    fn test_parse_fast_research_source() {
        let src = json!(["https://example.com", "Example Title", "A description", 1]);
        let parsed = parse_single_source(&src).unwrap();
        assert_eq!(parsed.url, Some("https://example.com".to_string()));
        assert_eq!(parsed.title, Some("Example Title".to_string()));
        assert_eq!(parsed.description, Some("A description".to_string()));
        assert_eq!(parsed.result_type, 1);
    }

    #[test]
    fn test_parse_deep_research_current_format() {
        // [None, [title, report_markdown], None, type, ...]
        let src = json!([
            null,
            ["Research Report", "# Report\n\nSome markdown content"],
            null,
            5
        ]);
        let parsed = parse_single_source(&src).unwrap();
        assert!(parsed.url.is_none());
        assert_eq!(parsed.title, Some("Research Report".to_string()));
        assert_eq!(
            parsed.description,
            Some("# Report\n\nSome markdown content".to_string())
        );
        assert_eq!(parsed.result_type, 5);
    }

    #[test]
    fn test_parse_deep_research_legacy_format() {
        // [None, "Report Title", None, type, null, null, ["chunk1", "chunk2"]]
        let src = json!([
            null,
            "Report Title",
            null,
            5,
            null,
            null,
            ["# Part 1", "# Part 2"]
        ]);
        let parsed = parse_single_source(&src).unwrap();
        assert!(parsed.url.is_none());
        assert_eq!(parsed.title, Some("Report Title".to_string()));
        assert_eq!(parsed.result_type, 5);
    }

    #[test]
    fn test_parse_sources_multiple_formats() {
        let sources = json!([
            ["https://example.com", "Fast Source", "desc", 1],
            [null, ["Deep Report", "# Full report"], null, 5],
            [
                null,
                "Legacy Report",
                null,
                5,
                null,
                null,
                ["chunk1", "chunk2"]
            ]
        ]);
        let arr = sources.as_array().unwrap();
        let parsed = parse_sources(arr);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].url, Some("https://example.com".to_string()));
        assert_eq!(parsed[1].title, Some("Deep Report".to_string()));
        assert_eq!(parsed[2].title, Some("Legacy Report".to_string()));
    }

    // --- Report extraction ---

    #[test]
    fn test_extract_report_current_format() {
        let sources = json!([[
            null,
            ["Report", "# My Research\n\nDetailed findings"],
            null,
            5
        ]]);
        let arr = sources.as_array().unwrap();
        let report = extract_report(arr, None);
        assert_eq!(
            report,
            Some("# My Research\n\nDetailed findings".to_string())
        );
    }

    #[test]
    fn test_extract_report_legacy_format() {
        let sources = json!([[
            null,
            "Report",
            null,
            5,
            null,
            null,
            ["# Part 1", "## Part 2"]
        ]]);
        let arr = sources.as_array().unwrap();
        let report = extract_report(arr, None);
        assert_eq!(report, Some("# Part 1\n\n## Part 2".to_string()));
    }

    #[test]
    fn test_extract_report_fallback_to_summary() {
        let sources = json!([]);
        let arr = sources.as_array().unwrap();
        let report = extract_report(arr, Some("Summary text".to_string()));
        assert_eq!(report, Some("Summary text".to_string()));
    }

    #[test]
    fn test_extract_report_none() {
        let sources = json!([]);
        let arr = sources.as_array().unwrap();
        let report = extract_report(arr, None);
        assert!(report.is_none());
    }

    // --- Task parsing ---

    #[test]
    fn test_parse_all_research_tasks_completed() {
        let response = json!([[
            "task-123",
            [
                null,
                ["AI research query"],
                null,
                [
                    [["https://example.com", "Source 1", "desc", 1]],
                    "Summary text"
                ],
                6
            ]
        ]]);
        let tasks = parse_all_research_tasks(&response);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].0, "task-123");
        assert!(tasks[0].1.is_complete);
        assert_eq!(tasks[0].1.status_code, 6);
        assert_eq!(tasks[0].1.query, Some("AI research query".to_string()));
        assert_eq!(tasks[0].1.sources.len(), 1);
    }

    #[test]
    fn test_parse_all_research_tasks_in_progress() {
        let response = json!([["task-456", [null, ["query"], null, [[], ""], 1]]]);
        let tasks = parse_all_research_tasks(&response);
        assert_eq!(tasks.len(), 1);
        assert!(!tasks[0].1.is_complete);
        assert_eq!(tasks[0].1.status_code, 1);
    }

    #[test]
    fn test_parse_all_research_tasks_empty() {
        let response = json!([]);
        let tasks = parse_all_research_tasks(&response);
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_parse_all_research_tasks_multiple() {
        let response = json!([
            ["task-old", [null, ["old query"], null, [[], ""], 2]],
            ["task-new", [null, ["new query"], null, [[], ""], 1]]
        ]);
        let tasks = parse_all_research_tasks(&response);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].0, "task-old");
        assert_eq!(tasks[1].0, "task-new");
    }

    #[test]
    fn test_deep_research_with_report_in_sources() {
        let response = json!([[
            "task-deep",
            [
                null,
                ["What is quantum computing?"],
                null,
                [
                    [
                        [
                            null,
                            [
                                "Quantum Computing Report",
                                "# Quantum Computing\n\n## Overview\n\nQuantum computers use qubits..."
                            ],
                            null,
                            5
                        ],
                        ["https://arxiv.org/paper", "Paper", "Academic paper", 1]
                    ],
                    ""
                ],
                6
            ]
        ]]);
        let tasks = parse_all_research_tasks(&response);
        assert_eq!(tasks.len(), 1);
        assert!(tasks[0].1.is_complete);
        assert_eq!(
            tasks[0].1.report,
            Some(
                "# Quantum Computing\n\n## Overview\n\nQuantum computers use qubits...".to_string()
            )
        );
        assert_eq!(tasks[0].1.sources.len(), 2);
    }
}
