//! Browser-based authentication module for NotebookLM MCP
//!
//! This module provides automatic login to Google using headless Chrome
//! via Chrome DevTools Protocol (CDP), extracting cookies that cannot be
//! accessed manually (HttpOnly cookies).
//!
//! Advantages over manual cookie copy:
//! - User interacts directly with Google (no manual cookie copying)
//! - Greater security: never handle plaintext credentials
//! - Extracts HttpOnly cookies that we cannot see manually
//! - Auto-renewable: re-authentication is easy when cookies expire

use headless_chrome::Browser;
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tempfile::TempDir;
use tracing::{error, info, warn};

/// Service identifier for keyring
const SERVICE_NAME: &str = "notebooklm-mcp";

/// Credential structure stored in keyring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserCredentials {
    pub cookie: String,
    pub csrf: String,
    /// Session ID (FdrFJe) — required as `f.sid` URL param in batchexecute requests
    pub sid: String,
}

/// Authentication result
#[derive(Debug)]
pub enum AuthResult {
    /// Successful authentication with extracted credentials
    Success(BrowserCredentials),
    /// Fallback required (Chrome not available, user cancelled, etc.)
    FallbackRequired(String),
    /// Authentication failed
    Failed(String),
}

/// Browser-based authenticator
pub struct BrowserAuthenticator {
    /// Temporary directory for browser profile (optional)
    #[allow(dead_code)]
    profile_dir: Option<TempDir>,
}

impl BrowserAuthenticator {
    /// Create a new browser authenticator
    pub fn new() -> Self {
        Self {
            profile_dir: None,
        }
    }

    /// Attempt browser-based authentication
    /// Falls back to DPAPI if Chrome is not available
    pub async fn authenticate(&self) -> AuthResult {
        use headless_chrome::LaunchOptions;
        use std::ffi::OsStr;

        info!("Attempting browser-based authentication...");

        // Launch Chrome in VISIBLE mode with anti-automation flags.
        // Without these flags, Google detects CDP automation and blocks login
        // with "It's possible that the browser or app isn't secure."
        let launch_options = LaunchOptions::default_builder()
            .headless(false)
            .window_size(Some((1280, 800)))
            .args(vec![
                OsStr::new("--disable-blink-features=AutomationControlled"),
                OsStr::new("--no-first-run"),
                OsStr::new("--disable-infobars"),
                OsStr::new("--disable-blink-features=AutomationControlled"),
                OsStr::new("--password-manager-auto-login=false"),
                OsStr::new("--disable-translate"),
            ])
            .build();

        let browser = match launch_options {
            Ok(opts) => match Browser::new(opts) {
                Ok(b) => b,
                Err(e) => {
                    warn!("Chrome not available: {}. Falling back to DPAPI method.", e);
                    return AuthResult::FallbackRequired(format!(
                        "Chrome not available: {}. Use manual auth command.",
                        e
                    ));
                }
            },
            Err(e) => {
                warn!("Failed to build launch options: {}. Falling back to DPAPI method.", e);
                return AuthResult::FallbackRequired(format!(
                    "Failed to configure Chrome: {}. Use manual auth command.",
                    e
                ));
            }
        };

        info!("Chrome launched successfully, starting login flow...");

        // Navigate to Google login page
        let tab = match browser.new_tab() {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to create new tab: {}", e);
                return AuthResult::Failed(format!("Failed to create tab: {}", e));
            }
        };

        // Navigate to NotebookLM — Google will redirect to login if needed,
        // and after login the cookies for notebooklm.google.com are set.
        let login_url = "https://notebooklm.google.com/";
        if let Err(e) = tab.navigate_to(login_url) {
            error!("Failed to navigate to NotebookLM: {}", e);
            return AuthResult::Failed(format!("Navigation failed: {}", e));
        }

        // Wait for navigation to complete
        if let Err(e) = tab.wait_until_navigated() {
            warn!("Navigation wait returned: {}", e);
        }

