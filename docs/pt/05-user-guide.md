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

### 6. Gerenciamento de Fontes

Além de adicionar fontes, você pode renomear, excluir e ler o texto extraído completo:

```bash
# Renomear uma fonte
source-rename --notebook-id <id> --source-id <source-id> --new-title "Título Melhor"

# Excluir uma fonte (idempotente — seguro chamar mesmo que já tenha sido excluída)
source-delete --notebook-id <id> --source-id <source-id>

# Obter o texto completo extraído de uma fonte (útil para PDFs, páginas web, etc.)
source-get-fulltext --notebook-id <id> --source-id <source-id>
```

A ferramenta `source_get_fulltext` é particularmente poderosa — ela retorna o **texto completo que o Google extraiu e indexou** da fonte, incluindo texto de OCR de PDFs e conteúdo analisado de páginas web. Isso permite que você leia o conteúdo do documento diretamente sem precisar fazer perguntas.

### 7. CRUD de Notas

Crie, liste e gerencie notas dentro de um caderno. As notas aparecem na interface web do NotebookLM junto com suas fontes:

```bash
# Criar uma nota
note-create --notebook-id <id> --title "Descobertas Importantes" --content "Insights importantes da pesquisa..."

# Listar todas as notas ativas (exclui notas excluídas temporariamente)
note-list --notebook-id <id>

# Excluir uma nota (exclusão temporária)
note-delete --notebook-id <id> --note-id <note-id>
```

> **Nota**: A criação de notas é um processo de duas etapas internamente (criar vazio → atualizar com conteúdo). A ferramenta MCP trata isso automaticamente.

### 8. Histórico de Chat

Recupere o histórico completo de conversas dos servidores do Google para qualquer caderno:

```bash
# Obter as últimas 20 mensagens (padrão)
chat-history --notebook-id <id>

# Obter as últimas 50 mensagens
chat-history --notebook-id <id> --limit 50
```

Retorna as mensagens em **ordem cronológica** (mais antigas primeiro), com cada mensagem mostrando o papel (`user` ou `assistant`) e o texto. Isso é útil para:

- Revisar quais perguntas foram feitas anteriormente
- Construir contexto para perguntas de acompanhamento
- Exportar logs de conversação

### 9. Deep Research

Inicie o mecanismo de pesquisa autônoma do Google a partir de qualquer cliente MCP. A ferramenta **bloqueia até que a pesquisa seja concluída** (até 300s) e, em seguida, importa automaticamente as fontes descobertas no caderno:

```bash
# Iniciar uma investigação de deep research
research --notebook-id <id> --query "Comparar arquiteturas de transformers para tarefas de NLP"

# Com timeout personalizado
research --notebook-id <id> --query "Aplicações de computação quântica" --timeout-secs 600
```

O fluxo de pesquisa:
1. Inicia uma tarefa de pesquisa nos servidores do Google
2. Faz polling a cada 5 segundos para verificar a conclusão
3. Quando concluído, importa todas as fontes web descobertas no caderno
4. Retorna um resumo das fontes descobertas

> **Dica**: O deep research pode levar de 2 a 5 minutos. Se o timeout for atingido, você recebe um resultado parcial com as fontes descobertas até o momento.

### 10. Interação com IA

A ferramenta `ask_question` suporta **respostas em streaming** — você recebe as respostas conforme são geradas, similar ao chat com o NotebookLM no navegador. A ferramenta recupera automaticamente o ID da conversa ativa dos servidores do Google para manter a continuidade da conversa.

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