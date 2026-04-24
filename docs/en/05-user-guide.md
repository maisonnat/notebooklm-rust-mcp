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

### 6. Source Management

Beyond adding sources, you can rename, delete, and read their full extracted text:

```bash
# Rename a source
source-rename --notebook-id <id> --source-id <source-id> --new-title "Better Title"

# Delete a source (idempotent — safe to call even if already deleted)
source-delete --notebook-id <id> --source-id <source-id>

# Get the full extracted text of a source (useful for PDFs, web pages, etc.)
source-get-fulltext --notebook-id <id> --source-id <source-id>
```

The `source_get_fulltext` tool is particularly powerful — it returns the **complete text that Google extracted and indexed** from the source, including OCR text from PDFs and parsed content from web pages. This lets you read document content directly without asking questions.

### 7. Notes CRUD

Create, list, and manage notes inside a notebook. Notes appear in the NotebookLM web UI alongside your sources:

```bash
# Create a note
note-create --notebook-id <id> --title "Key Findings" --content "Important insights from the research..."

# List all active notes (excludes soft-deleted notes)
note-list --notebook-id <id>

# Delete a note (soft-delete)
note-delete --notebook-id <id> --note-id <note-id>
```

> **Note**: Note creation is a two-step process internally (create empty → update with content). The MCP tool handles this automatically.

### 8. Chat History

Retrieve the full conversation history from Google's servers for any notebook:

```bash
# Get last 20 turns (default)
chat-history --notebook-id <id>

# Get last 50 turns
chat-history --notebook-id <id> --limit 50
```

Returns turns in **chronological order** (oldest first), with each turn showing the role (`user` or `assistant`) and text. This is useful for:

- Reviewing what questions were asked previously
- Building context for follow-up questions
- Exporting conversation logs

### 9. Deep Research

Launch Google's autonomous research engine from any MCP client. The tool **blocks until research completes** (up to 300s), then automatically imports discovered sources into the notebook:

```bash
# Start a deep research investigation
research --notebook-id <id> --query "Compare transformer architectures for NLP tasks"

# With custom timeout
research --notebook-id <id> --query "Quantum computing applications" --timeout-secs 600
```

The research flow:
1. Starts a research task on Google's servers
2. Polls for completion every 5 seconds
3. When complete, imports all discovered web sources into the notebook
4. Returns a summary of discovered sources

> **Tip**: Deep research can take 2-5 minutes. If the timeout is reached, you get a partial result with whatever sources were discovered so far.

### 10. AI Interaction

The `ask_question` tool supports **streaming responses** — you get answers as they're generated, similar to chatting with NotebookLM in the browser. The tool automatically retrieves the active conversation ID from Google's servers to maintain conversation continuity.

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
