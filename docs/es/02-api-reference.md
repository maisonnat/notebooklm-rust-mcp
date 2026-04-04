---
title: "Referencia de API — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: es
scan_type: full
---

# Referencia de API

## Herramientas MCP

### `notebook_list`

Lista todos los notebooks disponibles en la cuenta.

**Parametros:** Ninguno

**Devuelve:** Cadena formateada con la lista de notebooks.

```
Notebooks: [Notebook { id: "uuid", title: "My Notebook" }, ...]
```

---

### `notebook_create`

Crea un nuevo notebook con un titulo.

**Parametros:**

| Campo | Tipo | Descripcion |
|-------|------|-------------|
| `title` | `string` | Titulo para el nuevo notebook |

**Devuelve:** ID del notebook creado.

```
Cuaderno creado. ID: <uuid>
```

---

### `source_add`

Agrega una fuente de texto a un notebook.

**Parametros:**

| Campo | Tipo | Descripcion |
|-------|------|-------------|
| `notebook_id` | `string` | UUID del notebook destino |
| `title` | `string` | Titulo para la fuente |
| `content` | `string` | Contenido de texto de la fuente |

**Devuelve:** ID de la fuente.

```
Fuente anadida. ID: <uuid>
```

---

### `ask_question`

Hace una pregunta a un notebook. La pregunta se responde usando todas las fuentes del notebook como contexto.

**Parametros:**

| Campo | Tipo | Descripcion |
|-------|------|-------------|
| `notebook_id` | `string` | UUID del notebook destino |
| `question` | `string` | Pregunta a realizar |

**Devuelve:** Texto de respuesta generado por IA.

> **Nota:** El notebook debe tener al menos una fuente indexada. Si no hay fuentes disponibles, devuelve un error.

---

## Recursos MCP

### `notebook://<uuid>`

Recurso de lectura para cada notebook.

**Respuesta:** JSON con `id`, `title` y `uri`.

---

## Comandos CLI

### `auth`

Autenticacion manual con cookie y token CSRF.

```bash
notebooklm-mcp auth --cookie "YOUR_COOKIE" --csrf "YOUR_CSRF"
```

Las credenciales se encriptan con DPAPI y se almacenan en `~/.notebooklm-mcp/session.bin`.

### `auth-browser` (Recomendado)

Autenticacion basada en browser via Chrome headless.

```bash
notebooklm-mcp auth-browser
```

Abre Chrome para login de Google, extrae cookies via CDP, almacena en el keyring del SO.

### `auth-status`

Verifica el estado de autenticacion.

```bash
notebooklm-mcp auth-status
```

Muestra si Chrome esta disponible y si hay credenciales almacenadas.

### `verify`

Test de validacion E2E contra la API de NotebookLM.

```bash
notebooklm-mcp verify
```

Crea un notebook de prueba para verificar que la conexion funciona.

### `ask`

Hace una pregunta a un notebook directamente desde la CLI.

```bash
notebooklm-mcp ask --notebook-id <uuid> --question "Tu pregunta"
```

### `add-source`

Agrega una fuente de texto a un notebook desde la CLI.

```bash
notebooklm-mcp add-source --notebook-id <uuid> --title "Titulo de la fuente" --content "Contenido de la fuente"
```

---

## Transporte

El servidor MCP se comunica por **stdio** (stdin/stdout). Configurá tu cliente MCP para que lance el binario y se comunique via I/O estandar.
