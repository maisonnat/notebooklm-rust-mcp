# Exploration: Anti-Detection Hardening

## Current State

El servidor MCP interactúa con el endpoint interno `batchexecute` de Google NotebookLM usando `reqwest` con `rustls-tls`. Actualmente tiene defensas parciales:

- **Rate limiting**: `governor::RateLimiter` con 1 request cada 2s (~30 req/min)
- **Jitter**: 150-600ms aleatorios antes de cada request
- **Exponential backoff**: Para reintentos (pero con un BUG: usa `1^x` en vez de `2^x`)
- **Error detection**: `NotebookLmError` detecta 401, 400/CSRF, 429
- **Auth refresh**: `auth_helper.rs` tiene `refresh_tokens()` pero NO está cableado al error handling de RPC

### Lo que FALTA (y Google puede detectar):

1. **Solo 2 headers HTTP**: Cookie + Content-Type. Faltan User-Agent, Sec-Fetch-*, Referer, Origin, Accept-Language, Accept-Encoding
2. **No circuit breaker**: Si CSRF expira, reintenta 3 veces con el mismo token muerto
3. **Backoff roto**: `1u64.pow(attempt)` siempre es 1 — no es exponencial
4. **No auto-refresh**: El auth_helper.refresh_tokens() existe pero nunca se llama automáticamente
5. **No Retry-After**: Ignora el header de Google que dice cuánto esperar

### Evidencia de proyectos de referencia:

- **teng-lin/notebooklm-py** (500+ stars): Usa httpx, NO hace TLS spoofing, tiene auto-refresh con lock y `_try_refresh_and_retry()`. Funciona sin baneos.
- **K-dash/nblm-rs** (competidor Rust): Usa OAuth bearer tokens (diferente arquitectura), tiene retry con backon + jitter + Retry-After.

## Affected Areas

- `src/notebooklm_client.rs` — Headers HTTP, backoff, circuit breaker, auto-refresh (LÍNEAS PRINCIPALES)
- `src/main.rs` — Mensajes de error al usuario cuando el circuit breaker se abre
- `src/errors.rs` — Mapeo de errores HTTP → tipos
- `src/auth_helper.rs` — Cablear refresh_tokens() al error handling
- `src/auth_browser.rs` — Headers para el refresh de tokens
- `Cargo.toml` — Posible dependencia en `backon` crate (opcional)

## Approaches

### 1. Headers + Bugfix + Circuit Breaker (Recomendado)

Agrupar todas las mejoras en un módulo cohesivo. Inyectar headers Chrome-like, fix del backoff, auto-refresh con lock, y circuit breaker.

- **Pros**: Cambio incremental, bajo riesgo, alto impacto. No cambia la arquitectura.
- **Cons**: No resuelve TLS fingerprinting (pero la evidencia muestra que no es necesario).
- **Esfuerzo**: Medium (~200 líneas de código nuevo/modificado)

### 2. Headers + Switch a native-tls

Cambiar de rustls a native-tls (SChannel en Windows) para obtener un TLS fingerprint más "normal".

- **Pros**: Fingerprint TLS más cercano a un navegador real
- **Cons**: Requiere cambio de dependencia, posible incompatibilidad, SChannel no es portable. Pierde la ventaja de "zero unsafe" de rustls.
- **Esfuerzo**: High (cambio de dependencia + testing exhaustivo)

### 3. Proxy through Playwright/Chrome

Usar el browser headless ya existente (headless_chrome) como proxy para todas las requests.

- **Pros**: TLS fingerprint perfecto (es Chrome real), headers perfectos
- **Cons**: Requiere Chrome corriendo, lento, complejo, frágil. Cambio arquitectural masivo.
- **Esfuerzo**: Very High (casi rewrite del cliente HTTP)

## Recommendation

**Approach 1** — Headers + Bugfix + Circuit Breaker.

Razones:
1. La evidencia empírica del proyecto Python de referencia (500+ stars) muestra que Google NO está fingerprintando agresivamente `batchexecute` para NotebookLM
2. Los headers HTTP son la señal #1 que Google checkea — son fáciles de implementar
3. El circuit breaker previene el baneo más rápido: token expirado = parar todo, no martillar
4. El bug del backoff es trivial de fixear pero crítico
5. Si algún día Google aprieta, podemos agregar TLS spoofing en un Module 7

## Risks

- **Headers hardcodeados**: Si Google cambia las cabeceras esperadas, hay que actualizar. Mitigación: extraer a constante/modulo separado.
- **Auto-refresh puede fallar**: Si Google cambia el HTML, el regex del CSRF puede romperse. Mitigación: ya tenemos 7 tests para los extractores.
- **Falsa sensación de seguridad**: Estos cambios reducen el riesgo pero NO lo eliminan. El endpoint es no documentado y puede cambiar en cualquier momento.

## Ready for Proposal

**Yes.** El alcance está claro, los archivos afectos están identificados, y el approach es bajo riesgo. El orchestrador puede proceder a crear el proposal con estos specs:

1. **Spec 1**: Browser-Like HTTP Headers (User-Agent, Sec-Fetch-*, Referer, Origin, Accept-*)
2. **Spec 2**: Exponential Backoff Fix + Retry-After Support
3. **Spec 3**: Circuit Breaker + Auto-Refresh CSRF
