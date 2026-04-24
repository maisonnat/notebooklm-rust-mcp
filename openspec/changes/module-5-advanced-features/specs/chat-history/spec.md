# Spec: Chat History Sync

## Overview

Read the official chat conversation history stored on Google's servers, enabling the MCP server to resume web-initiated conversations or recover context after restarts.

## Requirements

### REQ-CH-1: Get Last Conversation ID

The system SHALL implement `get_last_conversation_id` in the client.

**RPC**: `hPTbtc`

**Payload**: `[[], null, notebook_id, 1]`

**Response format**: `[[[conv_id]]]` — triple-nested string extraction.

**Given** a notebook with at least one past conversation
**When** `get_last_conversation_id` is called
**Then** the most recent conversation_id MUST be returned
**And** if no conversation exists, an empty string or error MUST be returned

### REQ-CH-2: Get Conversation Turns

The system SHALL implement `get_conversation_turns` in the client.

**RPC**: `khqZz`

**Payload**: `[[], null, null, conversation_id, limit]`

**Given** a notebook with an existing conversation
**When** `get_conversation_turns` is called with the conversation_id and a limit
**Then** up to `limit` conversation turns MUST be returned
**And** turns MUST be in chronological order (oldest first)

### REQ-CH-3: Turn Type Detection

The parser MUST distinguish between user and AI turns:

- If `turn[2] == 1`: User message, text is in `turn[3]`
- If `turn[2] == 2`: AI response, text is in `turn[4][0][0]`

### REQ-CH-4: Chronological Ordering

Google returns turns newest-first. The parser MUST reverse the array before returning to ensure chronological order.

### REQ-CH-5: Integration with ask_question

The `ask_question` method SHOULD attempt to load the official conversation history from Google if no local cache exists. This provides seamless session continuity.

**Given** a fresh MCP server start with no local cache
**And** a notebook with an existing web conversation
**When** `ask_question` is called
**Then** the system SHOULD attempt to load the last conversation_id from Google
**And** if found, use the official conversation_id to continue the thread

### REQ-CH-6: MCP Tool

The system SHALL provide a `chat_history` MCP tool and `chat-history` CLI command that returns the conversation history for a notebook.

## Scenarios

### SC-CH-1: Retrieve conversation history

```gherkin
Given a notebook "nb-123" with a 5-turn conversation
When the user calls chat_history(notebook_id="nb-123", limit=10)
Then all 5 turns MUST be returned in chronological order
And each turn MUST be labeled as "user" or "assistant"
```

### SC-CH-2: No conversation exists

```gherkin
Given a notebook "nb-123" with no past conversations
When the user calls chat_history(notebook_id="nb-123", limit=10)
Then an empty history MUST be returned
And no error MUST be raised
```

### SC-CH-3: Resume web conversation from MCP

```gherkin
Given the user had a conversation in the NotebookLM web UI
And the MCP server is freshly started (no local cache)
When the MCP server calls ask_question on the same notebook
Then the system SHOULD detect and reuse the existing conversation_id
And the AI response MUST continue from the web conversation context
```

## Parameters

### chat_history

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| notebook_id | string | Yes | UUID of the notebook |
| limit | integer | No | Max turns to retrieve (default: 20) |
