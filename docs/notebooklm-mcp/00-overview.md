# NotebookLM MCP Server — Overview

## ¿Qué es?

**NotebookLM MCP Server** es un servidor MCP (Model Context Protocol) no oficial que permite interacting con Google NotebookLM a través de una interfaz estandarizada.

## ¿Para qué sirve?

Básicamente, te permite:
- **Listar libretas** existentes en tu cuenta de NotebookLM
- **Crear nuevas libretas** con título personalizado
- **Añadir fuentes de texto** a cualquier libreta
- **Hacer preguntas al chatbot de IA** de NotebookLM

Todo esto desde cualquier cliente MCP (Cursor, Windsurf, Claude Desktop, etc.).

## ¿Por qué existe?

Google NotebookLM es una herramienta poderosa para resumir documentos y chatear con ellos. Pero no tiene API oficial. Este proyecto hace **reverse engineering** de las APIs internas de Google para habilitar automatización y integración con agentes IA.

## Estado del Proyecto

| Aspecto | Estado |
|---------|--------|
| Servidor MCP | ✅ Funcional |
| Listar libretas | ✅ Funcional |
| Crear libretas | ✅ Funcional |
| Añadir fuentes | ✅ Funcional |
| Hacer preguntas | ✅ Funcional |
| Autenticación automática | ✅ Chrome headless |
| Recursos MCP | ✅ notebook:// URIs |

## Licencia

MIT — Uso bajo tu propio riesgo.
