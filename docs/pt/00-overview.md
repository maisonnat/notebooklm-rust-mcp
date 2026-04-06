---
title: "Overview — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: pt
scan_type: full
---

# Visão Geral

> **Servidor MCP não oficial para Google NotebookLM** — escrito em Rust com zero blocos `unsafe`.

## O Que É?

NotebookLM MCP Server é um servidor [Model Context Protocol](https://modelcontextprotocol.io) que permite que agentes de IA (Claude, Cursor, Windsurf, etc.) interajam com cadernos do Google NotebookLM de forma programática.

O Google NotebookLM **não possui API pública**. Este servidor faz engenharia reversa do protocolo RPC interno (o mesmo endpoint `batchexecute` que a interface web do NotebookLM utiliza) para conectar agentes de IA às operações de cadernos.

## Capacidades Principais

| Domínio | Ferramentas | Descrição |
|---------|-------------|-----------|
| **Gerenciamento de Cadernos** | 8 ferramentas | Criar, listar, renomear, excluir, obter detalhes, resumo com IA, status de compartilhamento, alternar compartilhamento |
| **Gerenciamento de Fontes** | 5 ferramentas | Adicionar fontes de texto, URL, YouTube, Google Drive ou arquivo local |
| **Geração de Artefatos** | 4 ferramentas | Gerar 9 tipos de artefatos, listar, excluir e baixar |
| **Interação com IA** | 1 ferramenta | Fazer perguntas com respostas em streaming |
| **Autenticação** | 2 comandos CLI | Autenticação via navegador com Chrome CDP, armazenamento de credenciais no keyring do SO |

**Total: 20 ferramentas MCP + 21 comandos CLI**

## Tipos de Artefato

| Tipo | Formato de Saída | Descrição |
|------|-----------------|-----------|
| `report` | PDF | Guia de estudo ou relatório a partir do conteúdo do caderno |
| `quiz` | PDF | Questionário de múltipla escolha (3-20 questões, dificuldade ajustável) |
| `flashcards` | PDF | Baralho de flashcards (3-20 cartões) |
| `audio` | Arquivo de áudio | Resumo em áudio estilo podcast (idioma e duração configuráveis) |
| `infographic` | PNG | Infográfico visual (paisagem/retrato, múltiplos estilos) |
| `slide_deck` | PDF / PPTX | Slides de apresentação (curto/médio/longo) |
| `mind_map` | JSON | Mapa mental estruturado de conceitos |
| `video` | Arquivo de vídeo | Resumo em vídeo (estilo cinematográfico/documentário) |
| `data_table` | PDF | Extração de dados tabulares |

## Início Rápido

```bash
# Build
cargo build --release

# Autenticar (abre o Chrome, salva credenciais no keyring do SO)
./target/release/notebooklm-mcp auth-browser

# Usar como servidor MCP (stdio)
./target/release/notebooklm-mcp
```

### Configuração do Cliente MCP

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/caminho/para/notebooklm-mcp",
      "args": []
    }
  }
}
```

## Stack Tecnológica

| Componente | Tecnologia |
|------------|------------|
| Linguagem | Rust (edição 2024) |
| Framework MCP | [rmcp](https://github.com/amodelotrust/rmcp) |
| Cliente HTTP | [reqwest](https://crates.io/crates/reqwest) (rustls TLS) |
| CLI | [clap](https://crates.io/crates/clap) |
| Runtime Assíncrono | [tokio](https://tokio.rs/) |
| Limitação de Taxa | [governor](https://crates.io/crates/governor) (token bucket, ~30 req/min) |
| Armazenamento de Credenciais | [keyring](https://crates.io/crates/keyring) (keyring do SO + fallback DPAPI) |
| Autenticação por Navegador | [headless_chrome](https://crates.io/crates/headless-chrome) (CDP) |

## Segurança

| Métrica | Valor |
|---------|-------|
| Blocos `unsafe` | **0** |
| Vulnerabilidades (cargo-audit) | **0** (334 dependências) |
| Backend TLS | rustls (sem OpenSSL) |
| Armazenamento de credenciais | Keyring do SO (nunca em variáveis de ambiente ou arquivos) |
| Licença | MIT |

## Estatísticas do Projeto

- **11 arquivos fonte** em `src/`
- **329 testes unitários** (5 testes E2E ignorados)
- **0 avisos do clippy**
- **Desenvolvimento Orientado a Especificações** com 8 domínios de especificação
- **4 módulos de desenvolvimento** concluídos e arquivados

## Documentação

| Documento | Descrição | Público |
|-----------|-----------|---------|
| [Arquitetura](./01-architecture.md) | Módulos, padrões de design, fluxo de dados | Engenheiros |
| [Referência de API](./02-api-reference.md) | Ferramentas MCP, comandos CLI, configuração | Integradores |
| [Modelos de Dados](./03-data-models.md) | Entidades de domínio e definições de tipos | Engenheiros |
| [Guia de Configuração](./04-setup.md) | Build, instalação, configuração | Usuários |
| [Guia do Usuário](./05-user-guide.md) | Fluxos de trabalho comuns e dicas | Usuários |
| [Registro de Alterações](./06-changelog.md) | Histórico de releases | Todos |
| [Postura de Segurança](./07-security-posture.md) | Autenticação, segurança de memória, auditoria | Engenheiros |

> **[English](../en/00-overview.md)** · **[Español](../es/00-overview.md)**
