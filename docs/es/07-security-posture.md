---
title: "Postura de Seguridad — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.3.1"
last_updated: "2026-04-06"
lang: es
scan_type: full
---

# Postura de Seguridad

## Resumen Ejecutivo

| Área | Estado | Detalles |
|------|--------|----------|
| Seguridad de Memoria | **LIMPIO** | Cero bloques `unsafe` |
| Cadena de Suministro | **LIMPIO** | 0 vulnerabilidades (335 deps) |
| Almacenamiento de Credenciales | **SEGURO** | Keyring del SO + fallback DPAPI |
| TLS | **SEGURO** | rustls (sin dependencias C) |
| Autenticación | **MODERADO** | API de Google con ingeniería inversa |
| Anti-Detección | **ENDURECIDO** | Headers tipo Chrome, circuit breaker, auto-refresh |

## Autenticación y Autorización

### Manejo de Cookies

- Extraídas vía Chrome DevTools Protocol (CDP)
- Apunta a `__Secure-1PSID` y `__Secure-1PSIDTS` (cookies HttpOnly)
- Nunca visibles en JavaScript — requiere automatización del navegador
- **Corrección crítica**: `Network.getCookies` de CDP devuelve nombre+valor pero NO los atributos SameSite/Secure/Path/Domain. El servidor de Google rechaza solicitudes sin los atributos de cookie correctos. Solución: inyección directa de headers desde cookies CDP con reconstrucción de atributos.

### Token CSRF

- Token `SNlM0e` extraído desde el HTML de NotebookLM vía regex
- No es estático — requiere refresco en errores 400
- Detección de expiración integrada en el manejo de errores
- **Módulo 6**: Auto-refresh en error de autenticación — refresco silencioso de CSRF+SID antes de que el usuario se dé cuenta

### Almacenamiento de Credenciales

```
Primario:   Keyring del SO (Windows Credential Manager / macOS Keychain / Linux Secret Service)
             Servicio: "notebooklm-mcp" | Entrada: "google-credentials"
Fallback:   Archivo cifrado con DPAPI (solo Windows)
```

> Las credenciales **nunca** se almacenan en variables de entorno, archivos de configuración ni registros.

## Endurecimiento Anti-Detección (Módulo 6)

### Suplantación de Huella del Navegador

Todas las solicitudes al endpoint `batchexecute` de Google incluyen headers HTTP tipo Chrome para evitar la detección del WAF:

| Header | Valor | Propósito |
|--------|-------|-----------|
| `User-Agent` | Chrome/136 en Windows | Identificación del navegador |
| `sec-fetch-dest` | `empty` | Tipo de solicitud XHR |
| `sec-fetch-mode` | `cors` | Modo CORS |
| `sec-fetch-site` | `same-origin` | Verificación de origen |
| `sec-ch-ua` | Chromium;v=136 | Client hint |
| `sec-ch-ua-mobile` | `?0` | Modo escritorio |
| `sec-ch-ua-platform` | "Windows" | Hint de SO |
| `origin` | https://notebooklm.google.com | Header de origen |
| `referer` | https://notebooklm.google.com/ | Header referer |
| `accept` | `*/*` | Tipo de contenido |
| `accept-language` | en-US,en;q=0.9 | Preferencia de idioma |

### Circuit Breaker

Evita saturar a Google con credenciales expiradas:

```
CLOSED ──(3 errores auth)──→ OPEN ──(60s enfriamiento)──→ HALF-OPEN
  ↑                                                    │
  └──────────(sondeo exitoso)──────────────────────────┘
                                                          │
                                     (sondeo fallido)──────→ OPEN
```

- **Umbral**: 3 errores de autenticación consecutivos (401/400/403)
- **Enfriamiento**: 60 segundos antes de permitir una solicitud de sondeo
- **Acción del usuario**: "Ejecuta `notebooklm-mcp auth-browser` para reautenticarte"
- **Implementación**: `AtomicU32` para contador sin locks, `Mutex<Option<Instant>>` para marca de tiempo

