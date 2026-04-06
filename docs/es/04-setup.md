---
title: "Guía de Configuración — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: es
scan_type: full
---

# Guía de Configuración

## Requisitos Previos

| Requisito | Versión | Notas |
|-----------|---------|-------|
| **Rust** | 1.70+ (edición 2024) | [rustup.rs](https://rustup.rs/) |
| **Chrome** | Cualquier versión reciente | Requerido para el comando `auth-browser` |
| **SO** | Windows, macOS, Linux | Keyring del SO para almacenamiento de credenciales |

## Compilación

```bash
# Clonar
git clone <url-del-repo>
cd notebooklm-rust-mcp

# Compilar binario de release
cargo build --release

# Ubicación del binario
# ./target/release/notebooklm-mcp
```

## Autenticación

### Opción 1: Autenticación por Navegador (Recomendada)

```bash
./target/release/notebooklm-mcp auth-browser
```

Esto abre Chrome, navega a Google NotebookLM y espera que inicies sesión. Una vez autenticado, las credenciales se guardan en tu **keyring del SO** — no se necesitan variables de entorno.

### Opción 2: Variables de Entorno

Si prefieres gestionar credenciales manualmente:

```bash
export NOTEBOOKLM_COOKIE="__Secure-1PSID=...;__Secure-1PSIDTS=..."
export NOTEBOOKLM_CSRF="tu_token_csrf"
export NOTEBOOKLM_SID="tu_id_de_sesion"
```

### Verificar Autenticación

```bash
./target/release/notebooklm-mcp auth-status
# o
./target/release/notebooklm-mcp verify
```

## Ejecutar como Servidor MCP

El servidor se comunica por **stdio** (entrada/salida estándar). Configura tu cliente MCP:

### Claude Desktop

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/ruta/absoluta/a/notebooklm-mcp",
      "args": []
    }
  }
}
```

### Cursor / Windsurf

La misma configuración en tu archivo de ajustes MCP.

### CLI Directo

Todas las operaciones están disponibles como comandos CLI:

```bash
./target/release/notebooklm-mcp list
./target/release/notebooklm-mcp create --title "Mi Notebook"
./target/release/notebooklm-mcp artifact-generate --notebook-id <id> --kind report
```

## Pruebas

```bash
# Ejecutar todas las pruebas unitarias
cargo test

# 329 pruebas, 5 ignoradas (las pruebas E2E requieren credenciales activas)
```

## Almacenamiento de Credenciales

Las credenciales se almacenan en el keyring de tu sistema operativo:

| SO | Backend | Detalles |
|----|---------|----------|
| **Windows** | DPAPI | Fallback via crate `windows-dpapi` |
| **macOS** | Keychain | Acceso nativo al keychain |
| **Linux** | Secret Service | D-Bus `org.freedesktop.secrets` |

> **Nota de seguridad:** Las credenciales **nunca** se escriben en variables de entorno, archivos de configuración ni registros. El keyring del SO es el único mecanismo de almacenamiento.

## Resolución de Problemas

| Problema | Solución |
|----------|----------|
| `auth-browser` falla | Asegúrate de que Chrome esté instalado y accesible |
| CSRF expirado | Ejecuta `auth-browser` nuevamente para refrescar credenciales |
| Limitado por tasa (429) | Espera unos minutos — el servidor tiene limitación de tasa integrada (~30 req/min) |
| "No se encontraron credenciales" | Ejecuta `auth-browser` primero, o configura las variables de entorno manualmente |

> **[English](../en/04-setup.md)** · **[Português](../pt/04-setup.md)**
