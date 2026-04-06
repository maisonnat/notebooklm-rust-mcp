# Anti-Detection Headers Specification

## Purpose
Inject browser-like HTTP headers into all requests to Google's batchexecute endpoint. This prevents Google's WAF from flagging requests as automated traffic based on missing or incorrect HTTP headers.

## Requirements

### Requirement: Chrome User-Agent Header
The system MUST include a Chrome-compatible User-Agent header in all batchexecute requests.

#### Scenario: User-Agent matches Chrome format
- GIVEN any batchexecute request is prepared
- WHEN the HTTP client sends the request
- THEN the User-Agent header MUST contain "Mozilla/5.0" and "Chrome/" and a recent Chrome version number

#### Scenario: User-Agent is static and consistent
- GIVEN multiple sequential batchexecute requests
- WHEN each request is sent
- THEN the User-Agent header MUST be identical across all requests

### Requirement: Sec-Fetch Headers
The system MUST include all Sec-Fetch-* headers that Chrome sends for XHR requests.

#### Scenario: Sec-Fetch headers present
- GIVEN any batchexecute POST request
- WHEN headers are inspected
- THEN sec-fetch-dest MUST be "empty", sec-fetch-mode MUST be "cors", sec-fetch-site MUST be "same-origin"

### Requirement: Sec-CH-UA Client Hints
The system MUST include Client Hints headers matching Chrome's reported values.

#### Scenario: Sec-CH-UA headers present
- GIVEN any batchexecute request
- WHEN headers are inspected
- THEN sec-ch-ua MUST contain at least "Google Chrome" with a version, sec-ch-ua-mobile MUST be "?0", sec-ch-ua-platform MUST be a valid OS name

### Requirement: Origin and Referer Headers
The system MUST include Origin and Referer headers matching NotebookLM's domain.

#### Scenario: Origin and Referer set correctly
- GIVEN any batchexecute request
- WHEN headers are inspected
- THEN Origin MUST be "https://notebooklm.google.com" and Referer MUST be "https://notebooklm.google.com/"

### Requirement: Accept Headers
The system MUST include Accept and Accept-Language headers consistent with a Chrome browser.

#### Scenario: Accept headers present
- GIVEN any batchexecute request
- WHEN headers are inspected
- THEN Accept MUST be "*/*" and Accept-Language MUST contain "en-US,en;q=0.9" or similar

### Requirement: Headers Centralized
All browser-like headers MUST be defined in a single module for easy maintenance.

#### Scenario: Headers module exists
- GIVEN the project structure
- THEN a module src/browser_headers.rs MUST exist containing all header constants as a function or struct
