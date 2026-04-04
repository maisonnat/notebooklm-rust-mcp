//! Conversation cache para mantener historial de conversaciones por notebook.
//! Evita crear nuevos UUID de conversación cada pregunta - reutiliza el mismo.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Historial de una conversación (preguntas y respuestas)
#[derive(Debug, Clone, Default)]
pub struct ConversationHistory {
    pub messages: Vec<ConversationMessage>,
}

#[derive(Debug, Clone)]
pub struct ConversationMessage {
    pub question: String,
    pub answer: String,
}

/// Cache de conversaciones por notebook
/// Usa RwLock para permitir lectura concurrente y escritura exclusiva
pub struct ConversationCache {
    /// Mapa de notebook_id -> (conversation_id, historial)
    conversations: RwLock<HashMap<String, (String, ConversationHistory)>>,
}

impl ConversationCache {
    pub fn new() -> Self {
        Self {
            conversations: RwLock::new(HashMap::new()),
        }
    }

    /// Obtener o crear una conversación para un notebook
    /// Si no existe, crea una nueva con el conversation_id dado
    pub async fn get_or_create(&self, notebook_id: &str, conversation_id: &str) -> String {
        let mut cache = self.conversations.write().await;
        
        if let Some((existing_conv_id, _)) = cache.get(notebook_id) {
            // Ya existe conversación para este notebook - devolver el ID existente
            return existing_conv_id.clone();
        }
        
        // Crear nueva conversación
        cache.insert(notebook_id.to_string(), (
            conversation_id.to_string(),
            ConversationHistory::default(),
        ));
        conversation_id.to_string()
    }

    /// Agregar un mensaje al historial de un notebook
    pub async fn add_message(&self, notebook_id: &str, question: String, answer: String) {
        let mut cache = self.conversations.write().await;
        
        if let Some((_, history)) = cache.get_mut(notebook_id) {
            history.messages.push(ConversationMessage { question, answer });
        }
    }

    /// Obtener el historial de un notebook
    pub async fn get_history(&self, notebook_id: &str) -> Option<Vec<ConversationMessage>> {
        let cache = self.conversations.read().await;
        cache.get(notebook_id).map(|(_, h)| h.messages.clone())
    }

    /// Obtener el conversation_id actual de un notebook
    pub async fn get_conversation_id(&self, notebook_id: &str) -> Option<String> {
        let cache = self.conversations.read().await;
        cache.get(notebook_id).map(|(conv_id, _)| conv_id.clone())
    }

    /// Limpiar el historial de un notebook (nueva conversación)
    pub async fn reset(&self, notebook_id: &str) {
        let mut cache = self.conversations.write().await;
        cache.remove(notebook_id);
    }
}

impl Default for ConversationCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Tipo alias para usar en el cliente
pub type SharedConversationCache = Arc<ConversationCache>;

/// Crear una nueva instancia compartida del cache
pub fn new_conversation_cache() -> SharedConversationCache {
    Arc::new(ConversationCache::new())
}