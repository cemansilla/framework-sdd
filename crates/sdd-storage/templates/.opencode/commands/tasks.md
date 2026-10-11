---
description: Genera o actualiza el desglose de tareas desde una especificación técnica.
---

Iniciando desglose de tareas...

1. Leer la especificación técnica relevante:
   - `.docs/02-technical-specification.md` para el framework.
   - `.docs/poc/technical-specification.md` para el PoC.
2. Delegar a `@planner` para analizar la especificación y generar tareas granulares.
3. `@planner` debe crear o actualizar:
   - `.docs/03-development-tasks.md` para el framework.
   - `.docs/poc/tasks.md` para el PoC.
4. Cada tarea debe cumplir la Definition of Ready:
   - Objetivo claro y específico.
   - Scope acotado.
   - Criterios de aceptación definidos.
   - Dependencias resueltas o no bloqueantes.
   - Interfaces relevantes definidas.
   - Testing strategy definida.
   - Agente asignado.
   - Skills requeridos identificados.
5. Reportar al usuario las tareas creadas y su estado.
