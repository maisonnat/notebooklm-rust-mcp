use clap::{Parser, Subcommand};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::RwLock;
use tracing::{info, error, Level};
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
}

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    tool, tool_handler, tool_router,
    ServerHandler, ServiceExt,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

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

pub mod notebooklm_client;
pub mod parser;
pub mod errors;
pub mod source_poller;
pub mod auth_helper;
pub mod conversation_cache;
pub mod auth_browser;
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
    #[tool(name = "notebook_list", description = "List all notebooks available in the account")]
    pub async fn notebook_list(&self) -> String {
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c.list_notebooks().await {
                Ok(ls) => format!("Notebooks: {:?}", ls),
                Err(e) => format!("Error listando cuadernos: {}", e)
            }
        } else {
            "Error: Servidor no autenticado".into()
        }
    }

    #[tool(name = "notebook_create", description = "Create a new notebook by title")]
    pub async fn notebook_create(&self, req: Parameters<NotebookCreateRequest>) -> String {
        let title_str = &req.0.title; 
        let lock = self.state.read().await;
        if let Some(c) = &*lock {
            match c.create_notebook(title_str).await {
                Ok(id) => format!("Cuaderno creado. ID: {}", id),
                Err(e) => format!("Error creando cuaderno: {}", e)
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
            match c.add_source(&request.notebook_id, &request.title, &request.content).await {
                Ok(id) => format!("Fuente añadida. ID: {}", id),
                Err(e) => format!("Error añadiendo fuente: {}", e)
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
            match c.ask_question(&request.notebook_id, &request.question).await {
                Ok(ans) => ans,
                Err(e) => format!("Error haciendo pregunta: {}", e)
            }
        } else {
            "Error: Servidor no autenticado".into()
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

    fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ListResourcesResult, rmcp::ErrorData>> + Send + '_ {
        async {
            let lock = self.state.read().await;
            if let Some(client) = &*lock {
                match client.list_notebooks().await {
                    Ok(notebooks) => {
                        let resources: Vec<rmcp::model::Resource> = notebooks.iter().map(|nb| {
                            let uri = format!("notebook://{}", nb.id);
                            let mut raw = rmcp::model::RawResource::new(&uri, &nb.title);
                            raw.description = Some(format!("NotebookLM notebook: {}", nb.title));
                            raw.mime_type = Some("application/json".to_string());
                            rmcp::model::Resource { raw, annotations: None }
                        }).collect();
                        Ok(rmcp::model::ListResourcesResult::with_all_items(resources))
                    },
                    Err(e) => Err(rmcp::ErrorData::internal_error(e, None)),
                }
            } else {
                Err(rmcp::ErrorData::internal_error("Servidor no autenticado", None))
            }
        }
    }

    fn read_resource(
        &self,
        request: rmcp::model::ReadResourceRequestParams,
        _context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> impl std::future::Future<Output = Result<rmcp::model::ReadResourceResult, rmcp::ErrorData>> + Send + '_ {
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
                                    )
                                ]))
                            } else {
                                Err(rmcp::ErrorData::resource_not_found(
                                    format!("Notebook {} no encontrado", notebook_id), None
                                ))
                            }
                        },
                        Err(e) => Err(rmcp::ErrorData::internal_error(e, None)),
                    }
                } else {
                    Err(rmcp::ErrorData::internal_error("Servidor no autenticado", None))
                }
            } else {
                Err(rmcp::ErrorData::invalid_params(
                    format!("URI no soportada: {}. Use notebook://<uuid>", uri), None
                ))
            }
        }
    }
}

// --- DPAPI Session Management ---

