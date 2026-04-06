# Auto CSRF Refresh Specification

## Purpose
Automatically refresh the CSRF token (SNlM0e) and Session ID (FdrFJe) when Google returns an authentication error, without requiring the user to manually run auth-browser. This prevents credential expiry from causing cascading failures during active sessions.

## Requirements

### Requirement: Auto-Refresh on Auth Error
The system MUST attempt to refresh the CSRF token when a batchexecute request fails with a 400/CSRF or 401 error.

#### Scenario: CSRF expired during normal operation
- GIVEN a batchexecute request fails with a CSRF-related error
- WHEN the error is detected
- THEN the system MUST call refresh_tokens() to obtain a new CSRF token
- AND the system MUST retry the original request exactly once with the new token

#### Scenario: Refresh succeeds and retry succeeds
- GIVEN a request failed with CSRF error
- WHEN refresh_tokens() returns a valid new CSRF token
- THEN the retried request MUST use the new CSRF token
- AND the consecutive error counter MUST reset

#### Scenario: Refresh fails
- GIVEN a request failed with CSRF error
- WHEN refresh_tokens() also fails
- THEN the original error MUST be propagated to the caller
- AND the error message MUST indicate that refresh was attempted

### Requirement: Single Retry Limit
The system MUST only retry once after refresh — no recursive retry loops.

#### Scenario: No infinite retry on second failure
- GIVEN a request was retried after CSRF refresh
- WHEN the retried request also fails with CSRF error
- THEN the system MUST NOT attempt another refresh
- AND the error MUST be returned immediately

### Requirement: Refresh Lock for Concurrency
The system MUST use a lock to prevent multiple concurrent CSRF refresh attempts.

#### Scenario: Concurrent requests share refresh
- GIVEN two requests fail with CSRF errors simultaneously
- WHEN both attempt to refresh
- THEN only ONE refresh call MUST be made to Google
- AND both requests MUST use the same refreshed token

### Requirement: Headers During Refresh
The refresh request MUST include the same browser-like headers as normal batchexecute requests.

#### Scenario: Refresh uses proper headers
- GIVEN a CSRF refresh is triggered
- WHEN the GET request to NotebookLM is made
- THEN the request MUST include Cookie header with the current session cookie
- AND the request MUST include User-Agent header
