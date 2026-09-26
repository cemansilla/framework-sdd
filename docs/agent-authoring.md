# Guía de Authoring de Agentes

Cómo crear agentes para el harness SDD (runtime OpenCode).

## 1. Ubicación y registro

```text
.opencode/agents/
├── orchestrator.md    # agente primario (punto de entrada)
├── architect.md
├── planner.md
├── coder.md
├── tester.md
└── reviewer.md
```

- Un archivo por agente; se cargan automáticamente desde `.opencode/agents/`.
- `opencode.json` fija `default_agent: "orchestrator"`.
- Registrar el agente en la tabla **Matriz de Agentes** de
  [`AGENTS.md`](../AGENTS.md) (sub-comando, modelo, mode, rol).
- Si el agente participa de un workflow, crear un slash command en
  `.opencode/commands/` (`description` en frontmatter + instrucciones).

## 2. Frontmatter

```yaml
---
name: reviewer
description: Auditoría de seguridad, calidad de código y adherencia a la especificación.
model: opencode-go/glm-5.3-flash
temperature: 0.1
mode: subagent
permission:
  read:
    src/*: allow
    tests/*: allow
  edit: deny
  bash:
    cargo fmt --check: allow
    cargo clippy *: allow
    "*": deny
  task: deny
---
```

| Campo | Regla |
|-------|-------|
| `name` | identificador corto (`snake_case`); coincide con el `@nombre` |
| `description` | una frase de rol; guía al orquestador en el enrutado |
| `model` | proveedor/modelo asignado (ver §4) |
| `temperature` | baja (0.0–0.2) para tareas deterministas; `tester` = `0.0` |
| `mode` | `primary` (agente de entrada) o `subagent` (invocado vía `task`) |
| `permission` | mapa de permisos por herramienta — **principio de mínimo privilegio** |

### Permisos

- `read` / `glob` / `grep`: rutas con `allow` explícito.
- `edit`: `deny` para agentes que no escriben (orchestrator, reviewer).
- `bash`: lista explícita de comandos con wildcard; terminar con
  `"*": deny`. Ej.: reviewer solo `cargo fmt/clippy/test` y `git
  status/diff/log` de solo lectura.
- `task`: `deny` salvo el orquestador (único que invoca subagentes).

## 3. Cuerpo del agente

Estructura canónica (ver `.opencode/agents/reviewer.md`):

1. **Role** — quién es y qué hace.
2. **Input** — artefactos de entrada con rutas exactas.
3. **Responsabilidades** — lista numerada.
4. **Checklist** — casillas verificables agrupadas por categoría.
5. **Quality Gates** — comandos exactos a ejecutar.
6. **Output** — forma del resultado.
7. **Output Contract** — contrato JSON estricto del dictamen/entregable.

Reglas:

- todo camino de archivo es concreto (`.docs/02-…`, `.sdd/tasks/`);
- los gates son los del workspace: `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace`;
- el output contract impide respuestas ambiguas entre agentes.

## 4. Asignación de modelos (matriz actual)

| Agente | Sub-comando | Modelo por defecto (GRATIS) | Alternativa de pago (manual) | Mode |
|--------|-------------|-----------------------------|------------------------------|------|
| orchestrator | `/orchestrate` | `opencode/longcat-2.5-preview-free` | `opencode-go/qwen3.7-plus` | primary |
| architect | `/plan` | `opencode/nemotron-3-ultra-free` | `opencode-go/kimi-k3` | subagent |
| planner | `/tasks` | `opencode/ling-3.0-flash-fin-free` | `opencode-go/qwen3.7-plus` | subagent |
| coder | `/implement` | `opencode/big-pickle` | `opencode-go/kimi-k2.7-code` | subagent |
| tester | `/test` | `opencode/nemotron-3.5-lightning-free` | `opencode-go/deepseek-v4.1-flash` | subagent |
| reviewer | `/review` | `opencode/muse-spark-1.3-contributor-free` | `opencode-go/glm-5.3-flash` | subagent |

