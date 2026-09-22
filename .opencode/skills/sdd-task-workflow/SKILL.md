---
name: sdd-task-workflow
description: "Ejecutar el ciclo completo de una tarea SDD desde branch hasta PR. Usar cuando se necesite implementar una tarea siguiendo el flujo SDD: crear branch, implementar, testear, revisar, commitear y crear PR para revisión del usuario."
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
8. Push + PR           → crear PR hacia develop
9. Esperar aprobación  → el usuario revisa y mergea
```

**NOTA: El agente NO mergea. El merge lo realiza el usuario.**

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

## Paso 8: Push y Crear PR

```bash
git push origin <branch-name>
gh pr create --base develop --title "<tipo>(<scope>): [TASK-ID] <descripcion>" --body "## Cambios\n\n- ..."
```

## Paso 9: Esperar Aprobación

El usuario revisa el PR y:
- **Aprueba**: El usuario mergea el PR.
- **Pide cambios**: Corregir y volver al Paso 3.

**El agente NO ejecuta merge. El merge lo realiza el usuario.**

## Reglas

- **El agente SOLO trabaja en su rama de trabajo** (la que creó para la tarea).
- **El agente NUNCA trabaja directamente en `develop`, `main` ni ninguna rama ajena a su rama de trabajo.**
- **El agente SÍ debe hacer `git push` de su rama de trabajo a remote.**
- **El agente NO ejecuta `git merge`, `git rebase` ni `git cherry-pick` sobre `develop` o `main`.**
- **El agente NO elimina branches remotas.**
- Una tarea = un branch = un commit (squash).
- Quality gates deben pasar antes del push.
- Review debe ser `APROBADO` antes del push.
- El trabajo del agente termina en el push + creación del PR.
