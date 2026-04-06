---
title: "Referencia de API — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: es
---

# Referencia de API

## Herramientas MCP

### Gestión de Notebooks

#### `notebook_list`

Lista todos los notebooks del usuario.

**Parámetros:** Ninguno

**Devuelve:** Lista formateada de notebooks con ID y título.

#### `notebook_create`

Crea un nuevo notebook.

**Parámetros:**
- `title` (cadena, requerido): El título del notebook

**Devuelve:** ID del notebook creado.

#### `notebook_delete`

Elimina un notebook por ID. Idempotente — no genera error si el notebook no existe.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook

**Devuelve:** Mensaje de confirmación.

#### `notebook_get`

Obtiene los detalles completos de un notebook, incluyendo cantidad de fuentes, propiedad y fecha de creación.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook

**Devuelve:** Detalles del notebook (título, ID, cantidad de fuentes, estado de propietario, fecha de creación).

#### `notebook_rename`

Renombra un notebook. Devuelve los detalles actualizados del notebook.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `new_title` (cadena, requerido): Nuevo título para el notebook

**Devuelve:** Detalles actualizados del notebook.

#### `notebook_summary`

Obtiene el resumen generado por IA y los temas sugeridos para un notebook.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook

**Devuelve:** Texto del resumen y lista de temas sugeridos (pares pregunta + prompt).

#### `notebook_share_status`

Obtiene la configuración de compartido de un notebook.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook

**Devuelve:** Estado público/privado, nivel de acceso, lista de usuarios compartidos con correos y permisos, URL de compartido.

#### `notebook_share_set`

Alterna la visibilidad del notebook entre público y privado. Devuelve el estado de compartido actualizado.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `public` (booleano, requerido): `true` para público, `false` para privado

**Devuelve:** Estado de compartido actualizado.

### Gestión de Fuentes

#### `source_add`

Agrega una fuente de texto a un notebook.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `title` (cadena, requerido): Título de la fuente
- `content` (cadena, requerido): Contenido de texto de la fuente

**Devuelve:** ID de la fuente.

#### `source_add_url`

Agrega una fuente URL a un notebook. Detecta automáticamente URLs de YouTube y utiliza el flujo de ingesta específico de YouTube.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `url` (cadena, requerido): URL a agregar
- `title` (cadena, opcional): Título personalizado (se extrae automáticamente si se omite)

**Devuelve:** ID de la fuente.

#### `source_add_youtube`

Agrega un video de YouTube como fuente.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `url` (cadena, requerido): URL del video de YouTube
- `title` (cadena, opcional): Título personalizado

**Devuelve:** ID de la fuente.

#### `source_add_drive`

Agrega un archivo de Google Drive como fuente.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `file_id` (cadena, requerido): ID del archivo de Google Drive
- `title` (cadena, opcional): Título personalizado

**Devuelve:** ID de la fuente.

#### `source_add_file`

Sube un archivo local como fuente.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `file_path` (cadena, requerido): Ruta del archivo local a subir
- `title` (cadena, opcional): Título personalizado

**Devuelve:** ID de la fuente.

### Gestión de Artefactos

#### `artifact_list`

Lista todos los artefactos de un notebook.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook

**Devuelve:** Lista de artefactos con ID, título, tipo y estado.

#### `artifact_generate`

Genera un artefacto. Los parámetros específicos del tipo se agregan según el parámetro `kind`.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `kind` (cadena, requerido): Tipo de artefacto
- Parámetros adicionales dependen de `kind` (ver abajo)

**Devuelve:** ID del artefacto para hacer polling de completitud.

**Tipos de Artefacto:**

| Kind | Parámetros Adicionales | Formato de Salida |
|------|------------------------|-------------------|
| `report` | `instructions` (opcional) | PDF |
| `quiz` | `difficulty` (easy/medium/hard), `quantity` (3-20) | PDF |
| `flashcards` | `quantity` (3-20) | PDF |
| `audio` | `language` (en/es/etc), `length` (short/medium/long), `instructions` (opcional) | Archivo de audio |
| `infographic` | `detail` (brief/standard), `orientation` (landscape/portrait), `style` (default/professional/casual) | PNG |
| `slide_deck` | `format` (pdf/pptx), `length` (short/medium/long) | PDF/PPTX |
| `mind_map` | — | JSON |
| `video` | `format` (cinematic/documentary), `style` (default/dramatic/cinematic) | Archivo de video |
| `data_table` | — | PDF |

#### `artifact_delete`

Elimina un artefacto de un notebook.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `artifact_id` (cadena, requerido): ID del artefacto a eliminar

**Devuelve:** Mensaje de confirmación.

#### `artifact_download`

Descarga un artefacto en el formato apropiado.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `artifact_id` (cadena, requerido): ID del artefacto
- `output` (cadena, opcional): Ruta del archivo de salida (por defecto un nombre auto-generado en el directorio actual)

**Devuelve:** Ruta del archivo del artefacto descargado.

### Interacción con IA

#### `ask_question`

Hace una pregunta sobre un notebook con respuesta en streaming.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `question` (cadena, requerido): Pregunta a realizar

**Devuelve:** Respuesta de texto en streaming (fragmentos).

### Gestión de Fuentes

#### `source_delete`

Elimina una fuente de un notebook. Idempotente — no genera error si la fuente no existe.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `source_id` (cadena, requerido): ID de la fuente a eliminar

