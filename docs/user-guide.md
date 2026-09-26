# Guía de Usuario — SDD AI Development Harness

Guía práctica para llevar un proyecto desde cero usando el framework SDD.

## 1. Requisitos

- Rust **1.80+** (MSRV; edition 2021)
- Git
- Un agente de IA compatible con OpenCode (para el flujo con agentes) — ver
  [authoring de agentes](agent-authoring.md)

## 2. Instalación

```bash
git clone https://github.com/cemansilla/framework-sdd
cd framework-sdd
cargo build --release
# el binario queda en target/release/sdd
export PATH="$PWD/target/release:$PATH"
sdd --version        # sdd 0.1.0
```

## 3. Inicializar un proyecto

```bash
mkdir mi-proyecto && cd mi-proyecto
sdd init
```

Esto crea la estructura `.sdd/` con plantillas editables y un
`manifest.json` cuyo `project_id` se deriva del nombre del directorio.

Propiedades de `sdd init`:

- **Idempotente**: re-ejecutarlo no destruye contenido existente (solo crea
  archivos que falten).
- Siempre puede reconstruir las plantillas perdidas sin tocar artefactos
  del usuario.

## 4. Flujo de trabajo SDD

```mermaid
flowchart LR
    A[brief] --> B[discovery]
    B --> C[requirements]
    C --> D[architecture]
    D --> E[design]
    E --> F[tasks]
    F --> G[implement]
    G --> H[test]
    H --> I[review]
    I --> J[changelog]
```

| Fase | Artefacto en `.sdd/` | Herramienta |
|------|----------------------|-------------|
| Brief | `brief/brief.md` | plantilla de `sdd init` |
| Discovery | `discovery/{questions,assumptions,risks}.md` | plantillas |
| Requisitos | `requirements/REQ-*.md` | agente `@planner` o manual |
| Arquitectura | `architecture/architecture.md`, `architecture/adr/` | agente `@architect` |
| Diseño | `design/` | agente `@architect` |
| Tareas | `tasks/TASK-*.md` | agente `@planner` (comando `/tasks`) |
| Implementación | código + `tasks/TASK-*.md` actualizado | agente `@coder` (`/implement`) |
| Testing | `tests/` + código | agente `@tester` (`/test`) |
| Review | dictamen | agente `@reviewer` (`/review`) |
| Cambios | `changes/CHANGELOG.md`, `changes/CHG-*.md` | `sdd changelog` + agentes |

La matriz completa de agentes, modelos y comandos está en
[`AGENTS.md`](../AGENTS.md).

## 5. Comandos para el día a día

```bash
sdd status                  # estado real de los artefactos
sdd validate                # validación estricta (exit 1 ante errores)
sdd changelog               # historial de cambios con trazabilidad
sdd changelog -p ../otro    # operar sobre otro proyecto
```

`validate` distingue:

- **errores** (exit 1): manifest inválido, o `brief.md`,
  `architecture.md`, `CHANGELOG.md` o `config/project.md` faltantes/vacíos;
- **advertencias** (exit 0): `requirements/` o `tasks/` sin archivos `.md`.

> Los demás comandos (`brainstorm`, `requirements`, `tasks`, `context`,
> `implement`, `test`, `review`, `change`, `impact`, `hooks`) son stubs en
> v0.1.0: imprimen orientación pero no persisten. El detalle y el estado de
> cada uno está en la [referencia CLI](cli-reference.md).

## 6. Registro de cambios

Todo cambio relevante (requisito, arquitectura, diseño, fix) se registra en
`.sdd/changes/CHANGELOG.md` bajo `## [Unreleased]`:

```markdown
### [CHG-001] 2026-09-26 — Título del cambio

- **Motivo**: por qué cambia
- **Origen**: feature | change | fix
- **Artefactos afectados**: rutas
- **Impacto**: qué cambia
- **Tareas afectadas**: TASK-*
```

Los registros individuales con análisis de impacto viven en
`.sdd/changes/CHG-*.md`. `sdd changelog` los lista por pantalla.

## 7. Integración con agentes de IA

El harness se usa desde OpenCode con los agentes definidos en
`.opencode/agents/` y las skills en `.opencode/skills/`. El punto de entrada
es el comando `/orchestrate`. Ver:

- [Agentes e integraciones](agents-integrations.md)
- [Authoring de agentes](agent-authoring.md)
- [Authoring de skills](skill-authoring.md)

## 8. Solución de problemas

| Síntoma | Causa probable | Solución |
|---------|----------------|----------|
| `✗ No SDD project found` | no existe `.sdd/` | `sdd init` |
| `validate` exit 1 | artefacto requerido faltante/vacío | crear el archivo indicado |
| `status` muestra ✗ | directorio sin contenido | agregar archivos `.md` |
| Manifest inválido | `manifest.json` corrupto | restaurar desde Git |
