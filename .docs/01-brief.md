# Brief de Producto — SDD AI Development Harness

## 1. Visión

`SDD` es un harness local de ingeniería de software basado en Spec-Driven Development (SDD). Su propósito es estructurar, coordinar y auditar el desarrollo asistido por modelos de IA durante todo el ciclo de vida de un proyecto.

El framework no es un LLM ni depende de un proveedor concreto. Mantiene el estado y la documentación del proyecto como artefactos versionables, prepara el contexto necesario para cada actividad y permite que agentes externos o integrados ejecuten las tareas.

El flujo debe cubrir desde brainstorming y definición inicial hasta requisitos, arquitectura, planificación, implementación, testing, revisión, fixes, refactoring y cierre.

## 2. Problema

El desarrollo asistido por IA puede degradarse cuando los agentes reciben demasiado contexto, cuando las decisiones quedan solamente en conversaciones o cuando la documentación y el código evolucionan de manera independiente.

Los problemas que el framework busca resolver son:

- exceso de contexto y consumo innecesario de tokens;
- pérdida de decisiones y conocimiento entre sesiones;
- agentes que reciben información irrelevante;
- falta de trazabilidad entre requisitos, arquitectura, tareas, código y tests;
- cambios de implementación que dejan documentación obsoleta;
- dificultad para reproducir cómo se llegó a una determinada implementación;
- dependencia excesiva de un agente, IDE o proveedor de modelos.

## 3. Propuesta de valor

### 3.1. Desarrollo basado en especificaciones

El framework transforma información inicial y conversaciones de planificación en artefactos estructurados:

`brief → requisitos → preguntas/decisiones → dominio → arquitectura → diseño técnico → tareas → ejecución → validación`.

Los agentes no deberían comenzar una tarea sin disponer de los contratos, dependencias y criterios de aceptación necesarios.

### 3.2. Context Slicing

Cada agente recibe una porción de contexto específica para su tarea. El contexto puede combinar:

- especificaciones;
- requisitos relacionados;
- decisiones y ADRs;
- tareas y dependencias;
- código relevante;
- firmas e interfaces obtenidas mediante AST;
- tests;
- skills;
- restricciones;
- resultados de ejecuciones anteriores.

La reducción de tokens debe medirse. No se considera un porcentaje de reducción predeterminado como garantía del producto.

### 3.3. Trazabilidad

Los artefactos deben relacionarse de forma explícita para responder preguntas como:

- ¿qué requisito implementa esta tarea?
- ¿qué tareas implementan este requisito?
- ¿qué tests verifican este comportamiento?
- ¿qué código está afectado por esta decisión?
- ¿qué documentación debe cambiar si cambia una implementación?
- ¿qué tareas quedan afectadas por un cambio de requisito?

### 3.4. Evolución controlada

Un cambio puede originarse en planificación, desarrollo, revisión, testing o un fix.

El framework debe detectar el impacto, actualizar los artefactos afectados, mantener la trazabilidad y registrar el cambio en el changelog.

## 4. Principios

1. **SDD-first:** las especificaciones son parte central del desarrollo.
2. **LLM-agnostic:** el framework no depende de un único modelo.
3. **Harness, no agente único:** coordina el trabajo de agentes y herramientas.
4. **Local-first:** el proyecto debe poder mantener su estado en `.sdd/` y Git.
5. **Human-in-the-loop:** existen puntos de aprobación explícitos.
6. **Contexto mínimo suficiente:** proporcionar lo necesario, no todo lo disponible.
7. **Trazabilidad completa:** conectar intención, decisiones, tareas, código y verificación.
8. **Documentación viva:** la documentación evoluciona junto con el código.
9. **Skills extensibles:** existen skills predefinidos y se pueden incorporar skills propios.
10. **Agentes extensibles:** existen roles predefinidos y posibilidad de definir agentes adicionales.
11. **Reproducibilidad:** el estado relevante debe poder reconstruirse desde el repositorio.
12. **Determinismo donde sea posible:** las operaciones de gestión, validación, relaciones y persistencia deben ser independientes del LLM.

## 5. Alcance

El framework debe soportar:

- brainstorming y exploración;
- brief y definición del proyecto;
- discovery;
- requisitos;
- preguntas abiertas;
- decisiones y supuestos;
- modelo de dominio;
- arquitectura;
- diseño técnico;
- ADRs;
- planificación;
- descomposición en tareas;
- asignación de agentes y skills;
- generación y actualización de contexto;
- implementación;
- testing;
- QA;
- review;
- fixes;
- refactoring;
- integración;
- documentación;
- changelog;
- trazabilidad;
- análisis de impacto de cambios;
- recuperación ante interrupciones.

## 6. Interfaces

El canal principal de interacción será mediante comandos del framework. MCP será una interfaz adicional para que agentes y herramientas externas puedan consultar y modificar el estado permitido.

Ejemplo conceptual:

```text
sdd init
sdd plan
sdd requirements
sdd architecture
sdd tasks
sdd task <id>
sdd context <id>
sdd implement <id>
sdd test <id>
sdd review <id>
sdd change <id>
sdd status
```

La CLI concreta y los comandos definitivos deberán quedar especificados en la especificación técnica.

## 7. Integración con modelos y agentes

El framework debe ser independiente del modelo utilizado.

Cuando una integración requiera conocimiento específico de una herramienta o agente, el framework podrá disponer de perfiles versionados para ese entorno. Estos perfiles describirán cómo preparar estructura, instrucciones, contexto o archivos requeridos por cada integración.

El conocimiento de estas integraciones forma parte del propio framework y evoluciona mediante Git.

## 8. Métricas

La primera versión debe registrar métricas que permitan evaluar:

- tokens de contexto;
- tokens recuperados;
- tokens de código/contexto;
- tokens de salida cuando estén disponibles;
- tiempo de ejecución;
- cantidad de iteraciones;
- fallos y reintentos;
- cobertura y resultados de tests;
- trazabilidad de requisitos a tests;
- porcentaje de artefactos afectados correctamente actualizados.

La eficiencia de tokens será una hipótesis medible, no una promesa fija.

## 9. PoC

La PoC será un **Shipping Logistics Gateway** con transportistas simulados/mockeados. Debe validar el flujo completo del framework sin depender de APIs externas reales.
