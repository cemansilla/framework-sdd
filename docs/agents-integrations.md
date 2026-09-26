# Agentes e Integraciones — Documentación para Integradores

Cómo integrar agentes de IA y herramientas externas con el framework SDD.

## 1. Superficies de integración

```mermaid
flowchart TB
    subgraph Embedders
        OC[OpenCode + .opencode/agents]
        MCP Client[Cliente MCP]
        App[Aplicación Rust]
    end

    OC -->|slash commands / skills| AG[Agentes del harness]
    MCP Client -->|JSON-RPC sobre stdio| S[McpServer · sdd-mcp]
    App -->|traits de sdd-core| P[Puertos de dominio]

    S --> R[(Repositorios sdd-core)]
    P --> R
    AG --> CLI[binario sdd · .sdd/]
```

| Superficie | Mecanismo | Audiencia |
|------------|-----------|-----------|
| OpenCode | Agentes y skills en `.opencode/` + binario `sdd` | usuarios del harness |
| MCP | `McpServer` (JSON-RPC 2.0 sobre stdio) | editores/agentes MCP |
| Biblioteca | Puertos (`sdd-core::repository`, `IndexPort`, `StoragePort`, …) | integradores Rust |

## 2. Contrato de agente (`sdd-integrations::agent_contract`)

Un agente externo se declara mediante `AgentContract`:

| Campo | Contenido |
|-------|-----------|
| `protocol` | `AgentProtocol` (mecanismo de invocación) |
| `capabilities` | `AgentCapability[]` (qué operaciones soporta) |
| `authentication` | `AuthenticationMethod` (incluye `OAuth2Flow`) |
| `rate_limits` | `RateLimits` (req/s, burst) |
| `retry_policy` | `RetryPolicy` (intentos, backoff) |
| `request_format` / `response_format` | `RequestFormat` / `ResponseFormat` |
| `validation` | `ValidationCheck[]` → `ValidationResults` |

Flujo de validación: `IntegrationValidator` verifica un contrato contra un
perfil y produce `ValidationResult` con warnings/errores por severidad.

## 3. Perfiles de modelo (`sdd-integrations::model_profile`)

Un `ModelProfile` describe cómo invocar a un modelo:

- `ModelProvider` + identificador del modelo;
- `ModelCapability[]` (texto, visión, tool-calling, …);
- `ContextFormat` (forma del contexto entregado);
- `InvocationMechanism` (API, CLI, MCP, …);
- `McpCapabilities` cuando el mecanismo es MCP.

Perfiles disponibles:

- `get_builtin_profiles()` — catálogo builtin;
- `create_opencode_profile()` / `create_warp_profile()` — perfiles concretos.

**Versionado:** `ProfileVersionManager` registra `VersionChange` con
`ChangeType` y produce `ProfileVersion` semver; los errores de versión se
reportan como `VersionError`. Nunca se rompe un perfil consumido sin
bump de versión.

## 4. Servidor MCP (`sdd-mcp`)

Para exponer el proyecto a clientes MCP incrustar el servidor:

```rust
use sdd_mcp::McpServer;

let server = McpServer::new(project_repo, task_repo, traceability_repo, change_repo);
server.run().await?;   // loop stdin→stdout, una respuesta JSON por línea
```

- **Transporte:** JSON-RPC 2.0 newline-delimited sobre stdio.
- **8 operaciones** documentadas en la [referencia MCP](mcp-reference.md).
- Requiere implementaciones de los puertos
  `ProjectRepository`, `TaskRepository`, `TraceabilityRepository`,
  `ChangeRepository` (v0.1.0 solo incluye mocks de test).

## 5. Puertos de dominio (`sdd-core::repository`)

Para integrar un backend de persistencia alternativo a `.sdd/` (filesystem),
implementar los traits:

```text
ProjectRepository · ArtifactRepository · RequirementRepository
TaskRepository    · ArchitectureRepository · TraceabilityRepository
VerificationRepository · ChangeRepository
```

Todos devuelven `Result<_, RepositoryError>`. El adaptador canónico es
`FilesystemAdapter` (`sdd-storage`), que implementa además `StoragePort`
con escritura atómica y `LockManager` para concurrencia.

Para retrieval, implementar `IndexPort` (`sdd-index`) —
implementación de referencia: `LexicalIndex` + `LocalVectorIndex` con
`SimpleHashEmbedding` (sin dependencias externas).

## 6. Cómo agregar una integración nueva

1. **Declarar el contrato**: crear/actualizar `AgentContract` y validar con
   `IntegrationValidator`.
2. **Definir el perfil**: `ModelProfile` con capabilities y mecanismo; si
   cambia un perfil existente, versionarlo con `ProfileVersionManager`.
3. **Elegir la superficie**:
   - agente OpenCode → ver [authoring de agentes](agent-authoring.md);
   - cliente MCP → implementar repositorios y arrancar `McpServer`;
   - código Rust → implementar los puertos directamente.
4. **Registrar**: agregar el perfil al catálogo y las skills/scope en
   `AGENTS.md`.
5. **Validar**: `IntegrationValidator` + quality gates del workspace.

## 7. Estado en v0.1.0

- Los contratos, perfiles y el servidor MCP existen como biblioteca con
  tests; no hay repositorios de producción para MCP (solo mocks de test).
- La integración productiva canónica es **OpenCode + binario `sdd`**
  sobre `.sdd/` (ver [guía de usuario](user-guide.md)).
