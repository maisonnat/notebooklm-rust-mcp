# Spec: Artifact Download

## Overview

Descarga de artefactos completados de NotebookLM. 3 estrategias: streaming HTTP (media), inline extraction (report/data_table), HTML parse (quiz/flashcard).

## Requirements

### REQ-DL-1: Download dispatcher

El sistema DEBE tener un método `download_artifact(notebook_id, artifact_id, output_path)` que:
1. Lista artefactos para encontrar el target
2. Verifica que esté completado
3. Detecta el tipo de artefacto
4. Dispatcha a la estrategia correcta

### REQ-DL-2: Streaming HTTP for media types

Audio, Video, Infographic y Slide Deck DEBEN descargarse via streaming HTTP:
- Chunk size de 64KB
- Escritura a archivo temporal `{path}.tmp`
- Rename atómico a `{path}` solo al completar exitosamente
- Si falla, el archivo temporal se elimina

### REQ-DL-3: URL extraction per media type

Las URLs de descarga DEBEN extraerse de las posiciones correctas en el artifact array:
- **Audio**: iterar `art[6][5]` buscando `item[2] == "audio/mp4"`, URL en `item[0]`
- **Video**: iterar `art[8]` buscando `item[2] == "video/mp4"`, preferir `item[1] == 4` (quality), URL en `item[0]`
- **Infographic**: forward-scan del array buscando nested `art[i][2][0][1][0]` que empiece con "http"
- **Slide Deck PDF**: `art[16][3]` (URL directa)
- **Slide Deck PPTX**: `art[16][4]` (URL directa)

### REQ-DL-4: URL domain validation

Antes de cualquier descarga HTTP, el sistema DEBE validar que la URL pertenece a un dominio Google confiable:
- `*.google.com`
- `*.googleusercontent.com`
- `*.googleapis.com`
- Protocol MUST be HTTPS

Si la URL no pasa validación, se retorna error `DownloadFailed`.

### REQ-DL-5: Inline extraction for reports

Reportes DEBEN extraerse del artifact array sin download HTTP:
- Contenido markdown en `art[7][0]`
- Escrito directamente al archivo de salida

### REQ-DL-6: Inline extraction for data tables

Data tables DEBEN extraerse del artifact array:
- Datos en `art[18]` (deeply nested rich-text)
- Parseado recursivo para extraer texto de celdas
- Escrito como CSV con BOM UTF-8 (`\xEF\xBB\xBF`) para compatibilidad Excel

### REQ-DL-7: HTML parse for quiz/flashcard

Quiz y Flashcard DEBEN descargarse en 3 pasos:
1. Llamar RPC `GET_INTERACTIVE_HTML` (v9rmvd) con `params = [artifact_id]`
2. Extraer atributo `data-app-data` del HTML response
3. Parsear el JSON del atributo (HTML-unescaped)
4. Escribir como JSON formateado

### REQ-DL-8: Mind map download

Mind maps DEBEN obtenerse del notes system:
- Via RPC `GET_NOTES_AND_MIND_MAPS` (cFji9)
- JSON string en `mind_map_data[1][1]`
- Escrito como JSON formateado

### REQ-DL-9: Delete artifact

El sistema DEBE soportar eliminar artefactos via RPC `DELETE_ARTIFACT` (V5N4be):
- `params = [[2], artifact_id]`
- Retorna éxito sin contenido

### REQ-DL-10: Artifact not found handling

Si el artifact_id no existe en el notebook, se DEBE retornar `ArtifactNotFound` error.

### REQ-DL-11: Artifact not ready handling

Si se intenta descargar un artefacto que no está completado, se DEBE retornar `ArtifactNotReady` error con el estado actual.

## Scenarios

### SC-DL-1: Download audio via streaming

```gherkin
Given a notebook with a completed audio artifact
And the audio artifact has a valid URL at art[6][5]
When I call download_artifact(notebook_id, artifact_id, "podcast.mp4")
Then a temp file "podcast.mp4.tmp" is created
And the file is downloaded in 64KB chunks
And on success, "podcast.mp4.tmp" is renamed to "podcast.mp4"
And the return value is the final path
```

### SC-DL-2: Download video via streaming

```gherkin
Given a notebook with a completed video artifact
And the video artifact has a URL with quality=4 at art[8]
When I call download_artifact(notebook_id, artifact_id, "video.mp4")
Then the highest quality URL is selected
And the file is streamed to disk
```

### SC-DL-3: Download slide deck PDF

```gherkin
Given a notebook with a completed slide deck artifact
When I call download_artifact(notebook_id, artifact_id, "slides.pdf")
Then the URL is extracted from art[16][3]
And the PDF is downloaded
```

### SC-DL-4: Download slide deck PPTX

```gherkin
Given a notebook with a completed slide deck artifact
When I call download_artifact(notebook_id, artifact_id, "slides.pptx", format: "pptx")
Then the URL is extracted from art[16][4]
And the PPTX is downloaded
```

### SC-DL-5: Download report inline

```gherkin
Given a notebook with a completed report artifact
When I call download_artifact(notebook_id, artifact_id, "report.md")
Then NO HTTP download is performed
And the markdown content from art[7][0] is written to "report.md"
```

### SC-DL-6: Download data table as CSV

```gherkin
Given a notebook with a completed data table artifact
When I call download_artifact(notebook_id, artifact_id, "data.csv")
Then NO HTTP download is performed
And the nested data from art[18] is parsed into CSV
And the file starts with UTF-8 BOM
```

### SC-DL-7: Download quiz via HTML parse

```gherkin
Given a notebook with a completed quiz artifact
When I call download_artifact(notebook_id, artifact_id, "quiz.json")
Then RPC GET_INTERACTIVE_HTML is called with the artifact_id
Then data-app-data attribute is extracted from HTML
Then the JSON is parsed and written to file
```

### SC-DL-8: Download mind map from notes

```gherkin
Given a notebook with a persisted mind map
When I call download_artifact(notebook_id, artifact_id, "mindmap.json")
Then the mind map JSON is fetched from the notes system
And written as formatted JSON
```

### SC-DL-9: URL domain validation blocks non-Google URL

```gherkin
Given a notebook with an audio artifact whose URL points to evil.com
When I call download_artifact(notebook_id, artifact_id, "file.mp4")
Then DownloadFailed error is returned
And NO HTTP request is made to evil.com
```

### SC-DL-10: Cannot download non-completed artifact

```gherkin
Given a notebook with a processing audio artifact
When I call download_artifact(notebook_id, artifact_id, "file.mp4")
Then ArtifactNotReady error is returned
And the error message includes the current status
```

### SC-DL-11: Delete artifact

```gherkin
Given a notebook with an existing artifact
When I call delete_artifact(notebook_id, artifact_id)
Then RPC DELETE_ARTIFACT (V5N4be) is called with params=[[2], artifact_id]
And success is returned
```

### SC-DL-12: Download failure cleans up temp file

```gherkin
Given a notebook with a completed audio artifact
When I call download_artifact and the HTTP connection fails mid-download
Then the temp file is deleted
And DownloadFailed error is returned
And NO partial file remains on disk
```
