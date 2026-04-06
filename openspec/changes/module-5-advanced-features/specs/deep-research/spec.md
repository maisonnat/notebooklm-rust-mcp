# Spec: Deep Research

## Overview

Expose Google's autonomous web research engine to trigger deep investigations, poll for completion, and import discovered sources into a notebook.

## Requirements

### REQ-DR-1: Start Deep Research

The system SHALL implement `start_deep_research` in the client.

**RPC**: `QA9ei`

**Payload**: `[null, [1], [query, 1], 5, notebook_id]`

**Given** a valid notebook with indexed sources
**When** `start_deep_research` is called with a query
**Then** a research task MUST be initiated on Google's servers
**And** a task_id MUST be returned for polling

### REQ-DR-2: Poll Research Status

The system SHALL implement `poll_research_status` in the client.

**RPC**: `e3bVqc`

**Payload**: `[null, null, notebook_id]`

**Given** a deep research task has been started
**When** `poll_research_status` is called with the task_id
**Then** the current status of the research task MUST be returned
**And** when status_code is 2 or 6 (Completed), the discovered sources MUST be extracted

### REQ-DR-3: Import Research Sources

The system SHALL implement `import_research_sources` in the client.

**RPC**: `LBwxtb`

**Given** a completed deep research task with discovered sources
**When** `import_research_sources` is called
**Then** the discovered web sources (type 2) MUST be imported into the notebook
**And** the main research report (type 3) MUST be imported into the notebook

### REQ-DR-4: Blocking MCP Tool

The system SHALL provide a `research_deep_dive` MCP tool that:

1. Starts the research
2. Polls until completion (using the artifact_poller pattern)
3. Imports discovered sources
4. Returns a summary of discovered sources to the LLM

The tool MUST block until completion and return a human-readable summary.

### REQ-DR-5: Timeout Handling

The deep research poller MUST have a configurable timeout. If the research exceeds the timeout, the tool MUST return a partial result with discovered sources so far.

### REQ-DR-6: CLI Command

The system SHALL provide a `research` CLI command with `--notebook-id` and `--query` flags.

## Scenarios

### SC-DR-1: Complete deep research flow

```gherkin
Given a notebook "nb-123" with indexed sources about machine learning
When the user calls research_deep_dive(notebook_id="nb-123", query="Latest advances in transformer architectures")
Then a deep research task MUST be initiated
And the system MUST poll until completion
And discovered web sources MUST be imported into the notebook
And a summary of findings MUST be returned
```

### SC-DR-2: Research timeout

```gherkin
Given a deep research task that takes longer than the configured timeout
When the poller exceeds the timeout
Then a partial result MUST be returned with any sources discovered so far
And a warning about timeout MUST be included
```

## Parameters

### research_deep_dive

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| notebook_id | string | Yes | UUID of the notebook |
| query | string | Yes | Research query |
| timeout_secs | integer | No | Max wait time (default: 300) |
