# NotebookLM MCP Server

> Unofficial MCP (Model Context Protocol) server for Google NotebookLM — bridges AI agents with notebooks programmatically via reverse-engineered internal API.

[![Rust](https://img.shields.io/badge/Rust-1.70+-dea584?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-Protocol-blue?style=flat-square)](https://modelcontextprotocol.io)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![unsafe: 0](https://img.shields.io/badge/unsafe-0-success?style=flat-square)](https://www.rust-lang.org)
[![vulns: 0](https://img.shields.io/badge/vulns-0-success?style=flat-square)](https://github.com/rustsec/rustsec)
[![tests: 377](https://img.shields.io/badge/tests-377-blue?style=flat-square)](https://www.rust-lang.org)
[![zread](https://img.shields.io/badge/Ask_Zread-_.svg?style=for-the-badge&color=00b0aa&labelColor=000000&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB3aWR0aD0iMTYiIGhlaWdodD0iMTYiIHZpZXdCb3g9IjAgMCAxNiAxNiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTQuOTYxNTYgMS42MDAxSDIuMjQxNTZDMS44ODgxIDEuNjAwMSAxLjYwMTU2IDEuODg2NjQgMS42MDE1NiAyLjI0MDFWNC45NjAxQzEuNjAxNTYgNS4zMTM1NiAxLjg4ODEgNS42MDAxIDIuMjQxNTYgNS42MDAxSDQuOTYxNTZDNS4zMTUwMiA1LjYwMDEgNS42MDE1NiA1LjMxMzU2IDUuNjAxNTYgNC45NjAxVjIuMjQwMUM1LjYwMTU2IDEuODg2NjQgNS4zMTUwMiAxLjYwMDEgNC45NjE1NiAxLjYwMDFaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik00Ljk2MTU2IDEwLjM5OTlIMi4yNDE1NkMxLjg4ODEgMTAuMzk5OSAxLjYwMTU2IDEwLjY4NjQgMS42MDE1NiAxMS4wMzk5VjEzLjc1OTlDMS42MDE1NiAxNC4xMTM0IDEuODg4MSAxNC4zOTk5IDIuMjQxNTYgMTQuMzk5OUg0Ljk2MTU2QzUuMzE1MDIgMTQuMzk5OSA1LjYwMTU2IDE0LjExMzQgNS42MDE1NiAxMy43NTk5VjExLjAzOTlDNS42MDE1NiAxMC42ODY0IDUuMzE1MDIgMTAuMzk5OSA0Ljk2MTU2IDEwLjM5OTlaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik0xMy43NTg0IDEuNjAwMUgxMS4wMzg0QzEwLjY4NSAxLjYwMDEgMTAuMzk4NCAxLjg4NjY0IDEwLjM5ODQgMi4yNDAxVjQuOTYwMUMxMC4zOTg0IDUuMzEzNTYgMTAuNjg1IDUuNjAwMSAxMS4wMzg0IDUuNjAwMUgxMy43NTg0QzE0LjExMTkgNS42MDAxIDE0LjM5ODQgNS4zMTM1NiAxNC4zOTg0IDQuOTYwMVYyLjI0MDFDMTQuMzk4NCAxLjg4NjY0IDE0LjExMTkgMS42MDAxIDEzLjc1ODQgMS42MDAxWiIgZmlsbD0iI2ZmZiIvPgo8cGF0aCBkPSJNNCAxMkwxMiA0TDQgMTJaIiBmaWxsPSIjZmZmIi8%2BCjxwYXRoIGQ9Ik00IDEyTDEyIDQiIHN0cm9rZT0iI2ZmZiIgc3Ryb2tlLXdpZHRoPSIxLjUiIHN0cm9rZS1saW5lY2FwPSJyb3VuZCIvPgo8L3N2Zz4K&logoColor=ffffff)](https://zread.ai/maisonnat/notebooklm-rust-mcp)

## What Is It?

An MCP server that lets AI agents (Claude, Cursor, Windsurf, etc.) interact with Google NotebookLM notebooks. Google has **no public API** — this server reverse-engineers the internal RPC protocol.

## Capabilities

### Notebook Management (8 tools)
Create, list, rename, delete, get details, AI summary, share status, toggle sharing

### Source Management (5 tools)
Add text, URL, YouTube, Google Drive, or local file sources

### Artifact Generation (4 tools)
Generate 9 content types: reports, quizzes, flashcards, audio, infographics, slide decks, mind maps, videos, data tables

### AI Interaction (1 tool)
Ask questions with streaming responses

**Total: 20 MCP tools + 21 CLI commands**

## Quick Start

```bash
# Build
cargo build --release

# Authenticate via Chrome
./target/release/notebooklm-mcp auth-browser

# Verify credentials
./target/release/notebooklm-mcp verify

# Run as MCP server (stdio)
./target/release/notebooklm-mcp
```

### MCP Client Configuration

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/path/to/notebooklm-mcp",
      "args": []
    }
  }
}
```

## Features

| Feature | Description |
|---------|-------------|
| **20 MCP Tools** | Full notebook, source, artifact, and AI interaction coverage |
| **21 CLI Commands** | All tools available from the command line |
| **Browser Auth** | Chrome CDP automation — credentials in OS keyring |
| **9 Artifact Types** | Reports, quizzes, flashcards, audio, infographics, slides, mind maps, videos, data tables |
| **5 Source Types** | Text, URL, YouTube, Google Drive, file upload |
| **Streaming** | Real-time AI responses via SSE |
| **Rate Limiting** | Token bucket (~30 req/min) with exponential backoff |
| **Defensive Parsing** | Zero `unwrap()` on external data |
| **Zero Unsafe** | No `unsafe` blocks |
| **Zero Vulnerabilities** | `cargo-audit` clean (334 deps) |
| **Cross-Platform** | Windows, macOS, Linux (OS keyring) |

## Documentation

### English

| Doc | Description |
|-----|-------------|
| [Overview](docs/en/00-overview.md) | What it is and why it exists |
| [Architecture](docs/en/01-architecture.md) | Module structure, design patterns, data flow |
| [API Reference](docs/en/02-api-reference.md) | MCP tools, CLI commands, configuration |
| [Data Models](docs/en/03-data-models.md) | Entities, enums, type definitions |
| [Setup Guide](docs/en/04-setup.md) | Installation, authentication, troubleshooting |
| [User Guide](docs/en/05-user-guide.md) | Common workflows, tips, limitations |
| [Changelog](docs/en/06-changelog.md) | Release history |
| [Security Posture](docs/en/07-security-posture.md) | Auth, credentials, memory safety, audit |

### Español

| Doc | Descripción |
|-----|-------------|
| [Vista General](docs/es/00-overview.md) | Qué es y para qué sirve |
| [Arquitectura](docs/es/01-architecture.md) | Estructura y patrones de diseño |
| [Referencia de API](docs/es/02-api-reference.md) | Herramientas MCP y comandos CLI |
| [Modelos de Datos](docs/es/03-data-models.md) | Tipos y entidades |
| [Guía de Configuración](docs/es/04-setup.md) | Instalación y configuración |
| [Guía de Usuario](docs/es/05-user-guide.md) | Cómo usarlo |
| [Registro de Cambios](docs/es/06-changelog.md) | Historial de versiones |
| [Postura de Seguridad](docs/es/07-security-posture.md) | Autenticación, seguridad, auditoría |

### Português

| Doc | Descrição |
|-----|----------|
| [Visão Geral](docs/pt/00-overview.md) | O que é e para que serve |
| [Arquitetura](docs/pt/01-architecture.md) | Estrutura e padrões de projeto |
| [Referência de API](docs/pt/02-api-reference.md) | Ferramentas MCP e comandos CLI |
| [Modelos de Dados](docs/pt/03-data-models.md) | Tipos e entidades |
| [Guia de Configuração](docs/pt/04-setup.md) | Instalação e configuração |
| [Guia do Usuário](docs/pt/05-user-guide.md) | Como usar |
| [Registro de Alterações](docs/pt/06-changelog.md) | Histórico de versões |
| [Postura de Segurança](docs/pt/07-security-posture.md) | Autenticação, segurança, auditoria |

### Engineers

| Resource | Description |
|----------|-------------|
| [CodeTour Walkthrough](.tours/architecture-walkthrough.tour) | Interactive IDE onboarding (VS Code / Cursor) |

## Tech Stack

Rust (edition 2024) · Tokio · rmcp · reqwest (rustls) · governor · headless_chrome · keyring · clap · serde

## Status

> **Experimental** — Reverse-engineers Google's internal APIs. Not affiliated with or endorsed by Google.

## License

MIT
