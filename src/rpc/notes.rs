//! Notes and Chat History types for Google NotebookLM RPC.
//!
//! Module 5 — Notes CRUD, Chat History sync, and related types.

use serde::{Deserialize, Serialize};

/// A note within a notebook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
}

/// A single turn in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTurn {
    pub role: String, // "user" or "assistant"
    pub text: String,
}

/// Status of a deep research task.
#[derive(Debug, Clone)]
pub struct ResearchStatus {
    pub status_code: u32,
    pub sources: Vec<serde_json::Value>,
    pub is_complete: bool,
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_note_struct_fields() {
        let note = super::Note {
            id: "note-123".to_string(),
            title: "My Note".to_string(),
            content: "Content here".to_string(),
        };
        assert_eq!(note.id, "note-123");
        assert_eq!(note.title, "My Note");
        assert_eq!(note.content, "Content here");
    }

    #[test]
    fn test_chat_turn_struct_fields() {
        let turn = super::ChatTurn {
            role: "user".to_string(),
            text: "Hello".to_string(),
        };
        assert_eq!(turn.role, "user");
        assert_eq!(turn.text, "Hello");
    }
}
