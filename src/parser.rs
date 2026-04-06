//! Parser defensivo para respuestas RPC de NotebookLM
//!
//! Lecciones del reverse engineering (notebooklm-py):
//! - Arrays posicionales: [id, null, null, text] no objetos JSON
//! - Payload anidado: doble serialización (objeto → String → json![])
//! - Anti-Hijacking: prefijos )]}' que deben removerse
//! - Acceso defensivo: nunca usar unwrap() en índices de array
//!
//! # Gotchas descubierto durante implementación
//!
//! - **Infographic nesting**: El URL está en `item[2][0][1][0]`, no `item[2][0][1][0][0]`.
//!   Cada nivel debe checkearse que sea array con length suficiente.
//! - **Video quality**: Python itera TODOS los items, guarda el último video/mp4 URL,
//!   pero solo BREAK cuando quality==4. Fallback = último video/mp4 encontrado.
//! - **Slide deck**: Los arrays de metadata empiezan en índice 16. Los índices 5-15
//!   son 11 nulls. El test data DEBE tener exactamente esa estructura.
//! - **Mind maps**: Usan sistema COMPLETAMENTE diferente al quiz/flashcards.
//!   RPC `cFji9` (notes system), no HTML scraping, JSON string directo en note.
//! - **Quiz/Flashcards**: Comparten pipeline: RPC `v9rmvd` → HTML → regex
//!   `data-app-data` → HTML unescape → JSON parse. Solo difieren en key y formato.
//! - **Data table CSV**: Necesita UTF-8 BOM (`\u{FEFF}`) para compatibilidad Excel.
//! - **HTML entities**: Los `data-app-data` contienen `&quot;` etc que deben
//!   decodificarse con `html_escape::decode_html_entities()` antes de JSON parse.

use serde_json::Value;
use crate::rpc::artifacts::{ArtifactTypeCode, ArtifactType, ArtifactStatus, GenerationStatus};

/// Resultado genérico de una llamada RPC batchexecute
/// Formato: [["wrb.fr", "rpc_id", "<inner_json>", ...], ...]
#[derive(Debug, Clone)]
pub struct RpcResponse {
    pub rpc_id: String,
    pub inner_json: Value,
}

/// Extrae el inner_json de una respuesta RPC buscando por rpc_id
/// DEVUELVE el valor en la posición 2 (inner_json) si coincide el rpc_id
/// Usa búsqueda directa en array vs línea por línea
pub fn extract_by_rpc_id(response: &Value, rpc_id: &str) -> Option<Value> {
    let arr = response.as_array()?;

    // Búsqueda directa en array - O(n) pero sin iterar líneas
    for item in arr {
        let item_arr = item.as_array()?;

        // item[0] = "wrb.fr" (identificador de respuesta)
        // item[1] = rpc_id (identificador de método)
        // item[2] = inner_json (payload real)

        if item_arr.first()?.as_str()? != "wrb.fr" {
            continue;
        }

        if item_arr.get(1)?.as_str()? != rpc_id {
            continue;
        }

        // Extraer inner_json como Value
        let inner_str = item_arr.get(2)?.as_str()?;
        let inner: Value = serde_json::from_str(inner_str).ok()?;

        return Some(inner);
    }

    None
}

/// Extrae un string del inner_json por índice de array validado
/// evita panic si el índice no existe
pub fn get_string_at(array: &[Value], index: usize) -> Option<String> {
    array.get(index)?.as_str().map(|s| s.to_string())
}

/// Extrae un string del inner_json por índice, con default
pub fn get_string_at_or_default(array: &[Value], index: usize, default: &str) -> String {
    get_string_at(array, index).unwrap_or_else(|| default.to_string())
}

/// Extrae un UUID (string de 36 caracteres) del inner_json por índice
pub fn get_uuid_at(array: &[Value], index: usize) -> Option<String> {
    let s = array.get(index)?.as_str()?;
    if s.len() == 36 {
        Some(s.to_string())
    } else {
        None
    }
}

/// Limpia el prefijo anti-XSSI de las respuestas HTTP de Google
/// El prefijo es: )]}'\n
/// Google batchexecute responde con MULTIPLES chunks separados por newlines.
/// Cada chunk tiene el formato: <length>\n<json>\n
/// Este parser extrae TODOS los chunks JSON y los mergea.
pub fn strip_antixssi_prefix(text: &str) -> String {
    // Buscar el primer '[' y retornar todo desde ahí
    if let Some(pos) = text.find('[') {
        let from_bracket = &text[pos..];

        // Google batchexecute format: multiple JSON chunks separated by \n<length>\n
        // e.g.:  [["wrb.fr",...]]\n25\n[["e",4,...]]\n
        // We need ALL chunks merged. Strategy: split by lines, skip length-only lines,
        // concatenate JSON lines, parse each chunk, merge into one array.
        let lines: Vec<&str> = from_bracket.lines().collect();
        let mut chunks: Vec<serde_json::Value> = Vec::new();
        let mut buffer = String::new();

        for line in &lines {
            // Skip lines that are just numbers (chunk length markers like "25", "130")
            if line.trim().chars().all(|c| c.is_ascii_digit()) {
                continue;
            }

            if buffer.is_empty() {
                buffer.push_str(line);
            } else {
                buffer.push('\n');
                buffer.push_str(line);
            }

            // Try to parse what we have so far
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&buffer) {
                chunks.push(parsed);
                buffer.clear();
            }
        }

        // If no chunks parsed, return raw text
        if chunks.is_empty() {
            return from_bracket.to_string();
        }

        // Merge all chunks: each is [[...]], we want to concatenate at top level
        let mut merged: Vec<serde_json::Value> = Vec::new();
        for chunk in &chunks {
            if let Some(arr) = chunk.as_array() {
                for item in arr {
                    merged.push(item.clone());
                }
            } else {
                merged.push(chunk.clone());
            }
        }

        serde_json::to_string(&merged).unwrap_or_else(|_| from_bracket.to_string())
    } else {
        text.to_string()
    }
}

/// Convierte un string JSON escapado en un Value parsed
pub fn parse_escaped_json(json_str: &str) -> Result<Value, String> {
    serde_json::from_str(json_str).map_err(|e| format!("Failed to parse inner JSON: {}", e))
}

/// Extrae un array del inner_json (posición 0 del wrapper doble)
pub fn extract_notebook_list(inner: &Value) -> Option<Vec<Value>> {
    let outer = inner.as_array()?;
    let first = outer.first()?;
    let array = first.as_array()?;
    Some(array.to_vec())
}

/// Extrae fuentes de un notebook desde notebook_data[1]
pub fn extract_sources(notebook_data: &[Value]) -> Option<Vec<String>> {
    let sources_elem = notebook_data.get(1)?;
    let sources_arr = sources_elem.as_array()?;

    let mut ids = Vec::new();
    for source_entry in sources_arr {
        // Formato: [[[source_id]], title, ...]
        let inner0 = source_entry.get(0)?.as_array()?;
        if let Some(sid) = inner0.first()?.as_str()
            && sid.len() == 36
        {
            ids.push(sid.to_string());
        }
    }

    Some(ids)
}

/// Find a specific source entry by source_id within notebook_data[1].
///
/// Each source entry: [[[source_id]], title, ..., [metadata, status_code, ...], ...]
/// Returns a clone of the matching source entry Value.
pub fn find_source_entry(notebook_data: &[Value], source_id: &str) -> Option<Value> {
    let sources_elem = notebook_data.get(1)?;
    let sources_arr = sources_elem.as_array()?;

    for source_entry in sources_arr {
        let inner0 = source_entry.get(0)?.as_array()?;
        if let Some(sid) = inner0.first()?.as_str()
            && sid == source_id
        {
            return Some(source_entry.clone());
        }
    }

    None
}

/// Deserializador personalizado para arrays posicionales
/// Maneja el caso donde valores nulos al final se omiten
/// NOTA: Función no crítica - removida para simplificar compilacion
#[allow(dead_code)]
pub fn deserialize_positional_array<T: Default>() -> T {
    T::default()
}

// Helper trait para valores por defecto en tipos que no implementan Default
#[allow(dead_code)]
trait OrDefault {
    fn or_default(self) -> Self;
}

impl OrDefault for Value {
    fn or_default(self) -> Self {
        Value::Null
    }
}

/// Extract a nested source ID from deeply nested array responses.
///
/// RPC `o4cbdc` (file registration) returns the source ID wrapped in
/// multiple nesting layers: `[[[[id]]]]` or sometimes `[[[id]]]`.
///
/// This function recursively unwraps single-element arrays until it finds
/// a string value, handling variable nesting depth gracefully.
pub fn extract_nested_source_id(value: &Value) -> Option<String> {
    // Base case: found a string (the ID)
    if let Some(s) = value.as_str() {
        if !s.is_empty() {
            return Some(s.to_string());
        }
        return None;
    }

    // Recursive case: unwrap single-element array
    let arr = value.as_array()?;

    // If array has exactly one element, dig deeper
    if arr.len() == 1 {
        return extract_nested_source_id(&arr[0]);
    }

    // If array has multiple elements, the ID is typically at index 0
    if !arr.is_empty() {
        return extract_nested_source_id(&arr[0]);
    }

    None
}

// =========================================================================
// Artifact Parsing — Module 2
// =========================================================================

