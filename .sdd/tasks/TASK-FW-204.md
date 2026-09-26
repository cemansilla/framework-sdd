# TASK-FW-204 — Verificar changelog

- **Fase**: 15 — Framework self-hosting
- **Agente**: reviewer/coder
- **Dependencias**: TASK-FW-200..202
- **Estado**: Done

## Pasos

1. Poblar `.sdd/changes/CHANGELOG.md` con el historial real de fases
   (0..14) reconstruido desde `git log --merges`, cada entrada con los
   campos de la spec §20 (motivo, origen, artefactos, impacto, tareas).
2. Registrar CHG-002: la Fase 13 (PR #14) se fusionó en `main` por error
   y se restaura en `develop` mediante `fix/TASK-FW-160_restore-phase13-tests`.
3. Agregar test de integración: escribir una entrada en el changelog de un
   proyecto temporal y verificar que `sdd changelog` la lista con todos
   sus campos.
4. Ejecutar `sdd changelog` en este repositorio y verificar la salida real.

## Criterios de aceptación

- El changelog del repo tiene ≥15 entradas históricas con campos completos.
- Test de integración cubre entrada anidada con campos.
- `sdd changelog` muestra las entradas reales por pantalla.
