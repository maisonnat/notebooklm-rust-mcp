---
title: "Security Posture — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: pt
scan_type: full
---

# Postura de Segurança

## Resumo Executivo

| Área | Status | Detalhes |
|------|--------|----------|
| Segurança de Memória | **LIMPO** | Zero blocos `unsafe` |
| Cadeia de Suprimentos | **LIMPA** | 0 vulnerabilidades (334 deps) |
| Armazenamento de Credenciais | **SEGURO** | Keyring do SO + fallback DPAPI |
| TLS | **SEGURO** | rustls (sem dependências C) |
| Autenticação | **MODERADO** | API do Google com engenharia reversa |

## Autenticação e Autorização

### Tratamento de Cookies

- Extraídos via Chrome DevTools Protocol (CDP)
- Alvos: `__Secure-1PSID` e `__Secure-1PSIDTS` (cookies HttpOnly)
- Nunca visíveis em JavaScript — requer automação de navegador
- **Correção crítica**: `Network.getCookies` do CDP retorna nome+valor mas NÃO os atributos SameSite/Secure/Path/Domain. O servidor do Google rejeita requisições sem os atributos de cookie adequados. Solução: injeção direta de headers a partir dos cookies do CDP com reconstrução de atributos.

### Token CSRF

- Token `SNlM0e` extraído do HTML do NotebookLM via regex
- Não é estático — requer renovação em erros 400
- Detecção de expiração integrada ao tratamento de erros

### Armazenamento de Credenciais

```
Principal:   Keyring do SO (Windows Credential Manager / macOS Keychain / Linux Secret Service)
             Serviço: "notebooklm-mcp" | Entrada: "google-credentials"
Fallback:    Arquivo criptografado via DPAPI (somente Windows)
```

> As credenciais **nunca** são armazenadas em variáveis de ambiente, arquivos de configuração ou logs.

## Segurança de Memória

- **Zero blocos `unsafe`** em todo o código-fonte
- Todo acesso a arrays utiliza helpers defensivos (`get_string_at`, `get_uuid_at`)
- Verificação com `cargo-audit`: **0 vulnerabilidades** em 334 dependências de crates
- Sem `unwrap()` em dados externos (respostas RPC)

## Tratamento de Dados Sensíveis

| Dado | Tratamento | Armazenamento |
|------|------------|---------------|
| Cookies do Google | Extraídos via CDP, nunca registrados em log | Keyring do SO |
| Token CSRF | Extraído do HTML, validado por requisição | Keyring do SO |
| ID de Sessão | Extraído do HTML | Keyring do SO |
| Conteúdo do caderno | Processado via RPC, nunca armazenado localmente | Servidores do Google |
| Consultas do usuário | Enviadas ao RPC do Google, não registradas em log | Não persistido |

## Limitação de Taxa e Retentativas

| Mecanismo | Valor | Finalidade |
|-----------|-------|------------|
| Cota de token bucket | Período de 2s (~30 req/min) | Prevenir abuso da API |
| Backoff exponencial | Jitter 150-600ms | Evitar efeito de manada |
| Máximo de retentativas | 3 tentativas | Resiliência |
| Limite de backoff | 30 segundos | Prevenir esperas infinitas |

## Cadeia de Suprimentos

- Todas as dependências do **crates.io** (repositório oficial de pacotes Rust)
- TLS via **rustls** (Rust puro, sem dependências OpenSSL/C)
- Automação do Chrome via **headless_chrome** (protocolo CDP, sem Selenium)
- Sem scripts de build que baixam binários externos

## Segurança de Downloads

Os downloads de artefatos validam:

- **Lista de domínios permitidos**: Apenas `googleapis.com` e `googleusercontent.com`
- **Exigência de esquema**: Somente HTTPS (sem downloads via HTTP)
- **Streaming**: Sem gravação de arquivos temporários para conteúdo em memória

## Riscos Conhecidos

| Risco | Severidade | Mitigação |
|-------|------------|-----------|
| Alterações na API do Google | Alto | Parsing defensivo, erros estruturados, camada RPC modular |
| Expiração de cookies | Médio | Auto-detecção, reautenticação fácil via `auth-browser` |
| Sem API oficial | Médio | Design modular para fácil adaptação a mudanças |
| Protocolo com engenharia reversa | Médio | Todo parsing é defensivo — formatos inesperados retornam erros |

> **[English](../en/07-security-posture.md)** · **[Español](../es/07-security-posture.md)**
