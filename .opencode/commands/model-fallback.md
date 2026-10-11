---
description: Recupera el harness de un fallo de modelo recorriendo la cadena de fallback del rol.
---

Recuperando el harness tras un fallo de modelo...

1. Identificar el rol afectado (por defecto `orchestrator`):
   ```bash
   sdd model status
   ```
2. Si el rol figura como `UNHEALTHY` (o el usuario pegó un error de
   proveedor: rate limit, cuota, timeout), ejecutar:
   ```bash
   sdd model fallback --role <rol>
   ```
   Esto prueba cada modelo de la cadena en `.sdd/config/models.json` y fija
   el primero sano en `.opencode/agents/<rol>.md`. Solo toca ese archivo y
   `.sdd/config/model-health.json`: tareas, código y git quedan intactos.
3. Anunciar en voz alta: modelo anterior → modelo nuevo + qué falló.
4. Reanudar donde quedó: `sdd status` muestra la tarea `IN_PROGRESS`; el
   trabajo a medio hacer se retoma, no se rehace.
5. Si el fallback informa que TODO falló (exit ≠ 0): no inventar resultados.
   Reportar cada error observado y pedir al usuario que actúe (esperar la
   ventana del rate limit, ampliar la cadena en `.sdd/config/models.json`,
   o activar facturación del proveedor).

**Reglas inquebrantables:**
- Nunca commitear, editar código ni tocar `.sdd/` como parte del fallback.
- Nunca declarar éxito si el comando salió con error.
- El fallo de modelo siempre se informa; el silencio invalida el harness.
