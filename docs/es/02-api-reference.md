---
title: "API Reference — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
last_commit: "b467e15"
scan_type: full
tags: [rust, mcp, documentation]
audience: developers
---

# Referencia de API (Español)

## Herramientas MCP

### notebook_list

Lista todas las libretas disponibles en la cuenta.

```json
{
  "name": "notebook_list",
  "description": "List all notebooks available in the account"
}
```

**Retorna:**
```
Notebooks: [{"id": "uuid-1", "title": "Mi Libreta"}, ...]
```

### notebook_create

Crea una nueva libreta.

```json
{
  "name": "notebook_create",
  "description": "Create a new notebook by title",
  "inputSchema": {
    "type": "object",
    "properties": {
      "title": { "type": "string", "description": "Title for the new notebook" }
    },
    "required": ["title"]
  }
}
```

**Parámetros:**
- `title` (string, required) — Título de la libreta

**Retorna:**
```
Cuaderno creado. ID: <uuid>
```

### source_add

Añade una fuente de texto a una libreta.

```json
{
  "name": "source_add",
  "description": "Add a text source to a notebook",
  "inputSchema": {
    "type": "object",
    "properties": {
      "notebook_id": { "type": "string", "description": "UUID of the notebook" },
      "title": { "type": "string", "description": "Title of the source" },
      "content": { "type": "string", "description": "Text content" }
    },
    "required": ["notebook_id", "title", "content"]
  }
}
```

**Parámetros:**
- `notebook_id` (string, required) — UUID de la libreta
- `title` (string, required) — Título de la fuente
- `content` (string, required) — Contenido de texto

**Retorna:**
```
Fuente añadida. ID: <source_uuid>
```

### ask_question

Hace una pregunta al chatbot de una libreta.

```json
{
  "name": "ask_question",
  "description": "Ask a question to a notebook",
  "inputSchema": {
    "type": "object",
    "properties": {
      "notebook_id": { "type": "string", "description": "UUID of the notebook" },
      "question": { "type": "string", "description": "Question to ask" }
    },
    "required": ["notebook_id", "question"]
  }
}
```

**Parámetros:**
- `notebook_id` (string, required) — UUID de la libreta
- `question` (string, required) — Pregunta a realizar

**Retorna:**
```
<respuesta del chatbot>
```

## Recursos MCP

### notebook://{uuid}

Recursos que representan libretas de NotebookLM.

```
notebook://550e8400-e29b-41d4-a716-446655440000
```

**Contenido:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "Mi Libreta",
  "uri": "notebook://550e8400-e29b-41d4-a716-446655440000"
}
```

## CLI

### auth

Guarda cookies encriptadas con DPAPI.

```bash
notebooklm-mcp auth --cookie "..." --csrf "..."
```

### auth-browser

Autenticación vía Chrome headless (recomendado).

```bash
notebooklm-mcp auth-browser
```

### auth-status

Verifica estado de autenticación.

```bash
notebooklm-mcp auth-status
```

### verify

Verifica conexión con NotebookLM.

```bash
notebooklm-mcp verify
```

### ask

Hace una pregunta desde CLI.

```bash
notebooklm-mcp ask --notebook-id "..." --question "..."
```

### add-source

Añade una fuente desde CLI.

```bash
notebooklm-mcp add-source --notebook-id "..." --title "..." --content "..."
```

## Errores

| Código | Descripción |
|--------|-------------|
| SESIÓN EXPIRADA | Cookies de Google expiraron — re-autenticar |
| CSRF EXPIRADO | Token CSRF inválido — refresh automático |
| FUENTE NO LISTA | Fuente indexándose — hacer polling |
| RATE LIMITED | Demasiados requests — reducir concurrencia |
| ERROR DE PARSEO | Respuesta inesperada de Google |
| ERROR DE RED | Problema de conectividad |