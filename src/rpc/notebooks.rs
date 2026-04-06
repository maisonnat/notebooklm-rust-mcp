//! Typed RPC types and parsers for notebook lifecycle and sharing operations.
//!
//! These types represent the responses from Google's batchexecute RPC for
//! notebook management (delete, rename, get details, summary) and sharing
//! (get status, set public/private).
//!
//! CRITICAL: Positional indices come from reverse engineering. A single wrong
//! index causes a silent parse failure. Reference: teng-lin/notebooklm-py.

use serde::{Deserialize, Serialize};
use std::fmt;

// =========================================================================
// RPC Endpoint IDs — Notebook Lifecycle
// =========================================================================

/// Known RPC endpoint IDs for notebook lifecycle operations.
pub mod rpc_ids {
    /// Delete a notebook
    pub const DELETE_NOTEBOOK: &str = "WWINqb";
    /// Rename a notebook (dual-use: also used for set_view_level with different payload)
    pub const RENAME_NOTEBOOK: &str = "s0tc2d";
    /// Get notebook details (reused: same as GET_NOTEBOOK for sources)
    pub const GET_NOTEBOOK: &str = "rLM1Ne";
    /// Get AI-generated summary and suggested topics
    pub const SUMMARIZE: &str = "VfAZjd";

    /// Get notebook sharing configuration
    pub const GET_SHARE_STATUS: &str = "JFMDGd";
    /// Set notebook visibility (dual-use: set public, add user, remove user)
    pub const SHARE_NOTEBOOK: &str = "QDyure";
}

// =========================================================================
// ShareAccess Enum
// =========================================================================

/// Notebook access level for public sharing.
/// Maps to integer codes in the SHARE_NOTEBOOK (QDyure) RPC.
///
/// Reference: teng-lin/notebooklm-py `ShareAccess`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShareAccess {
    /// Only explicitly shared users can access
    Restricted = 0,
    /// Anyone with the link can access
    AnyoneWithLink = 1,
}

impl ShareAccess {
    /// Convert to the integer code for wire format.
    pub fn code(self) -> i32 {
        self as i32
    }

    /// Parse from API integer response.
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::Restricted),
            1 => Some(Self::AnyoneWithLink),
            _ => None,
        }
    }
}

impl fmt::Display for ShareAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Restricted => write!(f, "RESTRICTED"),
            Self::AnyoneWithLink => write!(f, "ANYONE_WITH_LINK"),
        }
    }
}

// =========================================================================
// SharePermission Enum
// =========================================================================

/// User permission level for notebook sharing.
/// Maps to integer codes in the SHARE_NOTEBOOK (QDyure) RPC user entries.
///
/// Reference: teng-lin/notebooklm-py `SharePermission`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SharePermission {
    /// Full control (read-only, cannot assign)
    Owner = 1,
    /// Can edit notebook
    Editor = 2,
    /// Read-only access
    Viewer = 3,
}

impl SharePermission {
    /// Parse from API integer response.
    pub fn from_code(code: i32) -> Option<Self> {
        match code {
            1 => Some(Self::Owner),
            2 => Some(Self::Editor),
            3 => Some(Self::Viewer),
            _ => None,
        }
    }
}

impl fmt::Display for SharePermission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Owner => write!(f, "OWNER"),
            Self::Editor => write!(f, "EDITOR"),
            Self::Viewer => write!(f, "VIEWER"),
        }
    }
}

// =========================================================================
// Data Structs — Sharing
// =========================================================================

/// A user the notebook is shared with.
///
/// Parsed from GET_SHARE_STATUS response at `data[0][i]`.
/// Entry format: `[email, permission, [], [display_name, avatar_url]]`
///
/// Reference: teng-lin/notebooklm-py `SharedUser`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedUser {
    pub email: String,
    pub permission: SharePermission,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}

