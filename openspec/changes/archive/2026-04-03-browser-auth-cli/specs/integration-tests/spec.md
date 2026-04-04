# Delta for Integration Tests

## ADDED Requirements

### Requirement: Basic Integration Tests

The project SHOULD include integration tests that verify module functionality without requiring external services.

#### Scenario: All unit tests pass

- GIVEN the project compiles without errors
- WHEN `cargo test` is run
- THEN all unit tests in src/ modules MUST pass
- AND all test fixtures MUST be readable

#### Scenario: Parser integration

- GIVEN a sample RPC response from tests/fixtures/
- WHEN the parser processes it
- THEN it MUST correctly extract notebook list and source IDs

#### Scenario: Error handling integration

- GIVEN various error strings (401, 400, 429)
- WHEN NotebookLmError::from_string is called
- THEN it MUST correctly classify the error type

### Requirement: Module Compilation

The project MUST compile all modules including newly added ones.

#### Scenario: Full project compiles

- GIVEN all source files in src/
- WHEN `cargo build` is run
- THEN it MUST complete without errors
- AND all dependencies MUST resolve