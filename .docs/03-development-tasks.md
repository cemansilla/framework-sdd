# Plan de Desarrollo — SDD AI Development Harness

## Objetivo

Implementar el framework completo definido en `technical-specification.md`, utilizando el propio framework progresivamente cuando resulte posible.

## Definition of Done global

- `cargo fmt --check` sin errores.
- `cargo clippy -- -D warnings` sin errores.
- `cargo test` exitoso.
- Tests de integración y e2e relevantes.
- Documentación actualizada.
- Trazabilidad mantenida.
- Changelog actualizado para cambios relevantes.
- No existen artefactos derivados obligatorios sin mecanismo de reconstrucción/invalidation.
- Cada tarea terminada tiene evidencia de validación.

## Fase 0 — Gobierno del proyecto

### TASK-FW-001 — Scaffolding del repositorio [x]
Crear Cargo workspace, crates iniciales, CI y estructura documental.

### TASK-FW-002 — Convenciones de desarrollo [x]
Definir Rust/MSRV, formato, linting, testing, commits, branches, versionado y reglas de agentes.

### TASK-FW-003 — ADR de arquitectura [x]
Formalizar arquitectura hexagonal, límites del core y decisiones iniciales.

### TASK-FW-004 — CI base [x]
Implementar pipeline de fmt, clippy, test y build.

## Fase 1 — Modelo de dominio

### TASK-FW-010 — Modelo Project [x]
Representar proyecto, configuración y metadata.

### TASK-FW-011 — Modelo Artifact [x]
Representar artefactos persistentes, tipo, versión, hash, origen y relaciones.

### TASK-FW-012 — Modelo Requirement [x]
Representar requisitos, criterios de aceptación y estado.

### TASK-FW-013 — Modelo Question/Decision/Assumption/Risk [x]
Implementar incertidumbre y decisiones.

### TASK-FW-014 — Modelo Architecture/ADR [x]
Representar arquitectura y decisiones arquitectónicas.

### TASK-FW-015 — Modelo Task [x]
Implementar tareas, dependencias, scope, estado, agente, skills y outputs.

### TASK-FW-016 — Modelo Verification [x]
Representar tests, reviews, resultados y evidencias.

### TASK-FW-017 — Modelo Change [x]
Representar cambios, impacto, origen y relaciones.

### TASK-FW-018 — Modelo Traceability Graph [x]
Implementar relaciones entre artefactos y navegación bidireccional.

## Fase 2 — Persistencia

### TASK-FW-020 — Estructura `.sdd/` [x]
Crear estructura inicial y templates.

### TASK-FW-021 — Manifest [x]
Implementar manifest/versionado de metadata.

### TASK-FW-022 — Repository ports [x]
Definir puertos de persistencia en el core.

### TASK-FW-023 — Filesystem adapter [x]
Implementar almacenamiento atómico.

### TASK-FW-024 — Locks/concurrency [x]
Evitar corrupción por operaciones concurrentes.

### TASK-FW-025 — Migration system [x]
Implementar migraciones/versionado de formatos persistentes.

## Fase 3 — Lifecycle SDD

### TASK-FW-030 — Lifecycle state machine [x]
Implementar estados y transiciones.

### TASK-FW-031 — Brainstorming [x]
Permitir registrar y estructurar exploración inicial.

### TASK-FW-032 — Brief generation [x]
Crear y actualizar brief.

### TASK-FW-033 — Discovery [x]
Gestionar preguntas, respuestas, supuestos y riesgos.

### TASK-FW-034 — Requirements [x]
Crear, validar y versionar requisitos.

### TASK-FW-035 — Domain modeling [x]
Generar y mantener modelo de dominio.

### TASK-FW-036 — Architecture [x]
Crear y actualizar arquitectura.

### TASK-FW-037 — Technical design [x]
Crear diseño técnico derivado de requisitos y arquitectura.

### TASK-FW-038 — ADR workflow [x]
Crear, aprobar, versionar y relacionar ADRs.

### TASK-FW-039 — Planning [x]
Crear plan de implementación.

### TASK-FW-040 — Task decomposition [x]
Descomponer trabajo y calcular dependencias.

## Fase 4 — Agents y Skills

### TASK-FW-050 — Agent model [x]
Definir metadata y contratos de agentes.

