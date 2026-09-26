# TASK-FW-200 — Feature: `sdd changelog` lee `.sdd/changes/CHANGELOG.md`

- **Fase**: 15 — Framework self-hosting
- **Requisito**: REQ-CLI-001
- **Agente**: coder
- **Dependencias**: ninguna
- **Estado**: Done
- **Testing strategy**: unit tests del parser (función pura) + unit tests del
  formato de salida + verificación manual con `cargo run -p sdd-cli -- changelog`

## Pasos

1. Especificar el requisito en `.sdd/requirements/REQ-CLI-001-changelog-reads-disk.md`.
2. Crear `crates/sdd-storage/src/changelog.rs` con `ChangelogEntry` y
   `parse_changelog(&str) -> Vec<ChangelogEntry>` (parsing puro de los encabezados
   `## [ID] fecha — título`).
3. Registrar el módulo en `crates/sdd-storage/src/lib.rs`.
4. Reescribir `crates/sdd-cli/src/commands/changelog.rs`: leer el archivo,
   parsearlo y renderizar las entradas con una función `render` separada.
5. Tests unitarios: entradas válidas, archivo sin entradas, encabezados malformados.
6. Ejecutar quality gates del workspace.

## Criterios de aceptación

- `sdd changelog` en este repositorio lista las entradas reales del changelog.
- El parser no realiza I/O y está cubierto por unit tests.
- No se rompe ningún test existente del workspace.
