# Spec: Artifact Discovery

## Overview

Listado y monitoreo de estado de artefactos en un notebook via RPC `LIST_ARTIFACTS` (gArtLc).

## Requirements

### REQ-AD-1: List all artifacts

El sistema DEBE listar todos los artefactos de un notebook via RPC `LIST_ARTIFACTS` (gArtLc) y parsear la respuesta posicional en structs tipados.

### REQ-AD-2: Artifact struct

Cada artefacto listado DEBE contener:
- `id: String` — artifact ID (de `art[0]`)
- `title: String` — título (de `art[1]`)
- `kind: ArtifactType` — tipo mapeado (de `art[2]`, via type code → enum)
- `status: ArtifactStatus` — estado (de `art[4]`: PROCESSING=1, PENDING=2, COMPLETED=3, FAILED=4)
- `created_at: Option<u64>` — timestamp unix (de `art[15][0]`)
- `error: Option<String>` — razón de error (de `art[3]`, solo cuando failed)

### REQ-AD-3: Type code mapping

Los type codes internos DEBEN mapearse a enum `ArtifactType`:
- 1 → Audio, 2 → Report, 3 → Video, 5 → MindMap, 7 → Infographic, 8 → SlideDeck, 9 → DataTable
- Type 4 → distinción por variant: variant 1 = Flashcards, variant 2 = Quiz (de `art[9][1][0]`)
- Type 6 → UNKNOWN (unused)

### REQ-AD-4: Polling by task_id

NO existe un endpoint poll-by-ID. El sistema DEBE:
1. Llamar LIST_ARTIFACTS para obtener todos los artefactos
2. Escanear los resultados buscando el `task_id` especificado
3. Retornar `GenerationStatus` con el estado encontrado

### REQ-AD-5: Media ready gate

Para tipos de media (audio, video, infographic, slide_deck), incluso si `status == COMPLETED`, el sistema DEBE verificar que la URL de descarga esté poblada antes de declarar completion. Si la URL está vacía, el estado se degrada a `PROCESSING` para continuar el polling.

URL checks por tipo:
- Audio: `art[6][5]` no vacío
- Video: `art[8]` contiene URL con prefijo "http"
- Infographic: forward-scan del array para URL con prefijo "http"
- Slide Deck: `art[16][3]` no vacío

### REQ-AD-6: Exponential backoff polling

`wait_for_completion` DEBE implementar exponential backoff:
- `initial_interval`: 2s
- `max_interval`: 10s
- `timeout`: configurable, default 300s (5 min)
- Backoff: `interval = min(interval * 2, max_interval)`

### REQ-AD-7: Filter parameter

LIST_ARTIFACTS DEBE enviar el filtro: `'NOT artifact.status = "ARTIFACT_STATUS_SUGGESTED"'` como tercer parámetro del params array.

## Scenarios

### SC-AD-1: List artifacts returns typed results

```gherkin
Given a notebook with 3 artifacts: 1 audio (completed), 1 video (processing), 1 report (failed)
When I call list_artifacts(notebook_id)
Then I get a Vec<Artifact> with 3 elements
And artifact[0].kind = Audio
And artifact[0].status = Completed
And artifact[1].kind = Video
And artifact[1].status = Processing
And artifact[2].kind = Report
And artifact[2].status = Failed
And artifact[2].error contains the error reason
```

### SC-AD-2: Quiz vs Flashcard type mapping

```gherkin
Given a notebook with a type-4 artifact with variant=2
When I call list_artifacts(notebook_id)
Then the artifact has kind = Quiz

Given a notebook with a type-4 artifact with variant=1
When I call list_artifacts(notebook_id)
Then the artifact has kind = Flashcards
```

### SC-AD-3: Poll status finds task_id

```gherkin
Given a notebook with a generating artifact with task_id="abc123"
When I call poll_status(notebook_id, "abc123")
Then I get GenerationStatus with task_id="abc123" and status="in_progress"
```

### SC-AD-4: Poll status returns not_found

```gherkin
Given a notebook with no artifact matching task_id="nonexistent"
When I call poll_status(notebook_id, "nonexistent")
Then I get GenerationStatus with status="not_found"
```

### SC-AD-5: Media ready gate prevents premature completion

```gherkin
Given a notebook with an audio artifact with status=COMPLETED but art[6][5] is empty
When I call poll_status(notebook_id, task_id)
Then the status is degraded to "in_progress" (PROCESSING)
And polling continues
```

### SC-AD-6: Media ready gate allows completion when URL present

```gherkin
Given a notebook with an audio artifact with status=COMPLETED and art[6][5] contains a valid URL
When I call poll_status(notebook_id, task_id)
Then the status is "completed"
```

### SC-AD-7: Wait for completion with timeout

```gherkin
Given a notebook with an artifact that takes 60s to generate
When I call wait_for_completion(notebook_id, task_id, timeout=120)
Then polling starts at 2s intervals
And intervals double: 2s, 4s, 8s, 10s, 10s, ...
And after completion, returns GenerationStatus { status: "completed" }
```

### SC-AD-8: Wait for completion timeout

```gherkin
Given a notebook with an artifact that never completes
When I call wait_for_completion(notebook_id, task_id, timeout=10)
Then after 10 seconds, returns GenerationStatus { status: "in_progress" }
And no error is thrown
```
