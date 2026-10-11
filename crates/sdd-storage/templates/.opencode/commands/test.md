---
description: Genera y ejecuta la suite de pruebas sobre las modificaciones recientes.
---

Iniciando verificación de QA...

1. Identificar los archivos modificados recientemente:
   ```bash
   git diff --name-only HEAD~1
   ```
2. Leer `.docs/03-development-tasks.md` para obtener los criterios de aceptación de la tarea actual.
3. Delegar a `@tester` para:
   - Analizar el código modificado.
   - Generar tests unitarios en `src/` con `#[cfg(test)]`.
   - Generar tests de integración en `tests/` si aplica.
   - Cubrir happy path, edge cases, error handling y boundary conditions.
4. `@tester` debe ejecutar:
   ```bash
   cargo test --workspace -- --nocapture
   ```
5. Reportar al usuario:
   - Tests creados.
   - Tests pasados / fallados.
   - Detalles de fallos si los hay.
6. Si hay tests fallados, delegar a `@coder` para corregir y volver al paso 4.
