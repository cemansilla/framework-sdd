# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### [CHG-001] 2026-09-26 — Entradas de changelog anidadas bajo `[Unreleased]`

- **Motivo**: El parser solo aceptaba `## [ID]`, por lo que un changelog que siga la convención Keep a Changelog (entradas pendientes bajo `## [Unreleased]`) no mostraba ninguna entrada.
- **Origen**: Cambio de requisito (REQ-CLI-001)
- **Artefactos afectados**: crates/sdd-storage/src/changelog.rs, .sdd/requirements/REQ-CLI-001-changelog-reads-disk.md, .sdd/changes/CHANGELOG.md
- **Impacto**: El parser acepta encabezados `### [ID]` además de `## [ID]`; modelo y CLI sin cambios; entradas existentes compatibles.
- **Tareas afectadas**: TASK-FW-201
