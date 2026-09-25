# ADR-001: Arquitectura Hexagonal (Ports & Adapters)

## Estado

Aceptado

## Contexto

El framework SDD debe ser extensible, testeable y desacoplado de implementaciones concretas de storage, LLMs, parsers AST y sistemas de indexación vectorial. Se necesita una arquitectura que permita:

- Evolucionar el core sin afectar adapters externos.
- Reemplazar implementaciones (ej: cambiar de filesystem a base de datos).
- Testear el core de forma aislada.
- Soportar múltiples proveedores de LLM sin modificar el core.

## Decisión

Adoptar **Arquitectura Hexagonal** (Ports & Adapters) con las siguientes capas:

### Core (`sdd-core`)
- Modelos de dominio puros (sin dependencias externas).
- Puertos (traits) para interacción con el exterior.
- Lógica de negocio: lifecycle, traceability, context engine, planning.

### Adapters
- `sdd-storage`: Implementación de `StoragePort` para filesystem.
- `sdd-parser`: Adapter para Tree-sitter AST.
- `sdd-mcp`: Adapter para Model Context Protocol.
- `sdd-index`: Adapter para indexación vectorial.
- `sdd-integrations`: Adapter para proveedores de LLM.
- `sdd-cli`: Interfaz de línea de comandos.

### Reglas de Dependencia
- El core **nunca** depende de adapters concretos.
- Los adapters dependen del core (implementan sus puertos).
- `sdd-cli` es el único que conoce todos los adapters (composition root).

## Consecuencias

### Positivas
- Core testeable en aislamiento.
- Fácil sustitución de adapters.
- Claridad en límites de responsabilidad.
- Permite desarrollo paralelo de core y adapters.

### Negativas
- Mayor cantidad de crates/archivos.
- Overhead inicial de configuración.
- Curva de aprendizaje para nuevos contribuidores.

## Referencias

- [Alistair Cockburn - Hexagonal Architecture](https://alistair.cockburn.us/hexagonal-architecture/)
- Especificación técnica: Sección 2 (Arquitectura)
