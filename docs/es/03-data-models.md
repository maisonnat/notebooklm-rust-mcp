---
title: "Modelos de Datos — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: es
scan_type: full
---

# Modelos de Datos

## Entidades Principales

### Notebook

La entidad de dominio central que representa un notebook de Google NotebookLM.

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | `String` | Identificador UUID |
| `title` | `String` | Título del notebook definido por el usuario |
| `sources_count` | `u32` | Cantidad de fuentes ingeridas |
| `is_owner` | `bool` | Si el usuario actual es el propietario |
| `created_at` | `String` | Marca temporal ISO de creación |

### Source

Un material de referencia agregado a un notebook para procesamiento por IA.

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | `String` | Identificador de la fuente |
| `title` | `String` | Título para mostrar |
| `type` | `SourceType` | Uno de: Text, URL, YouTube, Drive, File |

### Artifact

Contenido generado a partir de las fuentes de un notebook.

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `id` | `String` | Identificador del artefacto |
| `title` | `String` | Título para mostrar |
| `type` | `ArtifactType` | Tipo de contenido (Report, Quiz, etc.) |
| `status` | `ArtifactStatus` | Estado actual de generación |
| `task_id` | `String` | ID de la tarea de generación async |
| `content_url` | `Option<String>` | URL de descarga (cuando está completado) |
| `metadata` | `HashMap<String, String>` | Metadatos específicos del tipo |

## Enums

### ArtifactType

Todos los tipos de generación de artefactos soportados:

| Variante | Salida | Parámetros |
|----------|--------|------------|
| `Report` | PDF | `instructions` (opcional) |
| `Quiz` | PDF | `difficulty` (easy/medium/hard), `quantity` (3-20) |
| `Flashcards` | PDF | `quantity` (3-20) |
| `Audio` | Archivo de audio | `language`, `length` (short/medium/long), `instructions` |
| `Infographic` | PNG | `detail`, `orientation`, `style` |
| `SlideDeck` | PDF/PPTX | `format`, `length` |
| `MindMap` | JSON | — |
| `Video` | Archivo de video | `format`, `style` |
| `DataTable` | PDF | — |

### ArtifactStatus

Rastrea el ciclo de vida de una solicitud de generación de artefacto:

| Variante | Descripción |
|----------|-------------|
| `New` | Solicitud enviada |
| `Pending` | En cola para procesamiento |
| `InProgress` | Generando actualmente |
| `Completed` | Listo para descarga |
| `Failed` | Error en la generación |
| `RateLimited` | Limitado por Google (reintentar más tarde) |

### ShareAccess

| Variante | Valor | Descripción |
|----------|-------|-------------|
| `Restricted` | 0 | Privado — solo usuarios invitados |
| `AnyoneWithLink` | 1 | Público — cualquiera con el enlace |

### SharePermission

| Variante | Valor | Descripción |
|----------|-------|-------------|
| `Owner` | 1 | Control total |
| `Editor` | 2 | Puede editar contenido |
| `Viewer` | 3 | Acceso de solo lectura |

## Tipos Compuestos

### ShareStatus

```rust
ShareStatus {
    notebook_id: String,
    is_public: bool,
    access: ShareAccess,
    shared_users: Vec<SharedUser>,
    share_url: String,
}
```

### SharedUser

```rust
SharedUser {
    email: String,
    permission: SharePermission,
    display_name: String,
    avatar_url: String,
}
```

### NotebookSummary

```rust
NotebookSummary {
    summary: String,
    suggested_topics: Vec<SuggestedTopic>,
}
```

### SuggestedTopic

```rust
SuggestedTopic {
    question: String,
    prompt: String,
}
```

## Tipos de Error

### NotebookLmError

Autodetectado desde respuestas HTTP:

| Variante | Disparador | Recuperación |
|----------|------------|--------------|
| `NotFound` | 404 o respuesta vacía | Verificar validez del ID |
| `NotReady` | Artefacto/fuente aún en procesamiento | Hacer polling para verificar disponibilidad |
| `GenerationFailed` | Google devolvió error | Reintentar o ajustar parámetros |
| `DownloadFailed` | URL expirada o inválida | Regenerar el artefacto |
| `AuthExpired` | Token CSRF o cookie expirados | Reautenticarse |
| `RateLimited` | Respuesta 429 | Esperar y reintentar |
| `HttpError` | Falla HTTP genérica | Reintentar con retroceso |
| `ParseError` | Formato de respuesta inesperado | Registrar y investigar |

## Almacenamiento

Este proyecto **no utiliza base de datos**. Todo el estado reside en los servidores de Google — el servidor MCP es sin estado y realiza llamadas RPC individuales para cada operación.

> **[English](../en/03-data-models.md)** · **[Português](../pt/03-data-models.md)**
