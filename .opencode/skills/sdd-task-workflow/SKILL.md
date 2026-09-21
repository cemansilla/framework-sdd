---
name: sdd-task-workflow
description: "Ejecutar el ciclo completo de una tarea SDD desde branch hasta merge. Usar cuando se necesite implementar una tarea siguiendo el flujo SDD: crear branch, implementar, testear, revisar, commitear y merge a develop."
---

# SDD Task Workflow

## Flujo Completo de una Tarea

```
1. Leer tarea          → .docs/03-development-tasks.md
2. Crear branch        → desde develop
3. Implementar         → @coder escribe código
4. Quality gates       → cargo fmt, clippy, test
5. Testear             → @tester genera tests
6. Revisar             → @reviewer audita
7. Commit              → conventional commit
8. Marcar done         → [x] en tasks.md
9. Merge a develop     → rebase + squash
10. Cleanup            → eliminar branch local
```

## Paso 1: Leer Tarea

```bash
# Identificar la siguiente tarea pendiente
grep -n "\[ \]" .docs/03-development-tasks.md | head -1
```

Verificar que la tarea cumple Definition of Ready:
- Objetivo claro
- Scope acotado
- Criterios de aceptación definidos
- Dependencias resueltas
- Sin bloqueos

## Paso 2: Crear Branch

```bash
git checkout develop
git pull origin develop
git checkout -b <tipo>/<TASK-ID>_<descripcion>
```

Naming: `feature/TASK-FW-001_cargo-workspace-scaffold`

## Paso 3: Implementar

Delegar a `@coder` con la tarea como contexto.

## Paso 4: Quality Gates

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

Todos deben pasar. Si alguno falla, volver a Paso 3.

## Paso 5: Testear

Delegar a `@tester` para generar suite de pruebas.

## Paso 6: Revisar

Delegar a `@reviewer` para auditoría.

Si `REQUIERE_CAMBIOS`: volver a Paso 3 con las observaciones.

## Paso 7: Commit

```bash
git add -A
git commit -m "<tipo>(<scope>): [TASK-ID] <descripcion>"
```

Ver skill `conventional-commits` para formato detallado.

## Paso 8: Marcar Done

Editar `.docs/03-development-tasks.md`:
- Cambiar `[ ]` a `[x]` en la tarea completada.
- Actualizar estado si aplica.

## Paso 9: Merge a Develop

```bash
git checkout develop
git merge --squash <branch-name>
git commit -m "<tipo>(<scope>): [TASK-ID] <descripcion>"
```

## Paso 10: Cleanup

```bash
git branch -d <branch-name>
```

## Reglas

- Nunca commitear directamente en `develop` o `main`.
- Una tarea = un branch = un commit (squash).
- Quality gates deben pasar antes del merge.
- Review debe ser `APROBADO` antes del merge.
