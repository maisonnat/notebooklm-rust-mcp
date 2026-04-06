---
title: "Arquitectura — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: es
scan_type: full
---

# Arquitectura

## Vista General del Sistema

```
┌─────────────┐     ┌──────────────────┐     ┌────────────────────┐
│  MCP Client  │────▶│  notebooklm-mcp  │────▶│  Google NotebookLM  │
│ (Claude/etc) │◀────│  (stdio server)  │◀────│  batchexecute RPC  │
└─────────────┘     └──────────────────┘     └────────────────────┘
       │                    │
       │              ┌─────┴─────┐
       │              │  Modules   │
       │              └───────────┘
  CLI (clap)     NotebookLmClient
                  auth_helper.rs
                  auth_browser.rs
                  parser.rs
                  rpc/*.rs
                  pollers
```

## Estructura de Módulos

```
src/
├── main.rs                # Punto de entrada CLI + servidor MCP + registro de herramientas
├── notebooklm_client.rs   # Cliente HTTP para la API RPC de NotebookLM (20+ métodos)
├── parser.rs              # Parser JSON defensivo para respuestas RPC de Google
├── errors.rs              # Enum de errores estructurado con autodetección
├── auth_browser.rs        # Automatización Chrome CDP + almacenamiento en keyring
├── auth_helper.rs         # Extracción de token CSRF + token de sesión desde HTML
├── artifact_poller.rs     # Polling async para generación de artefactos
├── source_poller.rs       # Polling async para indexación de fuentes
└── rpc/
    ├── mod.rs             # Declaraciones de módulos
    ├── artifacts.rs       # Tipos de artefacto + constructores de payload (9 tipos)
    ├── sources.rs         # Constructores de payload para fuentes (5 tipos)
    └── notebooks.rs       # Tipos de ciclo de vida de notebooks + parsers
```

## Responsabilidades de los Módulos

### `main.rs` — Punto de Entrada y Registro de Herramientas

El único punto de entrada del binario. Responsabilidades:

- **Análisis de argumentos CLI** vía `clap` — 21 comandos mapeados al enum `Commands`
- **Inicio del servidor MCP** vía `rmcp` — 20 métodos `#[tool]` registrados como herramientas MCP
- **Enrutamiento de solicitudes** — Los comandos CLI y herramientas MCP delegan a `NotebookLmClient`
- **Respuestas en streaming** — `ask_question` devuelve fragmentos formateados en SSE

### `notebooklm_client.rs` — Cliente HTTP

El cliente central que envuelve todas las interacciones RPC con Google:

- `batchexecute()` — Todas las llamadas a la API pasan por este único método HTTP POST
- **20+ métodos** que cubren notebooks, fuentes, artefactos y compartidos
- Limitación de tasa vía token bucket de `governor` (período de 2s, ~30 req/min)
- Inyección de cookies desde el keyring del SO o variables de entorno

### `parser.rs` — Parser JSON Defensivo

Maneja el formato anti-XSSI de las respuestas de Google:

- `strip_antixssi()` — Elimina el prefijo `)]}'` de las respuestas
- `extract_by_rpc_id()` — Enruta fragmentos de respuesta al handler correcto por ID de RPC
- **Cero `unwrap()`** en datos externos — todas las funciones de parseo devuelven `Option`/`Result`

### `src/rpc/` — Constructores de Payload

Separados por dominio:

| Módulo | Responsabilidad |
|--------|----------------|
| `rpc/artifacts.rs` | Enums de tipo de artefacto (`ArtifactType` con 9 variantes), códigos de estado, constructores de payload para generación |
| `rpc/sources.rs` | Constructores de payload para 5 tipos de fuente (texto, URL, YouTube, Drive, subida de archivo) |
| `rpc/notebooks.rs` | Tipos de ciclo de vida de notebooks, parsers para estado de compartido, resumen, detalles del notebook |

### `auth_helper.rs` — Extracción de Tokens

- Parsea el token CSRF y el ID de sesión desde las páginas HTML de NotebookLM
- Gestión y validación de cookies
- Detección de expiración de CSRF

