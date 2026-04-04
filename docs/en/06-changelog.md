# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-04-04

### Added
- **MCP Server completo** con 4 tools: `notebook_list`, `notebook_create`, `source_add`, `ask_question`
- **Recursos MCP**: notebooks disponibles como `notebook://{uuid}` URIs
- **Autenticación browser automation** via headless Chrome (CDP)
- **Autenticación manual** con DPAPI (Windows)
- **Keyring support**: Windows Credential Manager / Linux Secret Service
- **Rate limiting** con governor (2 req/segundo)
- **Retry con exponential backoff** para robustness
- **Parser defensivo** para respuestas RPC de Google
- **Conversation cache** para mantener contexto entre preguntas
- **Source poller** para esperar indexación de fuentes
- **Errores estructurados** para mejor debugging

### Technical Details

- **Runtime**: Tokio async
- **HTTP Client**: reqwest con streaming
- **Servidor MCP**: rmcp crate
- **Browser Automation**: headless_chrome
- **Credential Storage**: windows-dpapi + keyring

### RPC IDs Descubiertos

| RPC ID | Función |
|--------|---------|
| `wXbhsf` | Listar libretas |
| `CCqFvf` | Crear libreta |
| `izAoDd` | Añadir fuente |
| `rLM1Ne` | Obtener fuentes de libreta |
| `GenerateFreeFormStreamed` | Chat streaming |

### Autores

- Reverse engineering basado en notebooklm-py
- Implementación en Rust por el autor del proyecto
