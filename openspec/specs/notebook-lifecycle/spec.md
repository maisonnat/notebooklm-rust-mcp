# Notebook Lifecycle Specification

## Purpose

Operaciones CRUD sobre notebooks: eliminar, renombrar, obtener detalles completos y resumen AI. Completa el ciclo de vida básico del notebook más allá de create/list.

## Requirements

### Requirement: Delete Notebook

El sistema MUST eliminar un notebook por ID usando el RPC `WWINqb` con payload `[[notebook_id], [2]]`. La eliminación MUST ser idempotente: si el notebook no existe o ya fue eliminado, la operación retorna success sin error.

#### Scenario: Delete existing notebook

- GIVEN un notebook existente con ID válido
- WHEN el cliente invoca `delete_notebook(notebook_id)`
- THEN el sistema envía el RPC `WWINqb` con payload `[[notebook_id], [2]]`
- AND retorna `Ok(())`

#### Scenario: Delete non-existent notebook (idempotency)

- GIVEN un notebook_id que NO existe o ya fue eliminado
- WHEN el cliente invoca `delete_notebook(notebook_id)`
- THEN el sistema envía el RPC `WWINqb`
- AND retorna `Ok(())` (NO error — idempotente)
- AND NO lanza excepción ni panic

#### Scenario: RPC failure during delete

- GIVEN un notebook existente
- AND el RPC `WWINqb` retorna error HTTP o respuesta inválida
- WHEN el cliente invoca `delete_notebook(notebook_id)`
- THEN retorna `Err(String)` con descripción del error
- AND NO panic ni unwrap

### Requirement: Rename Notebook

El sistema MUST renombrar un notebook usando el RPC `s0tc2d` con payload `[notebook_id, [[null, null, null, [null, new_title]]]]`. Después del rename, el sistema MUST obtener los datos actualizados del notebook y retornar el struct `Notebook` completo.

#### Scenario: Rename existing notebook

- GIVEN un notebook existente con ID válido
- AND un new_title no vacío
- WHEN el cliente invoca `rename_notebook(notebook_id, new_title)`
- THEN el sistema envía el RPC `s0tc2d` con payload correcto
- AND luego invoca `get_notebook(notebook_id)` internamente
- AND retorna `Ok(Notebook)` con el title actualizado

#### Scenario: Rename with empty title

- GIVEN un notebook existente
- AND un new_title vacío (`""`)
- WHEN el cliente invoca `rename_notebook(notebook_id, new_title)`
- THEN el sistema envía el RPC con el título vacío (no filtra)
- AND retorna el resultado del RPC (comportamiento de Google define si es válido)

#### Scenario: Rename non-existent notebook

- GIVEN un notebook_id que NO existe
- WHEN el cliente invoca `rename_notebook(notebook_id, "New Title")`
- THEN el sistema envía el RPC
- AND retorna `Err(String)` si Google rechaza la operación

### Requirement: Get Notebook Details

El sistema MUST obtener detalles completos de un notebook usando el RPC `rLM1Ne` con payload `[notebook_id, null, [2], null, 0]`. El struct `Notebook` retornado MUST incluir: `id`, `title`, `sources_count`, `is_owner`, `created_at`.

#### Scenario: Get existing notebook with full metadata

- GIVEN un notebook existente con ID válido
- WHEN el cliente invoca `get_notebook(notebook_id)`
- THEN el sistema envía el RPC `rLM1Ne` con payload `[notebook_id, null, [2], null, 0]`
- AND retorna `Ok(Notebook)` con todos los campos poblados
- AND `sources_count` refleja la cantidad de fuentes del notebook
- AND `is_owner` indica si el usuario es dueño (true) o compartido (false)
- AND `created_at` contiene la fecha de creación si disponible

#### Scenario: Get non-existent notebook

- GIVEN un notebook_id que NO existe
- WHEN el cliente invoca `get_notebook(notebook_id)`
- THEN el sistema envía el RPC `rLM1Ne`
- AND retorna `Err(String)` indicando que no se encontró

