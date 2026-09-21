# ADR-003: MSRV 1.80 y Edition 2021

## Estado

Aceptado

## Contexto

El framework debe definir una versión mínima de Rust (MSRV) y una edition para garantizar:

- Compatibilidad con distribuciones estables.
- Acceso a features modernas del lenguaje.
- Estabilidad para usuarios enterprise.

## Decisión

- **MSRV**: Rust 1.80 (release: 2024-07-25).
- **Edition**: 2021.

### Justificación

Rust 1.80 incluye:
- `LazyCell` / `LazyLock` en std.
- `pattern` API improvements.
- Estabilidad para `async fn` en traits (parcial).
- Buen soporte para `thiserror 2.x`, `tokio 1.x`, `serde 1.x`.

Edition 2021 es la más reciente estable y ofrece:
- Disjoint capture en closures.
- `IntoIterator` para arrays.
- Or patterns en macro_rules.

## Consecuencias

### Positivas
- Features modernas disponibles.
- Base amplia de usuarios compatibles.
- Estabilidad a largo plazo.

### Negativas
- No se pueden usar features de Rust > 1.80.
- Requiere documentación clara del MSRV.

## Referencias

- [Rust Releases](https://github.com/rust-lang/rust/releases)
- [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/)
