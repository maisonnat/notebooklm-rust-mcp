# Proposal: Conectar Browser Auth al CLI y Completar Tests

## Intent

Completar la integración del módulo `auth_browser.rs` con el CLI existente, y asegurar que los tests de integración tengan cobertura básica.

## Scope

### In Scope
1. Agregar comando `auth-browser` al CLI en main.rs
2. Completar tests de integración básicos

### Out of Scope
- Tests E2E con credenciales reales
- Integración con MCP server tools

## Capabilities

### New Capabilities
- `browser-auth-cli`: Command to authenticate via headless Chrome

### Modified Capabilities
- `integration-tests`: Tests existentes tienen coverage parcial

## Approach

1. Agregar `AuthBrowser` command a enum `Commands` en main.rs
2. Llamar a `BrowserAuthenticator::authenticate()` desde el CLI
3. Si succeeds, guardar en keyring; si falla, fallback a DPAPI
4. Agregar tests de integración simples en `tests/`

## Affected Areas

| Area | Impact | Description |
|------|--------|-------------|
| `src/main.rs` | Modified | Agregar AuthBrowser command |
| `src/auth_browser.rs` | Modified | Ajustar para sync/async |
| `tests/` | Modified | Agregar integration tests |

## Risks

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Chrome no disponible | Medium | Fallback a DPAPI ya implementado |
| Keyring fails | Low | Graceful degradation |

## Rollback Plan

Revertir cambios en main.rs y eliminar command agregado.

## Dependencies

- headless_chrome, keyring ya en Cargo.toml

## Success Criteria

- [ ] Comando `auth-browser` funcional
- [ ] Tests de integración compilan
- [ ] Fallback a DPAPI funciona si Chrome no está