### TASK-FW-051 — Built-in agents [x]
Implementar Planner, Implementer, Reviewer y QA.

### TASK-FW-052 — Custom agents [x]
Permitir agentes propios.

### TASK-FW-053 — Skill model [x]
Definir metadata, versiones, dependencias y applicability.

### TASK-FW-054 — Built-in skills [x]
Crear catálogo inicial.

### TASK-FW-055 — Project skills [x]
Permitir skills propios por proyecto.

### TASK-FW-056 — Skill resolution [x]
Seleccionar skills aplicables a cada tarea.

### TASK-FW-057 — Agent resolution [x]
Seleccionar agente compatible con tarea y skills.

## Fase 5 — Context Engineering

### TASK-FW-060 — Context Bundle model [x]
Definir contrato del bundle.

### TASK-FW-061 — Explicit relation resolver [x]
Resolver contexto mediante grafo y dependencias.

### TASK-FW-062 — Scope resolver [x]
Determinar archivos y artefactos relevantes.

### TASK-FW-063 — AST adapter [x]
Integrar Tree-sitter.

### TASK-FW-064 — AST signatures [x]
Extraer interfaces, traits, tipos y firmas.

### TASK-FW-065 — Lexical retrieval [x]
Implementar recuperación textual.

### TASK-FW-066 — Vector retrieval port [x]
Definir adapter abstracto.

### TASK-FW-067 — Local vector index [x]
Implementar una primera implementación local.

### TASK-FW-068 — Semantic ranking [x]
Combinar señales explícitas y semánticas.

### TASK-FW-069 — Token budget [x]
Calcular presupuesto por tarea/agente/modelo.

### TASK-FW-070 — Context prioritization [x]
Aplicar P0/P1/P2/P3 y truncamiento.

### TASK-FW-071 — Context inspection [x]
Mostrar exactamente qué recibiría un agente.

### TASK-FW-072 — Context caching [x]
Cachear bundles y derivados reproducibles.

### TASK-FW-073 — Cache invalidation [x]
Invalidar según hashes, versiones y dependencias.

## Fase 6 — Orchestration

### TASK-FW-080 — Orchestrator core [x]
Implementar flujo estado → agente → contexto → ejecución → validación.

### TASK-FW-081 — Output contracts [x]
Definir contratos de salida por agente/tarea.

### TASK-FW-082 — Permission model [x]
Aplicar permisos mínimos.

### TASK-FW-083 — Approval gates [x]
Implementar gates humanos.

### TASK-FW-084 — Recovery/checkpoints [x]
Permitir reanudar operaciones.

### TASK-FW-085 — Execution history [x]
Registrar ejecuciones, fallos y reintentos.

## Fase 7 — Integraciones de modelos

### TASK-FW-090 — Integration profile model [x]
Definir perfiles de herramientas/modelos.

### TASK-FW-091 — External agent contract [x]
Definir cómo un agente externo solicita tareas/contexto.

### TASK-FW-092 — OpenCode profile [x]
Crear perfil inicial para probar el framework con OpenCode/Warp.

### TASK-FW-093 — Profile versioning [x]
Versionar capacidades y estructuras específicas por integración.

### TASK-FW-094 — Integration validation [x]
Detectar incompatibilidades de perfiles.

## Fase 8 — CLI

### TASK-FW-100 — `sdd init` [x]
### TASK-FW-101 — `sdd status` [x]
### TASK-FW-102 — `sdd brainstorm` [x]
### TASK-FW-103 — `sdd requirements` [x]
### TASK-FW-104 — `sdd questions` [x]
### TASK-FW-105 — `sdd architecture` [x]
### TASK-FW-106 — `sdd design` [x]
### TASK-FW-107 — `sdd tasks` [x]
### TASK-FW-108 — `sdd task` [x]
### TASK-FW-109 — `sdd context` [x]
### TASK-FW-110 — `sdd implement` [x]
### TASK-FW-111 — `sdd test` [x]
### TASK-FW-112 — `sdd review` [x]
### TASK-FW-113 — `sdd change` [x]
### TASK-FW-114 — `sdd impact` [x]
### TASK-FW-115 — `sdd validate` [x]
### TASK-FW-116 — `sdd changelog` [x]
### TASK-FW-117 — `sdd hooks` [x]