/// Current sharing configuration for a notebook.
///
/// Parsed from GET_SHARE_STATUS (JFMDGd) response.
/// Response format: `[[[users]], [is_public], 1000]`
///
/// Reference: teng-lin/notebooklm-py `ShareStatus`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareStatus {
    pub notebook_id: String,
    pub is_public: bool,
    pub access: ShareAccess,
    #[serde(default)]
    pub shared_users: Vec<SharedUser>,
    #[serde(default)]
    pub share_url: Option<String>,
}

impl ShareStatus {
    /// Create a default ShareStatus with safe defaults (private, no users).
    pub fn default_private(notebook_id: &str) -> Self {
        Self {
            notebook_id: notebook_id.to_string(),
            is_public: false,
            access: ShareAccess::Restricted,
            shared_users: vec![],
            share_url: None,
        }
    }

    /// Construct the public share URL for a notebook.
    pub fn build_share_url(notebook_id: &str) -> String {
        format!("https://notebooklm.google.com/notebook/{}", notebook_id)
    }
}

// =========================================================================
// Data Structs — Summary
// =========================================================================

/// A suggested topic/question from the notebook's AI summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestedTopic {
    pub question: String,
    pub prompt: String,
}

/// AI-generated summary and suggested topics for a notebook.
///
/// Parsed from SUMMARIZE (VfAZjd) response.
/// Response format: `[[[summary_string], [[topics]], ...]]`
/// - summary at `result[0][0][0]`
/// - topics at `result[0][1][0]` — each topic is `[question, prompt]`
///
/// Reference: teng-lin/notebooklm-py `NotebookDescription`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookSummary {
    pub summary: String,
    #[serde(default)]
    pub suggested_topics: Vec<SuggestedTopic>,
}

// =========================================================================
// Parsers — Notebook Details
// =========================================================================

/// Parse notebook details from GET_NOTEBOOK (rLM1Ne) response.
///
/// The response is the same format used by `get_notebook_sources()`, but we
/// extract additional metadata fields.
///
/// Response format (after extract_by_rpc_id): `[[notebook_data]]`
/// where notebook_data = [title, sources, id, null, null, metadata, ...]
/// - `notebook_data[0]` = title (string)
/// - `notebook_data[2]` = notebook_id (UUID)
/// - `notebook_data[1]` = sources array (derive sources_count)
/// - `notebook_data[5][1]` = is_owner (False = owner, True = shared)
/// - `notebook_data[5][5][0]` = created_at timestamp (unix seconds)
///
/// Reference: teng-linlm-py `Notebook.from_api_response`
pub fn parse_notebook_details(inner: &serde_json::Value) -> Option<NotebookDetails> {
    use crate::parser::{get_string_at, get_uuid_at};

    // Navigate: inner → [0] → notebook_data array
    let nb_data = inner.as_array()?.first()?.as_array()?;

    let id = get_uuid_at(nb_data, 2)?;
    if id.is_empty() {
        return None;
    }

    let title = get_string_at(nb_data, 0).unwrap_or_default();

    // sources_count: len of sources array at data[1]
    let sources_count = nb_data
        .get(1)
        .and_then(|v| v.as_array())
        .map(|arr| arr.len())
        .unwrap_or(0);

    // is_owner: data[5][1] — False means owner, True means shared
    let is_owner = nb_data
        .get(5)
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.get(1))
        .and_then(|v| v.as_bool())
        .map(|b| !b)
        .unwrap_or(true);

    // created_at: data[5][5][0][0] — unix timestamp (double-wrapped)
    let created_at = nb_data
        .get(5)
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.get(5))
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_i64())
        .map(|ts| ts.to_string());

    Some(NotebookDetails {
        id,
        title,
        sources_count,
        is_owner,
        created_at,
    })
}

/// Intermediate struct for parsed notebook details.
/// Used to construct the enriched `Notebook` from `notebooklm_client.rs`.
pub struct NotebookDetails {
    pub id: String,
    pub title: String,
    pub sources_count: usize,
    pub is_owner: bool,
    pub created_at: Option<String>,
}

// =========================================================================
// Parsers — Share Status
// =========================================================================