/// A parsed artifact from the LIST_ARTIFACTS response.
///
/// Maps to the positional array format discovered via reverse engineering:
/// ```text
/// art[0]  = artifact_id (string)
/// art[1]  = title (string)
/// art[2]  = type_code (int: ArtifactTypeCode)
/// art[3]  = error_reason (string, only when failed)
/// art[4]  = status_code (int: ArtifactStatus)
/// art[5]  = nested error payload (list, only when failed)
/// art[6]  = audio metadata
/// art[7]  = report content (markdown at [7][0])
/// art[8]  = video metadata
/// art[9]  = options ([[variant_code], ...] — variant at [9][1][0])
/// art[15] = timestamps ([[unix_ts], ...])
/// art[16] = slide deck metadata
/// art[18] = data table content
/// ```
#[derive(Debug, Clone)]
pub struct Artifact {
    pub id: String,
    pub title: String,
    pub kind: ArtifactType,
    pub status: ArtifactStatus,
    pub raw_data: Value,
}

impl Artifact {
    /// Parse a single artifact from its raw positional array.
    ///
    /// Defensively extracts each field, defaulting to safe values if any
    /// position is missing or malformed.
    pub fn from_api_response(art: &Value) -> Option<Self> {
        let arr = art.as_array()?;

        // art[0] = artifact_id
        let id = arr.first()?.as_str()?.to_string();
        if id.is_empty() {
            return None;
        }

        // art[1] = title
        let title = arr
            .get(1)
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();

        // art[2] = type_code (int)
        let type_code_int = arr.get(2)?.as_i64()?;
        let type_code = ArtifactTypeCode::from_code(type_code_int as i32)?;

        // art[9][1][0] = variant (for Quiz/Flashcards distinction)
        let variant = arr
            .get(9)
            .and_then(|v| v.as_array())
            .and_then(|v| v.get(1))
            .and_then(|v| v.as_array())
            .and_then(|v| v.first())
            .and_then(|v| v.as_i64());

        let kind = ArtifactType::from_type_code_and_variant(type_code, variant.map(|v| v as i32));

        // art[4] = status_code (int)
        let status = arr
            .get(4)
            .and_then(|v| v.as_i64())
            .and_then(|v| ArtifactStatus::from_code(v as i32))
            .unwrap_or(ArtifactStatus::Processing);

        Some(Artifact {
            id,
            title,
            kind,
            status,
            raw_data: art.clone(),
        })
    }

    /// Check if this artifact matches a given task_id.
    /// In NotebookLM, task_id == artifact_id.
    pub fn matches_task_id(&self, task_id: &str) -> bool {
        self.id == task_id
    }

    /// Check if this artifact is completed (status == COMPLETED).
    pub fn is_completed(&self) -> bool {
        self.status == ArtifactStatus::Completed
    }

    /// Check if this artifact failed.
    pub fn is_failed(&self) -> bool {
        self.status == ArtifactStatus::Failed
    }
}

/// Parse the LIST_ARTIFACTS RPC response into a list of typed `Artifact` structs.
///
/// The response format after `extract_by_rpc_id` is:
/// ```text
/// [[art1_array], [art2_array], ...]
/// ```
///
/// Each artifact is a positional array. This function iterates and parses
/// each one, skipping entries that don't match the expected format.
pub fn parse_artifact_list(inner: &Value) -> Vec<Artifact> {
    let mut artifacts = Vec::new();

    // The inner JSON from LIST_ARTIFACTS is an array of artifact arrays
    let outer = match inner.as_array() {
        Some(arr) => arr,
        None => return artifacts,
    };

    for item in outer {
        // Each item could be:
        // 1. A direct artifact array: [id, title, type_code, ...]
        // 2. A nested wrapper: [[id, title, ...], ...]
        let art_val = if item.is_array() {
            // Check if this is a direct artifact (has a string at index 0 = artifact_id)
            if item.as_array().unwrap().first().is_some_and(|v| v.is_string()) {
                item
            } else {
                // Might be nested — try first element
                item.as_array().unwrap().first().map_or(item, |v| v)
            }
        } else {
            item
        };

        if let Some(artifact) = Artifact::from_api_response(art_val) {
            artifacts.push(artifact);
        }
    }

    artifacts
}

/// Parse the CREATE_ARTIFACT response to extract task_id and initial status.
///
/// Response format:
/// ```text
/// result[0][0] = artifact_id (also used as task_id)
/// result[0][4] = initial status code (ArtifactStatus)
/// ```
pub fn parse_generation_result(inner: &Value) -> Option<GenerationStatus> {
    let arr = inner.as_array()?;
    let first = arr.first()?.as_array()?;

    // result[0][0] = artifact_id / task_id
    let task_id = first.first()?.as_str()?.to_string();
    if task_id.is_empty() {
        return None;
    }

    // result[0][4] = initial status
    let status = first
        .get(4)
        .and_then(|v| v.as_i64())
        .and_then(|v| ArtifactStatus::from_code(v as i32))
        .unwrap_or(ArtifactStatus::Processing);

    Some(GenerationStatus::new(task_id, status))
}

// =========================================================================
// URL Extraction — Phase 6.1
// =========================================================================

