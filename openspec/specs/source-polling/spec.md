# Source Polling Specification

## Purpose

Define el comportamiento del polling asíncrono que espera hasta que una fuente esté lista para consultas, detectando estados de procesamiento reales (Ready/Processing/Error) desde la respuesta de `rLM1Ne`.

## Requirements

### Requirement: Source State Detection

El sistema MUST determinar el estado real de una fuente (Ready, Processing, Error) desde la respuesta del RPC `rLM1Ne` (GET_NOTEBOOK), NO solo verificando presencia del source_id en la lista.

#### Scenario: Source is ready

- GIVEN un notebook con una fuente que terminó de procesarse
- WHEN el sistema consulta el estado via `get_notebook_sources` o respuesta raw de `rLM1Ne`
- AND la fuente tiene status `Ready` en los datos de la respuesta
- THEN el poller retorna `SourceState::Ready`

#### Scenario: Source is still processing

- GIVEN un notebook con una fuente recién añadida
- WHEN el sistema consulta el estado
- AND la fuente tiene status `Processing` o `Preparing`
- THEN el poller retorna `SourceState::Processing`

#### Scenario: Source processing failed

- GIVEN un notebook con una fuente que falló al procesarse
- WHEN el sistema consulta el estado
- AND la fuente tiene status `Error`
- THEN el poller retorna `SourceState::Error` con mensaje descriptivo

### Requirement: Exponential Backoff Polling

El poller MUST usar backoff exponencial entre verificaciones para reducir carga en la API.

#### Scenario: Backoff increases between retries

- GIVEN un intervalo inicial de 2 segundos y factor 1.5
- WHEN la fuente no está lista tras el primer check
- THEN el segundo check espera ~3 segundos
- AND el tercer check espera ~4.5 segundos
- AND el intervalo se capa en `max_interval` (default: 10 segundos)

#### Scenario: Timeout reached

- GIVEN un timeout de 120 segundos
- WHEN la fuente no está lista después de 120 segundos
- THEN el poller retorna error `SourceTimeoutError`
- AND incluye el último status conocido en el error

### Requirement: Multi-Source Parallel Polling

El sistema MUST soportar esperar múltiples fuentes en paralelo usando `tokio::join!` o similar.

#### Scenario: Wait for 3 sources simultaneously

- GIVEN 3 fuentes recién añadidas a un notebook
- WHEN se invoca `wait_for_sources(notebook_id, [id1, id2, id3])`
- THEN las 3 fuentes se polean en paralelo
- AND retorna cuando TODAS están ready
- OR retorna error si CUALQUIERA falla o timeoutea

#### Scenario: One source fails while others succeed

- GIVEN 3 fuentes polleando en paralelo
- WHEN la fuente 2 entra en estado Error
- THEN se cancelan las polls restantes
- AND se retorna error con el source_id y motivo de la fuente fallida

### Requirement: Integration with Source Operations

Los métodos de ingesta (add_url_source, add_file_source, add_drive_source) MAY aceptar un parámetro `wait` para polling automático post-ingesta.

#### Scenario: Add URL with wait=true

- GIVEN un notebook existente
- WHEN `add_url_source(notebook_id, url)` se ejecuta con `wait=true`
- THEN el sistema añade la fuente
- AND espera automáticamente hasta que esté `Ready`
- AND retorna el source_id con estado confirmado

#### Scenario: Add URL with wait=false (default)

- GIVEN un notebook existente
- WHEN `add_url_source(notebook_id, url)` se ejecuta con `wait=false`
- THEN el sistema añade la fuente
- AND retorna inmediatamente el source_id (puede estar Processing)

### Requirement: Pre-Request Jitter
(Previously: Jitter range was 150-600ms)

The system MUST add a random delay before each batchexecute request to simulate human timing patterns.

#### Scenario: Jitter applied before every request
- GIVEN any batchexecute request is prepared
- WHEN the request is about to be sent
- THEN the system MUST add a random delay between 800ms and 2000ms

#### Scenario: Jitter varies between requests
- GIVEN two sequential batchexecute requests
- WHEN both requests are sent
- THEN the delays MUST be different (non-deterministic)
