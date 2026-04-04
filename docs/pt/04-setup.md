---
title: "Configuracao — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: pt
scan_type: full
---

# Configuracao

## Prerequisitos

| Requisito | Versao | Notas |
|-----------|--------|-------|
| Rust | 1.70+ | Edicao 2024 |
| Chrome | Mais recente | Para `auth-browser` |
| Conta Google | — | Acesso ao NotebookLM |

## Instalacao

```bash
git clone https://github.com/maisonnat/notebooklm-rust-mcp
cd notebooklm-rust-mcp
cargo build --release
```

Binario: `./target/release/notebooklm-mcp`

## Autenticacao

### Autenticacao via Browser (Recomendada)

```bash
./target/release/notebooklm-mcp auth-browser
```

1. O Chrome inicia em modo headless
2. Complete o login no Google
3. Cookies extraidos via CDP
4. Armazenados no keyring do SO

### Autenticacao Manual

```bash
./target/release/notebooklm-mcp auth --cookie "..." --csrf "..."
```

Criptografado com DPAPI em `~/.notebooklm-mcp/session.bin`.

### Verificar Status

```bash
./target/release/notebooklm-mcp auth-status
```

## Verificacao

```bash
./target/release/notebooklm-mcp verify
```

## Configuracao do Cliente MCP

Configure seu cliente para iniciar o binario com transporte stdio:

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/caminho/para/notebooklm-mcp"
    }
  }
}
```

## Testes

```bash
cargo test
```

## Resolucao de Problemas

| Problema | Solucao |
|----------|---------|
| "Servidor nao autenticado" | Execute `auth-browser` |
| Chrome nao encontrado | Instale o Chrome ou use `auth` manual |
| Sessao expirada | Execute `auth-browser` novamente |
| Limitado por taxa | Tratado automaticamente (limite de 30 req/min) |
