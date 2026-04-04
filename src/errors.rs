//! Errores estructurados para NotebookLM Client
//!
//! Lecciones del reverse engineering:
//! - Cookies expiran frecuentemente (especialmente __Secure-1PSIDTS)
//! - CSRF token (SNlM0e) no es estático, necesita refresh
//! - No hacer panic - devolver errores estructurados para que el LLM pueda actuar

use std::fmt;

/// Enum de errores específicos de NotebookLM
#[derive(Debug, Clone)]
pub enum NotebookLmError {
    /// Sesión de Google expiró - el usuario debe actualizar las cookies
    SessionExpired(String),
    /// Token CSRF expiró o es inválido - intentar refresh automático
    CsrfExpired(String),
    /// La fuente aún no está indexada - hacer polling
    SourceNotReady(String),
    /// Rate limiting detectado - reducir concurrencia
    RateLimited(String),
    /// Error al parsear respuesta de Google
    ParseError(String),
    /// Error de red o HTTP
    NetworkError(String),
    /// Error genérico no categorizado
    Unknown(String),
}

impl fmt::Display for NotebookLmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotebookLmError::SessionExpired(msg) => write!(
                f,
                "SESIÓN EXPIRADA: {}. El usuario debe actualizar las cookies.",
                msg
            ),
            NotebookLmError::CsrfExpired(msg) => write!(
                f,
                "CSRF EXPIRADO: {}. Intentando refresh automático...",
                msg
            ),
            NotebookLmError::SourceNotReady(msg) => write!(
                f,
                "FUENTE NO LISTA: {}. Faça polling antes de consultar.",
                msg
            ),
            NotebookLmError::RateLimited(msg) => {
                write!(f, "RATE LIMITED: {}. Reducir concurrencia.", msg)
            }
            NotebookLmError::ParseError(msg) => write!(f, "ERROR DE PARSEO: {}", msg),
            NotebookLmError::NetworkError(msg) => write!(f, "ERROR DE RED: {}", msg),
            NotebookLmError::Unknown(msg) => write!(f, "ERROR DESCONOCIDO: {}", msg),
        }
    }
}

impl std::error::Error for NotebookLmError {}

impl NotebookLmError {
    /// Crear error desde string genérico - intentar detectar el tipo
    pub fn from_string(s: String) -> Self {
        let lower = s.to_lowercase();

        if lower.contains("401") || lower.contains("unauthorized") || lower.contains("session") {
            NotebookLmError::SessionExpired(s)
        } else if lower.contains("400") || lower.contains("csrf") || lower.contains("forbidden") {
            NotebookLmError::CsrfExpired(s)
        } else if lower.contains("429") || lower.contains("rate") || lower.contains("too many") {
            NotebookLmError::RateLimited(s)
        } else if lower.contains("parse") || lower.contains("json") {
            NotebookLmError::ParseError(s)
        } else if lower.contains("network")
            || lower.contains("connection")
            || lower.contains("timeout")
        {
            NotebookLmError::NetworkError(s)
        } else {
            NotebookLmError::Unknown(s)
        }
    }

    /// Verifica si es un error que requiere refresh de CSRF
    pub fn requires_csrf_refresh(&self) -> bool {
        matches!(self, NotebookLmError::CsrfExpired(_))
    }

    /// Verifica si es un error que requiere actualizar credenciales
    pub fn requires_new_credentials(&self) -> bool {
        matches!(self, NotebookLmError::SessionExpired(_))
    }
}

/// Resultado de una operación que puede fallar con error estructurado
pub type NotebookResult<T> = Result<T, NotebookLmError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NotebookLmError::SessionExpired("Cookie expirada".to_string());
        assert!(err.to_string().contains("SESIÓN EXPIRADA"));
    }

    #[test]
    fn test_error_from_string_detects_401() {
        let err = NotebookLmError::from_string("HTTP 401 Unauthorized".to_string());
        assert!(matches!(err, NotebookLmError::SessionExpired(_)));
    }

    #[test]
    fn test_error_from_string_detects_400() {
        let err = NotebookLmError::from_string("HTTP 400 Bad Request".to_string());
        assert!(matches!(err, NotebookLmError::CsrfExpired(_)));
    }

    #[test]
    fn test_requires_csrf_refresh() {
        let err = NotebookLmError::CsrfExpired("test".to_string());
        assert!(err.requires_csrf_refresh());

        let err2 = NotebookLmError::SessionExpired("test".to_string());
        assert!(!err2.requires_csrf_refresh());
    }

    #[test]
    fn test_error_detects_session_expired_various_forms() {
        let forms = vec![
            "401 Unauthorized",
            "HTTP 401",
            "unauthorized",
            "session expired",
            "Session invalid",
        ];

        for form in forms {
            let err = NotebookLmError::from_string(form.to_string());
            assert!(
                matches!(err, NotebookLmError::SessionExpired(_)),
                "Should detect session expired from: {}",
                form
            );
        }
    }

    #[test]
    fn test_error_detects_csrf_expired_various_forms() {
        let forms = vec!["400 Bad Request", "HTTP 400", "forbidden", "csrf invalid"];

        for form in forms {
            let err = NotebookLmError::from_string(form.to_string());
            assert!(
                matches!(err, NotebookLmError::CsrfExpired(_)),
                "Should detect CSRF expired from: {}",
                form
            );
        }
    }

    #[test]
    fn test_error_detects_rate_limited() {
        let forms = vec!["429 Too Many Requests", "rate limit", "too many requests"];

        for form in forms {
            let err = NotebookLmError::from_string(form.to_string());
            assert!(
                matches!(err, NotebookLmError::RateLimited(_)),
                "Should detect rate limited from: {}",
                form
            );
        }
    }

    #[test]
    fn test_requires_new_credentials() {
        let err_session = NotebookLmError::SessionExpired("expired".to_string());
        assert!(err_session.requires_new_credentials());

        let err_csrf = NotebookLmError::CsrfExpired("expired".to_string());
        assert!(!err_csrf.requires_new_credentials());

        let err_rate = NotebookLmError::RateLimited("too many".to_string());
        assert!(!err_rate.requires_new_credentials());
    }
}
