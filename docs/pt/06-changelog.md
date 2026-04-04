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

# Changelog (Português)

Todas as alterações_notáveis deste projeto serão documentadas neste arquivo.

## [0.1.0] - 2026-04-04

### Adicionado

- **Servidor MCP completo** com 4 ferramentas: `notebook_list`, `notebook_create`, `source_add`, `ask_question`
- **Recursos MCP**: notebooks disponíveis como URIs `notebook://{uuid}`
- **Autenticação por browser automation** via Chrome headless (CDP)
- **Autenticação manual** com DPAPI (Windows)
- **Suporte a keyring**: Windows Credential Manager / Linux Secret Service
- **Rate limiting** com governor (2 req/segundo)
- **Retry com exponential backoff** para robustez
- **Parser defensivo** para respostas RPC do Google
- **Conversation cache** para manter contexto entre perguntas
- **Source poller** para esperar indexação de fontes
- **Erros estruturados** para melhor debugging

### Detalhes Técnicos

- **Runtime**: Tokio async
- **HTTP Client**: reqwest com streaming
- **Servidor MCP**: rmcp crate
- **Browser Automation**: headless_chrome
- **Credential Storage**: windows-dpapi + keyring

### RPC IDs Descobertos

| RPC ID | Função |
|--------|---------|
| `wXbhsf` | Listar cadernos |
| `CCqFvf` | Criar caderno |
| `izAoDd` | Adicionar fonte |
| `rLM1Ne` | Obter fontes do caderno |
| `GenerateFreeFormStreamed` | Chat streaming |

### Autores

- Reverse engineering baseado em notebooklm-py
- Implementação em Rust pelo autor do projeto