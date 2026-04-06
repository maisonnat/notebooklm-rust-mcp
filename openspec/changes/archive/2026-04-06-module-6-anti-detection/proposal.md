# Proposal: Module 6 — Anti-Detection Hardening

## Intent

Hardening del servidor MCP contra la detección de automatización por parte de Google al interactuar con el endpoint interno `batchexecute`. Sin estas defensas, la cuenta de Google usada para autenticación corre riesgo de baneo por patrones de tráfico que delatan un bot (headers HTTP ausentes, intervalos perfectos, martillado con tokens expirados).

## Scope

### In Scope
- Inyectar cabeceras HTTP idénticas a Chrome (User-Agent, Sec-Fetch-*, Referer, Origin, Accept-*)
- Fix del bug de exponential backoff (`1^x` → `2^x`)
- Implementar circuit breaker: detener peticiones tras N errores consecutivos de autenticación
- Auto-refresh del CSRF token ante error 400/401
- Respetar el header `Retry-After` de Google cuando llega 429
- Aumentar el rango de jitter pre-request (150-600ms → 800-2000ms)

### Out of Scope
- TLS fingerprinting (JA3/JA4) — evidencia empírica muestra que Google no lo checkea para este endpoint
- Cambio de rustls a native-tls — innecesario según análisis
- Proxy a través de Playwright — cambio arquitectural masivo, fuera de alcance

## Capabilities

### New Capabilities
- `anti-detection-headers`: Inyección de cabeceras HTTP browser-like en todas las peticiones a batchexecute
- `circuit-breaker`: Patrón de circuit breaker que detiene peticiones tras errores consecutivos de auth
- `auto-csrf-refresh`: Refresco automático del token CSRF (SNlM0e) cuando se detecta expiración
- `retry-after-support`: Respeto del header `Retry-After` en respuestas 429 de Google

### Modified Capabilities
- `source-polling`: El jitter base aumenta de 150-600ms a 800-2000ms

## Approach

1. Crear un módulo `src/browser_headers.rs` con constantes de headers Chrome-like
2. Fix del backoff: cambiar `1u64.pow(attempt)` por `2u64.pow(attempt)` en `apply_exponential_backoff()`
3. Implementar circuit breaker con `AtomicU32` de conteo de errores consecutivos (threshold: 3)
4. Cablear `auth_helper.refresh_tokens()` al path de error en `batchexecute_with_retry()`
5. Parsear `Retry-After` header en respuestas 429
6. Actualizar jitter base en `apply_jitter()`

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/notebooklm_client.rs` | Modified | Headers HTTP, backoff fix, circuit breaker, auto-refresh, Retry-After |
| `src/main.rs` | Modified | Mensajes de error al usuario (circuit breaker abierto) |
| `src/errors.rs` | Modified | Nuevo error variant `CircuitOpen` |
| `src/auth_helper.rs` | Modified | Headers para refresh, nueva firma pública para refresh |
| `src/browser_headers.rs` | New | Constantes de headers Chrome-like |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Headers hardcodeados queden obsoletos | Low | Extraer a módulo separado, easy update |
| Auto-refresh falla si Google cambia HTML | Low | 7 tests existentes cubren los extractores |
| Circuit breaker falso positivo | Medium | Threshold configurable, log claro del motivo |

## Rollback Plan

Revertir commit. Los cambios son aditivos (no destructivos) — no se elimina funcionalidad existente. Si el circuit breaker genera falsos positivos, se puede desactivar con un flag.

## Dependencies

- Ninguna dependencia externa nueva (usamos `AtomicU32` de la stdlib para el circuit breaker)

## Success Criteria

- [ ] Todas las peticiones a batchexecute incluyen 7+ cabeceras browser-like
- [ ] `apply_exponential_backoff(1)` espera ~1s, `apply_exponential_backoff(3)` espera ~8s (no todas 1s)
- [ ] Tras 3 auth errors consecutivos, el circuit breaker se abre y devuelve error claro
- [ ] Ante un error 400/CSRF, el cliente intenta refresh automático del CSRF antes de fallar
- [ ] Ante un 429 con `Retry-After: 5`, el cliente espera 5s (no usa backoff propio)
- [ ] `cargo clippy` — 0 warnings
- [ ] `cargo test` — todos pasan, tests nuevos para backoff fix + circuit breaker
