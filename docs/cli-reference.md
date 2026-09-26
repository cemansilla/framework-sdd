# Referencia CLI — `sdd`

Binario del workspace (`crates/sdd-cli`), versión `0.1.0`.
Ayuda: `sdd --help` / `sdd <comando> --help`.

## 1. Comportamiento global

- **Ruta del proyecto:** cada subcomando acepta `-p, --path <PATH>`
  (defecto: directorio actual). Es una opción *por subcomando*, no global:
  usar `sdd status -p .`, no `sdd -p . status`.
- **Guard de proyecto:** si no existe `.sdd/`, todos los comandos imprimen
  `✗ No SDD project found` (exit 0), excepto `validate` (exit 1) e `init`
  (siempre opera).
- **Logging:** `tracing` escribe a stderr; `RUST_LOG` respetado (nivel
  defecto INFO). La salida de usuario va a stdout.

### Códigos de salida

| Código | Causa |
|--------|-------|
| `0` | éxito (incluye mensajes de "no encontrado" de los stubs) |
| `1` | error de ejecución: `validate` con errores; fallos de I/O o manifest en `init`/`status`/`changelog` |
| `2` | error de uso de clap (flag desconocido, argumento requerido ausente) |

## 2. Comandos con I/O real

| Comando | Flags | Qué hace |
|---------|-------|----------|
| `init` | `-p <PATH>` | Crea la estructura `.sdd/` (15 directorios), plantillas y `manifest.json`. **Idempotente**: nunca sobrescribe. `project_id` = nombre del directorio. |
| `status` | `-p <PATH>` | Lee `manifest.json` (error de parse → exit 1) y muestra ruta/project_id/versión + estado de brief, requirements, architecture y tasks (`✓`/`✗`). |
| `validate` | `-p <PATH>` | Validación estricta. **Errores → exit 1**: manifest inválido, o vacíos/faltantes `brief/brief.md`, `architecture/architecture.md`, `changes/CHANGELOG.md`, `config/project.md`. **Advertencias → exit 0**: `requirements/` o `tasks/` sin `.md`. |
| `changelog` | `-p <PATH>` | Lee `.sdd/changes/CHANGELOG.md`, parsea entradas `## [ID]` y anidadas `### [ID]` con campos `- **Key**: value`, e imprime `[ID] fecha — título` con sus campos. Archivo vacío → "No changes recorded yet." |

## 3. Comandos stub (solo lectura, salida orientativa)

Verifican la existencia de `.sdd/` e imprimen texto de ejemplo; **no leen
ni escriben artefactos**; siempre exit 0.

| Comando | Flags / args | Salida |
|---------|--------------|--------|
| `brainstorm` | `-p`, `-t <TITLE>` | Crea sesión en memoria e imprime banner/ID (nada persistido). |
| `tasks` | `-p`, `-l/--list`, `[TASK_ID]` | `No tasks found.` o `Task not found: <id>`. |
| `context` | `-p`, `<TASK_ID>` (req.) | Siempre `Task not found: <id>`. |
| `implement` | `-p`, `<TASK_ID>` (req.) | Siempre `Task not found: <id>`. |
| `test` | `-p`, `<TASK_ID>` (req.) | Siempre `Task not found: <id>`. |
| `review` | `-p`, `<TASK_ID>` (req.) | Siempre `Task not found: <id>`. |
| `design` | `-p`, `-s/--show` | `No technical design defined yet.` + checklist. |
| `change` | `-p`, `[CHANGE_ID]` | `Change not found: <id>` o texto de uso. |
| `impact` | `-p`, `<CHANGE_ID>` (req.) | Siempre `Change not found: <id>`. |

## 4. Comandos stub de escritura

Imprimen confirmaciones ("added", "installed", "defined") pero **no
persisten nada** en disco.

| Comando | Flags / args | Salida |
|---------|--------------|--------|
| `requirements` | `-p`, `-l/--list`, `-a/--add <TXT>` | `--list` → `No requirements found.`; `--add` → imprime `REQ-001` en memoria. Sin flag → uso. |
| `questions` | `-p`, `-l`, `-a <TXT>` | Igual patrón que `requirements`. |
| `architecture` | `-p`, `--show` (solo largo), `-s/--style <STYLE>` | `--show` → `No architecture defined yet.`; `--style` acepta `hexagonal\|layered\|microservices\|event-driven` y confirma en memoria; estilo desconocido → lista opciones (exit 0). |
| `hooks` | `-p`, `-l`, `-i/--install` | `--install` imprime "Hooks installed successfully" **sin instalar nada**; `--list` → `No hooks installed.` |

## 5. Ejemplos

```bash
sdd init                          # inicializar en el cwd
sdd init -p ../otro-proyecto      # inicializar en otra ruta
sdd status                        # estado de artefactos
sdd validate; echo $?             # 0 ok · 1 con errores
sdd changelog                     # historial de cambios
sdd requirements --add "Debe X"   # stub: no persiste (v0.1.0)
```

## 6. Estado en v0.1.0

Funcionales con disco: `init`, `status`, `validate`, `changelog`.
El resto son stubs planificados; su implementación progresiva se trackea
en `.docs/03-development-tasks.md`.