#[derive(Serialize, Deserialize)]
struct SessionData {
    cookie: String,
    csrf: String,
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
        let session = SessionData { cookie: cookie.clone(), csrf: csrf.clone() };
        save_session(&session)?;
        info!("Credenciales encriptadas con DPAPI y guardadas en {:?} ({} bytes de cookie).", session_path(), cookie.len());
        return Ok(());
    }

    // --- Comando AuthBrowser: autenticación vía Chrome headless ---
    if let Some(Commands::AuthBrowser) = &cli.command {
        use crate::auth_browser::{BrowserAuthenticator, AuthResult};
        
        println!("=== AUTENTICACIÓN POR NAVEGADOR ===");
        println!("Se abrirá una ventana del navegador Chrome.");
        println!("Por favor, inicia sesión en tu cuenta de Google.");
        println!("Una vez completado, las credenciales se guardarán automáticamente.");
        println!();
        
        // Run async auth in sync context
        let rt = tokio::runtime::Runtime::new()?;
        let auth_result = rt.block_on(async {
            let auth = BrowserAuthenticator::new();
            auth.authenticate().await
        });
        
        match auth_result {
            AuthResult::Success(creds) => {
                println!("¡Autenticación exitosa!");
                println!("Cookie extraída: {} bytes", creds.cookie.len());
                
                // Extraer CSRF usando auth_helper (sync wrapper needed)
                let csrf = match rt.block_on(crate::auth_helper::AuthHelper::new().refresh_csrf(&creds.cookie)) {
                    Ok(token) => {
                        println!("CSRF extraído: {} chars", token.len());
                        token
                    }
                    Err(e) => {
                        println!("Advertencia: No se pudo extraer CSRF automáticamente: {}", e);
                        String::new()
                    }
                };
                
                // Guardar en keyring o fallback a DPAPI
                if let Err(e) = crate::auth_browser::BrowserAuthenticator::store_in_keyring(&creds) {
                    println!("Advertencia: No se pudo guardar en keyring: {}", e);
                    // Fallback to DPAPI
                    let session = SessionData { cookie: creds.cookie, csrf: csrf.clone() };
                    if let Err(e2) = save_session(&session) {
                        error!("Error guardando credenciales: {}", e2);
                        return Ok(());
                    }
                    println!("Credenciales guardadas con DPAPI (fallback).");
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
        println!("Chrome disponible: {}", if status.chrome_available { "Sí" } else { "No" });
        println!("Credenciales almacenadas: {}", if status.has_stored_credentials { "Sí" } else { "No" });
        println!();
        
        if !status.chrome_available {
            println!("Consejo: Instalá Chrome para usar autenticación automática.");
        }
        
        return Ok(());
    }

    // --- Cargar sesión encriptada ---
    let (cookie, csrf) = match load_session() {
        Ok(s) => (s.cookie, s.csrf),
        Err(e) => {
            error!("No se pudieron cargar las credenciales: {}. Ejecuta 'auth --cookie ... --csrf ...'", e);
            if matches!(cli.command, Some(Commands::Verify)) {
                return Ok(());
            }
            (String::new(), String::new())
        }
    };

// Ejecución del contrato de validación (E2E Test)
    if let Some(Commands::Verify) = cli.command {
        info!("Iniciando contrato de validación E2E contra NotebookLM...");
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone());

        info!("0. Listando libretas existentes...");
        match client.list_notebooks().await {
            Ok(nbs) => info!("Libretas encontradas: {:?}", nbs),
            Err(e) => error!("Fallo al listar libretas: {}", e)
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
            },
            Err(e) => {
                error!("Fallo de integración al crear la libreta: {}", e);
            }
        }
        info!("Prueba de validación del contrato finalizada.");
        return Ok(());
    }

    // Comando AddSource - añadir una fuente a una libreta
    if let Some(Commands::AddSource { notebook_id, title, content }) = &cli.command {
        info!("Añadiendo fuente a la libreta {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone());
        
        match client.add_source(notebook_id, title, content).await {
            Ok(source_id) => {
                println!("\n=== FUENTE AÑADIDA ===\nSource ID: {}", source_id);
            },
            Err(e) => error!("Error al añadir fuente: {}", e)
        }
        return Ok(());
    }

    // Comando Ask - hacer una pregunta a una libreta
    if let Some(Commands::Ask { notebook_id, question }) = &cli.command {
        info!("Preguntando a la libreta {}...", notebook_id);
        let client = NotebookLmClient::new(cookie.clone(), csrf.clone());
        
        match client.ask_question(notebook_id, question).await {
            Ok(answer) => {
                println!("\n=== RESPUESTA ===\n{}", answer);
            },
            Err(e) => error!("Error al hacer pregunta: {}", e)
        }
        return Ok(());
    }

    // Comando AddSource - añadir una fuente a una libreta
    // We'll just use the notebooklm_client's add_source method
    info!("Iniciando NotebookLM Unofficial MCP Server...");

    let client = if !cookie.is_empty() && !csrf.is_empty() {
        Some(NotebookLmClient::new(cookie, csrf))
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

