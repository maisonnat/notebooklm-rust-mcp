---
title: "Registro de Cambios — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: es
scan_type: full
---

# Registro de Cambios

## [0.1.0] — 2026-04-04

### Agregado
- Servidor MCP con 4 herramientas: `notebook_list`, `notebook_create`, `source_add`, `ask_question`
- Autenticacion via browser con Chrome CDP (comando `auth-browser`)
- Almacenamiento de credenciales en keyring del SO con fallback DPAPI
- Extraccion de token CSRF desde HTML (`SNlM0e`)
- Limitacion de tasa via governor (periodo de 2s, ~30 req/min)
- Backoff exponencial con jitter para reintentos
- Sondeo de fuentes para verificar disponibilidad de indexacion async
- Cache de conversacion (en memoria, por notebook)
- Parser defensivo para respuestas RPC de Google (eliminacion de anti-XSSI)
- Enum de errores estructurado con auto-deteccion
- Parseo de respuestas en streaming para `ask_question`
- Autenticacion manual via flags `--cookie` / `--csrf`
- Comando `verify` para validacion E2E
- Comando `auth-status`
- Tests unitarios en todos los modulos

### Seguridad
- Cero bloques `unsafe`
- `cargo-audit`: 0 vulnerabilidades (305 dependencias)
- TLS via rustls (sin OpenSSL)
