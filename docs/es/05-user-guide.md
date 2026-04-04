---
title: "Guia de Usuario — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: es
scan_type: full
---

# Guia de Usuario

## Que Problema Resuelve?

Google NotebookLM no tiene una API publica. Este servidor MCP permite a los agentes de IA crear notebooks, agregar fuentes y consultar documentos de forma programatica — todo a traves del Model Context Protocol.

## Flujos Principales

### Configuracion Inicial

1. `cargo build --release`
2. `notebooklm-mcp auth-browser`
3. `notebooklm-mcp verify`
4. Configurar el cliente MCP

### Crear y Consultar

1. `notebook_create` — creá un notebook con un titulo
2. `source_add` — agregá contenido de texto como fuente
3. Esperar la indexacion (manejado automaticamente, 2-60s)
4. `ask_question` — consultá con respuestas generadas por IA

### Historial de Conversacion

- La primera pregunta crea un ID de conversacion
- Las preguntas siguientes reutilizan la misma conversacion
- El historial se envia con cada consulta para dar contexto

## Limitaciones

- Solo fuentes de texto (sin PDF/URL/YouTube via MCP todavia)
- Estado en memoria (se reinicia al rearrancar)
- Limite de ~30 req/min
- API con ingenieria inversa (puede romperse)
