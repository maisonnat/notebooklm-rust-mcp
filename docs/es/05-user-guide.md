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

# Guía de Usuario (Español)

## Primeros Pasos

### 1. Autenticación

Antes de usar el servidor, necesitás authenticate con tu cuenta de Google:

```bash
notebooklm-mcp auth-browser
```

Esto abre Chrome para que inicies sesión. Una vez hecho, las credenciales se guardan automáticamente.

### 2. Verificar conexión

```bash
notebooklm-mcp verify
```

Debería mostrarte las libretas existentes.

## Uso desde Cliente MCP

### Listar libretas

```json
{
  "name": "notebook_list"
}
```

### Crear una libreta

```json
{
  "name": "notebook_create",
  "arguments": {
    "title": "Mi Nuevo Proyecto"
  }
}
```

### Añadir una fuente

```json
{
  "name": "source_add",
  "arguments": {
    "notebook_id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Documento de Requisitos",
    "content": "Este documento describe los requisitos del proyecto..."
  }
}
```

### Hacer una pregunta

```json
{
  "name": "ask_question",
  "arguments": {
    "notebook_id": "550e8400-e29b-41d4-a716-446655440000",
    "question": "¿Cuál es el objetivo principal del proyecto?"
  }
}
```

## Uso desde CLI

También podés usar el servidor directamente desde línea de comandos:

```bash
# Crear libreta
notebooklm-mcp create-notebook "Mis Notas"

# Añadir fuente
notebooklm-mcp add-source \
  --notebook-id "uuid" \
  --title "Mi Fuente" \
  --content "Contenido..."

# Hacer pregunta
notebooklm-mcp ask \
  --notebook-id "uuid" \
  --question "¿Qué resume este documento?"
```

## Recursos

Los notebooks también están disponibles como recursos MCP:

```
notebook://550e8400-e29b-41d4-a716-446655440000
```

Podés usar este URI en tu cliente MCP para acceder a metadatos del notebook.

## Errores Comunes

### "SESIÓN EXPIRADA"

Las cookies de Google expiraron. Volvé a authenticate:

```bash
notebooklm-mcp auth-browser
```

### "FUENTE NO LISTA"

La fuente aún se está indexando. El cliente automáticamente hace polling, pero si persisté el error, esperá unos segundos más.

### "RATE LIMITED"

Demasiadas requests. Esperá un momento y reintentá.

## Tips

- **Mantené las credenciales actualizadas** — Las cookies de Google expiran cada ciertos días
- **Usá fuentes cortas al principio** — La indexación es más rápida
- **El historial de conversación se mantiene** — Entre preguntas al mismo notebook, el chatbot tiene contexto
- **Rate limiting protege tu cuenta** — No intentes hacer más de 2 requests por segundo