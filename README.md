# NotebookLM MCP Server

> Servidor MCP (Model Context Protocol) no oficial para Google NotebookLM — permite a agentes IA interactuar con libretas NotebookLM.

[![Rust](https://img.shields.io/badge/Rust-1.70+-dea584?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-Protocol-blue?style=flat-square)](https://modelcontextprotocol.io)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Experimental-orange?style=flat-square)](#estado)

## Descripción

Este servidor MCP permite a agentes IA y clientes MCP (Cursor, Claude Desktop, Windsurf) comunicarse con Google NotebookLM para:

- ✅ Crear libretas nuevas
- ✅ Añadir fuentes (URLs, PDFs, YouTube, Google Docs)
- ✅ Chatear con documentos
- ✅ Gestionar conversaciones

## Quick Start

```bash
# Compilar
cargo build --release

# Autenticarse (método recomendado)
./target/release/notebooklm-mcp auth-browser

# Verificar conexión
./target/release/notebooklm-mcp verify
```

Luego configurá tu cliente MCP apuntando al binario compilado.

## Características

| Característica | Descripción |
|----------------|-------------|
| 🔌 **Servidor MCP Completo** | Implementación full del protocolo MCP |
| 🌐 **Browser Automation** | Autenticación via Chrome headless |
| ⏱️ **Rate Limiting** | Protección contra límites de Google |
| 💾 **Cache Conversacional** | Historial persistido |
| 📡 **Polling Automático** | Monitoreo de fuentes |
| 🛡️ **Errores Estructurados** | Manejo robusto de excepciones |

## Documentación

### English
- [Overview](docs/en/00-overview.md)
- [Architecture](docs/en/01-architecture.md)
- [API Reference](docs/en/02-api-reference.md)
- [Data Models](docs/en/03-data-models.md)
- [Setup](docs/en/04-setup.md)
- [User Guide](docs/en/05-user-guide.md)
- [Security Posture](docs/en/07-security-posture.md)

### Español
- [Overview](docs/es/00-overview.md)
- [Arquitectura](docs/es/01-architecture.md)
- [Referencia API](docs/es/02-api-reference.md)
- [Modelos de Datos](docs/es/03-data-models.md)
- [Instalación](docs/es/04-setup.md)
- [Guía de Usuario](docs/es/05-user-guide.md)

### Português
- [Visão Geral](docs/pt/00-overview.md)
- [Arquitetura](docs/pt/01-architecture.md)
- [Referência API](docs/pt/02-api-reference.md)
- [Modelos de Dados](docs/pt/03-data-models.md)
- [Configuração](docs/pt/04-setup.md)
- [Guia do Usuário](docs/pt/05-user-guide.md)

## Estado

> ⚠️ **Experimental** — Este proyecto hace reverse engineering de APIs internas de Google. Usalo bajo tu propio riesgo.

Este es un proyecto no oficial y no está afiliado, patrocinado ni respaldado por Google.

## Licencia

MIT License — ver [LICENSE](LICENSE) para detalles.

---

¿Encontraste un bug? [Abrí un issue en GitHub](https://github.com/maisonnat/notebooklm-rust-mcp/issues).