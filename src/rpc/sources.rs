//! Typed RPC payload structs for source operations.
//!
//! These structs represent the positional arrays that Google's batchexecute
//! endpoint expects. Each struct maps 1:1 to the RPC payload format documented
//! in docs/rpc-reference.md of the reference Python implementation.
//!
//! CRITICAL: The positional indices here are NOT arbitrary — they come from
//! reverse engineering the Google NotebookLM web UI traffic. A single index
//! error will cause a silent RPC failure.
//!
//! SERIALIZATION NOTE: Google expects JSON ARRAYS (positional), not objects.
//! The helper functions (build_*_params) produce correct JSON arrays.
//! The structs themselves have a to_json_array() method for wire format.

use serde::Serialize;

// =========================================================================
// Source payload inner structures (for documentation & testing)
// =========================================================================

/// Inner array for a regular web URL source.
/// URL at position [2] of an 8-element array.
///
/// Wire format (JSON array): [null, null, [url], null, null, null, null, null]
#[derive(Debug, Clone)]
pub struct UrlSourceInner {
    pub p0: Option<serde_json::Value>,
    pub p1: Option<serde_json::Value>,
    pub p2: Vec<String>,
    pub p3: Option<serde_json::Value>,
    pub p4: Option<serde_json::Value>,
    pub p5: Option<serde_json::Value>,
    pub p6: Option<serde_json::Value>,
    pub p7: Option<serde_json::Value>,
}

impl UrlSourceInner {
    pub fn new(url: &str) -> Self {
        Self {
            p0: None,
            p1: None,
            p2: vec![url.to_string()],
            p3: None,
            p4: None,
            p5: None,
            p6: None,
            p7: None,
        }
    }

    /// Serialize as JSON array (not object) for wire format.
    pub fn to_json_array(&self) -> serde_json::Value {
        serde_json::json!([self.p0, self.p1, self.p2, self.p3, self.p4, self.p5, self.p6, self.p7])
    }
}

/// Inner array for a YouTube video source.
/// URL at position [7] of an 11-element array.
///
/// Wire format: [null, null, null, null, null, null, null, [url], null, null, 1]
#[derive(Debug, Clone)]
pub struct YoutubeSourceInner {
    pub p0: Option<serde_json::Value>,
    pub p1: Option<serde_json::Value>,
    pub p2: Option<serde_json::Value>,
    pub p3: Option<serde_json::Value>,
    pub p4: Option<serde_json::Value>,
    pub p5: Option<serde_json::Value>,
    pub p6: Option<serde_json::Value>,
    pub p7: Vec<String>,
    pub p8: Option<serde_json::Value>,
    pub p9: Option<serde_json::Value>,
    pub p10: u8,
}

impl YoutubeSourceInner {
    pub fn new(url: &str) -> Self {
        Self {
            p0: None,
            p1: None,
            p2: None,
            p3: None,
            p4: None,
            p5: None,
            p6: None,
            p7: vec![url.to_string()],
            p8: None,
            p9: None,
            p10: 1,
        }
    }

    /// Serialize as JSON array (not object) for wire format.
    pub fn to_json_array(&self) -> serde_json::Value {
        serde_json::json!([
            self.p0, self.p1, self.p2, self.p3, self.p4, self.p5, self.p6, self.p7, self.p8,
            self.p9, self.p10
        ])
    }
}

/// Inner array for a Google Drive source.
/// Single-wrapped (NOT double-wrapped).
///
/// Wire format: [[file_id, mime_type, 1, title], null, null, ..., null, 1]
#[derive(Debug, Clone)]
pub struct DriveSourceInner {
    pub file_info: Vec<serde_json::Value>,
    pub p1: Option<serde_json::Value>,
    pub p2: Option<serde_json::Value>,
    pub p3: Option<serde_json::Value>,
    pub p4: Option<serde_json::Value>,
    pub p5: Option<serde_json::Value>,
    pub p6: Option<serde_json::Value>,
    pub p7: Option<serde_json::Value>,
    pub p8: Option<serde_json::Value>,
    pub p9: Option<serde_json::Value>,
    pub p10: u8,
}

impl DriveSourceInner {
    pub fn new(file_id: &str, mime_type: &str, title: &str) -> Self {
        Self {
            file_info: vec![
                serde_json::Value::String(file_id.to_string()),
                serde_json::Value::String(mime_type.to_string()),
                serde_json::json!(1),
                serde_json::Value::String(title.to_string()),
            ],
            p1: None,
            p2: None,
            p3: None,
            p4: None,
            p5: None,
            p6: None,
            p7: None,
            p8: None,
            p9: None,
            p10: 1,
        }
    }

    /// Serialize as JSON array (not object) for wire format.
    pub fn to_json_array(&self) -> serde_json::Value {
        serde_json::json!([
            self.file_info,
            self.p1,
            self.p2,
            self.p3,
            self.p4,
            self.p5,
            self.p6,
            self.p7,
            self.p8,
            self.p9,
            self.p10
        ])
    }
}

// =========================================================================
// File Upload — Step 2: Resumable Session Body (HTTP POST, not RPC)
// =========================================================================

/// Body for the resumable upload session start request.
/// Sent as JSON to POST /upload/_/?authuser=0
/// Google expects UPPERCASE field names in the JSON body.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub struct UploadSessionBody {
    pub project_id: String,
    pub source_name: String,
    pub source_id: String,
}

impl UploadSessionBody {
    pub fn new(notebook_id: &str, filename: &str, source_id: &str) -> Self {
        Self {
            project_id: notebook_id.to_string(),
            source_name: filename.to_string(),
            source_id: source_id.to_string(),
        }
    }
}