#### Scenario: Notebook struct backward compatibility

- GIVEN que `list_notebooks()` retorna notebooks con solo `id` y `title`
- WHEN se obtiene un notebook vía `get_notebook()`
- THEN el struct `Notebook` tiene campos adicionales (`sources_count`, `is_owner`, `created_at`)
- AND estos campos son opcionales con defaults seguros (sources_count=0, is_owner=true, created_at=None)

### Requirement: Get Notebook Summary

El sistema MUST obtener el resumen AI y los suggested topics de un notebook usando el RPC `VfAZjd` con payload `[notebook_id, [2]]`.

#### Scenario: Get summary with topics

- GIVEN un notebook existente con fuentes procesadas
- WHEN el cliente invoca `get_summary(notebook_id)`
- THEN el sistema envía el RPC `VfAZjd`
- AND retorna `Ok(NotebookSummary)` con `summary` (texto del resumen AI)
- AND retorna `suggested_topics` como lista de pares `(question, prompt)`

#### Scenario: Get summary of empty notebook

- GIVEN un notebook existente sin fuentes
- WHEN el cliente invoca `get_summary(notebook_id)`
- THEN el sistema envía el RPC `VfAZjd`
- AND retorna `Ok(NotebookSummary)` con `summary` vacío y `suggested_topics` vacío
- AND NO retorna error

#### Scenario: Summary response with partial data

- GIVEN un notebook donde el RPC retorna summary pero NO topics
- WHEN el cliente invoca `get_summary(notebook_id)`
- THEN retorna `Ok(NotebookSummary)` con `summary` poblado y `suggested_topics` vacío
- AND NO retorna error por falta de topics

### Requirement: MCP Tool Registration

El servidor MCP MUST exponer 4 nuevas tools: `notebook_delete`, `notebook_rename`, `notebook_get`, `notebook_summary`.

#### Scenario: MCP tool notebook_delete

- GIVEN un MCP client conectado y autenticado
- WHEN invoca `notebook_delete` con `{ notebook_id }`
- THEN el servidor ejecuta `delete_notebook` y retorna confirmación

#### Scenario: MCP tool notebook_rename

- GIVEN un MCP client conectado y autenticado
- WHEN invoca `notebook_rename` con `{ notebook_id, new_title }`
- THEN el servidor ejecuta `rename_notebook` y retorna Notebook actualizado

#### Scenario: MCP tool notebook_get

- GIVEN un MCP client conectado y autenticado
- WHEN invoca `notebook_get` con `{ notebook_id }`
- THEN el servidor ejecuta `get_notebook` y retorna detalles completos

#### Scenario: MCP tool notebook_summary

- GIVEN un MCP client conectado y autenticado
- WHEN invoca `notebook_summary` con `{ notebook_id }`
- THEN el servidor ejecuta `get_summary` y retorna summary + topics

### Requirement: CLI Commands

El CLI MUST proveer subcomandos `delete`, `rename`, `get`, `summary` para las 4 operaciones de ciclo de vida.

#### Scenario: Delete via CLI

- GIVEN credenciales válidas almacenadas
- WHEN usuario ejecuta `notebooklm-mcp delete --notebook-id ID`
- THEN el sistema elimina el notebook e imprime confirmación

#### Scenario: Rename via CLI

- GIVEN credenciales válidas almacenadas
- WHEN usuario ejecuta `notebooklm-mcp rename --notebook-id ID --title "Nuevo Título"`
- THEN el sistema renombra e imprime datos actualizados

#### Scenario: Get via CLI

- GIVEN credenciales válidas almacenadas
- WHEN usuario ejecuta `notebooklm-mcp get --notebook-id ID`
- THEN el sistema imprime detalles completos del notebook

#### Scenario: Summary via CLI

- GIVEN credenciales válidas almacenadas
- WHEN usuario ejecuta `notebooklm-mcp summary --notebook-id ID`
- THEN el sistema imprime resumen AI y suggested topics
