# Spec: Artifact Generation

## Overview

Generación de los 10 tipos de artefacto de NotebookLM via RPC `CREATE_ARTIFACT` (R7cb6c) y `GENERATE_MIND_MAP` (yyryJe).

## Requirements

### REQ-AG-1: ArtifactConfig type-safe enum

El sistema DEBE usar un enum `ArtifactConfig` donde cada variante contiene solo los parámetros válidos para ese tipo de artefacto. El compilador DEBE rechazar configuraciones inválidas en compile-time.

**Rationale**: Quiz requiere `difficulty`, Audio no. Video tiene `style`, Report no. El sistema de tipos de Rust previene errores de configuración.

### REQ-AG-2: Single generation dispatcher

TODOS los tipos de artefacto (excepto Mind Map) DEBEN usar el mismo método `generate_artifact(notebook_id, config)` que dispatcha al RPC `CREATE_ARTIFACT` (R7cb6c). La variante del enum determina el payload posicional.

### REQ-AG-3: Positional payload correctness

Cada variante de `ArtifactConfig` DEBE generar el array posicional exacto que espera la API de Google. Las posiciones del config array varían por tipo:
- Audio: config en índice 6
- Report: config en índice 7
- Video: config en índice 8
- Quiz/Flashcard: config en índice 9
- Infographic: config en índice 14
- Slide Deck: config en índice 16
- Data Table: config en índice 18

### REQ-AG-4: Mind Map two-step generation

Mind Map DEBE usar un método dedicado `generate_mind_map()` que:
1. Llama RPC `GENERATE_MIND_MAP` (yyryJe) con el payload especial (no CREATE_ARTIFACT)
2. Persiste el resultado via RPC `CREATE_NOTE` (CYK0Xb)
3. Retorna `MindMapResult { note_id, mind_map_data }`

### REQ-AG-5: Source ID formatting

Los source IDs DEBEN ser formateados en dos formatos según posición en el payload:
- Triple-nested: `[[[sid1]], [[sid2]], ...]` — posición principal del array
- Double-nested: `[[sid1], [sid2], ...]` — dentro del config wrapper

### REQ-AG-6: Rate limiting handling

Cuando la API devuelve `rpc_code == "USER_DISPLAYABLE_ERROR"`, el sistema DEBE:
- Capturar el error sin lanzar exception
- Retornar `GenerationStatus { status: "failed", error_code: "USER_DISPLAYABLE_ERROR" }`
- Permitir al caller distinguir rate limiting retryable de errores reales via `is_rate_limited`

### REQ-AG-7: Generation result parsing

La respuesta de CREATE_ARTIFACT DEBE ser parseada para extraer:
- `result[0][0]` → `task_id` (same as artifact_id)
- `result[0][4]` → initial status code (ArtifactStatus)

### REQ-AG-8: Quiz vs Flashcard distinction

Type code 4 DEBE distinguir Quiz de Flashcards por el variant code en `config[9][1][0]`:
- Variant 1 = Flashcards
- Variant 2 = Quiz

Los payloads DEBEN respetar el orden invertido de params:
- Quiz: `[quantity, difficulty]` en posición `[7]`
- Flashcards: `[difficulty, quantity]` en posición `[6]`

## Scenarios

### SC-AG-1: Generate audio overview

```gherkin
Given a notebook with ready sources
When I call generate_artifact with ArtifactConfig::Audio { format: DeepDive, length: Default, instructions: "focus on chapter 3", language: "en", source_ids: ["src1"] }
Then the RPC CREATE_ARTIFACT is called with type_code=1
And the config array is at position 6
And config[6][1][0] = "focus on chapter 3"
And config[6][1][1] = 2 (DEFAULT length)
And config[6][1][4] = "en"
And config[6][1][6] = 1 (DEEP_DIVE)
And the response returns GenerationStatus with task_id
```

### SC-AG-2: Generate video explainer

```gherkin
Given a notebook with ready sources
When I call generate_artifact with ArtifactConfig::Video { format: Explainer, style: Classic, instructions: null, language: "es", source_ids: ["src1"] }
Then the RPC CREATE_ARTIFACT is called with type_code=3
And the config array is at position 8
And config[8][2][1] = "es"
And config[8][2][4] = 1 (EXPLAINER)
And config[8][2][5] = 3 (CLASSIC style)
```

