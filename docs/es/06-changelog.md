---
title: "Registro de Cambios — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: es
scan_type: full
---

# Registro de Cambios

## [0.3.1] — 2026-04-06

### Módulo 6: Endurecimiento Anti-Detección

- **Suplantación de huella del navegador**: 12 headers HTTP tipo Chrome inyectados en todas las solicitudes batchexecute (User-Agent, Sec-Fetch-*, Sec-CH-UA, Origin, Referer, Accept-*)
- **Circuit breaker**: detiene solicitudes tras 3 errores de autenticación consecutivos, se reabre automáticamente tras 60s de enfriamiento
- **Auto-refresh de CSRF**: refresh silencioso de tokens en 401/400/403 con `tokio::sync::Mutex` para coordinación concurrente
- **Corrección de backoff exponencial**: corregido `1^x` (siempre 1) a `2^x` (exponencial real: 2, 4, 8, 16s...)
- **Jitter tipo humano**: incrementado de 150-600ms a 800-2000ms para simular mejor el ritmo humano
- **Soporte de Retry-After**: respeta el header `Retry-After` de Google en respuestas 429 (segundos enteros + fecha HTTP)
- **Nueva dependencia**: `httpdate` para parseo de fechas Retry-After
- 342 tests, 0 advertencias de clippy

## [0.3.0] — 2026-04-06

### Módulo 5: Funcionalidades Avanzadas

- **Gestión de fuentes**: `source_delete` (idempotente), `source_rename`, `source_get_fulltext` (extracción recursiva de texto desde fuentes indexadas)
- **CRUD de notas**: `note_create` (RPC de dos pasos: crear vacía → actualizar), `note_list` (filtra eliminadas con soft-delete), `note_delete`
- **Historial de chat**: `chat_history` — recupera la conversación completa desde los servidores de Google en orden cronológico
- **Investigación profunda**: `research_deep_dive` — lanza el motor de investigación autónoma de Google, realiza polling hasta completar, importa fuentes descubiertas
- **Parser recursivo**: `extract_all_text` con max-depth configurable para respuestas RPC de Google profundamente anidadas
- **Continuidad de conversación**: `ask_question` recupera automáticamente el ID de conversación activa desde los servidores de Google
- 8 nuevas herramientas MCP, 8 nuevos comandos CLI, 11 nuevos métodos de cliente
- 333 tests, 0 advertencias de clippy
- Ciclo SDD completo (Explorar → Proponer → 5 Especificaciones → Diseñar → Tareas → Aplicar → Verificar)

## [0.2.1] — 2026-04-05

### Módulo 4: Ciclo de Vida de Notebooks y Compartido

- **CRUD de notebooks**: Herramientas `notebook_delete`, `notebook_get`, `notebook_rename`
- **Resumen IA**: `notebook_summary` — resumen generado por IA + temas sugeridos
- **Compartido**: `notebook_share_status`, `notebook_share_set` — alternar público/privado, ver usuarios compartidos
- **Lectura post-mutación**: Las operaciones de escritura devuelven estado autoritativo confirmado
- 6 nuevas herramientas MCP, 6 nuevos comandos CLI, 6 nuevos métodos de cliente
- Ciclo SDD completo (Explorar → Proponer → Especificar → Diseñar → Tareas → Aplicar → Verificar → Archivar)

## [0.2.0] — 2026-04-05

### Módulo 3: Generación y Descarga de Artefactos

- **9 tipos de artefactos**: Report, Quiz, Flashcards, Audio, Infographic, Slide Deck, Mind Map, Video, Data Table
- **Gestión de artefactos**: Herramientas `artifact_list`, `artifact_generate`, `artifact_delete`, `artifact_download`
- **Polling async de artefactos**: `artifact_poller.rs` — realiza polling del estado de generación hasta completarse
- **Parámetros específicos por tipo**: Dificultad, cantidad, idioma, duración, estilo, formato
- **Descargas en streaming**: Descarga directa desde URLs de almacenamiento de Google
- 4 nuevas herramientas MCP, 4 nuevos comandos CLI

## [0.1.1] — 2026-04-04

### Módulo 2: Soporte Multifuentes

- **5 tipos de fuentes**: Texto, URL, YouTube, Google Drive, Subida de archivo
- **Autodetección de YouTube**: `source_add_url` detecta URLs de YouTube y utiliza la ingesta específica de YouTube
- **Integración con Google Drive**: Agregar archivos de Drive por ID de archivo
- **Subida de archivos**: Subir archivos locales como fuentes
- **Polling async de fuentes**: `source_poller.rs` — realiza polling del estado de indexación hasta que la fuente esté lista
- **Extracción de módulo RPC**: `rpc/sources.rs` — constructores de payload dedicados
- 4 nuevas herramientas MCP, 4 nuevos comandos CLI

## [0.1.0] — 2026-03-28

### Lanzamiento Inicial

- Servidor MCP con 4 herramientas: `notebook_list`, `notebook_create`, `source_add`, `ask_question`
- Autenticación por navegador vía Chrome CDP (comando `auth-browser`)
- Almacenamiento de credenciales en keyring del SO con fallback a DPAPI
- Extracción de token CSRF desde HTML (`SNlM0e`)
- Limitación de tasa vía governor (período de 2s, ~30 req/min)
- Retroceso exponencial con jitter para reintentos
- Polling de fuentes para verificar disponibilidad de indexación async
- Parser defensivo para respuestas RPC de Google (eliminación anti-XSSI)
- Enum de errores estructurado con autodetección
- Parseo de respuestas en streaming para `ask_question`
- Autenticación manual vía variables de entorno
- Comandos CLI `verify` y `auth-status`

### Seguridad

- Cero bloques `unsafe`
- `cargo-audit`: 0 vulnerabilidades (334 deps)
- TLS vía rustls (sin OpenSSL)

> **[English](../en/06-changelog.md)** · **[Português](../pt/06-changelog.md)**
