# NotebookLM MCP Server

[![Rust](https://img.shields.io/badge/Rust-1.70+-dea584?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![MCP](https://img.shields.io/badge/MCP-Protocol-blue?style=flat-square)](https://modelcontextprotocol.io)
[![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)](LICENSE)

Servidor MCP (Model Context Protocol) no oficial para Google NotebookLM. Permite a agentes IA interacting con libretas NotebookLM — crear libretas, añadir fuentes, y chatear con documentos.

## Quick Start

```bash
# Compilar
cargo build --release

# Autenticarse (método recomendado)
./target/release/notebooklm-mcp auth-browser

# Verificar conexión
./target/release/notebooklm-mcp verify
```

Luego configurá tu cliente MCP (Cursor, Claude Desktop, Windsurf) pointing al binario.

## Documentación

| Guía | Descripción |
|------|-------------|
| [Overview](00-overview.md) | ¿Qué es y para qué sirve? |
| [Arquitectura](01-architecture.md) | Diseño técnico y módulos |
| [API Reference](02-api-reference.md) | Herramientas MCP y CLI |
| [Modelos de Datos](03-data-models.md) | Estructuras y tipos |
| [Instalación](04-setup.md) | Setup completo |
| [Guía de Usuario](05-user-guide.md) | Uso práctico |
| [Changelog](06-changelog.md) | Historial de cambios |

## Características

- ✅ Servidor MCP completo
- ✅ Autenticación browser automation (Chrome headless)
- ✅ Rate limiting integrado
- ✅ Cache conversacional
- ✅ Polling automático de fuentes
- ✅ Errores estructurados

## Estado

**Experimental** — Este proyecto hace reverse engineering de APIs internas de Google. Usalo bajo tu propio riesgo.

---

¿Encontraste un bug? Abrí un issue en GitHub.
