---
description: Orquesta el flujo SDD completo para una tarea o fase del desarrollo.
---

Iniciando orquestación del flujo SDD...

1. Analizar la solicitud del usuario y determinar la fase actual del ciclo de desarrollo.
2. Verificar el estado actual del proyecto:
   - Leer `.docs/03-development-tasks.md` para identificar tareas pendientes.
   - Verificar archivos modificados con `git status`.
3. Determinar la fase correspondiente:
   - Si no hay especificación técnica → delegar a `@architect` (fase Architecture).
   - Si hay spec pero no hay tareas → delegar a `@planner` (fase Planning).
   - Si hay tareas pendientes → delegar a `@coder` (fase Implementation).
   - Si hay código sin tests → delegar a `@tester` (fase Testing).
   - Si hay código con tests → delegar a `@reviewer` (fase Review).
4. Delegar la tarea al agente especializado correspondiente.
5. Reportar al usuario el resultado de la delegación y el estado actual.

Si la solicitud no encaja en ninguna fase, preguntar al usuario para clarificar.

Usar `todowrite` para trackear el progreso de las tareas delegadas.
