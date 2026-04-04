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

# Guia do Usuário (Português)

## Primeiros Passos

### 1. Autenticação

Antes de usar o servidor, você precisa autenticar com sua conta do Google:

```bash
notebooklm-mcp auth-browser
```

Isso abre o Chrome para você fazer login. Uma vez feito, as credenciais são salvas automaticamente.

### 2. Verificar conexão

```bash
notebooklm-mcp verify
```

Deveria mostrar os cadernos existentes.

## Uso do Cliente MCP

### Listar cadernos

```json
{
  "name": "notebook_list"
}
```

### Criar um caderno

```json
{
  "name": "notebook_create",
  "arguments": {
    "title": "Meu Novo Projeto"
  }
}
```

### Adicionar uma fonte

```json
{
  "name": "source_add",
  "arguments": {
    "notebook_id": "550e8400-e29b-41d4-a716-446655440000",
    "title": "Documento de Requisitos",
    "content": "Este documento descreve os requisitos do projeto..."
  }
}
```

### Fazer uma pergunta

```json
{
  "name": "ask_question",
  "arguments": {
    "notebook_id": "550e8400-e29b-41d4-a716-446655440000",
    "question": "Qual é o objetivo principal do projeto?"
  }
}
```

## Uso via CLI

Você também pode usar o servidor diretamente da linha de comandos:

```bash
# Criar caderno
notebooklm-mcp create-notebook "Minhas Notas"

# Adicionar fonte
notebooklm-mcp add-source \
  --notebook-id "uuid" \
  --title "Minha Fonte" \
  --content "Conteúdo..."

# Fazer pergunta
notebooklm-mcp ask \
  --notebook-id "uuid" \
  --question "O que resume este documento?"
```

## Recursos

Os notebooks também estão disponíveis como recursos MCP:

```
notebook://550e8400-e29b-41d4-a716-446655440000
```

Você pode usar este URI no seu cliente MCP para acessar metadados do notebook.

## Erros Comuns

### "SESSÃO EXPIRADA"

As cookies do Google expiraram. Autentique novamente:

```bash
notebooklm-mcp auth-browser
```

### "FONTE NÃO PRONTA"

A fonte ainda está sendo indexada. O cliente automaticamente faz polling, mas se o erro persistir, espere mais alguns segundos.

### "RATE LIMITED"

Muitas requests. Espere um momento e tente novamente.

## Dicas

- **Mantenha as credenciais atualizadas** — As cookies do Google expiram a cada poucos dias
- **Use fontes curtas no início** — A indexação é mais rápida
- **O histórico de conversa é mantido** — Entre perguntas ao mesmo notebook, o chatbot tem contexto
- **Rate limiting protege sua conta** — Não tente fazer mais de 2 requests por segundo