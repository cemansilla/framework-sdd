# TASK-FW-201 — Cambio: entradas de changelog anidadas bajo `[Unreleased]`

- **Fase**: 15 — Framework self-hosting
- **Cambio**: CHG-001 (`.sdd/changes/CHG-001-support-unreleased-nesting.md`)
- **Agente**: coder
- **Dependencias**: TASK-FW-200
- **Estado**: Done
- **Testing strategy**: unit tests del parser para nivel 3 y orden mixto

## Pasos

1. Registrar el cambio en `.sdd/changes/CHG-001-support-unreleased-nesting.md`
   con motivo, origen, artefactos afectados, impacto y plan de propagación.
2. Extender `parse_heading`/`parse_changelog` para aceptar encabezados `### [ID]`.
3. Agregar tests: entrada anidada bajo `[Unreleased]`, orden mixto `##`/`###`.
4. Propagar: actualizar REQ-CLI-001 y registrar CHG-001 en
   `.sdd/changes/CHANGELOG.md` con todos los campos de la spec §20.
5. Ejecutar quality gates.

## Criterios de aceptación

- Un changelog con `## [Unreleased]` + `### [CHG-001] …` lista la entrada.
- Las entradas de nivel 2 siguen funcionando (regresión cubierta por tests).
- El cambio queda registrado en `.sdd/changes/` y en el CHANGELOG.
