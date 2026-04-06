---
title: "User Guide — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: en
scan_type: full
---

# User Guide

## What Problem Does It Solve?

Google NotebookLM has **no public API**. This MCP server reverse-engineers the internal RPC protocol to let AI agents interact with notebooks programmatically — create, manage, query, and generate content from any MCP-compatible client.

## Key User Flows

### 1. First-Time Setup

1. Build the binary: `cargo build --release`
2. Authenticate: `notebooklm-mcp auth-browser`
3. Verify credentials: `notebooklm-mcp verify`
4. Configure your MCP client (Claude Desktop, Cursor, Windsurf)

### 2. Create and Query a Notebook

1. `notebook_create` — create a notebook with a title
2. `source_add` — add text content as a source
3. Wait for indexing (auto-handled via source poller, 2-60s)
4. `ask_question` — query with AI-powered streaming responses

### 3. Add Multiple Source Types

- `source_add` — plain text content
- `source_add_url` — any URL (auto-detects YouTube)
- `source_add_youtube` — YouTube video URL
- `source_add_drive` — Google Drive file by ID
- `source_add_file` — upload a local file

### 4. Generate Artifacts

```bash
# Generate a study guide
artifact-generate --notebook-id <id> --kind report

# Generate a quiz (medium difficulty, 10 questions)
artifact-generate --notebook-id <id> --kind quiz --difficulty medium --quantity 10

# Generate audio overview in Spanish
artifact-generate --notebook-id <id> --kind audio --language es --length medium

# Download the generated artifact
artifact-download --notebook-id <id> --artifact-id <artifact-id>
```

### 5. Manage Notebooks

```bash
# List all notebooks
list

# Get full details (sources count, ownership, creation date)
get --notebook-id <id>

# Rename a notebook
rename --notebook-id <id> --title "New Title"

# Get AI-generated summary and suggested topics
summary --notebook-id <id>

# Share publicly
share-set --notebook-id <id> --public

# Check sharing status
share-status --notebook-id <id>

# Delete
delete --notebook-id <id>
```

### 6. AI Interaction

The `ask_question` tool supports **streaming responses** — you get answers as they're generated, similar to chatting with NotebookLM in the browser.

## Supported Artifact Types

| Type | Output | Best For |
|------|--------|----------|
| Report | PDF | Study guides, summaries |
| Quiz | PDF | Knowledge testing |
| Flashcards | PDF | Review and memorization |
| Audio | Audio file | Podcast-style overviews |
| Infographic | PNG | Visual summaries |
| Slide Deck | PDF/PPTX | Presentations |
| Mind Map | JSON | Concept mapping |
| Video | Video file | Video overviews |
| Data Table | PDF | Tabular data extraction |

## Tips

- **Start with text sources** — they index fastest (2-10s)
- **YouTube and Drive sources** take longer to process (up to 60s)
- **Rate limit**: ~30 requests/minute — the server handles this automatically
- **Artifact generation** can take 30-120s depending on type and content size
- **Sharing**: Use `share-set --public` to get a shareable link for your notebook

## Limitations

- **Reverse-engineered API** — Google may change the internal RPC format at any time
- **No official support** — this is an unofficial tool, not affiliated with Google
- **Cookie-based auth** — credentials expire and need periodic refresh
- **Stateless server** — no persistent state between restarts (state lives on Google's servers)

> **[Español](../es/05-user-guide.md)** · **[Português](../pt/05-user-guide.md)**
