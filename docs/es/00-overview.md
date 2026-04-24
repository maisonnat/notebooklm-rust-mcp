---
title: "Vista General — NotebookLM MCP Server"
repo: "notebooklm-rust-mcp"
version: "0.1.0"
last_updated: "2026-04-06"
lang: es
scan_type: full
---

# Vista General

> **Servidor MCP no oficial para Google NotebookLM** — escrito en Rust sin código unsafe.

## ¿Qué Es?

NotebookLM MCP Server es un servidor de [Model Context Protocol](https://modelcontextprotocol.io) que permite a los agentes de IA (Claude, Cursor, Windsurf, etc.) interactuar con notebooks de Google NotebookLM de forma programática.

Google NotebookLM **no tiene API pública**. Este servidor realiza ingeniería inversa del protocolo RPC interno (el mismo endpoint `batchexecute` que utiliza la interfaz web de NotebookLM) para conectar agentes de IA con las operaciones de notebooks.

## Capacidades Principales

| Dominio | Herramientas | Descripción |
|---------|-------------|-------------|
| **Gestión de Notebooks** | 8 herramientas | Crear, listar, renombrar, eliminar, obtener detalles, resumen IA, estado de compartido, activar/desactivar compartido |
| **Gestión de Fuentes** | 5 herramientas | Agregar fuentes de texto, URL, YouTube, Google Drive o archivos locales |
| **Generación de Artefactos** | 4 herramientas | Generar 9 tipos de artefactos, listar, eliminar y descargar |
| **Interacción con IA** | 1 herramienta | Hacer preguntas con respuestas en streaming |
| **Autenticación** | 2 comandos CLI | Autenticación vía navegador usando Chrome CDP, almacenamiento de credenciales en keyring del SO |

**Total: 20 herramientas MCP + 21 comandos CLI**

## Tipos de Artefactos

| Tipo | Formato de Salida | Descripción |
|------|-------------------|-------------|
| `report` | PDF | Guía de estudio o informe a partir del contenido del notebook |
| `quiz` | PDF | Cuestionario de opción múltiple (3-20 preguntas, dificultad ajustable) |
| `flashcards` | PDF | Mazo de flashcards (3-20 tarjetas) |
| `audio` | Archivo de audio | Resumen de audio estilo podcast (idioma y duración configurables) |
| `infographic` | PNG | Infografía visual (apaisado/vertical, múltiples estilos) |
| `slide_deck` | PDF / PPTX | Diapositivas de presentación (corto/mediano/largo) |
| `mind_map` | JSON | Mapa mental estructurado de conceptos |
| `video` | Archivo de video | Resumen en video (estilo cinematográfico/documental) |
| `data_table` | PDF | Extracción de datos tabulares |

## Inicio Rápido

```bash
# Compilar
cargo build --release

# Autenticarse (abre Chrome, guarda credenciales en el keyring del SO)
./target/release/notebooklm-mcp auth-browser

# Usar como servidor MCP (stdio)
./target/release/notebooklm-mcp
```

### Configuración del Cliente MCP

```json
{
  "mcpServers": {
    "notebooklm": {
      "command": "/ruta/a/notebooklm-mcp",
      "args": []
    }
  }
}
```

## Stack Tecnológico

| Componente | Tecnología |
|-----------|-----------|
| Lenguaje | Rust (edición 2024) |
| Framework MCP | [rmcp](https://github.com/amodelotrust/rmcp) |
| Cliente HTTP | [reqwest](https://crates.io/crates/reqwest) (rustls TLS) |
| CLI | [clap](https://crates.io/crates/clap) |
| Runtime Async | [tokio](https://tokio.rs/) |
| Limitación de Tasa | [governor](https://crates.io/crates/governor) (token bucket, ~30 req/min) |
| Almacenamiento de Credenciales | [keyring](https://crates.io/crates/keyring) (keyring del SO + fallback DPAPI) |
| Autenticación por Navegador | [headless_chrome](https://crates.io/crates/headless-chrome) (CDP) |

## Seguridad

| Métrica | Valor |
|---------|-------|
| Bloques unsafe | **0** |
| Vulnerabilidades (cargo-audit) | **0** (334 dependencias) |
| Backend TLS | rustls (sin OpenSSL) |
| Almacenamiento de credenciales | Keyring del SO (nunca en variables de entorno ni archivos) |
| Licencia | MIT |

## Estadísticas del Proyecto

- **11 archivos fuente** en `src/`
- **329 pruebas unitarias** (5 pruebas E2E ignoradas)
- **0 advertencias de clippy**
- **Desarrollo Especificado** con 8 dominios de especificación
- **4 módulos de desarrollo** completados y archivados

## Documentación

| Documento | Descripción | Audiencia |
|-----------|-------------|-----------|
| [Arquitectura](./01-architecture.md) | Módulos, patrones de diseño, flujo de datos | Ingenieros |
| [Referencia de API](./02-api-reference.md) | Herramientas MCP, comandos CLI, configuración | Integradores |
| [Modelos de Datos](./03-data-models.md) | Entidades de dominio y definiciones de tipos | Ingenieros |
| [Guía de Configuración](./04-setup.md) | Compilar, instalar, configurar | Usuarios |
| [Guía de Usuario](./05-user-guide.md) | Flujos de trabajo comunes y consejos | Usuarios |
| [Registro de Cambios](./06-changelog.md) | Historial de versiones | Todos |
| [Postura de Seguridad](./07-security-posture.md) | Autenticación, seguridad de memoria, auditoría | Ingenieros |

> **[English](../en/00-overview.md)** · **[Português](../pt/00-overview.md)**
