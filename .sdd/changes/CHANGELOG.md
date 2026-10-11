# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### [CHG-003] 2026-10-11 — Sistema de fallback de modelos del framework (agnóstico al proveedor)

- **Motivo**: OpenCode no soporta fallback automático de modelos por agente: un modelo con rate limit dejaba el flujo muerto sin alternativa gestionada por el harness. El workaround previo (orquestador sin pin) resolvía solo el caso del default global.
- **Origen**: feature — resiliencia del harness ante fallos de modelo
- **Artefactos afectados**: crates/sdd-core (módulo `model_chain`: esquema de cadenas/providers/salud + parsing y validación puros), crates/sdd-storage (persistencia de `config/models.json` y `config/model-health.json`, edición atómica del `model:` en frontmatter, plantilla `config/models.json`), crates/sdd-cli (subcomando `sdd model status|health|fallback|report`, probe HTTP genérico openai/anthropic con credenciales por env o archivo, nunca literales), .opencode/agents/orchestrator.md (pin = cabeza de cadena + health check al arrancar + auto-fallback), .opencode/commands/model-fallback.md, AGENTS.md, docs/agent-authoring.md (§4.2), docs/user-guide.md, docs/cli-reference.md
- **Impacto**: cada rol tiene su cadena ordenada terminada en `inherit`; `sdd model fallback --role <rol>` prueba y fija el primer modelo sano (write-back por rol, atómico); si todo falla exit ≠ 0 sin escribir nada. El fallback no toca tareas, código ni git: lo avanzado se reanuda con `sdd status`. Fallos siempre en voz alta, nunca silenciosos. Pin de cada agente = cabeza de su cadena (test `test_agent_pins_match_chain_heads`).
- **Tareas afectadas**: TASK-FW-230

### [CHG-002] 2026-10-11 — Orquestador sin modelo fijado + cadena de preferencia ante rate limits

