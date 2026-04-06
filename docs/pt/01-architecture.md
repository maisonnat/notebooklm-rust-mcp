---
title: "Architecture — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: pt
scan_type: full
---

# Arquitetura

## Visão Geral do Sistema

```
┌─────────────┐     ┌──────────────────┐     ┌────────────────────┐
│  MCP Client  │────▶│  notebooklm-mcp  │────▶│  Google NotebookLM  │
│ (Claude/etc) │◀────│  (stdio server)  │◀────│  batchexecute RPC  │
└─────────────┘     └──────────────────┘     └────────────────────┘
       │                    │
       │              ┌─────┴─────┐
       │              │  Modules   │
       │              └───────────┘
  CLI (clap)     NotebookLmClient
                  auth_helper.rs
                  auth_browser.rs
                  parser.rs
                  rpc/*.rs
                  pollers
```

## Estrutura de Módulos

```
src/
├── main.rs                # CLI entrypoint + MCP server + tool registration
├── notebooklm_client.rs   # HTTP client for NotebookLM RPC API (20+ methods)
├── parser.rs              # Defensive JSON parser for Google RPC responses
├── errors.rs              # Structured error enum with auto-detection
├── auth_browser.rs        # Chrome CDP automation + keyring storage
├── auth_helper.rs         # CSRF + session token extraction from HTML
├── artifact_poller.rs     # Async polling for artifact generation
├── source_poller.rs       # Async polling for source indexing
└── rpc/
    ├── mod.rs             # Module declarations
    ├── artifacts.rs       # Artifact types + payload builders (9 types)
    ├── sources.rs         # Source payload builders (5 types)
    └── notebooks.rs       # Notebook lifecycle types + parsers
```

## Responsabilidades dos Módulos

### `main.rs` — Ponto de Entrada e Registro de Ferramentas

O único ponto de entrada do binário. Responsabilidades:

- **Análise de argumentos CLI** via `clap` — 21 comandos mapeados para o enum `Commands`
- **Inicialização do servidor MCP** via `rmcp` — 20 métodos `#[tool]` registrados como ferramentas MCP
- **Roteamento de requisições** — comandos CLI e ferramentas MCP delegam para `NotebookLmClient`
- **Respostas em streaming** — `ask_question` retorna chunks formatados em SSE

### `notebooklm_client.rs` — Cliente HTTP

O cliente principal que encapsula todas as interações com o RPC do Google:

- `batchexecute()` — Todas as chamadas de API passam por este único método HTTP POST
- **20+ métodos** cobrindo cadernos, fontes, artefatos e compartilhamento
- Limitação de taxa via token bucket `governor` (período de 2s, ~30 req/min)
- Injeção de cookies a partir do keyring do SO ou variáveis de ambiente

### `parser.rs` — Parser JSON Defensivo

Trata o formato de resposta anti-XSSI do Google:

- `strip_antixssi()` — Remove o prefixo `)]}'` das respostas
- `extract_by_rpc_id()` — Direciona fragmentos de resposta para o handler correto por RPC ID
- **Zero `unwrap()`** em dados externos — todas as funções de parse retornam `Option`/`Result`

### `src/rpc/` — Construtores de Payload

Separados por domínio:

| Módulo | Responsabilidade |
|--------|-----------------|
| `rpc/artifacts.rs` | Enums de tipo de artefato (`ArtifactType` com 9 variantes), códigos de status, construtores de payload para geração |
| `rpc/sources.rs` | Construtores de payload para 5 tipos de fonte (texto, URL, YouTube, Drive, upload de arquivo) |
| `rpc/notebooks.rs` | Tipos de ciclo de vida de cadernos, parsers para status de compartilhamento, resumo, detalhes do caderno |

### `auth_helper.rs` — Extração de Tokens

- Analisa o token CSRF e o ID de sessão a partir de páginas HTML do NotebookLM
- Gerenciamento e validação de cookies
- Detecção de expiração de CSRF

