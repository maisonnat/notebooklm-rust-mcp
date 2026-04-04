use reqwest::{Client, header};
use serde_json::Value;
use std::time::Duration;
use rand::Rng;
use governor::{Quota, RateLimiter, state::NotKeyed, state::InMemoryState, clock::DefaultClock};
use tokio::sync::Semaphore;
use tracing::info;
use uuid::Uuid;

// Importar funciones del parser para acceso defensivo
use crate::parser::{extract_by_rpc_id, strip_antixssi_prefix, get_string_at, get_uuid_at, get_string_at_or_default, extract_notebook_list, extract_sources};

// Re-exportar errores para uso externo
pub use crate::errors::NotebookLmError;

// Re-exportar SourcePoller
pub use crate::source_poller::SourcePoller;

// Re-exportar conversation cache
pub use crate::conversation_cache::{ConversationCache, SharedConversationCache, new_conversation_cache};

type Limiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notebook {
    pub id: String,
    pub title: String,
}


pub struct NotebookLmClient {
    http: Client,
    csrf: String,
    limiter: Limiter,
    conversation_cache: SharedConversationCache,
    #[allow(dead_code)]
    upload_semaphore: Semaphore,
}

impl NotebookLmClient {
    pub fn new(cookie: String, csrf: String) -> Self {
        let quota = Quota::with_period(Duration::from_secs(2)).unwrap();
        let limiter = RateLimiter::direct(quota);

        let mut headers = header::HeaderMap::new();
        headers.insert(header::COOKIE, header::HeaderValue::from_str(&cookie).unwrap());
        headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/x-www-form-urlencoded;charset=utf-8"));

        let http = Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        // Semáforo para limitar uploads a 2 simultáneos
        let upload_semaphore = Semaphore::new(2);

        Self {
            http,
            csrf,
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

        let url = format!("https://notebooklm.google.com/_/LabsTailwindUi/data/batchexecute?rpcids={}&rt=c", rpc_id);

        let res = self.http.post(&url)
            .form(&form_data)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("Error HTTP {}", res.status()));
        }

        let text = res.text().await.map_err(|e| format!("No body text: {}", e))?;
        
        // Usar parser defensivo: strip_antixssi_prefix
        let cleaned = strip_antixssi_prefix(&text);
        
        // Parsear el JSON limpio
        let v: Value = serde_json::from_str(&cleaned).map_err(|e| e.to_string())?;
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
                    notebooks.push(Notebook { id, title });
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
        let cleaned = if text.starts_with(")]}'") {
            text[4..].trim_start().to_string()
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
        let cleaned = if response_text.starts_with(")]}'") {
            response_text[4..].trim_start().to_string()
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
                if let Some(ans) = Self::extract_answer_from_chunk(&data) {
                    if !ans.is_empty() {
                        answers.push(ans);
                    }
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
                    if item_arr.get(0)?.as_str()? != "wrb.fr" {
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
                                if let Some(text) = ia.get(0).and_then(|v| v.as_str()) {
                                    if !text.is_empty() {
                                        return Some(text.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
