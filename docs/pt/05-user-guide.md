---
title: "Guia do Usuario — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: pt
scan_type: full
---

# Guia do Usuario

## Qual Problema Resolve?

O Google NotebookLM nao possui uma API publica. Este servidor MCP permite que agentes de IA criem cadernos, adicionem fontes e consultem documentos de forma programatica — tudo atraves do Model Context Protocol.

## Fluxos Principais

### Configuracao Inicial

1. `cargo build --release`
2. `notebooklm-mcp auth-browser`
3. `notebooklm-mcp verify`
4. Configure o cliente MCP

### Criar e Consultar

1. `notebook_create` — crie um caderno com um titulo
2. `source_add` — adicione conteudo de texto como fonte
3. Aguarde a indexacao (tratado automaticamente, de 2 a 60 segundos)
4. `ask_question` — consulte com respostas geradas por IA

### Historico de Conversa

- A primeira pergunta cria um ID de conversa
- Perguntas subsequentes reutilizam a mesma conversa
- O historico e enviado a cada consulta para contexto

## Limitacoes

- Somente fontes de texto (sem suporte a PDF/URL/YouTube via MCP ainda)
- Estado em memoria (reinicia ao reiniciar)
- Limite de taxa de ~30 req/min
- API obtida por engenharia reversa (pode parar de funcionar)