Criterio: orquestación → ventana de contexto grande (1M para mantener el
estado del ciclo + salidas de subagentes); arquitectura → rigor lógico +
contexto; planificación → estructuración y formato; implementación → modelo
validado en código real; verificación → modelos rápidos; review → contexto
amplio para auditar diff + spec.

### 4.1 Modelos gratuitos vs de pago

Los 8 modelos gratuitos disponibles (`opencode/*`, `$0/Mtok` en OpenCode Zen):

| Modelo | ctx | output | Rol recomendado |
|--------|-----|--------|-----------------|
| `longcat-2.5-preview-free` | 1.0M | 131K | orquestación, lectura de repo grande (**preview**) |
| `nemotron-3-ultra-free` | 1.0M | 128K | arquitectura y auditoría (mayor rigor) |
| `muse-spark-1.3-contributor-free` | 1.05M | 131K | review con contexto grande; ideación |
| `space-bunny-free` | 1.05M | 524K | respaldo de contexto/output grande |
| `ling-3.0-flash-fin-free` | 262K | 32K | planificación, estructuración de tablas/reglas |
| `nemotron-3.5-lightning-free` | 262K | 262K | QA rápido, linter-like |
| `big-pickle` | 200K | 32K | codificación, brainstorming |
| `mimo-v2.6-flash-free` | 200K | 32K | `small_model`: títulos/resúmenes/compactación |

Reglas:

1. **El sufijo `-free` no prueba gratuidad**: `deepseek-v4.1-flash` y
   `muse-spark-1.3` son de pago; `big-pickle` es gratis. Verificar con
   `opencode models -v` (costos) o models.dev.
2. **Defaults = gratis** en `.opencode/agents/*.md` y `opencode.json`;
   los de pago se eligen a mano con `/models` (nada está deshabilitado).
3. `model`/`small_model` en `opencode.json` cubren los built-in
   (`build`/`plan`) y las tareas ligeras (títulos, compactación).
4. Los modelos gratuitos pueden cambiar o retirarse (los `*-preview-*` con
   más frecuencia): documentar siempre un respaldo por rol (p. ej.
   `space-bunny-free` para contexto grande).
5. Los sustitutos gratuitos "de familia" (`qwen3.6-plus-free`,
   `kimi-k2.5-free`, `glm-5-free`, `deepseek-v4-flash-free`) existen en
   models.dev pero **no están habilitados** en esta instalación; si
   aparecen en `opencode models`, son los primeros candidatos por
   continuidad de familia.

**Brainstorming (rol sin agente propio):** la fase de ideación la ejecuta
el `orchestrator`; si se dispara a mano, usar `opencode/big-pickle`
(versatilidad, sin sesgo estructural) o
`opencode/muse-spark-1.3-contributor-free` (desglose conceptual).

## 5. Flujo entre agentes

```mermaid
flowchart LR
    O[orchestrator] -->|task| A[architect]
    O --> P[planner]
    O --> C[coder]
    O --> T[tester]
    O --> R[reviewer]
    A --> P --> C --> T --> R
```

- Solo el `orchestrator` tiene `task: allow` (invoca subagentes).
- El `coder` escribe; `orchestrator` y `reviewer` tienen `edit: deny`.
- Las skills cargadas por el agente (§6) refuerzan workflow y convenciones.

## 6. Skills vinculadas

| Agente | Skills habituales |
|--------|-------------------|
| todos | `sdd-task-workflow`, `branching-sdd` |
| coder | `cargo-workspace`, `conventional-commits`, `rust-quality` |
| tester / reviewer | `rust-quality`, `conventional-commits` |

Ver [guía de skills](skill-authoring.md).

## 7. Checklist de autoría

- [ ] frontmatter completo (`name`, `description`, `model`, `mode`, `permission`);
- [ ] mínimo privilegio: `edit: deny` si no escribe; bash con `"*": deny` final;
- [ ] input/output con rutas y contratos concretos;
- [ ] quality gates del workspace incluidos;
- [ ] matriculado en `AGENTS.md`;
- [ ] slash command asociado si tiene sub-comando;
- [ ] modelo asignado según la matriz y temperatura coherente con el rol.
