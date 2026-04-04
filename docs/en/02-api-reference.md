---
title: "API Reference — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: en
scan_type: full
---

# API Reference

## MCP Tools

### `notebook_list`

List all notebooks available in the account.

**Parameters:** None

**Returns:** Formatted string with notebook list.

```
Notebooks: [Notebook { id: "uuid", title: "My Notebook" }, ...]
```

---

### `notebook_create`

Create a new notebook by title.

**Parameters:**

| Field | Type | Description |
|-------|------|-------------|
| `title` | `string` | Title for the new notebook |

**Returns:** Created notebook ID.

```
Cuaderno creado. ID: <uuid>
```

---

### `source_add`

Add a text source to a notebook.

**Parameters:**

| Field | Type | Description |
|-------|------|-------------|
| `notebook_id` | `string` | UUID of the target notebook |
| `title` | `string` | Title for the source |
| `content` | `string` | Text content of the source |

**Returns:** Source ID.

```
Fuente añadida. ID: <uuid>
```

---

### `ask_question`

Ask a question to a notebook. The question is answered using all sources in the notebook as context.

**Parameters:**

| Field | Type | Description |
|-------|------|-------------|
| `notebook_id` | `string` | UUID of the target notebook |
| `question` | `string` | Question to ask |

**Returns:** AI-generated answer text.

> **Note:** The notebook must have at least one indexed source. If no sources are available, returns an error.

---

## MCP Resources

### `notebook://<uuid>`

Read resource for each notebook.

**Response:** JSON with `id`, `title`, and `uri`.

---

## CLI Commands

### `auth`

Manual authentication with cookie and CSRF token.

```bash
notebooklm-mcp auth --cookie "YOUR_COOKIE" --csrf "YOUR_CSRF"
```

Credentials are encrypted with DPAPI and stored at `~/.notebooklm-mcp/session.bin`.

### `auth-browser` (Recommended)

Browser-based authentication via Chrome headless.

```bash
notebooklm-mcp auth-browser
```

Opens Chrome for Google login, extracts cookies via CDP, stores in OS keyring.

### `auth-status`

Check authentication state.

```bash
notebooklm-mcp auth-status
```

Shows whether Chrome is available and if credentials are stored.

### `verify`

E2E validation test against NotebookLM API.

```bash
notebooklm-mcp verify
```

Creates a test notebook to verify the connection works.

### `ask`

Ask a question to a notebook directly from CLI.

```bash
notebooklm-mcp ask --notebook-id <uuid> --question "Your question"
```

### `add-source`

Add a text source to a notebook from CLI.

```bash
notebooklm-mcp add-source --notebook-id <uuid> --title "Source Title" --content "Source content"
```

---

## Transport

The MCP server communicates over **stdio** (stdin/stdout). Configure your MCP client to launch the binary and communicate via standard I/O.
