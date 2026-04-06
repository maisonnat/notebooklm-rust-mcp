# Spec: Notes CRUD

## Overview

Interact with the Notes system within a notebook. Notes are user-created entries independent of sources, providing persistent memory visible in the NotebookLM web UI.

## Requirements

### REQ-N-1: Create Note

The system SHALL provide a `note_create` MCP tool and `note-create` CLI command.

**Two-step process**: Google requires creating an empty note first, then updating it.

**Step 1 — Create Empty Note (RPC: `CYK0Xb`)**
- Payload: `[notebook_id, "", [1], null, "New Note"]`
- Returns: `note_id`

**Step 2 — Update Note Content (RPC: `cYAfTb`)**
- Payload: `[notebook_id, note_id, [[[content, title, [], 0]]]]`

**Given** a valid notebook
**When** `note_create` is called with notebook_id, title, and content
**Then** an empty note MUST be created first
**And** immediately updated with the provided title and content
**And** the note_id MUST be returned

### REQ-N-2: List Notes

The system SHALL provide a `note_list` MCP tool and `note-list` CLI command.

**RPC**: `cFji9`

**Payload**: `[notebook_id]`

**Given** a notebook with existing notes
**When** `note_list` is called
**Then** all active notes MUST be returned
**And** soft-deleted notes (status == 2) MUST be filtered out

### REQ-N-3: Delete Note

The system SHALL provide a `note_delete` MCP tool and `note-delete` CLI command.

**RPC**: `AH0mwd`

**Payload**: `[notebook_id, null, [note_id]]`

**Given** a notebook with an existing note
**When** `note_delete` is called
**Then** the note MUST be soft-deleted (status set to 2)
**And** a confirmation message MUST be returned

### REQ-N-4: Soft-Delete Filtering

The list_notes parser MUST skip any note entry where the status field equals 2 (deleted).

## Scenarios

### SC-N-1: Create a note with title and content

```gherkin
Given a notebook "nb-123"
When the user calls note_create(notebook_id="nb-123", title="Key Findings", content="The main conclusion is...")
Then a new note MUST appear in the NotebookLM web UI with that title and content
And the note_id MUST be returned
```

### SC-N-2: List notes excluding deleted ones

```gherkin
Given a notebook "nb-123" with 3 notes, 1 of which is soft-deleted (status=2)
When the user calls note_list(notebook_id="nb-123")
Then exactly 2 notes MUST be returned
And the soft-deleted note MUST NOT appear
```

### SC-N-3: Delete a note

```gherkin
Given a notebook "nb-123" with note "note-456"
When the user calls note_delete(notebook_id="nb-123", note_id="note-456")
Then the note "note-456" MUST no longer appear in note_list
And a confirmation message MUST be returned
```

## Parameters

### note_create

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| notebook_id | string | Yes | UUID of the notebook |
| title | string | Yes | Note title |
| content | string | Yes | Note content |

### note_list

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| notebook_id | string | Yes | UUID of the notebook |

### note_delete

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| notebook_id | string | Yes | UUID of the notebook |
| note_id | string | Yes | ID of the note to delete |
