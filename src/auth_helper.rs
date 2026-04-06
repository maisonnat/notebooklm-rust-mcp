//! Auth Helper - Extrae y refresca tokens CSRF y Session ID desde la página de NotebookLM
//!
//! Lecciones del reverse engineering (notebooklm-py):
//! - El token CSRF (SNlM0e) no es estático - debe extraerse del HTML
//! - El session ID (FdrFJe) se extrae del mismo HTML, va como f.sid en la URL
//! - Después de un error 400, intentar refresh silencioso y reintentar
//! - Ambos están en el HTML como: "SNlM0e":"valor" y "FdrFJe":"valor"
//!
//! Aclaración: Las cookies se manejan por fuera (DPAPI/Keyring)
//! Este módulo se encarga de los tokens CSRF y Session ID

use regex::Regex;
use tracing::info;

/// Extrae el token CSRF (SNlM0e) del HTML de NotebookLM
/// Retorna el valor del token o error si no se encuentra
pub fn extract_csrf_from_html(html: &str) -> Result<String, String> {
    // El token está en el HTML como: "SNlM0e":"valor"
    // También puede aparecer como: "SNlM0e":"valor","other_field":...
    let re = Regex::new(r#""SNlM0e"\s*:\s*"([^"]+)""#)
        .map_err(|e| format!("Regex inválido: {}", e))?;
    
    if let Some(caps) = re.captures(html)
        && let Some(token) = caps.get(1) {
            let token_str = token.as_str();
            if token_str.is_empty() {
                return Err("Token CSRF vacío".to_string());
            }
            info!("Token CSRF extraído: {} chars", token_str.len());
            return Ok(token_str.to_string());
        }
    
    // Backup: buscar en otros formatos posibles
    let re2 = Regex::new(r#"SNlM0e["']?\s*[:=]\s*["']([^"']+)["']"#)
        .map_err(|e| format!("Regex backup inválido: {}", e))?;
    
    if let Some(caps) = re2.captures(html)
        && let Some(token) = caps.get(1) {
            return Ok(token.as_str().to_string());
    }
    
    Err("No se encontró token CSRF (SNlM0e) en el HTML".to_string())
}

/// Extrae el session ID (FdrFJe) del HTML de NotebookLM.
/// Este valor se pasa como `f.sid` en la URL de batchexecute.
/// Sin él, Google retorna 200 con datos vacíos.
pub fn extract_session_id_from_html(html: &str) -> Result<String, String> {
    let re = Regex::new(r#""FdrFJe"\s*:\s*"([^"]+)""#)
        .map_err(|e| format!("Regex inválido: {}", e))?;
    
    if let Some(caps) = re.captures(html)
        && let Some(token) = caps.get(1) {
            let token_str = token.as_str();
            if token_str.is_empty() {
                return Err("Session ID vacío".to_string());
            }
            info!("Session ID (FdrFJe) extraído: {} chars", token_str.len());
            return Ok(token_str.to_string());
        }
    
    Err("No se encontró session ID (FdrFJe) en el HTML".to_string())
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
        let (csrf, _sid) = self.refresh_tokens(cookie).await?;
        Ok(csrf)
    }

    /// Hace un GET a la página principal y extrae AMBOS tokens:
    /// - CSRF (SNlM0e) → va en el body POST como campo `at=`
    /// - Session ID (FdrFJe) → va en la URL como `f.sid=`
    ///
    /// Ambos viven en el mismo HTML, en un `<script>` con `WIZ_global_data`.
    /// Un solo GET basta para obtener ambos.
    pub async fn refresh_tokens(&self, cookie: &str) -> Result<(String, String), String> {
        info!("Refrescando tokens CSRF y Session ID desde NotebookLM...");
        
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
        
        let csrf = extract_csrf_from_html(&html)?;
        let sid = extract_session_id_from_html(&html)
            .map_err(|e| {
                // CSRF found but no FdrFJe — this happens when Google changes the page.
                // Log warning but don't fail: f.sid is optional-ish (empty = omitted from URL)
                info!("No se pudo extraer FdrFJe: {}", e);
                String::new()
            })?;
        
        Ok((csrf, sid))
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
        if let Some(loc) = response.headers().get("location")
            && loc.to_str().unwrap_or("").contains("accounts.google.com") {
                return Ok(false);
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

    #[test]
    fn test_extract_session_id_standard_format() {
        let html = r#"<script>var WIZ_global_data={"FdrFJe":"-39204812345"}</script>"#;
        let result = extract_session_id_from_html(html);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "-39204812345");
    }

    #[test]
    fn test_extract_session_id_with_spaces() {
        let html = r#"<script>"FdrFJe" : "-12345"</script>"#;
        let result = extract_session_id_from_html(html);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_session_id_not_found() {
        let html = r#"<div>No session ID here</div>"#;
        let result = extract_session_id_from_html(html);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_session_id_empty() {
        let html = r#"<script>"FdrFJe":""</script>"#;
        let result = extract_session_id_from_html(html);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_both_tokens_same_html() {
        let html = r#"<script>WIZ_global_data={"SNlM0e":"AF1QpN-abc","FdrFJe":"-392048","other":"val"}</script>"#;
        let csrf = extract_csrf_from_html(html).unwrap();
        let sid = extract_session_id_from_html(html).unwrap();
        assert_eq!(csrf, "AF1QpN-abc");
        assert_eq!(sid, "-392048");
    }
}