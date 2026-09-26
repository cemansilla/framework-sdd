# Guía de Authoring de Skills

Cómo crear y registrar skills para el harness SDD.

## 1. Qué es una skill

Una skill es un documento Markdown con instrucciones especializadas que el
agente carga bajo demanda cuando la tarea coincide con su descripción.
No es código: es conocimiento operativo reutilizable (convenciones,
workflows, checklists).

## 2. Estructura de archivos

```text
.opencode/skills/
└── <nombre-skill>/
    └── SKILL.md
```

- Un directorio por skill; el nombre del directorio = nombre de la skill.
- `opencode.json` expone el registro con
  `"skills": { "paths": [".opencode/skills"] }`.

## 3. Formato del archivo

### Frontmatter (obligatorio)

```yaml
---
name: nombre-skill
description: Qué hace y CUÁNDO usarlo. Usar cuando se necesite <triggers>.
---
```

| Campo | Regla |
|-------|-------|
| `name` | idéntico al nombre del directorio; kebab-case |
| `description` | una frase de propósito + frases de activación tipo *"Usar cuando …"* — el agente decide la carga por esta línea |

### Cuerpo

Markdown libre con la convención del proyecto:

- tablas para referencias (tipos, scopes, ejemplos);
- bloques de código con lenguaje explícito;
- ejemplos *correcto / incorrecto*;
- numerar reglas duras; mantener el texto accionable y conciso.

## 4. Plantilla

````markdown
---
name: mi-skill
description: Propósito de la skill. Usar cuando se necesite X o Y.
---

# Título

## Reglas

1. Regla dura y verificable.
2. Otra regla.

## Referencia

| Caso | Acción |
|------|--------|
| a    | hacer b |

## Ejemplos

```bash
comando correcto
```
````

## 5. Registro y descubrimiento

1. Crear `.opencode/skills/<nombre>/SKILL.md`.
2. Verificar que `opencode.json` incluye `.opencode/skills` en `skills.paths`
   (ya está para skills del repo).
3. Registrar la skill en la tabla **Skills Registry** de
   [`AGENTS.md`](../AGENTS.md) (propósito + ubicación).
4. Si la skill guía un workflow con quality gates, referenciar también
   `.opencode/skills/rust-quality`.

## 6. Skills existentes (registro)

| Skill | Propósito |
|-------|-----------|
| `cargo-workspace` | inicializar/gerenciar workspace Cargo |
| `sdd-task-workflow` | ciclo completo de tarea: branch → implement → test → review → commit → PR |
| `conventional-commits` | formato y agrupación de commits |
| `branching-sdd` | branching y flujo de PR (nunca merge directo) |
| `rust-quality` | quality gates: fmt, clippy, test |

## 7. Checklist de autoría

- [ ] `name` == nombre del directorio;
- [ ] `description` incluye *"Usar cuando …"* (activadores claros);
- [ ] sin ambigüedad con otras skills (si compiten, afinar triggers);
- [ ] reglas verificables (comandos exactos, rutas exactas);
- [ ] ejemplos ejecutables/válidos;
- [ ] registrada en `AGENTS.md`;
- [ ] lenguaje consistente con las skills existentes (español).
