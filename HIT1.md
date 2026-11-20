# Hito 1 — Crear una app dentro de VoidXStudio

Documento de estado del **Hit 1**: la base de proyectos (`.vxs`), la pantalla Home con la acción "Create Project", el modal de creación y la vista World Space.

---

## Arquitectura (3 decisiones cerradas)

1. **Los nodos SON el IR.** No hay capa intermedia extra. Cada nodo/bloque de un grupo `Engine` o `Network` se modela en Rust como un AST semántico tipado (`LogicNode::OnClick`, `LogicNode::If`, `LogicNode::SetState`, `LogicNode::HttpRequest`). El motor de Live Preview recorre ese grafo para emitir **Lua** (`mlua`); el exportador final recorre el mismo grafo para emitir **Kotlin**. (Lua/Kotlin están pendientes; este Hito 1 solo construye la base.)

2. **Separación UI (Compose) vs Engine/Network (Kotlin).**
   - Grupos tipo `UI` → se compilan a **Jetpack Compose** (`@Composable fun NombreArchivoScreen(...)`), frames a `Box`/`Column`/`Row`, textos a `Text`, iconos/vectores exportados a `app/src/main/res/drawable/*.xml`.
   - Grupos tipo `Engine` / `Network` → se compilan a **clases o singletons puros en Kotlin** con estados reactivos (`mutableStateOf` / `StateFlow`) y funciones que las pantallas `@Composable` invocan desde el modo prototipo.
   - Los exportadores Lua y Kotlin **no se modifican en este hito**.

3. **Serialización `.vxs` = contenedor ZIP (tipo `.fig`).**
   - `project.json`: nombre de la app, `package_name` (mínimo 2 puntos, ej. `com.voidx.app`), lenguaje target (`Kotlin` activo; otros marcados `Coming Soon`), icono opcional, y layout/conexiones del **World Space**.
   - `files/<nombre>.figcraft`: documento visual de cada grupo tipo `UI` (formato `.figcraft` intacto, sin romper el canvas vectorial existente).
   - `files/<nombre>.logic.json`: grafo de nodos IR de cada grupo `Engine`/`Network` y enlaces de eventos desde los elementos UI.

---

## Qué ya existe (estado actual)

### Módulo `crates/ui-egui/src/project.rs` (nuevo)
- `ProjectDraft`: borrador que edita el modal Create Project (`name`, `package_name`, `language`, `icon`).
- `VoidXProject`: proyecto creado (`name`, `package_name`, `language`, `icon`, `world`, `files`).
- `FileGroup`: grupo de archivo (`name`, `kind: Ui | Engine | Network`, `visual: Option<String>`, `logic: Option<Value>`).
- `WorldConnection` (`from`, `to`) y `World` (`connections: Vec<WorldConnection>`).
- `ProjectLanguage` (Kotlin default; Flutter, Dart, Slint como `Coming Soon`).
- `FileGroupKind` (`Ui`, `Engine`, `Network`) con `.label()`.
- `package_name_ok(package_name)`: valida al menos 2 puntos (`com.ejemplo.app`).
- `save(path, project)` y `load(path)`: escribe/lee `.vxs` como ZIP con `project.json` + `files/<name>.logic.json` (crate `zip`, `SimpleFileOptions` + `Deflated`).

### Pantalla Home (`crates/ui-egui/src/home.rs`)
- **"Create Project" action card** en `recents()` (`Icon::Layers`, título "Create Project", sub "A .vxs project for an app").
- Al tocarla, se abre `app.ui.project_dialog = Some(ProjectDraft::default())`.
- `create_project_dialog()`: modal con:
  - Selector de lenguaje (Kotlin por defecto; Flutter/Dart/Slint visibles como *Coming Soon*).
  - Inputs **App Name** y **Package Name** con validación en tiempo real (≥ 2 puntos); botón **Create** desactivado hasta que sea válido.
  - Input opcional de ícono.
  - Botones **Create** (guarda `.vxs`, abre el World Space) y **Cancel**.
