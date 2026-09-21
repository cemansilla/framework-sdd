# SDD AI Harness — Agentes y Asignación de Modelos

Este documento define la matriz de agentes del framework SDD, la asignación de modelos por tipo de tarea y sus responsabilidades dentro del ciclo de desarrollo.

## 🤖 Matriz de Agentes

| Agente | Sub-comando | Modelo Asignado | Rol Principal |
| :--- | :--- | :--- | :--- |
| **Orquestador** | `/orchestrate` | `qwen3.7-plus` | Analizar el contexto, enrutar tareas y controlar el flujo SDD. |
| **Arquitecto** | `/plan` | `kimi-k3` | Diseño de arquitectura, contratos de API e interfaces complejas. |
| **Planificador** | `/tasks` | `qwen3.7-plus` | Desglose de especificaciones en listas de tareas granulares (`tasks.md`). |
| **Desarrollador** | `/implement` | `kimi-k2.7-code` | Escritura, modificación y refactorización de código fuente. |
| **QA / Tester** | `/test` | `deepseek-v4.1-flash` | Generación y ejecución masiva de pruebas unitarias e integración. |
| **Revisor** | `/review` | `glm-5.3-flash` | Auditoría de seguridad, calidad de código y adherencia a la especificación. |

---

## 📋 Responsabilidades y Flujo por Agente

### 1. Orchestrator (`@orchestrator`)
- **Propósito:** Supervisar la transición entre fases (Especificación -> Tareas -> Código -> Pruebas -> Revisión).
- **Herramientas:** Lectura de directorio y estado de `tasks.md`.

### 2. Architect (`@architect`)
- **Propósito:** Consumir los documentos de especificación (`.docs/02-technical-specification.md`) y traducir requerimientos en arquitectura de software.
- **Modelo:** Utiliza **Kimi K3** por su alta capacidad de razonamiento lógico.

### 3. Planner (`@planner`)
- **Propósito:** Generar o actualizar `.docs/development-tasks.md` o `.docs/poc/tasks.md` garantizando criterios de aceptación claros.

### 4. Coder (`@coder`)
- **Propósito:** Ejecutar las tareas pendientes de la lista.
- **Modelo:** Utiliza **Kimi K2.7 Code** para producir código limpio, tipado y quirúrgico.

### 5. Tester (`@tester`)
- **Propósito:** Crear la suite de pruebas automatizadas y ejecutarlas mediante CLI.
- **Modelo:** Utiliza **DeepSeek V4.1 Flash** debido a su alto volumen de peticiones y velocidad.

### 6. Reviewer (`@reviewer`)
- **Propósito:** Validar que el código implementado satisfaga la especificación antes de dar por completada una fase.