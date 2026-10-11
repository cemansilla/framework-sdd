---
description: Ejecuta la implementación de la siguiente tarea pendiente siguiendo el flujo SDD completo.
---

Iniciando fase de implementación...

1. Leer `.docs/03-development-tasks.md` (o `.docs/poc/tasks.md`) para identificar la siguiente tarea pendiente (`[ ]`).
2. Verificar que la tarea cumple la Definition of Ready.
3. Crear branch desde `develop`:
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b <tipo>/<TASK-ID>_<descripcion>
   ```
4. Delegar a `@coder` la implementación de la tarea.
5. `@coder` debe:
   - Implementar el código en `src/`.
   - Ejecutar quality gates: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.
6. Delegar a `@tester` para generar y ejecutar la suite de pruebas.
7. Delegar a `@reviewer` para auditar el código.
8. Si `@reviewer` dictamina `REQUIERE_CAMBIOS`, volver al paso 4 con las observaciones.
9. Si `@reviewer` dictamina `APROBADO`:
   - Crear commit convencional: `<tipo>(<scope>): [TASK-ID] <descripcion>`.
   - Marcar la tarea como completada (`[x]`) en `tasks.md`.
   - Push de la branch a remote.
   - Crear Pull Request hacia `develop`.
10. Reportar al usuario el resultado y el link del PR creado.

**Reglas inquebrantables:**
- El agente SOLO trabaja en su rama de trabajo (la que creó para la tarea).
- El agente NUNCA trabaja directamente en `develop`, `main` ni ninguna rama ajena a su rama de trabajo.
- El agente SÍ debe hacer `git push` de su rama de trabajo a remote.
- El agente NO ejecuta `git merge`, `git rebase` ni `git cherry-pick` sobre `develop` o `main`.
- El agente NO elimina branches remotas.
- El merge lo realiza el usuario tras revisar el PR.
