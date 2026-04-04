---
title: "Setup — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
last_commit: "b467e15"
scan_type: full
tags: [rust, mcp, documentation]
audience: users
---

# Instalação (Português)

## Requisitos

- **Rust** 1.70+ (edição 2024)
- **Windows** (para DPAPI) — no Linux/macOS tem fallback
- **Google Chrome** (para autenticação automática)

## Compilação

```bash
# Clonar o repositório
git clone <repo-url>
cd notebooklm-rust-mcp

# Compilar
cargo build --release

# O binário fica em target/release/notebooklm-mcp.exe
```

## Autenticação

### Método 1: Chrome headless (recomendado)

Este método abre uma janela do Chrome para você fazer login:

```bash
./target/release/notebooklm-mcp auth-browser
```

**Fluxo:**
1. Abre janela do Chrome
2. Você faz login na sua conta do Google
3. O script detecta as cookies automaticamente
4. São salvas no Windows Credential Manager

### Método 2: Manual

Se você preferir não usar Chrome:

1. Abra DevTools (F12) em notebooklm.google.com
2. Application → Cookies → Copie o valor de `__Secure-1PSID` e `__Secure-1PSIDTS`
3. Faça um GET para notebooklm.google.com e procure por `"SNlM0e":"..."` no HTML
4. Execute:

```bash
notebooklm-mcp auth \
  --cookie "__Secure-1PSID=xxx; __Secure-1PSIDTS=yyy" \
  --csrf "SNlM0e_xxx"
```

## Configuração do Cliente MCP

### Cursor

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "path/to/notebooklm-mcp.exe"
    }
  }
}
```

### Claude Desktop

Em `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "path/to/notebooklm-mcp.exe"
    }
  }
}
```

### Windsurf

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "path/to/notebooklm-mcp.exe"
    }
  }
}
```

## Verificação

```bash
# Ver estado da autenticação
notebooklm-mcp auth-status

# Testar conexão
notebooklm-mcp verify
```

Se você ver "Cadernos encontrados: [...]" ✓

## Variáveis de Ambiente

Não há variáveis de ambiente necessárias — as credenciais são salvas em:

- **Windows**: Windows Credential Manager (via keyring) ou DPAPI
- **Fallback**: `~/.notebooklm-mcp/session.bin`

## Atualização de Credenciais

As cookies do Google expiram frequentemente. Se você ver erros de autenticação:

```bash
# Regenerar credenciais
notebooklm-mcp auth-browser

# Ou manual
notebooklm-mcp auth --cookie "nova_cookie" --csrf "novo_csrf"
```