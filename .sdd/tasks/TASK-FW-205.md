# TASK-FW-205 — Verificar reproducibilidad

- **Fase**: 15 — Framework self-hosting
- **Agente**: tester/coder
- **Dependencias**: TASK-FW-200..204
- **Estado**: Done

## Defecto detectado durante la verificación

`FilesystemAdapter::initialize` reescribe siempre las plantillas y el
manifest: volver a correr `sdd init` en un proyecto existente **destruye**
el contenido (incluido el changelog). Esto rompe la propiedad de
reproducibilidad operativa: los artefactos derivados deben poder
reconstruirse, pero los artefactos del usuario no deben perderse.

## Pasos

1. Test A (reproducibilidad pura): dos `initialize` en directorios
   distintos con el mismo `project_id` producen la misma estructura y los
   mismos archivos de plantilla byte a byte; el manifest es idéntico
   normalizando `created_at`/`updated_at`.
2. Fix: `initialize` solo escribe plantillas/manifest si no existen
   (write-if-missing). Los artefactos existentes nunca se sobreescriben.
3. Test B (idempotencia): inicializar, modificar `brief.md` y regenerar el
   changelog, volver a inicializar → contenido preservado e idéntico al
   manifest original.
4. Ejecutar quality gates del workspace.

## Criterios de aceptación

- Test A y B pasan; regresiones existentes no se rompen.
- `sdd init` dos veces seguidas en un proyecto poblado no altera nada.
- Resultado verificado con `cargo test --workspace`.
