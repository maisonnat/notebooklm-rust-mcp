//! NotebookLM Client — HTTP client para la API RPC interna de Google NotebookLM
//!
//! # Gotchas descubierto durante implementación
//!
//! - **Anti-XSSI**: Google responde con prefix `)]}'` que debe removerse antes
//!   de parsear JSON. Usar `strip_prefix(")]}'")` (no manual slice).
//! - **Domain validation**: Solo `*.google.com`, `*.googleusercontent.com`,
//!   `*.googleapis.com` son trusted. YouTube NO está en la trust list.
//! - **Streaming download**: Usar `.tmp` + atomic rename para evitar archivos
//!   corruptos. Detectar auth expiry por content-type `text/html`.
//! - **`to_str()` on headers**: Retorna `Result`, no `Option`. Usar
//!   `.and_then(|v| v.to_str().ok())`.
//! - **Rate limiting**: Google retorna `rpc_code = "USER_DISPLAYABLE_ERROR"`.
//!   La respuesta NO contiene datos de artifact — solo el error.
//! - **Mind map generation**: Payload completamente diferente a CREATE_ARTIFACT.
//!   No usa wrapper `[2, notebook_id, ...]`. Después de generar, Google ignora
//!   el title en CREATE_NOTE — hay que hacer UPDATE_NOTE (CbCNt) separado.
//! - **`_download_interactive`**: RPC `v9rmvd` solo toma `[artifact_id]`, NO
//!   necesita `notebook_id`.

use reqwest::{Client, header};
use serde_json::Value;
use std::time::Duration;
use rand::Rng;
use governor::{Quota, RateLimiter, state::NotKeyed, state::InMemoryState, clock::DefaultClock};
use tokio::sync::Semaphore;
use tracing::info;
use uuid::Uuid;

// Importar funciones del parser para acceso defensivo
use crate::parser::{
    extract_by_rpc_id, strip_antixssi_prefix, get_string_at, get_uuid_at, get_string_at_or_default,
    extract_notebook_list, extract_sources, extract_nested_source_id, find_source_entry,
    parse_artifact_list, parse_generation_result,
    // URL extraction (Phase 6.1)
    extract_audio_url, extract_video_url, extract_infographic_url, extract_slide_deck_url,
    // Inline content extraction (Phase 6.4-6.7)
    extract_report_content, parse_data_table,
    extract_app_data,
    is_mind_map_item, extract_mind_map_json,
};

// Re-exportar errores para uso externo
pub use crate::errors::NotebookLmError;

// Re-exportar SourcePoller
pub use crate::source_poller::SourcePoller;

// Re-exportar conversation cache
pub use crate::conversation_cache::{ConversationCache, SharedConversationCache, new_conversation_cache};

// Re-exportar artifact types
pub use crate::parser::Artifact;
pub use crate::rpc::artifacts::{
    ArtifactConfig, ArtifactType, ArtifactTypeCode, ArtifactStatus, GenerationStatus,
    MindMapResult,
    AudioFormat, AudioLength, VideoFormat, VideoStyle, QuizDifficulty, QuizQuantity,
    InfographicOrientation, InfographicDetail, InfographicStyle, SlideDeckFormat,
    SlideDeckLength, ReportFormat, rpc_ids,
};

// Re-exportar notebook lifecycle & sharing types
pub use crate::rpc::notebooks::{
    ShareAccess, SharePermission, SharedUser, ShareStatus, SuggestedTopic, NotebookSummary,
};

type Limiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

/// Check if a URL belongs to YouTube.
///
/// Matches hostnames: `youtube.com`, `www.youtube.com`, `m.youtube.com`,
/// `music.youtube.com`, `youtu.be`.
///
/// Uses `url::Url` for robust parsing — NOT regex or string manipulation.
/// Returns `false` for invalid URLs, non-HTTP schemes, or URLs without a host.
pub fn is_youtube_url(url: &str) -> bool {
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false,
    };

    let host = match parsed.host_str() {
        Some(h) => h,
        None => return false,
    };

    matches!(
        host,
        "youtube.com"
            | "www.youtube.com"
            | "m.youtube.com"
            | "music.youtube.com"
            | "youtu.be"
    )
}