/// Parse share status from GET_SHARE_STATUS (JFMDGd) response.
///
/// Response format: `[[[users]], [is_public], 1000]`
/// - `data[0]` = array of user entries, each: `[email, permission, [], [name, avatar]]`
/// - `data[1][0]` = is_public (bool)
///
/// Uses defensive parsing with safe defaults for null/empty responses.
///
/// Reference: teng-linlm-py `ShareStatus.from_api_response`
pub fn parse_share_status(inner: &serde_json::Value, notebook_id: &str) -> ShareStatus {
    let arr = match inner.as_array() {
        Some(a) => a,
        None => return ShareStatus::default_private(notebook_id),
    };

    // Parse users from data[0] = [[users]]
    // data[0] is an array containing a single array of user entries.
    // Each user entry: [email, permission, [], [name, avatar]]
    let shared_users = arr
        .first()
        .and_then(|v| v.as_array())
        .and_then(|users_wrapper| users_wrapper.first())
        .and_then(|v| v.as_array())
        .map(|users_arr| {
            users_arr
                .iter()
                .filter_map(|user_entry| {
                    let entry = user_entry.as_array()?;
                    if entry.is_empty() {
                        return None;
                    }

                    let email = entry.first()?.as_str()?.to_string();
                    if email.is_empty() {
                        return None;
                    }

                    let permission = entry
                        .get(1)
                        .and_then(|v| v.as_i64())
                        .and_then(|code| SharePermission::from_code(code as i32))
                        .unwrap_or(SharePermission::Viewer);

                    let display_name = entry
                        .get(3)
                        .and_then(|v| v.as_array())
                        .and_then(|info| info.first())
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    let avatar_url = entry
                        .get(3)
                        .and_then(|v| v.as_array())
                        .and_then(|info| info.get(1))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());

                    Some(SharedUser {
                        email,
                        permission,
                        display_name,
                        avatar_url,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    // Parse is_public from data[1][0]
    let is_public = arr
        .get(1)
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let access = if is_public {
        ShareAccess::AnyoneWithLink
    } else {
        ShareAccess::Restricted
    };

    let share_url = if is_public {
        Some(ShareStatus::build_share_url(notebook_id))
    } else {
        None
    };

    ShareStatus {
        notebook_id: notebook_id.to_string(),
        is_public,
        access,
        shared_users,
        share_url,
    }
}

// =========================================================================
// Parsers — Notebook Summary
// =========================================================================

/// Parse AI-generated summary and suggested topics from SUMMARIZE (VfAZjd) response.
///
/// Response format: `[[[summary_string], [[topics]], ...]]`
/// - `result[0][0][0]` = summary string
/// - `result[0][1][0]` = array of topics, each: `[question, prompt]`
///
/// Handles partial data gracefully: if summary exists but topics don't,
/// returns NotebookSummary with empty suggested_topics.
///
/// Reference: teng-linlm-py `NotebookLMClient.get_description()`
pub fn parse_summary(inner: &serde_json::Value) -> NotebookSummary {
    // Navigate: inner → [0] → [0] → [0] = summary string
    let summary = inner
        .as_array()
        .and_then(|arr| arr.first()) // → [[summary], [topics], ...]
        .and_then(|item| item.as_array())
        .and_then(|row| row.first()) // → [summary_string]
        .and_then(|item| item.as_array())
        .and_then(|summary_arr| summary_arr.first())
        .and_then(|item| item.as_str())
        .unwrap_or_default()
        .to_string();

    // Navigate: inner → [0] → [1] = array of topics, each: [question, prompt]
    let suggested_topics = inner
        .as_array()
        .and_then(|arr| arr.first()) // → [[summary], [topics], ...]
        .and_then(|item| item.as_array())
        .and_then(|row| row.get(1)) // → [[question, prompt], ...]
        .and_then(|v| v.as_array())
        .map(|topics| {
            topics
                .iter()
                .filter_map(|topic| {
                    let entry = topic.as_array()?;
                    let question = entry.first()?.as_str()?.to_string();
                    let prompt = entry.get(1)?.as_str()?.to_string();
                    if question.is_empty() && prompt.is_empty() {
                        return None;
                    }
                    Some(SuggestedTopic { question, prompt })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    NotebookSummary {
        summary,
        suggested_topics,
    }
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // ShareAccess
    // -----------------------------------------------------------------------

    #[test]
    fn test_share_access_codes() {
        assert_eq!(ShareAccess::Restricted.code(), 0);
        assert_eq!(ShareAccess::AnyoneWithLink.code(), 1);
    }

    #[test]
    fn test_share_access_from_code_roundtrip() {
        assert_eq!(ShareAccess::from_code(0), Some(ShareAccess::Restricted));
        assert_eq!(ShareAccess::from_code(1), Some(ShareAccess::AnyoneWithLink));
        assert_eq!(ShareAccess::from_code(99), None);
    }

    #[test]
    fn test_share_access_display() {
        assert_eq!(ShareAccess::Restricted.to_string(), "RESTRICTED");
        assert_eq!(ShareAccess::AnyoneWithLink.to_string(), "ANYONE_WITH_LINK");
    }

    // -----------------------------------------------------------------------
    // SharePermission
    // -----------------------------------------------------------------------

    #[test]
    fn test_share_permission_from_code() {
        assert_eq!(SharePermission::from_code(1), Some(SharePermission::Owner));
        assert_eq!(SharePermission::from_code(2), Some(SharePermission::Editor));
        assert_eq!(SharePermission::from_code(3), Some(SharePermission::Viewer));
        assert_eq!(SharePermission::from_code(99), None);
    }

    // -----------------------------------------------------------------------
    // parse_notebook_details
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_notebook_details_full() {
        // data[0]=title, data[2]=id, data[1]=sources, data[5][1]=is_owner, data[5][5][0]=ts
        let inner = serde_json::json!([[
            "My Notebook",
            [
                [["11111111-1111-1111-1111-111111111111"], "Source 1"],
                [["22222222-2222-2222-2222-222222222222"], "Source 2"]
            ],
            "33333333-3333-3333-3333-333333333333",
            null,
            null,
            [null, true, null, null, null, [[1714000000]]]
        ]]);

        let details = parse_notebook_details(&inner).unwrap();
        assert_eq!(details.id, "33333333-3333-3333-3333-333333333333");
        assert_eq!(details.title, "My Notebook");
        assert_eq!(details.sources_count, 2);
        assert_eq!(details.is_owner, false); // True in response = shared
        assert_eq!(details.created_at, Some("1714000000".to_string()));
    }

    #[test]
    fn test_parse_notebook_details_owner() {
        let inner = serde_json::json!([[
            "Owner Notebook",
            [],
            "44444444-4444-4444-4444-444444444444",
            null,
            null,
            [null, false, null, null, null, []]
        ]]);

        let details = parse_notebook_details(&inner).unwrap();
        assert_eq!(details.is_owner, true); // False in response = owner
    }

    #[test]
    fn test_parse_notebook_details_minimal() {
        // Only title and id, no sources, no metadata
        let inner =
            serde_json::json!([["Minimal NB", null, "55555555-5555-5555-5555-555555555555"]]);

        let details = parse_notebook_details(&inner).unwrap();
        assert_eq!(details.id, "55555555-5555-5555-5555-555555555555");
        assert_eq!(details.title, "Minimal NB");
        assert_eq!(details.sources_count, 0);
        assert!(details.is_owner); // default
        assert!(details.created_at.is_none()); // default
    }

    #[test]
    fn test_parse_notebook_details_empty_response() {
        let inner = serde_json::json!([]);
        assert!(parse_notebook_details(&inner).is_none());
    }

    #[test]
    fn test_parse_notebook_details_null_response() {
        let inner = serde_json::Value::Null;
        assert!(parse_notebook_details(&inner).is_none());
    }

    // -----------------------------------------------------------------------
    // parse_share_status
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_share_status_public() {
        // [[[users]], [true], 1000]
        let inner = serde_json::json!([
            [[["user@example.com", 3, [], ["Alice", "avatar.png"]]]],
            [true],
            1000
        ]);

        let status = parse_share_status(&inner, "nb-123");
        assert_eq!(status.notebook_id, "nb-123");
        assert_eq!(status.notebook_id, "nb-123");
        assert!(status.is_public);
        assert_eq!(status.access, ShareAccess::AnyoneWithLink);
        assert_eq!(status.shared_users.len(), 1);
        assert_eq!(status.shared_users[0].email, "user@example.com");
        assert_eq!(status.shared_users[0].permission, SharePermission::Viewer);
        assert_eq!(
            status.shared_users[0].display_name,
            Some("Alice".to_string())
        );
        assert_eq!(
            status.shared_users[0].avatar_url,
            Some("avatar.png".to_string())
        );
        assert_eq!(
            status.share_url,
            Some("https://notebooklm.google.com/notebook/nb-123".to_string())
        );
    }

    #[test]
    fn test_parse_share_status_private() {
        let inner = serde_json::json!([[[]], [false], 1000]);

        let status = parse_share_status(&inner, "nb-456");
        assert!(!status.is_public);
        assert_eq!(status.access, ShareAccess::Restricted);
        assert!(status.shared_users.is_empty());
        assert!(status.share_url.is_none());
    }

    #[test]
    fn test_parse_share_status_null_response() {
        let status = parse_share_status(&serde_json::Value::Null, "nb-789");
        assert!(!status.is_public);
        assert!(status.shared_users.is_empty());
        assert!(status.share_url.is_none());
    }

    #[test]
    fn test_parse_share_status_empty_users() {
        // Users array is present but empty
        let inner = serde_json::json!([[[]], [true], 1000]);
        let status = parse_share_status(&inner, "nb-000");
        assert!(status.is_public);
        assert!(status.shared_users.is_empty());
    }

    #[test]
    fn test_parse_share_status_malformed_user_entry() {
        // User entry missing email — should be skipped
        let inner = serde_json::json!([["not-an-array", true], [false], 1000]);
        let status = parse_share_status(&inner, "nb-111");
        assert!(!status.is_public);
        assert!(status.shared_users.is_empty());
    }

    // -----------------------------------------------------------------------
    // parse_summary
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_summary_with_topics() {
        // [[summary], [[topics]]]
        let inner = serde_json::json!([[
            ["This is the AI summary"],
            [["What is X?", "Explore X"], ["What is Y?", "Explore Y"]]
        ]]);

        let summary = parse_summary(&inner);
        assert_eq!(summary.summary, "This is the AI summary");
        assert_eq!(summary.suggested_topics.len(), 2);
        assert_eq!(summary.suggested_topics[0].question, "What is X?");
        assert_eq!(summary.suggested_topics[0].prompt, "Explore X");
        assert_eq!(summary.suggested_topics[1].question, "What is Y?");
    }

    #[test]
    fn test_parse_summary_empty_notebook() {
        // Empty response — no summary, no topics
        let inner = serde_json::json!([]);
        let summary = parse_summary(&inner);
        assert!(summary.summary.is_empty());
        assert!(summary.suggested_topics.is_empty());
    }

    #[test]
    fn test_parse_summary_only_summary_no_topics() {
        // Summary exists but topics array is missing/empty
        let inner = serde_json::json!([[["Summary text"]]]);
        let summary = parse_summary(&inner);
        assert_eq!(summary.summary, "Summary text");
        assert!(summary.suggested_topics.is_empty());
    }

    #[test]
    fn test_parse_summary_null_response() {
        let summary = parse_summary(&serde_json::json!(null));
        assert!(summary.summary.is_empty());
        assert!(summary.suggested_topics.is_empty());
    }

    #[test]
    fn test_parse_summary_skips_empty_topics() {
        // Topics array present but entries are empty
        let inner = serde_json::json!([[["Summary"], [[]]]]);
        let summary = parse_summary(&inner);
        assert_eq!(summary.summary, "Summary");
        assert!(summary.suggested_topics.is_empty());
    }
}
