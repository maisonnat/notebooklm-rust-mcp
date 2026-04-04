# Tasks: Conectar Browser Auth al CLI y Completar Tests

## Phase 1: CLI Command Integration

- [x] 1.1 Agregar `AuthBrowser` variant al enum `Commands` en main.rs
- [x] 1.2 Agregar `AuthStatus` variant al enum `Commands` en main.rs
- [x] 1.3 Importar `crate::auth_browser` en main.rs
- [x] 1.4 Implementar handler para `AuthBrowser` que llame a `BrowserAuthenticator::authenticate()`
- [x] 1.5 Implementar handler para `AuthStatus` que llame a `auth_browser::get_auth_status()`

## Phase 2: Auth Flow Implementation

- [x] 2.1 Modificar `authenticate()` para que sea sync ( Tokio block_on desde CLI)
- [x] 2.2 Si auth exitosa, extraer CSRF via auth_helper y guardar en keyring
- [x] 2.3 Si fallback requerido, usar método DPAPI existente
- [x] 2.4 Agregar output de mensaje claro al usuario (success/error)

## Phase 3: Tests de Integración

- [x] 3.1 Crear `tests/integration.rs` con test que compile todos los módulos
- [x] 3.2 Agregar test que verifique parser con fixture de tests/fixtures/
- [x] 3.3 Agregar test que verifique error classification
- [x] 3.4 Ejecutar `cargo test` y verificar que todos pasen

## Phase 4: Verificación Final

- [x] 4.1 Ejecutar `cargo check` para verificar compilación
- [x] 4.2 Ejecutar `cargo clippy` para verificar linting
- [x] 4.3 Verificar que `auth-status` muestre información correcta
- [x] 4.4 Documentar el nuevo comando en comments del código