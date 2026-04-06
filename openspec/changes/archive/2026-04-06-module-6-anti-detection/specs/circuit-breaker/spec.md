# Circuit Breaker Specification

## Purpose
Prevent the server from hammering Google's endpoints with expired or invalid credentials. After consecutive authentication failures, the circuit breaker opens and stops all outgoing requests, forcing the user to re-authenticate.

## Requirements

### Requirement: Consecutive Auth Error Tracking
The system MUST track consecutive authentication errors (401/400/CSRF responses).

#### Scenario: Auth error increments counter
- GIVEN the circuit breaker is in closed state
- WHEN a batchexecute request fails with a 401 or CSRF error
- THEN the consecutive error counter MUST increment by 1

#### Scenario: Successful request resets counter
- GIVEN the consecutive error counter is at 2
- WHEN a batchexecute request succeeds
- THEN the counter MUST reset to 0

#### Scenario: Non-auth errors do not affect counter
- GIVEN the circuit breaker is in closed state
- WHEN a batchexecute request fails with a network timeout
- THEN the consecutive error counter MUST remain unchanged

### Requirement: Circuit Opens After Threshold
The system MUST open the circuit after a configurable number of consecutive auth errors (default: 3).

#### Scenario: Circuit opens at threshold
- GIVEN 3 consecutive auth errors have occurred
- WHEN the next request is attempted
- THEN the request MUST be rejected immediately without sending to Google
- AND the error message MUST indicate "circuit breaker is open" and suggest running auth-browser

#### Scenario: Circuit remains closed below threshold
- GIVEN 2 consecutive auth errors have occurred
- WHEN the next request is attempted
- THEN the request MUST proceed normally

### Requirement: Circuit Breaker Error Message
When the circuit is open, the system MUST return a clear actionable error.

#### Scenario: Error message guides user
- GIVEN the circuit breaker is open
- WHEN the tool returns an error
- THEN the message MUST mention "auth-browser" as the solution
- AND the message MUST NOT suggest retrying

### Requirement: Half-Open State
After the circuit opens, the system MAY allow one probe request to test if auth has been restored.

#### Scenario: Probe request after circuit opens
- GIVEN the circuit has been open for at least 60 seconds
- WHEN a request is attempted
- THEN the system MAY send exactly one request as a probe
- AND if the probe succeeds, the circuit MUST close and reset the counter
- AND if the probe fails, the circuit MUST remain open