### SC-AG-3: Generate cinematic video

```gherkin
Given a notebook with ready sources
When I call generate_artifact with ArtifactConfig::CinematicVideo { instructions: "documentary about AI", language: "en", source_ids: ["src1"] }
Then the RPC CREATE_ARTIFACT is called with type_code=3
And config[8][2][4] = 3 (CINEMATIC format)
And config[8][2][5] = null (no style for cinematic)
```

### SC-AG-4: Generate report

```gherkin
Given a notebook with ready sources
When I call generate_artifact with ArtifactConfig::Report { format: BriefingDoc, language: "en", source_ids: ["src1"], extra_instructions: null }
Then the RPC CREATE_ARTIFACT is called with type_code=2
And the config array is at position 7
And config[7][1][0] = "Briefing Doc" (template title)
And config[7][1][5] contains the built-in briefing doc prompt
```

### SC-AG-5: Generate custom report

```gherkin
Given a notebook with ready sources
When I call generate_artifact with ArtifactConfig::Report { format: Custom { prompt: "Create a white paper about..." }, language: "en", source_ids: ["src1"] }
Then config[7][1][0] = "Custom Report"
And config[7][1][5] = "Create a white paper about..."
```

### SC-AG-6: Generate quiz

```gherkin
Given a notebook with ready sources
When I call generate_artifact with ArtifactConfig::Quiz { difficulty: Hard, quantity: Standard, instructions: "focus on vocabulary", source_ids: ["src1"] }
Then the RPC CREATE_ARTIFACT is called with type_code=4
And the config array is at position 9
And config[9][1][0] = 2 (VARIANT = quiz)
And config[9][1][7] = [2, 3] (STANDARD quantity, HARD difficulty)
```

### SC-AG-7: Generate flashcards (reversed order!)

```gherkin
Given a notebook with ready sources
When I call generate_artifact with ArtifactConfig::Flashcards { difficulty: Easy, quantity: Fewer, instructions: null, source_ids: ["src1"] }
Then the RPC CREATE_ARTIFACT is called with type_code=4
And the config array is at position 9
And config[9][1][0] = 1 (VARIANT = flashcards)
And config[9][1][6] = [1, 1] (EASY difficulty, FEWER quantity — REVERSED!)
```

### SC-AG-8: Generate mind map (two-step)

```gherkin
Given a notebook with ready sources
When I call generate_mind_map with source_ids: ["src1", "src2"]
Then RPC GENERATE_MIND_MAP (yyryJe) is called with the special payload
And RPC CREATE_NOTE (CYK0Xb) is called to persist the result
And the response contains MindMapResult { note_id: String, mind_map_data: Value }
```

### SC-AG-9: Rate limited generation

```gherkin
Given a notebook
When I call generate_artifact and the API returns USER_DISPLAYABLE_ERROR
Then NO exception is thrown
And GenerationStatus is returned with status="failed"
And GenerationStatus.error_code = "USER_DISPLAYABLE_ERROR"
And GenerationStatus.is_rate_limited() = true
```

### SC-AG-10: Generate infographic

```gherkin
Given a notebook with ready sources
When I call generate_artifact with ArtifactConfig::Infographic { orientation: Landscape, detail: Standard, style: Professional, instructions: null, language: "en", source_ids: ["src1"] }
Then type_code=7
And config is at position 14
And config[14][0][3] = 1 (LANDSCAPE)
And config[14][0][4] = 2 (STANDARD detail)
And config[14][0][5] = 3 (PROFESSIONAL style)
```

### SC-AG-11: Generate slide deck

```gherkin
Given a notebook with ready sources
When I call generate_artifact with ArtifactConfig::SlideDeck { format: DetailedDeck, length: Short, instructions: null, language: "en", source_ids: ["src1"] }
Then type_code=8
And config is at position 16
And config[16][0][2] = 1 (DETAILED_DECK)
And config[16][0][3] = 2 (SHORT)
```

### SC-AG-12: Generate data table

```gherkin
Given a notebook with ready sources
When I call generate_artifact with ArtifactConfig::DataTable { instructions: "comparison table", language: "en", source_ids: ["src1"] }
Then type_code=9
And config is at position 18
And config[18][1][0] = "comparison table"
And config[18][1][1] = "en"
```