### `auth_browser.rs` — Automatización del Navegador

- Chrome headless vía CDP (crate `headless_chrome`)
- Automatiza el flujo de inicio de sesión de Google
- Extrae y almacena credenciales en el keyring del SO
- **Corrección crítica**: Usa `Network.getCookies` de CDP + inyección directa de headers (Google rechaza el reenvío de cookies HTTP simple)

### `errors.rs` — Manejo de Errores Estructurado

- Enum `NotebookLmError` con autodetección desde respuestas HTTP
- Cubre: NotFound, NotReady, GenerationFailed, DownloadFailed, AuthExpired, RateLimited

### `artifact_poller.rs` — Polling Async de Artefactos

Realiza polling del estado de generación de artefactos hasta completarse o fallar, con retroceso exponencial.

### `source_poller.rs` — Polling Async de Fuentes

Realiza polling del estado de indexación de fuentes después de la ingesta hasta que la fuente esté procesada.

## Patrones de Diseño

### Patrón Batch Execute

Toda interacción con la API de Google sigue este pipeline:

```
Herramienta MCP / Comando CLI
  → NotebookLmClient.{método}()
    → batchexecute() HTTP POST a notebooklm.google.com
      → Respuesta con prefijo anti-XSSI
        → strip_antixssi()
          → extract_by_rpc_id()
            → Parseo defensivo → Resultado estructurado
              → Respuesta formateada como texto
```

### Herramientas MCP de Solicitud-Respuesta

Cada método `#[tool]` realiza **una llamada RPC** y devuelve una cadena formateada. No se mantiene estado entre llamadas — el servidor es sin estado.

### Lectura Post-Mutación

Las operaciones de escritura (renombrar, share_set) **leen los datos confirmados** después de la mutación para devolver un estado autoritativo al llamador.

### Parseo Defensivo

Cero `unwrap()` en datos externos. Todas las funciones de parseo devuelven `Option<T>` o `Result<T, E>`. Las respuestas RPC de Google son impredecibles — el parser nunca asume una estructura.

### Limitación de Tasa

Token bucket vía `governor`: 2 solicitudes por segundo, ~30 solicitudes por minuto. Retroceso exponencial en respuestas 429.

### Almacenamiento de Credenciales

Las credenciales se almacenan en el **keyring del SO** (vía crate `keyring`) con fallback a DPAPI en Windows. Nunca en variables de entorno, archivos de configuración ni registros.

## Flujo de Datos

```
Usuario / Agente de IA
    │
    ├── CLI: clap analiza args → enum Commands → match → método de NotebookLmClient
    │
    └── MCP: rmcp despacha llamada a herramienta → método #[tool] → método de NotebookLmClient
                                                                  │
                                                    batchexecute() POST
                                                                  │
                                                    ┌───────────┴───────────┐
                                                    │  Respuesta RPC Google  │
                                                    │  )]}'\n + array JSON   │
                                                    └───────────┬───────────┘
                                                                  │
                                                    strip_antixssi()
                                                                  │
                                                    extract_by_rpc_id()
                                                                  │
                                                    Parseo defensivo
                                                                  │
                                                    Cadena formateada → Cliente
```

## Evolución Temporal

| Período | Fecha | Resumen |
|---------|-------|---------|
| **Fundación** | 2026-03-28 | Servidor MCP inicial con 4 herramientas, autenticación por navegador, limitación de tasa, parser defensivo |
| **Documentación v1** | 2026-04-01 → 04-02 | Documentación en inglés auto-generada, README.md, CodeTour |
| **Multilenguaje** | 2026-04-03 → 04-04 | Traducciones a ES y PT, documentación versionada en git |
| **Módulo 2: Multifuentes** | 2026-04-04 → 04-05 | Fuentes URL, YouTube, Drive, subida de archivos; polling async de fuentes |
| **Módulo 3: Artefactos + Ciclo de Vida** | 2026-04-05 → 04-06 | 9 tipos de artefactos, CRUD de notebooks, compartidos, ciclo SDD completo |

> **[English](../en/01-architecture.md)** · **[Português](../pt/01-architecture.md)**
