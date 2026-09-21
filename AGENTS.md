# SDD AI Harness — Agentes, Skills y Convenciones

Este documento define la matriz de agentes del framework SDD, la asignación de modelos, sus responsabilidades, las convenciones de branching, commits y el registro de skills.

---

## Matriz de Agentes

| Agente | Sub-comando | Modelo | Mode | Rol Principal |
| :--- | :--- | :--- | :--- | :--- |
| **Orquestador** | `/orchestrate` | `qwen3.7-plus` | `primary` | Analizar contexto, enrutar tareas y controlar el flujo SDD. |
| **Arquitecto** | `/plan` | `kimi-k3` | `subagent` | Diseño de arquitectura, contratos de API e interfaces. |
| **Planificador** | `/tasks` | `qwen3.7-plus` | `subagent` | Desglose de especificaciones en tareas granulares. |
| **Desarrollador** | `/implement` | `kimi-k2.7-code` | `subagent` | Escritura, modificación y refactorización de código Rust. |
| **QA / Tester** | `/test` | `deepseek-v4.1-flash` | `subagent` | Generación y ejecución de pruebas unitarias e integración. |
| **Revisor** | `/review` | `glm-5.3-flash` | `subagent` | Auditoría de seguridad, calidad y adherencia a la especificación. |

---

## Flujo de Trabajo SDD

```
@architect → @planner → @coder → @tester → @reviewer
```

| Fase | Agente | Input | Output |
|------|--------|-------|--------|
| Architecture | `@architect` | `.docs/01-brief.md`, `.docs/02-technical-specification.md` | Spec actualizada con arquitectura |
| Planning | `@planner` | `.docs/02-technical-specification.md` | `.docs/03-development-tasks.md` |
| Implementation | `@coder` | Tarea de `tasks.md`, spec | Código en `src/` |
| Testing | `@tester` | Código modificado, criterios | Tests en `tests/` |
| Review | `@reviewer` | Código + tests, spec | Dictamen `APROBADO` / `REQUIERE_CAMBIOS` |

---

## Branching Rules

### Regla Principal

**Siempre ramificar desde `develop`.** Nunca desde `main`.

### Naming Convention

```
<tipo>/<TASK-ID>_<descripcion-corta>
```

### Tipos de Rama

| Tipo | Uso | Ejemplo |
|------|-----|---------|
| `feature` | Nueva funcionalidad | `feature/TASK-FW-001_cargo-workspace-scaffold` |
| `fix` | Corrección de bug | `fix/TASK-FW-005_correct-trait-signature` |
| `chore` | Mantenimiento, config | `chore/update-opencode-dependencies` |
| `docs` | Documentación | `docs/TASK-FW-003_add-initial-adrs` |
| `refactor` | Refactorización | `refactor/TASK-FW-006_extract-filesystem-port` |
| `test` | Pruebas | `test/TASK-FW-010_add-domain-model-tests` |
| `ci` | CI/CD | `ci/TASK-FW-004_github-actions-pipeline` |
| `release` | Preparar release | `release/v0.1.0` |

### Merge Strategy

- **Develop ← Feature**: Rebase + squash (un commit por tarea).
- **Main ← Develop**: Squash merge (un commit por release).
- Nunca merge con merge commit.

---

## Conventional Commits

### Formato

```
<tipo>(<scope>): [TASK-ID] <descripcion>
```

### Tipos

| Tipo | Uso |
|------|-----|
| `feat` | Nueva funcionalidad |
| `fix` | Corrección de bug |
| `docs` | Documentación |
| `chore` | Mantenimiento, config, deps |
| `refactor` | Refactorización sin cambio funcional |
| `test` | Agregar/modificar pruebas |
| `ci` | Cambios en CI/CD |
| `style` | Formato, whitespace |
| `perf` | Mejora de rendimiento |
| `build` | Cambios en build system |

### Scopes

| Scope | Descripción |
|-------|-------------|
| `cli` | sdd-cli crate |
| `core` | sdd-core crate |
| `storage` | sdd-storage crate |
| `parser` | sdd-parser crate |
| `mcp` | sdd-mcp crate |
| `index` | sdd-index crate |
| `integrations` | sdd-integrations crate |
| `sdd` | Framework general |
| `deps` | Dependencias |
| `github` | CI/CD |
| `poc` | Proof of Concept |
| `docs` | Documentación |

### Reglas de Agrupación

1. Un commit por tarea — no mezclar tareas distintas.
2. Scope coherente — si toca un solo módulo, un commit.
3. Commits atómicos — cada commit debe ser funcional por sí mismo.
4. Sin merges — usar rebase + squash.

### Ejemplos

```
feat(context): [TASK-FW-01] implement context bundle resolver
fix(parser): [TASK-FW-05] correct trait signature extraction
docs(sdd): [TASK-FW-03] add initial ADRs
ci(github): [TASK-FW-004] add cargo fmt/clippy/test pipeline
```

---

## Skills Registry

| Skill | Propósito | Ubicación |
|-------|-----------|-----------|
| `cargo-workspace` | Inicializar y gerenciar workspace Cargo | `.opencode/skills/cargo-workspace/SKILL.md` |
| `sdd-task-workflow` | Ejecutar ciclo completo de una tarea SDD | `.opencode/skills/sdd-task-workflow/SKILL.md` |
| `conventional-commits` | Reglas de formato y agrupación de commits | `.opencode/skills/conventional-commits/SKILL.md` |
| `branching-sdd` | Flujo de branching y merge | `.opencode/skills/branching-sdd/SKILL.md` |
| `rust-quality` | Quality gates para Rust (fmt, clippy, test) | `.opencode/skills/rust-quality/SKILL.md` |

---

## Comandos Disponibles

| Comando | Descripción |
|---------|-------------|
| `/orchestrate` | Orquesta el flujo SDD completo para una tarea o fase. |
| `/plan` | Diseño de arquitectura y planificación de tareas. |
| `/tasks` | Genera o actualiza el desglose de tareas. |
| `/implement` | Ejecuta la implementación de la siguiente tarea pendiente. |
| `/test` | Genera y ejecuta la suite de pruebas. |
| `/review` | Auditoría y code review del código. |

---

## Definition of Done Global

- `cargo fmt --check` sin errores.
- `cargo clippy -- -D warnings` sin errores.
- `cargo test` exitoso.
- Tests de integración y E2E relevantes.
- Documentación actualizada.
- Trazabilidad mantenida.
- Changelog actualizado para cambios relevantes.
- Cada tarea terminada tiene evidencia de validación.

---

## Definition of Ready

Una tarea está lista para implementar cuando:

1. Objetivo claro y específico.
2. Scope acotado (no más de 1 día de trabajo).
3. Criterios de aceptación definidos.
4. Dependencias resueltas o no bloqueantes.
5. Interfaces relevantes definidas.
6. Testing strategy definida.
7. Agente asignado.
8. Skills requeridos identificados.
9. Sin preguntas bloqueantes.
