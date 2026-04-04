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

# NotebookLM MCP Server — Overview

## What this is

**NotebookLM MCP Server** es un servidor MCP (Model Context Protocol) no oficial que permite interacting con Google NotebookLM a través de una interfaz estandarizada.

Básicamente, te permite:
- **Listar libretas** existentes en tu cuenta de NotebookLM
- **Crear nuevas libretas** con título personalizado
- **Añadir fuentes de texto** a cualquier libreta
- **Hacer preguntas al chatbot de IA** de NotebookLM

Todo esto desde cualquier cliente MCP (Cursor, Windsurf, Claude Desktop, etc.).

## Technology stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Language | Rust | 1.70+ |
| Framework | rmcp (MCP) | 1.2 |
| HTTP Client | reqwest | 0.12 |
| Async Runtime | tokio | 1.50 |
| Build | Cargo | latest |
| CLI | clap | 4.4 |

## Quick start

> [!TIP]
> These steps get you from zero to running in under 5 minutes.

1. **Clone y build**
   ```bash
   git clone https://github.com/maisonnat/notebooklm-rust-mcp
   cd notebooklm-rust-mcp
   cargo build --release
   ```

2. **Autenticarse (recomendado)**
   ```bash
   ./target/release/notebooklm-mcp auth-browser
   ```

3. **Verificar conexión**
   ```bash
   ./target/release/notebooklm-mcp verify
   ```

Para instalación completa, ver [[04-setup]].

## Repository structure

```
notebooklm-rust-mcp/
├── src/                    — Código fuente
│   ├── main.rs              — Entry point + CLI + MCP server
│   ├── notebooklm_client.rs — Cliente HTTP + rate limiting
│   ├── auth_browser.rs      — Autenticación Chrome headless
│   ├── auth_helper.rs       — Extracción CSRF
│   ├── parser.rs            — Parser defensivo RPC
│   ├── source_poller.rs     — Polling de fuentes
│   ├── conversation_cache.rs — Cache conversacional
│   └── errors.rs            — Errores estructurados
├── docs/                   — Documentación
├── Cargo.toml              — Dependencias
└── Cargo.lock
```

## License & maintainers

- **License:** MIT
- **Repository:** https://github.com/maisonnat/notebooklm-rust-mcp

> [!WARNING] Experimental
> Este proyecto hace reverse engineering de APIs internas de Google. Usalo bajo tu propio riesgo.
