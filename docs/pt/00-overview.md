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

# NotebookLM MCP Server — Overview (Português)

## O que é

**NotebookLM MCP Server** é um servidor MCP (Model Context Protocol) não oficial que permite interagir com Google NotebookLM através de uma interface padronizada.

Basicamente, permite:
- **Listar cadernos** existentes na sua conta do NotebookLM
- **Criar novos cadernos** com título personalizado
- **Adicionar fontes de texto** a qualquer caderno
- **Fazer perguntas ao chatbot de IA** do NotebookLM

Tudo isso de qualquer cliente MCP (Cursor, Windsurf, Claude Desktop, etc.).

## Technology stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Language | Rust | 1.70+ |
| Framework | rmcp (MCP) | 1.2 |
| HTTP Client | reqwest | 0.12 |
| Async Runtime | tokio | 1.50 |
| Build | Cargo | latest |
| CLI | clap | 4.4 |

## Quick start

> [!TIP]
> These steps get you from zero to running in under 5 minutes.

1. **Clone e build**
   ```bash
   git clone https://github.com/maisonnat/notebooklm-rust-mcp
   cd notebooklm-rust-mcp
   cargo build --release
   ```

2. **Autenticar (recomendado)**
   ```bash
   ./target/release/notebooklm-mcp auth-browser
   ```

3. **Verificar conexão**
   ```bash
   ./target/release/notebooklm-mcp verify
   ```

Para instalação completa, ver [[04-setup]].

## Repository structure

```
notebooklm-rust-mcp/
├── src/                    — Código fonte
│   ├── main.rs              — Entry point + CLI + MCP server
│   ├── notebooklm_client.rs — Cliente HTTP + rate limiting
│   ├── auth_browser.rs      — Autenticação Chrome headless
│   ├── auth_helper.cs       — Extração CSRF
│   ├── parser.rs            — Parser defensivo RPC
│   ├── source_poller.rs     — Polling de fontes
│   ├── conversation_cache.rs — Cache de conversa
│   └── errors.rs            — Erros estruturados
├── docs/                   — Documentação
├── Cargo.toml              — Dependências
└── Cargo.lock
```

## License & maintainers

- **License:** MIT
- **Repository:** https://github.com/maisonnat/notebooklm-rust-mcp

> [!WARNING] Experimental
> Este projeto faz engenharia reversa de APIs internas do Google. Use sob seu próprio risco.
