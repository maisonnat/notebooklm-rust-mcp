---
title: "Data Models — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: en
scan_type: full
---

# Data Models

## Core Entities

### Notebook

The central domain entity representing a Google NotebookLM notebook.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | UUID identifier |
| `title` | `String` | User-defined notebook title |
| `sources_count` | `u32` | Number of ingested sources |
| `is_owner` | `bool` | Whether the current user is the owner |
| `created_at` | `String` | ISO timestamp of creation |

### Source

A reference material added to a notebook for AI processing.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Source identifier |
| `title` | `String` | Display title |
| `type` | `SourceType` | One of: Text, URL, YouTube, Drive, File |

### Artifact

Generated content produced from notebook sources.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Artifact identifier |
| `title` | `String` | Display title |
| `type` | `ArtifactType` | Content type (Report, Quiz, etc.) |
| `status` | `ArtifactStatus` | Current generation status |
| `task_id` | `String` | Async generation task ID |
| `content_url` | `Option<String>` | Download URL (when completed) |
| `metadata` | `HashMap<String, String>` | Type-specific metadata |

## Enums

### ArtifactType

All supported artifact generation types:

| Variant | Output | Parameters |
|---------|--------|------------|
| `Report` | PDF | `instructions` (optional) |
| `Quiz` | PDF | `difficulty` (easy/medium/hard), `quantity` (3-20) |
| `Flashcards` | PDF | `quantity` (3-20) |
| `Audio` | Audio file | `language`, `length` (short/medium/long), `instructions` |
| `Infographic` | PNG | `detail`, `orientation`, `style` |
| `SlideDeck` | PDF/PPTX | `format`, `length` |
| `MindMap` | JSON | — |
| `Video` | Video file | `format`, `style` |
| `DataTable` | PDF | — |

### ArtifactStatus

Tracks the lifecycle of an artifact generation request:

| Variant | Description |
|---------|-------------|
| `New` | Request submitted |
| `Pending` | Queued for processing |
| `InProgress` | Currently generating |
| `Completed` | Ready for download |
| `Failed` | Generation error |
| `RateLimited` | Throttled by Google (retry later) |

### ShareAccess

| Variant | Value | Description |
|---------|-------|-------------|
| `Restricted` | 0 | Private — only invited users |
| `AnyoneWithLink` | 1 | Public — anyone with the link |

### SharePermission

| Variant | Value | Description |
|---------|-------|-------------|
| `Owner` | 1 | Full control |
| `Editor` | 2 | Can edit content |
| `Viewer` | 3 | Read-only access |

## Composite Types

### ShareStatus

```rust
ShareStatus {
    notebook_id: String,
    is_public: bool,
    access: ShareAccess,
    shared_users: Vec<SharedUser>,
    share_url: String,
}
```

### SharedUser

```rust
SharedUser {
    email: String,
    permission: SharePermission,
    display_name: String,
    avatar_url: String,
}
```

### NotebookSummary

```rust
NotebookSummary {
    summary: String,
    suggested_topics: Vec<SuggestedTopic>,
}
```

### SuggestedTopic

```rust
SuggestedTopic {
    question: String,
    prompt: String,
}
```

## Error Types

### NotebookLmError

Auto-detected from HTTP responses:

| Variant | Trigger | Recovery |
|---------|---------|----------|
| `NotFound` | 404 or empty response | Check ID validity |
| `NotReady` | Artifact/source still processing | Poll for readiness |
| `GenerationFailed` | Google returned error | Retry or adjust params |
| `DownloadFailed` | URL expired or invalid | Re-generate artifact |
| `AuthExpired` | CSRF token or cookie expired | Re-authenticate |
| `RateLimited` | 429 response | Wait and retry |
| `HttpError` | Generic HTTP failure | Retry with backoff |
| `ParseError` | Unexpected response format | Log and investigate |

## Storage

This project uses **no database**. All state lives in Google's servers — the MCP server is stateless and makes individual RPC calls for each operation.

> **[Español](../es/03-data-models.md)** · **[Português](../pt/03-data-models.md)**
