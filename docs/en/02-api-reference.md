---
title: "API Reference — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: en
---

# API Reference

## MCP Tools

### Notebook Management

#### `notebook_list`

List all user notebooks.

**Parameters:** None

**Returns:** Formatted list of notebooks with ID and title.

#### `notebook_create`

Create a new notebook.

**Parameters:**
- `title` (string, required): The notebook title

**Returns:** Created notebook ID.

#### `notebook_delete`

Delete a notebook by ID. Idempotent — does not error if the notebook doesn't exist.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook

**Returns:** Confirmation message.

#### `notebook_get`

Get full notebook details including sources count, ownership, and creation date.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook

**Returns:** Notebook details (title, ID, sources count, owner status, creation date).

#### `notebook_rename`

Rename a notebook. Returns the updated notebook details.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook
- `new_title` (string, required): New title for the notebook

**Returns:** Updated notebook details.

#### `notebook_summary`

Get AI-generated summary and suggested topics for a notebook.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook

**Returns:** Summary text and list of suggested topics (question + prompt pairs).

#### `notebook_share_status`

Get sharing configuration for a notebook.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook

**Returns:** Public/private status, access level, shared users list with emails and permissions, share URL.

#### `notebook_share_set`

Toggle notebook visibility to public or private. Returns updated sharing status.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook
- `public` (boolean, required): `true` for public, `false` for private

**Returns:** Updated sharing status.

### Source Management

#### `source_add`

Add a text source to a notebook.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook
- `title` (string, required): Source title
- `content` (string, required): Source text content

**Returns:** Source ID.

#### `source_add_url`

Add a URL source to a notebook. Auto-detects YouTube URLs and uses the YouTube-specific ingestion flow.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook
- `url` (string, required): URL to add
- `title` (string, optional): Custom title (auto-extracted if omitted)

**Returns:** Source ID.

#### `source_add_youtube`

Add a YouTube video as a source.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook
- `url` (string, required): YouTube video URL
- `title` (string, optional): Custom title

**Returns:** Source ID.

#### `source_add_drive`

Add a Google Drive file as a source.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook
- `file_id` (string, required): Google Drive file ID
- `title` (string, optional): Custom title

**Returns:** Source ID.

#### `source_add_file`

Upload a local file as a source.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook
- `file_path` (string, required): Local file path to upload
- `title` (string, optional): Custom title

**Returns:** Source ID.

### Artifact Management

#### `artifact_list`

List all artifacts in a notebook.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook

**Returns:** List of artifacts with ID, title, type, and status.

#### `artifact_generate`

Generate an artifact. Type-specific parameters are added based on the `kind` parameter.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook
- `kind` (string, required): Artifact type
- Additional parameters depend on `kind` (see below)

**Returns:** Artifact ID for polling completion.

**Artifact Types:**

| Kind | Additional Parameters | Output Format |
|------|---------------------|--------------|
| `report` | `instructions` (optional) | PDF |
| `quiz` | `difficulty` (easy/medium/hard), `quantity` (3-20) | PDF |
| `flashcards` | `quantity` (3-20) | PDF |
| `audio` | `language` (en/es/etc), `length` (short/medium/long), `instructions` (optional) | Audio file |
| `infographic` | `detail` (brief/standard), `orientation` (landscape/portrait), `style` (default/professional/casual) | PNG |
| `slide_deck` | `format` (pdf/pptx), `length` (short/medium/long) | PDF/PPTX |
| `mind_map` | — | JSON |
| `video` | `format` (cinematic/documentary), `style` (default/dramatic/cinematic) | Video file |
| `data_table` | — | PDF |

#### `artifact_delete`

Delete an artifact from a notebook.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook
- `artifact_id` (string, required): ID of the artifact to delete

**Returns:** Confirmation message.

#### `artifact_download`

Download an artifact in the appropriate format.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook
- `artifact_id` (string, required): ID of the artifact
- `output` (string, optional): Output file path (defaults to auto-generated name in current directory)

**Returns:** File path of downloaded artifact.

### AI Interaction

#### `ask_question`

Ask a question about a notebook with streaming response.

**Parameters:**
- `notebook_id` (string, required): UUID of the notebook
- `question` (string, required): Question to ask

**Returns:** Streaming text response (chunks).

## CLI Commands

| Command | Flags | Description |
|---------|-------|-------------|
| `auth-browser` | — | Authenticate via headless Chrome |
| `auth-status` | — | Check authentication state |
| `verify` | — | E2E validation of credentials |
| `list` | — | List all notebooks |
| `create` | `--title` | Create a notebook |
| `delete` | `--notebook-id` | Delete a notebook |
| `get` | `--notebook-id` | Get notebook details |
| `rename` | `--notebook-id` `--title` | Rename a notebook |
| `summary` | `--notebook-id` | Get AI summary |
| `share-status` | `--notebook-id` | Get sharing config |
| `share-set` | `--notebook-id` `--public` / `--private` | Toggle sharing |
| `source-add` | `--notebook-id` `--title` `--content` | Add text source |
| `source-add-url` | `--notebook-id` `--url` `--title` | Add URL source |
| `source-add-youtube` | `--notebook-id` `--url` `--title` | Add YouTube source |
| `source-add-drive` | `--notebook-id` `--file-id` `--title` | Add Drive source |
| `source-add-file` | `--notebook-id` `--file-path` `--title` | Upload file source |
| `artifact-list` | `--notebook-id` | List artifacts |
| `artifact-generate` | `--notebook-id` `--kind` + type-specific flags | Generate artifact |
| `artifact-delete` | `--notebook-id` `--artifact-id` | Delete artifact |
| `artifact-download` | `--notebook-id` `--artifact-id` `--output` | Download artifact |
| `ask` | `--notebook-id` `--question` | Ask question |

## Configuration

### Environment Variables

| Variable | Type | Description |
|----------|------|-------------|
| `NOTEBOOKLM_COOKIE` | string | Google auth cookie (from OS keyring if not set) |
| `NOTEBOOKLM_CSRF` | string | CSRF token (from OS keyring if not set) |
| `NOTEBOOKLM_SID` | string | Session ID (from OS keyring if not set) |

### MCP Client Configuration

To use this server with an MCP client (Cursor, Claude Desktop, Windsurf):

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/path/to/notebooklm-mcp",
      "args": []
    }
  }
}
```
