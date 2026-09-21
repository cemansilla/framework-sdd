---
name: rust-quality
description: Quality gates y estándares de calidad para código Rust. Usar cuando se necesite verificar la calidad del código, ejecutar fmt/clippy/test, definir estándares de código, o revisar convenciones de naming en Rust.
---

# Rust Quality Gates

## Quality Gates Obligatorios

Todo código Rust debe pasar estos checks antes de commitear:

### 1. Formateo

```bash
cargo fmt --check
```

Si falla, aplicar formato:

```bash
cargo fmt
```

### 2. Clippy (Linting)

```bash
cargo clippy --workspace -- -D warnings
```

Todos los warnings son errores. No se permite `#[allow(...)]` sin justificación.

### 3. Tests

```bash
cargo test --workspace
```

Todos los tests deben pasar.

### 4. Build

```bash
cargo build --workspace
```

Compilación exitosa sin warnings.

## Convenciones de Naming

| Elemento | Convención | Ejemplo |
|----------|------------|---------|
| Funciones | `snake_case` | `fn calculate_total()` |
| Variables | `snake_case` | `let item_count = 10;` |
| Tipos (struct, enum) | `PascalCase` | `struct QuoteRequest` |
| Traits | `PascalCase` | `trait CarrierAdapter` |
| Constantes | `SCREAMING_SNAKE_CASE` | `const MAX_RETRIES: u32 = 3;` |
| Módulos | `snake_case` | `mod context_engine;` |
| Crates | `kebab-case` (Cargo.toml) | `name = "sdd-core"` |
| Archivos | `snake_case.rs` | `context_bundle.rs` |
| Variantes enum | `PascalCase` | `QuoteRequestStatus::Pending` |
| Genéricos | Una letra mayúscula o `PascalCase` | `<T>`, `<E>`, `<T: Display>` |
| Lifetimes | `'` + letra minúscula | `<'a>`, `<'de>` |

## Documentación

### Doc Comments (público)

```rust
/// Calcula el total de un quote request.
///
/// # Arguments
///
/// * `request` - El quote request con los items y dirección.
///
/// # Returns
///
/// El total calculado incluyendo márgenes.
///
/// # Errors
///
/// Retorna `Error::InvalidAddress` si la dirección es inválida.
///
/// # Examples
///
/// ```
/// let total = calculate_total(&request)?;
/// ```
pub fn calculate_total(request: &QuoteRequest) -> Result<Decimal, Error> {
    // ...
}
```

### Comentarios internos

```rust
// TODO: refactor this to use the new adapter pattern
// FIXME: this panics on empty input
// HACK: workaround for upstream bug
// NOTE: this is intentional, see ADR-003
```

## Manejo de Errores

### Reglas

- **Nunca** `unwrap()` en código de producción.
- Usar `Result<T, E>` para funciones que pueden fallar.
- Usar `?` operator para propagar errores.
- Definir errores específicos con `thiserror`.
- Usar `anyhow` solo en el boundary (CLI, main).

### Ejemplo

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("file not found: {path}")]
    NotFound { path: String },
    #[error("permission denied: {path}")]
    PermissionDenied { path: String },
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn read_artifact(path: &str) -> Result<Artifact, StorageError> {
    // ...
}
```

## Testing

### Unit Tests (en el mismo archivo)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_total_with_valid_input() {
        let request = QuoteRequest::fixture();
        let total = calculate_total(&request).unwrap();
        assert_eq!(total, Decimal::new(1500, 2));
    }

    #[test]
    fn test_calculate_total_with_empty_items() {
        let request = QuoteRequest::empty();
        let result = calculate_total(&request);
        assert!(matches!(result, Err(Error::EmptyItems)));
    }
}
```

### Integration Tests (en tests/)

```rust
// tests/integration_test.rs
use sdd_core::*;

#[test]
fn test_full_lifecycle() {
    // ...
}
```

## Checklist de Calidad

Antes de commitear, verificar:

- [ ] `cargo fmt --check` pasa
- [ ] `cargo clippy -- -D warnings` pasa
- [ ] `cargo test --workspace` pasa
- [ ] `cargo build --workspace` pasa
- [ ] Sin `unwrap()` en código de producción
- [ ] Doc comments en APIs públicas
- [ ] Naming conventions respetadas
- [ ] Tests para funcionalidad nueva
- [ ] Edge cases cubiertos
- [ ] Sin código duplicado
- [ ] Sin imports no usados