Cada comando debe tener contrato de entrada/salida, códigos de error y tests CLI.

## Fase 9 — MCP

### TASK-FW-120 — MCP server [x]
Implementar transporte y lifecycle.

### TASK-FW-121 — get_project_status [x]
### TASK-FW-122 — get_task [x]
### TASK-FW-123 — get_task_context [x]
### TASK-FW-124 — get_traceability [x]
### TASK-FW-125 — submit_task_output [x]
### TASK-FW-126 — submit_validation [x]
### TASK-FW-127 — register_change [x]
### TASK-FW-128 — get_impact [x]

## Fase 10 — Change Management

### TASK-FW-130 — Change detection [x]
Detectar cambios originados en cualquier fase.

### TASK-FW-131 — Impact graph [x]
Resolver artefactos afectados.

### TASK-FW-132 — Documentation synchronization [x]
Actualizar documentación afectada.

### TASK-FW-133 — Task synchronization [x]
Actualizar tareas afectadas.

### TASK-FW-134 — Test synchronization [x]
Actualizar/crear tests afectados.

### TASK-FW-135 — Changelog [x]
Registrar cambios.

### TASK-FW-136 — Revalidation [x]
Determinar qué gates deben repetirse.

## Fase 11 — Git y trazabilidad externa

### TASK-FW-140 — Branch conventions [x]
### TASK-FW-141 — Commit metadata [x]
### TASK-FW-142 — Task/commit mapping [x]
### TASK-FW-143 — Optional issue tracker adapter [x]
### TASK-FW-144 — Scope validation hook [x]

## Fase 12 — Observability

### TASK-FW-150 — Execution metrics [x]
### TASK-FW-151 — Token metrics [x]
### TASK-FW-152 — Retrieval metrics [x]
### TASK-FW-153 — Agent metrics [x]
### TASK-FW-154 — Validation metrics [x]
### TASK-FW-155 — Audit report [x]

## Fase 13 — Quality

### TASK-FW-160 — Unit test suite
### TASK-FW-161 — Integration test suite
### TASK-FW-162 — Contract tests
### TASK-FW-163 — CLI tests
### TASK-FW-164 — MCP tests
### TASK-FW-165 — E2E lifecycle test
### TASK-FW-166 — Failure/recovery tests
### TASK-FW-167 — Change propagation tests
### TASK-FW-168 — Context budget tests

## Fase 14 — PoC

### TASK-FW-180 — Initialize Logistics Gateway [x]
### TASK-FW-181 — PoC brief/discovery [x]
### TASK-FW-182 — PoC requirements [x]
### TASK-FW-183 — PoC domain model [x]
### TASK-FW-184 — PoC architecture [x]
### TASK-FW-185 — PoC technical design [x]
### TASK-FW-186 — PoC task generation [x]
### TASK-FW-187 — Mock carrier interfaces [x]
### TASK-FW-188 — Quote endpoint [x]
### TASK-FW-189 — Carrier adapters [x]
### TASK-FW-190 — Margin engine [x]
### TASK-FW-191 — Shipping order flow [x]
### TASK-FW-192 — Persistence [x]
### TASK-FW-193 — Testing [x]
### TASK-FW-194 — Review/QA [x]
### TASK-FW-195 — Change/fix exercise [x]
### TASK-FW-196 — PoC final audit [x]

## Fase 15 — Framework self-hosting

### TASK-FW-200 — Develop a feature using SDD [x]
### TASK-FW-201 — Develop a change using SDD [x]
### TASK-FW-202 — Develop a fix from a failed test [x]
### TASK-FW-203 — Verify documentation synchronization [x]
### TASK-FW-204 — Verify changelog [x]
### TASK-FW-205 — Verify reproducibility [x]

## Fase 16 — Documentation and release

### TASK-FW-210 — User documentation
### TASK-FW-211 — Agent/integration documentation
### TASK-FW-212 — Architecture documentation
### TASK-FW-213 — CLI reference
### TASK-FW-214 — MCP reference
### TASK-FW-215 — Skill authoring guide
### TASK-FW-216 — Agent authoring guide
### TASK-FW-217 — Release packaging
### TASK-FW-218 — Versioning/release process
### TASK-FW-219 — Final E2E release validation