- **Motivo**: OpenCode no soporta fallback automático de modelos por agente. El orquestador tenía `model:` fijado a un free tier que al agotarse (rate limit, 6h de espera) dejaba el flujo muerto sin alternativa, y `bash: "*": deny` en los agentes custom disparaba además `AI_APICallError: OpenCode's free tier can only be used from within OpenCode` con el mismo modelo que `build`/`plan` usan sin problema (bugs upstream opencode#50627, #53347, #49680).
- **Origen**: fix — compatibilidad del harness con free tier y rate limits
- **Artefactos afectados**: .opencode/agents/*.md (orquestador sin pin + sección "Política de modelo"; `deny`→`ask` en coder/orchestrator/tester/reviewer), AGENTS.md (matriz + notas), docs/agent-authoring.md (§4.2), plantillas en crates/sdd-storage/templates/ (replican todo lo anterior)
- **Impacto**: el orquestador hereda siempre el modelo global activo (agnóstico al proveedor); ante un fallo de modelo informa en voz alta modelo + error + siguiente sugerido de la cadena (global → space-bunny-free → nemotron-3-ultra-free → modelo fuerte del proveedor pago) y nunca degrada en silencio. Los subagentes conservan pin por rol; `/models` mueve solo al orquestador. Los proyectos existentes adoptan el cambio con `sdd init --update`.
- **Tareas afectadas**: TASK-FW-230

### [CHG-001] 2026-10-11 — `sdd init` instala los archivos de proyecto del harness

- **Motivo**: `sdd init` solo creaba `.sdd/`. Un proyecto externo instalado con el binario quedaba sin agentes, comandos, skills ni `AGENTS.md`, es decir sin harness, por lo que el framework no era utilizable fuera de su propio repositorio. Tampoco existía forma de adoptar mejoras del harness al actualizar el framework.
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-storage (módulo `project_templates`, plantillas embebidas con `include_str!`, `install_project_files`, `update_project_files`), crates/sdd-cli (`init --update`), README.md, docs/user-guide.md
- **Impacto**: `sdd init` instala ahora `AGENTS.md`, `.opencode/{agents,commands,skills}/` y `.gitignore` además de `.sdd/`, de forma idempotente (no pisa archivos existentes). `sdd init --update` refresca esos archivos tras `cargo install --force`, sin tocar los artefactos del usuario en `.sdd/`. El framework se distribuye como binario (`cargo install --git https://github.com/cemansilla/framework-sdd sdd-cli`) y los proyectos externos no dependen de él en su `Cargo.toml`.
- **Tareas afectadas**: TASK-FW-230

### [TASK-FW-220] 2026-09-26 — Defaults a modelos gratuitos (OpenCode Zen)

- **Motivo**: Se agotó la cuota del plan pago `opencode-go`; los agentes y los defaults globales deben correr con modelos gratuitos ($0/Mtok) sin perder acceso a los de pago.
- **Origen**: change — cambio de configuración de modelos
- **Artefactos afectados**: .opencode/agents/*.md (6 agentes), opencode.json, AGENTS.md, docs/agent-authoring.md, docs/user-guide.md
- **Impacto**: orchestrator=longcat-2.5-preview-free, architect=nemotron-3-ultra-free, planner=ling-3.0-flash-fin-free, coder=big-pickle, tester=nemotron-3.5-lightning-free, reviewer=muse-spark-1.3-contributor-free; `model`=longcat y `small_model`=mimo-v2.6-flash-free (títulos/resúmenes/compactación sin cuota); `opencode-go/*` intacto para selección manual con `/models`; matriz rol→modelo (gratis vs pago) documentada en AGENTS.md y docs/agent-authoring.md
- **Tareas afectadas**: TASK-FW-220

## [v0.1.0] — 2026-09-26

### [TASK-FW-210..219] 2026-09-26 — Fase 16 — Documentación y release

- **Motivo**: Cerrar el framework con documentación completa para usuarios, integradores y autores, y preparar el proceso de release.
- **Origen**: feature
- **Artefactos afectados**: README.md, LICENSE, docs/ (user-guide, architecture, agents-integrations, cli-reference, mcp-reference, skill-authoring, agent-authoring, release-process), crates/*/Cargo.toml, .docs/03-development-tasks.md
- **Impacto**: 8 guías nuevas con índices enlazados desde el README; binario verificado en E2E (init/status/validate/changelog sobre el repo real: 0 errores y 0 warnings); metadatos de empaquetado (description, repository, publish=false) y LICENSE MIT; proceso de versionado y checklist de release documentados
- **Tareas afectadas**: TASK-FW-210..219

### [TASK-FW-200..205] 2026-09-26 — Fase 15 — Framework self-hosting

- **Motivo**: Ejecutar el flujo SDD sobre el propio framework: una feature (changelog lee el disco), un cambio (CHG-001), un fix desde test fallido, sincronización de documentación, verificación de changelog y de reproducibilidad.
- **Origen**: feature
- **Artefactos afectados**: .sdd/ (specs del proyecto), crates/sdd-storage (parser de changelog, initialize idempotente), crates/sdd-cli (comandos changelog/validate/status/init/architecture, binario `sdd`), crates/sdd-cli/tests/cli.rs, README.md, Cargo.toml, poc/logistics-gateway/Cargo.toml
- **Impacto**: CLI confiable (validate falla de verdad ante artefactos faltantes, status mide contenido real, init ya no destruye artefactos), nombre de binario unificado, 16 entradas de changelog con trazabilidad completa, specs SDD viviendo en `.sdd/`
- **Tareas afectadas**: TASK-FW-200..205

### [CHG-002] 2026-09-26 — Fase 13 restaurada en develop

- **Motivo**: El PR #14 (suite de calidad) se fusionó accidentalmente en `main` en lugar de `develop`; el crate `tests/` nunca llegó a develop.
- **Origen**: fix — corrección de integración
- **Artefactos afectados**: tests/, Cargo.toml, .docs/03-development-tasks.md
- **Impacto**: develop recupera los 9 targets de la suite de calidad (472 tests en total); el contenido de main queda de forma subconjunto de develop hasta el squash de release v0.1.0
- **Tareas afectadas**: TASK-FW-160..168

### [CHG-001] 2026-09-26 — Entradas de changelog anidadas bajo `[Unreleased]`

- **Motivo**: El parser solo aceptaba `## [ID]`, por lo que un changelog que siga la convención Keep a Changelog (entradas pendientes bajo `## [Unreleased]`) no mostraba ninguna entrada.
- **Origen**: Cambio de requisito (REQ-CLI-001)
- **Artefactos afectados**: crates/sdd-storage/src/changelog.rs, .sdd/requirements/REQ-CLI-001-changelog-reads-disk.md, .sdd/changes/CHANGELOG.md
- **Impacto**: El parser acepta encabezados `### [ID]` además de `## [ID]`; modelo y CLI sin cambios; entradas existentes compatibles.
- **Tareas afectadas**: TASK-FW-201

### [TASK-FW-180..196] 2026-09-25 — Fase 14 — PoC Logistics Gateway

- **Motivo**: Validar el framework SDD de punta a punta desarrollando un PoC real: gateway de cotización y gestión de envíos con carriers simulados.
- **Origen**: feature
- **Artefactos afectados**: poc/logistics-gateway/, Cargo.toml, .docs/poc/
- **Impacto**: Nuevo miembro del workspace con arquitectura hexagonal (domain/application/infrastructure), motor de márgenes, 3 carriers mock y demo end-to-end funcional
- **Tareas afectadas**: TASK-FW-180..196

### [TASK-FW-150..155] 2026-09-25 — Fase 12 — Observabilidad

- **Motivo**: Instrumentar el framework con métricas y auditoría de ejecuciones.
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-core (execution/token/retrieval/agent/validation metrics, audit_report)
- **Impacto**: Métricas agregadas y reportes de auditoría disponibles para orquestación y CLI
- **Tareas afectadas**: TASK-FW-150..155

### [TASK-FW-140..144] 2026-09-25 — Fase 11 — Trazabilidad Git

- **Motivo**: Vincular el estado SDD con branches, commits e issues externos.
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-core (branch_convention, commit_metadata, task_commit_mapping, issue_tracker, scope_validation)
- **Impacto**: Convención de branches y mapeo tarea↔commit verificables; integración con trackers externos
- **Tareas afectadas**: TASK-FW-140..144

### [TASK-FW-130..136] 2026-09-25 — Fase 10 — Change management

- **Motivo**: Soportar el ciclo completo de cambios: detección, impacto, propagación, conflictos, validación, historial y rollback.
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-core (change_detection, impact_analysis, change_propagation, conflict_resolution, change_validation, change_history, change_rollback)
- **Impacto**: Los cambios de requisito/architectura/diseño/tarea se registran y propagan con trazabilidad
- **Tareas afectadas**: TASK-FW-130..136

### [TASK-FW-120..128] 2026-09-25 — Fase 9 — Servidor MCP

- **Motivo**: Exponer las capacidades del framework a clientes MCP (editores de IA).
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-mcp (server JSON-RPC 2.0 sobre stdio, 8 operaciones, esquemas)
- **Impacto**: Los clientes MCP pueden consultar contexto, tareas y estado del proyecto
- **Tareas afectadas**: TASK-FW-120..128

### [TASK-FW-100..117] 2026-09-25 — Fase 8 — CLI

- **Motivo**: Proveer una interfaz de terminal para todo el ciclo SDD.
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-cli (17 subcomandos: init, status, brainstorm, requirements, questions, architecture, design, tasks, context, implement, test, review, change, impact, validate, changelog, hooks)
- **Impacto**: `sdd` se vuelve la puerta de entrada principal al framework
- **Tareas afectadas**: TASK-FW-100..117

### [TASK-FW-090..094] 2026-09-25 — Fase 7 — Integraciones de modelos

- **Motivo**: Estandarizar cómo se declara y valida el uso de modelos de IA.
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-integrations (model_profile, agent_contract, profiles, profile_versioning, validation)
- **Impacto**: Perfiles de modelo versionables y contratos de agente validables
- **Tareas afectadas**: TASK-FW-090..094

### [TASK-FW-080..085] 2026-09-25 — Fase 6 — Orquestación

- **Motivo**: Controlar el flujo de ejecución de agentes con contratos y permisos.
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-core (orchestrator, output_contract, permission_engine, approval_gates, execution_history)
- **Impacto**: Ejecuciones auditadas, permisos por agente y gates de aprobación
- **Tareas afectadas**: TASK-FW-080..085

### [TASK-FW-060..073] 2026-09-25 — Fase 5 — Context engineering

- **Motivo**: Construir Context Bundles reproducibles y presupuestados por tarea.
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-core (context_bundle, context_resolver, scope_resolver, semantic_ranker, token_budget, context_prioritizer, context_inspector, context_cache), crates/sdd-parser (AST), crates/sdd-index (léxico/vectorial)
- **Impacto**: Los agentes reciben contexto priorizado dentro de un presupuesto de tokens
- **Tareas afectadas**: TASK-FW-060..073

### [TASK-FW-050..057] 2026-09-25 — Fase 4 — Agentes y skills

- **Motivo**: Modelar la matriz de agentes, sus skills y su resolución.
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-core (agent/skill models, registries, resolver)
- **Impacto**: Skills declarativas resolubles por nombre con dependencias verificables
- **Tareas afectadas**: TASK-FW-050..057

### [TASK-FW-030..040] 2026-09-25 — Fase 3 — Lifecycle SDD

- **Motivo**: Implementar la máquina de estados del ciclo de vida de una tarea SDD.
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-core (lifecycle)
- **Impacto**: Transiciones de estado validadas con guards y efectos
- **Tareas afectadas**: TASK-FW-030..040

### [TASK-FW-020..025] 2026-09-25 — Fase 2 — Capa de persistencia

- **Motivo**: Persistir manifest, artefactos y estado de forma atómica y migrable.
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-storage (filesystem, manifest, structure, migration, lock)
- **Impacto**: `.sdd/` como fuente de verdad persistente con escritura atómica y file locking
- **Tareas afectadas**: TASK-FW-020..025

### [TASK-FW-010..018] 2026-09-25 — Fase 1 — Modelo de dominio

- **Motivo**: Definir los modelos centrales del framework (brief, requirements, tasks, agents, skills, changes, etc.).
- **Origen**: feature
- **Artefactos afectados**: crates/sdd-core (9 modelos de dominio)
- **Impacto**: Lenguaje común tipado para todo el framework
- **Tareas afectadas**: TASK-FW-010..018

### [TASK-FW-001..004] 2026-09-24 — Fase 0 — Scaffolding

- **Motivo**: Montar el workspace Cargo multi-crate con CI y decisiones arquitectónicas iniciales.
- **Origen**: feature
- **Artefactos afectados**: Cargo.toml (workspace), crates/ (7 crates), .github/workflows/ci.yml, docs/adr/
- **Impacto**: Base compilable con fmt/clippy/test en CI y ADRs 001-003
- **Tareas afectadas**: TASK-FW-001..004
