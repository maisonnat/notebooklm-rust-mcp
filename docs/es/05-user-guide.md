---
title: "Guía de Usuario — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: es
scan_type: full
---

# Guía de Usuario

## ¿Qué Problema Resuelve?

Google NotebookLM **no tiene API pública**. Este servidor MCP realiza ingeniería inversa del protocolo RPC interno para permitir que los agentes de IA interactúen con notebooks de forma programática — crear, gestionar, consultar y generar contenido desde cualquier cliente compatible con MCP.

## Flujos de Usuario Principales

### 1. Configuración Inicial

1. Compilar el binario: `cargo build --release`
2. Autenticarse: `notebooklm-mcp auth-browser`
3. Verificar credenciales: `notebooklm-mcp verify`
4. Configurar tu cliente MCP (Claude Desktop, Cursor, Windsurf)

### 2. Crear y Consultar un Notebook

1. `notebook_create` — crear un notebook con un título
2. `source_add` — agregar contenido de texto como fuente
3. Esperar la indexación (manejado automáticamente vía source poller, 2-60s)
4. `ask_question` — consultar con respuestas en streaming de IA

### 3. Agregar Múltiples Tipos de Fuentes

- `source_add` — contenido de texto plano
- `source_add_url` — cualquier URL (detecta YouTube automáticamente)
- `source_add_youtube` — URL de video de YouTube
- `source_add_drive` — archivo de Google Drive por ID
- `source_add_file` — subir un archivo local

### 4. Generar Artefactos

```bash
# Generar una guía de estudio
artifact-generate --notebook-id <id> --kind report

# Generar un cuestionario (dificultad media, 10 preguntas)
artifact-generate --notebook-id <id> --kind quiz --difficulty medium --quantity 10

# Generar resumen de audio en español
artifact-generate --notebook-id <id> --kind audio --language es --length medium

# Descargar el artefacto generado
artifact-download --notebook-id <id> --artifact-id <artifact-id>
```

### 5. Gestionar Notebooks

```bash
# Listar todos los notebooks
list

# Obtener detalles completos (cantidad de fuentes, propiedad, fecha de creación)
get --notebook-id <id>

# Renombrar un notebook
rename --notebook-id <id> --title "Nuevo Título"

# Obtener resumen generado por IA y temas sugeridos
summary --notebook-id <id>

# Compartir públicamente
share-set --notebook-id <id> --public

# Verificar estado de compartido
share-status --notebook-id <id>

# Eliminar
delete --notebook-id <id>
```

### 6. Gestión de Fuentes

Además de agregar fuentes, puedes renombrarlas, eliminarlas y leer su texto extraído completo:

```bash
# Renombrar una fuente
source-rename --notebook-id <id> --source-id <source-id> --new-title "Mejor Título"

# Eliminar una fuente (idempotente — seguro de llamar incluso si ya fue eliminada)
source-delete --notebook-id <id> --source-id <source-id>

# Obtener el texto extraído completo de una fuente (útil para PDFs, páginas web, etc.)
source-get-fulltext --notebook-id <id> --source-id <source-id>
```

La herramienta `source_get_fulltext` es especialmente poderosa — devuelve el **texto completo que Google extrajo e indexó** de la fuente, incluyendo texto OCR de PDFs y contenido parseado de páginas web. Esto te permite leer el contenido del documento directamente sin hacer preguntas.

### 7. CRUD de Notas

Crea, lista y gestiona notas dentro de un notebook. Las notas aparecen en la interfaz web de NotebookLM junto con tus fuentes:

```bash
# Crear una nota
note-create --notebook-id <id> --title "Hallazgos Clave" --content "Insights importantes de la investigación..."

# Listar todas las notas activas (excluye notas eliminadas con soft-delete)
note-list --notebook-id <id>

# Eliminar una nota (soft-delete)
note-delete --notebook-id <id> --note-id <note-id>
```

> **Nota**: La creación de notas es un proceso de dos pasos internamente (crear vacía → actualizar con contenido). La herramienta MCP maneja esto automáticamente.

### 8. Historial de Chat

Recupera el historial completo de conversación desde los servidores de Google para cualquier notebook:

```bash
# Obtener los últimos 20 turnos (por defecto)
chat-history --notebook-id <id>

# Obtener los últimos 50 turnos
chat-history --notebook-id <id> --limit 50
```

Devuelve los turnos en **orden cronológico** (del más antiguo al más reciente), con cada turno mostrando el rol (`user` o `assistant`) y el texto. Esto es útil para:

- Revisar qué preguntas se hicieron previamente
- Construir contexto para preguntas de seguimiento
- Exportar registros de conversación

### 9. Investigación Profunda

Lanza el motor de investigación autónoma de Google desde cualquier cliente MCP. La herramienta **bloquea hasta que la investigación se completa** (hasta 300s), luego importa automáticamente las fuentes descubiertas al notebook:

```bash
# Iniciar una investigación profunda
research --notebook-id <id> --query "Comparar arquitecturas de transformers para tareas de NLP"

# Con timeout personalizado
research --notebook-id <id> --query "Aplicaciones de computación cuántica" --timeout-secs 600
```

El flujo de investigación:
1. Inicia una tarea de investigación en los servidores de Google
2. Realiza polling cada 5 segundos para verificar completitud
3. Al completar, importa todas las fuentes web descubiertas al notebook
4. Devuelve un resumen de las fuentes descubiertas

> **Tip**: La investigación profunda puede tardar 2-5 minutos. Si se alcanza el timeout, obtienes un resultado parcial con las fuentes descubiertas hasta ese momento.

### 10. Interacción con IA

La herramienta `ask_question` soporta **respuestas en streaming** — recibes las respuestas a medida que se generan, similar a chatear con NotebookLM en el navegador. La herramienta recupera automáticamente el ID de conversación activa desde los servidores de Google para mantener la continuidad de la conversación.

## Tipos de Artefactos Soportados

| Tipo | Salida | Mejor Para |
|------|--------|------------|
| Report | PDF | Guías de estudio, resúmenes |
| Quiz | PDF | Evaluación de conocimientos |
| Flashcards | PDF | Repaso y memorización |
| Audio | Archivo de audio | Resúmenes estilo podcast |
| Infographic | PNG | Resúmenes visuales |
| Slide Deck | PDF/PPTX | Presentaciones |
| Mind Map | JSON | Mapeo de conceptos |
| Video | Archivo de video | Resúmenes en video |
| Data Table | PDF | Extracción de datos tabulares |

## Consejos

- **Empieza con fuentes de texto** — indexan más rápido (2-10s)
- **Las fuentes de YouTube y Drive** tardan más en procesarse (hasta 60s)
- **Límite de tasa**: ~30 solicitudes/minuto — el servidor lo maneja automáticamente
- **La generación de artefactos** puede tardar 30-120s dependiendo del tipo y tamaño del contenido
- **Compartido**: Usa `share-set --public` para obtener un enlace compartible para tu notebook

## Limitaciones

- **API con ingeniería inversa** — Google puede cambiar el formato RPC interno en cualquier momento
- **Sin soporte oficial** — esta es una herramienta no oficial, no afiliada a Google
- **Autenticación por cookies** — las credenciales expiran y necesitan renovación periódica
- **Servidor sin estado** — no hay estado persistente entre reinicios (el estado reside en los servidores de Google)

> **[English](../en/05-user-guide.md)** · **[Português](../pt/05-user-guide.md)**
