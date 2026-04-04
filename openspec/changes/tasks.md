# Tasks: Mejoras al Parser y Sistema de Polling

## Phase 1: Foundation - Parser Defensivo

- [x] 1.1 Crear `src/parser.rs` con funciones para parseo de respuestas RPC usando acceso defensivo
- [x] 1.2 Implementar `extract_by_rpc_id` con búsqueda directa en array por índice validado
- [x] 1.3 Reemplazar `.get(index).unwrap()` por funciones del parser (get_string_at, get_uuid_at)
- [x] 1.4 Extraer función `strip_antixssi_prefix()` - buscar primer `[` y cortar todo anterior

## Phase 2: Core - Polling y Errores

- [x] 2.1 Crear `src/source_poller.rs` con bucle polling para verificar estado de fuente hasta SUCCESS
- [x] 2.2 Implementar `wait_for_source_ready(notebook_id, source_id)` con timeout y reintentos
- [x] 2.3 Crear enum `NotebookLmError` con variantes: SessionExpired, CsrfExpired, SourceNotReady, RateLimited, ParseError
- [x] 2.4 Implementar refresh automático de CSRF - crear `src/auth_helper.rs` con extracción desde HTML + validación de sesión

## Phase 3: Integración - Estado y Cache

- [x] 3.1 Crear `src/conversation_cache.rs` con RwLock para cachear conversation IDs por notebook
- [x] 3.2 Modificar `ask_question` para usar conversation cache y mantener historial
- [x] 3.3 Agregar `tokio::sync::Semaphore` para limitar uploads a 1-2 simultáneos (ya tenemos governor)
- [x] 3.4 Implementar exponential backoff con jitter para reintentos en `batchexecute`

## Phase 4: Testing

- [x] 4.1 Crear carpeta `tests/fixtures/` con respuestas HTTP grabadas (VCR pattern)
- [x] 4.2 Escribir tests unitarios para parser con respuestas de ejemplo
- [x] 4.3 Escribir test de integración para polling de fuentes
- [x] 4.4 Testear manejo de errores: sesión expirada, CSRF expirado

## Phase 5: Cleanup

- [x] 5.1 Documentar lecciones del colega en comments del código
- [x] 5.2 Limpiar código temporal de debugging
- [x] 5.3 Actualizar Cargo.toml con dependencias de test (wiremock si es necesario)

## Phase 6: Browser-Based Authentication (MEJOR QUE MANUAL)

- [x] 6.1 Añadir dependencias: headless_chrome, keyring, tempfile a Cargo.toml
- [x] 6.2 Crear `src/auth_browser.rs` con flujo de login automático usando Chrome CDP
- [x] 6.3 Implementar extracción de cookies (__Secure-1PSID, __Secure-1PSIDTS) vía CDP
- [x] 6.4 Integrar con keyring para OS credential storage (Windows Credential Manager / Linux Secret Service)
- [x] 6.5 Mantener fallback al método actual (DPAPI) para sistemas sin Chrome/keyring
- [x] 6.6 Documentar: El CSRF se sigue extrayendo desde Rust (GET + regex) NO desde el navegador

### Por qué este enfoque es MEJOR que el manual:
- Usuario interactúa directamente con Google (no copia cookies)
- Mayor seguridad: nunca tocamos credenciales en texto plano
- Extrae cookies HttpOnly que no podemos ver manualmente
- Auto-renovable: re-autenticación es fácil cuando expiran