### `auth_browser.rs` — Automação de Navegador

- Chrome headless via CDP (crate `headless_chrome`)
- Automatiza o fluxo de login do Google
- Extrai e armazena credenciais no keyring do SO
- **Correção crítica**: Utiliza `Network.getCookies` do CDP + injeção direta de headers (o Google rejeita o encaminhamento simples de cookies via HTTP)

### `errors.rs` — Tratamento Estruturado de Erros

- Enum `NotebookLmError` com auto-detecção a partir de respostas HTTP
- Cobertura: NotFound, NotReady, GenerationFailed, DownloadFailed, AuthExpired, RateLimited

### `artifact_poller.rs` — Polling Assíncrono de Artefatos

Faz polling do status de geração do artefato até conclusão ou falha com backoff exponencial.

### `source_poller.rs` — Polling Assíncrono de Fontes

Faz polling do status de indexação da fonte após a ingestão até que a fonte esteja processada.

## Padrões de Design

### Padrão Batch Execute

Toda interação com a API do Google segue este pipeline:

```
MCP Tool / CLI Command
  → NotebookLmClient.{method}()
    → batchexecute() HTTP POST to notebooklm.google.com
      → Response with anti-XSSI prefix
        → strip_antixssi()
          → extract_by_rpc_id()
            → Defensive parse → Structured result
              → Formatted string response
```

### Ferramentas MCP de Requisição-Resposta

Cada método `#[tool]` faz **uma chamada RPC** e retorna uma string formatada. Nenhum estado é mantido entre chamadas — o servidor é stateless.

### Leitura Pós-Mutação

Operações de escrita (rename, share_set) **relêem os dados confirmados** após a mutação para retornar o estado autoritativo ao chamador.

### Parsing Defensivo

Zero `unwrap()` em dados externos. Todas as funções de parse retornam `Option<T>` ou `Result<T, E>`. As respostas RPC do Google são imprevisíveis — o parser nunca presume a estrutura.

### Limitação de Taxa

Token bucket via `governor`: 2 requisições por segundo, ~30 requisições por minuto. Backoff exponencial em respostas 429.

### Armazenamento de Credenciais

Credenciais armazenadas no **keyring do SO** (via crate `keyring`) com fallback DPAPI no Windows. Nunca em variáveis de ambiente, arquivos de configuração ou logs.

## Fluxo de Dados

```
User / AI Agent
    │
    ├── CLI: clap parses args → Commands enum → match → NotebookLmClient method
    │
    └── MCP: rmcp dispatches tool call → #[tool] method → NotebookLmClient method
                                                                  │
                                                    batchexecute() POST
                                                                  │
                                                    ┌───────────┴───────────┐
                                                    │  Google RPC Response   │
                                                    │  )]}'\n + JSON array   │
                                                    └───────────┬───────────┘
                                                                  │
                                                    strip_antixssi()
                                                                  │
                                                    extract_by_rpc_id()
                                                                  │
                                                    Defensive parse
                                                                  │
                                                    Formatted string → Client
```

## Evolução Temporal

| Período | Data | Resumo |
|---------|------|--------|
| **Fundação** | 2026-03-28 | Servidor MCP inicial com 4 ferramentas, autenticação por navegador, limitação de taxa, parser defensivo |
| **Documentação v1** | 2026-04-01 → 04-02 | Documentação em inglês gerada automaticamente, README.md, CodeTour |
| **Multilíngue** | 2026-04-03 → 04-04 | Traduções ES e PT, documentação versionada no git |
| **Módulo 2: Multi-Fonte** | 2026-04-04 → 04-05 | Fontes URL, YouTube, Drive, upload de arquivo; polling assíncrono de fontes |
| **Módulo 3: Artefatos + Ciclo de Vida** | 2026-04-05 → 04-06 | 9 tipos de artefato, CRUD de cadernos, compartilhamento, ciclo SDD completo |

> **[English](../en/01-architecture.md)** · **[Español](../es/01-architecture.md)**
