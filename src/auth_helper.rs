//! Auth Helper - Extrae y refresca tokens CSRF desde la página de NotebookLM
//!
//! Lecciones del reverse engineering (notebooklm-py):
//! - El token CSRF (SNlM0e) no es estático - debe extraerse del HTML
//! - Después de un error 400, intentar refresh silencioso y reintentar
//! - El CSRF se encuentra en el HTML como: "SNlM0e":"valor"
//!
//! Aclaración: Las cookies se manejan por fuera (DPAPI/Keyring)
//! Este módulo solo se encarga del token CSRF

use regex::Regex;
use tracing::info;

/// Extrae el token CSRF (SNlM0e) del HTML de NotebookLM
/// Retorna el valor del token o error si no se encuentra
pub fn extract_csrf_from_html(html: &str) -> Result<String, String> {
    // El token está en el HTML como: "SNlM0e":"valor"
    // También puede aparecer como: "SNlM0e":"valor","other_field":...
    let re = Regex::new(r#""SNlM0e"\s*:\s*"([^"]+)""#)
        .map_err(|e| format!("Regex inválido: {}", e))?;
    
    if let Some(caps) = re.captures(html) {
        if let Some(token) = caps.get(1) {
            let token_str = token.as_str();
            if token_str.is_empty() {
                return Err("Token CSRF vacío".to_string());
            }
            info!("Token CSRF extraído: {} chars", token_str.len());
            return Ok(token_str.to_string());
        }
    }
    
    // Backup: buscar en otros formatos posibles
    let re2 = Regex::new(r#"SNlM0e["']?\s*[:=]\s*["']([^"']+)["']"#)
        .map_err(|e| format!("Regex backup inválido: {}", e))?;
    
    if let Some(caps) = re2.captures(html) {
        if let Some(token) = caps.get(1) {
            return Ok(token.as_str().to_string());
        }
    }
    
    Err("No se encontró token CSRF (SNlM0e) en el HTML".to_string())
}

/// Auth helper que puede extraer CSRF y validar cookies
pub struct AuthHelper {
    http_client: reqwest::Client,
}

impl AuthHelper {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");
        
        Self { http_client: client }
    }

    /// Hace un GET a la página principal y extrae el CSRF token
    /// Requiere cookies válidas en el header
    pub async fn refresh_csrf(&self, cookie: &str) -> Result<String, String> {
        info!("Refrescando token CSRF...");
        
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::COOKIE,
            reqwest::header::HeaderValue::from_str(cookie)
                .map_err(|e| format!("Cookie inválida: {}", e))?
        );
        
        let response = self.http_client
            .get("https://notebooklm.google.com/")
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("GET falló: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("GET a página principal falló: {}", response.status()));
        }
        
        let html = response.text().await
            .map_err(|e| format!("No se pudo leer HTML: {}", e))?;
        
        extract_csrf_from_html(&html)
    }

    /// Valida que las cookies aún sean válidas haciendo un GET simple
    /// Retorna Ok si la sesión sigue activa, error si expiró
    pub async fn validate_session(&self, cookie: &str) -> Result<bool, String> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::COOKIE,
            reqwest::header::HeaderValue::from_str(cookie)
                .map_err(|e| format!("Cookie inválida: {}", e))?
        );

        let response = self.http_client
            .get("https://notebooklm.google.com/")
            .headers(headers)
            .send()
            .await
            .map_err(|e| format!("GET falló: {}", e))?;

        // Si devuelve 401/403, la sesión expiró
        if response.status() == reqwest::StatusCode::UNAUTHORIZED 
            || response.status() == reqwest::StatusCode::FORBIDDEN {
            return Ok(false);
        }

        // Si redirection a accounts.google.com, también expiró
        if let Some(loc) = response.headers().get("location") {
            if loc.to_str().unwrap_or("").contains("accounts.google.com") {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl Default for AuthHelper {
    fn default() -> Self {
        Self::new()
    }
}

use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_csrf_standard_format() {
        let html = r#"<script>var T="SNlM0e":"abc123xyz"</script>"#;
        let result = extract_csrf_from_html(html);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_csrf_not_found() {
        let html = r#"<div>No CSRF here</div>"#;
        let result = extract_csrf_from_html(html);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_csrf_empty_token() {
        let html = r#"<script>"SNlM0e":""</script>"#;
        let result = extract_csrf_from_html(html);
        assert!(result.is_err());
    }
}