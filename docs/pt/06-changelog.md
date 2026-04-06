---
title: "Changelog — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: pt
scan_type: full
---

# Registro de Alterações

## [Não Lançado]

### Módulo 4: Ciclo de Vida de Cadernos e Compartilhamento

- **CRUD de Cadernos**: ferramentas `notebook_delete`, `notebook_get`, `notebook_rename`
- **Resumo com IA**: `notebook_summary` — resumo gerado por IA + tópicos sugeridos
- **Compartilhamento**: `notebook_share_status`, `notebook_share_set` — alternar público/privado, visualizar usuários com acesso compartilhado
- **Leitura pós-mutação**: Operações de escrita retornam estado autoritativo confirmado
- 6 novas ferramentas MCP, 6 novos comandos CLI, 6 novos métodos de cliente
- Ciclo SDD completo (Explore → Propose → Spec → Design → Tasks → Apply → Verify → Archive)

## [0.2.0] — 2026-04-05

### Módulo 3: Geração e Download de Artefatos

- **9 tipos de artefato**: Report, Quiz, Flashcards, Audio, Infographic, Slide Deck, Mind Map, Video, Data Table
- **Gerenciamento de artefatos**: ferramentas `artifact_list`, `artifact_generate`, `artifact_delete`, `artifact_download`
- **Polling assíncrono de artefatos**: `artifact_poller.rs` — faz polling do status de geração até a conclusão
- **Parâmetros específicos por tipo**: Dificuldade, quantidade, idioma, duração, estilo, formato
- **Downloads em streaming**: Download direto a partir de URLs de armazenamento do Google
- 4 novas ferramentas MCP, 4 novos comandos CLI

## [0.1.1] — 2026-04-04

### Módulo 2: Suporte a Múltiplas Fontes

- **5 tipos de fonte**: Texto, URL, YouTube, Google Drive, Upload de arquivo
- **Auto-detecção do YouTube**: `source_add_url` detecta URLs do YouTube e utiliza a ingestão específica do YouTube
- **Integração com Google Drive**: Adicionar arquivos do Drive por ID de arquivo
- **Upload de arquivo**: Upload de arquivos locais como fontes
- **Polling assíncrono de fontes**: `source_poller.rs` — faz polling do status de indexação até a fonte estar pronta
- **Extração de módulo RPC**: `rpc/sources.rs` — construtores de payload dedicados
- 4 novas ferramentas MCP, 4 novos comandos CLI

## [0.1.0] — 2026-03-28

### Lançamento Inicial

- Servidor MCP com 4 ferramentas: `notebook_list`, `notebook_create`, `source_add`, `ask_question`
- Autenticação por navegador via Chrome CDP (comando `auth-browser`)
- Armazenamento de credenciais no keyring do SO com fallback DPAPI
- Extração de token CSRF a partir do HTML (`SNlM0e`)
- Limitação de taxa via governor (período de 2s, ~30 req/min)
- Backoff exponencial com jitter para retentativas
- Polling de fontes para verificação de disponibilidade de indexação assíncrona
- Parser defensivo para respostas RPC do Google (remoção de anti-XSSI)
- Enum de erro estruturado com auto-detecção
- Parsing de resposta em streaming para `ask_question`
- Autenticação manual via variáveis de ambiente
- Comandos CLI `verify` e `auth-status`

### Segurança

- Zero blocos `unsafe`
- `cargo-audit`: 0 vulnerabilidades (334 deps)
- TLS via rustls (sem OpenSSL)
