---
description: Realiza una auditoría y code review sobre el código del proyecto.
---

Iniciando revisión de código...

1. Identificar los archivos de código fuente creados o modificados:
   ```bash
   git diff --name-only develop
   ```
2. Leer `.docs/02-technical-specification.md` para validar adherencia a la arquitectura.
3. Leer `.docs/03-development-tasks.md` para verificar criterios de aceptación.
4. Delegar a `@reviewer` para auditar:
   - **Seguridad**: sin secrets expuestos, manejo de errores, input validation.
   - **Calidad**: naming conventions, documentación, complejidad, duplicación.
   - **Adherencia**: arquitectura hexagonal, puertos/interfaces, separación core/adapters.
   - **Testing**: tests existen, cobertura adecuada, edge cases.
   - **Performance**: sin allocations innecesarias, patterns idiomáticos.
5. `@reviewer` debe ejecutar quality gates:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   cargo test
   ```
6. Reportar al usuario el dictamen:
   - `APROBADO` → proceder con commit y merge.
   - `REQUIERE_CAMBIOS` → listar observaciones concretas y delegar a `@coder` para corregir.
