# Multi-Source Ingestion Specification

## Purpose

Extiende la capacidad del servidor MCP para ingerir URLs web, videos de YouTube, archivos binarios y documentos de Google Drive como fuentes de notebook, superando la limitación actual de solo texto.

## Requirements

### Requirement: URL Source Ingestion

El sistema MUST permitir añadir una URL web como fuente de un notebook usando el RPC `izAoDd` con la URL en posición `[2]` de un array de 8 elementos.

#### Scenario: Add valid web URL

- GIVEN un notebook existente con ID válido
- AND una URL web válida (ej. `https://example.com/article`)
- WHEN el cliente invoca `add_url_source(notebook_id, url)`
- THEN el sistema envía el RPC `izAoDd` con el payload posicional correcto
- AND devuelve el source_id de la fuente creada

#### Scenario: URL with special characters

- GIVEN un notebook existente
- AND una URL con query params y caracteres especiales (ej. `https://site.com/path?a=1&b=foo+bar`)
- WHEN el cliente invoca `add_url_source`
- THEN la URL se serializa correctamente sin corrupción
- AND el RPC devuelve source_id válido

### Requirement: YouTube Auto-Detection

El sistema MUST detectar automáticamente URLs de YouTube y usar el payload de 11 elementos (URL en posición `[7]`) en lugar del payload de URL regular.

#### Scenario: YouTube watch URL detected

- GIVEN una URL `https://www.youtube.com/watch?v=dQw4w9WgXcQ`
- WHEN el cliente invoca `add_url_source`
- THEN el sistema detecta que es YouTube por hostname
- AND usa el payload de YouTube (11-element array con URL en `[7]`)
- AND incluye el config extra `[1, None, ..., [1]]` en posición `[3]`

#### Scenario: YouTube short URL detected

- GIVEN una URL `https://youtu.be/dQw4w9WgXcQ`
- WHEN el cliente invoca `add_url_source`
- THEN el sistema detecta YouTube por hostname `youtu.be`
- AND usa el payload de YouTube

#### Scenario: Non-YouTube URL uses regular payload

- GIVEN una URL `https://vimeo.com/123456`
- WHEN el cliente invoca `add_url_source`
- THEN el sistema NO la trata como YouTube
- AND usa el payload de URL regular (8-element array)

#### Scenario: Invalid YouTube URL format

- GIVEN una URL `https://youtube.com/` sin video ID
- WHEN el cliente invoca `add_url_source`
- THEN el sistema la trata como URL regular (no YouTube)

### Requirement: File Upload via Resumable Protocol

El sistema MUST soportar la subida de archivos binarios (PDF, TXT, MD, EPUB, DOCX) usando el protocolo de 3 pasos de Google: register RPC → start resumable → stream upload.

#### Scenario: Upload PDF file successfully

- GIVEN un notebook existente
- AND un archivo PDF en disco (ej. `/docs/research.pdf`)
- WHEN el cliente invoca `add_file_source(notebook_id, file_path)`
- THEN el sistema ejecuta el paso 1: RPC `o4cbdc` con filename double-nested
- AND extrae SOURCE_ID de la respuesta (nested: `[[[[source_id]]]]`)
- AND ejecuta el paso 2: POST a `/upload/_/?authuser=0` con headers de resumable upload
- AND extrae `x-goog-upload-url` de los response headers
- AND ejecuta el paso 3: POST streaming del archivo en chunks de 64KB
- AND devuelve el source_id

#### Scenario: Upload streams file without loading entirely in RAM

- GIVEN un archivo de 100MB
- WHEN el sistema sube el archivo
- THEN el pico de uso de RAM NO excede el tamaño del chunk (64KB) + overhead
- AND el archivo se lee de disco con `tokio::fs::File`
- AND los bytes se envían con `reqwest::Body::wrap_stream()`

#### Scenario: File not found

- GIVEN un path que no existe en disco
- WHEN el cliente invoca `add_file_source`
- THEN el sistema devuelve error `FileNotFound` con el path en el mensaje
- AND NO se ejecuta ningún RPC ni HTTP request

#### Scenario: Path is a directory not a file

- GIVEN un path que apunta a un directorio
- WHEN el cliente invoca `add_file_source`
- THEN el sistema devuelve error `ValidationError`

### Requirement: Google Drive Source

El sistema MUST permitir añadir un documento de Google Drive como fuente usando file_id, mime_type y title via RPC `izAoDd`.

#### Scenario: Add Google Doc as source

- GIVEN un notebook existente
- AND un file_id de Google Drive (ej. `1abc123xyz`)
- AND un mime_type válido (ej. `application/vnd.google-apps.document`)
- WHEN el cliente invoca `add_drive_source(notebook_id, file_id, title, mime_type)`
- THEN el sistema envía el RPC `izAoDd` con source_data single-wrapped (NO double)
- AND incluye `[file_id, mime_type, 1, title]` en posición `[0]` del source_data
- AND devuelve el source_id

#### Scenario: Default mime_type for Google Docs

- GIVEN un file_id sin mime_type explícito
- WHEN el cliente invoca `add_drive_source` sin mime_type
- THEN el sistema usa `application/vnd.google-apps.document` como default

### Requirement: MCP Tool Registration

El servidor MCP MUST exponer 3 nuevas tools: `source_add_url`, `source_add_file`, `source_add_drive`.

#### Scenario: MCP tool source_add_url

- GIVEN un MCP client conectado
- WHEN invoca `source_add_url` con `{ notebook_id, url }`
- THEN el servidor ejecuta `add_url_source` y devuelve source_id

#### Scenario: MCP tool source_add_file

- GIVEN un MCP client conectado
- WHEN invoca `source_add_file` con `{ notebook_id, file_path }`
- THEN el servidor ejecuta `add_file_source` y devuelve source_id

#### Scenario: MCP tool source_add_drive

- GIVEN un MCP client conectado
- WHEN invoca `source_add_drive` con `{ notebook_id, file_id, title, mime_type? }`
- THEN el servidor ejecuta `add_drive_source` y devuelve source_id

### Requirement: CLI Commands

El CLI MUST proveer subcomandos `add-url`, `add-file` y `add-drive` para uso directo desde terminal.

#### Scenario: Add URL via CLI

- GIVEN credenciales válidas almacenadas
- WHEN usuario ejecuta `notebooklm-mcp add-url --notebook-id ID --url URL`
- THEN el sistema invoca `add_url_source` e imprime source_id

#### Scenario: Add file via CLI

- GIVEN credenciales válidas y archivo existe
- WHEN usuario ejecuta `notebooklm-mcp add-file --notebook-id ID --file-path PATH`
- THEN el sistema sube el archivo e imprime source_id

### Requirement: Error Handling for Source Operations

El sistema MUST devolver errores estructurados para todos los fallos de source ingestion, SIN usar `unwrap()` en datos de red.

#### Scenario: RPC returns error

- GIVEN un notebook_id válido pero el RPC falla (ej. rate limit)
- WHEN el cliente invoca `add_url_source`
- THEN devuelve error `NotebookLmError` con la categoría correcta
- AND NO panic ni unwrap

#### Scenario: Upload session fails to start

- GIVEN el paso 1 (register) exitoso
- WHEN el paso 2 (start resumable) devuelve HTTP error o no incluye `x-goog-upload-url`
- THEN devuelve error `UploadFailed` con contexto del paso fallido
