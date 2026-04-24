# Spec: Source Management (Delete / Rename)

## Overview

Mutable operations for sources already added to a notebook. Enables cleanup and metadata correction without using the web UI.

## Requirements

### REQ-SM-1: Delete Source

The system SHALL provide a `source_delete` MCP tool and `source-delete` CLI command that removes a source from a notebook.

**RPC**: `tGMBJ`

**Payload**: `[[[source_id]]]` (triple-nested array)

**Given** a notebook with an existing source
**When** `source_delete` is called with the notebook_id and source_id
**Then** the source MUST be removed from the notebook
**And** a confirmation message MUST be returned

### REQ-SM-2: Rename Source

The system SHALL provide a `source_rename` MCP tool and `source-rename` CLI command that changes the title of an existing source.

**RPC**: `b7Wfje`

**Payload**: `[null, [source_id], [[[new_title]]]]`

**Given** a notebook with an existing source
**When** `source_rename` is called with the notebook_id, source_id, and new_title
**Then** the source title MUST be updated
**And** the updated source details MUST be returned

### REQ-SM-3: Idempotent Delete

Source delete SHOULD be idempotent — deleting a non-existent source MUST NOT error.

## Scenarios

### SC-SM-1: Delete a text source

```gherkin
Given a notebook "nb-123" with source "src-456" titled "My Document"
When the user calls source_delete(notebook_id="nb-123", source_id="src-456")
Then the source "src-456" MUST no longer appear in source_list
And a confirmation message MUST be returned
```

### SC-SM-2: Delete a non-existent source

```gherkin
Given a notebook "nb-123" with no source "src-999"
When the user calls source_delete(notebook_id="nb-123", source_id="src-999")
Then no error MUST be raised
And a graceful message MUST be returned
```

### SC-SM-3: Rename a source

```gherkin
Given a notebook "nb-123" with source "src-456" titled "Old Title"
When the user calls source_rename(notebook_id="nb-123", source_id="src-456", new_title="New Title")
Then the source "src-456" MUST have title "New Title"
```

## Parameters

### source_delete

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| notebook_id | string | Yes | UUID of the notebook |
| source_id | string | Yes | ID of the source to delete |

### source_rename

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| notebook_id | string | Yes | UUID of the notebook |
| source_id | string | Yes | ID of the source to rename |
| new_title | string | Yes | New title for the source |
