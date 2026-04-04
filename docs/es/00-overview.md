---
title: "Resumen General — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: es
scan_type: full
---

# Resumen General

> **Servidor MCP no oficial para Google NotebookLM** — escrito en Rust con cero bloques `unsafe`.

## Que Es?

NotebookLM MCP Server es un servidor de [Model Context Protocol](https://modelcontextprotocol.io) que permite a los agentes de IA (Claude, Cursor, Windsurf, etc.) interactuar con notebooks de Google NotebookLM de forma programatica.

**Capacidades principales:**
- Crear, listar y administrar notebooks
- Agregar fuentes de texto a los notebooks
- Hacer preguntas y recibir respuestas generadas por IA con historial de conversacion
- Sondeo automatico de fuentes (espera a que se indexen antes de consultar)

## Inicio Rapido

```bash
# Compilar
cargo build --release

# Autenticarse (metodo recomendado)
./target/release/notebooklm-mcp auth-browser

# Verificar conexion
./target/release/notebooklm-mcp verify
```

Luego configurá tu cliente MCP para que apunte al binario (transporte stdio).

## Stack Tecnologico

| Componente | Tecnologia |
|-----------|-----------|
| Lenguaje | Rust (edicion 2024) |
| Runtime Async | Tokio |
| Framework MCP | rmcp 1.2 |
| Cliente HTTP | reqwest 0.12 (rustls-tls) |
| Parser CLI | clap 4.4 |
| Limitacion de tasa | governor 0.6 |
| Autenticacion via browser | headless_chrome 1 (CDP) |
| Almacenamiento de credenciales | keyring 3 + fallback DPAPI |

## Estado

> **Experimental** — Este proyecto realiza ingenieria inversa de las APIs internas de Google. Usalo bajo tu propio riesgo.

- Sin soporte oficial de API por parte de Google
- Los endpoints RPC internos pueden cambiar sin aviso
- Las cookies de sesion expiran frecuentemente

## Licencia

MIT
