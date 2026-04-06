# Delta for Source Polling

## MODIFIED Requirements

### Requirement: Pre-Request Jitter
(Previously: Jitter range was 150-600ms)

The system MUST add a random delay before each batchexecute request to simulate human timing patterns.

#### Scenario: Jitter applied before every request
- GIVEN any batchexecute request is prepared
- WHEN the request is about to be sent
- THEN the system MUST add a random delay between 800ms and 2000ms

#### Scenario: Jitter varies between requests
- GIVEN two sequential batchexecute requests
- WHEN both requests are sent
- THEN the delays MUST be different (non-deterministic)
