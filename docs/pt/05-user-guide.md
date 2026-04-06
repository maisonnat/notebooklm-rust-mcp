---
title: "User Guide — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: pt
scan_type: full
---

# Guia do Usuário

## Qual Problema Ele Resolve?

O Google NotebookLM **não possui API pública**. Este servidor MCP faz engenharia reversa do protocolo RPC interno para permitir que agentes de IA interajam com cadernos de forma programática — criar, gerenciar, consultar e gerar conteúdo a partir de qualquer cliente compatível com MCP.

## Principais Fluxos do Usuário

### 1. Configuração Inicial

1. Compile o binário: `cargo build --release`
2. Autentique-se: `notebooklm-mcp auth-browser`
3. Verifique as credenciais: `notebooklm-mcp verify`
4. Configure seu cliente MCP (Claude Desktop, Cursor, Windsurf)

### 2. Criar e Consultar um Caderno

1. `notebook_create` — crie um caderno com um título
2. `source_add` — adicione conteúdo de texto como fonte
3. Aguarde a indexação (tratada automaticamente via source poller, 2-60s)
4. `ask_question` — consulte com respostas em streaming geradas por IA

### 3. Adicionar Múltiplos Tipos de Fonte

- `source_add` — conteúdo de texto simples
- `source_add_url` — qualquer URL (detecta YouTube automaticamente)
- `source_add_youtube` — URL de vídeo do YouTube
- `source_add_drive` — arquivo do Google Drive por ID
- `source_add_file` — upload de um arquivo local

### 4. Gerar Artefatos

```bash
# Gerar um guia de estudo
artifact-generate --notebook-id <id> --kind report

# Gerar um quiz (dificuldade média, 10 questões)
artifact-generate --notebook-id <id> --kind quiz --difficulty medium --quantity 10

# Gerar resumo em áudio em espanhol
artifact-generate --notebook-id <id> --kind audio --language es --length medium

# Baixar o artefato gerado
artifact-download --notebook-id <id> --artifact-id <artifact-id>
```

### 5. Gerenciar Cadernos

```bash
# Listar todos os cadernos
list

# Obter detalhes completos (contagem de fontes, propriedade, data de criação)
get --notebook-id <id>

# Renomear um caderno
rename --notebook-id <id> --title "Novo Título"

# Obter resumo gerado por IA e tópicos sugeridos
summary --notebook-id <id>

# Compartilhar publicamente
share-set --notebook-id <id> --public

# Verificar status de compartilhamento
share-status --notebook-id <id>

# Excluir
delete --notebook-id <id>
```

### 6. Interação com IA

A ferramenta `ask_question` suporta **respostas em streaming** — você recebe as respostas conforme são geradas, similar ao chat com o NotebookLM no navegador.

## Tipos de Artefato Suportados

| Tipo | Saída | Melhor Para |
|------|-------|-------------|
| Report | PDF | Guias de estudo, resumos |
| Quiz | PDF | Teste de conhecimento |
| Flashcards | PDF | Revisão e memorização |
| Audio | Arquivo de áudio | Resumos estilo podcast |
| Infographic | PNG | Resumos visuais |
| Slide Deck | PDF/PPTX | Apresentações |
| Mind Map | JSON | Mapeamento de conceitos |
| Video | Arquivo de vídeo | Resumos em vídeo |
| Data Table | PDF | Extração de dados tabulares |

## Dicas

- **Comece com fontes de texto** — elas indexam mais rápido (2-10s)
- **Fontes do YouTube e Drive** demoram mais para processar (até 60s)
- **Limite de taxa**: ~30 requisições/minuto — o servidor trata isso automaticamente
- **Geração de artefatos** pode levar de 30 a 120s dependendo do tipo e tamanho do conteúdo
- **Compartilhamento**: Use `share-set --public` para obter um link compartilhável para seu caderno

## Limitações

- **API com engenharia reversa** — o Google pode alterar o formato RPC interno a qualquer momento
- **Sem suporte oficial** — esta é uma ferramenta não oficial, não afiliada ao Google
- **Autenticação por cookie** — as credenciais expiram e precisam de renovação periódica
- **Servidor stateless** — nenhum estado persistente entre reinicializações (o estado reside nos servidores do Google)

> **[English](../en/05-user-guide.md)** · **[Español](../es/05-user-guide.md)**
