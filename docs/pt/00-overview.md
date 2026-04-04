---
title: "Visao Geral — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: pt
scan_type: full
---

# Visao Geral

> **Servidor MCP nao oficial para Google NotebookLM** — escrito em Rust com zero codigo unsafe.

## O que e?

O NotebookLM MCP Server e um servidor do [Model Context Protocol](https://modelcontextprotocol.io) que permite que agentes de IA (Claude, Cursor, Windsurf, etc.) interajam com cadernos do Google NotebookLM de forma programatica.

**Funcionalidades principais:**
- Criar, listar e gerenciar cadernos
- Adicionar fontes de texto aos cadernos
- Fazer perguntas e receber respostas geradas por IA com historico de conversa
- Polling automatico de fontes (aguarda indexacao antes de consultar)

## Inicio Rapido

```bash
# Build
cargo build --release

# Autenticacao (metodo recomendado)
./target/release/notebooklm-mcp auth-browser

# Verificar conexao
./target/release/notebooklm-mcp verify
```

Em seguida, configure seu cliente MCP para apontar para o binario (transporte stdio).

## Stack Tecnologica

| Componente | Tecnologia |
|------------|------------|
| Linguagem | Rust (edicao 2024) |
| Runtime Assincrono | Tokio |
| Framework MCP | rmcp 1.2 |
| Cliente HTTP | reqwest 0.12 (rustls-tls) |
| Parser de CLI | clap 4.4 |
| Limitacao de Taxa | governor 0.6 |
| Autenticacao via Browser | headless_chrome 1 (CDP) |
| Armazenamento de Credenciais | keyring 3 + fallback DPAPI |

## Status

> **Experimental** — Este projeto faz engenharia reversa das APIs internas do Google. Use por sua conta e risco.

- Sem suporte oficial de API por parte do Google
- Endpoints RPC internos podem mudar sem aviso previo
- Cookies de sessao expiram com frequencia

## Licenca

MIT
