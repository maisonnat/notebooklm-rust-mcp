---
title: "Configuracion — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-04"
lang: es
scan_type: full
---

# Configuracion

## Requisitos Previos

| Requisito | Version | Notas |
|-----------|---------|-------|
| Rust | 1.70+ | Edicion 2024 |
| Chrome | Ultima version | Para `auth-browser` |
| Cuenta de Google | — | Acceso a NotebookLM |

## Instalacion

```bash
git clone https://github.com/maisonnat/notebooklm-rust-mcp
cd notebooklm-rust-mcp
cargo build --release
```

Binario resultante: `./target/release/notebooklm-mcp`

## Autenticacion

### Auth via Browser (Recomendado)

```bash
./target/release/notebooklm-mcp auth-browser
```

1. Chrome se lanza en modo headless
2. Completá el login de Google
3. Las cookies se extraen via CDP
4. Se almacenan en el keyring del SO

### Auth Manual

```bash
./target/release/notebooklm-mcp auth --cookie "..." --csrf "..."
```

Encriptado con DPAPI en `~/.notebooklm-mcp/session.bin`.

### Verificar Estado

```bash
./target/release/notebooklm-mcp auth-status
```

## Verificar Conexion

```bash
./target/release/notebooklm-mcp verify
```

## Configuracion del Cliente MCP

Configurá tu cliente para que lance el binario con transporte stdio:

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/ruta/al/notebooklm-mcp"
    }
  }
}
```

## Tests

```bash
cargo test
```

## Resolucion de Problemas

| Problema | Solucion |
|----------|----------|
| "Servidor no autenticado" | Ejecutá `auth-browser` |
| Chrome no encontrado | Instalá Chrome o usá `auth` manual |
| Sesion expirada | Volvé a ejecutar `auth-browser` |
| Rate limited | Se maneja automaticamente (limite de 30 req/min) |
