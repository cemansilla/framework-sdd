# TASK-FW-203 — Verificar sincronización de documentación

- **Fase**: 15 — Framework self-hosting
- **Agente**: reviewer/coder
- **Dependencias**: TASK-FW-200..202
- **Estado**: Done

## Auditoría (docs vs. realidad)

| # | Discrepancia | Evidencia | Resolución |
|---|---|---|---|
| 1 | `sdd init` sugiere editar `.sdd/config.toml`, archivo inexistente | `commands/init.rs` | Texto corregido a `.sdd/config/project.md` |
| 2 | `sdd tasks` recomienda `sdd plan`, subcomando inexistente | `commands/tasks.rs` | Texto corregido: tareas en `.sdd/tasks/` |
| 3 | Binario `sdd-cli` vs. ayuda que dice `sdd` (3 formas distintas de nombrar) | `sdd-cli/Cargo.toml`, `main.rs` | `[[bin]] name = "sdd"`; ayuda, `--version` y argv0 coinciden |
| 4 | `poc/logistics-gateway` declara `rust-version = "1.75"` vs. MSRV 1.80 del workspace (warning de clippy en CI) | `poc/logistics-gateway/Cargo.toml` | Hereda `rust-version.workspace` |
| 5 | README lista `agents.md`; el archivo es `AGENTS.md` | `README.md` | Corregido |
| 6 | `sdd validate` imprime "All checks passed" incondicionalmente | `commands/validate.rs` | Validación real con exit code 1 ante errores |
| 7 | `sdd status` marca ✓ porque `init` crea los directorios, sin verificar contenido | `commands/status.rs` | Chequea archivos reales (brief.md, architecture.md) y contenido (≥1 .md en requirements/tasks) |

## No incluido (fuera de alcance de esta fase)

Los comandos `brainstorm`, `requirements`, `questions`, `design`, `tasks`,
`context`, `implement`, `test`, `review`, `change`, `impact` y `hooks` siguen
siendo stubs sin persistencia; se documentarán como tales en la referencia CLI
de la Fase 16 (TASK-FW-213).

## Criterios de aceptación

- `cargo run -p sdd-cli -- --help` y `--version` usan el nombre `sdd`.
- `sdd validate` falla (exit 1) si se borra un artefacto requerido y pasa en un
  proyecto inicializado.
- `sdd status` distingue directorios vacíos de directorios con contenido.
- `cargo clippy --workspace` ya no emite warning de MSRV.
- Tests de integración cubren validate/status/help.
