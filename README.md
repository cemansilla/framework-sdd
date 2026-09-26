# SDD AI Development Harness

Framework de desarrollo de software asistido por IA basado en
Specification-Driven Development (SDD): especificación, planificación,
implementación, testing y review dirigidos por agentes de IA, con el estado
del proyecto persistido en un directorio `.sdd/` versionado con Git.

## Inicio rápido

```bash
# Requisitos: Rust 1.80+ (MSRV), Git
cargo build --release
export PATH="$PWD/target/release:$PATH"

# Inicializar un proyecto SDD en el directorio actual
sdd init

# Ver estado y validar artefactos
sdd status
sdd validate          # exit 1 si faltan artefactos requeridos

# Ver el historial de cambios
sdd changelog
```

`sdd init` es idempotente: nunca sobrescribe artefactos existentes.

## Estructura de un proyecto `.sdd/`

```text
.sdd/
├── manifest.json          # identidad y versión de formato
├── brief/                 # brief de producto
├── discovery/             # preguntas, supuestos, riesgos
├── requirements/          # requisitos (REQ-*.md)
├── domain/                # modelo de dominio
├── architecture/          # arquitectura + adr/
├── design/                # diseño técnico
├── tasks/                 # tareas (TASK-*.md)
├── agents/ · skills/      # agentes y skills del proyecto
├── tests/                 # plan/pruebas
├── changes/               # CHANGELOG.md + registros CHG-*
├── context/               # context bundles (derivado, reconstruible)
└── index/                 # índices de retrieval (derivado, reconstruible)
```

`context/` e `index/` son derivados: pueden borrarse y reconstruirse.

## Documentación

| Documento | Contenido |
|-----------|-----------|
| [Guía de usuario](docs/user-guide.md) | Flujo completo de uso del framework |
| [Arquitectura](docs/architecture.md) | Diseño hexagonal, crates y flujo de datos |
| [Referencia CLI](docs/cli-reference.md) | Los 17 comandos de `sdd` con flags y estado |
| [Referencia MCP](docs/mcp-reference.md) | Protocolo y operaciones del servidor MCP |
| [Agentes e integraciones](docs/agents-integrations.md) | Contratos de agente y perfiles de modelo |
| [Guía de authoring de skills](docs/skill-authoring.md) | Cómo crear skills para el harness |
| [Guía de authoring de agentes](docs/agent-authoring.md) | Cómo crear agentes para el harness |
| [Proceso de release](docs/release-process.md) | Versionado, branches y checklist de release |
| [ADRs](docs/adr/) | Decisiones de arquitectura registradas |
| [Especificación técnica](.docs/02-technical-specification.md) | Especificación completa (fuente de verdad) |
| [Plan de desarrollo](.docs/03-development-tasks.md) | Tareas por fase con trazabilidad |
| [AGENTS.md](AGENTS.md) | Matriz de agentes, branching y convenciones |

## Workspace

| Crate | Propósito |
|-------|-----------|
| `sdd-core` | Modelo de dominio y puertos (sin I/O) |
| `sdd-storage` | Persistencia en `.sdd/` (filesystem, manifest, changelog, locks) |
| `sdd-parser` | Parsing de código (firmas AST) y markdown |
| `sdd-index` | Retrieval léxico y vectorial |
| `sdd-integrations` | Contratos de agente y perfiles de modelo |
| `sdd-mcp` | Servidor JSON-RPC 2.0 sobre stdio (biblioteca) |
| `sdd-cli` | Binario `sdd` |
| `poc/logistics-gateway` | PoC: gateway logístico (validación del framework) |

## Convenciones

- **Trazabilidad:** `.docs/` + `.sdd/` + Git son la fuente de verdad; todo
  cambio relevante se registra en `.sdd/changes/CHANGELOG.md`.
- **Diagramas:** Mermaid es el formato obligatorio.
- **Commits:** Conventional Commits con `[TASK-ID]` (ver `AGENTS.md`).
- **Ramas:** siempre desde `develop`; integración exclusivamente vía PR.
- **Quality gates:** `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo test --workspace`.
