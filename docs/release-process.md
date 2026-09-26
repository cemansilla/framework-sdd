# Proceso de Versionado y Release

Define cómo se versiona, integra y publica el framework SDD.

## 1. Versionado

- **SemVer** sobre `0.x`: mientras `0.x`, los minor pueden romper
  compatibilidad; a partir de `1.0` se aplica compatibilidad estable.
- **Fuente única de versión**: `[workspace.package] version` en la raíz
  del `Cargo.toml` (los crates usan `version.workspace = true`).
- **MSRV**: `rust-version = "1.80"` en workspace; cambiarlo es decisión
  registrada como ADR.
- **Formato de artefactos**: `Manifest::CURRENT_FORMAT_VERSION` en
  `sdd-storage` se migra con `MigrationManager`; no se cambia sin migración.

## 2. Flujo de branches

```mermaid
flowchart LR
    F[feature|fix/…_task] -->|PR + gates| D[develop]
    D -->|PR squash 1:1| M[main]
    M -->|tag v0.x.y| R[release]
```

- Toda rama nace de `develop`; integración **solo por PR** (el agente
  jamás mergea — ver `AGENTS.md`).
- **develop ← feature**: rebase + squash (un commit por tarea).
- **main ← develop**: **squash merge, un commit por release**.
- `main` solo recibe releases; su historia se re-alinea en cada squash de
  release (el contenido es lo que importa, no el historial previo).

## 3. Gestión del changelog

- Formato Keep-a-Chango adaptado al spec §20:
  `## [Unreleased]` con entradas `### [ID]` (ID = `CHG-NNN`) o entradas
  sueltas `## [ID] …`.
- Todo cambio relevante se agrega **al momento** en
  `.sdd/changes/CHANGELOG.md` (Definition of Done).
- En release: mover entradas de `[Unreleased]` a `## [v0.x.y] — fecha`
  y abrir un `[Unreleased]` vacío.
- `sdd changelog` lee y renderiza este archivo.

## 4. Checklist de release

1. **Pre-flight**
   - [ ] PR de fase pendientes mergeados (incl. recovery de fases caídas);
   - [ ] `cargo fmt --all -- --check` ✓;
   - [ ] `cargo clippy --workspace --all-targets -- -D warnings` ✓;
   - [ ] `cargo test --workspace` ✓;
   - [ ] `cargo build --release` ✓ y smoke test del binario (`init`,
         `status`, `validate`, `changelog`);
   - [ ] plan `.docs/03-development-tasks.md` refleja el estado real;
   - [ ] CHANGELOG con las entradas del periodo.
2. **Versionado**
   - [ ] bump de `[workspace.package] version`;
   - [ ] título de release en CHANGELOG.
3. **Integración**
   - [ ] PR `develop → main` con **squash merge** (mensaje: versión);
   - [ ] verificar que `main` contiene el árbol completo esperado
         (workspace, `tests/`, PoC, `.sdd/`, docs).
4. **Tag y publicación**
   - [ ] `git tag -a v0.x.y -m "v0.x.y"` sobre el squash de `main`;
   - [ ] push del tag;
   - [ ] artefacto: binario desde `cargo build --release`
         (`sdd --version` → `sdd 0.x.y`);
   - [ ] release notes desde el CHANGELOG.
5. **Post-release**
   - [ ] `develop` continúa desde su HEAD (no se retroalimenta `main`);
   - [ ] verificar CI verde en ambos branches.

## 5. Estado de publicación de crates

Los crates son **`publish = false`** en v0.1.0: dependencias solo `path`
(sin versión), por lo que **crates.io está fuera de alcance** hasta
preparar dependencias versionadas. El artefacto de release es el
**binario** `sdd`.

## 6. CI

`.github/workflows/ci.yml` ejecuta en cada PR los tres gates
(fmt / clippy / test) sobre el workspace completo; un release nunca se
crea con CI rojo.
