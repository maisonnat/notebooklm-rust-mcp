# NotebookLM MCP Server

> Servidor MCP (Model Context Protocol) no oficial para Google NotebookLM â€” permite a agentes IA interactuar con libretas NotebookLM.

[![Rust](https://img.shields.io/badge/Rust-1.70+-dea584?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-Protocol-blue?style=flat-square)](https://modelcontextprotocol.io)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)
[![Status](https://img.shields.io/badge/Status-Experimental-orange?style=flat-square)](#estado)

## DescripciÃ³n

Este servidor MCP permite a agentes IA y clientes MCP (Cursor, Claude Desktop, Windsurf) comunicarse con Google NotebookLM para:

- âœ… Crear libretas nuevas
- âœ… AÃ±adir fuentes (URLs, PDFs, YouTube, Google Docs)
- âœ… Chatear con documentos
- âœ… Gestionar conversaciones

## Quick Start

```bash
# Compilar
cargo build --release

# Autenticarse (mÃ©todo recomendado)
./target/release/notebooklm-mcp auth-browser

# Verificar conexiÃ³n
./target/release/notebooklm-mcp verify
```

Luego configurÃ¡ tu cliente MCP apuntando al binario compilado.

## CaracterÃ­sticas

| CaracterÃ­stica | DescripciÃ³n |
|----------------|-------------|
| ðŸ”Œ **Servidor MCP Completo** | ImplementaciÃ³n full del protocolo MCP |
| ðŸŒ **Browser Automation** | AutenticaciÃ³n via Chrome headless |
| â±ï¸ **Rate Limiting** | ProtecciÃ³n contra lÃ­mites de Google |
| ðŸ’¾ **Cache Conversacional** | Historial persistido |
| ðŸ“¡ **Polling AutomÃ¡tico** | Monitoreo de fuentes |
| ðŸ›¡ï¸ **Errores Estructurados** | Manejo robusto de excepciones |

## DocumentaciÃ³n

### English
- [Overview](docs/en/00-overview.md)
- [Architecture](docs/en/01-architecture.md)
- [API Reference](docs/en/02-api-reference.md)
- [Data Models](docs/en/03-data-models.md)
- [Setup](docs/en/04-setup.md)
- [User Guide](docs/en/05-user-guide.md)
- [Security Posture](docs/en/07-security-posture.md)

### EspaÃ±ol
- [Overview](docs/es/00-overview.md)
- [Arquitectura](docs/es/01-architecture.md)
- [Referencia API](docs/es/02-api-reference.md)
- [Modelos de Datos](docs/es/03-data-models.md)
- [InstalaciÃ³n](docs/es/04-setup.md)
- [GuÃ­a de Usuario](docs/es/05-user-guide.md)

### PortuguÃªs
- [VisÃ£o Geral](docs/pt/00-overview.md)
- [Arquitetura](docs/pt/01-architecture.md)
- [ReferÃªncia API](docs/pt/02-api-reference.md)
- [Modelos de Dados](docs/pt/03-data-models.md)
- [ConfiguraÃ§Ã£o](docs/pt/04-setup.md)
- [Guia do UsuÃ¡rio](docs/pt/05-user-guide.md)

## Estado

> âš ï¸ **Experimental** â€” Este proyecto hace reverse engineering de APIs internas de Google. Usalo bajo tu propio riesgo.

Este es un proyecto no oficial y no estÃ¡ afiliado, patrocinado ni respaldado por Google.

## Licencia

MIT License â€” ver [LICENSE](LICENSE) para detalles.

---

Â¿Encontraste un bug? [AbrÃ­ un issue en GitHub](https://github.com/tu-repo/issues).