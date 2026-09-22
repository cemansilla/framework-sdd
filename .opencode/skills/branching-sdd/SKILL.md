---
name: branching-sdd
description: Flujo de branching y PR para el desarrollo SDD. Usar cuando se necesite crear una rama, push de cambios, resolver conflictos de merge, o entender la estrategia de branching del proyecto. El agente NO mergea directamente, todo pasa por PR.
---

# Branching Strategy SDD

## Regla Principal

**Siempre ramificar desde `develop`.** Nunca desde `main`.

## Estructura de Ramas

```
main ──────────────────────────────────────── (releases)
  │
  └── develop ─────────────────────────────── (integración)
        │
        ├── feature/TASK-FW-001_cargo-scaffold
        ├── feature/TASK-FW-002_coding-conventions
        ├── fix/TASK-FW-005_correct-trait-signature
        ├── docs/TASK-FW-003_add-adrs
        └── chore/TASK-FW-004_ci-pipeline
```

## Tipos de Rama

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

## Naming Convention

```
<tipo>/<TASK-ID>_<descripcion-corta>
```

- `<tipo>`: feature, fix, chore, docs, refactor, test, ci, release
- `<TASK-ID>`: TASK-FW-XXX o TASK-POC-XXX (opcional para chores)
- `<descripcion>`: kebab-case, corta y descriptiva

## Flujo de Trabajo

### 1. Crear Branch

```bash
git checkout develop
git pull origin develop
git checkout -b <tipo>/<TASK-ID>_<descripcion>
```

### 2. Trabajar en la Branch

- Hacer commits atómicos siguiendo conventional commits.
- Ejecutar quality gates antes de cada commit.

### 3. Push y Crear PR

```bash
git push origin <branch-name>
gh pr create --base develop --title "<tipo>(<scope>): [TASK-ID] <descripcion>" --body "..."
```

### 4. Esperar Aprobación del Usuario

- El usuario revisa el PR.
- El usuario aprueba o pide cambios.
- Si pide cambios, corregir y pushear nuevos commits.

### 5. Merge (REALIZADO POR EL USUARIO)

El usuario mergea el PR via GitHub UI o CLI.

### 6. Cleanup (REALIZADO POR EL USUARIO)

El usuario elimina la branch local y remota después del merge.

### 7. Release a Main (cuando corresponda, REALIZADO POR EL USUARIO)

```bash
git checkout main
git merge --squash develop
git tag -a v0.1.0 -m "release: v0.1.0"
git push origin main --tags
```

## Merge Strategy

- **Develop ← Feature**: Rebase + squash (un commit por tarea).
- **Main ← Develop**: Squash merge (un commit por release).
- **Nunca**: Merge con merge commit (historial sucio).

## Reglas

1. Nunca commitear directamente en `develop` o `main`.
2. Siempre actualizar `develop` antes de crear una nueva branch.
3. Una tarea = un branch = un commit (squash).
4. Quality gates deben pasar antes del push.
5. Review `APROBADO` antes del push.
6. **El agente NUNCA ejecuta `git merge` sobre `develop` o `main`.**
7. **El agente NUNCA elimina branches remotas.**
8. El trabajo del agente termina en el push + creación del PR.
9. El merge y cleanup son responsabilidad exclusiva del usuario.
10. No mantener branches vivas más de 1 semana.

## Resolver Conflictos

```bash
git checkout develop
git pull origin develop
git checkout <branch-name>
git rebase develop
# Resolver conflictos
git add .
git rebase --continue
```

## Branch Protection (recomendado)

- `main`: solo merge desde `develop`, requiere review.
- `develop`: solo merge desde branches de tarea.
- Requieren quality gates passing.
