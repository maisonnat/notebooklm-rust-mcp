# Notebook Sharing Specification

## Purpose

Gestión de visibilidad de notebooks: leer estado de sharing (público/privado, usuarios compartidos) y toggle público/privado. Cubre el caso de uso de automatización donde un agente IA publica notebooks para acceso vía link.

## Requirements

### Requirement: Get Share Status

El sistema MUST obtener el estado de sharing de un notebook usando el RPC `JFMDGd` con payload `[notebook_id, [2]]`. La respuesta MUST parsearse desde el formato posicional `[[[users]], [is_public], 1000]`.

#### Scenario: Get status of public notebook

- GIVEN un notebook existente con sharing público habilitado
- WHEN el cliente invoca `get_share_status(notebook_id)`
- THEN el sistema envía el RPC `JFMDGd` con payload `[notebook_id, [2]]`
- AND retorna `Ok(ShareStatus)` con `is_public = true`
- AND retorna `access = AnyoneWithLink`
- AND retorna `share_url` con la URL pública del notebook

#### Scenario: Get status of private notebook

- GIVEN un notebook existente con sharing restringido
- WHEN el cliente invoca `get_share_status(notebook_id)`
- THEN retorna `Ok(ShareStatus)` con `is_public = false`
- AND retorna `access = Restricted`
- AND retorna `share_url = None`

#### Scenario: Get status with shared users

- GIVEN un notebook compartido con usuarios específicos
- WHEN el cliente invoca `get_share_status(notebook_id)`
- THEN retorna `Ok(ShareStatus)` con `shared_users` no vacío
- AND cada `SharedUser` contiene `email`, `permission`, `display_name`, `avatar_url`

#### Scenario: Share status of non-existent notebook

- GIVEN un notebook_id que NO existe
- WHEN el cliente invoca `get_share_status(notebook_id)`
- THEN el sistema envía el RPC `JFMDGd`
- AND retorna `Err(String)` indicando que no se encontró

#### Scenario: Share status response with null/empty data

- GIVEN un notebook donde el RPC retorna respuesta vacía o null
- WHEN el cliente invoca `get_share_status(notebook_id)`
- THEN retorna `Ok(ShareStatus)` con defaults seguros
- AND `is_public = false`, `shared_users = []`, `share_url = None`
- AND NO retorna error por datos faltantes

### Requirement: Set Sharing Public

El sistema MUST habilitar o deshabilitar el acceso público de un notebook usando el RPC `QDyure`. Para habilitar (público), el payload MUST ser `[[[notebook_id, null, [1], [1, ""]]], 1, null, [2]]`. Para deshabilitar (privado), el payload MUST ser `[[[notebook_id, null, [0], [0, ""]]], 1, null, [2]]`. Después del toggle, el sistema MUST invocar `get_share_status` y retornar el estado actualizado.

#### Scenario: Enable public sharing

- GIVEN un notebook existente con sharing privado
- WHEN el cliente invoca `set_sharing_public(notebook_id, true)`
- THEN el sistema envía el RPC `QDyure` con `access = 1` (ANYONE_WITH_LINK)
- AND luego invoca `get_share_status(notebook_id)` internamente
- AND retorna `Ok(ShareStatus)` con `is_public = true`
- AND `share_url` contiene la URL pública

#### Scenario: Disable public sharing

- GIVEN un notebook existente con sharing público
- WHEN el cliente invoca `set_sharing_public(notebook_id, false)`
- THEN el sistema envía el RPC `QDyure` con `access = 0` (RESTRICTED)
- AND luego invoca `get_share_status(notebook_id)` internamente
- AND retorna `Ok(ShareStatus)` con `is_public = false`
- AND `share_url = None`

#### Scenario: Toggle sharing on non-existent notebook

- GIVEN un notebook_id que NO existe
- WHEN el cliente invoca `set_sharing_public(notebook_id, true)`
- THEN el sistema envía el RPC `QDyure`
- AND retorna `Err(String)` indicando fallo
- AND NO retorna un ShareStatus falso

#### Scenario: Toggle sharing with RPC failure

- GIVEN un notebook existente
- AND el RPC `QDyure` retorna error
- WHEN el cliente invoca `set_sharing_public(notebook_id, true)`
- THEN retorna `Err(String)` con descripción del error
- AND NO se invoca `get_share_status` (fall-fast antes del read)

### Requirement: ShareAccess Enum

El sistema MUST definir un enum `ShareAccess` con variantes `Restricted = 0` y `AnyoneWithLink = 1`, mapeando los códigos enteros que Google usa en el RPC `QDyure`.

#### Scenario: ShareAccess integer mapping

- GIVEN el enum `ShareAccess`
- THEN `Restricted.code() == 0`
- AND `AnyoneWithLink.code() == 1`
- AND `ShareAccess::from_code(0) == Some(Restricted)`
- AND `ShareAccess::from_code(1) == Some(AnyoneWithLink)`
- AND `ShareAccess::from_code(99) == None`

### Requirement: MCP Tool Registration

El servidor MCP MUST exponer 2 nuevas tools: `notebook_share_status`, `notebook_share_set`.

#### Scenario: MCP tool notebook_share_status

- GIVEN un MCP client conectado y autenticado
- WHEN invoca `notebook_share_status` con `{ notebook_id }`
- THEN el servidor ejecuta `get_share_status` y retorna estado de sharing

#### Scenario: MCP tool notebook_share_set

- GIVEN un MCP client conectado y autenticado
- WHEN invoca `notebook_share_set` con `{ notebook_id, public: true }`
- THEN el servidor ejecuta `set_sharing_public` y retorna estado actualizado

### Requirement: CLI Commands

El CLI MUST proveer subcomandos `share-status` y `share-set` para las 2 operaciones de sharing.

#### Scenario: Share status via CLI

- GIVEN credenciales válidas almacenadas
- WHEN usuario ejecuta `notebooklm-mcp share-status --notebook-id ID`
- THEN el sistema imprime estado de sharing (público/privado, usuarios, URL)

#### Scenario: Share set via CLI

- GIVEN credenciales válidas almacenadas
- WHEN usuario ejecuta `notebooklm-mcp share-set --notebook-id ID --public`
- THEN el sistema habilita sharing público e imprime estado actualizado

#### Scenario: Share set private via CLI

- GIVEN credenciales válidas almacenadas
- WHEN usuario ejecuta `notebooklm-mcp share-set --notebook-id ID --private`
- THEN el sistema deshabilita sharing público e imprime confirmación
