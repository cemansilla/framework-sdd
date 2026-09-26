# Referencia MCP — Servidor del framework

`crates/sdd-mcp` expone operaciones SDD mediante **JSON-RPC 2.0 sobre stdio**
(nuevo línea-delimitado). Es una **biblioteca**: no incluye binario propio;
se incrusta desde otro proceso (ver §5).

## 1. Transporte y framing

- stdin: una petición JSON por línea (líneas vacías se ignoran);
- stdout: una respuesta JSON por línea + flush;
- el bucle termina en EOF; peticiones se procesan secuencialmente (sin
  concurrencia ni batching);
- respuesta: `{"jsonrpc":"2.0", "id": <eco>, "result": ...}` o
  `{"jsonrpc":"2.0","id":...,"error":{"code":...,"message":...}}`;
- las notificaciones (sin `id`) también reciben respuesta con `id: null`.

### Códigos de error

| Código | Cuándo |
|--------|--------|
| `-32700` | línea JSON inválida (`id: null`) |
| `-32000` | cualquier fallo de handler, incluido `Method not found: <method>` y parámetros ausentes/inválidos |

> No se implementan `-32601/-32602/-32603` ni el ciclo de vida MCP
> (`initialize`, `tools/list`, `ping`): caen en el default → `-32000`.

## 2. Operaciones

`McpOperations::list_operations()` documenta las mismas 8 operaciones con
esquemas JSON-Schema (`input_schema`).

| Método | Parámetros | Resultado | Efecto |
|--------|-----------|-----------|--------|
| `get_project_status` | `project_id` (UUID, req.) | `{project_id, name, description, version, created_at, updated_at}` | lectura |
| `get_task` | `task_id` (ID humano, req.) | `{task_id` (UUID), `title, description, status, assigned_agent, dependencies, created_at, updated_at}` | lectura |
| `get_task_context` | `task_id` (req.) | `{task_id, context: {task, related_artifacts, dependencies}, generated_at}` | lectura |
| `get_traceability` | `artifact_id` (UUID, req.) | `{artifact_id, related_artifacts[]}` (vecinos del grafo) | lectura |
| `submit_task_output` | `task_id`, `output` (req.) | `{status: "success", task_id, saved_at}` | **escritura**: añade a `task.outputs.artifacts_created` y `save` |
| `submit_validation` | `task_id`, `validation_result` (req.) | `{status: "success", task_id, validated_at}` | **escritura**: `outputs.metadata["validation_result"]` y `save` |
| `register_change` | `change_type`, `description` (req.) | `{status: "success", change_id` (UUID), `registered_at}` | **escritura**: `change_repo.save` |
| `get_impact` | `change_id` (UUID, req.) | `{change_id, impact: {change_type, affected_artifacts[], estimated_effort}}` | lectura |

Notas:

- **`register_change`**: `change_type` válido ∈ `requirement | architecture
  | design | implementation | bugfix | refactor | documentation |
  configuration`; otro valor → error. Devuelve el **UUID**, que es la clave
  para `get_impact` (round-trip).
- **`get_task`**: la búsqueda usa el ID humano (`TASK-001`), pero el campo
  `task_id` de la respuesta es el **UUID**.
- **`get_impact`**: `estimated_effort` es `"medium"` fijo en v0.1.0.
- Parámetro ausente o UUID inválido → error `-32000`
  (`"<campo> is required"`).

### Ejemplo

```json
{"jsonrpc":"2.0","id":1,"method":"get_project_status","params":{"project_id":"…-…"}}
→ {"jsonrpc":"2.0","id":1,"result":{"project_id":"…","name":"…","…"}}
```

## 3. Catálogo de esquemas

`McpOperations::list_operations()` → `Vec<McpOperation{name, description,
input_schema}>`. Descripciones oficiales (ej. `get_task_context`: "Get the
context bundle for a specific task"). Hoy es documentación
bibliotecaria — el server no lo invoca.

## 4. Limitaciones v0.1.0

- **Sin binario ni config**: no hay `[[bin]]`, flags ni variables de entorno;
  el server se construye con repositorios inyectados.
- **Sin repositorios de producción**: los únicos `impl` de
  `ProjectRepository`/`TaskRepository`/`TraceabilityRepository`/
  `ChangeRepository` son los `Mock*Repo` de los tests del crate.
- **Sin ciclo de vida MCP** estándar (handshake/tools).

## 5. Incrustación

```rust
use sdd_mcp::McpServer;

let server = McpServer::new(project_repo, task_repo, traceability_repo, change_repo);
server.run().await?;   // loop stdin→stdout bajo un runtime tokio
```

Ver también: [agentes e integraciones](agents-integrations.md),
[arquitectura](architecture.md).
