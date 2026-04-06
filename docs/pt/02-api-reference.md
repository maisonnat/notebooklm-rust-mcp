---
title: "API Reference — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: pt
---

# Referência de API

## Ferramentas MCP

### Gerenciamento de Cadernos

#### `notebook_list`

Lista todos os cadernos do usuário.

**Parâmetros:** Nenhum

**Retorna:** Lista formatada de cadernos com ID e título.

#### `notebook_create`

Cria um novo caderno.

**Parâmetros:**
- `title` (string, obrigatório): O título do caderno

**Retorna:** ID do caderno criado.

#### `notebook_delete`

Exclui um caderno por ID. Idempotente — não gera erro se o caderno não existir.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno

**Retorna:** Mensagem de confirmação.

#### `notebook_get`

Obtém detalhes completos do caderno, incluindo contagem de fontes, propriedade e data de criação.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno

**Retorna:** Detalhes do caderno (título, ID, contagem de fontes, status de proprietário, data de criação).

#### `notebook_rename`

Renomeia um caderno. Retorna os detalhes atualizados do caderno.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `new_title` (string, obrigatório): Novo título para o caderno

**Retorna:** Detalhes atualizados do caderno.

#### `notebook_summary`

Obtém o resumo gerado por IA e os tópicos sugeridos para um caderno.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno

**Retorna:** Texto do resumo e lista de tópicos sugeridos (pares pergunta + prompt).

#### `notebook_share_status`

Obtém a configuração de compartilhamento de um caderno.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno

**Retorna:** Status público/privado, nível de acesso, lista de usuários com quem foi compartilhado com e-mails e permissões, URL de compartilhamento.

#### `notebook_share_set`

Alterna a visibilidade do caderno para público ou privado. Retorna o status de compartilhamento atualizado.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `public` (boolean, obrigatório): `true` para público, `false` para privado

**Retorna:** Status de compartilhamento atualizado.

### Gerenciamento de Fontes

#### `source_add`

Adiciona uma fonte de texto a um caderno.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `title` (string, obrigatório): Título da fonte
- `content` (string, obrigatório): Conteúdo textual da fonte

**Retorna:** ID da fonte.

#### `source_add_url`

Adiciona uma fonte de URL a um caderno. Detecta automaticamente URLs do YouTube e utiliza o fluxo de ingestão específico do YouTube.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `url` (string, obrigatório): URL a ser adicionada
- `title` (string, opcional): Título personalizado (extraído automaticamente se omitido)

**Retorna:** ID da fonte.

#### `source_add_youtube`

Adiciona um vídeo do YouTube como fonte.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `url` (string, obrigatório): URL do vídeo do YouTube
- `title` (string, opcional): Título personalizado

**Retorna:** ID da fonte.

#### `source_add_drive`

Adiciona um arquivo do Google Drive como fonte.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `file_id` (string, obrigatório): ID do arquivo no Google Drive
- `title` (string, opcional): Título personalizado

**Retorna:** ID da fonte.

#### `source_add_file`

Faz upload de um arquivo local como fonte.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `file_path` (string, obrigatório): Caminho do arquivo local para upload
- `title` (string, opcional): Título personalizado

**Retorna:** ID da fonte.

### Gerenciamento de Artefatos

#### `artifact_list`

Lista todos os artefatos de um caderno.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno

**Retorna:** Lista de artefatos com ID, título, tipo e status.

#### `artifact_generate`

Gera um artefato. Parâmetros específicos por tipo são adicionados com base no parâmetro `kind`.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `kind` (string, obrigatório): Tipo de artefato
- Parâmetros adicionais dependem de `kind` (ver abaixo)

**Retorna:** ID do artefato para polling de conclusão.

**Tipos de Artefato:**

| Kind | Parâmetros Adicionais | Formato de Saída |
|------|----------------------|-----------------|
| `report` | `instructions` (opcional) | PDF |
| `quiz` | `difficulty` (easy/medium/hard), `quantity` (3-20) | PDF |
| `flashcards` | `quantity` (3-20) | PDF |
| `audio` | `language` (en/es/etc), `length` (short/medium/long), `instructions` (opcional) | Arquivo de áudio |
| `infographic` | `detail` (brief/standard), `orientation` (landscape/portrait), `style` (default/professional/casual) | PNG |
| `slide_deck` | `format` (pdf/pptx), `length` (short/medium/long) | PDF/PPTX |
| `mind_map` | — | JSON |
| `video` | `format` (cinematic/documentary), `style` (default/dramatic/cinematic) | Arquivo de vídeo |
| `data_table` | — | PDF |

#### `artifact_delete`

Exclui um artefato de um caderno.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `artifact_id` (string, obrigatório): ID do artefato a ser excluído

**Retorna:** Mensagem de confirmação.

#### `artifact_download`

Baixa um artefato no formato apropriado.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `artifact_id` (string, obrigatório): ID do artefato
- `output` (string, opcional): Caminho do arquivo de saída (padrão: nome gerado automaticamente no diretório atual)

**Retorna:** Caminho do arquivo do artefato baixado.

### Interação com IA

#### `ask_question`

Faz uma pergunta sobre um caderno com resposta em streaming.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `question` (string, obrigatório): Pergunta a ser feita

**Retorna:** Resposta em texto com streaming (chunks).

### Gerenciamento de Fontes

