use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{Level, error, info, warn};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser)]
#[command(name = "notebooklm-mcp")]
#[command(about = "NotebookLM Unofficial MCP Server", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Autenticar guardando las cookies encriptadas con DPAPI
    Auth {
        #[arg(long)]
        cookie: String,
        #[arg(long)]
        csrf: String,
    },
    /// Autenticar usando navegador Chrome (headless) - recomendado
    AuthBrowser,
    /// Verificar estado de autenticación (disponibilidad de Chrome y credenciales)
    AuthStatus,
    /// Verifica la conexión con Google NotebookLM creando una libreta de prueba
    Verify,
    /// Hacer una pregunta a una libreta
    Ask {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// Pregunta a realizar
        #[arg(long)]
        question: String,
    },
    /// Añadir una fuente de texto a una libreta
    AddSource {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// Título de la fuente
        #[arg(long)]
        title: String,
        /// Contenido de la fuente
        #[arg(long)]
        content: String,
    },
    /// Añadir una fuente URL (web o YouTube) a una libreta
    AddUrl {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// URL a añadir como fuente
        #[arg(long)]
        url: String,
    },
    /// Añadir un documento de Google Drive como fuente a una libreta
    AddDrive {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// ID del archivo en Google Drive
        #[arg(long)]
        file_id: String,
        /// Título del documento
        #[arg(long)]
        title: String,
        /// MIME type del documento (default: Google Docs)
        #[arg(long)]
        mime_type: Option<String>,
    },
    /// Subir un archivo como fuente a una libreta (PDF, TXT, MD, EPUB, DOCX)
    AddFile {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// Ruta al archivo en disco
        #[arg(long)]
        file_path: String,
    },
    /// Listar artefactos de una libreta
    ArtifactList {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// Filtrar por tipo (audio, video, report, quiz, flashcards, mind_map, infographic, slide_deck, data_table)
        #[arg(long)]
        kind: Option<String>,
    },
    /// Generar un artefacto (audio, video, report, quiz, etc.)
    ArtifactGenerate {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// Tipo de artefacto
        #[arg(long)]
        kind: String,
        /// Instrucciones para la generación
        #[arg(long)]
        instructions: Option<String>,
        /// Idioma (default: en)
        #[arg(long, default_value = "en")]
        language: String,
        /// Formato de audio (deep_dive, brief, critique, debate)
        #[arg(long)]
        audio_format: Option<String>,
        /// Duración del audio (short, default, long)
        #[arg(long)]
        audio_length: Option<String>,
        /// Formato de video (explainer, brief, cinematic)
        #[arg(long)]
        video_format: Option<String>,
        /// Estilo de video (auto, classic, whiteboard, etc.)
        #[arg(long)]
        video_style: Option<String>,
        /// Dificultad del quiz/flashcards (easy, medium, hard)
        #[arg(long)]
        quiz_difficulty: Option<String>,
        /// Cantidad de preguntas/tarjetas (fewer, standard, more)
        #[arg(long)]
        quiz_quantity: Option<String>,
        /// Formato del reporte (briefing_doc, study_guide, blog_post, custom)
        #[arg(long)]
        report_format: Option<String>,
        /// Prompt personalizado para reporte custom
        #[arg(long)]
        custom_prompt: Option<String>,
    },
    /// Descargar un artefacto completado
    ArtifactDownload {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// ID del artefacto
        #[arg(long)]
        artifact_id: String,
        /// Ruta local para guardar el archivo
        #[arg(long)]
        output_path: String,
        /// Formato (solo para slide_deck: pdf o pptx)
        #[arg(long)]
        format: Option<String>,
    },
    /// Eliminar un artefacto
    ArtifactDelete {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// ID del artefacto
        #[arg(long)]
        artifact_id: String,
    },
    /// Eliminar una libreta
    Delete {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
    },
    /// Obtener detalles de una libreta (fuentes, ownership, fecha)
    Get {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
    },
    /// Renombrar una libreta
    Rename {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// Nuevo título
        #[arg(long)]
        title: String,
    },
    /// Obtener resumen IA y temas sugeridos de una libreta
    Summary {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
    },
    /// Ver estado de compartido de una libreta
    ShareStatus {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
    },
    /// Hacer pública o privada una libreta
    ShareSet {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// Hacer pública (anyone with the link)
        #[arg(long, group = "visibility")]
        public: bool,
        /// Hacer privada (solo acceso restringido)
        #[arg(long, group = "visibility")]
        private: bool,
    },
    /// Eliminar una fuente de una libreta
    SourceDelete {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// ID de la fuente a eliminar
        #[arg(long)]
        source_id: String,
    },
    /// Renombrar una fuente de una libreta
    SourceRename {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// ID de la fuente a renombrar
        #[arg(long)]
        source_id: String,
        /// Nuevo título para la fuente
        #[arg(long)]
        new_title: String,
    },
    /// Extraer el texto completo indexado de una fuente
    SourceGetFulltext {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// ID de la fuente
        #[arg(long)]
        source_id: String,
    },
    /// Crear una nota en una libreta
    NoteCreate {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// Título de la nota
        #[arg(long)]
        title: String,
        /// Contenido de la nota
        #[arg(long)]
        content: String,
    },
    /// Listar notas activas de una libreta
    NoteList {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
    },
    /// Eliminar una nota de una libreta (soft-delete)
    NoteDelete {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// ID de la nota
        #[arg(long)]
        note_id: String,
    },
    /// Obtener historial de chat oficial de Google para una libreta
    ChatHistory {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// Cantidad máxima de turns a obtener (default: 20)
        #[arg(long)]
        limit: Option<u32>,
    },
    /// Iniciar investigación profunda (Deep Research) en una libreta
    Research {
        /// UUID de la libreta
        #[arg(long)]
        notebook_id: String,
        /// Query de investigación
        #[arg(long)]
        query: String,
        /// Timeout en segundos (default: 300)
        #[arg(long)]
        timeout_secs: Option<u64>,
    },
    /// Verificar si hay una nueva versión disponible en GitHub Releases
    UpdateCheck,
}

