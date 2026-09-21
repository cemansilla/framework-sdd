---
name: cargo-workspace
description: Inicializar y gerenciar workspace Cargo con múltiples crates. Usar cuando se necesite crear, modificar o gestionar la estructura del workspace Rust, agregar crates, configurar dependencias compartidas o resolver problemas de compilación del workspace.
---

# Cargo Workspace Management

## Crear Workspace desde Cero

### 1. Cargo.toml root

```toml
[workspace]
resolver = "2"
members = [
    "crates/sdd-cli",
    "crates/sdd-core",
    "crates/sdd-storage",
    "crates/sdd-parser",
    "crates/sdd-mcp",
    "crates/sdd-index",
    "crates/sdd-integrations",
]

[workspace.package]
edition = "2021"
rust-version = "1.80"
license = "MIT"
repository = "https://github.com/..."

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
thiserror = "2"
anyhow = "1"
clap = { version = "4", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 2. Estructura de directorios

```
.
├── Cargo.toml              (workspace root)
├── Cargo.lock
├── crates/
│   ├── sdd-cli/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   ├── sdd-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs
│   └── ...
└── tests/                  (integration tests)
```

### 3. Cargo.toml de cada crate

```toml
[package]
name = "sdd-core"
version.workspace = true
edition.workspace = true
rust-version.workspace = true

[dependencies]
serde.workspace = true
thiserror.workspace = true
```

## Comandos Habituales

```bash
cargo build                          (build all crates)
cargo build -p sdd-cli               (build single crate)
cargo test                           (test all)
cargo test -p sdd-core               (test single crate)
cargo clippy --workspace -- -D warnings
cargo fmt --all
cargo tree                           (dependency tree)
cargo update                         (update lockfile)
cargo check --workspace              (fast type-check)
```

## Agregar un Nuevo Crate

```bash
cargo new crates/sdd-new-crate --lib
```

Luego agregar a `workspace.members` en el root `Cargo.toml`.

## Dependencias entre Crates

```toml
# En crates/sdd-cli/Cargo.toml
[dependencies]
sdd-core = { path = "../sdd-core" }
```

## Reglas

- Resolver siempre `"2"` (MSRV compatible).
- Versiones compartidas en `[workspace.dependencies]`.
- Cada crate debe compilar independientemente.
- No circular dependencies entre crates.
- `cargo fmt` y `cargo clippy` deben pasar en todo el workspace.
