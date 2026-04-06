# Retry-After Support Specification

## Purpose
Respect Google's Retry-After HTTP header when rate limited (429), which provides server-side guidance on how long to wait. This prevents both premature retries (wasting quota) and excessive delays (hurting user experience).

## Requirements

### Requirement: Parse Retry-After Header
The system MUST parse the Retry-After header from 429 responses.

#### Scenario: Retry-After with seconds value
- GIVEN a batchexecute response returns HTTP 429
- WHEN the Retry-After header contains "5"
- THEN the system MUST interpret this as a 5-second delay

#### Scenario: Retry-After with HTTP date value
- GIVEN a batchexecute response returns HTTP 429
- WHEN the Retry-After header contains an HTTP date
- THEN the system MUST calculate the delay as the difference between the date and current time

#### Scenario: No Retry-After header
- GIVEN a batchexecute response returns HTTP 429
- WHEN no Retry-After header is present
- THEN the system MUST fall back to the standard exponential backoff delay

### Requirement: Retry-After Overrides Backoff
When Retry-After is present, the system MUST use the server-specified delay instead of the calculated exponential backoff.

#### Scenario: Retry-After takes priority over backoff
- GIVEN a batchexecute response returns HTTP 429 with Retry-After: 10
- WHEN the retry delay is calculated
- THEN the delay MUST be 10 seconds regardless of the exponential backoff calculation

### Requirement: Retry-After Applies to Next Attempt Only
The Retry-After value MUST only affect the immediate next retry, not all future retries.

#### Scenario: After Retry-After, backoff resumes
- GIVEN a retry was delayed by Retry-After: 10
- WHEN the retried request also fails with 429 (no Retry-After)
- THEN the next retry MUST use the standard exponential backoff (not another 10s)

### Requirement: Retry-After Logged
The system MUST log when Retry-After is detected for debugging.

#### Scenario: Retry-After is logged at info level
- GIVEN a 429 response with Retry-After: 5
- WHEN the header is parsed
- THEN an info-level log message MUST include the delay value
