use serde::Deserialize;
use std::cmp::Ordering;

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
}

#[derive(Debug)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: String,
}

/// Compare two semver strings (strips 'v' prefix if present).
/// Returns Ordering::Less if current < latest (update available).
pub fn compare_versions(current: &str, latest: &str) -> Ordering {
    let parse = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect()
    };
    let a = parse(current);
    let b = parse(latest);
    a.cmp(&b)
}

/// Check for updates by comparing the current version against the latest GitHub release.
/// Does NOT require authentication — uses public GitHub API.
pub async fn check_for_updates_async(
    current_version: &str,
) -> Result<UpdateCheckResult, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .user_agent(format!("notebooklm-mcp/{}", current_version))
        .timeout(std::time::Duration::from_secs(5))
        .build()?;

    let release: GitHubRelease = client
        .get("https://api.github.com/repos/maisonnat/notebooklm-rust-mcp/releases/latest")
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let latest = &release.tag_name;
    let update_available = compare_versions(current_version, latest) == Ordering::Less;

    Ok(UpdateCheckResult {
        current_version: current_version.to_string(),
        latest_version: latest.clone(),
        update_available,
        download_url: release.html_url,
    })
}

impl std::fmt::Display for UpdateCheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.update_available {
            write!(
                f,
                "New version available: {} (current: {}) — download: {}",
                self.latest_version, self.current_version, self.download_url
            )
        } else {
            write!(f, "Up to date (v{})", self.current_version)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_equal() {
        assert_eq!(compare_versions("0.1.0", "0.1.0"), Ordering::Equal);
    }

    #[test]
    fn test_compare_current_older() {
        assert_eq!(compare_versions("0.1.0", "0.2.0"), Ordering::Less);
    }

    #[test]
    fn test_compare_current_newer() {
        assert_eq!(compare_versions("0.2.0", "0.1.0"), Ordering::Greater);
    }

    #[test]
    fn test_compare_strips_v_prefix() {
        assert_eq!(compare_versions("0.1.0", "v0.1.0"), Ordering::Equal);
        assert_eq!(compare_versions("v0.1.0", "0.2.0"), Ordering::Less);
    }

    #[test]
    fn test_display_up_to_date() {
        let result = UpdateCheckResult {
            current_version: "0.1.0".to_string(),
            latest_version: "v0.1.0".to_string(),
            update_available: false,
            download_url: "https://github.com/...".to_string(),
        };
        assert_eq!(format!("{}", result), "Up to date (v0.1.0)");
    }

    #[test]
    fn test_display_update_available() {
        let result = UpdateCheckResult {
            current_version: "0.1.0".to_string(),
            latest_version: "v0.2.0".to_string(),
            update_available: true,
            download_url: "https://github.com/...".to_string(),
        };
        let display = format!("{}", result);
        assert!(display.contains("New version available: v0.2.0"));
        assert!(display.contains("current: 0.1.0"));
    }
}
