---
description: Inicia el proceso de diseño de arquitectura y planificación de tareas.
---

Iniciando proceso de planificación de arquitectura...

1. Leer los documentos de entrada:
   - `.docs/01-brief.md` — Visión del producto y principios.
   - `.docs/02-technical-specification.md` — Especificación técnica existente.
   - `.docs/poc/` — Especificación del PoC si aplica.
2. Delegar a `@architect` para analizar requerimientos y definir:
   - Arquitectura del sistema (hexagonal, ports & adapters).
   - Contratos de API (DTOs, endpoints, interfaces).
   - Modelos de datos y relaciones.
   - Decisiones arquitectónicas (ADRs).
3. `@architect` debe actualizar `.docs/02-technical-specification.md` con la arquitectura definida.
4. Una vez definida la arquitectura, delegar a `@planner` para generar la lista de tareas en:
   - `.docs/03-development-tasks.md` (framework).
   - `.docs/poc/tasks.md` (PoC).
5. Reportar al usuario la arquitectura definida y las tareas generadas.
