---
title: "Overview — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
last_commit: "b467e15"
scan_type: full
tags: [rust, mcp, documentation]
audience: both
---

# Changelog (Español)

Todos los cambios notables de este proyecto serán documentados en este archivo.

## [0.1.0] - 2026-04-04

### Agregado

- **Servidor MCP completo** con 4 herramientas: `notebook_list`, `notebook_create`, `source_add`, `ask_question`
- **Recursos MCP**: notebooks disponibles como URIs `notebook://{uuid}`
- **Autenticación por browser automation** vía Chrome headless (CDP)
- **Autenticación manual** con DPAPI (Windows)
- **Soporte de keyring**: Windows Credential Manager / Linux Secret Service
- **Rate limiting** con governor (2 req/segundo)
- **Retry con exponential backoff** para robustez
- **Parser defensivo** para respuestas RPC de Google
- **Conversation cache** para mantener contexto entre preguntas
- **Source poller** para esperar indexación de fuentes
- **Errores estructurados** para mejor debugging

### Detalles Técnicos

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