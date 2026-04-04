# Browser Auth CLI Specification

## Purpose

This specification describes the browser-based authentication feature that allows users to authenticate to NotebookLM using a headless Chrome browser instead of manually copying cookies.

## Requirements

### Requirement: Auth-Browser Command

The CLI MUST provide an `auth-browser` command that performs browser-based authentication using headless Chrome.

The command MUST launch a headless Chrome instance, navigate to Google accounts page, wait for user to complete login, extract cookies via CDP, and store them securely.

#### Scenario: Successful browser authentication

- GIVEN Chrome is installed and available
- WHEN user runs `notebooklm-mcp auth-browser`
- AND user completes login in browser window
- THEN the system extracts `__Secure-1PSID` and `__Secure-1PSIDTS` cookies
- AND stores them in OS keyring (Windows Credential Manager / Linux Secret Service)
- AND prints "Authentication successful"

#### Scenario: Chrome not available

- GIVEN Chrome is not installed or cannot be launched
- WHEN user runs `notebooklm-mcp auth-browser`
- THEN the system prints "Chrome not available. Use manual auth: notebooklm-mcp auth --cookie ... --csrf ..."
- AND exits with error code

#### Scenario: User cancels login

- GIVEN user started browser authentication
- WHEN user closes browser without completing login
- THEN the system times out after 120 seconds
- AND prints "Login timeout. Please try again"

### Requirement: Auth-Browser Status

The CLI MUST provide an `auth-status` command that shows whether browser authentication is available and if credentials are stored.

#### Scenario: Check authentication status

- GIVEN the application is installed
- WHEN user runs `notebooklm-mcp auth-status`
- THEN the system prints:
  - "Chrome available: true/false"
  - "Stored credentials: true/false"

### Requirement: Integration Tests

The test suite MUST include basic integration tests that verify module loading and function signatures.

#### Scenario: All tests pass

- GIVEN the project compiles
- WHEN `cargo test` is run
- THEN all unit tests in src/ modules MUST pass
- AND all test fixtures MUST be readable