/// Google's upload URL for NotebookLM file uploads.
pub const UPLOAD_URL: &str = "https://notebooklm.google.com/upload/_/";

// =========================================================================
// Config array shared across YouTube, Drive, and File sources
// =========================================================================

/// Wire format: [1, null, null, null, null, null, null, null, null, null, [1]]
pub(crate) fn source_config() -> serde_json::Value {
    serde_json::json!([1, null, null, null, null, null, null, null, null, null, [1]])
}

// =========================================================================
// Build helpers — produce correct JSON arrays for batchexecute
// =========================================================================

/// Builds the complete params for izAoDd with a regular URL.
/// Wire: [[[null, null, [url], null, null, null, null, null]], notebook_id, [2], null, null]
pub fn build_url_source_params(notebook_id: &str, url: &str) -> serde_json::Value {
    serde_json::json!([
        [UrlSourceInner::new(url).to_json_array()],
        notebook_id,
        [2],
        null,
        null
    ])
}

/// Builds the complete params for izAoDd with a YouTube URL.
/// Wire: [[[null, ..., [url], ..., 1]], notebook_id, [2], [1, null, ..., [1]]]
pub fn build_youtube_source_params(notebook_id: &str, url: &str) -> serde_json::Value {
    serde_json::json!([
        [YoutubeSourceInner::new(url).to_json_array()],
        notebook_id,
        [2],
        source_config()
    ])
}

/// Builds the complete params for izAoDd with a Drive document.
/// NOTE: Single-wrapped source_data (not double).
/// Wire: [[drive_data], notebook_id, [2], [1, null, ..., [1]]]
pub fn build_drive_source_params(
    notebook_id: &str,
    file_id: &str,
    mime_type: &str,
    title: &str,
) -> serde_json::Value {
    serde_json::json!([
        [DriveSourceInner::new(file_id, mime_type, title).to_json_array()],
        notebook_id,
        [2],
        source_config()
    ])
}

/// Builds the complete params for o4cbdc (file registration).
/// Wire: [[[filename]], notebook_id, [2], [1, null, ..., [1]]]
pub fn build_file_register_params(notebook_id: &str, filename: &str) -> serde_json::Value {
    serde_json::json!([[[filename]], notebook_id, [2], source_config()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_inner_to_json_array() {
        let inner = UrlSourceInner::new("https://example.com");
        let arr = inner.to_json_array();
        assert!(arr.is_array());
        let items = arr.as_array().unwrap();
        assert_eq!(items.len(), 8);
        assert_eq!(
            items[2].as_array().unwrap()[0].as_str().unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn test_youtube_inner_to_json_array() {
        let inner = YoutubeSourceInner::new("https://youtu.be/abc");
        let arr = inner.to_json_array();
        assert!(arr.is_array());
        let items = arr.as_array().unwrap();
        assert_eq!(items.len(), 11);
        assert_eq!(
            items[7].as_array().unwrap()[0].as_str().unwrap(),
            "https://youtu.be/abc"
        );
        assert_eq!(items[10].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_drive_inner_to_json_array() {
        let inner = DriveSourceInner::new("fid", "application/pdf", "Doc");
        let arr = inner.to_json_array();
        assert!(arr.is_array());
        let items = arr.as_array().unwrap();
        assert_eq!(items.len(), 11);
        assert_eq!(items[0].as_array().unwrap()[0].as_str().unwrap(), "fid");
    }

    #[test]
    fn test_upload_session_body_uppercase() {
        let body = UploadSessionBody::new("nb-id", "file.pdf", "src-id");
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("PROJECT_ID"));
        assert!(json.contains("SOURCE_NAME"));
        assert!(json.contains("SOURCE_ID"));
    }

    #[test]
    fn test_build_url_source_params_is_array() {
        let params = build_url_source_params("nb-123", "https://example.com");
        let arr = params.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[1].as_str().unwrap(), "nb-123");
        assert_eq!(arr[2].as_array().unwrap()[0].as_u64().unwrap(), 2);
        assert!(arr[3].is_null());
        assert!(arr[4].is_null());
    }

    #[test]
    fn test_build_youtube_source_params_has_config_array() {
        let params = build_youtube_source_params("nb-123", "https://youtu.be/abc");
        let arr = params.as_array().unwrap();
        assert!(arr[3].is_array());
        let config = arr[3].as_array().unwrap();
        assert_eq!(config.len(), 11);
        assert_eq!(config[0].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_build_drive_source_params_single_wrapped() {
        let params = build_drive_source_params("nb-123", "fid", "application/pdf", "Doc");
        let arr = params.as_array().unwrap();
        let outer = arr[0].as_array().unwrap();
        assert_eq!(outer.len(), 1);
    }

    #[test]
    fn test_build_file_register_params() {
        let params = build_file_register_params("nb-123", "paper.pdf");
        let arr = params.as_array().unwrap();
        let p0 = arr[0].as_array().unwrap();
        assert_eq!(p0.len(), 1);
        let inner = p0[0].as_array().unwrap();
        assert_eq!(inner[0].as_str().unwrap(), "paper.pdf");
    }

    #[test]
    fn test_source_config_is_array() {
        let config = source_config();
        assert!(config.is_array());
        let items = config.as_array().unwrap();
        assert_eq!(items.len(), 11);
        assert_eq!(items[0].as_u64().unwrap(), 1);
        assert!(items[10].is_array());
    }
}
