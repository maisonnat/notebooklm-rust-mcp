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
    /// Archivo no encontrado en disco (file upload)
    FileNotFound(String),
    /// Falló la subida del archivo (upload session o streaming)
    UploadFailed(String),
    /// Input inválido (path es directorio, formato incorrecto, etc.)
    ValidationError(String),
    /// Artefacto aún no completado — no se puede descargar
    ArtifactNotReady(String),
    /// Artefacto no encontrado en el notebook
    ArtifactNotFound(String),
    /// Error al descargar artefacto (HTTP o parsing)
    DownloadFailed(String),
    /// Falló la generación del artefacto (no rate-limit)
    GenerationFailed(String),
    /// Circuit breaker abierto — demasiados errores de auth consecutivos
    CircuitOpen(String),
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
            NotebookLmError::FileNotFound(msg) => write!(f, "ARCHIVO NO ENCONTRADO: {}", msg),
            NotebookLmError::UploadFailed(msg) => write!(f, "ERROR DE SUBIDA: {}", msg),
            NotebookLmError::ValidationError(msg) => write!(f, "VALIDACIÓN FALLIDA: {}", msg),
            NotebookLmError::ArtifactNotReady(msg) => write!(
                f,
                "ARTEFACTO NO LISTO: {}. Esperar a que la generación complete.",
                msg
            ),
            NotebookLmError::ArtifactNotFound(msg) => {
                write!(f, "ARTEFACTO NO ENCONTRADO: {}", msg)
            }
            NotebookLmError::DownloadFailed(msg) => {
                write!(f, "ERROR DE DESCARGA: {}", msg)
            }
            NotebookLmError::GenerationFailed(msg) => {
                write!(f, "GENERACIÓN FALLIDA: {}", msg)
            }
            NotebookLmError::CircuitOpen(msg) => write!(
                f,
                "CIRCUIT BREAKER ABIERTO: {}. Ejecuta `notebooklm-mcp auth-browser` para re-autenticar.",
                msg
            ),
        }
    }
}

impl std::error::Error for NotebookLmError {}