#### `source_delete`

Exclui uma fonte de um caderno. Idempotente — não gera erro se a fonte não existir.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `source_id` (string, obrigatório): ID da fonte a ser excluída

**Retorna:** Mensagem de confirmação.

#### `source_rename`

Renomeia uma fonte em um caderno.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `source_id` (string, obrigatório): ID da fonte a ser renomeada
- `new_title` (string, obrigatório): Novo título para a fonte

**Retorna:** Mensagem de confirmação.

#### `source_get_fulltext`

Obtém o texto completo indexado de uma fonte (extraído pelo Google de PDFs, páginas web, etc.). Útil para ler o conteúdo do documento sem fazer perguntas.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `source_id` (string, obrigatório): ID da fonte

**Retorna:** Conteúdo textual completo extraído.

### Notas

#### `note_create`

Cria uma nota em um caderno. As notas ficam visíveis na interface web do NotebookLM.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `title` (string, obrigatório): Título da nota
- `content` (string, obrigatório): Conteúdo da nota

**Retorna:** ID da nota.

#### `note_list`

Lista todas as notas ativas de um caderno (exclui notas excluídas temporariamente).

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno

**Retorna:** Lista de notas com ID e título.

#### `note_delete`

Exclui uma nota de um caderno (exclusão temporária).

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `note_id` (string, obrigatório): ID da nota a ser excluída

**Retorna:** Mensagem de confirmação.

### Histórico de Chat

#### `chat_history`

Obtém o histórico oficial de conversas dos servidores do Google para um caderno. Retorna as mensagens em ordem cronológica (mais antigas primeiro).

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `limit` (integer, opcional): Número máximo de mensagens a recuperar (padrão: 20)

**Retorna:** Lista de mensagens com papel ("user" ou "assistant") e texto.

### Deep Research

#### `research_deep_dive`

Inicia uma investigação de deep research usando o mecanismo de pesquisa autônoma do Google. Bloqueia até completar (timeout de até 300s) e, em seguida, importa as fontes descobertas no caderno.

**Parâmetros:**
- `notebook_id` (string, obrigatório): UUID do caderno
- `query` (string, obrigatório): Query de pesquisa
- `timeout_secs` (integer, opcional): Tempo máximo de espera em segundos (padrão: 300)

**Retorna:** Resumo das fontes descobertas.

## Comandos CLI

| Comando | Flags | Descrição |
|---------|-------|-----------|
| `auth-browser` | — | Autenticar via Chrome headless |
| `auth-status` | — | Verificar estado da autenticação |
| `verify` | — | Validação E2E das credenciais |
| `list` | — | Listar todos os cadernos |
| `create` | `--title` | Criar um caderno |
| `delete` | `--notebook-id` | Excluir um caderno |
| `get` | `--notebook-id` | Obter detalhes do caderno |
| `rename` | `--notebook-id` `--title` | Renomear um caderno |
| `summary` | `--notebook-id` | Obter resumo com IA |
| `share-status` | `--notebook-id` | Obter configuração de compartilhamento |
| `share-set` | `--notebook-id` `--public` / `--private` | Alternar compartilhamento |
| `source-add` | `--notebook-id` `--title` `--content` | Adicionar fonte de texto |
| `source-add-url` | `--notebook-id` `--url` `--title` | Adicionar fonte de URL |
| `source-add-youtube` | `--notebook-id` `--url` `--title` | Adicionar fonte do YouTube |
| `source-add-drive` | `--notebook-id` `--file-id` `--title` | Adicionar fonte do Drive |
| `source-add-file` | `--notebook-id` `--file-path` `--title` | Upload de arquivo como fonte |
| `source-delete` | `--notebook-id` `--source-id` | Excluir uma fonte |
| `source-rename` | `--notebook-id` `--source-id` `--new-title` | Renomear uma fonte |
| `source-get-fulltext` | `--notebook-id` `--source-id` | Obter texto completo da fonte |
| `artifact-list` | `--notebook-id` | Listar artefatos |
| `artifact-generate` | `--notebook-id` `--kind` + flags específicas por tipo | Gerar artefato |
| `artifact-delete` | `--notebook-id` `--artifact-id` | Excluir artefato |
| `artifact-download` | `--notebook-id` `--artifact-id` `--output` | Baixar artefato |
| `note-create` | `--notebook-id` `--title` `--content` | Criar uma nota |
| `note-list` | `--notebook-id` | Listar notas |
| `note-delete` | `--notebook-id` `--note-id` | Excluir uma nota |
| `chat-history` | `--notebook-id` `--limit` | Obter histórico de chat |
| `research` | `--notebook-id` `--query` `--timeout-secs` | Deep research |
| `ask` | `--notebook-id` `--question` | Fazer pergunta |

## Configuração

### Variáveis de Ambiente

| Variável | Tipo | Descrição |
|----------|------|-----------|
| `NOTEBOOKLM_COOKIE` | string | Cookie de autenticação do Google (do keyring do SO se não definido) |
| `NOTEBOOKLM_CSRF` | string | Token CSRF (do keyring do SO se não definido) |
| `NOTEBOOKLM_SID` | string | ID de sessão (do keyring do SO se não definido) |

### Configuração do Cliente MCP

Para usar este servidor com um cliente MCP (Cursor, Claude Desktop, Windsurf):

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
