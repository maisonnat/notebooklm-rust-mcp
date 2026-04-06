---
title: "Security Posture — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.3.1"
last_updated: "2026-04-06"
lang: pt
scan_type: full
---

# Postura de Segurança

## Resumo Executivo

| Área | Status | Detalhes |
|------|--------|----------|
| Segurança de Memória | **LIMPO** | Zero blocos `unsafe` |
| Cadeia de Suprimentos | **LIMPA** | 0 vulnerabilidades (335 deps) |
| Armazenamento de Credenciais | **SEGURO** | Keyring do SO + fallback DPAPI |
| TLS | **SEGURO** | rustls (sem dependências C) |
| Autenticação | **MODERADO** | API do Google com engenharia reversa |
| Anti-detecção | **REFORÇADO** | Headers estilo Chrome, circuit breaker, renovação automática |

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
- **Módulo 6**: Renovação automática em erro de autenticação — renovação silenciosa de CSRF+SID antes que o usuário perceba

### Armazenamento de Credenciais

```
Principal:   Keyring do SO (Windows Credential Manager / macOS Keychain / Linux Secret Service)
             Serviço: "notebooklm-mcp" | Entrada: "google-credentials"
Fallback:    Arquivo criptografado via DPAPI (somente Windows)
```

> As credenciais **nunca** são armazenadas em variáveis de ambiente, arquivos de configuração ou logs.

## Reforço Anti-detecção (Módulo 6)

### Falsificação de Impressão Digital do Navegador

Todas as requisições ao endpoint `batchexecute` do Google incluem headers HTTP estilo Chrome para evitar detecção pelo WAF:

| Header | Valor | Finalidade |
|--------|-------|------------|
| `User-Agent` | Chrome/136 no Windows | Identificação do navegador |
| `sec-fetch-dest` | `empty` | Tipo de requisição XHR |
| `sec-fetch-mode` | `cors` | Modo CORS |
| `sec-fetch-site` | `same-origin` | Verificação de origem |
| `sec-ch-ua` | Chromium;v=136 | Client hint |
| `sec-ch-ua-mobile` | `?0` | Modo desktop |
| `sec-ch-ua-platform` | "Windows" | Hint do SO |
| `origin` | https://notebooklm.google.com | Header de origem |
| `referer` | https://notebooklm.google.com/ | Header de referer |
| `accept` | `*/*` | Tipo de conteúdo |
| `accept-language` | en-US,en;q=0.9 | Preferência de idioma |

### Circuit Breaker

Impede sobrecarga do Google com credenciais expiradas:

```
CLOSED ──(3 erros de autenticação)──→ OPEN ──(60s de resfriamento)──→ HALF-OPEN
  ↑                                                              │
  └──────────(teste bem-sucedido)────────────────────────────────┘
                                                                   │
                                       (teste falhou)──────────→ OPEN
```

- **Limiar**: 3 erros de autenticação consecutivos (401/400/403)
- **Resfriamento**: 60 segundos antes de permitir uma requisição de teste
- **Ação do usuário**: "Execute `notebooklm-mcp auth-browser` para reautenticar"
- **Implementação**: `AtomicU32` para contador sem lock, `Mutex<Option<Instant>>` para timestamp

### Renovação Automática de CSRF

Quando um erro de autenticação é detectado, o servidor automaticamente:

1. Adquire um lock `tokio::sync::Mutex` (impede renovações concorrentes)
2. Chama `refresh_tokens()` para obter um novo CSRF + Session ID do Google
3. Tenta a requisição original exatamente uma vez com os novos tokens
4. Se a renovação falhar → incrementa o contador do circuit breaker → propaga o erro

### Suporte a Retry-After

Quando o Google retorna HTTP 429 com um header `Retry-After`:

- Analisa segundos inteiros (`"5"` → 5000ms) e formatos de data HTTP
- Usa o atraso especificado pelo servidor em vez do backoff calculado
- Limita a 120 segundos para evitar esperas excessivas
- Recorre ao backoff exponencial se o header estiver ausente

## Limitação de Taxa e Retentativas

| Mecanismo | Valor | Finalidade |
|-----------|-------|------------|
| Cota de token bucket | Período de 2s (~30 req/min) | Prevenir abuso da API |
| Jitter pré-requisição | 800-2000ms | Simular temporização humana |
| Backoff exponencial | 2^x segundos (2, 4, 8, 16...) | Espaçamento de retentativas |
| Jitter de backoff | 800-2000ms | Evitar efeito de manada |
| Máximo de retentativas | 3 tentativas | Resiliência |
| Limite de backoff | 30 segundos | Prevenir esperas infinitas |
| Retry-After | Especificado pelo servidor (até 120s) | Respeitar orientação do Google |

## Segurança de Memória

- **Zero blocos `unsafe`** em todo o código-fonte
- Todo acesso a arrays utiliza helpers defensivos (`get_string_at`, `get_uuid_at`)
- Verificação com `cargo-audit`: **0 vulnerabilidades** em 335 dependências de crates
- Sem `unwrap()` em dados externos (respostas RPC)

## Tratamento de Dados Sensíveis

| Dado | Tratamento | Armazenamento |
|------|------------|---------------|
| Cookies do Google | Extraídos via CDP, nunca registrados em log | Keyring do SO |
| Token CSRF | Extraído do HTML, renovado automaticamente ao expirar | Keyring do SO |
| ID de Sessão | Extraído do HTML, renovado automaticamente ao expirar | Keyring do SO |
| Conteúdo do caderno | Processado via RPC, nunca armazenado localmente | Servidores do Google |
| Consultas do usuário | Enviadas ao RPC do Google, não registradas em log | Não persistido |

## Cadeia de Suprimentos

- Todas as dependências do **crates.io** (repositório oficial de pacotes Rust)
- TLS via **rustls** (Rust puro, sem dependências OpenSSL/C)
- Automação do Chrome via **headless_chrome** (protocolo CDP, sem Selenium)
- Sem scripts de build que baixam binários externos
- Crate **httpdate** para parsing de datas Retry-After (zero dependências)

## Segurança de Downloads

Os downloads de artefatos validam:

- **Lista de domínios permitidos**: Apenas `googleapis.com` e `googleusercontent.com`
- **Exigência de esquema**: Somente HTTPS (sem downloads via HTTP)
- **Streaming**: Sem gravação de arquivos temporários para conteúdo em memória

## Riscos Conhecidos

| Risco | Severidade | Mitigação |
|-------|------------|-----------|
| Alterações na API do Google | Alto | Parsing defensivo, erros estruturados, camada RPC modular |
| Expiração de cookies | Médio | Auto-detecção, renovação automática, reautenticação fácil via `auth-browser` |
| Sem API oficial | Médio | Design modular para fácil adaptação a mudanças |
| Protocolo com engenharia reversa | Médio | Todo parsing é defensivo — formatos inesperados retornam erros |
| Detecção por WAF | Baixo | Headers estilo Chrome, jitter humano, circuit breaker |

> **[English](../en/07-security-posture.md)** · **[Español](../es/07-security-posture.md)**
