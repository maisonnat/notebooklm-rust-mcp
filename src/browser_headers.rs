//! Chrome-like HTTP headers for batchexecute requests.
//!
//! Google's WAF checks for browser-consistent headers. Without them,
//! requests from reqwest/rustls are flagged as automated traffic.
//! This module centralizes all spoofed headers for easy maintenance.

use reqwest::header::{HeaderMap, HeaderValue};

/// Build a HeaderMap with Chrome-like headers for batchexecute requests.
///
/// Returns headers WITHOUT Cookie and Content-Type — those are caller-specific
/// and must be added by the caller after merging these headers.
pub fn browser_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();

    // --- User-Agent (Chrome 136 on Windows) ---
    headers.insert(
        reqwest::header::USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 \
             (KHTML, like Gecko) Chrome/136.0.0.0 Safari/537.36",
        ),
    );

    // --- Sec-Fetch-* (XHR from same-origin) ---
    headers.insert("sec-fetch-dest", HeaderValue::from_static("empty"));
    headers.insert("sec-fetch-mode", HeaderValue::from_static("cors"));
    headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));

    // --- Sec-CH-UA Client Hints ---
    headers.insert(
        "sec-ch-ua",
        HeaderValue::from_static(
            "\"Chromium\";v=\"136\", \"Google Chrome\";v=\"136\", \"Not?A_Brand\";v=\"99\"",
        ),
    );
    headers.insert("sec-ch-ua-mobile", HeaderValue::from_static("?0"));
    headers.insert(
        "sec-ch-ua-platform",
        HeaderValue::from_static("\"Windows\""),
    );

    // --- Origin & Referer ---
    headers.insert(
        "origin",
        HeaderValue::from_static("https://notebooklm.google.com"),
    );
    headers.insert(
        "referer",
        HeaderValue::from_static("https://notebooklm.google.com/"),
    );

    // --- Accept ---
    headers.insert("accept", HeaderValue::from_static("*/*"));
    headers.insert(
        "accept-language",
        HeaderValue::from_static("en-US,en;q=0.9"),
    );

    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_headers_has_user_agent() {
        let headers = browser_headers();
        let ua = headers.get("user-agent").unwrap().to_str().unwrap();
        assert!(ua.contains("Chrome/"), "UA must contain Chrome/");
        assert!(ua.contains("Mozilla/5.0"), "UA must contain Mozilla/5.0");
    }

    #[test]
    fn test_browser_headers_has_sec_fetch() {
        let headers = browser_headers();
        assert_eq!(headers.get("sec-fetch-dest").unwrap(), "empty");
        assert_eq!(headers.get("sec-fetch-mode").unwrap(), "cors");
        assert_eq!(headers.get("sec-fetch-site").unwrap(), "same-origin");
    }

    #[test]
    fn test_browser_headers_has_origin_referer() {
        let headers = browser_headers();
        assert_eq!(
            headers.get("origin").unwrap(),
            "https://notebooklm.google.com"
        );
        assert_eq!(
            headers.get("referer").unwrap(),
            "https://notebooklm.google.com/"
        );
    }

    #[test]
    fn test_browser_headers_no_cookie_or_content_type() {
        let headers = browser_headers();
        assert!(
            headers.get("cookie").is_none(),
            "browser_headers() must NOT include Cookie — caller adds it"
        );
        assert!(
            headers.get("content-type").is_none(),
            "browser_headers() must NOT include Content-Type — caller adds it"
        );
    }

    #[test]
    fn test_browser_headers_has_sec_ch_ua() {
        let headers = browser_headers();
        let ch_ua = headers.get("sec-ch-ua").unwrap().to_str().unwrap();
        assert!(ch_ua.contains("Google Chrome"));
        assert_eq!(headers.get("sec-ch-ua-mobile").unwrap(), "?0");
        assert_eq!(headers.get("sec-ch-ua-platform").unwrap(), "\"Windows\"");
    }

    #[test]
    fn test_browser_headers_has_accept() {
        let headers = browser_headers();
        assert_eq!(headers.get("accept").unwrap(), "*/*");
        let lang = headers.get("accept-language").unwrap().to_str().unwrap();
        assert!(lang.contains("en-US"));
    }
}