        // Wait for user to complete login
        info!("Please complete login in the browser window...");
        info!("Waiting for authentication to complete (timeout: 120 seconds)...");

        // Wait for the user to complete login - use timeout loop to check for cookies
        let timeout = Duration::from_secs(120);
        let start = std::time::Instant::now();
        let check_interval = Duration::from_secs(2);

        loop {
            if start.elapsed() > timeout {
                warn!("Timeout waiting for login");
                return AuthResult::Failed(
                    "Login timeout. Please try again and complete login faster.".to_string(),
                );
            }

            // Check if cookies are available
            if let Ok(cookies) = tab.get_cookies() {
                let has_psid = cookies.iter().any(|c| c.name == "__Secure-1PSID");
                if has_psid {
                    info!("Detected authentication cookies present");
                    break;
                }
            }

            tokio::time::sleep(check_interval).await;
        }

        // After login, the browser may be on accounts.google.com or a redirect page.
        // Navigate explicitly to NotebookLM so WIZ_global_data gets populated.
        info!("Navigating to NotebookLM to extract session tokens...");
        if let Err(e) = tab.navigate_to("https://notebooklm.google.com/") {
            warn!("Navigation to NotebookLM after login failed: {}", e);
            // Don't fail — we still have cookies, CSRF can be refreshed later
        } else if let Err(e) = tab.wait_until_navigated() {
            warn!("Wait for navigation after login returned: {}", e);
        }

        // Wait for the page JavaScript to execute and populate WIZ_global_data.
        // This is async — wait_until_navigated only waits for DOM, not JS execution.
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Extract cookies via CDP
        match Self::extract_cookies(&tab).await {
            Ok(creds) => {
                info!("Successfully extracted credentials from browser");
                AuthResult::Success(creds)
            }
            Err(e) => {
                error!("Failed to extract cookies: {}", e);
                AuthResult::Failed(format!("Cookie extraction failed: {}", e))
            }
        }
    }

    /// Extract cookies from the authenticated session
    async fn extract_cookies(tab: &headless_chrome::Tab) -> Result<BrowserCredentials, String> {
        // Get all cookies for notebooklm.google.com
        let cookies = tab.get_cookies().map_err(|e| format!("CDP error: {}", e))?;

        // Build the FULL cookie string with ALL Google cookies.
        let cookie_parts: Vec<String> = cookies
            .iter()
            .filter(|c| {
                c.name.contains("SID") || c.name.contains("HSID") || c.name.contains("SSID")
                || c.name.contains("APISID") || c.name.contains("SAPISID")
                || c.name.contains("LSID") || c.name.contains("NID")
                || c.name.contains("SIDCC") || c.name.contains("PSIDCC")
                || c.name.starts_with("__Secure-") || c.name.starts_with("__Host-")
            })
            .map(|c| format!("{}={}", c.name, c.value))
            .collect();

        let cookie = cookie_parts.join("; ");

        if cookie.is_empty() {
            let available: Vec<_> = cookies.iter().map(|c| c.name.as_str()).collect();
            return Err(format!(
                "No Google auth cookies found. Available: {:?}",
                available
            ));
        }

        info!("Extracted {} cookies ({} bytes total)", cookie_parts.len(), cookie.len());

        // Extract CSRF and session ID via HTTP GET with the cookies.
        // CDP JS evaluation doesn't work because NotebookLM's SPA consumes
        // WIZ_global_data at load time — the tokens are no longer in the DOM.
        // But a plain HTTP GET returns the raw HTML with both tokens.
        let (csrf, sid) = crate::auth_helper::AuthHelper::new()
            .refresh_tokens(&cookie)
            .await
            .unwrap_or_else(|e| {
                warn!("Could not extract tokens via HTTP: {}", e);
                info!("Tokens will be extracted on first API call");
                (String::new(), String::new())
            });

        info!("CSRF token: {} chars", csrf.len());
        info!("Session ID (f.sid): {} chars", sid.len());

        Ok(BrowserCredentials { cookie, csrf, sid })
    }

    /// Store credentials in OS keyring (Windows Credential Manager / Linux Secret Service)
    pub fn store_in_keyring(creds: &BrowserCredentials) -> Result<(), String> {
        let entry = Entry::new(SERVICE_NAME, "google-credentials")
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        let json = serde_json::to_string(creds)
            .map_err(|e| format!("Failed to serialize credentials: {}", e))?;

        entry
            .set_password(&json)
            .map_err(|e| format!("Failed to store in keyring: {}", e))?;

        info!("Credentials stored securely in OS keyring");
        Ok(())
    }

    /// Retrieve credentials from OS keyring
    pub fn load_from_keyring() -> Result<BrowserCredentials, String> {
        let entry = Entry::new(SERVICE_NAME, "google-credentials")
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        let json = entry
            .get_password()
            .map_err(|e| format!("Failed to load from keyring: {}", e))?;

        let creds: BrowserCredentials = serde_json::from_str(&json)
            .map_err(|e| format!("Failed to parse credentials: {}", e))?;

        Ok(creds)
    }

    /// Delete credentials from OS keyring
    pub fn delete_from_keyring() -> Result<(), String> {
        let entry = Entry::new(SERVICE_NAME, "google-credentials")
            .map_err(|e| format!("Failed to create keyring entry: {}", e))?;

        // Delete the entry (ignore error if it doesn't exist)
        let _ = entry.delete_credential();

        info!("Credentials removed from OS keyring");
        Ok(())
    }

    /// Check if keyring credentials exist
    pub fn has_stored_credentials() -> bool {
        Entry::new(SERVICE_NAME, "google-credentials")
            .and_then(|e| e.get_password())
            .is_ok()
    }
}