impl NotebookLmError {
    /// Crear error desde string genérico - intentar detectar el tipo
    pub fn from_string(s: String) -> Self {
        let lower = s.to_lowercase();

        if lower.contains("401") || lower.contains("unauthorized") {
            NotebookLmError::SessionExpired(s)
        } else if lower.contains("400") || lower.contains("csrf") || lower.contains("forbidden") {
            NotebookLmError::CsrfExpired(s)
        } else if lower.contains("429") || lower.contains("rate") || lower.contains("too many") {
            NotebookLmError::RateLimited(s)
        } else if lower.contains("not found")
            || lower.contains("no such file")
            || lower.contains("file_not_found")
            || lower.contains("artifact_not_found")
        {
            // Disambiguate: if it mentions artifact, use ArtifactNotFound; otherwise FileNotFound
            if lower.contains("artifact") {
                NotebookLmError::ArtifactNotFound(s)
            } else {
                NotebookLmError::FileNotFound(s)
            }
        } else if lower.contains("upload")
            || lower.contains("upload_url")
            || lower.contains("resumable")
        {
            NotebookLmError::UploadFailed(s)
        } else if lower.contains("download")
            || lower.contains("stream")
            || lower.contains("write failed")
        {
            NotebookLmError::DownloadFailed(s)
        } else if lower.contains("generation failed")
            || lower.contains("artifact failed")
            || lower.contains("generation_error")
        {
            NotebookLmError::GenerationFailed(s)
        } else if lower.contains("not ready")
            || lower.contains("still processing")
            || lower.contains("artifact_not_ready")
        {
            NotebookLmError::ArtifactNotReady(s)
        } else if lower.contains("validation")
            || lower.contains("not a file")
            || lower.contains("is a directory")
        {
            NotebookLmError::ValidationError(s)
        } else if lower.contains("parse") || lower.contains("json") {
            NotebookLmError::ParseError(s)
        } else if lower.contains("network")
            || lower.contains("connection")
            || lower.contains("timeout")
        {
            NotebookLmError::NetworkError(s)
        } else if lower.contains("session") {
            NotebookLmError::SessionExpired(s)
        } else if lower.contains("circuit") || lower.contains("breaker") {
            NotebookLmError::CircuitOpen(s)
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
        matches!(
            self,
            NotebookLmError::SessionExpired(_) | NotebookLmError::CircuitOpen(_)
        )
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

    // =========================================================================
    // 6.5 — New error variant detection tests
    // =========================================================================

    #[test]
    fn test_error_detects_file_not_found() {
        let forms = vec![
            "File not found: /docs/research.pdf",
            "No such file or directory",
            "file_not_found error",
        ];
        for form in forms {
            let err = NotebookLmError::from_string(form.to_string());
            assert!(
                matches!(err, NotebookLmError::FileNotFound(_)),
                "Should detect FileNotFound from: {}",
                form
            );
        }
    }

    #[test]
    fn test_error_detects_upload_failed() {
        let forms = vec![
            "Upload session failed",
            "Response missing x-goog-upload-url header",
            "File stream upload failed: connection reset",
            "resumable upload error",
        ];
        for form in forms {
            let err = NotebookLmError::from_string(form.to_string());
            assert!(
                matches!(err, NotebookLmError::UploadFailed(_)),
                "Should detect UploadFailed from: {}",
                form
            );
        }
    }

    #[test]
    fn test_error_detects_validation_error() {
        let forms = vec![
            "ValidationError: Path is a directory",
            "Path is a directory, not a file: /tmp",
            "validation failed: invalid input",
        ];
        for form in forms {
            let err = NotebookLmError::from_string(form.to_string());
            assert!(
                matches!(err, NotebookLmError::ValidationError(_)),
                "Should detect ValidationError from: {}",
                form
            );
        }
    }

    #[test]
    fn test_new_error_variants_display() {
        let err = NotebookLmError::FileNotFound("/docs/research.pdf".to_string());
        assert!(err.to_string().contains("ARCHIVO NO ENCONTRADO"));
        assert!(err.to_string().contains("/docs/research.pdf"));

        let err = NotebookLmError::UploadFailed("session start failed".to_string());
        assert!(err.to_string().contains("ERROR DE SUBIDA"));

        let err = NotebookLmError::ValidationError("path is directory".to_string());
        assert!(err.to_string().contains("VALIDACIÓN FALLIDA"));
    }

    // =========================================================================
    // Module 2 — Artifact error variant tests
    // =========================================================================

    #[test]
    fn test_artifact_error_variants_display() {
        let err = NotebookLmError::ArtifactNotReady("still processing".to_string());
        assert!(err.to_string().contains("ARTEFACTO NO LISTO"));
        assert!(err.to_string().contains("still processing"));

        let err = NotebookLmError::ArtifactNotFound("art-123 not found".to_string());
        assert!(err.to_string().contains("ARTEFACTO NO ENCONTRADO"));

        let err = NotebookLmError::DownloadFailed("connection reset".to_string());
        assert!(err.to_string().contains("ERROR DE DESCARGA"));

        let err = NotebookLmError::GenerationFailed("USER_DISPLAYABLE_ERROR".to_string());
        assert!(err.to_string().contains("GENERACIÓN FALLIDA"));
    }

    #[test]
    fn test_error_from_string_detects_artifact_not_ready() {
        let forms = vec![
            "Artifact not ready: still processing",
            "artifact_not_ready error",
            "Still processing, please wait",
        ];
        for form in forms {
            let err = NotebookLmError::from_string(form.to_string());
            assert!(
                matches!(err, NotebookLmError::ArtifactNotReady(_)),
                "Should detect ArtifactNotReady from: {}",
                form
            );
        }
    }

    #[test]
    fn test_error_from_string_detects_artifact_not_found() {
        let forms = vec![
            "Artifact not found: art-abc-123",
            "artifact_not_found in notebook",
        ];
        for form in forms {
            let err = NotebookLmError::from_string(form.to_string());
            assert!(
                matches!(err, NotebookLmError::ArtifactNotFound(_)),
                "Should detect ArtifactNotFound from: {}",
                form
            );
        }
    }

    #[test]
    fn test_error_from_string_detects_download_failed() {
        let forms = vec![
            "Download failed: connection reset",
            "Stream error during download",
            "Write failed: disk full",
        ];
        for form in forms {
            let err = NotebookLmError::from_string(form.to_string());
            assert!(
                matches!(err, NotebookLmError::DownloadFailed(_)),
                "Should detect DownloadFailed from: {}",
                form
            );
        }
    }

    #[test]
    fn test_error_from_string_detects_generation_failed() {
        let forms = vec![
            "Generation failed: internal error",
            "artifact failed during processing",
            "generation_error: unknown reason",
        ];
        for form in forms {
            let err = NotebookLmError::from_string(form.to_string());
            assert!(
                matches!(err, NotebookLmError::GenerationFailed(_)),
                "Should detect GenerationFailed from: {}",
                form
            );
        }
    }

    #[test]
    fn test_error_from_string_disambiguates_file_vs_artifact_not_found() {
        // "artifact not found" → ArtifactNotFound
        let err = NotebookLmError::from_string("Artifact not found".to_string());
        assert!(matches!(err, NotebookLmError::ArtifactNotFound(_)));

        // plain "not found" → FileNotFound (backward compatible)
        let err = NotebookLmError::from_string("File not found: /path/to/file".to_string());
        assert!(matches!(err, NotebookLmError::FileNotFound(_)));
    }

    // =========================================================================
    // Module 6 — Circuit breaker error variant tests
    // =========================================================================

    #[test]
    fn test_circuit_open_display() {
        let err = NotebookLmError::CircuitOpen("3 consecutive auth errors".to_string());
        let msg = err.to_string();
        assert!(msg.contains("CIRCUIT BREAKER ABIERTO"));
        assert!(msg.contains("auth-browser"));
    }

    #[test]
    fn test_circuit_open_requires_new_credentials() {
        let err = NotebookLmError::CircuitOpen("test".to_string());
        assert!(err.requires_new_credentials());
    }

    #[test]
    fn test_from_string_detects_circuit_open() {
        let err = NotebookLmError::from_string("Circuit breaker is open".to_string());
        assert!(matches!(err, NotebookLmError::CircuitOpen(_)));

        let err2 =
            NotebookLmError::from_string("circuit breaker opened after 3 errors".to_string());
        assert!(matches!(err2, NotebookLmError::CircuitOpen(_)));
    }
}