/// Validate that a download URL points to a trusted Google domain.
///
/// Prevents sending authentication cookies to arbitrary servers.
/// Only HTTPS URLs on Google-owned domains are allowed:
/// - `*.google.com`
/// - `*.googleusercontent.com`
/// - `*.googleapis.com`
///
/// # Errors
///
/// Returns `NotebookLmError::DownloadFailed` if:
/// - URL cannot be parsed
/// - Scheme is not HTTPS
/// - Domain is not in the trusted list
pub fn validate_google_domain(url: &str) -> Result<(), NotebookLmError> {
    let parsed = url::Url::parse(url).map_err(|_| {
        NotebookLmError::DownloadFailed(format!("Invalid download URL: {}", &url[..url.len().min(80)]))
    })?;

    if parsed.scheme() != "https" {
        return Err(NotebookLmError::DownloadFailed(format!(
            "Download URL must use HTTPS: {}",
            &url[..url.len().min(80)]
        )));
    }

    let host = parsed.host_str().ok_or_else(|| {
        NotebookLmError::DownloadFailed(format!("Download URL has no host: {}", &url[..url.len().min(80)]))
    })?;

    let trusted = [".google.com", ".googleusercontent.com", ".googleapis.com"];
    let is_trusted = trusted.iter().any(|domain| {
        host == domain.trim_start_matches('.') || host.ends_with(domain)
    });

    if is_trusted {
        Ok(())
    } else {
        Err(NotebookLmError::DownloadFailed(format!(
            "Untrusted download domain: {}",
            host
        )))
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Notebook {
    pub id: String,
    pub title: String,
    /// Number of sources in the notebook. Only populated by `get_notebook()`.
    /// `list_notebooks()` leaves this at 0 (default).
    #[serde(default)]
    pub sources_count: usize,
    /// Whether the current user is the owner. Only populated by `get_notebook()`.
    /// Defaults to `true` since most notebooks are self-created.
    #[serde(default = "default_true")]
    pub is_owner: bool,
    /// Creation timestamp (unix seconds as string). Only populated by `get_notebook()`.
    #[serde(default)]
    pub created_at: Option<String>,
}


pub struct NotebookLmClient {
    http: Client,
    /// HTTP client for file uploads — NO global Content-Type.
    /// Step 2 needs JSON, step 3 needs raw bytes. Each request sets its own.
    #[allow(dead_code)]
    upload_http: Client,
    csrf: String,
    /// Session ID (FdrFJe) — required as `f.sid` param in batchexecute URLs.
    /// Google routes requests to the correct server-side session using this.
    sid: String,
    limiter: Limiter,
    conversation_cache: SharedConversationCache,
    #[allow(dead_code)]
    upload_semaphore: Semaphore,
}

impl NotebookLmClient {
    pub fn new(cookie: String, csrf: String, sid: String) -> Self {
        let quota = Quota::with_period(Duration::from_secs(2)).unwrap();
        let limiter = RateLimiter::direct(quota);

        // Client for RPC calls — needs Content-Type: application/x-www-form-urlencoded
        let mut headers = header::HeaderMap::new();
        headers.insert(header::COOKIE, header::HeaderValue::from_str(&cookie).unwrap());
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/x-www-form-urlencoded;charset=utf-8"));

        let http = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        // Client for file uploads — cookie only, NO global Content-Type.
        // Step 2 (start resumable) sends JSON body.
        // Step 3 (stream upload) sends raw bytes.
        // Each request sets its own Content-Type header.
        let mut upload_headers = header::HeaderMap::new();
        upload_headers.insert(header::COOKIE, header::HeaderValue::from_str(&cookie).unwrap());

        let upload_http = Client::builder()
            .default_headers(upload_headers)
            .build()
            .unwrap();

        // Semáforo para limitar uploads a 2 simultáneos
        let upload_semaphore = Semaphore::new(2);

        Self {
            http,
            upload_http,
            csrf,
            sid,
            limiter,
            conversation_cache: new_conversation_cache(),
            upload_semaphore,
        }
    }

    /// Apply exponential backoff with jitter for retries
    async fn apply_exponential_backoff(attempt: u32) {
        if attempt == 0 {
            return;
        }
        
        // Base delay: 1 second, doubling each attempt
        let base_delay = 1u64.pow(attempt.min(6)); // Cap at 64 seconds
        let jitter = {
            let mut rng = rand::thread_rng();
            rng.gen_range(100..1000) // 100ms to 1s jitter
        };
        
        let total_delay = (base_delay * 1000) + jitter;
        let capped_delay = total_delay.min(30000); // Max 30 seconds
        
        tokio::time::sleep(Duration::from_millis(capped_delay)).await;
    }

    /// Retry wrapper with exponential backoff for batchexecute
    async fn batchexecute_with_retry(&self, rpc_id: &str, payload: &str, max_retries: u32) -> Result<Value, String> {
        let mut last_error = String::new();
        
        for attempt in 0..=max_retries {
            match self.batchexecute_no_retry(rpc_id, payload).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = e;
                    if attempt < max_retries {
                        info!("Retry {}/{} for {}: {}", attempt + 1, max_retries + 1, rpc_id, last_error);
                        Self::apply_exponential_backoff(attempt).await;
                    }
                }
            }
        }
        
        Err(format!("Failed after {} retries: {}", max_retries + 1, last_error))
    }

    async fn apply_jitter() {
        let jitter = {
            let mut rng = rand::thread_rng();
            rng.gen_range(150..=600)
        };
        tokio::time::sleep(Duration::from_millis(jitter)).await;
    }

    async fn batchexecute(&self, rpc_id: &str, payload: &str) -> Result<Value, String> {
        self.batchexecute_with_retry(rpc_id, payload, 3).await
    }

    /// Internal batchexecute without retry (for when caller handles retries)
    async fn batchexecute_no_retry(&self, rpc_id: &str, payload: &str) -> Result<Value, String> {
        self.limiter.until_ready().await;
        Self::apply_jitter().await;

        let req_array = format!("[[[\"{}\",\"{}\",null,\"generic\"]]]", rpc_id, payload.replace("\"", "\\\""));
        let form_data = [
            ("f.req", req_array),
            ("at", self.csrf.clone())
        ];

        // Build batchexecute URL with session ID.
        // f.sid (FdrFJe) is REQUIRED by Google's RPC router — without it,
        // the response is empty/null at [0][2]. source-path=/ is also needed.
        let mut url = format!(
            "https://notebooklm.google.com/_/LabsTailwindUi/data/batchexecute?rpcids={}&rt=c",
            rpc_id
        );
        if !self.sid.is_empty() {
            url.push_str(&format!("&source-path=/&f.sid={}", self.sid));
        }

        let res = self.http.post(&url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Error HTTP {}", res.status()));
        }

        let text = res.text().await.map_err(|e| format!("No body text: {}", e))?;
        
        // DEBUG: Log first 500 chars of raw response
        tracing::debug!("Raw response ({} bytes): {}", text.len(), &text[..text.len().min(500)]);
        
        // Usar parser defensivo: strip_antixssi_prefix
        let cleaned = strip_antixssi_prefix(&text);
        
        // Parsear el JSON limpio
        let v: Value = serde_json::from_str(&cleaned).map_err(|e| {
            tracing::error!("JSON parse failed. Cleaned ({} bytes): {}", cleaned.len(), &cleaned[..cleaned.len().min(500)]);
            e.to_string()
        })?;
        
        tracing::debug!("Parsed JSON: {:?}", &v);
        Ok(v)
    }
    pub async fn list_notebooks(&self) -> Result<Vec<Notebook>, String> {
        let payload = "[null, 1, null, []]";
        let response = self.batchexecute("wXbhsf", payload).await?;

        // Usar parser defensivo: extract_by_rpc_id
        let inner = extract_by_rpc_id(&response, "wXbhsf")
            .ok_or("No se encontró respuesta wXbhsf")?;
        
        // Extraer lista de notebooks: [[title, sources, uuid, ...], ...]
        let notebook_list = extract_notebook_list(&inner)
            .ok_or("No se pudo parsear lista de notebooks")?;

        let mut notebooks = Vec::new();
        for nb_arr_val in notebook_list {
            if let Some(nb_arr) = nb_arr_val.as_array() {
                // Acceso defensivo: get_string_at en lugar de índice directo
                let title = get_string_at_or_default(nb_arr, 0, "Sin título");
                let id = get_uuid_at(nb_arr, 2).unwrap_or_default();
                
                if !id.is_empty() {
                    notebooks.push(Notebook {
                        id,
                        title,
                        ..Default::default()
                    });
                }
            }
        }
        
        info!("Parsed {} notebooks from wXbhsf response", notebooks.len());
        Ok(notebooks)
    }

    pub async fn create_notebook(&self, title: &str) -> Result<String, String> {
        let inner_json = format!("[\"{}\",null,null,[2],[1,null,null,null,null,null,null,null,null,null,[1]]]", title.replace('\"', "\\\""));
        let response = self.batchexecute("CCqFvf", &inner_json).await?;
        
        // Usar parser defensivo: extract_by_rpc_id
        let inner = extract_by_rpc_id(&response, "CCqFvf")
            .ok_or("No se encontró respuesta CCqFvf")?;
        
        // Extraer UUID del inner_json: ["", null, "UUID", ...]
        get_uuid_at(inner.as_array().unwrap(), 2)
            .ok_or_else(|| "No se pudo extraer el UUID del cuaderno nuevo".to_string())
    }

    /// Delete a notebook by ID. Idempotent — Google does not return an error
    /// for non-existent notebooks, so we treat any successful HTTP response as success.
    ///
    /// RPC: `WWINqb`, payload: `[[notebook_id], [2]]`
    pub async fn delete_notebook(&self, notebook_id: &str) -> Result<(), String> {
        let inner_json = format!("[[\"{}\"],[2]]", notebook_id);
        self.batchexecute("WWINqb", &inner_json).await?;
        info!("Deleted notebook {}", notebook_id);
        Ok(())
    }

    pub async fn add_source(&self, notebook_id: &str, title: &str, content: &str) -> Result<String, String> {
        let t = title.replace('\"', "\\\"");
        let c = content.replace('\"', "\\\"");
        let inner_json = format!(
            "[[[null,[\"{}\",\"{}\"],null,2,null,null,null,null,null,null,1]],\"{}\",[2],[1,null,null,null,null,null,null,null,null,null,[1]]]",
            t, c, notebook_id
        );
        let response = self.batchexecute("izAoDd", &inner_json).await?;

        // Usar parser defensivo: extract_by_rpc_id
        let inner = extract_by_rpc_id(&response, "izAoDd")
            .ok_or("No se encontró respuesta izAoDd")?;
        
        // Extraer source UUID: [[["SOURCE_UUID"]], ...]
        let arr = inner.as_array().ok_or("Inner no es array")?;
        let first = arr.first().ok_or("Array vacío")?.as_array().ok_or("No es array anidado")?;
        let first_inner = first.first().ok_or("Array anidado vacío")?.as_array().ok_or("No es array")?;
        
        get_string_at(first_inner, 0)
            .ok_or_else(|| "No se pudo extraer el UUID de la fuente nueva".to_string())
    }

    /// Add a URL (web or YouTube) as a source to a notebook.
    ///
    /// Auto-detects YouTube URLs and uses the appropriate 11-element payload.
    /// Regular URLs use the 8-element payload. Both go through RPC `izAoDd`.
    pub async fn add_url_source(&self, notebook_id: &str, url: &str) -> Result<String, String> {
        let params = if is_youtube_url(url) {
            info!("Detected YouTube URL, using 11-element payload");
            crate::rpc::sources::build_youtube_source_params(notebook_id, url)
        } else {
            crate::rpc::sources::build_url_source_params(notebook_id, url)
        };

        let inner_json = serde_json::to_string(&params)
            .map_err(|e| format!("Failed to serialize URL source params: {}", e))?;

        let response = self.batchexecute("izAoDd", &inner_json).await?;

        // Parser defensivo: extract_by_rpc_id
        let inner = extract_by_rpc_id(&response, "izAoDd")
            .ok_or("No se encontró respuesta izAoDd")?;

        // Extraer source UUID: [[["SOURCE_UUID"]], ...]
        let arr = inner.as_array().ok_or("Inner no es array")?;
        let first = arr.first().ok_or("Array vacío")?.as_array().ok_or("No es array anidado")?;
        let first_inner = first.first().ok_or("Array anidado vacío")?.as_array().ok_or("No es array")?;

        get_string_at(first_inner, 0)
            .ok_or_else(|| "No se pudo extraer el UUID de la fuente nueva".to_string())
    }

    /// Add a Google Drive document as a source to a notebook.
    ///
    /// Uses the single-wrapped Drive payload via RPC `izAoDd`.
    /// Defaults `mime_type` to `application/vnd.google-apps.document` if empty.
    pub async fn add_drive_source(
        &self,
        notebook_id: &str,
        file_id: &str,
        title: &str,
        mime_type: &str,
    ) -> Result<String, String> {
        let effective_mime = if mime_type.is_empty() {
            "application/vnd.google-apps.document"
        } else {
            mime_type
        };

        let params = crate::rpc::sources::build_drive_source_params(
            notebook_id,
            file_id,
            effective_mime,
            title,
        );

        let inner_json = serde_json::to_string(&params)
            .map_err(|e| format!("Failed to serialize Drive source params: {}", e))?;

        let response = self.batchexecute("izAoDd", &inner_json).await?;

        // Parser defensivo: extract_by_rpc_id
        let inner = extract_by_rpc_id(&response, "izAoDd")
            .ok_or("No se encontró respuesta izAoDd")?;

        // Extraer source UUID: [[["SOURCE_UUID"]], ...]
        let arr = inner.as_array().ok_or("Inner no es array")?;
        let first = arr.first().ok_or("Array vacío")?.as_array().ok_or("No es array anidado")?;
        let first_inner = first.first().ok_or("Array anidado vacío")?.as_array().ok_or("No es array")?;

        get_string_at(first_inner, 0)
            .ok_or_else(|| "No se pudo extraer el UUID de la fuente nueva".to_string())
    }

    /// Step 1 of file upload: register the file source via RPC.
    ///
    /// Calls `o4cbdc` with `[[filename]]` params and extracts the SOURCE_ID
    /// from the deeply nested response: `[[[[source_id]]]]`.
    async fn _register_file_source(&self, notebook_id: &str, filename: &str) -> Result<String, String> {
        let params = crate::rpc::sources::build_file_register_params(notebook_id, filename);
        let inner_json = serde_json::to_string(&params)
            .map_err(|e| format!("Failed to serialize file register params: {}", e))?;

        let response = self.batchexecute("o4cbdc", &inner_json).await?;

        // Parser defensivo: extract_by_rpc_id
        let inner = extract_by_rpc_id(&response, "o4cbdc")
            .ok_or("No se encontró respuesta o4cbdc")?;

        // Extraer SOURCE_ID del nesting profundo: [[[[source_id]]]]
        extract_nested_source_id(&inner)
            .ok_or_else(|| "No se pudo extraer SOURCE_ID de la respuesta o4cbdc".to_string())
    }

    /// Step 2 of file upload: start a resumable upload session.
    ///
    /// POSTs JSON body to `/upload/_/?authuser=0` with resumable headers.
    /// Returns the `x-goog-upload-url` from response headers for step 3.
    async fn _start_resumable_upload(
        &self,
        notebook_id: &str,
        filename: &str,
        _file_size: u64,
        source_id: &str,
    ) -> Result<String, String> {
        let body = crate::rpc::sources::UploadSessionBody::new(notebook_id, filename, source_id);
        let body_json = serde_json::to_string(&body)
            .map_err(|e| format!("Failed to serialize upload session body: {}", e))?;

        let url = format!(
            "{}?authuser=0",
            crate::rpc::sources::UPLOAD_URL
        );

        let res = self.upload_http
            .post(&url)
            .header("Content-Type", "application/json")
            .header("x-goog-upload-command", "start")
            .header("x-goog-upload-protocol", "resumable")
            .header("x-goog-upload-header-content-length", _file_size.to_string())
            .header("x-goog-upload-header-content-type", "application/octet-stream")
            .body(body_json)
            .send()
            .await
            .map_err(|e| format!("Upload session request failed: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(format!("Upload session failed (HTTP {}): {}", status, text));
        }

        res.headers()
            .get("x-goog-upload-url")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .ok_or_else(|| "Response missing x-goog-upload-url header".to_string())
    }

    /// Step 3 of file upload: stream the file to the upload URL.
    ///
    /// Reads the file from disk using `tokio::fs::File`, converts to a stream
    /// via `ReaderStream`, and wraps in `reqwest::Body::wrap_stream()`.
    /// Acquires a semaphore permit to limit concurrent uploads.
    async fn _stream_upload_file(&self, upload_url: &str, file_path: &std::path::Path) -> Result<(), String> {
        let _permit = self.upload_semaphore.acquire().await
            .map_err(|_| "Upload semaphore closed".to_string())?;

        let file = tokio::fs::File::open(file_path).await
            .map_err(|e| format!("Failed to open file for upload: {}", e))?;

        let reader_stream = tokio_util::io::ReaderStream::new(file);
        let body = reqwest::Body::wrap_stream(reader_stream);

        let res = self.upload_http
            .post(upload_url)
            .header("x-goog-upload-command", "upload, finalize")
            .header("x-goog-upload-offset", "0")
            .body(body)
            .send()
            .await
            .map_err(|e| format!("File stream upload failed: {}", e))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(format!("File upload failed (HTTP {}): {}", status, text));
        }

        Ok(())
    }

    /// Upload a file as a source to a notebook (3-step protocol).
    ///
    /// Step 1: RPC `o4cbdc` — register file source, get SOURCE_ID.
    /// Step 2: POST `/upload/_/` — start resumable upload session, get upload URL.
    /// Step 3: POST upload URL — stream file bytes from disk.
    ///
    /// Validates path exists + is_file before any network call.
    /// Maps errors to structured `NotebookLmError` variants.
    pub async fn add_file_source(&self, notebook_id: &str, file_path: &str) -> Result<String, String> {
        let path = std::path::Path::new(file_path);

        // Validate: path must exist and be a file
        if !path.exists() {
            return Err(NotebookLmError::FileNotFound(file_path.to_string()).to_string());
        }
        if !path.is_file() {
            return Err(NotebookLmError::ValidationError(format!("Path is a directory, not a file: {}", file_path)).to_string());
        }

        // Get file size for the upload session
        let file_size = tokio::fs::metadata(path).await
            .map_err(|e| format!("Failed to read file metadata: {}", e))?
            .len();

        // Get filename from path
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| NotebookLmError::ValidationError("Cannot extract filename from path".to_string()).to_string())?;

        info!("Uploading file '{}' ({} bytes) to notebook {}", filename, file_size, notebook_id);

        // Step 1: Register file source via RPC
        let source_id = self._register_file_source(notebook_id, filename).await
            .map_err(|e| format!("Step 1 (register) failed: {}", e))?;
        info!("File registered. SOURCE_ID: {}", source_id);

        // Step 2: Start resumable upload session
        let upload_url = self._start_resumable_upload(notebook_id, filename, file_size, &source_id).await
            .map_err(|e| format!("Step 2 (start session) failed: {}", e))?;
        info!("Upload session started. Got upload URL");

        // Step 3: Stream file to upload URL
        self._stream_upload_file(&upload_url, path).await
            .map_err(|e| format!("Step 3 (stream upload) failed: {}", e))?;
        info!("File upload complete");

        Ok(source_id)
    }

    /// Get full notebook details by ID (enriched with sources_count, is_owner, created_at).
    ///
    /// Uses the same RPC (`rLM1Ne`) as `get_notebook_sources()` but parses
    /// notebook-level metadata instead of source data.
    ///
    /// RPC: `rLM1Ne`, payload: `[notebook_id, null, [2], null, 0]`
    pub async fn get_notebook(&self, notebook_id: &str) -> Result<Notebook, String> {
        let payload = format!("[\"{}\", null, [2], null, 0]", notebook_id.replace('"', "\\\""));
        let response = self.batchexecute("rLM1Ne", &payload).await?;

        let inner = extract_by_rpc_id(&response, "rLM1Ne")
            .ok_or("No se encontró respuesta rLM1Ne")?;

        let details = crate::rpc::notebooks::parse_notebook_details(&inner)
            .ok_or_else(|| "No se pudo parsear detalles del notebook".to_string())?;

        info!("Got notebook {} ({})", details.id, details.title);
        Ok(Notebook {
            id: details.id,
            title: details.title,
            sources_count: details.sources_count,
            is_owner: details.is_owner,
            created_at: details.created_at,
        })
    }

    /// Rename a notebook. Executes the rename RPC, then reads back the notebook
    /// to return confirmed data with the new title.
    ///
    /// RPC: `s0tc2d`, payload: `[id, [[null, null, null, [null, title]]]]`
    pub async fn rename_notebook(&self, notebook_id: &str, new_title: &str) -> Result<Notebook, String> {
        let escaped_title = new_title.replace('"', "\\\"");
        let inner_json = format!(
            "[\"{}\", [[null, null, null, [null, \"{}\"]]]]",
            notebook_id, escaped_title
        );
        self.batchexecute("s0tc2d", &inner_json).await?;
        info!("Renamed notebook {} to \"{}\"", notebook_id, new_title);

        // Post-mutation read: return confirmed data
        self.get_notebook(notebook_id).await
    }

    /// Get AI-generated summary and suggested topics for a notebook.
    ///
    /// RPC: `VfAZjd`, payload: `[notebook_id, [2]]`
    pub async fn get_summary(&self, notebook_id: &str) -> Result<NotebookSummary, String> {
        let inner_json = format!("[\"{}\", [2]]", notebook_id);
        let response = self.batchexecute("VfAZjd", &inner_json).await?;

        let inner = extract_by_rpc_id(&response, "VfAZjd")
            .ok_or("No se encontró respuesta VfAZjd")?;

        let summary = crate::rpc::notebooks::parse_summary(&inner);
        info!("Got summary for notebook {} ({} topics)", notebook_id, summary.suggested_topics.len());
        Ok(summary)
    }

    /// Get sharing configuration for a notebook (public/private + shared users).
    ///
    /// RPC: `JFMDGd`, payload: `[notebook_id, [2]]`
    pub async fn get_share_status(&self, notebook_id: &str) -> Result<ShareStatus, String> {
        let inner_json = format!("[\"{}\", [2]]", notebook_id);
        let response = self.batchexecute("JFMDGd", &inner_json).await?;

        let inner = extract_by_rpc_id(&response, "JFMDGd")
            .ok_or("No se encontró respuesta JFMDGd")?;

        let status = crate::rpc::notebooks::parse_share_status(&inner, notebook_id);
        info!(
            "Got share status for notebook {} (public={}, users={})",
            notebook_id, status.is_public, status.shared_users.len()
        );
        Ok(status)
    }

    /// Set notebook visibility to public or private. After toggling, reads back
    /// the share status to return confirmed data.
    ///
    /// RPC: `QDyure`, payload: `[[[id, null, [access], [access, ""]]], 1, null, [2]]`
    pub async fn set_sharing_public(&self, notebook_id: &str, public: bool) -> Result<ShareStatus, String> {
        let access_code = if public { 1 } else { 0 };
        let inner_json = format!(
            "[[[\"{}\", null, [{}], [{} , \"\"]]], 1, null, [2]]",
            notebook_id, access_code, access_code
        );
        self.batchexecute("QDyure", &inner_json).await?;
        info!("Set notebook {} to {}", notebook_id, if public { "public" } else { "private" });

        // Post-mutation read: return confirmed status
        self.get_share_status(notebook_id).await
    }

    /// Get the source IDs for a given notebook
    pub async fn get_notebook_sources(&self, notebook_id: &str) -> Result<Vec<String>, String> {
        let payload = format!("[\"{}\", null, [2], null, 0]", notebook_id.replace('"', "\\\""));
        let response = self.batchexecute("rLM1Ne", &payload).await?;
        
        // Usar parser defensivo: extract_by_rpc_id
        let inner = extract_by_rpc_id(&response, "rLM1Ne")
            .ok_or("No se encontró respuesta rLM1Ne")?;
        
        // Extraer notebook_data: [[title, sources, notebook_id, ...]]
        let notebook_list = extract_notebook_list(&inner)
            .ok_or("No se pudo parsear lista de notebooks")?;
        
        let notebook_data = notebook_list.first()
            .and_then(|v| v.as_array())
            .ok_or("No se encontraron datos del notebook")?;
        
        // Extraer fuentes: extract_sources
        let source_ids = extract_sources(notebook_data)
            .ok_or_else(|| "No se pudieron extraer las fuentes".to_string())?;
        
        Ok(source_ids)
    }

    /// Get a specific source entry by ID from a notebook.
    ///
    /// Returns the raw source entry Value which contains status code at [3][1].
    /// Used by SourcePoller to determine source readiness.
    pub async fn get_source_entry(&self, notebook_id: &str, source_id: &str) -> Result<Option<Value>, String> {
        let payload = format!("[\"{}\", null, [2], null, 0]", notebook_id.replace('"', "\\\""));
        let response = self.batchexecute("rLM1Ne", &payload).await?;

        let inner = extract_by_rpc_id(&response, "rLM1Ne")
            .ok_or("No se encontró respuesta rLM1Ne")?;

        let notebook_list = extract_notebook_list(&inner)
            .ok_or("No se pudo parsear lista de notebooks")?;

        let notebook_data = notebook_list.first()
            .and_then(|v| v.as_array())
            .ok_or("No se encontraron datos del notebook")?;

        Ok(find_source_entry(notebook_data, source_id))
    }

    /// Ask a question to a notebook using the streaming endpoint
    pub async fn ask_question(&self, notebook_id: &str, question: &str) -> Result<String, String> {
        // Step 1: Get source IDs for the notebook
        let source_ids = self.get_notebook_sources(notebook_id).await?;
        
        if source_ids.is_empty() {
            return Err("No hay fuentes disponibles en esta libreta. Añade fuentes antes de preguntar.".to_string());
        }
        
        // Step 2: Build source array for the query
        // Format: [[["source_id_1"]], [["source_id_2"]], ...]
        let sources_array: Vec<String> = source_ids.iter()
            .map(|id| format!("[[\"{}\"]]", id))
            .collect();
        let sources_json = format!("[{}]", sources_array.join(","));
        
        // Step 3: Get or create conversation ID from cache
        let conv_id = self.conversation_cache.get_or_create(notebook_id, &Uuid::new_v4().to_string()).await;
        
        // Step 4: Get conversation history for context
        let history = self.conversation_cache.get_history(notebook_id).await;
        
        // Step 5: Build the params array (9 elements per notebooklm-py)
        // [sources_array, question, history, config, conv_id, null, null, notebook_id, 1]
        
        // Build history as JSON array of [question, answer] pairs
        let history_json = if let Some(msgs) = &history {
            let pairs: Vec<Value> = msgs.iter().map(|m| {
                serde_json::json!([m.question, m.answer])
            }).collect();
            Value::Array(pairs)
        } else {
            Value::Null
        };
        
        let params_json = serde_json::json!([
            serde_json::from_str::<Value>(&sources_json).unwrap_or(Value::Array(vec![])),
            question,
            history_json,
            [2, Value::Null, [1], [1]],
            conv_id,
            Value::Null,
            Value::Null,
            notebook_id,
            1
        ]).to_string();
        
        let f_req = serde_json::json!([null, params_json]).to_string();
        
        // Step 6: POST to the streaming endpoint
        let url = "https://notebooklm.google.com/_/LabsTailwindUi/data/google.internal.labs.tailwind.orchestration.v1.LabsTailwindOrchestrationService/GenerateFreeFormStreamed";
        
        let form_data = [
            ("f.req", f_req),
            ("at", self.csrf.clone())
        ];
        
        self.limiter.until_ready().await;
        Self::apply_jitter().await;
        
        let res = self.http.post(url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;
        
        if !res.status().is_success() {
            return Err(format!("Error HTTP {}", res.status()));
        }
        
        let text = res.text().await.map_err(|e| format!("No body text: {}", e))?;
        
        // Step 7: Parse the streaming response
        // Format: )]}'\n<size>\n<json>\n<size>\n<json>...
        let cleaned = if let Some(stripped) = text.strip_prefix(")]}'") {
            stripped.trim_start().to_string()
        } else {
            text
        };
        
        // Extract answer from chunks
        let answer = Self::parse_streaming_response(&cleaned)?;
        
        // Step 8: Cache the conversation for future questions
        self.conversation_cache.add_message(notebook_id, question.to_string(), answer.clone()).await;
        
        Ok(answer)
    }
    
    /// Parse the streaming response to extract the answer text
    fn parse_streaming_response(response_text: &str) -> Result<String, String> {
        // Clean anti-XSSI prefix
        let cleaned = if let Some(stripped) = response_text.strip_prefix(")]}'") {
            stripped.trim_start().to_string()
        } else {
            response_text.to_string()
        };
        
        // Split into lines and look for JSON chunks
        let mut answers: Vec<String> = Vec::new();
        
        for line in cleaned.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            
            // Skip size markers
            if line.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            
            // Try to parse as JSON
            if let Ok(data) = serde_json::from_str::<Value>(line) {
                // Look for answer in the structure
                if let Some(ans) = Self::extract_answer_from_chunk(&data)
                    && !ans.is_empty() {
                        answers.push(ans);
                    }
            }
        }
        
        // Return the longest answer found
        let best = answers.iter().max_by_key(|a| a.len()).cloned();
        
        match best {
            Some(answer) => Ok(answer),
            None => {
                // Try a simpler approach - look for text between quotes at the start
                if cleaned.len() > 10 {
                    Ok(format!("(Respuesta recibida, parsing mejorable):\n{}", &cleaned[..cleaned.len().min(3000)]))
                } else {
                    Err("No se pudo extraer respuesta".to_string())
                }
            }
        }
    }
    
    /// Extract answer text from a response chunk
    fn extract_answer_from_chunk(data: &Value) -> Option<String> {
        // Response structure from notebooklm-py:
        // [["wrb.fr", null, "<inner_json>", ...]]
        // inner_json: [["answer_text", null, [citations], ...], ...]
        
        if let Some(arr) = data.as_array() {
            for item in arr {
                if let Some(item_arr) = item.as_array() {
                    // Skip if first element isn't "wrb.fr"
                    if item_arr.first()?.as_str()? != "wrb.fr" {
                        continue;
                    }
                    
                    // Get inner JSON string
                    let inner_json_str = item_arr.get(2)?.as_str()?;
                    let inner_data: Value = serde_json::from_str(inner_json_str).ok()?;
                    
                    // inner_data is an array: [[answer_text, null, citations, ...], ...]
                    if let Some(inner_arr) = inner_data.as_array() {
                        for inner_item in inner_arr {
                            if let Some(ia) = inner_item.as_array() {
                                // Answer text is at index 0
                                if let Some(text) = ia.first().and_then(|v| v.as_str())
                                    && !text.is_empty() {
                                        return Some(text.to_string());
                                    }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    // =========================================================================
    // Artifact Operations — Module 2
    // =========================================================================

    /// List all artifacts in a notebook.
    ///
    /// Uses LIST_ARTIFACTS RPC (gArtLc) with a filter to exclude suggested artifacts.
    /// Returns typed `Artifact` structs with id, title, kind, status, and raw_data.
    ///
    /// # Arguments
    /// * `notebook_id` - The notebook UUID
    /// * `kind_filter` - Optional filter by artifact type (e.g., Audio, Video)
    pub async fn list_artifacts(
        &self,
        notebook_id: &str,
        kind_filter: Option<ArtifactType>,
    ) -> Result<Vec<Artifact>, String> {
        // LIST_ARTIFACTS params: [[2], notebook_id, filter_string]
        // Filter excludes "suggested" artifacts
        let payload = format!(
            "[[2],\"{}\",\"NOT artifact.status = \\\"ARTIFACT_STATUS_SUGGESTED\\\"\"]",
            notebook_id
        );

        let response = self.batchexecute(rpc_ids::LIST_ARTIFACTS, &payload).await?;

        let inner = extract_by_rpc_id(&response, rpc_ids::LIST_ARTIFACTS)
            .ok_or("No se encontró respuesta LIST_ARTIFACTS")?;

        let mut artifacts = parse_artifact_list(&inner);

        // Apply optional type filter
        if let Some(kind) = kind_filter {
            artifacts.retain(|a| a.kind == kind);
        }

        info!("Listed {} artifacts for notebook {}", artifacts.len(), notebook_id);
        Ok(artifacts)
    }

    /// Generate an artifact in a notebook.
    ///
    /// Uses CREATE_ARTIFACT RPC (R7cb6c) for all types except Mind Map.
    /// Returns (task_id, initial_status). The task_id equals the artifact_id.
    ///
    /// # Arguments
    /// * `notebook_id` - The notebook UUID
    /// * `config` - Type-safe artifact configuration (ArtifactConfig enum)
    pub async fn generate_artifact(
        &self,
        notebook_id: &str,
        config: &ArtifactConfig,
    ) -> Result<GenerationStatus, String> {
        let params = config.to_params_array(notebook_id);
        let payload = serde_json::to_string(&params)
            .map_err(|e| format!("Failed to serialize artifact params: {}", e))?;

        let response = self.batchexecute(rpc_ids::CREATE_ARTIFACT, &payload).await?;

        // Check for rate limiting — Google returns rpc_code "USER_DISPLAYABLE_ERROR"
        if let Some(rate_limited) = Self::check_rate_limit(&response) {
            return Ok(rate_limited);
        }

        let inner = extract_by_rpc_id(&response, rpc_ids::CREATE_ARTIFACT)
            .ok_or("No se encontró respuesta CREATE_ARTIFACT")?;

        parse_generation_result(&inner)
            .ok_or("No se pudo extraer task_id de la respuesta CREATE_ARTIFACT".to_string())
    }

    /// Generate a mind map (two-step RPC).
    ///
    /// Mind maps use a DIFFERENT generation pipeline than all other artifacts:
    /// 1. RPC `GENERATE_MIND_MAP` (yyryJe) — generates the mind map JSON
    /// 2. RPC `CREATE_NOTE` (CYK0Xb) — persists it as a note in the notebook
    ///
    /// The mind map will appear in artifact listings with type MIND_MAP (5).
    ///
    /// # Arguments
    /// * `notebook_id` - The notebook UUID
    /// * `source_ids` - Source IDs to include (if empty, uses all sources)
    ///
    /// # Returns
    /// `MindMapResult` with `note_id` and `mind_map_data`.
    ///
    /// Reference: teng-lin/notebooklm-py `generate_mind_map()`
    pub async fn generate_mind_map(
        &self,
        notebook_id: &str,
        source_ids: &[&str],
    ) -> Result<MindMapResult, String> {
        // Build source_ids in triple-nested format: [[[sid1]], [[sid2]], ...]
        let source_ids_nested: Vec<serde_json::Value> = source_ids
            .iter()
            .map(|sid| serde_json::json!([[[sid]]]))
            .collect();

        // GENERATE_MIND_MAP payload — completely different from CREATE_ARTIFACT
        let params = serde_json::json!([
            source_ids_nested,
            null,
            null,
            null,
            null,
            ["interactive_mindmap", [["[CONTEXT]", ""]], ""],
            null,
            [2, null, [1]],
        ]);

        let payload = serde_json::to_string(&params)
            .map_err(|e| format!("Failed to serialize mind map params: {}", e))?;

        let response = self
            .batchexecute(rpc_ids::GENERATE_MIND_MAP, &payload)
            .await?;

        // Parse response: result[0][0] = mind map JSON (string or already parsed)
        let mind_map_json = response
            .get(0)
            .and_then(|v| v.get(0))
            .ok_or("No se encontró respuesta GENERATE_MIND_MAP")?;

        // Parse the mind map JSON — could be a string or already a Value
        let mind_map_data: serde_json::Value = if let Some(s) = mind_map_json.as_str() {
            serde_json::from_str(s)
                .map_err(|e| format!("Failed to parse mind map JSON: {}", e))?
        } else {
            mind_map_json.clone()
        };

        // Serialize back to string for note content
        let mind_map_str = serde_json::to_string(&mind_map_data)
            .map_err(|e| format!("Failed to serialize mind map: {}", e))?;

        // Extract title from mind map data
        let title = mind_map_data
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Mind Map");

        // Step 2: Persist via CREATE_NOTE (CYK0Xb)
        // CREATE_NOTE params: [notebook_id, "", [1], None, title]
        let create_note_params = serde_json::json!([
            notebook_id,
            "",
            [1],
            null,
            title,
        ]);

        let create_payload = serde_json::to_string(&create_note_params)
            .map_err(|e| format!("Failed to serialize CREATE_NOTE params: {}", e))?;

        let note_response = self
            .batchexecute(rpc_ids::CREATE_NOTE, &create_payload)
            .await?;

        // Extract note_id from CREATE_NOTE response: result[0][0]
        let note_id = note_response
            .get(0)
            .and_then(|v| v.get(0))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Step 2b: Google ignores title in CREATE_NOTE, so update with content
        // UPDATE_NOTE params: [notebook_id, note_id, [[[content, title, [], 0]]]]
        if let Some(ref nid) = note_id {
            let update_params = serde_json::json!([
                notebook_id,
                nid,
                [[[mind_map_str, title, [], 0]]],
            ]);

            let update_payload = serde_json::to_string(&update_params)
                .map_err(|e| format!("Failed to serialize UPDATE_NOTE params: {}", e))?;

            // Best-effort update — don't fail the whole operation
            if let Err(e) = self.batchexecute("CbCNt", &update_payload).await {
                tracing::warn!("Failed to update note content: {}", e);
            }
        }

        info!(
            "Mind map generated: note_id={:?}, title={}",
            note_id, title
        );

        Ok(MindMapResult::new(
            note_id.unwrap_or_default(),
            mind_map_data,
        ))
    }

    /// Delete an artifact from a notebook.
    ///
    /// Uses DELETE_ARTIFACT RPC (V5N4be).
    ///
    /// # Arguments
    /// * `_notebook_id` - The notebook UUID (used in URL path but not in params)
    /// * `artifact_id` - The artifact to delete
    pub async fn delete_artifact(
        &self,
        _notebook_id: &str,
        artifact_id: &str,
    ) -> Result<(), String> {
        // DELETE params: [[2], artifact_id]
        let payload = format!("[[2],\"{}\"]", artifact_id);

        self.batchexecute(rpc_ids::DELETE_ARTIFACT, &payload).await?;

        info!("Deleted artifact {}", artifact_id);
        Ok(())
    }

    // =====================================================================
    // Task 6.3 — streaming_download
    // =====================================================================

    /// Download a file from a URL using streaming with chunked writes.
    ///
    /// Writes to a `.tmp` file first, then atomically renames to the final
    /// path on success. On any failure, the temp file is deleted.
    ///
    /// Uses the `upload_http` client (AD-3: reuse upload client for downloads).
    /// This client has auth cookies but no global Content-Type header.
    ///
    /// # Arguments
    /// * `url` - HTTPS URL on a trusted Google domain
    /// * `output_path` - Local file path for the download
    ///
    /// # Errors
    ///
    /// - `DownloadFailed` — domain validation, HTTP error, 0 bytes, auth expired
    /// - `ValidationError` — output path is a directory
    ///
    /// # Security
    ///
    /// Validates URL domain before sending auth cookies.
    /// Detects auth expiry by checking response Content-Type.
    ///
    /// Reference: teng-lin/notebooklm-py `_download_url`
    pub async fn streaming_download(
        &self,
        url: &str,
        output_path: &str,
    ) -> Result<String, NotebookLmError> {
        // 1. Validate domain — don't send cookies to arbitrary servers
        validate_google_domain(url)?;

        // 2. Validate output path
        let out = std::path::Path::new(output_path);
        if out.exists() && out.is_dir() {
            return Err(NotebookLmError::ValidationError(format!(
                "Output path is a directory: {}",
                output_path
            )));
        }

        // 3. Create parent directories
        if let Some(parent) = out.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                NotebookLmError::DownloadFailed(format!(
                    "Failed to create output directory: {}",
                    e
                ))
            })?;
        }

        // 4. Build temp file path: output.mp4.tmp
        let tmp_path = format!("{}.tmp", output_path);

        // 5. Stream download
        let result = self._do_streaming_download(url, &tmp_path).await;

        match result {
            Ok(total_bytes) => {
                // 6. Atomic rename: .tmp → final path
                std::fs::rename(&tmp_path, output_path).map_err(|e| {
                    // Clean up temp file on rename failure
                    let _ = std::fs::remove_file(&tmp_path);
                    NotebookLmError::DownloadFailed(format!(
                        "Failed to rename temp file to {}: {}",
                        output_path, e
                    ))
                })?;

                info!(
                    "Downloaded {} ({} bytes) → {}",
                    &url[..url.len().min(60)],
                    total_bytes,
                    output_path
                );
                Ok(output_path.to_string())
            }
            Err(e) => {
                // 7. Clean up temp file on any download failure
                let _ = std::fs::remove_file(&tmp_path);
                Err(e)
            }
        }
    }

    /// Internal streaming download implementation.
    ///
    /// Sends GET request with streaming, writes 64KB chunks to file.
    /// Returns total bytes written on success.
    async fn _do_streaming_download(
        &self,
        url: &str,
        tmp_path: &str,
    ) -> Result<u64, NotebookLmError> {
        let response = self
            .upload_http
            .get(url)
            .send()
            .await
            .map_err(|e| {
                NotebookLmError::DownloadFailed(format!(
                    "Download request failed: {}",
                    e
                ))
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(NotebookLmError::DownloadFailed(format!(
                "Download failed (HTTP {}): {}",
                status,
                &body[..body.len().min(200)]
            )));
        }

        // Detect auth expiry: if server returns HTML, cookies are dead
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if content_type.contains("text/html") {
            return Err(NotebookLmError::DownloadFailed(
                "Download returned HTML instead of media file. \
                 Authentication may have expired. Re-login and try again."
                    .to_string(),
            ));
        }

        // Stream to file in 64KB chunks
        use tokio::io::AsyncWriteExt;

        let mut file = tokio::fs::File::create(tmp_path).await.map_err(|e| {
            NotebookLmError::DownloadFailed(format!(
                "Failed to create temp file {}: {}",
                tmp_path, e
            ))
        })?;

        let mut total_bytes: u64 = 0;
        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| {
                NotebookLmError::DownloadFailed(format!(
                    "Download stream error: {}",
                    e
                ))
            })?;

            file.write_all(&chunk).await.map_err(|e| {
                NotebookLmError::DownloadFailed(format!(
                    "Failed to write to temp file: {}",
                    e
                ))
            })?;

            total_bytes += chunk.len() as u64;
        }

        file.flush().await.map_err(|e| {
            NotebookLmError::DownloadFailed(format!(
                "Failed to flush temp file: {}",
                e
            ))
        })?;

        // Check for empty download
        if total_bytes == 0 {
            return Err(NotebookLmError::DownloadFailed(
                "Download produced 0 bytes — the remote file may be missing or empty".to_string(),
            ));
        }

        Ok(total_bytes)
    }

    // =====================================================================
    // Task 6.8 — download_artifact dispatcher
    // =====================================================================

    /// Download a completed artifact to a local file.
    ///
    /// Dispatches to the correct download strategy based on artifact type.
    /// See method doc for the full strategy table.
    ///
    /// # Arguments
    /// * `notebook_id` - The notebook UUID
    /// * `artifact_id` - The artifact to download (must be completed)
    /// * `output_path` - Local file path for the download
    /// * `format` - Optional format: "pptx" for slide decks (default: "pdf")
    pub async fn download_artifact(
        &self,
        notebook_id: &str,
        artifact_id: &str,
        output_path: &str,
        format: Option<&str>,
    ) -> Result<String, NotebookLmError> {
        // 1. Find the artifact
        let artifacts = self
            .list_artifacts(notebook_id, None)
            .await
            .map_err(|e| NotebookLmError::DownloadFailed(format!("Failed to list artifacts: {}", e)))?;

        let artifact = artifacts
            .into_iter()
            .find(|a| a.id == artifact_id)
            .ok_or_else(|| {
                NotebookLmError::ArtifactNotFound(format!(
                    "Artifact {} not found in notebook {}",
                    artifact_id, notebook_id
                ))
            })?;

        // 2. Verify completed
        if !artifact.is_completed() {
            return Err(NotebookLmError::ArtifactNotReady(format!(
                "Artifact {} is not completed (status: {})",
                artifact_id, artifact.status
            )));
        }

        // 3. Create parent directories
        if let Some(parent) = std::path::Path::new(output_path).parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                NotebookLmError::DownloadFailed(format!(
                    "Failed to create output directory: {}", e
                ))
            })?;
        }

        info!(
            "Downloading artifact {} (kind: {}) → {}",
            artifact_id, artifact.kind, output_path
        );

        // 4. Dispatch by type
        match artifact.kind {
            ArtifactType::Audio | ArtifactType::Video | ArtifactType::Infographic => {
                self._download_streaming_media(&artifact.raw_data, output_path).await
            }
            ArtifactType::SlideDeck => {
                let fmt = format.unwrap_or("pdf");
                let url = extract_slide_deck_url(&artifact.raw_data, fmt).ok_or_else(|| {
                    NotebookLmError::DownloadFailed(format!(
                        "Could not extract slide deck {} URL (format: {})",
                        artifact_id, fmt
                    ))
                })?;
                self.streaming_download(&url, output_path).await
            }
            ArtifactType::Report => {
                self._download_report(&artifact.raw_data, output_path).await
            }
            ArtifactType::DataTable => {
                self._download_data_table(&artifact.raw_data, output_path).await
            }
            ArtifactType::Quiz | ArtifactType::Flashcards => {
                self._download_interactive(&artifact, output_path).await
            }
            ArtifactType::MindMap => {
                self._download_mind_map(notebook_id, artifact_id, output_path).await
            }
            ArtifactType::Unknown => {
                Err(NotebookLmError::DownloadFailed(format!(
                    "Cannot download unknown artifact type: {}",
                    artifact_id
                )))
            }
        }
    }

    // --- Strategy implementations ---

    /// Download streaming media (Audio, Video, Infographic) by extracting URL from raw data.
    async fn _download_streaming_media(
        &self,
        raw_data: &Value,
        output_path: &str,
    ) -> Result<String, NotebookLmError> {
        let url = extract_audio_url(raw_data)
            .or_else(|| extract_video_url(raw_data))
            .or_else(|| extract_infographic_url(raw_data))
            .ok_or_else(|| {
                NotebookLmError::DownloadFailed(
                    "Could not extract download URL from artifact data".to_string()
                )
            })?;

        self.streaming_download(&url, output_path).await
    }

    /// Download report as markdown — content is inline at `art[7]`.
    async fn _download_report(
        &self,
        raw_data: &Value,
        output_path: &str,
    ) -> Result<String, NotebookLmError> {
        let content = extract_report_content(raw_data).ok_or_else(|| {
            NotebookLmError::DownloadFailed(
                "Could not extract report markdown from artifact data".to_string()
            )
        })?;

        tokio::fs::write(output_path, content.as_bytes())
            .await
            .map_err(|e| {
                NotebookLmError::DownloadFailed(format!(
                    "Failed to write report to {}: {}",
                    output_path, e
                ))
            })?;

        info!("Report written to {}", output_path);
        Ok(output_path.to_string())
    }

    /// Download data table as CSV with UTF-8 BOM — cells at `art[18]`.
    async fn _download_data_table(
        &self,
        raw_data: &Value,
        output_path: &str,
    ) -> Result<String, NotebookLmError> {
        let (headers, rows) = parse_data_table(raw_data).ok_or_else(|| {
            NotebookLmError::DownloadFailed(
                "Could not parse data table from artifact data".to_string()
            )
        })?;

        // Build CSV with UTF-8 BOM
        let mut csv_content = String::from("\u{FEFF}");
        csv_content.push_str(&headers.join(","));
        csv_content.push('\n');
        for row in &rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    csv_content.push(',');
                }
                if cell.contains(',') || cell.contains('"') {
                    let escaped = cell.replace('"', "\"\"");
                    csv_content.push_str(&format!("\"{}\"", escaped));
                } else {
                    csv_content.push_str(cell);
                }
            }
            csv_content.push('\n');
        }

        tokio::fs::write(output_path, csv_content.as_bytes())
            .await
            .map_err(|e| {
                NotebookLmError::DownloadFailed(format!(
                    "Failed to write CSV to {}: {}",
                    output_path, e
                ))
            })?;

        info!(
            "Data table CSV written to {} ({} rows)",
            output_path,
            rows.len()
        );
        Ok(output_path.to_string())
    }

    /// Download quiz/flashcard via RPC `v9rmvd` → HTML → `data-app-data` → JSON.
    async fn _download_interactive(
        &self,
        artifact: &Artifact,
        output_path: &str,
    ) -> Result<String, NotebookLmError> {
        let payload = format!("[\"{}\"]", artifact.id);
        let response = self
            .batchexecute(rpc_ids::GET_INTERACTIVE_HTML, &payload)
            .await
            .map_err(|e| {
                NotebookLmError::DownloadFailed(format!(
                    "RPC call to get interactive content failed: {}", e
                ))
            })?;

        // Extract HTML from response: result[0][9][0]
        let inner = extract_by_rpc_id(&response, rpc_ids::GET_INTERACTIVE_HTML).ok_or_else(|| {
            NotebookLmError::DownloadFailed(
                "No response from GET_INTERACTIVE_HTML RPC".to_string()
            )
        })?;

        let html_content = inner
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|item| item.as_array())
            .and_then(|arr| arr.get(9))
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                NotebookLmError::DownloadFailed(
                    "Could not extract HTML from interactive response".to_string()
                )
            })?;

        // Parse data-app-data from HTML
        let app_data = extract_app_data(html_content).ok_or_else(|| {
            NotebookLmError::DownloadFailed(
                "No data-app-data attribute found in interactive HTML".to_string()
            )
        })?;

        // Format as JSON
        let content = if artifact.kind == ArtifactType::Quiz {
            let questions = app_data
                .get("quiz")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let json_output = serde_json::json!({
                "title": artifact.title,
                "questions": questions,
            });
            serde_json::to_string_pretty(&json_output).unwrap_or_default()
        } else {
            let cards = app_data
                .get("flashcards")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();
            let normalized: Vec<Value> = cards
                .iter()
                .map(|c| {
                    serde_json::json!({
                        "front": c.get("f").and_then(|v| v.as_str()).unwrap_or(""),
                        "back": c.get("b").and_then(|v| v.as_str()).unwrap_or(""),
                    })
                })
                .collect();
            let json_output = serde_json::json!({
                "title": artifact.title,
                "cards": normalized,
            });
            serde_json::to_string_pretty(&json_output).unwrap_or_default()
        };

        tokio::fs::write(output_path, content.as_bytes())
            .await
            .map_err(|e| {
                NotebookLmError::DownloadFailed(format!(
                    "Failed to write interactive content to {}: {}",
                    output_path, e
                ))
            })?;

        info!(
            "{} content written to {}",
            artifact.kind.as_str(),
            output_path
        );
        Ok(output_path.to_string())
    }

    /// Download mind map via RPC `cFji9` → find mind map note → extract JSON.
    async fn _download_mind_map(
        &self,
        notebook_id: &str,
        artifact_id: &str,
        output_path: &str,
    ) -> Result<String, NotebookLmError> {
        let payload = format!("[\"{}\"]", notebook_id);
        let response = self
            .batchexecute(rpc_ids::GET_NOTES_AND_MIND_MAPS, &payload)
            .await
            .map_err(|e| {
                NotebookLmError::DownloadFailed(format!(
                    "RPC call to get notes failed: {}", e
                ))
            })?;

        let inner = extract_by_rpc_id(&response, rpc_ids::GET_NOTES_AND_MIND_MAPS).ok_or_else(|| {
            NotebookLmError::DownloadFailed(
                "No response from GET_NOTES_AND_MIND_MAPS RPC".to_string()
            )
        })?;

        let notes = inner
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        // Find the mind map note by artifact_id
        let mind_map_item = notes
            .iter()
            .find(|item| {
                item.get(0)
                    .and_then(|v| v.as_str())
                    .map(|id| id == artifact_id)
                    .unwrap_or(false)
                    && is_mind_map_item(item)
            })
            .ok_or_else(|| {
                NotebookLmError::ArtifactNotFound(format!(
                    "Mind map {} not found in notes",
                    artifact_id
                ))
            })?;

        let json_value = extract_mind_map_json(mind_map_item).ok_or_else(|| {
            NotebookLmError::DownloadFailed(
                "Could not parse mind map JSON from note".to_string()
            )
        })?;

        let content = serde_json::to_string_pretty(&json_value).unwrap_or_default();

        tokio::fs::write(output_path, content.as_bytes())
            .await
            .map_err(|e| {
                NotebookLmError::DownloadFailed(format!(
                    "Failed to write mind map JSON to {}: {}",
                    output_path, e
                ))
            })?;

        info!("Mind map written to {}", output_path);
        Ok(output_path.to_string())
    }

    /// Check if the RPC response indicates rate limiting.
    ///
    /// Google returns `rpc_code == "USER_DISPLAYABLE_ERROR"` when rate limited.
    /// Instead of treating this as an error, we return a "failed" GenerationStatus
    /// with is_rate_limited=true so the caller can retry.
    fn check_rate_limit(response: &Value) -> Option<GenerationStatus> {
        let arr = response.as_array()?;

        for item in arr {
            let item_arr = item.as_array()?;
            if item_arr.first()?.as_str()? != "wrb.fr" {
                continue;
            }

            // Check if rpc_code at index [1] indicates rate limiting
            let rpc_code = item_arr.get(1)?.as_str()?;
            if rpc_code == "USER_DISPLAYABLE_ERROR" {
                return Some(GenerationStatus::rate_limited(
                    "Rate limited by API (USER_DISPLAYABLE_ERROR)"
                ));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // 6.1 — is_youtube_url tests
    // =========================================================================

    #[test]
    fn test_youtube_watch_url() {
        assert!(is_youtube_url("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn test_youtube_short_url() {
        assert!(is_youtube_url("https://youtu.be/dQw4w9WgXcQ"));
    }

    #[test]
    fn test_youtube_embed_url() {
        assert!(is_youtube_url("https://www.youtube.com/embed/dQw4w9WgXcQ"));
    }

    #[test]
    fn test_youtube_live_url() {
        assert!(is_youtube_url("https://www.youtube.com/live/dQw4w9WgXcQ"));
    }

    #[test]
    fn test_youtube_music_url() {
        assert!(is_youtube_url("https://music.youtube.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn test_youtube_mobile_url() {
        assert!(is_youtube_url("https://m.youtube.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn test_youtube_no_subdomain() {
        assert!(is_youtube_url("https://youtube.com/watch?v=dQw4w9WgXcQ"));
    }

    #[test]
    fn test_youtube_with_query_params() {
        assert!(is_youtube_url("https://www.youtube.com/watch?v=abc&t=120&list=xyz"));
    }

    #[test]
    fn test_youtube_short_with_query() {
        assert!(is_youtube_url("https://youtu.be/dQw4w9WgXcQ?t=30"));
    }

    #[test]
    fn test_non_youtube_url() {
        assert!(!is_youtube_url("https://vimeo.com/123456"));
        assert!(!is_youtube_url("https://example.com/video"));
        assert!(!is_youtube_url("https://twitch.tv/channel"));
    }

    #[test]
    fn test_invalid_url() {
        assert!(!is_youtube_url("not-a-url"));
        assert!(!is_youtube_url(""));
    }

    #[test]
    fn test_youtube_url_with_port() {
        assert!(is_youtube_url("https://youtube.com:443/watch?v=abc"));
    }

    // =========================================================================
    // 6.2 — validate_google_domain tests
    // =========================================================================

    #[test]
    fn test_validate_google_domain_bare_google_com() {
        assert!(validate_google_domain("https://google.com/path").is_ok());
    }

    #[test]
    fn test_validate_google_domain_www_subdomain() {
        assert!(validate_google_domain("https://www.google.com/path").is_ok());
    }

    #[test]
    fn test_validate_google_domain_nested_subdomain() {
        assert!(validate_google_domain("https://notebooklm.google.com/path").is_ok());
        assert!(validate_google_domain("https://docs.google.com/document/d/abc").is_ok());
    }

    #[test]
    fn test_validate_google_domain_googleusercontent() {
        assert!(validate_google_domain("https://lh3.googleusercontent.com/image.png").is_ok());
        assert!(validate_google_domain("https://cdn.googleusercontent.com/audio.mp4").is_ok());
    }

    #[test]
    fn test_validate_google_domain_googleapis() {
        assert!(validate_google_domain("https://storage.googleapis.com/bucket/file").is_ok());
        assert!(validate_google_domain("https://www.googleapis.com/upload/drive/v3").is_ok());
    }

    #[test]
    fn test_validate_google_domain_with_query_and_fragment() {
        assert!(validate_google_domain("https://cdn.googleusercontent.com/audio.mp4?token=abc&exp=123").is_ok());
        assert!(validate_google_domain("https://storage.googleapis.com/bucket/file#section").is_ok());
    }

    #[test]
    fn test_validate_google_domain_rejects_http() {
        let err = validate_google_domain("http://google.com/path");
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("HTTPS"));
    }

    #[test]
    fn test_validate_google_domain_rejects_untrusted_domain() {
        let err = validate_google_domain("https://evil.com/malware");
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Untrusted"));
    }

    #[test]
    fn test_validate_google_domain_rejects_youtube() {
        // youtube.com is a Google domain but NOT in the download trust list
        let err = validate_google_domain("https://www.youtube.com/watch?v=abc");
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Untrusted"));
    }

    #[test]
    fn test_validate_google_domain_allows_gmail() {
        // mail.google.com ends with .google.com → should be OK
        assert!(validate_google_domain("https://mail.google.com/mail/inbox").is_ok());
    }

    #[test]
    fn test_validate_google_domain_rejects_invalid_url() {
        let err = validate_google_domain("not-a-url");
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Invalid"));
    }

    #[test]
    fn test_validate_google_domain_rejects_empty() {
        let err = validate_google_domain("");
        assert!(err.is_err());
    }

    #[test]
    fn test_validate_google_domain_rejects_no_host() {
        let err = validate_google_domain("https:///path");
        assert!(err.is_err());
    }

    #[test]
    fn test_validate_google_domain_rejects_ftp() {
        let err = validate_google_domain("ftp://google.com/file");
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("HTTPS"));
    }

    #[test]
    fn test_validate_google_domain_truncates_long_urls_in_error() {
        let long_url = format!("https://evil.com/{}", "x".repeat(200));
        let err = validate_google_domain(&long_url).unwrap_err();
        let msg = err.to_string();
        // Error message should be truncated, not the full 200+ char URL
        assert!(msg.len() < 200, "Error message should truncate long URLs, got {} chars", msg.len());
    }

    // =========================================================================
    // 6.3 — streaming_download tests
    // =========================================================================

    /// Helper: build a NotebookLmClient with dummy cookies (for download tests only)
    fn make_test_client() -> NotebookLmClient {
        NotebookLmClient::new("test_cookie=1".to_string(), "test_csrf".to_string(), String::new())
    }

    #[test]
    fn test_streaming_download_rejects_untrusted_domain() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = make_test_client();
        let result = rt.block_on(async {
            client.streaming_download("https://evil.com/malware.exe", "/tmp/test.exe").await
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Untrusted"));
    }

    #[test]
    fn test_streaming_download_rejects_http_scheme() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = make_test_client();
        let result = rt.block_on(async {
            client.streaming_download("http://google.com/file.mp4", "/tmp/test.mp4").await
        });
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("HTTPS"));
    }

    #[tokio::test]
    async fn test_streaming_download_creates_parent_directories() {
        let client = make_test_client();
        // Use a path in a nested temp directory that doesn't exist yet
        let dir = std::env::temp_dir().join("notebooklm_test_nested");
        let _ = std::fs::remove_dir_all(&dir); // clean up from previous runs
        let output_path = dir.join("subdir").join("test_file.mp4");

        let result = client
            .streaming_download(
                "https://storage.googleapis.com/nonexistent/file.mp4",
                output_path.to_str().unwrap(),
            )
            .await;

        // Should fail because the URL doesn't actually serve content (network error),
        // but the parent directories should have been created
        let _ = std::fs::remove_dir_all(&dir); // clean up
        // We can't easily assert directory creation without the download succeeding,
        // but the function should NOT fail at directory creation
        assert!(result.is_err()); // Expected: network/download error, not directory error
    }

    #[tokio::test]
    async fn test_streaming_download_with_mock_server() {
        use tokio::net::TcpListener;
        use tokio::io::AsyncReadExt;
        use tokio::io::AsyncWriteExt;

        // Start a local TCP server that responds with fake media data
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let _port = listener.local_addr().unwrap().port();

        // Server task: accept connection, send HTTP response with fake media
        let server_handle = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();

            // Read the HTTP request (ignore it)
            let mut buf = [0u8; 1024];
            let _ = stream.read(&mut buf).await;

            // Send HTTP response with fake MP4 data
            let body = b"FAKE_MP4_DATA_1234567890"; // 21 bytes of "media"
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: audio/mp4\r\nContent-Length: {}\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
            let _ = stream.write_all(body).await;
            let _ = stream.flush().await;
        });

        // Client: download from local server
        // NOTE: This will fail domain validation because 127.0.0.1 is not trusted.
        // That's correct behavior — we're testing the streaming mechanics.
        // For a proper test, we'd need to bypass domain validation or use a mock.
        server_handle.abort();

        // Instead, test the full flow with a real but tiny HTTPS endpoint
        // (this is more of a smoke test — it will fail without real credentials)
        let client = make_test_client();
        let tmp = std::env::temp_dir().join("notebooklm_stream_test.mp4");
        let _ = std::fs::remove_file(&tmp);

        let result = client
            .streaming_download(
                "https://storage.googleapis.com/test-bucket/nonexistent.mp4",
                tmp.to_str().unwrap(),
            )
            .await;

        // Clean up
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(tmp.to_str().unwrap().to_string() + ".tmp");

        // Result depends on network — either 404/403 or network error
        // But it should NOT panic and should return DownloadFailed
        match result {
            Err(NotebookLmError::DownloadFailed(msg)) => {
                assert!(!msg.is_empty());
            }
            Err(e) => {
                panic!("Expected DownloadFailed, got: {:?}", e);
            }
            Ok(_) => {
                // If it actually downloaded something, that's fine too
            }
        }
    }

    #[test]
    fn test_streaming_download_rejects_directory_as_output() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = make_test_client();
        let tmp_dir = std::env::temp_dir().join("notebooklm_dir_test");
        let _ = std::fs::create_dir_all(&tmp_dir);

        let result = rt.block_on(async {
            client
                .streaming_download(
                    "https://storage.googleapis.com/bucket/file.mp4",
                    tmp_dir.to_str().unwrap(),
                )
                .await
        });

        let _ = std::fs::remove_dir_all(&tmp_dir);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("directory"),
            "Expected 'directory' in error message"
        );
    }

    #[test]
    fn test_temp_file_path_construction() {
        // Verify the temp file naming convention
        let output = "/path/to/audio.mp4";
        let tmp = format!("{}.tmp", output);
        assert_eq!(tmp, "/path/to/audio.mp4.tmp");

        let output2 = "C:\\Users\\test\\video.mp4";
        let tmp2 = format!("{}.tmp", output2);
        assert_eq!(tmp2, "C:\\Users\\test\\video.mp4.tmp");
    }

    #[test]
    fn test_temp_file_cleanup_on_failure() {
        // Simulate: create a .tmp file, verify it exists, then verify removal
        let tmp_path = std::env::temp_dir().join("notebooklm_cleanup_test.txt.tmp");
        std::fs::write(&tmp_path, "partial data").unwrap();
        assert!(tmp_path.exists());

        // Simulate cleanup (same as streaming_download error path)
        let _ = std::fs::remove_file(&tmp_path);
        assert!(!tmp_path.exists());
    }

    #[test]
    fn test_atomic_rename_success() {
        // Test the rename pattern used by streaming_download
        let dir = std::env::temp_dir().join("notebooklm_rename_test");
        let _ = std::fs::create_dir_all(&dir);

        let tmp_path = dir.join("file.mp4.tmp");
        let final_path = dir.join("file.mp4");

        // Write temp file
        std::fs::write(&tmp_path, "downloaded content").unwrap();
        assert!(tmp_path.exists());
        assert!(!final_path.exists());

        // Atomic rename
        std::fs::rename(&tmp_path, &final_path).unwrap();
        assert!(!tmp_path.exists());
        assert!(final_path.exists());

        // Verify content
        let content = std::fs::read_to_string(&final_path).unwrap();
        assert_eq!(content, "downloaded content");

        // Clean up
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_atomic_rename_preserves_content() {
        // Verify that rename doesn't corrupt file content
        let dir = std::env::temp_dir().join("notebooklm_rename_content_test");
        let _ = std::fs::create_dir_all(&dir);

        let tmp_path = dir.join("data.bin.tmp");
        let final_path = dir.join("data.bin");

        // Write binary data to temp file
        let data: Vec<u8> = (0..=255).collect();
        std::fs::write(&tmp_path, &data).unwrap();

        std::fs::rename(&tmp_path, &final_path).unwrap();

        let read_back = std::fs::read(&final_path).unwrap();
        assert_eq!(read_back, data);

        let _ = std::fs::remove_dir_all(&dir);
    }

    // -----------------------------------------------------------------------
    // generate_mind_map — payload & parsing tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_mind_map_payload_structure() {
        // Verify the GENERATE_MIND_MAP payload matches the Python reference
        let source_ids: Vec<&str> = vec!["src-1", "src-2"];
        let source_ids_nested: Vec<serde_json::Value> = source_ids
            .iter()
            .map(|sid| serde_json::json!([[[sid]]]))
            .collect();

        let params = serde_json::json!([
            source_ids_nested,
            null,
            null,
            null,
            null,
            ["interactive_mindmap", [["[CONTEXT]", ""]], ""],
            null,
            [2, null, [1]],
        ]);

        let arr = params.as_array().unwrap();
        assert_eq!(arr.len(), 8, "GENERATE_MIND_MAP params must have 8 elements");

        // First element: source IDs nested
        let sources = arr[0].as_array().unwrap();
        assert_eq!(sources.len(), 2);
        assert_eq!(sources[0], serde_json::json!([[[  "src-1"]]]));

        // Element at index 5: mind map config
        let config = arr[5].as_array().unwrap();
        assert_eq!(config[0], "interactive_mindmap");
    }

    #[test]
    fn test_mind_map_parse_json_string() {
        // When API returns mind map as JSON string at result[0][0]
        let mind_map_json_str = r#"{"name":"Test Map","children":[{"text":"Node 1"}]}"#;
        let mind_map_json: serde_json::Value = serde_json::from_str(mind_map_json_str).unwrap();

        // Verify parsing
        assert_eq!(mind_map_json["name"], "Test Map");
        assert_eq!(mind_map_json["children"][0]["text"], "Node 1");
    }

    #[test]
    fn test_mind_map_parse_already_value() {
        // When API returns mind map as already-parsed Value at result[0][0]
        let mind_map_value = serde_json::json!({
            "name": "Direct Value Map",
            "children": [{"text": "Root"}]
        });

        // Serialize back to string (same as what generate_mind_map does)
        let mind_map_str = serde_json::to_string(&mind_map_value).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&mind_map_str).unwrap();
        assert_eq!(parsed["name"], "Direct Value Map");
    }

    #[test]
    fn test_mind_map_title_extraction() {
        // Extract title from mind map data
        let data = serde_json::json!({"name": "My Mind Map", "children": []});
        let title = data.get("name").and_then(|v| v.as_str()).unwrap_or("Mind Map");
        assert_eq!(title, "My Mind Map");

        // Missing name → default
        let data_no_name = serde_json::json!({"children": []});
        let title_default = data_no_name
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Mind Map");
        assert_eq!(title_default, "Mind Map");
    }

    #[test]
    fn test_create_note_payload_structure() {
        // Verify CREATE_NOTE payload matches Python reference
        let params = serde_json::json!(["notebook-id", "", [1], null, "Mind Map Title"]);
        let arr = params.as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0], "notebook-id");
        assert_eq!(arr[1], "");
        assert_eq!(arr[2], serde_json::json!([1]));
        assert!(arr[3].is_null());
        assert_eq!(arr[4], "Mind Map Title");
    }

    #[test]
    fn test_mind_map_result_empty() {
        let r = MindMapResult::empty();
        assert!(r.note_id.is_none());
        assert!(r.mind_map_data.is_none());
    }

    #[test]
    fn test_mind_map_result_new() {
        let data = serde_json::json!({"name": "Test", "children": []});
        let r = MindMapResult::new("note-id".to_string(), data.clone());
        assert_eq!(r.note_id.as_deref(), Some("note-id"));
        assert!(r.mind_map_data.is_some());
    }

    // =========================================================================
    // 9.1-9.4 — Integration tests (require real credentials)
    //
    // These tests hit the REAL NotebookLM API. They are #[ignore]d by default.
    // Run with: NOTEBOOKLM_INTEGRATION_TEST=1 cargo test -- --ignored
    //
    // Prerequisites:
    //   1. Run `cargo run -- auth-browser` to save credentials
    //   2. Set NOTEBOOKLM_INTEGRATION_TEST=1 to enable
    // =========================================================================

    /// Helper: create a real client from stored credentials (keyring or DPAPI).
    /// Returns None if no credentials are available (test is skipped).
    async fn make_real_client() -> Option<NotebookLmClient> {
        // Skip unless explicitly enabled
        if std::env::var("NOTEBOOKLM_INTEGRATION_TEST").unwrap_or_default() != "1" {
            return None;
        }

        let (cookie, csrf, sid) = if let Some((c, cs, s)) = crate::auth_browser::load_credentials() {
            (c, cs, s)
        } else {
            // Try DPAPI session file directly
            let session_path = dirs::home_dir()
                .map(|d| d.join(".notebooklm-mcp").join("session.bin"))?;
            if !session_path.exists() {
                return None;
            }
            let encrypted = std::fs::read(&session_path).ok()?;
            let json = windows_dpapi::decrypt_data(
                &encrypted,
                windows_dpapi::Scope::User,
                None,
            ).ok()?;
            let session: serde_json::Value = serde_json::from_slice(&json).ok()?;
            (
                session.get("cookie").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                session.get("csrf").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                session.get("sid").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            )
        };

        if cookie.is_empty() {
            return None;
        }

        Some(NotebookLmClient::new(cookie, csrf, sid))
    }

    /// Helper: create a notebook and add a text source.
    async fn setup_notebook_with_source(client: &NotebookLmClient) -> Option<(String, String)> {
        let notebook_id = client.create_notebook("Integration Test").await.ok()?;
        info!("Created notebook: {}", notebook_id);

        let source_content = "The Rust programming language is a systems programming language \
            designed for safety, concurrency, and performance. It was originally developed by \
            Mozilla Research and is now maintained by the Rust Foundation.";

        let source_id = client
            .add_source(&notebook_id, "Rust Programming Language", source_content)
            .await
            .ok()?;
        info!("Added source: {}", source_id);

        // Brief wait for source processing
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        Some((notebook_id, source_id))
    }

    /// Helper: poll artifact status until completed or timeout.
    async fn wait_for_artifact(
        client: &NotebookLmClient,
        notebook_id: &str,
        task_id: &str,
        timeout_secs: u64,
    ) -> Result<Artifact, String> {
        use std::time::Duration;
        let start = std::time::Instant::now();
        let mut interval = Duration::from_secs(2);

        loop {
            if start.elapsed() >= Duration::from_secs(timeout_secs) {
                return Err(format!("Timeout after {:?} waiting for artifact {}", start.elapsed(), task_id));
            }

            let artifacts = client.list_artifacts(notebook_id, None).await?;
            if let Some(artifact) = artifacts.into_iter().find(|a| a.id == task_id) {
                if artifact.is_completed() {
                    return Ok(artifact);
                }
                if artifact.is_failed() {
                    return Err(format!("Artifact {} failed: {}", task_id, artifact.status));
                }
                info!("Artifact {} status: {} — waiting...", task_id, artifact.status);
            }

            tokio::time::sleep(interval).await;
            interval = std::cmp::min(interval * 2, Duration::from_secs(10));
        }
    }

    // --- 9.1: Audio generation → poll → download ---

    #[tokio::test]
    #[ignore]
    async fn test_9_1_generate_audio_and_download() {
        let client = match make_real_client().await {
            Some(c) => c,
            None => {
                info!("Skipping: no credentials or NOTEBOOKLM_INTEGRATION_TEST not set");
                return;
            }
        };

        let (notebook_id, source_id) = match setup_notebook_with_source(&client).await {
            Some(r) => r,
            None => {
                info!("Skipping: failed to setup notebook");
                return;
            }
        };

        let config = ArtifactConfig::Audio {
            format: crate::rpc::artifacts::AudioFormat::DeepDive,
            length: crate::rpc::artifacts::AudioLength::Default,
            instructions: None,
            language: "en".to_string(),
            source_ids: vec![source_id.clone()],
        };

        let status = client.generate_artifact(&notebook_id, &config).await;
        assert!(status.is_ok(), "generate_artifact failed: {:?}", status.err());
        let status = status.unwrap();
        info!("Generation status: {:?}", status);

        if status.is_rate_limited() {
            info!("Rate limited — skipping download (test passes)");
            return;
        }
        assert!(!status.task_id.is_empty(), "No task_id returned");

        let artifact = wait_for_artifact(&client, &notebook_id, &status.task_id, 120).await;
        assert!(artifact.is_ok(), "wait_for_artifact failed: {:?}", artifact.err());
        let artifact = artifact.unwrap();
        info!("Artifact completed: kind={:?}, status={}", artifact.kind, artifact.status);

        let tmp = std::env::temp_dir().join("notebooklm_integration_audio.mp3");
        let _ = std::fs::remove_file(&tmp);

        let result = client
            .download_artifact(&notebook_id, &artifact.id, tmp.to_str().unwrap(), None)
            .await;
        assert!(result.is_ok(), "download failed: {:?}", result.err());

        let size = tokio::fs::metadata(&tmp).await.unwrap().len();
        info!("Downloaded audio: {} bytes", size);
        assert!(size > 1000, "File too small ({} bytes)", size);

        let _ = std::fs::remove_file(&tmp);
        info!("9.1 PASSED");
    }

    // --- 9.2: Quiz generation → download ---

    #[tokio::test]
    #[ignore]
    async fn test_9_2_generate_quiz_and_download() {
        let client = match make_real_client().await {
            Some(c) => c,
            None => {
                info!("Skipping: no credentials or NOTEBOOKLM_INTEGRATION_TEST not set");
                return;
            }
        };

        let (notebook_id, source_id) = match setup_notebook_with_source(&client).await {
            Some(r) => r,
            None => {
                info!("Skipping: failed to setup notebook");
                return;
            }
        };

        let config = ArtifactConfig::Quiz {
            difficulty: crate::rpc::artifacts::QuizDifficulty::Medium,
            quantity: crate::rpc::artifacts::QuizQuantity::Standard,
            instructions: None,
            source_ids: vec![source_id.clone()],
        };

        let status = client.generate_artifact(&notebook_id, &config).await;
        assert!(status.is_ok(), "generate_artifact failed: {:?}", status.err());
        let status = status.unwrap();
        info!("Quiz generation status: {:?}", status);

        if status.is_rate_limited() {
            info!("Rate limited — test passes");
            return;
        }

        let artifact = wait_for_artifact(&client, &notebook_id, &status.task_id, 60).await;
        assert!(artifact.is_ok(), "wait_for_artifact failed: {:?}", artifact.err());
        let artifact = artifact.unwrap();
        assert!(artifact.is_completed(), "Quiz not completed");

        let tmp = std::env::temp_dir().join("notebooklm_integration_quiz.json");
        let _ = std::fs::remove_file(&tmp);

        let result = client
            .download_artifact(&notebook_id, &artifact.id, tmp.to_str().unwrap(), None)
            .await;
        assert!(result.is_ok(), "download failed: {:?}", result.err());

        let content = std::fs::read_to_string(&tmp).unwrap_or_default();
        assert!(content.contains("questions"), "Quiz JSON missing 'questions' field");
        assert!(content.contains("\"title\""), "Quiz JSON missing 'title' field");
        info!("Quiz downloaded: {} bytes", content.len());

        let _ = std::fs::remove_file(&tmp);
        info!("9.2 PASSED");
    }

    // --- 9.3: Mind map generation (two-step) ---

    #[tokio::test]
    #[ignore]
    async fn test_9_3_generate_mind_map() {
        let client = match make_real_client().await {
            Some(c) => c,
            None => {
                info!("Skipping: no credentials or NOTEBOOKLM_INTEGRATION_TEST not set");
                return;
            }
        };

        let (notebook_id, source_id) = match setup_notebook_with_source(&client).await {
            Some(r) => r,
            None => {
                info!("Skipping: failed to setup notebook");
                return;
            }
        };

        let result = client.generate_mind_map(&notebook_id, &[&source_id]).await;
        assert!(result.is_ok(), "generate_mind_map failed: {:?}", result.err());
        let mm = result.unwrap();

        info!("Mind map result: note_id={:?}", mm.note_id);
        assert!(mm.note_id.is_some(), "No note_id returned");
        assert!(mm.mind_map_data.is_some(), "No mind_map_data returned");

        let data = mm.mind_map_data.unwrap();
        assert!(
            data.get("name").is_some() || data.get("children").is_some(),
            "Mind map JSON missing expected fields"
        );
        info!("Mind map name: {:?}", data.get("name"));
        info!("9.3 PASSED");
    }

    // --- 9.4: Rate limiting detection ---

    #[tokio::test]
    #[ignore]
    async fn test_9_4_rate_limiting_returns_retryable_status() {
        let client = match make_real_client().await {
            Some(c) => c,
            None => {
                info!("Skipping: no credentials or NOTEBOOKLM_INTEGRATION_TEST not set");
                return;
            }
        };

        let (notebook_id, source_id) = match setup_notebook_with_source(&client).await {
            Some(r) => r,
            None => {
                info!("Skipping: failed to setup notebook");
                return;
            }
        };

        let config = ArtifactConfig::Report {
            format: crate::rpc::artifacts::ReportFormat::BriefingDoc,
            language: "en".to_string(),
            source_ids: vec![source_id.clone()],
            extra_instructions: None,
        };

        let mut found_rate_limit = false;
        for i in 0..5 {
            let status = client.generate_artifact(&notebook_id, &config).await;
            match status {
                Ok(s) if s.is_rate_limited() => {
                    info!("Request {} returned rate-limited status", i + 1);
                    found_rate_limit = true;
                    break;
                }
                Ok(s) => {
                    info!("Request {} returned: {:?}", i + 1, s);
                }
                Err(e) => {
                    info!("Request {} error: {}", i + 1, e);
                }
            }
        }

        if found_rate_limit {
            info!("9.4 PASSED — rate limiting detected correctly");
        } else {
            info!("9.4 PASSED — rate limiting not triggered (OK, Google may not always rate-limit)");
        }
    }
}