- **World Space** (`world_space()`): muestra los grupos del proyecto como bloques clickeables (nombre + label del kind). Click en un grupo `Ui` abre el editor actual (`file.new`, `home = false`).
- El World Space se activa cuando `app.ui.project` es `Some`; el modal cuando `app.ui.project_dialog` es `Some`.
- Duplicate `world_space` eliminado (queda una sola función).

### Estado (`crates/ui-egui/src/lib.rs`)
- `UiState.project_dialog: Option<ProjectDraft>` y `UiState.project: Option<VoidXProject>`, inicializados a `None`.
- `home: true` (la app arranca directamente en la pantalla Home).
- `pub mod project;` registrado.

### Servicios y rutas (`crates/ui-egui/src/services.rs`, `apps/figcraft-android/src/browser.rs`)
- Nuevo método `Services::documents_dir()` → directorio de documentos de la app (`$HOME` en desktop).
- En Android: `permiso::directorio_privado()` → `/data/data/io.VoidXStudio.app/documents`.
- `create_project_dialog` guarda el `.vxs` en `documents_dir()` (no en `$HOME`, que falla silenciosamente en Android).

### Formato abierto (`crates/ui-egui/src/open.rs`)
- `OPEN_EXTENSIONS` incluye ahora `"vxs"` para que el diálogo nativo Abierto pueda abrir proyectos `.vxs`.

### Dependencias
- `crates/ui-egui/Cargo.toml`: `zip = { workspace = true }`.
- `Cargo.toml` (workspace): `zip = { version = "2", default-features = false, features = ["deflate"] }` (ya existía).
- `Cargo.lock` actualizado.

### Commits en `release-prep`
- `bf167dc8`: rename app/package a VoidXStudio / io.VoidXStudio.app, icono, About, CI verde.
- `d6ba17d5`: H1 base — módulo `project`, Create Project modal, World Space.
- `df11596f`: H1 fixes — arranque en Home, modal no parpadea (fixed `Id` + `Order::Foreground` + render vía `ctx`), ruta de guardado Android, `.vxs` en Open.

---

## Qué todavía NO está hecho (pendiente)

### Inmediato (cerrar el Hit 1)
1. **Probar en el dispositivo** (`192.168.100.51:41167`, app `io.VoidXStudio.app`):
   - App arranca en Home.
   - Tarjeta "Create Project" abre el modal y se queda abierto.
   - Crear proyecto → guarda `.vxs` en `/data/data/io.VoidXStudio.app/documents/` → abre World Space.
   - Click en bloque `Main` (UI) → abre el editor actual.
2. **CI verde** para el commit actual (`df11596f`): runs `build-apk-release` y `build-android-so`.

### Después del Hit 1 (no tocar todavía, per instrucciones del usuario)
- Exportador Lua (Live Preview recorre el grafo de nodos IR).
- Exportador Kotlin (grupos `Engine`/`Network` → clases/singletons Kotlin).
- Pantalla de editar grupo: abrir el `.figcraft` del grupo `Ui` en el editor de FigCraft.
- Modal para crear nuevos grupos (`UI`, `Engine`, `Network`) dentro del World Space.
- Puertos de conexión entre bloques del World Space (`WorldConnection`).
- Selector de icono real (ahora es solo un input de texto opcional).
- Validación de nombre de archivo único al guardar `.vxs`.
- Tests del módulo `project` (save/load round-trip).

### Pruebles pendientes en el dispositivo (app renombrada `io.VoidXStudio.app`)
- Añadir nodo, lupa, long-press multiselección, copy/paste.
- Verificación de About e ícono.

---

## Cómo se ejecuta localmente

Gate obligatorio antes de cada commit:

```bash
cargo check -p figcraft-ui-egui --all-targets
```

El repositorio no permite `cargo xtask ci` localmente (solo `cargo check`); el CI de GitHub corre la suite completa (~5.078 tests, ~16 min por ronda).

Reglas de la casa (`AGENTS.md`): no `unwrap`/`expect`/`panic` en código de producción; todo es un comando; assets en `ATTRIBUTION.md` (no se agregaron assets nuevos en este hito, solo código); `main` nunca se rompe (los cambios van directo a `main`/`release-prep` de este port, ramas+PR solo para merges/releases upstream).
