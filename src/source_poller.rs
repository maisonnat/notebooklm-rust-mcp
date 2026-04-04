//! Source Poller - Espera hasta que una fuente esté lista para consulta
//!
//! Lecciones del reverse engineering (notebooklm-py):
//! - Después de subir una fuente, entra en estado asíncrono de indexación
//! - Si preguntas al chat inmediatamente, fallará (RAG-strict)
//! - Debe implementarse polling que consulte el estado hasta SUCCESS
//! - Timeout necesario para no quedar atascado infinitamente

use crate::errors::{NotebookLmError, NotebookResult};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Referencia al cliente de NotebookLM (para evitar Clone)
pub type NotebookClientRef = Arc<RwLock<crate::NotebookLmClient>>;

/// Configuración para el polling de fuentes
#[derive(Debug, Clone)]
pub struct PollerConfig {
    /// Intervalo entre verificaciones (default: 2 segundos)
    pub check_interval: Duration,
    /// Timeout máximo para esperar (default: 60 segundos)
    pub timeout: Duration,
    /// Máximo de reintentos antes de dar error (default: 30)
    pub max_retries: usize,
}

impl Default for PollerConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(2),
            timeout: Duration::from_secs(60),
            max_retries: 30,
        }
    }
}

/// Estado de una fuente según la API de NotebookLM
#[derive(Debug, Clone, PartialEq)]
pub enum SourceState {
    /// Fuente exitosa y lista para consultas
    Ready,
    /// Fuente todavía procesándose
    Processing,
    /// Hubo un error al procesar la fuente
    Error(String),
    /// Estado desconocido
    Unknown,
}

impl SourceState {
    /// Parsear el estado desde la respuesta de get_source
    pub fn from_response(response: &serde_json::Value) -> Self {
        if response.is_null() || response.as_array().map(|a| a.is_empty()).unwrap_or(true) {
            SourceState::Processing
        } else {
            SourceState::Ready
        }
    }
}

/// Poller para esperar hasta que una fuente esté lista
pub struct SourcePoller {
    client: NotebookClientRef,
    config: PollerConfig,
}

impl SourcePoller {
    /// Crear poller con Arc<RwLock<NotebookLmClient>>
    pub fn new(client: NotebookClientRef) -> Self {
        Self::with_config(client, PollerConfig::default())
    }

    pub fn with_config(client: NotebookClientRef, config: PollerConfig) -> Self {
        Self { client, config }
    }

    /// Espera hasta que la fuente esté lista para consulta
    pub async fn wait_for_source_ready(&self, notebook_id: &str, source_id: &str) -> NotebookResult<SourceState> {
        info!("Iniciando polling para fuente {} en notebook {}", source_id, notebook_id);
        
        let mut retries = 0;
        
        loop {
            if retries >= self.config.max_retries {
                warn!("Timeout: máximo de reintentos alcanzado para fuente {}", source_id);
                return Err(NotebookLmError::SourceNotReady(
                    format!("Timeout después de {} intentos", self.config.max_retries)
                ));
            }

            match self.check_source_state(notebook_id, source_id).await {
                Ok(state) => {
                    match state {
                        SourceState::Ready => {
                            info!("Fuente {} lista para consulta", source_id);
                            return Ok(state);
                        }
                        SourceState::Processing => {
                            info!("Fuente {} aún procesando (intento {}/{})", 
                                  source_id, retries + 1, self.config.max_retries);
                        }
                        SourceState::Error(msg) => {
                            return Err(NotebookLmError::SourceNotReady(msg));
                        }
                        SourceState::Unknown => {
                            warn!("Estado desconocido para fuente {}, continuando...", source_id);
                        }
                    }
                }
                Err(e) => {
                    warn!("Error al verificar fuente {}: {}", source_id, e);
                }
            }

            tokio::time::sleep(self.config.check_interval).await;
            retries += 1;
        }
    }

    /// Consulta el estado de una fuente específica
    async fn check_source_state(&self, notebook_id: &str, source_id: &str) -> NotebookResult<SourceState> {
        let client = self.client.read().await;
        
        let sources = client.get_notebook_sources(notebook_id)
            .await
            .map_err(NotebookLmError::from_string)?;
        
        if sources.contains(&source_id.to_string()) {
            Ok(SourceState::Ready)
        } else {
            Ok(SourceState::Processing)
        }
    }
}

/// Wrapper async para añadir fuente con polling automático
pub async fn add_source_with_polling(
    client: &Arc<RwLock<crate::NotebookLmClient>>,
    notebook_id: &str,
    title: &str,
    content: &str,
) -> NotebookResult<String> {
    let poller = SourcePoller::new(client.clone());
    
    // Primero añadir la fuente
    let source_id = {
        let c = client.read().await;
        c.add_source(notebook_id, title, content)
            .await
            .map_err(NotebookLmError::from_string)?
    };
    
    info!("Fuente añadida con ID: {}, esperando que esté lista...", source_id);
    
    // Luego esperar hasta que esté lista
    poller.wait_for_source_ready(notebook_id, &source_id).await?;
    
    Ok(source_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_state_from_response() {
        let empty: serde_json::Value = serde_json::Value::Null;
        assert_eq!(SourceState::from_response(&empty), SourceState::Processing);
    }

    #[test]
    fn test_poller_default_config() {
        let config = PollerConfig::default();
        assert_eq!(config.check_interval, Duration::from_secs(2));
        assert_eq!(config.timeout, Duration::from_secs(60));
        assert_eq!(config.max_retries, 30);
    }

    #[test]
    fn test_poller_custom_config() {
        let config = PollerConfig {
            check_interval: Duration::from_secs(5),
            timeout: Duration::from_secs(120),
            max_retries: 10,
        };
        assert_eq!(config.check_interval, Duration::from_secs(5));
        assert_eq!(config.timeout, Duration::from_secs(120));
        assert_eq!(config.max_retries, 10);
    }

    #[test]
    fn test_source_state_ready() {
        // Source con datos = ready
        let data = serde_json::json!([[["src-uuid"]]]);
        let state = SourceState::from_response(&data);
        assert_eq!(state, SourceState::Ready);
    }

    #[test]
    fn test_source_state_processing_empty_array() {
        let empty_arr: serde_json::Value = serde_json::json!([]);
        let state = SourceState::from_response(&empty_arr);
        assert_eq!(state, SourceState::Processing);
    }
}