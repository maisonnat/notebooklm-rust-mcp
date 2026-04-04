# Instalación

## Requisitos

- **Rust** 1.70+ (edición 2024)
- **Windows** (para DPAPI) — en Linux/macOS hay fallback
- **Google Chrome** (para autenticación automática)

## Compilación

```bash
# Clonar el repositorio
git clone <repo-url>
cd notebooklm-mcp

# Compilar
cargo build --release

# El binario queda en target/release/notebooklm-mcp.exe
```

## Autenticación

### Método 1: Chrome headless (recomendado)

Este método abre una ventana de Chrome para que inicies sesión:

```bash
./target/release/notebooklm-mcp auth-browser
```

**Flujo:**
1. Se abre ventana de Chrome
2. Iniciás sesión en tu cuenta de Google
3. El script detecta las cookies automáticamente
4. Se guardan en Windows Credential Manager

### Método 2: Manual

Si preferís no usar Chrome:

1. Abrí DevTools (F12) en notebooklm.google.com
2. Application → Cookies → Copiá el valor de `__Secure-1PSID` y `__Secure-1PSIDTS`
3. Hacé un GET a notebooklm.google.com y buscá `"SNlM0e":"..."` en el HTML
4. Ejecutá:

```bash
notebooklm-mcp auth \
  --cookie "__Secure-1PSID=xxx; __Secure-1PSIDTS=yyy" \
  --csrf "SNlM0e_xxx"
```

## Configuración del Cliente MCP

### Cursor

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "path/to/notebooklm-mcp.exe"
    }
  }
}
```

### Claude Desktop

En `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "path/to/notebooklm-mcp.exe"
    }
  }
}
```

### Windsurf

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "path/to/notebooklm-mcp.exe"
    }
  }
}
```

## Verificación

```bash
# Ver estado de autenticación
notebooklm-mcp auth-status

# Probar conexión
notebooklm-mcp verify
```

Si ves "Libretas encontradas: [...]" ✓

## Variables de Entorno

No hay variables de entorno necesarias — las credenciales se guardan en:

- **Windows**: Windows Credential Manager (via keyring) o DPAPI
- **Fallback**: `~/.notebooklm-mcp/session.bin`

## Actualización de Credenciales

Las cookies de Google expiran frecuentemente. Si ves errores de autenticación:

```bash
# Regenerar credenciales
notebooklm-mcp auth-browser

# O manual
notebooklm-mcp auth --cookie "nueva_cookie" --csrf "nuevo_csrf"
```