**Devuelve:** Mensaje de confirmación.

#### `source_rename`

Renombra una fuente de un notebook.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `source_id` (cadena, requerido): ID de la fuente a renombrar
- `new_title` (cadena, requerido): Nuevo título para la fuente

**Devuelve:** Mensaje de confirmación.

#### `source_get_fulltext`

Obtiene el texto indexado completo de una fuente (extraído por Google de PDFs, páginas web, etc.). Útil para leer el contenido del documento sin hacer preguntas.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `source_id` (cadena, requerido): ID de la fuente

**Devuelve:** Contenido de texto extraído completo.

### Notas

#### `note_create`

Crea una nota en un notebook. Las notas son visibles en la interfaz web de NotebookLM.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `title` (cadena, requerido): Título de la nota
- `content` (cadena, requerido): Contenido de la nota

**Devuelve:** ID de la nota.

#### `note_list`

Lista todas las notas activas de un notebook (excluye notas eliminadas con soft-delete).

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook

**Devuelve:** Lista de notas con ID y título.

#### `note_delete`

Elimina una nota de un notebook (soft-delete).

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `note_id` (cadena, requerido): ID de la nota a eliminar

**Devuelve:** Mensaje de confirmación.

### Historial de Chat

#### `chat_history`

Obtiene el historial oficial de conversación de chat desde los servidores de Google para un notebook. Devuelve los turnos en orden cronológico (del más antiguo al más reciente).

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `limit` (entero, opcional): Máximo de turnos a recuperar (por defecto: 20)

**Devuelve:** Lista de turnos con rol ("user" o "assistant") y texto.

### Investigación Profunda

#### `research_deep_dive`

Inicia una investigación profunda usando el motor de investigación autónoma de Google. Bloquea hasta completar (timeout de hasta 300s), luego importa las fuentes descubiertas al notebook.

**Parámetros:**
- `notebook_id` (cadena, requerido): UUID del notebook
- `query` (cadena, requerido): Consulta de investigación
- `timeout_secs` (entero, opcional): Tiempo máximo de espera en segundos (por defecto: 300)

**Devuelve:** Resumen de las fuentes descubiertas.

## Comandos CLI

| Comando | Flags | Descripción |
|---------|-------|-------------|
| `auth-browser` | — | Autenticarse vía Chrome headless |
| `auth-status` | — | Verificar estado de autenticación |
| `verify` | — | Validación E2E de credenciales |
| `list` | — | Listar todos los notebooks |
| `create` | `--title` | Crear un notebook |
| `delete` | `--notebook-id` | Eliminar un notebook |
| `get` | `--notebook-id` | Obtener detalles de un notebook |
| `rename` | `--notebook-id` `--title` | Renombrar un notebook |
| `summary` | `--notebook-id` | Obtener resumen IA |
| `share-status` | `--notebook-id` | Obtener config de compartido |
| `share-set` | `--notebook-id` `--public` / `--private` | Alternar compartido |
| `source-add` | `--notebook-id` `--title` `--content` | Agregar fuente de texto |
| `source-add-url` | `--notebook-id` `--url` `--title` | Agregar fuente URL |
| `source-add-youtube` | `--notebook-id` `--url` `--title` | Agregar fuente YouTube |
| `source-add-drive` | `--notebook-id` `--file-id` `--title` | Agregar fuente Drive |
| `source-add-file` | `--notebook-id` `--file-path` `--title` | Subir archivo como fuente |
| `source-delete` | `--notebook-id` `--source-id` | Eliminar una fuente |
| `source-rename` | `--notebook-id` `--source-id` `--new-title` | Renombrar una fuente |
| `source-get-fulltext` | `--notebook-id` `--source-id` | Obtener texto completo de fuente |
| `artifact-list` | `--notebook-id` | Listar artefactos |
| `artifact-generate` | `--notebook-id` `--kind` + flags específicos del tipo | Generar artefacto |
| `artifact-delete` | `--notebook-id` `--artifact-id` | Eliminar artefacto |
| `artifact-download` | `--notebook-id` `--artifact-id` `--output` | Descargar artefacto |
| `note-create` | `--notebook-id` `--title` `--content` | Crear una nota |
| `note-list` | `--notebook-id` | Listar notas |
| `note-delete` | `--notebook-id` `--note-id` | Eliminar una nota |
| `chat-history` | `--notebook-id` `--limit` | Obtener historial de chat |
| `research` | `--notebook-id` `--query` `--timeout-secs` | Investigación profunda |
| `ask` | `--notebook-id` `--question` | Hacer pregunta |

## Configuración

### Variables de Entorno

| Variable | Tipo | Descripción |
|----------|------|-------------|
| `NOTEBOOKLM_COOKIE` | cadena | Cookie de autenticación de Google (desde el keyring del SO si no está configurada) |
| `NOTEBOOKLM_CSRF` | cadena | Token CSRF (desde el keyring del SO si no está configurada) |
| `NOTEBOOKLM_SID` | cadena | ID de sesión (desde el keyring del SO si no está configurada) |

### Configuración del Cliente MCP

Para usar este servidor con un cliente MCP (Cursor, Claude Desktop, Windsurf):

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/ruta/a/notebooklm-mcp",
      "args": []
    }
  }
}
```

> **[English](../en/02-api-reference.md)** · **[Português](../pt/02-api-reference.md)**
