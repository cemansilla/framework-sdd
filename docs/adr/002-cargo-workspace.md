# ADR-002: Cargo Workspace como Estructura de Proyecto

## Estado

Aceptado

## Contexto

El proyecto SDD requiere múltiples crates interrelacionados que comparten dependencias y convenciones. Se necesita una estructura que:

- Permita desarrollo coordinado entre crates.
- Comparta dependencias y versiones.
- Facilita CI/CD unificado.
- Mantenga compilación incremental eficiente.

## Decisión

Utilizar **Cargo Workspace** con la siguiente estructura:

```
sdd-framework/
├── Cargo.toml              (workspace root)
├── Cargo.lock              (committed para binarios)
├── crates/
│   ├── sdd-cli/            (binary crate - composition root)
│   ├── sdd-core/           (domain models + ports)
│   ├── sdd-storage/        (filesystem adapter)
│   ├── sdd-parser/         (tree-sitter adapter)
│   ├── sdd-mcp/            (MCP adapter)
│   ├── sdd-index/          (vector index adapter)
│   └── sdd-integrations/   (LLM provider adapters)
└── tests/                  (integration tests)
```

### Configuración Workspace

- `resolver = "2"` (MSRV compatible).
- Dependencias compartidas en `[workspace.dependencies]`.
- Versión unificada en `[workspace.package]`.
- MSRV definido: `rust-version = "1.80"`.

## Consecuencias

### Positivas
- Una sola `cargo build/test` para todo el proyecto.
- Dependencias consistentes entre crates.
- Lockfile compartido.
- CI simplificado.

### Negativas
- Requiere coordinación al cambiar versiones.
- Compilación completa más lenta si cambia el workspace.

## Referencias

- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- Especificación técnica: Sección 4 (Estructura inicial del workspace)
