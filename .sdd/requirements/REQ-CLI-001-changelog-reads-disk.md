# REQ-CLI-001 — `sdd changelog` lee el changelog del disco

- **ID**: REQ-CLI-001
- **Estado**: Implemented
- **Origen**: Framework self-hosting (Fase 15)
- **Tarea**: TASK-FW-200
- **Especificación**: `.docs/02-technical-specification.md` §20 (Changelog)

## Descripción

El comando `sdd changelog` debe leer `.sdd/changes/CHANGELOG.md` y listar las entradas
registradas. Actualmente imprime un mensaje fijo ("No changes recorded yet") sin
consultar el disco, por lo que el changelog creado por `sdd init` es inútil.

## Criterios de aceptación

1. Sin proyecto `.sdd/` → mensaje "No SDD project found" y sugerencia de `sdd init`.
2. Proyecto presente pero sin `changes/CHANGELOG.md` → mensaje claro indicando que
   el archivo no existe.
3. Archivo presente sin entradas → "No changes recorded yet."
4. Archivo con entradas → lista cada entrada con identificador, fecha y título.
5. El parsing del markdown es una función pura sin I/O, ubicada en `sdd-storage`,
   testeable con unit tests.
6. El formato de salida se genera en una función separada, testeable sin capturar
   stdout.
7. `cargo fmt`, `cargo clippy -D warnings` y `cargo test` pasan en todo el workspace.

## Formato de entrada soportado

```markdown
## [CHG-001] 2026-09-26 — Título del cambio

- **Campo**: valor
```

Entradas anidadas bajo `## [Unreleased]` (convención Keep a Changelog) usando
encabezados de nivel 3 — agregado por CHG-001:

```markdown
## [Unreleased]

### [CHG-001] 2026-09-26 — Título del cambio
```

## Fuente de verdad

La especificación §20 define los campos que cada cambio relevante debe registrar:
identificador, fecha, motivo, origen, artefactos afectados, impacto y tareas
afectadas. TASK-FW-201 introduce la exigencia de esos campos como cambio de
requisito (ver `.sdd/changes/CHG-001-traceable-changelog-entries.md`).
