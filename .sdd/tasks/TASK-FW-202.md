# TASK-FW-202 — Fix desde un test fallido: CLI con bugs reales

- **Fase**: 15 — Framework self-hosting
- **Agente**: coder
- **Dependencias**: ninguna
- **Estado**: Done
- **Testing strategy**: tests de integración que ejecutan el binario real
  (`CARGO_BIN_EXE_sdd-cli`); primero rojos, luego verdes tras el fix

## Contexto (bugs detectados en la auditoría de autoevaluación)

1. `sdd architecture` panica en build debug: `--show` y `--style` ambos declaran
   short `-s` (clap assert → exit 101). Afecta a toda invocación del subcomando.
2. `sdd init` en el directorio actual produce `project_id: "sdd-project"`:
   `PathBuf::from(".").file_name()` es `None`, en lugar del nombre real del
   directorio.

## Pasos

1. Escribir tests de integración en `crates/sdd-cli/tests/cli.rs` que fallen:
   - `architecture --show` y `architecture --style hexagonal` deben salir con 0.
   - `sdd init` (cwd = directorio con nombre conocido) debe producir un
     manifest cuyo `project_id` sea el nombre del directorio.
2. Ejecutar y confirmar el fallo (rojo).
3. Fix: quitar el short flag duplicado en `architecture`; derivar `project_id`
   con fallback a `canonicalize`.
4. Reejecutar hasta verde + quality gates.

## Criterios de aceptación

- Los tests de integración pasan en build debug.
- `sdd architecture --show` no panica.
- `sdd init` en cualquier directorio registra el nombre correcto del proyecto.
