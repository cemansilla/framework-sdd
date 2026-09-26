# CHG-001 — Entradas anidadas bajo `## [Unreleased]`

- **ID**: CHG-001
- **Fecha**: 2026-09-26
- **Origen**: Cambio de requisito (REQ-CLI-001)
- **Tareas afectadas**: TASK-FW-201
- **Estado**: Applied

## Motivo

La convención Keep a Changelog agrupa las entradas pendientes bajo el encabezado
`## [Unreleased]`. El parser de changelog (`sdd-storage::changelog`) solo acepta
entradas de nivel 2 (`## [ID]`), por lo que un changelog que siga la convención
no mostraría ninguna entrada en `sdd changelog`.

Se requiere soportar entradas de nivel 3 (`### [ID] …`) debajo de
`## [Unreleased]`, manteniendo el soporte de nivel 2.

## Artefactos afectados

- `crates/sdd-storage/src/changelog.rs` — parser y tests
- `.sdd/requirements/REQ-CLI-001-changelog-reads-disk.md` — criterios de aceptación
- `.sdd/changes/CHANGELOG.md` — este registro, escrito en el formato nuevo

## Impacto

- **Parser**: aceptar `### [ID]` como inicio de entrada; el marcador
  `## [Unreleased]` sigue ignorándose. Sin cambios en el modelo `ChangelogEntry`.
- **CLI**: sin cambios (render es agnóstico al nivel del encabezado).
- **Compatibilidad**: entradas `## [ID]` existentes siguen parseando igual.
- **Tests**: se agregan casos de anidamiento y orden mixto de niveles.

## Plan de propagación

1. Implementar el soporte de nivel 3 en el parser + tests.
2. Actualizar los criterios de aceptación de REQ-CLI-001.
3. Registrar este cambio en `.sdd/changes/CHANGELOG.md` usando el formato nuevo.
4. Ejecutar quality gates del workspace.
