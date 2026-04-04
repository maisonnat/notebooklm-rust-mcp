# NotebookLM MCP Server

> Unofficial MCP (Model Context Protocol) server for Google NotebookLM — allows AI agents to interact with notebooks programmatically.

[![Rust](https://img.shields.io/badge/Rust-1.70+-dea584?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-Protocol-blue?style=flat-square)](https://modelcontextprotocol.io)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Experimental-orange?style=flat-square)](#status)
[![unsafe: 0](https://img.shields.io/badge/unsafe-0-success?style=flat-square)](https://www.rust-lang.org)

## What Is It?

An MCP server that bridges AI agents with Google NotebookLM:

- Create and list notebooks
- Add text sources to notebooks
- Ask questions with AI-powered answers and conversation history

## Quick Start

```bash
cargo build --release
./target/release/notebooklm-mcp auth-browser
./target/release/notebooklm-mcp verify
```

Then configure your MCP client (Cursor, Claude Desktop, Windsurf) pointing to the binary.

## Features

| Feature | Description |
|---------|-------------|
| MCP Server | Full Model Context Protocol implementation |
| Browser Auth | Chrome headless automation via CDP |
| Rate Limiting | Token bucket (30 req/min) with exponential backoff |
| Conversation Cache | Per-notebook history with `RwLock<HashMap>` |
| Source Polling | Automatic wait for source indexing |
| Defensive Parsing | Zero `unwrap()` on external data |
| Zero Unsafe | No `unsafe` blocks in the codebase |

## Documentation

### English (Primary)

| Doc | Description |
|-----|-------------|
| [Overview](docs/en/00-overview.md) | What it is and why it exists |
| [Architecture](docs/en/01-architecture.md) | Module structure, design patterns, data flow |
| [API Reference](docs/en/02-api-reference.md) | MCP tools, CLI commands |
| [Data Models](docs/en/03-data-models.md) | Types and entities |
| [Setup](docs/en/04-setup.md) | Installation and configuration |
| [User Guide](docs/en/05-user-guide.md) | How to use it |
| [Changelog](docs/en/06-changelog.md) | Version history |
| [Security Posture](docs/en/07-security-posture.md) | Auth, credentials, memory safety, supply chain |

### Espanol

| Doc | Descripcion |
|-----|-------------|
| [Overview](docs/es/00-overview.md) | Que es y para que sirve |
| [Arquitectura](docs/es/01-architecture.md) | Estructura y patrones de diseno |
| [Referencia API](docs/es/02-api-reference.md) | Herramientas MCP y comandos CLI |
| [Modelos de Datos](docs/es/03-data-models.md) | Tipos y entidades |
| [Instalacion](docs/es/04-setup.md) | Instalacion y configuracion |
| [Guia de Usuario](docs/es/05-user-guide.md) | Como usarlo |
| [Changelog](docs/es/06-changelog.md) | Historial de cambios |

### Portugues

| Doc | Descricao |
|-----|----------|
| [Visao Geral](docs/pt/00-overview.md) | O que e e para que serve |
| [Arquitetura](docs/pt/01-architecture.md) | Estrutura e padroes de projeto |
| [Referencia API](docs/pt/02-api-reference.md) | Ferramentas MCP e comandos CLI |
| [Modelos de Dados](docs/pt/03-data-models.md) | Tipos e entidades |
| [Configuracao](docs/pt/04-setup.md) | Instalacao e configuracao |
| [Guia do Usuario](docs/pt/05-user-guide.md) | Como usar |
| [Changelog](docs/pt/06-changelog.md) | Historico de alteracoes |

### Engineers

| Resource | Description |
|----------|-------------|
| [OpenAPI Spec](docs/openapi.yaml) | Internal Google RPC API documentation |
| [CodeTour](.tours/architecture-walkthrough.tour) | Interactive IDE walkthrough (VS Code / Cursor) |

## Tech Stack

Rust (edition 2024) + Tokio + rmcp + reqwest (rustls) + governor + headless_chrome + keyring

## Status

> **Experimental** — Reverse-engineers Google's internal APIs. Not affiliated with or endorsed by Google.

## License

MIT
