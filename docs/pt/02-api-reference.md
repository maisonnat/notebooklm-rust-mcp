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

# Referência da API (Português)

## Ferramentas MCP

### notebook_list

Lista todos os cadernos disponíveis na conta.

```json
{
  "name": "notebook_list",
  "description": "List all notebooks available in the account"
}
```

**Retorna:**
```
Notebooks: [{"id": "uuid-1", "title": "Meu Caderno"}, ...]
```

### notebook_create

Cria um novo caderno.

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

**Parâmetros:**
- `title` (string, required) — Título do caderno

**Retorna:**
```
Caderno criado. ID: <uuid>
```

### source_add

Adiciona uma fonte de texto a um caderno.

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

**Parâmetros:**
- `notebook_id` (string, required) — UUID do caderno
- `title` (string, required) — Título da fonte
- `content` (string, required) — Conteúdo do texto

**Retorna:**
```
Fonte adicionada. ID: <source_uuid>
```

### ask_question

Faz uma pergunta ao chatbot de um caderno.

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

**Parâmetros:**
- `notebook_id` (string, required) — UUID do caderno
- `question` (string, required) — Pergunta a fazer

**Retorna:**
```
<resposta do chatbot>
```

## Recursos MCP

### notebook://{uuid}

Recursos que representam cadernos do NotebookLM.

```
notebook://550e8400-e29b-41d4-a716-446655440000
```

**Conteúdo:**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "title": "Meu Caderno",
  "uri": "notebook://550e8400-e29b-41d4-a716-446655440000"
}
```

## CLI

### auth

Salva cookies encriptadas com DPAPI.

```bash
notebooklm-mcp auth --cookie "..." --csrf "..."
```

### auth-browser

Autenticação via Chrome headless (recomendado).

```bash
notebooklm-mcp auth-browser
```

### auth-status

Verifica estado da autenticação.

```bash
notebooklm-mcp auth-status
```

### verify

Verifica conexão com NotebookLM.

```bash
notebooklm-mcp verify
```

### ask

Faz uma pergunta via CLI.

```bash
notebooklm-mcp ask --notebook-id "..." --question "..."
```

### add-source

Adiciona uma fonte via CLI.

```bash
notebooklm-mcp add-source --notebook-id "..." --title "..." --content "..."
```

## Erros

| Código | Descrição |
|--------|-------------|
| SESSÃO EXPIRADA | Cookies do Google expiraram — re-autenticar |
| CSRF EXPIRADO | Token CSRF inválido — refresh automático |
| FONTE NÃO PRONTA | Fonte sendo indexada — fazer polling |
| RATE LIMITED | Muitos requests — reduzir concorrência |
| ERROR DE PARSEO | Resposta inesperada do Google |
| ERROR DE REDE | Problema de conectividade |