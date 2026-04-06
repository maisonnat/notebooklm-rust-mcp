---
title: "Data Models — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: pt
scan_type: full
---

# Modelos de Dados

## Entidades Principais

### Notebook

A entidade de domínio central que representa um caderno do Google NotebookLM.

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `id` | `String` | Identificador UUID |
| `title` | `String` | Título do caderno definido pelo usuário |
| `sources_count` | `u32` | Número de fontes ingeridas |
| `is_owner` | `bool` | Se o usuário atual é o proprietário |
| `created_at` | `String` | Timestamp ISO da criação |

### Source

Um material de referência adicionado a um caderno para processamento por IA.

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `id` | `String` | Identificador da fonte |
| `title` | `String` | Título de exibição |
| `type` | `SourceType` | Um de: Text, URL, YouTube, Drive, File |

### Artifact

Conteúdo gerado produzido a partir das fontes do caderno.

| Campo | Tipo | Descrição |
|-------|------|-----------|
| `id` | `String` | Identificador do artefato |
| `title` | `String` | Título de exibição |
| `type` | `ArtifactType` | Tipo de conteúdo (Report, Quiz, etc.) |
| `status` | `ArtifactStatus` | Status atual de geração |
| `task_id` | `String` | ID da tarefa assíncrona de geração |
| `content_url` | `Option<String>` | URL de download (quando concluído) |
| `metadata` | `HashMap<String, String>` | Metadados específicos por tipo |

## Enums

### ArtifactType

Todos os tipos de geração de artefato suportados:

| Variante | Saída | Parâmetros |
|----------|-------|------------|
| `Report` | PDF | `instructions` (opcional) |
| `Quiz` | PDF | `difficulty` (easy/medium/hard), `quantity` (3-20) |
| `Flashcards` | PDF | `quantity` (3-20) |
| `Audio` | Arquivo de áudio | `language`, `length` (short/medium/long), `instructions` |
| `Infographic` | PNG | `detail`, `orientation`, `style` |
| `SlideDeck` | PDF/PPTX | `format`, `length` |
| `MindMap` | JSON | — |
| `Video` | Arquivo de vídeo | `format`, `style` |
| `DataTable` | PDF | — |

### ArtifactStatus

Rastreia o ciclo de vida de uma requisição de geração de artefato:

| Variante | Descrição |
|----------|-----------|
| `New` | Requisição enviada |
| `Pending` | Na fila para processamento |
| `InProgress` | Em geração no momento |
| `Completed` | Pronto para download |
| `Failed` | Erro na geração |
| `RateLimited` | Limitado pelo Google (tentar novamente depois) |

### ShareAccess

| Variante | Valor | Descrição |
|----------|-------|-----------|
| `Restricted` | 0 | Privado — apenas usuários convidados |
| `AnyoneWithLink` | 1 | Público — qualquer pessoa com o link |

### SharePermission

| Variante | Valor | Descrição |
|----------|-------|-----------|
| `Owner` | 1 | Controle total |
| `Editor` | 2 | Pode editar o conteúdo |
| `Viewer` | 3 | Acesso somente leitura |

## Tipos Compostos

### ShareStatus

```rust
ShareStatus {
    notebook_id: String,
    is_public: bool,
    access: ShareAccess,
    shared_users: Vec<SharedUser>,
    share_url: String,
}
```

### SharedUser

```rust
SharedUser {
    email: String,
    permission: SharePermission,
    display_name: String,
    avatar_url: String,
}
```

### NotebookSummary

```rust
NotebookSummary {
    summary: String,
    suggested_topics: Vec<SuggestedTopic>,
}
```

### SuggestedTopic

```rust
SuggestedTopic {
    question: String,
    prompt: String,
}
```

## Tipos de Erro

### NotebookLmError

Auto-detectado a partir de respostas HTTP:

| Variante | Gatilho | Recuperação |
|----------|---------|-------------|
| `NotFound` | 404 ou resposta vazia | Verificar validade do ID |
| `NotReady` | Artefato/fonte ainda em processamento | Fazer polling para verificar disponibilidade |
| `GenerationFailed` | Google retornou erro | Tentar novamente ou ajustar parâmetros |
| `DownloadFailed` | URL expirada ou inválida | Regenerar o artefato |
| `AuthExpired` | Token CSRF ou cookie expirado | Reautenticar |
| `RateLimited` | Resposta 429 | Aguardar e tentar novamente |
| `HttpError` | Falha HTTP genérica | Tentar novamente com backoff |
| `ParseError` | Formato de resposta inesperado | Registrar log e investigar |

## Armazenamento

Este projeto **não utiliza banco de dados**. Todo o estado reside nos servidores do Google — o servidor MCP é stateless e faz chamadas RPC individuais para cada operação.

> **[English](../en/03-data-models.md)** · **[Español](../es/03-data-models.md)**
