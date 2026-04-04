---
title: "Registro de Alteracoes — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: pt
scan_type: full
---

# Registro de Alteracoes

## [0.1.0] — 2026-04-04

### Adicionado
- Servidor MCP com 4 ferramentas: `notebook_list`, `notebook_create`, `source_add`, `ask_question`
- Autenticacao baseada em browser via Chrome CDP (comando `auth-browser`)
- Armazenamento de credenciais no keyring do SO com fallback DPAPI
- Extracao de token CSRF do HTML (`SNlM0e`)
- Limitacao de taxa via governor (periodo de 2s, ~30 req/min)
- Backoff exponencial com jitter para retentativas
- Polling de fontes para verificacao de prontidao de indexacao assincrona
- Cache de conversa (em memoria, por caderno)
- Parser defensivo para respostas RPC do Google (remocao de anti-XSSI)
- Enum de erros estruturado com deteccao automatica
- Parsing de resposta streaming para `ask_question`
- Autenticacao manual via flags `--cookie` / `--csrf`
- Comando `verify` para validacao E2E
- Comando `auth-status`
- Testes unitarios em todos os modulos

### Seguranca
- Zero blocos `unsafe`
- `cargo-audit`: 0 vulnerabilidades (305 dependencias)
- TLS via rustls (sem OpenSSL)
