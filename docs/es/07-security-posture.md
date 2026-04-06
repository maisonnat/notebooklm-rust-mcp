---
title: "Postura de Seguridad — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: es
scan_type: full
---

# Postura de Seguridad

## Resumen Ejecutivo

| Área | Estado | Detalles |
|------|--------|----------|
| Seguridad de Memoria | **LIMPIO** | Cero bloques `unsafe` |
| Cadena de Suministro | **LIMPIO** | 0 vulnerabilidades (334 deps) |
| Almacenamiento de Credenciales | **SEGURO** | Keyring del SO + fallback DPAPI |
| TLS | **SEGURO** | rustls (sin dependencias C) |
| Autenticación | **MODERADO** | API de Google con ingeniería inversa |

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

### Almacenamiento de Credenciales

```
Primario:   Keyring del SO (Windows Credential Manager / macOS Keychain / Linux Secret Service)
             Servicio: "notebooklm-mcp" | Entrada: "google-credentials"
Fallback:   Archivo cifrado con DPAPI (solo Windows)
```

> Las credenciales **nunca** se almacenan en variables de entorno, archivos de configuración ni registros.

## Seguridad de Memoria

- **Cero bloques `unsafe`** en todo el código fuente
- Todo acceso a arrays usa helpers defensivos (`get_string_at`, `get_uuid_at`)
- Escaneo de `cargo-audit`: **0 vulnerabilidades** en 334 dependencias de crates
- Sin `unwrap()` en datos externos (respuestas RPC)

## Manejo de Datos Sensibles

| Dato | Manejo | Almacenamiento |
|------|--------|----------------|
| Cookies de Google | Extraídas vía CDP, nunca registradas | Keyring del SO |
| Token CSRF | Extraído desde HTML, validado por solicitud | Keyring del SO |
| ID de Sesión | Extraído desde HTML | Keyring del SO |
| Contenido de Notebooks | Procesado vía RPC, nunca almacenado localmente | Servidores de Google |
| Consultas del usuario | Enviadas a RPC de Google, no registradas | No persistido |

## Limitación de Tasa y Reintentos

| Mecanismo | Valor | Propósito |
|-----------|-------|-----------|
| Cuota de token bucket | Período de 2s (~30 req/min) | Prevenir abuso de API |
| Retroceso exponencial | Jitter 150-600ms | Evitar efecto thundering herd |
| Reintentos máximos | 3 intentos | Resiliencia |
| Tope de retroceso | 30 segundos | Prevenir esperas infinitas |

## Cadena de Suministro

- Todas las dependencias desde **crates.io** (registro oficial de paquetes Rust)
- TLS vía **rustls** (Rust puro, sin dependencias OpenSSL/C)
- Automatización de Chrome vía **headless_chrome** (protocolo CDP, sin Selenium)
- Sin scripts de build que descarguen binarios externos

## Seguridad de Descargas

Las descargas de artefactos validan:

- **Lista blanca de dominios**: Solo `googleapis.com` y `googleusercontent.com`
- **Cumplimiento de esquema**: Solo HTTPS (sin descargas HTTP)
- **Streaming**: Sin escrituras a archivos temporales para contenido en memoria

## Riesgos Conocidos

| Riesgo | Severidad | Mitigación |
|--------|-----------|------------|
| Cambios en la API de Google | Alto | Parseo defensivo, errores estructurados, capa RPC modular |
| Expiración de cookies | Medio | Autodetección, reautenticación fácil vía `auth-browser` |
| Sin API oficial | Medio | Diseño modular para adaptación fácil a cambios |
| Protocolo con ingeniería inversa | Medio | Todo el parseo es defensivo — formatos inesperados devuelven errores |

> **[English](../en/07-security-posture.md)** · **[Português](../pt/07-security-posture.md)**
