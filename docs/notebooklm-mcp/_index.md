# NotebookLM MCP Server

Servidor MCP (Model Context Protocol) no oficial para Google NotebookLM.

## Descripción

Este proyecto permite a agentes IA y clientes MCP interacting con libretas de Google NotebookLM — crear libretas, añadir fuentes de texto, y hacer preguntas al chatbot de IA.

## Características

- **Servidor MCP**: Implementación completa del protocolo MCP
- **Autenticación segura**: Múltiples métodos (DPAPI, Keyring, Chrome headless)
- **Rate limiting**: Protege contra límites de la API de Google
- **Cache conversacional**: Mantiene contexto entre preguntas
- **Polling automático**: Espera indexación de fuentes

## Enlaces

- [Overview](00-overview.md)
- [Arquitectura](01-architecture.md)
- [Referencia de API](02-api-reference.md)
- [Modelos de Datos](03-data-models.md)
- [Instalación](04-setup.md)
- [Guía de Usuario](05-user-guide.md)
- [Changelog](06-changelog.md)