### Auto-Refresh de CSRF

Cuando se detecta un error de autenticación, el servidor automáticamente:

1. Adquiere un lock de `tokio::sync::Mutex` (evita refreshes concurrentes)
2. Llama a `refresh_tokens()` para obtener un nuevo CSRF + Session ID desde Google
3. Reintenta la solicitud original exactamente una vez con los nuevos tokens
4. Si el refresh falla → incrementa el contador del circuit breaker → propaga el error

### Soporte de Retry-After

Cuando Google devuelve HTTP 429 con un header `Retry-After`:

- Parsea segundos enteros (`"5"` → 5000ms) y formatos de fecha HTTP
- Usa el retardo especificado por el servidor en lugar del backoff calculado
- Límite de 120 segundos para evitar esperas excesivas
- Recurre al backoff exponencial si el header está ausente

## Limitación de Tasa y Reintentos

| Mecanismo | Valor | Propósito |
|-----------|-------|-----------|
| Cuota de token bucket | Período de 2s (~30 req/min) | Prevenir abuso de API |
| Jitter previo a solicitud | 800-2000ms | Simular ritmo humano |
| Retroceso exponencial | 2^x segundos (2, 4, 8, 16...) | Espaciado de reintentos |
| Jitter de retroceso | 800-2000ms | Evitar efecto thundering herd |
| Reintentos máximos | 3 intentos | Resiliencia |
| Tope de retroceso | 30 segundos | Prevenir esperas infinitas |
| Retry-After | Especificado por servidor (hasta 120s) | Respetar la guía de Google |

## Seguridad de Memoria

- **Cero bloques `unsafe`** en todo el código fuente
- Todo acceso a arrays usa helpers defensivos (`get_string_at`, `get_uuid_at`)
- Escaneo de `cargo-audit`: **0 vulnerabilidades** en 335 dependencias de crates
- Sin `unwrap()` en datos externos (respuestas RPC)

## Manejo de Datos Sensibles

| Dato | Manejo | Almacenamiento |
|------|--------|----------------|
| Cookies de Google | Extraídas vía CDP, nunca registradas | Keyring del SO |
| Token CSRF | Extraído desde HTML, auto-refresh al expirar | Keyring del SO |
| ID de Sesión | Extraído desde HTML, auto-refresh al expirar | Keyring del SO |
| Contenido de notebooks | Procesado vía RPC, nunca almacenado localmente | Servidores de Google |
| Consultas del usuario | Enviadas a RPC de Google, no registradas | No persistido |

## Cadena de Suministro

- Todas las dependencias desde **crates.io** (registro oficial de paquetes Rust)
- TLS vía **rustls** (Rust puro, sin dependencias OpenSSL/C)
- Automatización de Chrome vía **headless_chrome** (protocolo CDP, sin Selenium)
- Sin scripts de build que descarguen binarios externos
- Crate **httpdate** para parseo de fechas Retry-After (cero dependencias)

## Seguridad de Descargas

Las descargas de artefactos validan:

- **Lista blanca de dominios**: Solo `googleapis.com` y `googleusercontent.com`
- **Cumplimiento de esquema**: Solo HTTPS (sin descargas HTTP)
- **Streaming**: Sin escrituras a archivos temporales para contenido en memoria

## Riesgos Conocidos

| Riesgo | Severidad | Mitigación |
|--------|-----------|------------|
| Cambios en la API de Google | Alto | Parseo defensivo, errores estructurados, capa RPC modular |
| Expiración de cookies | Medio | Autodetección, auto-refresh, reautenticación fácil vía `auth-browser` |
| Sin API oficial | Medio | Diseño modular para adaptación fácil a cambios |
| Protocolo con ingeniería inversa | Medio | Todo el parseo es defensivo — formatos inesperados devuelven errores |
| Detección por WAF | Bajo | Headers tipo Chrome, jitter humano, circuit breaker |

> **[English](../en/07-security-posture.md)** · **[Português](../pt/07-security-posture.md)**
