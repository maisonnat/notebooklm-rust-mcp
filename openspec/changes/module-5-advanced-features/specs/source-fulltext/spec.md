# Spec: Source Fulltext Extraction

## Overview

Retrieve the complete text that Google extracted, parsed, and indexed from a source document. Especially useful for PDFs and web pages where the LLM needs raw content for local analysis.

## Requirements

### REQ-SF-1: Get Source Fulltext

The system SHALL provide a `source_get_fulltext` MCP tool and `source-get-fulltext` CLI command that retrieves the indexed text of a source.

**RPC**: `hizoJc`

**Payload**: `[[source_id], [2], [2]]`

**Given** a notebook with a source that has been fully indexed
**When** `source_get_fulltext` is called with notebook_id and source_id
**Then** the complete extracted text MUST be returned
**And** the text MUST be joined with newlines from all fragments

### REQ-SF-2: Recursive Text Extraction

The system MUST implement a recursive text extractor (`extract_all_text`) in `parser.rs` that:

- Accepts a `serde_json::Value`, current depth, and max depth
- If the value is a String, adds it to the result vector
- If the value is an Array, iterates recursively
- Returns a flattened `Vec<String>` joined with `"\n"`
- MUST enforce a max_depth limit to prevent infinite recursion on circular data

### REQ-SF-3: Depth Limit

The recursive extractor MUST default to a max_depth of 10 and MUST NOT recurse beyond it.

### REQ-SF-4: Error Handling

If the source has not been indexed yet, the system MUST return a clear error indicating the source is not ready.

## Scenarios

### SC-SF-1: Extract fulltext from a PDF source

```gherkin
Given a notebook "nb-123" with a PDF source "src-456" that is fully indexed
When the user calls source_get_fulltext(notebook_id="nb-123", source_id="src-456")
Then the complete extracted text MUST be returned as a single string
And the text MUST contain the document content (not just metadata)
```

### SC-SF-2: Source not yet indexed

```gherkin
Given a notebook "nb-123" with a source "src-789" that is still processing
When the user calls source_get_fulltext(notebook_id="nb-123", source_id="src-789")
Then an error message MUST indicate the source is not ready
```

### SC-SF-3: Recursive extraction of nested arrays

```gherkin
Given a Google RPC response with text fragments nested at various depths in result[3][0]
When the parser processes the response
Then all text fragments MUST be extracted regardless of nesting depth
And fragments MUST be joined with newlines in order
```

## Parameters

### source_get_fulltext

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| notebook_id | string | Yes | UUID of the notebook |
| source_id | string | Yes | ID of the source to extract text from |
