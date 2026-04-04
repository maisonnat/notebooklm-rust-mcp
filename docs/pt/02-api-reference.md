---
title: "Referencia de API — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: pt
scan_type: full
---

# Referencia de API

## Ferramentas MCP

### `notebook_list`

Lista todos os cadernos disponiveis na conta.

**Parametros:** Nenhum

**Retorna:** String formatada com a lista de cadernos.

```
Notebooks: [Notebook { id: "uuid", title: "Meu Caderno" }, ...]
```

---

### `notebook_create`

Cria um novo caderno com um titulo.

**Parametros:**

| Campo | Tipo | Descricao |
|-------|------|-----------|
| `title` | `string` | Titulo do novo caderno |

**Retorna:** ID do caderno criado.

```
Caderno criado. ID: <uuid>
```

---

### `source_add`

Adiciona uma fonte de texto a um caderno.

**Parametros:**

| Campo | Tipo | Descricao |
|-------|------|-----------|
| `notebook_id` | `string` | UUID do caderno de destino |
| `title` | `string` | Titulo da fonte |
| `content` | `string` | Conteudo de texto da fonte |

**Retorna:** ID da fonte.

```
Fonte adicionada. ID: <uuid>
```

---

### `ask_question`

Faz uma pergunta a um caderno. A pergunta e respondida utilizando todas as fontes do caderno como contexto.

**Parametros:**

| Campo | Tipo | Descricao |
|-------|------|-----------|
| `notebook_id` | `string` | UUID do caderno de destino |
| `question` | `string` | Pergunta a ser feita |

**Retorna:** Texto da resposta gerada por IA.

> **Nota:** O caderno deve ter pelo menos uma fonte indexada. Se nao houver fontes disponiveis, retorna um erro.

---

## Recursos MCP

### `notebook://<uuid>`

Recurso de leitura para cada caderno.

**Resposta:** JSON com `id`, `title` e `uri`.

---

## Comandos CLI

### `auth`

Autenticacao manual com cookie e token CSRF.

```bash
notebooklm-mcp auth --cookie "YOUR_COOKIE" --csrf "YOUR_CSRF"
```

As credenciais sao criptografadas com DPAPI e armazenadas em `~/.notebooklm-mcp/session.bin`.

### `auth-browser` (Recomendado)

Autenticacao baseada em browser via Chrome headless.

```bash
notebooklm-mcp auth-browser
```

Abre o Chrome para login no Google, extrai cookies via CDP e armazena no keyring do SO.

### `auth-status`

Verifica o estado da autenticacao.

```bash
notebooklm-mcp auth-status
```

Mostra se o Chrome esta disponivel e se ha credenciais armazenadas.

### `verify`

Teste de validacao E2E contra a API do NotebookLM.

```bash
notebooklm-mcp verify
```

Cria um caderno de teste para verificar se a conexao esta funcionando.

### `ask`

Faz uma pergunta a um caderno diretamente pela CLI.

```bash
notebooklm-mcp ask --notebook-id <uuid> --question "Sua pergunta"
```

### `add-source`

Adiciona uma fonte de texto a um caderno pela CLI.

```bash
notebooklm-mcp add-source --notebook-id <uuid> --title "Titulo da Fonte" --content "Conteudo da fonte"
```

---

## Transporte

O servidor MCP se comunica via **stdio** (stdin/stdout). Configure seu cliente MCP para iniciar o binario e comunicar via I/O padrao.