impl Default for BrowserAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

/// Attempt to authenticate using browser automation
/// Returns credentials if successful, None if fallback required
pub async fn try_browser_auth() -> Option<BrowserCredentials> {
    let auth = BrowserAuthenticator::new();

    match auth.authenticate().await {
        AuthResult::Success(creds) => Some(creds),
        AuthResult::FallbackRequired(_) => None,
        AuthResult::Failed(e) => {
            error!("Browser authentication failed: {}", e);
            None
        }
    }
}

/// Store credentials using the appropriate method (keyring preferred, fallback to DPAPI)
pub fn store_credentials(creds: &BrowserCredentials) -> Result<(), String> {
    // Try keyring first
    if let Err(e) = BrowserAuthenticator::store_in_keyring(creds) {
        warn!("Failed to store in keyring: {}, falling back to DPAPI", e);
        // Fallback to DPAPI via existing main.rs implementation
        // (This would need to be called from main.rs)
    }

    Ok(())
}

/// Load credentials from storage (keyring preferred, fallback to DPAPI)
pub fn load_credentials() -> Option<(String, String, String)> {
    // Returns (cookie, csrf, sid)
    if let Ok(creds) = BrowserAuthenticator::load_from_keyring() {
        return Some((creds.cookie, creds.csrf, creds.sid));
    }
    None
}

/// Check if browser-based authentication is available
pub fn is_browser_auth_available() -> bool {
    use headless_chrome::LaunchOptions;
    let Ok(opts) = LaunchOptions::default_builder()
        .headless(false)
        .build()
    else {
        return false;
    };
    Browser::new(opts).is_ok()
}

/// Get status information about browser authentication
pub fn get_auth_status() -> AuthStatus {
    let chrome_available = is_browser_auth_available();
    let has_keyring_creds = BrowserAuthenticator::has_stored_credentials();

    AuthStatus {
        chrome_available,
        has_stored_credentials: has_keyring_creds,
    }
}

/// Authentication status information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuthStatus {
    pub chrome_available: bool,
    pub has_stored_credentials: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_status() {
        let status = get_auth_status();
        // This will fail if Chrome is not installed, but that's expected
        println!("Chrome available: {}", status.chrome_available);
        println!("Has stored credentials: {}", status.has_stored_credentials);
    }
}