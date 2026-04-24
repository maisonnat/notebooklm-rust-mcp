---
title: "Setup Guide — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: pt
scan_type: full
---

# Guia de Configuração

## Pré-requisitos

| Requisito | Versão | Notas |
|-----------|--------|-------|
| **Rust** | 1.70+ (edição 2024) | [rustup.rs](https://rustup.rs/) |
| **Chrome** | Qualquer versão recente | Necessário para o comando `auth-browser` |
| **SO** | Windows, macOS, Linux | Keyring do SO para armazenamento de credenciais |

## Build

```bash
# Clonar
git clone <repo-url>
cd notebooklm-rust-mcp

# Build do binário de release
cargo build --release

# Localização do binário
# ./target/release/notebooklm-mcp
```

## Autenticação

### Opção 1: Autenticação por Navegador (Recomendada)

```bash
./target/release/notebooklm-mcp auth-browser
```

Isso abre o Chrome, navega até o Google NotebookLM e aguarda seu login. Após a autenticação, as credenciais são salvas no **keyring do SO** — sem necessidade de variáveis de ambiente.

### Opção 2: Variáveis de Ambiente

Se preferir gerenciamento manual de credenciais:

```bash
export NOTEBOOKLM_COOKIE="__Secure-1PSID=...;__Secure-1PSIDTS=..."
export NOTEBOOKLM_CSRF="your_csrf_token"
export NOTEBOOKLM_SID="your_session_id"
```

### Verificar Autenticação

```bash
./target/release/notebooklm-mcp auth-status
# ou
./target/release/notebooklm-mcp verify
```

## Executando como Servidor MCP

O servidor comunica-se via **stdio** (entrada/saída padrão). Configure seu cliente MCP:

### Claude Desktop

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/caminho/absoluto/para/notebooklm-mcp",
      "args": []
    }
  }
}
```

### Cursor / Windsurf

Mesma configuração no arquivo de configurações MCP.

### CLI Direto

Todas as operações estão disponíveis como comandos CLI:

```bash
./target/release/notebooklm-mcp list
./target/release/notebooklm-mcp create --title "Meu Caderno"
./target/release/notebooklm-mcp artifact-generate --notebook-id <id> --kind report
```

## Testes

```bash
# Executar todos os testes unitários
cargo test

# 329 testes, 5 ignorados (testes E2E requerem credenciais ativas)
```

## Armazenamento de Credenciais

As credenciais são armazenadas no keyring do SO:

| SO | Backend | Detalhes |
|----|---------|----------|
| **Windows** | DPAPI | Fallback via crate `windows-dpapi` |
| **macOS** | Keychain | Acesso nativo ao keychain |
| **Linux** | Secret Service | D-Bus `org.freedesktop.secrets` |

> **Nota de segurança:** As credenciais **nunca** são gravadas em variáveis de ambiente, arquivos de configuração ou logs. O keyring do SO é o único mecanismo de armazenamento.

## Solução de Problemas

| Problema | Solução |
|----------|---------|
| `auth-browser` falha | Verifique se o Chrome está instalado e acessível |
| CSRF expirado | Execute `auth-browser` novamente para renovar as credenciais |
| Limite de taxa (429) | Aguarde alguns minutos — o servidor possui limitação de taxa integrada (~30 req/min) |
| "Nenhuma credencial encontrada" | Execute `auth-browser` primeiro, ou defina as variáveis de ambiente manualmente |

> **[English](../en/04-setup.md)** · **[Español](../es/04-setup.md)**
