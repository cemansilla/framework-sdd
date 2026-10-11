---
name: conventional-commits
description: Reglas de formato y agrupación de commits convencionales. Usar cuando se necesite crear un commit, verificar el formato de un mensaje de commit, o definir la estrategia de commits del proyecto.
---

# Conventional Commits

## Formato

```
<tipo>(<scope>): [TASK-ID] <descripcion>
```

## Tipos Permitidos

| Tipo | Uso | Ejemplo |
|------|-----|---------|
| `feat` | Nueva funcionalidad | `feat(context): [TASK-FW-01] implement context bundle resolver` |
| `fix` | Corrección de bug | `fix(parser): [TASK-FW-05] correct trait signature extraction` |
| `docs` | Documentación | `docs(sdd): [TASK-FW-03] add initial ADRs` |
| `chore` | Mantenimiento, config, deps | `chore(deps): update opencode plugin to 1.18.31` |
| `refactor` | Refactorización sin cambio funcional | `refactor(storage): [TASK-FW-06] extract filesystem port` |
| `test` | Agregar/modificar pruebas | `test(core): [TASK-FW-10] add domain model unit tests` |
| `ci` | Cambios en CI/CD | `ci(github): [TASK-FW-004] add cargo fmt/clippy/test pipeline` |
| `style` | Formato, whitespace, sin cambio lógico | `style: apply cargo fmt to all crates` |
| `perf` | Mejora de rendimiento | `perf(index): [TASK-FW-09] optimize vector search` |
| `build` | Cambios en build system | `build(workspace): add sdd-mcp crate` |

## Scope

El scope indica el módulo o área afectada:

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
| `github` | CI/CD GitHub Actions |
| `poc` | Proof of Concept |
| `docs` | Documentación |

## TASK-ID

Referencia a la tarea en `.docs/03-development-tasks.md`:

```
[TASK-FW-001]
[TASK-POC-015]
```

## Reglas de Agrupación

1. **Un commit por tarea** — No mezclar tareas distintas en un commit.
2. **Scope coherente** — Si una tarea toca un solo módulo, un commit.
3. **Múltiples scopes** — Si una tarea toca múltiples módulos, un commit por scope con la misma TASK-ID.
4. **Commits atómicos** — Cada commit debe ser funcional por sí mismo.
5. **Sin merges** — Usar rebase + squash para mantener historial limpio.

## Ejemplos

### Correcto

```
feat(context): [TASK-FW-01] implement context bundle resolver
fix(parser): [TASK-FW-05] correct trait signature extraction
docs(sdd): [TASK-FW-03] add initial ADRs
test(core): [TASK-FW-10] add domain model unit tests
ci(github): [TASK-FW-004] add cargo fmt/clippy/test pipeline
```

### Incorrecto

```
feat: implement context bundle resolver and fix parser bug
```
(Mezcla dos tareas distintas)

```
[TASK-FW-01] implement context bundle resolver
```
(Falta tipo y scope)

## Breaking Changes

Si un cambio rompe compatibilidad:

```
feat(core)!: [TASK-FW-020] change artifact model API

BREAKING CHANGE: Artifact trait now requires async methods.
```

## Body (opcional)

Para cambios complejos, agregar body después de una línea en blanco:

```
feat(storage): [TASK-FW-06] implement filesystem adapter

Implement the StoragePort trait for local filesystem operations.
The adapter reads/writes artifacts to .sdd/ directory structure.

- Add FilesystemAdapter struct
- Implement read/write/list operations
- Add unit tests for all operations
- Handle permission errors gracefully
```