use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NotebookCreateRequest {
    pub title: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SourceAddRequest {
    pub notebook_id: String,
    pub title: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct AskQuestionRequest {
    pub notebook_id: String,
    pub question: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SourceAddUrlRequest {
    pub notebook_id: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SourceAddDriveRequest {
    pub notebook_id: String,
    pub file_id: String,
    pub title: String,
    pub mime_type: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SourceAddFileRequest {
    pub notebook_id: String,
    pub file_path: String,
}

// --- Artifact MCP Tool Request Structs ---

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ArtifactListRequest {
    pub notebook_id: String,
    pub kind: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ArtifactGenerateRequest {
    pub notebook_id: String,
    pub kind: String,
    pub instructions: Option<String>,
    pub language: Option<String>,
    pub audio_format: Option<String>,
    pub audio_length: Option<String>,
    pub video_format: Option<String>,
    pub video_style: Option<String>,
    pub quiz_difficulty: Option<String>,
    pub quiz_quantity: Option<String>,
    pub report_format: Option<String>,
    pub infographic_orientation: Option<String>,
    pub infographic_detail: Option<String>,
    pub slide_deck_format: Option<String>,
    pub slide_deck_length: Option<String>,
    pub custom_prompt: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ArtifactDownloadRequest {
    pub notebook_id: String,
    pub artifact_id: String,
    pub output_path: String,
    pub format: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ArtifactDeleteRequest {
    pub notebook_id: String,
    pub artifact_id: String,
}

// --- Notebook Lifecycle & Sharing MCP Tool Request Structs ---

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NotebookDeleteRequest {
    pub notebook_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NotebookGetRequest {
    pub notebook_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NotebookRenameRequest {
    pub notebook_id: String,
    pub new_title: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NotebookSummaryRequest {
    pub notebook_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NotebookShareStatusRequest {
    pub notebook_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NotebookShareSetRequest {
    pub notebook_id: String,
    pub public: bool,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SourceDeleteRequest {
    pub notebook_id: String,
    pub source_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SourceRenameRequest {
    pub notebook_id: String,
    pub source_id: String,
    pub new_title: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SourceGetFulltextRequest {
    pub notebook_id: String,
    pub source_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NoteCreateRequest {
    pub notebook_id: String,
    pub title: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NoteListRequest {
    pub notebook_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct NoteDeleteRequest {
    pub notebook_id: String,
    pub note_id: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ChatHistoryRequest {
    pub notebook_id: String,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ResearchDeepDiveRequest {
    pub notebook_id: String,
    pub query: String,
    pub timeout_secs: Option<u64>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ResearchDeepDiveStartRequest {
    pub notebook_id: String,
    pub query: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ResearchDeepDiveStatusRequest {
    pub notebook_id: String,
    pub task_id: Option<String>,
}

pub mod artifact_poller;
pub mod auth_browser;
pub mod auth_helper;
pub mod browser_headers;
pub mod conversation_cache;
pub mod errors;
pub mod notebooklm_client;
pub mod parser;
pub mod research_poller;
pub mod rpc;
pub mod source_poller;
pub mod update_checker;
use notebooklm_client::NotebookLmClient;

// Definición de ServerState encapsulado
#[derive(Debug, Clone)]
pub struct ServerState {
    // Necesitamos que Client se pueda clonar y pasar entre hilos si NotebookLmServer hace derivar Clone.
    // Usualmente Arc<RwLock<T>> lo permite.
}

#[derive(Clone)]
pub struct NotebookLmServer {
    state: Arc<RwLock<Option<NotebookLmClient>>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl NotebookLmServer {
    #[tool(
        name = "notebook_list",
        description = "List all notebooks available in the account"
    )]
    pub async fn notebook_list(&self) -> String {
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c.list_notebooks().await {
                Ok(ls) => format!("Notebooks: {:?}", ls),
                Err(e) => format!("Error listando cuadernos: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(name = "source_add", description = "Add a text source to a notebook")]
    pub async fn source_add(&self, req: Parameters<SourceAddRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .add_source(&request.notebook_id, &request.title, &request.content)
                .await
            {
                Ok(id) => format!("Fuente añadida. ID: {}", id),
                Err(e) => format!("Error añadiendo fuente: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(name = "ask_question", description = "Ask a question to a notebook")]
    pub async fn ask_question(&self, req: Parameters<AskQuestionRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .ask_question(&request.notebook_id, &request.question)
                .await
            {
                Ok(ans) => ans,
                Err(e) => format!("Error haciendo pregunta: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "source_add_url",
        description = "Add a web URL or YouTube video as a source to a notebook. Auto-detects YouTube URLs."
    )]
    pub async fn source_add_url(&self, req: Parameters<SourceAddUrlRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c.add_url_source(&request.notebook_id, &request.url).await {
                Ok(id) => format!("Fuente URL añadida. ID: {}", id),
                Err(e) => format!("Error añadiendo fuente URL: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "source_add_drive",
        description = "Add a Google Drive document as a source to a notebook. Provide file_id, title, and optionally mime_type (defaults to Google Docs)."
    )]
    pub async fn source_add_drive(&self, req: Parameters<SourceAddDriveRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            let mime = request.mime_type.as_deref().unwrap_or("");
            match c
                .add_drive_source(&request.notebook_id, &request.file_id, &request.title, mime)
                .await
            {
                Ok(id) => format!("Fuente Drive añadida. ID: {}", id),
                Err(e) => format!("Error añadiendo fuente Drive: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "source_add_file",
        description = "Upload a file (PDF, TXT, MD, EPUB, DOCX) as a source to a notebook using Google's resumable upload protocol."
    )]
    pub async fn source_add_file(&self, req: Parameters<SourceAddFileRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .add_file_source(&request.notebook_id, &request.file_path)
                .await
            {
                Ok(id) => format!("Fuente archivo añadida. ID: {}", id),
                Err(e) => format!("Error añadiendo fuente archivo: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "artifact_list",
        description = "List all artifacts in a notebook with type, status, and metadata. Optionally filter by kind (audio, video, report, quiz, flashcards, mind_map, infographic, slide_deck, data_table)."
    )]
    pub async fn artifact_list(&self, req: Parameters<ArtifactListRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            let kind_filter = request
                .kind
                .as_deref()
                .and_then(notebooklm_client::ArtifactType::from_str_key);
            match c.list_artifacts(&request.notebook_id, kind_filter).await {
                Ok(artifacts) => {
                    if artifacts.is_empty() {
                        return format!("No artifacts found in notebook {}", request.notebook_id);
                    }
                    let lines: Vec<String> = artifacts
                        .iter()
                        .map(|a| {
                            format!(
                                "- [{}] {} (status: {}, id: {})",
                                a.kind, a.title, a.status, a.id
                            )
                        })
                        .collect();
                    format!(
                        "Artifacts in notebook {} ({} total):\n{}",
                        request.notebook_id,
                        artifacts.len(),
                        lines.join("\n")
                    )
                }
                Err(e) => format!("Error listing artifacts: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "artifact_generate",
        description = "Generate an AI artifact in a notebook. Supports: audio, video, report, quiz, flashcards, infographic, slide_deck, data_table, mind_map. Returns a task_id for polling. Use artifact_list to check status."
    )]
    pub async fn artifact_generate(&self, req: Parameters<ArtifactGenerateRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            let lang = request.language.as_deref().unwrap_or("en");

            // Mind map uses a separate method
            if request.kind == "mind_map" {
                match c.generate_mind_map(&request.notebook_id, &[]).await {
                    Ok(result) => {
                        let note_id = result.note_id.as_deref().unwrap_or("unknown");
                        format!("Mind map generated. Note ID: {}", note_id)
                    }
                    Err(e) => format!("Error generating mind map: {}", e),
                }
            } else {
                // Build ArtifactConfig from kind + optional params
                let config = match build_artifact_config(&request.kind, request, lang) {
                    Ok(cfg) => cfg,
                    Err(e) => return format!("Error: {}", e),
                };

                match c.generate_artifact(&request.notebook_id, &config).await {
                    Ok(status) => {
                        let mut msg = format!(
                            "Artifact generation started.\nTask ID: {}\nStatus: {}",
                            status.task_id, status.status
                        );
                        if status.is_rate_limited() {
                            msg.push_str("\n⚠ Rate limited — retry after a delay.");
                        }
                        msg.push_str("\nUse artifact_list to check when status becomes COMPLETED.");
                        msg
                    }
                    Err(e) => format!("Error generating artifact: {}", e),
                }
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "artifact_download",
        description = "Download a completed artifact to a local file. Supports audio (mp4), video (mp4), report (markdown), quiz/flashcards (json), infographic (png), slide_deck (pdf/pptx), data_table (csv), mind_map (json)."
    )]
    pub async fn artifact_download(&self, req: Parameters<ArtifactDownloadRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .download_artifact(
                    &request.notebook_id,
                    &request.artifact_id,
                    &request.output_path,
                    request.format.as_deref(),
                )
                .await
            {
                Ok(path) => format!("✅ Artifact downloaded to: {}", path),
                Err(e) => format!("Error downloading artifact: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "artifact_delete",
        description = "Delete an artifact from a notebook."
    )]
    pub async fn artifact_delete(&self, req: Parameters<ArtifactDeleteRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .delete_artifact(&request.notebook_id, &request.artifact_id)
                .await
            {
                Ok(()) => format!("✅ Artifact {} deleted.", request.artifact_id),
                Err(e) => format!("Error deleting artifact: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    // --- Notebook Lifecycle & Sharing Tools ---

    #[tool(
        name = "notebook_delete",
        description = "Delete a notebook by ID. Idempotent — does not error if the notebook does not exist."
    )]
    pub async fn notebook_delete(&self, req: Parameters<NotebookDeleteRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c.delete_notebook(&request.notebook_id).await {
                Ok(()) => format!("✅ Notebook {} deleted.", request.notebook_id),
                Err(e) => format!("Error deleting notebook: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "notebook_get",
        description = "Get full notebook details by ID including sources count, ownership, and creation date."
    )]
    pub async fn notebook_get(&self, req: Parameters<NotebookGetRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c.get_notebook(&request.notebook_id).await {
                Ok(nb) => format!(
                    "Notebook: \"{}\" (id: {})\n  Sources: {}\n  Owner: {}\n  Created: {}",
                    nb.title,
                    nb.id,
                    nb.sources_count,
                    if nb.is_owner { "yes" } else { "no" },
                    nb.created_at.as_deref().unwrap_or("unknown")
                ),
                Err(e) => format!("Error getting notebook: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "notebook_rename",
        description = "Rename a notebook. Returns the updated notebook details."
    )]
    pub async fn notebook_rename(&self, req: Parameters<NotebookRenameRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .rename_notebook(&request.notebook_id, &request.new_title)
                .await
            {
                Ok(nb) => format!(
                    "✅ Notebook renamed to \"{}\" (id: {})\n  Sources: {}\n  Owner: {}",
                    nb.title,
                    nb.id,
                    nb.sources_count,
                    if nb.is_owner { "yes" } else { "no" }
                ),
                Err(e) => format!("Error renaming notebook: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "notebook_summary",
        description = "Get AI-generated summary and suggested topics for a notebook."
    )]
    pub async fn notebook_summary(&self, req: Parameters<NotebookSummaryRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c.get_summary(&request.notebook_id).await {
                Ok(s) => {
                    let mut msg = format!("Summary:\n{}\n", s.summary);
                    if !s.suggested_topics.is_empty() {
                        msg.push_str(&format!(
                            "\nSuggested Topics ({}):\n{}",
                            s.suggested_topics.len(),
                            s.suggested_topics
                                .iter()
                                .enumerate()
                                .map(|(i, t)| {
                                    format!(
                                        "  {}. {}\n     Prompt: {}",
                                        i + 1,
                                        t.question,
                                        t.prompt
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        ));
                    }
                    msg
                }
                Err(e) => format!("Error getting summary: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "notebook_share_status",
        description = "Get sharing configuration for a notebook — public/private status and list of shared users."
    )]
    pub async fn notebook_share_status(
        &self,
        req: Parameters<NotebookShareStatusRequest>,
    ) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c.get_share_status(&request.notebook_id).await {
                Ok(s) => {
                    let mut msg = format!(
                        "Sharing status for notebook {}:\n  Public: {}\n  Access: {:?}",
                        request.notebook_id, s.is_public, s.access
                    );
                    if let Some(url) = &s.share_url {
                        msg.push_str(&format!("\n  Share URL: {}", url));
                    }
                    if !s.shared_users.is_empty() {
                        msg.push_str(&format!(
                            "\n\nShared Users ({}):\n{}",
                            s.shared_users.len(),
                            s.shared_users
                                .iter()
                                .map(|u| {
                                    format!(
                                        "  - {} ({:?}){}",
                                        u.email,
                                        u.permission,
                                        u.display_name
                                            .as_ref()
                                            .map(|n| format!(" — {}", n))
                                            .unwrap_or_default()
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join("\n")
                        ));
                    }
                    msg
                }
                Err(e) => format!("Error getting share status: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "notebook_share_set",
        description = "Set a notebook to public (anyone with the link can view) or private (restricted). Returns the updated sharing status."
    )]
    pub async fn notebook_share_set(&self, req: Parameters<NotebookShareSetRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .set_sharing_public(&request.notebook_id, request.public)
                .await
            {
                Ok(s) => {
                    let mut msg = format!(
                        "✅ Notebook {} is now {}\n  Access: {:?}",
                        request.notebook_id,
                        if s.is_public { "PUBLIC" } else { "PRIVATE" },
                        s.access
                    );
                    if let Some(url) = &s.share_url {
                        msg.push_str(&format!("\n  Share URL: {}", url));
                    }
                    msg
                }
                Err(e) => format!("Error setting sharing: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "source_delete",
        description = "Delete a source from a notebook. Idempotent — does not error if the source doesn't exist."
    )]
    pub async fn source_delete(&self, req: Parameters<SourceDeleteRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .delete_source(&request.notebook_id, &request.source_id)
                .await
            {
                Ok(()) => format!(
                    "✅ Fuente {} eliminada del cuaderno {}",
                    request.source_id, request.notebook_id
                ),
                Err(e) => format!("Error eliminando fuente: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "source_rename",
        description = "Rename a source in a notebook. Returns confirmation."
    )]
    pub async fn source_rename(&self, req: Parameters<SourceRenameRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .rename_source(&request.notebook_id, &request.source_id, &request.new_title)
                .await
            {
                Ok(()) => format!(
                    "✅ Fuente {} renombrada a '{}' en cuaderno {}",
                    request.source_id, request.new_title, request.notebook_id
                ),
                Err(e) => format!("Error renombrando fuente: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "source_get_fulltext",
        description = "Get the full indexed text of a source (extracted by Google from PDFs, web pages, etc.). Returns the complete text content."
    )]
    pub async fn source_get_fulltext(&self, req: Parameters<SourceGetFulltextRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .get_source_fulltext(&request.notebook_id, &request.source_id)
                .await
            {
                Ok(text) => text,
                Err(e) => format!("Error extrayendo texto de fuente: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "note_create",
        description = "Create a note in a notebook with a title and content. Notes are visible in the NotebookLM web UI."
    )]
    pub async fn note_create(&self, req: Parameters<NoteCreateRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .create_note(&request.notebook_id, &request.title, &request.content)
                .await
            {
                Ok(id) => format!("✅ Nota creada. ID: {}", id),
                Err(e) => format!("Error creando nota: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "note_list",
        description = "List all active notes in a notebook (excludes soft-deleted notes)."
    )]
    pub async fn note_list(&self, req: Parameters<NoteListRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c.list_notes(&request.notebook_id).await {
                Ok(notes) => {
                    if notes.is_empty() {
                        "No hay notas en este cuaderno.".to_string()
                    } else {
                        notes
                            .iter()
                            .map(|n| format!("- [{}] {}", n.id, n.title))
                            .collect::<Vec<_>>()
                            .join("\n")
                    }
                }
                Err(e) => format!("Error listando notas: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "note_delete",
        description = "Delete a note from a notebook (soft-delete)."
    )]
    pub async fn note_delete(&self, req: Parameters<NoteDeleteRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c.delete_note(&request.notebook_id, &request.note_id).await {
                Ok(()) => format!(
                    "✅ Nota {} eliminada del cuaderno {}",
                    request.note_id, request.notebook_id
                ),
                Err(e) => format!("Error eliminando nota: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "chat_history",
        description = "Get the official chat conversation history from Google servers for a notebook. Returns turns in chronological order."
    )]
    pub async fn chat_history(&self, req: Parameters<ChatHistoryRequest>) -> String {
        let request = &req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            let limit = request.limit.unwrap_or(20);
            match c.get_last_conversation_id(&request.notebook_id).await {
                Ok(Some(conv_id)) => {
                    match c
                        .get_conversation_turns(&request.notebook_id, &conv_id, limit)
                        .await
                    {
                        Ok(turns) => {
                            if turns.is_empty() {
                                format!(
                                    "No conversation history found for notebook {} (conv_id: {})",
                                    request.notebook_id, conv_id
                                )
                            } else {
                                let lines: Vec<String> = turns
                                    .iter()
                                    .map(|t| format!("[{}] {}", t.role, t.text))
                                    .collect();
                                format!(
                                    "Chat history ({} turns, conv_id: {}):\n{}",
                                    turns.len(),
                                    conv_id,
                                    lines.join("\n\n")
                                )
                            }
                        }
                        Err(e) => format!("Error getting conversation turns: {}", e),
                    }
                }
                Ok(None) => format!("No conversation found for notebook {}", request.notebook_id),
                Err(e) => format!("Error getting conversation ID: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    // --- Research Deep Dive Tools ---

    #[tool(
        name = "research_deep_dive_start",
        description = "Start a deep research investigation using Google's autonomous research engine. Returns a task_id immediately without blocking. Use research_deep_dive_status to poll progress."
    )]
    pub async fn research_deep_dive_start(
        &self,
        req: Parameters<ResearchDeepDiveStartRequest>,
    ) -> String {
        let request = req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c
                .start_deep_research(&request.notebook_id, &request.query)
                .await
            {
                Ok(task_id) => {
                    info!("Deep research started. Task ID: {}", task_id);
                    serde_json::json!({
                        "task_id": task_id,
                        "notebook_id": request.notebook_id,
                        "query": request.query,
                        "status": "started"
                    })
                    .to_string()
                }
                Err(e) => format!("Error starting deep research: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(
        name = "research_deep_dive_status",
        description = "Check the status of a deep research task. Non-blocking — returns current status immediately. Returns status, sources found, and report markdown when complete."
    )]
    pub async fn research_deep_dive_status(
        &self,
        req: Parameters<ResearchDeepDiveStatusRequest>,
    ) -> String {
        let request = req.0;
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            let tid = request.task_id.as_deref().unwrap_or("");
            match c.poll_research_status(&request.notebook_id, tid).await {
                Ok(status) => {
                    let status_str = if status.is_complete {
                        "completed"
                    } else if status.status_code == 0 {
                        "no_research"
                    } else {
                        "in_progress"
                    };

                    let sources_json: Vec<serde_json::Value> = status
                        .sources
                        .iter()
                        .map(|s| {
                            serde_json::json!({
                                "url": s.url,
                                "title": s.title,
                                "description": s.description,
                                "result_type": s.result_type
                            })
                        })
                        .collect();

                    serde_json::json!({
                        "status": status_str,
                        "status_code": status.status_code,
                        "task_id": request.task_id,
                        "query": status.query,
                        "sources_found": sources_json.len(),
                        "sources": sources_json,
                        "report": status.report,
                        "is_complete": status.is_complete
                    })
                    .to_string()
                }
                Err(e) => format!("Error polling research status: {}", e),
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    /// DEPRECATED: Use research_deep_dive_start + research_deep_dive_status instead.
    /// This tool will be removed in the next major release.
    #[tool(
        name = "research_deep_dive",
        description = "DEPRECATED: Use research_deep_dive_start + research_deep_dive_status instead. This tool will be removed in the next major release. Start a deep research investigation using Google's autonomous research engine. Blocks until complete (up to timeout), then imports discovered sources into the notebook."
    )]
    pub async fn research_deep_dive(&self, req: Parameters<ResearchDeepDiveRequest>) -> String {
        let request = req.0;
        let timeout = request.timeout_secs.unwrap_or(300);
        let notebook_id = request.notebook_id.clone();
        let query = request.query.clone();

        let task_id = {
            let lock = self.state.read().await;
            if let Some(c) = &*lock {
                match c.start_deep_research(&notebook_id, &query).await {
                    Ok(id) => id,
                    Err(e) => return format!("Error starting deep research: {}", e),
                }
            } else {
                return "Error: Servidor no autenticado".into();
            }
        };

        info!(
            "Deep research started (deprecated wrapper). Task ID: {}. Timeout: {}s",
            task_id, timeout
        );

        // Blocking poll with exponential backoff: 2s → 4s → 8s → 10s cap
        let start = std::time::Instant::now();
        let mut interval = std::time::Duration::from_secs(2);
        let max_interval = std::time::Duration::from_secs(10);

        loop {
            if start.elapsed().as_secs() >= timeout {
                return format!(
                    "Deep research timed out after {}s. Task ID: {}",
                    timeout, task_id
                );
            }

            let status = {
                let lock = self.state.read().await;
                if let Some(c) = &*lock {
                    match c.poll_research_status(&notebook_id, &task_id).await {
                        Ok(s) => s,
                        Err(e) => return format!("Error polling research status: {}", e),
                    }
                } else {
                    return "Error: Servidor no autenticado".into();
                }
            };

            if status.is_complete {
                let lock = self.state.read().await;
                if let Some(c) = &*lock {
                    match c
                        .import_research_sources(&notebook_id, &task_id, serde_json::json!([]))
                        .await
                    {
                        Ok(()) => {
                            return format!(
                                "✅ Deep research complete. Sources imported. Task ID: {}. Sources found: {}",
                                task_id,
                                status.sources.len()
                            );
                        }
                        Err(e) => {
                            return format!(
                                "Research complete but failed to import sources: {}",
                                e
                            );
                        }
                    }
                } else {
                    return "Error: Servidor no autenticado".into();
                }
            }

            info!(
                "Research status: code={} — polling in {:?}...",
                status.status_code, interval
            );
            tokio::time::sleep(interval).await;
            interval = (interval * 2).min(max_interval);
        }
    }

    #[tool(
        name = "check_for_updates",
        description = "Check if a new version of notebooklm-mcp is available from GitHub Releases. Returns current version, latest version, and download URL if update available."
    )]
    pub async fn check_for_updates(&self) -> String {
        let current = env!("CARGO_PKG_VERSION");
        match crate::update_checker::check_for_updates_async(current).await {
            Ok(result) => serde_json::json!({
                "current_version": result.current_version,
                "latest_version": result.latest_version,
                "update_available": result.update_available,
                "download_url": result.download_url
            })
            .to_string(),
            Err(e) => format!("Error checking for updates: {}", e),
        }
    }
}

#[tool_handler]
impl ServerHandler for NotebookLmServer {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo::new(
            rmcp::model::ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build()
        )
        .with_server_info(
            rmcp::model::Implementation::new("notebooklm-mcp", env!("CARGO_PKG_VERSION"))
                .with_description("NotebookLM Unofficial MCP Server")
        )
        .with_instructions("Use notebook:// URIs to access NotebookLM notebooks as resources. Tools are also available for creating notebooks and adding sources.")
    }

    #[allow(clippy::manual_async_fn)]
    fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListResourcesResult, rmcp::ErrorData>>
    + Send
    + '_ {
        async {
            let lock = self.state.read().await;
            if let Some(client) = &*lock {
                match client.list_notebooks().await {
                    Ok(notebooks) => {
                        let resources: Vec<rmcp::model::Resource> = notebooks
                            .iter()
                            .map(|nb| {
                                let uri = format!("notebook://{}", nb.id);
                                let mut raw = rmcp::model::RawResource::new(&uri, &nb.title);
                                raw.description =
                                    Some(format!("NotebookLM notebook: {}", nb.title));
                                raw.mime_type = Some("application/json".to_string());
                                rmcp::model::Resource {
                                    raw,
                                    annotations: None,
                                }
                            })
                            .collect();
                        Ok(rmcp::model::ListResourcesResult::with_all_items(resources))
                    }
                    Err(e) => Err(rmcp::ErrorData::internal_error(e, None)),
                }
            } else {
                Err(rmcp::ErrorData::internal_error(
                    "Servidor no autenticado",
                    None,
                ))
            }
        }
    }

    #[allow(clippy::manual_async_fn)]
    fn read_resource(
        &self,
        request: rmcp::model::ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ReadResourceResult, rmcp::ErrorData>>
    + Send
    + '_ {
        async move {
            let uri = &request.uri;

            if let Some(notebook_id) = uri.strip_prefix("notebook://") {
                let lock = self.state.read().await;
                if let Some(client) = &*lock {
                    match client.list_notebooks().await {
                        Ok(notebooks) => {
                            if let Some(nb) = notebooks.iter().find(|n| n.id == notebook_id) {
                                let content = serde_json::json!({
                                    "id": nb.id,
                                    "title": nb.title,
                                    "uri": uri,
                                });
                                Ok(rmcp::model::ReadResourceResult::new(vec![
                                    rmcp::model::ResourceContents::text(
                                        content.to_string(),
                                        uri.to_string(),
                                    ),
                                ]))
                            } else {
                                Err(rmcp::ErrorData::resource_not_found(
                                    format!("Notebook {} no encontrado", notebook_id),
                                    None,
                                ))
                            }
                        }
                        Err(e) => Err(rmcp::ErrorData::internal_error(e, None)),
                    }
                } else {
                    Err(rmcp::ErrorData::internal_error(
                        "Servidor no autenticado",
                        None,
                    ))
                }
            } else {
                Err(rmcp::ErrorData::invalid_params(
                    format!("URI no soportada: {}. Use notebook://<uuid>", uri),
                    None,
                ))
            }
        }
    }
}

// --- Artifact Config Builder ---

use notebooklm_client::{
    ArtifactConfig, AudioFormat, AudioLength, InfographicDetail, InfographicOrientation,
    InfographicStyle, QuizDifficulty, QuizQuantity, ReportFormat, SlideDeckFormat, SlideDeckLength,
    VideoFormat, VideoStyle,
};

/// Build an `ArtifactConfig` from the MCP tool request parameters.
fn build_artifact_config(
    kind: &str,
    req: &ArtifactGenerateRequest,
    lang: &str,
) -> Result<ArtifactConfig, String> {
    let instructions = req.instructions.as_ref().cloned();

    match kind {
        "audio" => {
            let format = req
                .audio_format
                .as_ref()
                .and_then(|v| AudioFormat::from_str_key(v))
                .unwrap_or(AudioFormat::DeepDive);
            let length = req
                .audio_length
                .as_ref()
                .and_then(|v| AudioLength::from_str_key(v))
                .unwrap_or(AudioLength::Default);
            Ok(ArtifactConfig::Audio {
                format,
                length,
                instructions,
                language: lang.to_string(),
                source_ids: vec![],
            })
        }
        "video" => {
            let format = match req.video_format.as_deref() {
                Some("cinematic") => VideoFormat::Cinematic,
                Some(v) => VideoFormat::from_str_key(v).ok_or_else(|| {
                    format!(
                        "Invalid video_format: '{}'. Valid: explainer, brief, cinematic",
                        v
                    )
                })?,
                None => VideoFormat::Explainer,
            };
            let style = req
                .video_style
                .as_ref()
                .and_then(|v| VideoStyle::from_str_key(v));
            Ok(ArtifactConfig::Video {
                format,
                style,
                instructions,
                language: lang.to_string(),
                source_ids: vec![],
            })
        }
        "report" => {
            let format = match req.report_format.as_deref() {
                Some("custom") => {
                    let prompt = req.custom_prompt.clone().unwrap_or_default();
                    if prompt.is_empty() {
                        return Err("custom_prompt is required when report_format=custom".to_string());
                    }
                    ReportFormat::Custom { prompt }
                }
                Some(v) => ReportFormat::from_str_key(v)
                    .ok_or_else(|| format!("Invalid report_format: '{}'. Valid: briefing_doc, study_guide, blog_post, custom", v))?,
                None => ReportFormat::BriefingDoc,
            };
            Ok(ArtifactConfig::Report {
                format,
                language: lang.to_string(),
                source_ids: vec![],
                extra_instructions: instructions,
            })
        }
        "quiz" => {
            let difficulty = req
                .quiz_difficulty
                .as_ref()
                .and_then(|v| QuizDifficulty::from_str_key(v))
                .unwrap_or(QuizDifficulty::Medium);
            let quantity = req
                .quiz_quantity
                .as_ref()
                .and_then(|v| QuizQuantity::from_str_key(v))
                .unwrap_or(QuizQuantity::Standard);
            Ok(ArtifactConfig::Quiz {
                difficulty,
                quantity,
                instructions,
                source_ids: vec![],
            })
        }
        "flashcards" => {
            let difficulty = req
                .quiz_difficulty
                .as_ref()
                .and_then(|v| QuizDifficulty::from_str_key(v))
                .unwrap_or(QuizDifficulty::Medium);
            let quantity = req
                .quiz_quantity
                .as_ref()
                .and_then(|v| QuizQuantity::from_str_key(v))
                .unwrap_or(QuizQuantity::Standard);
            Ok(ArtifactConfig::Flashcards {
                difficulty,
                quantity,
                instructions,
                source_ids: vec![],
            })
        }
        "infographic" => {
            let orientation = req
                .infographic_orientation
                .as_ref()
                .and_then(|v| InfographicOrientation::from_str_key(v))
                .unwrap_or(InfographicOrientation::Landscape);
            let detail = req
                .infographic_detail
                .as_ref()
                .and_then(|v| InfographicDetail::from_str_key(v))
                .unwrap_or(InfographicDetail::Standard);
            Ok(ArtifactConfig::Infographic {
                orientation,
                detail,
                style: InfographicStyle::AutoSelect,
                instructions,
                language: lang.to_string(),
                source_ids: vec![],
            })
        }
        "slide_deck" => {
            let format = req
                .slide_deck_format
                .as_ref()
                .and_then(|v| SlideDeckFormat::from_str_key(v))
                .unwrap_or(SlideDeckFormat::DetailedDeck);
            let length = req
                .slide_deck_length
                .as_ref()
                .and_then(|v| SlideDeckLength::from_str_key(v))
                .unwrap_or(SlideDeckLength::Default);
            Ok(ArtifactConfig::SlideDeck {
                format,
                length,
                instructions,
                language: lang.to_string(),
                source_ids: vec![],
            })
        }
        "data_table" => Ok(ArtifactConfig::DataTable {
            instructions: req.instructions.clone().unwrap_or_default(),
            language: lang.to_string(),
            source_ids: vec![],
        }),
        _ => Err(format!(
            "Unknown artifact kind: '{}'. Valid: audio, video, report, quiz, flashcards, infographic, slide_deck, data_table, mind_map",
            kind
        )),
    }
}

// --- DPAPI Session Management ---

#[derive(Serialize, Deserialize)]
struct SessionData {
    cookie: String,
    csrf: String,
    /// Session ID (FdrFJe) — defaults to empty string for backward compat
    /// with sessions saved before this field was added.
    #[serde(default)]
    sid: String,
}

fn session_path() -> PathBuf {
    let home = dirs::home_dir().expect("No se pudo encontrar el directorio home");
    home.join(".notebooklm-mcp").join("session.bin")
}

fn save_session(data: &SessionData) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_vec(data)?;
    let encrypted = windows_dpapi::encrypt_data(&json, windows_dpapi::Scope::User, None)?;

    let path = session_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, &encrypted)?;
    Ok(())
}

fn load_session() -> Result<SessionData, Box<dyn std::error::Error>> {
    let path = session_path();
    let encrypted = std::fs::read(&path)?;
    let decrypted = windows_dpapi::decrypt_data(&encrypted, windows_dpapi::Scope::User, None)?;
    let data: SessionData = serde_json::from_slice(&decrypted)?;
    Ok(data)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(std::io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Fallo al establecer el subscriber de tracing");

    let cli = Cli::parse();

    // --- Comando Auth: encriptar y guardar en disco ---
    if let Some(Commands::Auth { cookie, csrf }) = &cli.command {
        let session = SessionData {
            cookie: cookie.clone(),
            csrf: csrf.clone(),
            sid: String::new(),
        };
        save_session(&session)?;
        info!(
            "Credenciales encriptadas con DPAPI y guardadas en {:?} ({} bytes de cookie).",
            session_path(),
            cookie.len()
        );
        return Ok(());
    }

    // --- Comando AuthBrowser: autenticación vía Chrome headless ---
    if let Some(Commands::AuthBrowser) = &cli.command {
        use crate::auth_browser::{AuthResult, BrowserAuthenticator, BrowserCredentials};

        println!("=== AUTENTICACIÓN POR NAVEGADOR ===");
        println!("Se abrirá una ventana del navegador Chrome.");
        println!("Por favor, inicia sesión en tu cuenta de Google.");
        println!("Una vez completado, las credenciales se guardarán automáticamente.");
        println!();

        // We're already inside #[tokio::main] — just await directly
        let auth_result = {
            let auth = BrowserAuthenticator::new();
            auth.authenticate().await
        };

        match auth_result {
            AuthResult::Success(creds) => {
                println!("¡Autenticación exitosa!");
                println!("Cookie extraída: {} bytes", creds.cookie.len());
                println!(
                    "CSRF: {} chars, Session ID (f.sid): {} chars",
                    creds.csrf.len(),
                    creds.sid.len()
                );

                // If tokens weren't extracted during auth, try HTTP fallback.
                let (csrf, sid) = if creds.csrf.is_empty() {
                    match crate::auth_helper::AuthHelper::new()
                        .refresh_tokens(&creds.cookie)
                        .await
                    {
                        Ok((token, session_id)) => {
                            println!(
                                "Tokens extraídos via HTTP: csrf={} chars, sid={} chars",
                                token.len(),
                                session_id.len()
                            );
                            (token, session_id)
                        }
                        Err(e) => {
                            println!("Advertencia: No se pudo extraer tokens: {}", e);
                            println!(
                                "Se extraerán automáticamente en la primera llamada a la API."
                            );
                            (String::new(), String::new())
                        }
                    }
                } else {
                    (creds.csrf, creds.sid)
                };

                // Guardar en keyring o fallback a DPAPI
                let final_creds = BrowserCredentials {
                    cookie: creds.cookie,
                    csrf,
                    sid,
                };

                // Always save to DPAPI as reliable backup (keyring may silently fail on some Windows configs)
                let session = SessionData {
                    cookie: final_creds.cookie.clone(),
                    csrf: final_creds.csrf.clone(),
                    sid: final_creds.sid.clone(),
                };
                if let Err(e) = save_session(&session) {
                    error!("Error guardando credenciales DPAPI: {}", e);
                } else {
                    println!("Credenciales guardadas con DPAPI.");
                }

                // Also try keyring (may or may not work depending on Windows config)
                if let Err(e) =
                    crate::auth_browser::BrowserAuthenticator::store_in_keyring(&final_creds)
                {
                    println!("Advertencia: No se pudo guardar en keyring: {}", e);
                } else {
                    println!("Credenciales guardadas en keyring.");
                }

                println!("\n¡Listo! Ya podés usar el servidor MCP.");
            }
            AuthResult::FallbackRequired(msg) => {
                println!("Navegador no disponible: {}", msg);
                println!("Por favor, usá el método manual:");
                println!("  notebooklm-mcp auth --cookie TU_COOKIE --csrf TU_CSRF");
            }
            AuthResult::Failed(msg) => {
                error!("Error de autenticación: {}", msg);
            }
        }
        return Ok(());
    }

    // --- Comando AuthStatus: verificar estado de autenticación ---
    if let Some(Commands::AuthStatus) = &cli.command {
        use crate::auth_browser::get_auth_status;

        let status = get_auth_status();

        println!("=== ESTADO DE AUTENTICACIÓN ===");
        println!(
            "Chrome disponible: {}",
            if status.chrome_available { "Sí" } else { "No" }
        );
        println!(
            "Credenciales almacenadas: {}",
            if status.has_stored_credentials {
                "Sí"
            } else {
                "No"
            }
        );
        println!();

        if !status.chrome_available {
            println!("Consejo: Instalá Chrome para usar autenticación automática.");
        }

        return Ok(());
    }

    // --- Cargar sesión encriptada ---
    // Try keyring first (browser auth), then fallback to DPAPI session file
    let (cookie, csrf, sid) = if let Some((c, cs, s)) = crate::auth_browser::load_credentials() {
        info!("Credenciales cargadas desde keyring");
        (c, cs, s)
    } else {
        match load_session() {
            Ok(s) => {
                info!("Credenciales cargadas desde DPAPI session file");
                (s.cookie, s.csrf, s.sid)
            }
            Err(e) => {
                error!(
                    "No se pudieron cargar las credenciales: {}. Ejecuta 'notebooklm-mcp auth-browser' o 'auth --cookie ... --csrf ...'",
                    e
                );
                if matches!(cli.command, Some(Commands::Verify)) {
                    return Ok(());
                }
                (String::new(), String::new(), String::new())
            }
        }
    };

    // If session ID (f.sid) is missing, refresh via HTTP.
    // Old sessions saved before f.sid was added won't have it.
    // A single GET to notebooklm.google.com returns both CSRF and FdrFJe.
    let (cookie, csrf, sid) = if !cookie.is_empty() && sid.is_empty() {
        info!("Session ID (f.sid) ausente — refrescando tokens via HTTP...");
        match crate::auth_helper::AuthHelper::new()
            .refresh_tokens(&cookie)
            .await
        {
            Ok((new_csrf, new_sid)) => {
                info!(
                    "Tokens refrescados: csrf={} chars, sid={} chars",
                    new_csrf.len(),
                    new_sid.len()
                );
                // Update DPAPI session with new tokens
                let session = SessionData {
                    cookie: cookie.clone(),
                    csrf: if new_csrf.is_empty() {
                        csrf.clone()
                    } else {
                        new_csrf
                    },
                    sid: new_sid,
                };
                let _ = save_session(&session); // best-effort update
                (session.cookie, session.csrf, session.sid)
            }
            Err(e) => {
                warn!("No se pudieron refrescar tokens: {}", e);
                (cookie, csrf, sid)
            }
        }
    } else {
        (cookie, csrf, sid)
    };

    // Ejecución del contrato de validación (E2E Test)
    if let Some(Commands::Verify) = cli.command {
        info!("Iniciando contrato de validación E2E contra NotebookLM...");
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        info!("0. Listando libretas existentes...");
        match client.list_notebooks().await {
            Ok(nbs) => info!("Libretas encontradas: {:?}", nbs),
            Err(e) => error!("Fallo al listar libretas: {}", e),
        }

        info!("1. Creando libreta de prueba...");
        match client.create_notebook("NotebookLM MCP Test").await {
            Ok(notebook_id) => {
                info!("¡Libreta creada con éxito! ID: {}", notebook_id);

                // info!("2. Añadiendo fuente de texto...");
                // match client.add_source(&notebook_id, "Origen de Prueba", "Este texto confirma que la API MCP funciona.").await {
                //     Ok(source_id) => {
                //         info!("¡Fuente añadida con éxito! ID: {}", source_id);
                //     },
                //     Err(e) => error!("Fallo al añadir fuente: {}", e)
                // }
            }
            Err(e) => {
                error!("Fallo de integración al crear la libreta: {}", e);
            }
        }
        info!("Prueba de validación del contrato finalizada.");
        return Ok(());
    }

    // Comando AddSource - añadir una fuente a una libreta
    if let Some(Commands::AddSource {
        notebook_id,
        title,
        content,
    }) = &cli.command
    {
        info!("Añadiendo fuente a la libreta {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.add_source(notebook_id, title, content).await {
            Ok(source_id) => {
                println!("\n=== FUENTE AÑADIDA ===\nSource ID: {}", source_id);
            }
            Err(e) => error!("Error al añadir fuente: {}", e),
        }
        return Ok(());
    }

    // Comando AddUrl - añadir una fuente URL a una libreta
    if let Some(Commands::AddUrl { notebook_id, url }) = &cli.command {
        info!("Añadiendo fuente URL a la libreta {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.add_url_source(notebook_id, url).await {
            Ok(source_id) => {
                println!("\n=== FUENTE URL AÑADIDA ===\nSource ID: {}", source_id);
            }
            Err(e) => error!("Error al añadir fuente URL: {}", e),
        }
        return Ok(());
    }

    // Comando AddDrive - añadir un documento de Google Drive como fuente
    if let Some(Commands::AddDrive {
        notebook_id,
        file_id,
        title,
        mime_type,
    }) = &cli.command
    {
        info!("Añadiendo fuente Drive a la libreta {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        let mime = mime_type.as_deref().unwrap_or("");
        match client
            .add_drive_source(notebook_id, file_id, title, mime)
            .await
        {
            Ok(source_id) => {
                println!("\n=== FUENTE DRIVE AÑADIDA ===\nSource ID: {}", source_id);
            }
            Err(e) => error!("Error al añadir fuente Drive: {}", e),
        }
        return Ok(());
    }

    // Comando AddFile - subir un archivo como fuente
    if let Some(Commands::AddFile {
        notebook_id,
        file_path,
    }) = &cli.command
    {
        info!("Subiendo archivo a la libreta {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.add_file_source(notebook_id, file_path).await {
            Ok(source_id) => {
                println!("\n=== FUENTE ARCHIVO AÑADIDA ===\nSource ID: {}", source_id);
            }
            Err(e) => error!("Error al subir archivo: {}", e),
        }
        return Ok(());
    }

    // Comando Ask - hacer una pregunta a una libreta
    if let Some(Commands::Ask {
        notebook_id,
        question,
    }) = &cli.command
    {
        info!("Preguntando a la libreta {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.ask_question(notebook_id, question).await {
            Ok(answer) => {
                println!("\n=== RESPUESTA ===\n{}", answer);
            }
            Err(e) => error!("Error al hacer pregunta: {}", e),
        }
        return Ok(());
    }

    // --- Artifact CLI Commands ---

    // Comando ArtifactList
    if let Some(Commands::ArtifactList { notebook_id, kind }) = &cli.command {
        info!("Listando artefactos de la libreta {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        let kind_filter = kind
            .as_deref()
            .and_then(notebooklm_client::ArtifactType::from_str_key);
        match client.list_artifacts(notebook_id, kind_filter).await {
            Ok(artifacts) => {
                if artifacts.is_empty() {
                    println!("\nNo artifacts found in notebook {}", notebook_id);
                } else {
                    println!("\n=== ARTIFACTOS ({} total) ===", artifacts.len());
                    for a in &artifacts {
                        println!(
                            "  [{}] {} (status: {}, id: {})",
                            a.kind, a.title, a.status, a.id
                        );
                    }
                }
            }
            Err(e) => error!("Error listando artefactos: {}", e),
        }
        return Ok(());
    }

    // Comando ArtifactGenerate
    if let Some(Commands::ArtifactGenerate {
        notebook_id,
        kind,
        instructions,
        language,
        audio_format,
        audio_length,
        video_format,
        video_style,
        quiz_difficulty,
        quiz_quantity,
        report_format,
        custom_prompt,
        ..
    }) = &cli.command
    {
        info!(
            "Generando artefacto {} en la libreta {}...",
            kind, notebook_id
        );
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        let req = ArtifactGenerateRequest {
            notebook_id: notebook_id.clone(),
            kind: kind.clone(),
            instructions: instructions.clone(),
            language: Some(language.clone()),
            audio_format: audio_format.clone(),
            audio_length: audio_length.clone(),
            video_format: video_format.clone(),
            video_style: video_style.clone(),
            quiz_difficulty: quiz_difficulty.clone(),
            quiz_quantity: quiz_quantity.clone(),
            report_format: report_format.clone(),
            infographic_orientation: None,
            infographic_detail: None,
            slide_deck_format: None,
            slide_deck_length: None,
            custom_prompt: custom_prompt.clone(),
        };

        if kind == "mind_map" {
            match client.generate_mind_map(notebook_id, &[]).await {
                Ok(result) => {
                    let note_id = result.note_id.as_deref().unwrap_or("unknown");
                    println!("\n=== MIND MAP GENERADO ===\nNote ID: {}", note_id);
                }
                Err(e) => error!("Error generando mind map: {}", e),
            }
        } else {
            match build_artifact_config(kind, &req, language) {
                Ok(config) => match client.generate_artifact(notebook_id, &config).await {
                    Ok(status) => {
                        println!("\n=== GENERACIÓN INICIADA ===");
                        println!("Task ID: {}", status.task_id);
                        println!("Status: {}", status.status);
                        if status.is_rate_limited() {
                            println!("⚠ Rate limited — reintentá después de un delay.");
                        }
                    }
                    Err(e) => error!("Error generando artefacto: {}", e),
                },
                Err(e) => error!("Error: {}", e),
            }
        }
        return Ok(());
    }

    // Comando ArtifactDownload
    if let Some(Commands::ArtifactDownload {
        notebook_id,
        artifact_id,
        output_path,
        format,
    }) = &cli.command
    {
        info!("Descargando artefacto {}...", artifact_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client
            .download_artifact(notebook_id, artifact_id, output_path, format.as_deref())
            .await
        {
            Ok(path) => println!("\n✅ Artefacto descargado a: {}", path),
            Err(e) => error!("Error descargando artefacto: {}", e),
        }
        return Ok(());
    }

    // Comando ArtifactDelete
    if let Some(Commands::ArtifactDelete {
        notebook_id,
        artifact_id,
    }) = &cli.command
    {
        info!("Eliminando artefacto {}...", artifact_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.delete_artifact(notebook_id, artifact_id).await {
            Ok(()) => println!("\n✅ Artefacto {} eliminado.", artifact_id),
            Err(e) => error!("Error eliminando artefacto: {}", e),
        }
        return Ok(());
    }

    // --- Notebook Lifecycle & Sharing CLI Commands ---

    if let Some(Commands::Delete { notebook_id }) = &cli.command {
        info!("Eliminando notebook {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.delete_notebook(notebook_id).await {
            Ok(()) => println!("\n✅ Notebook {} eliminado.", notebook_id),
            Err(e) => error!("Error eliminando notebook: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::Get { notebook_id }) = &cli.command {
        info!("Obteniendo detalles del notebook {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.get_notebook(notebook_id).await {
            Ok(nb) => println!(
                "\n📝 Notebook: \"{}\" (id: {})\n   Sources: {}\n   Owner: {}\n   Created: {}",
                nb.title,
                nb.id,
                nb.sources_count,
                if nb.is_owner { "yes" } else { "no" },
                nb.created_at.as_deref().unwrap_or("unknown")
            ),
            Err(e) => error!("Error obteniendo notebook: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::Rename { notebook_id, title }) = &cli.command {
        info!("Renombrando notebook {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.rename_notebook(notebook_id, title).await {
            Ok(nb) => println!(
                "\n✅ Notebook renombrado a \"{}\" (id: {})\n   Sources: {}\n   Owner: {}",
                nb.title,
                nb.id,
                nb.sources_count,
                if nb.is_owner { "yes" } else { "no" }
            ),
            Err(e) => error!("Error renombrando notebook: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::Summary { notebook_id }) = &cli.command {
        info!("Obteniendo resumen del notebook {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.get_summary(notebook_id).await {
            Ok(s) => {
                println!("\n📋 Summary:\n{}", s.summary);
                if !s.suggested_topics.is_empty() {
                    println!("\n💡 Suggested Topics ({}):", s.suggested_topics.len());
                    for (i, t) in s.suggested_topics.iter().enumerate() {
                        println!("  {}. {}\n     Prompt: {}", i + 1, t.question, t.prompt);
                    }
                }
            }
            Err(e) => error!("Error obteniendo resumen: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::ShareStatus { notebook_id }) = &cli.command {
        info!(
            "Obteniendo estado de compartido del notebook {}...",
            notebook_id
        );
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.get_share_status(notebook_id).await {
            Ok(s) => {
                println!(
                    "\n🔗 Sharing status for notebook {}:\n   Public: {}\n   Access: {:?}",
                    notebook_id, s.is_public, s.access
                );
                if let Some(url) = &s.share_url {
                    println!("   Share URL: {}", url);
                }
                if !s.shared_users.is_empty() {
                    println!("\n👥 Shared Users ({}):", s.shared_users.len());
                    for u in &s.shared_users {
                        let name = u
                            .display_name
                            .as_ref()
                            .map(|n| format!(" — {}", n))
                            .unwrap_or_default();
                        println!("  - {} ({:?}){}", u.email, u.permission, name);
                    }
                }
            }
            Err(e) => error!("Error obteniendo estado de compartido: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::ShareSet {
        notebook_id,
        public,
        private: _,
    }) = &cli.command
    {
        let make_public = *public;
        info!(
            "Setting notebook {} to {}...",
            notebook_id,
            if make_public { "public" } else { "private" }
        );
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.set_sharing_public(notebook_id, make_public).await {
            Ok(s) => {
                println!(
                    "\n✅ Notebook {} is now {}\n   Access: {:?}",
                    notebook_id,
                    if s.is_public { "PUBLIC" } else { "PRIVATE" },
                    s.access
                );
                if let Some(url) = &s.share_url {
                    println!("   Share URL: {}", url);
                }
            }
            Err(e) => error!("Error configurando compartido: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::SourceDelete {
        notebook_id,
        source_id,
    }) = &cli.command
    {
        info!(
            "Eliminando fuente {} del notebook {}...",
            source_id, notebook_id
        );
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.delete_source(notebook_id, source_id).await {
            Ok(()) => println!(
                "\n✅ Fuente {} eliminada del cuaderno {}",
                source_id, notebook_id
            ),
            Err(e) => error!("Error eliminando fuente: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::SourceRename {
        notebook_id,
        source_id,
        new_title,
    }) = &cli.command
    {
        info!(
            "Renombrando fuente {} a '{}' en notebook {}...",
            source_id, new_title, notebook_id
        );
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client
            .rename_source(notebook_id, source_id, new_title)
            .await
        {
            Ok(()) => println!(
                "\n✅ Fuente {} renombrada a '{}' en cuaderno {}",
                source_id, new_title, notebook_id
            ),
            Err(e) => error!("Error renombrando fuente: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::SourceGetFulltext {
        notebook_id,
        source_id,
    }) = &cli.command
    {
        info!(
            "Extrayendo texto completo de fuente {} en notebook {}...",
            source_id, notebook_id
        );
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.get_source_fulltext(notebook_id, source_id).await {
            Ok(text) => println!("\n📄 Fulltext de fuente {}:\n{}", source_id, text),
            Err(e) => error!("Error extrayendo texto: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::NoteCreate {
        notebook_id,
        title,
        content,
    }) = &cli.command
    {
        info!("Creando nota en notebook {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.create_note(notebook_id, title, content).await {
            Ok(id) => println!("\n✅ Nota creada. ID: {}", id),
            Err(e) => error!("Error creando nota: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::NoteList { notebook_id }) = &cli.command {
        info!("Listando notas del notebook {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.list_notes(notebook_id).await {
            Ok(notes) => {
                if notes.is_empty() {
                    println!("\nNo hay notas en este cuaderno.");
                } else {
                    println!("\n📝 Notas ({}):", notes.len());
                    for n in &notes {
                        println!(
                            "  - [{}] {}{}",
                            n.id,
                            n.title,
                            if n.content.is_empty() {
                                String::new()
                            } else {
                                format!(": {}", &n.content[..n.content.len().min(80)])
                            }
                        );
                    }
                }
            }
            Err(e) => error!("Error listando notas: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::NoteDelete {
        notebook_id,
        note_id,
    }) = &cli.command
    {
        info!(
            "Eliminando nota {} del notebook {}...",
            note_id, notebook_id
        );
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());

        match client.delete_note(notebook_id, note_id).await {
            Ok(()) => println!(
                "\n✅ Nota {} eliminada del cuaderno {}",
                note_id, notebook_id
            ),
            Err(e) => error!("Error eliminando nota: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::ChatHistory { notebook_id, limit }) = &cli.command {
        info!(
            "Obteniendo historial de chat del notebook {}...",
            notebook_id
        );
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());
        let limit_val = limit.unwrap_or(20);

        match client.get_last_conversation_id(notebook_id).await {
            Ok(Some(conv_id)) => {
                match client
                    .get_conversation_turns(notebook_id, &conv_id, limit_val)
                    .await
                {
                    Ok(turns) => {
                        if turns.is_empty() {
                            println!("\nNo conversation history found (conv_id: {})", conv_id);
                        } else {
                            println!(
                                "\n=== CHAT HISTORY ({} turns, conv_id: {}) ===",
                                turns.len(),
                                conv_id
                            );
                            for t in &turns {
                                println!("\n[{}]", t.role);
                                println!("{}", t.text);
                            }
                        }
                    }
                    Err(e) => error!("Error obteniendo turns: {}", e),
                }
            }
            Ok(None) => println!(
                "\nNo se encontró conversación para el notebook {}",
                notebook_id
            ),
            Err(e) => error!("Error obteniendo conversation ID: {}", e),
        }
        return Ok(());
    }

    if let Some(Commands::Research {
        notebook_id,
        query,
        timeout_secs,
    }) = &cli.command
    {
        info!("Iniciando deep research en notebook {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone(), sid.clone());
        let timeout_val = timeout_secs.unwrap_or(300);

        match client.start_deep_research(notebook_id, query).await {
            Ok(task_id) => {
                println!("\n=== DEEP RESEARCH INICIADO ===");
                println!("Task ID: {}", task_id);
                println!("Timeout: {}s", timeout_val);

                let start = std::time::Instant::now();
                loop {
                    if start.elapsed().as_secs() >= timeout_val {
                        println!(
                            "\n⏰ Timeout después de {}s. Task ID: {}",
                            timeout_val, task_id
                        );
                        break;
                    }

                    match client.poll_research_status(notebook_id, &task_id).await {
                        Ok(status) if status.is_complete => {
                            match client
                                .import_research_sources(
                                    notebook_id,
                                    &task_id,
                                    serde_json::json!([]),
                                )
                                .await
                            {
                                Ok(()) => {
                                    println!("\n✅ Deep research completado. Fuentes importadas.")
                                }
                                Err(e) => error!(
                                    "Research completado pero error importando fuentes: {}",
                                    e
                                ),
                            }
                            break;
                        }
                        Ok(status) => {
                            info!(
                                "Research status: code={} — esperando...",
                                status.status_code
                            );
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        }
                        Err(e) => {
                            error!("Error consultando status: {}", e);
                            break;
                        }
                    }
                }
            }
            Err(e) => error!("Error iniciando deep research: {}", e),
        }
        return Ok(());
    }

    // Comando UpdateCheck — verificar actualizaciones
    if let Some(Commands::UpdateCheck) = &cli.command {
        let current = env!("CARGO_PKG_VERSION");
        info!("Verificando actualizaciones (current: v{})...", current);

        match crate::update_checker::check_for_updates_async(current).await {
            Ok(result) => {
                println!("{}", result);
            }
            Err(e) => {
                println!("Error checking for updates: {}", e);
                println!("Verify your internet connection and try again.");
            }
        }
        return Ok(());
    }

    // We'll just use the notebooklm_client's add_source method
    info!("Iniciando NotebookLM Unofficial MCP Server...");

    let client = if !cookie.is_empty() && !csrf.is_empty() {
        Some(NotebookLmClient::new(cookie, csrf, sid))
    } else {
        None
    };

    let server = NotebookLmServer {
        state: Arc::new(RwLock::new(client)),
        tool_router: NotebookLmServer::tool_router(),
    };

    info!("Conectando RMCP en stdio...");
    let transport = rmcp::transport::io::stdio();

    if let Err(e) = server.serve(transport).await?.waiting().await {
        error!("Error en servidor RMCP: {}", e);
    }

    info!("Servidor detenido.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_req(kind: &str) -> ArtifactGenerateRequest {
        ArtifactGenerateRequest {
            notebook_id: "nb-1".to_string(),
            kind: kind.to_string(),
            instructions: None,
            language: None,
            audio_format: None,
            audio_length: None,
            video_format: None,
            video_style: None,
            quiz_difficulty: None,
            quiz_quantity: None,
            report_format: None,
            infographic_orientation: None,
            infographic_detail: None,
            slide_deck_format: None,
            slide_deck_length: None,
            custom_prompt: None,
        }
    }

    #[test]
    fn test_build_audio_config_defaults() {
        let req = empty_req("audio");
        let cfg = build_artifact_config("audio", &req, "en").unwrap();
        match cfg {
            ArtifactConfig::Audio {
                format,
                length,
                language,
                ..
            } => {
                assert_eq!(format, AudioFormat::DeepDive);
                assert_eq!(length, AudioLength::Default);
                assert_eq!(language, "en");
            }
            _ => panic!("Expected Audio variant"),
        }
    }

    #[test]
    fn test_build_video_config_defaults() {
        let req = empty_req("video");
        let cfg = build_artifact_config("video", &req, "es").unwrap();
        match cfg {
            ArtifactConfig::Video {
                format,
                style,
                language,
                ..
            } => {
                assert_eq!(format, VideoFormat::Explainer);
                assert!(style.is_none());
                assert_eq!(language, "es");
            }
            _ => panic!("Expected Video variant"),
        }
    }

    #[test]
    fn test_build_video_config_cinematic() {
        let mut req = empty_req("video");
        req.video_format = Some("cinematic".to_string());
        let cfg = build_artifact_config("video", &req, "en").unwrap();
        match cfg {
            ArtifactConfig::Video { format, style, .. } => {
                assert_eq!(format, VideoFormat::Cinematic);
                assert!(style.is_none());
            }
            _ => panic!("Expected Video variant"),
        }
    }

    #[test]
    fn test_build_report_config_defaults() {
        let req = empty_req("report");
        let cfg = build_artifact_config("report", &req, "en").unwrap();
        match cfg {
            ArtifactConfig::Report { format, .. } => {
                assert_eq!(format, ReportFormat::BriefingDoc);
            }
            _ => panic!("Expected Report variant"),
        }
    }

    #[test]
    fn test_build_report_config_custom_without_prompt() {
        let mut req = empty_req("report");
        req.report_format = Some("custom".to_string());
        let result = build_artifact_config("report", &req, "en");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("custom_prompt"));
    }

    #[test]
    fn test_build_report_config_custom_with_prompt() {
        let mut req = empty_req("report");
        req.report_format = Some("custom".to_string());
        req.custom_prompt = Some("Create a white paper".to_string());
        let cfg = build_artifact_config("report", &req, "en").unwrap();
        match cfg {
            ArtifactConfig::Report { format, .. } => match format {
                ReportFormat::Custom { prompt } => {
                    assert_eq!(prompt, "Create a white paper");
                }
                _ => panic!("Expected Custom variant"),
            },
            _ => panic!("Expected Report variant"),
        }
    }

    #[test]
    fn test_build_quiz_config_defaults() {
        let req = empty_req("quiz");
        let cfg = build_artifact_config("quiz", &req, "en").unwrap();
        match cfg {
            ArtifactConfig::Quiz {
                difficulty,
                quantity,
                ..
            } => {
                assert_eq!(difficulty, QuizDifficulty::Medium);
                assert_eq!(quantity, QuizQuantity::Standard);
            }
            _ => panic!("Expected Quiz variant"),
        }
    }

    #[test]
    fn test_build_infographic_config_defaults() {
        let req = empty_req("infographic");
        let cfg = build_artifact_config("infographic", &req, "en").unwrap();
        match cfg {
            ArtifactConfig::Infographic {
                orientation,
                detail,
                style,
                ..
            } => {
                assert_eq!(orientation, InfographicOrientation::Landscape);
                assert_eq!(detail, InfographicDetail::Standard);
                assert_eq!(style, InfographicStyle::AutoSelect);
            }
            _ => panic!("Expected Infographic variant"),
        }
    }

    #[test]
    fn test_build_data_table_config() {
        let mut req = empty_req("data_table");
        req.instructions = Some("comparison table".to_string());
        let cfg = build_artifact_config("data_table", &req, "en").unwrap();
        match cfg {
            ArtifactConfig::DataTable {
                instructions,
                language,
                ..
            } => {
                assert_eq!(instructions, "comparison table");
                assert_eq!(language, "en");
            }
            _ => panic!("Expected DataTable variant"),
        }
    }

    #[test]
    fn test_build_unknown_kind() {
        let req = empty_req("invalid_type");
        let result = build_artifact_config("invalid_type", &req, "en");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unknown artifact kind"));
    }

    #[test]
    fn test_build_config_with_instructions() {
        let mut req = empty_req("audio");
        req.instructions = Some("Focus on chapter 3".to_string());
        req.language = Some("es".to_string());
        let cfg = build_artifact_config("audio", &req, "es").unwrap();
        match cfg {
            ArtifactConfig::Audio {
                instructions,
                language,
                ..
            } => {
                assert_eq!(instructions.as_deref(), Some("Focus on chapter 3"));
                assert_eq!(language, "es");
            }
            _ => panic!("Expected Audio variant"),
        }
    }

    #[test]
    fn test_build_slide_deck_config_defaults() {
        let req = empty_req("slide_deck");
        let cfg = build_artifact_config("slide_deck", &req, "en").unwrap();
        match cfg {
            ArtifactConfig::SlideDeck { format, length, .. } => {
                assert_eq!(format, SlideDeckFormat::DetailedDeck);
                assert_eq!(length, SlideDeckLength::Default);
            }
            _ => panic!("Expected SlideDeck variant"),
        }
    }
}