/// Extract audio download URL from artifact raw data.
///
/// Audio URL location: `art[6][5]` — media URL list.
/// Each item: `[url, quality?, "audio/mp4"]`.
/// Prefers item where `item[2] == "audio/mp4"`, fallback to `media_list[0][0]`.
///
/// Reference: teng-lin/notebooklm-py `_artifacts.py` audio download
pub fn extract_audio_url(artifact_data: &Value) -> Option<String> {
    let arr = artifact_data.as_array()?;
    let media_list = arr.get(6)?.as_array()?.get(5)?.as_array()?;
    if media_list.is_empty() {
        return None;
    }

    // First pass: look for audio/mp4 mime type
    for item in media_list {
        let item_arr = item.as_array()?;
        if item_arr.len() > 2
            && item_arr.get(2).and_then(|v| v.as_str()) == Some("audio/mp4")
                && let Some(url) = item_arr.first().and_then(|v| v.as_str())
                    && !url.is_empty() {
                        return Some(url.to_string());
                    }
    }

    // Fallback: first item's first element
    media_list
        .first()
        .and_then(|v| v.as_array())
        .and_then(|v| v.first())
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// Extract video download URL from artifact raw data.
///
/// Video URL location: `art[8]` — array of media groups.
/// First, find the group whose `item[0][0]` starts with "http".
/// Then iterate items looking for `"video/mp4"`, preferring `quality=4` (highest).
///
/// Each media item: `[url, quality, "video/mp4"]`.
/// Quality 4 = highest quality. Falls back to last video/mp4 URL found.
///
/// Reference: teng-lin/notebooklm-py `_artifacts.py` video download
pub fn extract_video_url(artifact_data: &Value) -> Option<String> {
    let arr = artifact_data.as_array()?;
    let metadata = arr.get(8)?.as_array()?;
    if metadata.is_empty() {
        return None;
    }

    // Find the media list: first group whose first item's first element starts with "http"
    let media_list = metadata.iter().find_map(|group| {
        let group_arr = group.as_array()?;
        let first_item = group_arr.first()?.as_array()?;
        let url = first_item.first()?.as_str()?;
        if url.starts_with("http") {
            Some(group_arr)
        } else {
            None
        }
    })?;

    // Iterate: prefer quality=4 (highest), fallback to any video/mp4
    let mut fallback_url: Option<String> = None;
    for item in media_list {
        let item_arr = item.as_array()?;
        if item_arr.len() > 2 {
            let mime = item_arr.get(2).and_then(|v| v.as_str());
            let url = item_arr.first().and_then(|v| v.as_str());
            let quality = item_arr.get(1).and_then(|v| v.as_i64());

            if mime == Some("video/mp4")
                && let Some(url) = url
                    && !url.is_empty() {
                        if quality == Some(4) {
                            return Some(url.to_string());
                        }
                        fallback_url = Some(url.to_string());
                    }
        }
    }

    fallback_url
}

/// Extract infographic image URL from artifact raw data.
///
/// Infographic URLs are deeply nested with no fixed position.
/// Forward-scans all top-level items looking for the pattern:
/// `item[2][0][1][0]` = URL string starting with "http".
///
/// Forward iteration (not reversed) prefers lower indices = canonical content URL.
/// Filters out notebooklm.google.com URLs (those are API URLs, not CDN).
///
/// Reference: teng-lin/notebooklm-py `_artifacts.py` `_find_infographic_url`
pub fn extract_infographic_url(artifact_data: &Value) -> Option<String> {
    let arr = artifact_data.as_array()?;

    for item in arr {
        // Use `continue` not `?` — non-array items (strings, numbers, nulls)
        // are normal in the artifact array and should be skipped, not cause early return
        let Some(item_arr) = item.as_array() else { continue };
        if item_arr.len() <= 2 {
            continue;
        }

        let Some(content) = item_arr.get(2).and_then(|v| v.as_array()) else { continue };
        if content.is_empty() {
            continue;
        }

        let Some(first_content) = content.first().and_then(|v| v.as_array()) else { continue };
        if first_content.len() <= 1 {
            continue;
        }

        let Some(img_data) = first_content.get(1).and_then(|v| v.as_array()) else { continue };
        if img_data.is_empty() {
            continue;
        }

        if let Some(url) = img_data.first().and_then(|v| v.as_str())
            && url.starts_with("http") && !url.contains("notebooklm.google.com") {
                return Some(url.to_string());
            }
    }

    None
}

/// Extract slide deck download URL from artifact raw data.
///
/// Slide deck metadata at `art[16]`: `[config, title, slides, pdf_url, pptx_url]`.
/// - `art[16][3]` = PDF download URL
/// - `art[16][4]` = PPTX download URL
///
/// # Arguments
/// * `artifact_data` - Raw artifact array from LIST_ARTIFACTS
/// * `format` - Download format: "pdf" (default) or "pptx"
///
/// Reference: teng-lin/notebooklm-py `_artifacts.py` slide deck download
pub fn extract_slide_deck_url(artifact_data: &Value, format: &str) -> Option<String> {
    let arr = artifact_data.as_array()?;
    let metadata = arr.get(16)?.as_array()?;

    let index: usize = match format.to_lowercase().as_str() {
        "pptx" => 4,
        _ => 3, // default to PDF
    };

    let url = metadata.get(index)?.as_str()?;
    if url.is_empty() || !url.starts_with("http") {
        return None;
    }

    Some(url.to_string())
}

// =========================================================================
// Inline Content Extraction — Phase 6.4 & 6.5
// =========================================================================

/// Extract report markdown content from artifact raw data.
///
/// Report content lives at `art[7]` — either a string (markdown directly)
/// or a 1-element list `["markdown string"]` that needs unwrapping.
///
/// # Arguments
/// * `artifact_data` - Raw artifact array from LIST_ARTIFACTS
///
/// # Returns
///
/// The markdown string, or None if the structure is invalid.
///
/// Reference: teng-lin/notebooklm-py `_artifacts.py` `download_report`
pub fn extract_report_content(artifact_data: &Value) -> Option<String> {
    let wrapper = artifact_data.as_array()?.get(7)?;

    let content = if let Some(arr) = wrapper.as_array() {
        // wrapper is a list → unwrap first element
        if arr.is_empty() {
            return None;
        }
        arr.first()?.as_str()?
    } else {
        // wrapper is already a string
        wrapper.as_str()?
    };

    if content.is_empty() {
        return None;
    }

    Some(content.to_string())
}

/// Recursively extract text from a nested data table cell.
///
/// Data table cells have deeply nested arrays with position markers
/// (integers) and text content (strings). This function traverses the
/// structure and concatenates all text fragments found.
///
/// - Strings → kept as-is
/// - Integers → skipped (position markers)
/// - Arrays → recurse into children, concatenate results
/// - Other types → empty string
///
/// Reference: teng-lin/notebooklm-py `_artifacts.py` `_extract_cell_text`
pub fn extract_cell_text(cell: &Value) -> String {
    match cell {
        Value::String(s) => s.clone(),
        Value::Number(_) => String::new(), // position markers
        Value::Array(arr) => arr
            .iter()
            .map(extract_cell_text)
            .collect(),
        _ => String::new(),
    }
}

/// Parse a data table artifact into headers and rows for CSV output.
///
/// The data table structure is deeply nested:
/// ```text
/// artifact[18]            — raw table data
///   [0][0][0][0]          — 4 wrapper layers
///   [4]                   — table content section [type, flags, rows_array]
///   [2]                   — the actual rows array
/// ```
///
/// Each row: `[start_pos, end_pos, cell_array]`
/// Row 0 → headers, rows 1+ → data rows.
/// Each cell is deeply nested and needs recursive text extraction.
///
/// # Arguments
/// * `artifact_data` - Raw artifact array from LIST_ARTIFACTS
///
/// # Returns
///
/// `(headers, rows)` where headers is a Vec of column names and rows
/// is a Vec of Vecs of cell text strings.
///
/// Reference: teng-lin/notebooklm-py `_artifacts.py` `_parse_data_table`
pub fn parse_data_table(artifact_data: &Value) -> Option<(Vec<String>, Vec<Vec<String>>)> {
    let raw_data = artifact_data.as_array()?.get(18)?.as_array()?;

    // Navigate: [0][0][0][0][4][2] = rows_array
    let rows_array = raw_data.first()?.as_array()?.first()?.as_array()?.first()?.as_array()?.first()?.as_array()?
        .get(4)?.as_array()?
        .get(2)?.as_array()?;

    if rows_array.is_empty() {
        return None;
    }

    let mut headers = Vec::new();
    let mut rows = Vec::new();

    for (i, row_section) in rows_array.iter().enumerate() {
        let row_arr = row_section.as_array()?;
        if row_arr.len() < 3 {
            continue;
        }

        let cell_array = row_arr.get(2)?.as_array()?;
        let row_values: Vec<String> = cell_array
            .iter()
            .map(extract_cell_text)
            .collect();

        if i == 0 {
            headers = row_values;
        } else {
            rows.push(row_values);
        }
    }

    if headers.is_empty() {
        return None;
    }

    Some((headers, rows))
}

// =========================================================================
// Quiz / Flashcard Content Extraction — Phase 6.6
// =========================================================================

/// Extract JSON from `data-app-data` HTML attribute.
///
/// Quiz and flashcard HTML embeds structured JSON in a `data-app-data`
/// attribute with HTML-encoded content (e.g., `&quot;` for quotes).
///
/// Flow: regex search → HTML unescape → JSON parse → return Value.
///
/// # Arguments
/// * `html_content` - Raw HTML string from the interactive artifact endpoint
///
/// # Errors
///
/// Returns `None` if no `data-app-data` attribute is found or JSON is invalid.
///
/// Reference: teng-lin/notebooklm-py `_artifacts.py` `_extract_app_data`
pub fn extract_app_data(html_content: &str) -> Option<Value> {
    use regex::Regex;

    let re = Regex::new(r#"data-app-data="([^"]+)""#).ok()?;
    let caps = re.captures(html_content)?;
    let encoded_json = caps.get(1)?.as_str();

    // Decode HTML entities: &quot; → ", &amp; → &, &lt; → <, &gt; → >
    let decoded = html_escape::decode_html_entities(encoded_json);

    serde_json::from_str(&decoded).ok()
}

/// Format quiz questions as markdown.
///
/// # Arguments
/// * `title` - Quiz title
/// * `questions` - Array of question objects from `app_data["quiz"]`
///
/// # Output format
///
/// ```markdown
/// # Quiz Title
///
/// ## Question 1
/// What is X?
///
/// - [x] Option A (correct)
/// - [ ] Option B
/// - [ ] Option C
///
/// **Hint:** Optional hint
/// ```
///
/// Reference: teng-lin/notebooklm-py `_artifacts.py` `_format_quiz_markdown`
pub fn format_quiz_markdown(title: &str, questions: &[Value]) -> String {
    let mut lines = vec![format!("# {}", title), String::new()];

    for (i, q) in questions.iter().enumerate() {
        let question_text = q.get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("(no question text)");

        lines.push(format!("## Question {}", i + 1));
        lines.push(question_text.to_string());
        lines.push(String::new());

        if let Some(options) = q.get("answerOptions").and_then(|v| v.as_array()) {
            for opt in options {
                let text = opt.get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let is_correct = opt.get("isCorrect")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let marker = if is_correct { "[x]" } else { "[ ]" };
                lines.push(format!("- {} {}", marker, text));
            }
        }

        if let Some(hint) = q.get("hint").and_then(|v| v.as_str())
            && !hint.is_empty() {
                lines.push(String::new());
                lines.push(format!("**Hint:** {}", hint));
            }

        lines.push(String::new());
    }

    lines.join("\n")
}

/// Format flashcards as markdown.
///
/// # Arguments
/// * `title` - Flashcard deck title
/// * `cards` - Array of card objects from `app_data["flashcards"]`
///
/// # Output format
///
/// ```markdown
/// # Flashcard Title
///
/// ## Card 1
///
/// **Q:** Front text
///
/// **A:** Back text
///
/// ---
/// ```
///
/// Reference: teng-lin/notebooklm-py `_artifacts.py` `_format_flashcards_markdown`
pub fn format_flashcards_markdown(title: &str, cards: &[Value]) -> String {
    let mut lines = vec![format!("# {}", title), String::new()];

    for (i, card) in cards.iter().enumerate() {
        let front = card.get("f").and_then(|v| v.as_str()).unwrap_or("");
        let back = card.get("b").and_then(|v| v.as_str()).unwrap_or("");

        lines.push(format!("## Card {}", i + 1));
        lines.push(String::new());
        lines.push(format!("**Q:** {}", front));
        lines.push(String::new());
        lines.push(format!("**A:** {}", back));
        lines.push(String::new());
        lines.push("---".to_string());
        lines.push(String::new());
    }

    lines.join("\n")
}

// =========================================================================
// Mind Map Content Extraction — Phase 6.7
// =========================================================================

/// Check if a note item is a mind map.
///
/// Mind maps are identified by their content containing `"children":` or `"nodes":`.
/// The content can be in two formats:
/// - Old format: `item[1]` is a string directly
/// - New format: `item[1]` is an array, `item[1][1]` is the string
///
/// Reference: teng-lin/notebooklm-py `_notes.py` `_is_mind_map`
pub fn is_mind_map_item(item: &Value) -> bool {
    let content = extract_note_content(item);
    match content {
        Some(s) => s.contains(r#""children":"#) || s.contains(r#""nodes":"#),
        None => false,
    }
}

/// Extract note content string from a note item.
///
/// Supports two formats:
/// - Old format: `item[1]` is a string
/// - New format: `item[1]` is an array, `item[1][1]` is the string
///
/// Reference: teng-lin/notebooklm-py `_notes.py` `_extract_content`
pub fn extract_note_content(item: &Value) -> Option<String> {
    let field = item.get(1)?;

    // Old format: field is a string directly
    if let Some(s) = field.as_str() {
        return Some(s.to_string());
    }

    // New format: field is an array, content at index 1
    if let Some(arr) = field.as_array()
        && arr.len() > 1
            && let Some(s) = arr.get(1).and_then(|v| v.as_str()) {
                return Some(s.to_string());
            }

    None
}

/// Extract mind map JSON from a note item.
///
/// Parses the JSON string embedded in the note content.
///
/// # Arguments
/// * `item` - A note item from the GET_NOTES_AND_MIND_MAPS response
///
/// # Returns
///
/// The parsed JSON Value, or None if the content is not valid JSON.
pub fn extract_mind_map_json(item: &Value) -> Option<Value> {
    let content = extract_note_content(item)?;
    serde_json::from_str(&content).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_antixssi_prefix_with_newline() {
        let input = ")]}'\n[[\"wrb.fr\", \"test\", \"{}\"]]";
        let result = strip_antixssi_prefix(input);
        assert!(result.starts_with("["));
    }

    #[test]
    fn test_strip_antixssi_prefix_without_prefix() {
        let input = "[[\"wrb.fr\", \"test\", \"{}\"]]";
        let result = strip_antixssi_prefix(input);
        assert!(result.starts_with("["));
    }

    #[test]
    fn test_strip_antixssi_prefix_finds_first_bracket() {
        let input = "garbage\nrandom\n[[\"wrb.fr\"";
        let result = strip_antixssi_prefix(input);
        assert!(result.starts_with("[["));
    }

    #[test]
    fn test_get_string_at_valid_index() {
        let arr = vec![
            Value::String("hello".to_string()),
            Value::Null,
            Value::String("world".to_string()),
        ];
        assert_eq!(get_string_at(&arr, 0), Some("hello".to_string()));
        assert_eq!(get_string_at(&arr, 1), None);
        assert_eq!(get_string_at(&arr, 5), None);
    }

    #[test]
    fn test_get_uuid_at_valid() {
        let arr = vec![Value::String(
            "550e8400-e29b-41d4-a716-446655440000".to_string(),
        )];
        assert_eq!(
            get_uuid_at(&arr, 0),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn test_get_uuid_at_invalid_length() {
        let arr = vec![Value::String("not-a-uuid".to_string())];
        assert_eq!(get_uuid_at(&arr, 0), None);
    }

    #[test]
    fn test_extract_by_rpc_id() {
        let response: Value =
            serde_json::from_str(r#"[["wrb.fr", "testRpc", "{\"key\": \"value\"}"]]"#).unwrap();
        let result = extract_by_rpc_id(&response, "testRpc");
        assert!(result.is_some());
        let inner = result.unwrap();
        assert_eq!(inner.get("key").and_then(|v| v.as_str()), Some("value"));
    }

    #[test]
    fn test_extract_by_rpc_id_not_found() {
        let response: Value = serde_json::from_str(r#"[["wrb.fr", "testRpc", "{}"]]"#).unwrap();
        let result = extract_by_rpc_id(&response, "otherRpc");
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_list_notebooks_response() {
        // Este test verifica el parsing del inner JSON
        // El formato real requiere double-encoding del payload RPC
        // Test simplificado: verificar que extract_notebook_list funciona con arrays
        let inner_json: Value = serde_json::from_str(
            r#"[["Test Notebook", [], "nb-uuid-001-1234-5678-9abc-def0-123456789abc", null, null, []]]"#
        ).unwrap();

        let notebooks = extract_notebook_list(&inner_json);
        assert!(notebooks.is_some(), "Should extract notebook list");

        let nb_list = notebooks.unwrap();
        assert!(!nb_list.is_empty(), "Should have at least one notebook");
    }

    #[test]
    #[ignore = "Test requires double-encoded JSON format matching real RPC responses"]
    fn test_extract_sources_real_format() {
        // Este test requiere el formato exacto de la API:
        // [title, [[source_id, title], ...]]
        // Marcado como ignore hasta tener fixture real
    }

    #[test]
    fn test_extract_sources_from_notebook_data() {
        // Este test verifica el parsing básico de arrays
        // El formato real de fuentes requiere análisis más detallado de la API
        let notebook_data: Vec<Value> = vec![
            Value::String("Test Notebook".to_string()),
            Value::Null, // Fuentes no disponibles
        ];

        let sources = extract_sources(&notebook_data);
        // Cuando el segundo elemento es null, el parsing debe manejarlo
        // Verificar que no hay panic
        assert!(sources.is_none() || sources.unwrap().is_empty());
    }

    #[test]
    fn test_strip_antixssi_from_google_response() {
        // Simular respuesta real de Google con prefijo anti-XSSI
        let input = ")]}'\n234\n[[\"wrb.fr\",null,\"data\"]]";
        let cleaned = strip_antixssi_prefix(input);
        assert!(cleaned.starts_with("[["), "Should start with array");
        assert!(
            !cleaned.starts_with(")]}'"),
            "Should not have anti-XSSI prefix"
        );
    }

    // =========================================================================
    // extract_nested_source_id tests
    // =========================================================================

    #[test]
    fn test_nested_source_id_4_levels() {
        let val = serde_json::json!([[[["550e8400-e29b-41d4-a716-446655440000"]]]]);
        assert_eq!(
            extract_nested_source_id(&val),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn test_nested_source_id_3_levels() {
        let val = serde_json::json!([[["550e8400-e29b-41d4-a716-446655440000"]]]);
        assert_eq!(
            extract_nested_source_id(&val),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn test_nested_source_id_2_levels() {
        let val = serde_json::json!([["550e8400-e29b-41d4-a716-446655440000"]]);
        assert_eq!(
            extract_nested_source_id(&val),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn test_nested_source_id_1_level() {
        let val = serde_json::json!(["550e8400-e29b-41d4-a716-446655440000"]);
        assert_eq!(
            extract_nested_source_id(&val),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn test_nested_source_id_bare_string() {
        let val = serde_json::json!("550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(
            extract_nested_source_id(&val),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    #[test]
    fn test_nested_source_id_null() {
        let val = serde_json::Value::Null;
        assert_eq!(extract_nested_source_id(&val), None);
    }

    #[test]
    fn test_nested_source_id_empty_string() {
        let val = serde_json::json!("".to_string());
        assert_eq!(extract_nested_source_id(&val), None);
    }

    #[test]
    fn test_nested_source_id_empty_array() {
        let val = serde_json::json!([]);
        assert_eq!(extract_nested_source_id(&val), None);
    }

    #[test]
    fn test_nested_source_id_multi_element_array() {
        // Array with multiple elements — should try index 0
        let val = serde_json::json!(["550e8400-e29b-41d4-a716-446655440000", "other"]);
        assert_eq!(
            extract_nested_source_id(&val),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }

    // =========================================================================
    // Artifact parsing tests — Module 2
    // =========================================================================

    fn make_artifact_array(type_code: i32, status_code: i32, variant: Option<i32>) -> Value {
        // Build a minimal artifact array with the critical positions
        let mut arr = vec![
            Value::String("art-abc-123-def-456".to_string()), // [0] artifact_id
            Value::String("My Artifact Title".to_string()),   // [1] title
            Value::from(type_code),                           // [2] type_code
            Value::Null,                                      // [3] error_reason
            Value::from(status_code),                         // [4] status_code
        ];
        // Pad up to index 9 for variant
        while arr.len() < 10 {
            arr.push(Value::Null);
        }
        // [9] = options: the variant is at [9][1][0]
        // Real format: [[...], [variant_code], ...]
        if let Some(v) = variant {
            arr[9] = serde_json::json!([null, [v]]);
        }
        Value::Array(arr)
    }

    #[test]
    fn test_artifact_from_api_response_audio() {
        let art = make_artifact_array(1, 3, None); // Audio, Completed
        let artifact = Artifact::from_api_response(&art).unwrap();
        assert_eq!(artifact.id, "art-abc-123-def-456");
        assert_eq!(artifact.title, "My Artifact Title");
        assert_eq!(artifact.kind, ArtifactType::Audio);
        assert_eq!(artifact.status, ArtifactStatus::Completed);
        assert!(artifact.is_completed());
        assert!(!artifact.is_failed());
    }

    #[test]
    fn test_artifact_from_api_response_video() {
        let art = make_artifact_array(3, 1, None); // Video, Processing
        let artifact = Artifact::from_api_response(&art).unwrap();
        assert_eq!(artifact.kind, ArtifactType::Video);
        assert_eq!(artifact.status, ArtifactStatus::Processing);
        assert!(!artifact.is_completed());
    }

    #[test]
    fn test_artifact_from_api_response_quiz_variant() {
        let art = make_artifact_array(4, 3, Some(2)); // QuizFlashcards, Completed, variant=Quiz
        let artifact = Artifact::from_api_response(&art).unwrap();
        assert_eq!(artifact.kind, ArtifactType::Quiz);
        assert!(artifact.is_completed());
    }

    #[test]
    fn test_artifact_from_api_response_flashcards_variant() {
        let art = make_artifact_array(4, 3, Some(1)); // QuizFlashcards, Completed, variant=Flashcards
        let artifact = Artifact::from_api_response(&art).unwrap();
        assert_eq!(artifact.kind, ArtifactType::Flashcards);
    }

    #[test]
    fn test_artifact_from_api_response_infographic() {
        let art = make_artifact_array(7, 3, None); // Infographic, Completed
        let artifact = Artifact::from_api_response(&art).unwrap();
        assert_eq!(artifact.kind, ArtifactType::Infographic);
    }

    #[test]
    fn test_artifact_from_api_response_slide_deck() {
        let art = make_artifact_array(8, 3, None); // SlideDeck, Completed
        let artifact = Artifact::from_api_response(&art).unwrap();
        assert_eq!(artifact.kind, ArtifactType::SlideDeck);
    }

    #[test]
    fn test_artifact_from_api_response_data_table() {
        let art = make_artifact_array(9, 3, None); // DataTable, Completed
        let artifact = Artifact::from_api_response(&art).unwrap();
        assert_eq!(artifact.kind, ArtifactType::DataTable);
    }

    #[test]
    fn test_artifact_from_api_response_mind_map() {
        let art = make_artifact_array(5, 3, None); // MindMap, Completed
        let artifact = Artifact::from_api_response(&art).unwrap();
        assert_eq!(artifact.kind, ArtifactType::MindMap);
    }

    #[test]
    fn test_artifact_from_api_response_failed() {
        let art = make_artifact_array(1, 4, None); // Audio, Failed
        let artifact = Artifact::from_api_response(&art).unwrap();
        assert_eq!(artifact.status, ArtifactStatus::Failed);
        assert!(artifact.is_failed());
    }

    #[test]
    fn test_artifact_from_api_response_invalid_type_code() {
        let art = make_artifact_array(99, 3, None); // Invalid type
        assert!(Artifact::from_api_response(&art).is_none());
    }

    #[test]
    fn test_artifact_from_api_response_empty_id() {
        let art = serde_json::json!([
            "",    // empty id
            "Title",
            1,
            null,
            3,
        ]);
        assert!(Artifact::from_api_response(&art).is_none());
    }

    #[test]
    fn test_artifact_from_api_response_not_array() {
        assert!(Artifact::from_api_response(&Value::Null).is_none());
        assert!(Artifact::from_api_response(&serde_json::json!("string")).is_none());
    }

    #[test]
    fn test_artifact_matches_task_id() {
        let art = make_artifact_array(1, 3, None);
        let artifact = Artifact::from_api_response(&art).unwrap();
        assert!(artifact.matches_task_id("art-abc-123-def-456"));
        assert!(!artifact.matches_task_id("different-id"));
    }

    #[test]
    fn test_parse_artifact_list_multiple() {
        let inner = serde_json::json!([
            make_artifact_array(1, 3, None),  // Audio, Completed
            make_artifact_array(3, 1, None),  // Video, Processing
            make_artifact_array(2, 3, None),  // Report, Completed
        ]);
        let artifacts = parse_artifact_list(&inner);
        assert_eq!(artifacts.len(), 3);
        assert_eq!(artifacts[0].kind, ArtifactType::Audio);
        assert_eq!(artifacts[1].kind, ArtifactType::Video);
        assert_eq!(artifacts[2].kind, ArtifactType::Report);
    }

    #[test]
    fn test_parse_artifact_list_empty() {
        let artifacts = parse_artifact_list(&serde_json::json!([]));
        assert!(artifacts.is_empty());
    }

    #[test]
    fn test_parse_artifact_list_non_array() {
        let artifacts = parse_artifact_list(&Value::Null);
        assert!(artifacts.is_empty());
    }

    #[test]
    fn test_parse_artifact_list_skips_invalid() {
        let inner = serde_json::json!([
            make_artifact_array(1, 3, None),  // valid
            "not an array",                   // invalid — skipped
            make_artifact_array(99, 3, None), // invalid type code — skipped
            make_artifact_array(2, 3, None),  // valid
        ]);
        let artifacts = parse_artifact_list(&inner);
        assert_eq!(artifacts.len(), 2);
        assert_eq!(artifacts[0].kind, ArtifactType::Audio);
        assert_eq!(artifacts[1].kind, ArtifactType::Report);
    }

    #[test]
    fn test_parse_generation_result() {
        // result[0][0] = task_id, result[0][4] = status
        let inner = serde_json::json!([
            ["art-task-123", null, null, null, 1]  // Processing
        ]);
        let result = parse_generation_result(&inner);
        assert!(result.is_some());
        let status = result.unwrap();
        assert_eq!(status.task_id, "art-task-123");
        assert_eq!(status.status, ArtifactStatus::Processing);
        assert!(status.is_in_progress());
    }

    #[test]
    fn test_parse_generation_result_completed() {
        let inner = serde_json::json!([
            ["art-done-456", null, null, null, 3]  // Completed
        ]);
        let result = parse_generation_result(&inner).unwrap();
        assert_eq!(result.task_id, "art-done-456");
        assert_eq!(result.status, ArtifactStatus::Completed);
        assert!(result.is_complete());
    }

    #[test]
    fn test_parse_generation_result_empty_id() {
        let inner = serde_json::json!([["", null, null, null, 1]]);
        assert!(parse_generation_result(&inner).is_none());
    }

    #[test]
    fn test_parse_generation_result_non_array() {
        assert!(parse_generation_result(&Value::Null).is_none());
    }

    // =========================================================================
    // URL extraction tests — Task 6.1
    // =========================================================================

    // --- extract_audio_url ---

    #[test]
    fn test_extract_audio_url_with_mp4_mime() {
        // art[6][5] = media list with audio/mp4 entry
        let raw = serde_json::json!([
            "audio-id", "Audio Title", 1, null, 3,
            null,
            [
                null, null, null, null, null,
                [
                    ["https://cdn.google.com/audio_lo.mp4", 2, "audio/mp4"],
                    ["https://cdn.google.com/audio_hi.mp4", 4, "audio/mp4"],
                ]
            ]
        ]);
        let url = extract_audio_url(&raw);
        assert!(url.is_some());
        // Returns FIRST audio/mp4 match
        assert_eq!(url.unwrap(), "https://cdn.google.com/audio_lo.mp4");
    }

    #[test]
    fn test_extract_audio_url_fallback_first_item() {
        // art[6][5] = media list with NO audio/mp4 match → fallback to first item
        let raw = serde_json::json!([
            "audio-id", "Audio Title", 1, null, 3,
            null,
            [
                null, null, null, null, null,
                [
                    ["https://cdn.google.com/audio.mp4", 4, "audio/mpeg"],
                ]
            ]
        ]);
        let url = extract_audio_url(&raw);
        assert_eq!(url, Some("https://cdn.google.com/audio.mp4".to_string()));
    }

    #[test]
    fn test_extract_audio_url_empty_media_list() {
        // art[6][5] = empty list
        let raw = serde_json::json!([
            "audio-id", "Audio Title", 1, null, 3,
            null,
            [null, null, null, null, null, []]
        ]);
        assert!(extract_audio_url(&raw).is_none());
    }

    #[test]
    fn test_extract_audio_url_missing_metadata() {
        // art[6] = null (not yet populated)
        let raw = serde_json::json!(["audio-id", "Audio Title", 1, null, 3, null, null]);
        assert!(extract_audio_url(&raw).is_none());
    }

    #[test]
    fn test_extract_audio_url_empty_string_url() {
        // URL is empty string → should not return it
        let raw = serde_json::json!([
            "audio-id", "Audio Title", 1, null, 3,
            null,
            [
                null, null, null, null, null,
                [["", 4, "audio/mp4"]]
            ]
        ]);
        assert!(extract_audio_url(&raw).is_none());
    }

    #[test]
    fn test_extract_audio_url_non_array_data() {
        assert!(extract_audio_url(&Value::Null).is_none());
        assert!(extract_audio_url(&serde_json::json!("string")).is_none());
    }

    #[test]
    fn test_extract_audio_url_item_not_array() {
        // art[6][5] has non-array items
        let raw = serde_json::json!([
            "audio-id", "Audio Title", 1, null, 3,
            null,
            [null, null, null, null, null, ["not-an-array", "audio/mp4"]]
        ]);
        assert!(extract_audio_url(&raw).is_none());
    }

    // --- extract_video_url ---

    #[test]
    fn test_extract_video_url_quality_4() {
        // art[8] = media groups, first group has quality=4 entry
        let raw = serde_json::json!([
            "video-id", "Video Title", 3, null, 3,
            null, null, null,
            [
                // media group: first item is [url, ...] with http URL
                [
                    ["https://cdn.google.com/video_lo.mp4", 2, "video/mp4"],
                    ["https://cdn.google.com/video_hi.mp4", 4, "video/mp4"],
                ]
            ]
        ]);
        let url = extract_video_url(&raw);
        assert_eq!(url, Some("https://cdn.google.com/video_hi.mp4".to_string()));
    }

    #[test]
    fn test_extract_video_url_fallback_lower_quality() {
        // Only quality=2 available → returns it as fallback
        let raw = serde_json::json!([
            "video-id", "Video Title", 3, null, 3,
            null, null, null,
            [
                [
                    ["https://cdn.google.com/video_2.mp4", 2, "video/mp4"],
                ]
            ]
        ]);
        let url = extract_video_url(&raw);
        assert_eq!(url, Some("https://cdn.google.com/video_2.mp4".to_string()));
    }

    #[test]
    fn test_extract_video_url_prefers_quality_4_over_lower() {
        // Multiple entries, quality=4 is second → should still find it
        let raw = serde_json::json!([
            "video-id", "Video Title", 3, null, 3,
            null, null, null,
            [
                [
                    ["https://cdn.google.com/video_1.mp4", 1, "video/mp4"],
                    ["https://cdn.google.com/video_2.mp4", 2, "video/mp4"],
                    ["https://cdn.google.com/video_4.mp4", 4, "video/mp4"],
                ]
            ]
        ]);
        let url = extract_video_url(&raw);
        assert_eq!(url, Some("https://cdn.google.com/video_4.mp4".to_string()));
    }

    #[test]
    fn test_extract_video_url_empty_metadata() {
        let raw = serde_json::json!(["video-id", "Video Title", 3, null, 3, null, null, null, []]);
        assert!(extract_video_url(&raw).is_none());
    }

    #[test]
    fn test_extract_video_url_no_http_group() {
        // Groups exist but none have HTTP URLs
        let raw = serde_json::json!([
            "video-id", "Video Title", 3, null, 3,
            null, null, null,
            [
                [["not-a-url", 4, "video/mp4"]],
            ]
        ]);
        assert!(extract_video_url(&raw).is_none());
    }

    #[test]
    fn test_extract_video_url_missing_metadata() {
        // art[8] not present (array too short)
        let raw = serde_json::json!(["video-id", "Video Title", 3, null, 3]);
        assert!(extract_video_url(&raw).is_none());
    }

    #[test]
    fn test_extract_video_url_non_video_mime_ignored() {
        // Group has entries but none are video/mp4
        let raw = serde_json::json!([
            "video-id", "Video Title", 3, null, 3,
            null, null, null,
            [
                [
                    ["https://cdn.google.com/video.mp4", 4, "audio/mp4"],
                    ["https://cdn.google.com/thumb.jpg", 4, "image/jpeg"],
                ]
            ]
        ]);
        assert!(extract_video_url(&raw).is_none());
    }

    // --- extract_infographic_url ---

    #[test]
    fn test_extract_infographic_url_valid() {
        // Nested pattern: item[2][0][1][0] = HTTP URL
        // The item must have len > 2 for the function to inspect it
        let raw = serde_json::json!([
            "info-id", "Info Title", 7, null, 3,
            null, null, null, null, null, null, null, null, null, null,
            [null, null, [[null, ["https://cdn.google.com/infographic.png"]]]]
        ]);
        let url = extract_infographic_url(&raw);
        assert_eq!(url, Some("https://cdn.google.com/infographic.png".to_string()));
    }

    #[test]
    fn test_extract_infographic_url_filters_notebooklm() {
        // URL is notebooklm.google.com → filtered out
        let raw = serde_json::json!([
            "info-id", "Info Title", 7, null, 3,
            null, null, null, null, null, null, null, null, null, null,
            [null, null, [[null, ["https://notebooklm.google.com/something"]]]]
        ]);
        assert!(extract_infographic_url(&raw).is_none());
    }

    #[test]
    fn test_extract_infographic_url_returns_first_valid() {
        // Multiple items with URLs → returns first valid CDN URL
        let raw = serde_json::json!([
            "info-id", "Info Title", 7, null, 3,
            null, null, null, null, null, null, null, null, null, null,
            [null, null, [[null, ["https://cdn.google.com/first.png"]]]],
            [null, null, [[null, ["https://cdn.google.com/second.png"]]]],
        ]);
        let url = extract_infographic_url(&raw);
        assert_eq!(url, Some("https://cdn.google.com/first.png".to_string()));
    }

    #[test]
    fn test_extract_infographic_url_skips_short_items() {
        // Items with len <= 2 are skipped
        let raw = serde_json::json!([
            "info-id", "Info Title", 7, null, 3,
            null, null, null, null, null, null, null, null, null, null,
            [null, null],  // len=2, skipped
            [null, null, [[null, ["https://cdn.google.com/info.png"]]]],  // valid
        ]);
        let url = extract_infographic_url(&raw);
        assert_eq!(url, Some("https://cdn.google.com/info.png".to_string()));
    }

    #[test]
    fn test_extract_infographic_url_no_url_found() {
        // No nested URL in any item
        let raw = serde_json::json!([
            "info-id", "Info Title", 7, null, 3,
            null, null, null, null, null, null, null, null, null, null,
            [null, null, [[null, []]]]
        ]);
        assert!(extract_infographic_url(&raw).is_none());
    }

    #[test]
    fn test_extract_infographic_url_empty_content() {
        // content is empty array
        let raw = serde_json::json!([
            "info-id", "Info Title", 7, null, 3,
            null, null, null, null, null, null, null, null, null, null,
            [null, null, []]
        ]);
        assert!(extract_infographic_url(&raw).is_none());
    }

    #[test]
    fn test_extract_infographic_url_non_array_data() {
        assert!(extract_infographic_url(&Value::Null).is_none());
    }

    #[test]
    fn test_extract_infographic_url_skips_non_http() {
        // URL doesn't start with http
        let raw = serde_json::json!([
            "info-id", "Info Title", 7, null, 3,
            null, null, null, null, null, null, null, null, null, null,
            [null, null, [[null, ["ftp://cdn.google.com/info.png"]]]]
        ]);
        assert!(extract_infographic_url(&raw).is_none());
    }

    // --- extract_slide_deck_url ---

    fn make_slide_deck_raw(pdf_url: &str, pptx_url: &str) -> Value {
        // art[16] = metadata: [config, title, slides, pdf_url, pptx_url]
        // Need exactly 16 nulls + padding before index 16
        serde_json::json!([
            "deck-id", "Deck Title", 8, null, 3,  // indices 0-4
            null, null, null, null, null, null, null, null, null, null, null,  // indices 5-15 (11 nulls)
            ["config", "Title", [], pdf_url, pptx_url]  // index 16 ✓
        ])
    }

    #[test]
    fn test_extract_slide_deck_url_pdf() {
        let raw = make_slide_deck_raw("https://cdn.google.com/deck.pdf", "https://cdn.google.com/deck.pptx");
        let url = extract_slide_deck_url(&raw, "pdf");
        assert_eq!(url, Some("https://cdn.google.com/deck.pdf".to_string()));
    }

    #[test]
    fn test_extract_slide_deck_url_pptx() {
        let raw = make_slide_deck_raw("https://cdn.google.com/deck.pdf", "https://cdn.google.com/deck.pptx");
        let url = extract_slide_deck_url(&raw, "pptx");
        assert_eq!(url, Some("https://cdn.google.com/deck.pptx".to_string()));
    }

    #[test]
    fn test_extract_slide_deck_url_default_pdf() {
        let raw = make_slide_deck_raw("https://cdn.google.com/deck.pdf", "https://cdn.google.com/deck.pptx");
        let url = extract_slide_deck_url(&raw, "unknown_format");
        assert_eq!(url, Some("https://cdn.google.com/deck.pdf".to_string()));
    }

    #[test]
    fn test_extract_slide_deck_url_case_insensitive() {
        let raw = make_slide_deck_raw("https://cdn.google.com/deck.pdf", "https://cdn.google.com/deck.pptx");
        assert_eq!(extract_slide_deck_url(&raw, "PDF"), Some("https://cdn.google.com/deck.pdf".to_string()));
        assert_eq!(extract_slide_deck_url(&raw, "Pptx"), Some("https://cdn.google.com/deck.pptx".to_string()));
    }

    #[test]
    fn test_extract_slide_deck_url_empty_url() {
        let raw = make_slide_deck_raw("", "https://cdn.google.com/deck.pptx");
        assert!(extract_slide_deck_url(&raw, "pdf").is_none());
    }

    #[test]
    fn test_extract_slide_deck_url_non_http() {
        let raw = make_slide_deck_raw("ftp://cdn.google.com/deck.pdf", "");
        assert!(extract_slide_deck_url(&raw, "pdf").is_none());
    }

    #[test]
    fn test_extract_slide_deck_url_missing_metadata() {
        // art[16] not present (array too short)
        let raw = serde_json::json!(["deck-id", "Deck Title", 8, null, 3]);
        assert!(extract_slide_deck_url(&raw, "pdf").is_none());
    }

    #[test]
    fn test_extract_slide_deck_url_pptx_missing() {
        // PPTX requested but art[16] only has 4 elements (no index 4)
        let raw = serde_json::json!([
            "deck-id", "Deck Title", 8, null, 3,
            null, null, null, null, null, null, null, null, null, null, null,
            ["config", "Title", [], "https://cdn.google.com/deck.pdf"]
        ]);
        assert!(extract_slide_deck_url(&raw, "pptx").is_none());
    }

    // =========================================================================
    // Report content extraction tests — Task 6.4
    // =========================================================================

    #[test]
    fn test_extract_report_content_direct_string() {
        // art[7] is already a string
        let raw = serde_json::json!([
            "report-id", "Report Title", 2, null, 3,
            null, null,
            "# My Report\n\nThis is the content."
        ]);
        let content = extract_report_content(&raw);
        assert_eq!(content, Some("# My Report\n\nThis is the content.".to_string()));
    }

    #[test]
    fn test_extract_report_content_wrapped_in_array() {
        // art[7] is a 1-element list
        let raw = serde_json::json!([
            "report-id", "Report Title", 2, null, 3,
            null, null,
            ["# Wrapped Report\n\nContent here."]
        ]);
        let content = extract_report_content(&raw);
        assert_eq!(content, Some("# Wrapped Report\n\nContent here.".to_string()));
    }

    #[test]
    fn test_extract_report_content_empty_string() {
        let raw = serde_json::json!([
            "report-id", "Report Title", 2, null, 3,
            null, null, ""
        ]);
        assert!(extract_report_content(&raw).is_none());
    }

    #[test]
    fn test_extract_report_content_empty_wrapper() {
        // art[7] is an empty list
        let raw = serde_json::json!([
            "report-id", "Report Title", 2, null, 3,
            null, null, []
        ]);
        assert!(extract_report_content(&raw).is_none());
    }

    #[test]
    fn test_extract_report_content_null() {
        let raw = serde_json::json!([
            "report-id", "Report Title", 2, null, 3,
            null, null, null
        ]);
        assert!(extract_report_content(&raw).is_none());
    }

    #[test]
    fn test_extract_report_content_missing_index() {
        // art[7] not present
        let raw = serde_json::json!(["report-id", "Report Title", 2, null, 3]);
        assert!(extract_report_content(&raw).is_none());
    }

    #[test]
    fn test_extract_report_content_with_markdown_formatting() {
        let markdown = "# Title\n\n## Section\n\n- Item 1\n- Item 2\n\n**Bold** and *italic*";
        let raw = serde_json::json!([
            "report-id", "Report Title", 2, null, 3,
            null, null, [markdown]
        ]);
        let content = extract_report_content(&raw).unwrap();
        assert!(content.contains("# Title"));
        assert!(content.contains("**Bold**"));
        assert!(content.contains("*italic*"));
    }

    #[test]
    fn test_extract_report_content_non_array_data() {
        assert!(extract_report_content(&Value::Null).is_none());
    }

    // =========================================================================
    // Data table extraction tests — Task 6.5
    // =========================================================================

    // --- extract_cell_text ---

    #[test]
    fn test_extract_cell_text_string() {
        assert_eq!(extract_cell_text(&Value::String("hello".to_string())), "hello");
    }

    #[test]
    fn test_extract_cell_text_integer_skipped() {
        assert_eq!(extract_cell_text(&Value::Number(serde_json::Number::from(42))), "");
    }

    #[test]
    fn test_extract_cell_text_null_skipped() {
        assert_eq!(extract_cell_text(&Value::Null), "");
    }

    #[test]
    fn test_extract_cell_text_nested_array() {
        // [[42, "hello"], ["world"]] → "helloworld"
        let cell = serde_json::json!([[42, "hello"], ["world"]]);
        assert_eq!(extract_cell_text(&cell), "helloworld");
    }

    #[test]
    fn test_extract_cell_text_deeply_nested() {
        // Simulate the real nesting: [pos, pos, [[pos, pos, [[pos, pos, [["text"]]]]]]]
        let cell = serde_json::json!([1, 2, [[3, 4, [[5, 6, [["Cell Value"]]]]]]]);
        assert_eq!(extract_cell_text(&cell), "Cell Value");
    }

    #[test]
    fn test_extract_cell_text_mixed_content() {
        // Position markers (ints) mixed with text
        let cell = serde_json::json!([0, 15, [0, 8, [["prefix"]]], [8, 15, [["suffix"]]]]);
        assert_eq!(extract_cell_text(&cell), "prefixsuffix");
    }

    // --- parse_data_table ---

    fn make_data_table_raw(headers: Vec<&str>, rows: Vec<Vec<&str>>) -> Value {
        // Build the deeply nested structure:
        // artifact[18] = [0][0][0][0][4][2] = rows_array
        // Each row: [start_pos, end_pos, cell_array]
        // Each cell: [pos, pos, [[pos, pos, [[pos, pos, [["text"]]]]]]]
        let mut all_rows = Vec::new();

        // Header row
        let header_cells: Vec<Value> = headers.iter().map(|h| {
            serde_json::json!([0, 10, [[0, 10, [[0, 10, [[h]]]]]]])
        }).collect();
        all_rows.push(serde_json::json!([0, 100, header_cells]));

        // Data rows
        for row in rows {
            let cells: Vec<Value> = row.iter().map(|c| {
                serde_json::json!([0, 10, [[0, 10, [[0, 10, [[c]]]]]]])
            }).collect();
            all_rows.push(serde_json::json!([100, 200, cells]));
        }

        serde_json::json!([
            "table-id", "Table Title", 9, null, 3,
            null, null, null, null, null, null, null, null, null, null, null, null, null,
            [
                [[[[1, "table_type", 0, 0, [0, 0, all_rows]]]]]
            ]
        ])
    }

    #[test]
    fn test_parse_data_table_simple() {
        let raw = make_data_table_raw(
            vec!["Name", "Age", "City"],
            vec![vec!["Alice", "30", "NYC"], vec!["Bob", "25", "LA"]],
        );
        let result = parse_data_table(&raw);
        assert!(result.is_some());
        let (headers, rows) = result.unwrap();
        assert_eq!(headers, vec!["Name", "Age", "City"]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["Alice", "30", "NYC"]);
        assert_eq!(rows[1], vec!["Bob", "25", "LA"]);
    }

    #[test]
    fn test_parse_data_table_single_column() {
        let raw = make_data_table_raw(
            vec!["Item"],
            vec![vec!["Apple"], vec!["Banana"], vec!["Cherry"]],
        );
        let (headers, rows) = parse_data_table(&raw).unwrap();
        assert_eq!(headers, vec!["Item"]);
        assert_eq!(rows.len(), 3);
    }

    #[test]
    fn test_parse_data_table_empty_rows() {
        // Only headers, no data rows
        let raw = make_data_table_raw(vec!["Col1", "Col2"], vec![]);
        let (headers, rows) = parse_data_table(&raw).unwrap();
        assert_eq!(headers, vec!["Col1", "Col2"]);
        assert!(rows.is_empty());
    }

    #[test]
    fn test_parse_data_table_missing_index_18() {
        let raw = serde_json::json!(["table-id", "Table", 9, null, 3]);
        assert!(parse_data_table(&raw).is_none());
    }

    #[test]
    fn test_parse_data_table_null_at_18() {
        let raw = serde_json::json!([
            "table-id", "Table", 9, null, 3,
            null, null, null, null, null, null, null, null, null, null, null, null, null, null
        ]);
        assert!(parse_data_table(&raw).is_none());
    }

    #[test]
    fn test_parse_data_table_empty_rows_array() {
        // rows_array exists but is empty
        let raw = serde_json::json!([
            "table-id", "Table", 9, null, 3,
            null, null, null, null, null, null, null, null, null, null, null, null, null,
            [[[[[1, "type", 0, 0, [0, 0, []]]]]]]
        ]);
        assert!(parse_data_table(&raw).is_none());
    }

    #[test]
    fn test_parse_data_table_skips_short_row_sections() {
        // Row sections with len < 3 are skipped
        let raw = serde_json::json!([
            "table-id", "Table", 9, null, 3,
            null, null, null, null, null, null, null, null, null, null, null, null, null,
            [[[[[1, "type", 0, 0, [0, 0, [
                [0, 100, [[0, 10, [[0, 10, [["H1"]]]]]]],  // valid header row
                [100, 200],                                   // too short → skipped
                [200, 300, [[0, 10, [[0, 10, [["D1"]]]]]]],  // valid data row
            ]]]]]]]
        ]);
        let (headers, rows) = parse_data_table(&raw).unwrap();
        assert_eq!(headers, vec!["H1"]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], vec!["D1"]);
    }

    #[test]
    fn test_parse_data_table_non_array_data() {
        assert!(parse_data_table(&Value::Null).is_none());
    }

    // =========================================================================
    // Quiz / Flashcard extraction tests — Task 6.6
    // =========================================================================

    // --- extract_app_data ---

    fn make_html_with_app_data(json_obj: &Value) -> String {
        let json_str = serde_json::to_string(json_obj).unwrap();
        // HTML-encode quotes (as the real HTML does)
        let encoded = json_str.replace("\"", "&quot;");
        format!(r#"<div data-app-data="{}"></div>"#, encoded)
    }

    #[test]
    fn test_extract_app_data_quiz() {
        let json = serde_json::json!({
            "quiz": [
                {
                    "question": "What is Rust?",
                    "answerOptions": [
                        {"text": "A systems language", "isCorrect": true},
                        {"text": "A web framework", "isCorrect": false}
                    ],
                    "hint": "Think Mozilla"
                }
            ]
        });
        let html = make_html_with_app_data(&json);
        let result = extract_app_data(&html).unwrap();
        assert_eq!(result["quiz"][0]["question"], "What is Rust?");
        assert_eq!(result["quiz"][0]["answerOptions"][0]["isCorrect"], true);
    }

    #[test]
    fn test_extract_app_data_flashcards() {
        let json = serde_json::json!({
            "flashcards": [
                {"f": "What is ownership?", "b": "A memory management model"},
                {"f": "What is borrowing?", "b": "Referencing without taking ownership"}
            ]
        });
        let html = make_html_with_app_data(&json);
        let result = extract_app_data(&html).unwrap();
        assert_eq!(result["flashcards"][0]["f"], "What is ownership?");
        assert_eq!(result["flashcards"][1]["b"], "Referencing without taking ownership");
    }

    #[test]
    fn test_extract_app_data_no_attribute() {
        let html = "<div><p>No data here</p></div>";
        assert!(extract_app_data(html).is_none());
    }

    #[test]
    fn test_extract_app_data_empty_html() {
        assert!(extract_app_data("").is_none());
    }

    #[test]
    fn test_extract_app_data_invalid_json() {
        let html = r#"<div data-app-data="not-json"></div>"#;
        assert!(extract_app_data(html).is_none());
    }

    #[test]
    fn test_extract_app_data_decodes_html_entities() {
        // Test that &quot; is properly decoded
        let json = serde_json::json!({"key": "value with \"quotes\""});
        let json_str = serde_json::to_string(&json).unwrap();
        let encoded = json_str.replace("\"", "&quot;");
        let html = format!(r#"<div data-app-data="{}"></div>"#, encoded);
        let result = extract_app_data(&html).unwrap();
        assert_eq!(result["key"], "value with \"quotes\"");
    }

    #[test]
    fn test_extract_app_data_with_surrounding_html() {
        let json = serde_json::json!({"quiz": []});
        let json_str = serde_json::to_string(&json).unwrap();
        let encoded = json_str.replace("\"", "&quot;");
        let html = format!(
            r#"<html><head><title>Quiz</title></head><body><div data-app-data="{}"></div></body></html>"#,
            encoded
        );
        let result = extract_app_data(&html).unwrap();
        assert!(result["quiz"].as_array().unwrap().is_empty());
    }

    // --- format_quiz_markdown ---

    #[test]
    fn test_format_quiz_markdown_basic() {
        let questions = serde_json::json!([
            {
                "question": "What is 2+2?",
                "answerOptions": [
                    {"text": "3", "isCorrect": false},
                    {"text": "4", "isCorrect": true},
                    {"text": "5", "isCorrect": false}
                ]
            }
        ]);
        let md = format_quiz_markdown("Math Quiz", questions.as_array().unwrap());
        assert!(md.starts_with("# Math Quiz"));
        assert!(md.contains("## Question 1"));
        assert!(md.contains("What is 2+2?"));
        assert!(md.contains("- [ ] 3"));
        assert!(md.contains("- [x] 4"));
        assert!(md.contains("- [ ] 5"));
    }

    #[test]
    fn test_format_quiz_markdown_with_hint() {
        let questions = serde_json::json!([
            {
                "question": "Capital of France?",
                "answerOptions": [
                    {"text": "London", "isCorrect": false},
                    {"text": "Paris", "isCorrect": true}
                ],
                "hint": "Think romance"
            }
        ]);
        let md = format_quiz_markdown("Geo Quiz", questions.as_array().unwrap());
        assert!(md.contains("**Hint:** Think romance"));
    }

    #[test]
    fn test_format_quiz_markdown_empty_hint_omitted() {
        let questions = serde_json::json!([
            {
                "question": "Q1",
                "answerOptions": [{"text": "A", "isCorrect": true}],
                "hint": ""
            }
        ]);
        let md = format_quiz_markdown("Quiz", questions.as_array().unwrap());
        assert!(!md.contains("**Hint:**"));
    }

    #[test]
    fn test_format_quiz_markdown_multiple_questions() {
        let questions = serde_json::json!([
            {"question": "Q1?", "answerOptions": [{"text": "A", "isCorrect": true}]},
            {"question": "Q2?", "answerOptions": [{"text": "B", "isCorrect": false}]}
        ]);
        let md = format_quiz_markdown("Test", questions.as_array().unwrap());
        assert!(md.contains("## Question 1"));
        assert!(md.contains("## Question 2"));
    }

    #[test]
    fn test_format_quiz_markdown_empty_questions() {
        let questions: Vec<Value> = vec![];
        let md = format_quiz_markdown("Empty Quiz", &questions);
        assert_eq!(md, "# Empty Quiz\n");
    }

    #[test]
    fn test_format_quiz_markdown_no_options() {
        let questions = serde_json::json!([
            {"question": "Open-ended question?"}
        ]);
        let md = format_quiz_markdown("Quiz", questions.as_array().unwrap());
        assert!(md.contains("## Question 1"));
        assert!(md.contains("Open-ended question?"));
    }

    // --- format_flashcards_markdown ---

    #[test]
    fn test_format_flashcards_markdown_basic() {
        let cards = serde_json::json!([
            {"f": "What is ownership?", "b": "Memory management model"},
            {"f": "What is borrowing?", "b": "Reference without ownership"}
        ]);
        let md = format_flashcards_markdown("Rust Concepts", cards.as_array().unwrap());
        assert!(md.starts_with("# Rust Concepts"));
        assert!(md.contains("## Card 1"));
        assert!(md.contains("**Q:** What is ownership?"));
        assert!(md.contains("**A:** Memory management model"));
        assert!(md.contains("---"));
        assert!(md.contains("## Card 2"));
        assert!(md.contains("**Q:** What is borrowing?"));
        assert!(md.contains("**A:** Reference without ownership"));
    }

    #[test]
    fn test_format_flashcards_markdown_single_card() {
        let cards = serde_json::json!([
            {"f": "Front", "b": "Back"}
        ]);
        let md = format_flashcards_markdown("Single", cards.as_array().unwrap());
        assert!(md.contains("## Card 1"));
        assert!(!md.contains("## Card 2"));
    }

    #[test]
    fn test_format_flashcards_markdown_empty() {
        let cards: Vec<Value> = vec![];
        let md = format_flashcards_markdown("Empty Deck", &cards);
        assert_eq!(md, "# Empty Deck\n");
    }

    #[test]
    fn test_format_flashcards_markdown_missing_keys() {
        // Cards without "f" or "b" keys
        let cards = serde_json::json!([
            {"f": "Front only"},
            {"b": "Back only"},
            {"other": "irrelevant"}
        ]);
        let md = format_flashcards_markdown("Partial", cards.as_array().unwrap());
        assert!(md.contains("**Q:** Front only"));
        assert!(md.contains("**A:** Back only"));
        // Third card has no f/b → empty strings
        assert!(md.contains("**Q:** "));
        assert!(md.contains("**A:** "));
    }

    // =========================================================================
    // Mind map extraction tests — Task 6.7
    // =========================================================================

    // --- extract_note_content ---

    #[test]
    fn test_extract_note_content_old_format() {
        // Old format: item[1] is a string
        let item = serde_json::json!(["note-id", r#"{"children": []}"#, "metadata"]);
        let content = extract_note_content(&item);
        assert_eq!(content, Some(r#"{"children": []}"#.to_string()));
    }

    #[test]
    fn test_extract_note_content_new_format() {
        // New format: item[1] is an array, item[1][1] is the string
        let item = serde_json::json!(["note-id", ["note-id", r#"{"nodes": []}"#], null, null, "Title"]);
        let content = extract_note_content(&item);
        assert_eq!(content, Some(r#"{"nodes": []}"#.to_string()));
    }

    #[test]
    fn test_extract_note_content_null_field() {
        let item = serde_json::json!(["note-id", null]);
        assert!(extract_note_content(&item).is_none());
    }

    #[test]
    fn test_extract_note_content_array_too_short() {
        let item = serde_json::json!(["note-id", ["only-one"]]);
        assert!(extract_note_content(&item).is_none());
    }

    #[test]
    fn test_extract_note_content_array_non_string() {
        let item = serde_json::json!(["note-id", ["note-id", 42]]);
        assert!(extract_note_content(&item).is_none());
    }

    // --- is_mind_map_item ---

    #[test]
    fn test_is_mind_map_item_children() {
        let item = serde_json::json!(["note-id", r#"{"children": [{"text": "Node 1"}]}"#]);
        assert!(is_mind_map_item(&item));
    }

    #[test]
    fn test_is_mind_map_item_nodes() {
        let item = serde_json::json!(["note-id", r#"{"nodes": [{"id": "n1", "label": "Root"}]}"#]);
        assert!(is_mind_map_item(&item));
    }

    #[test]
    fn test_is_mind_map_item_not_mind_map() {
        let item = serde_json::json!(["note-id", r#"{"text": "Just a regular note"}"#]);
        assert!(!is_mind_map_item(&item));
    }

    #[test]
    fn test_is_mind_map_item_null_content() {
        let item = serde_json::json!(["note-id", null]);
        assert!(!is_mind_map_item(&item));
    }

    #[test]
    fn test_is_mind_map_item_new_format() {
        let item = serde_json::json!(["note-id", ["note-id", r#"{"children": []}"#]]);
        assert!(is_mind_map_item(&item));
    }

    // --- extract_mind_map_json ---

    #[test]
    fn test_extract_mind_map_json_children() {
        let json_str = r#"{"children": [{"text": "Root", "children": [{"text": "Child 1"}, {"text": "Child 2"}]}]}"#;
        let item = serde_json::json!(["note-id", json_str]);
        let result = extract_mind_map_json(&item).unwrap();
        assert_eq!(result["children"][0]["text"], "Root");
        assert_eq!(result["children"][0]["children"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_extract_mind_map_json_nodes() {
        let json_str = r#"{"nodes": [{"id": "r", "label": "Root"}, {"id": "c1", "label": "Child"}]}"#;
        let item = serde_json::json!(["note-id", json_str]);
        let result = extract_mind_map_json(&item).unwrap();
        assert_eq!(result["nodes"][0]["label"], "Root");
    }

    #[test]
    fn test_extract_mind_map_json_new_format() {
        let json_str = r#"{"children": [{"text": "Node"}]}"#;
        let item = serde_json::json!(["note-id", ["note-id", json_str]]);
        let result = extract_mind_map_json(&item).unwrap();
        assert_eq!(result["children"][0]["text"], "Node");
    }

    #[test]
    fn test_extract_mind_map_json_invalid_json() {
        let item = serde_json::json!(["note-id", "not-json"]);
        assert!(extract_mind_map_json(&item).is_none());
    }

    #[test]
    fn test_extract_mind_map_json_null_content() {
        let item = serde_json::json!(["note-id", null]);
        assert!(extract_mind_map_json(&item).is_none());
    }

    #[test]
    fn test_extract_mind_map_json_preserves_unicode() {
        // serde_json preserves Unicode by default
        let json_str = r#"{"children": [{"text": "日本語 テスト 🎉"}]}"#;
        let item = serde_json::json!(["note-id", json_str]);
        let result = extract_mind_map_json(&item).unwrap();
        assert_eq!(result["children"][0]["text"], "日本語 テスト 🎉");
    }
}
