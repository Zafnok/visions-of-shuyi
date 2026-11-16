# ADR-0124: The debug build reads its folders from disk under `run --watch`, and reloads palettes and scenes

- **Status:** Accepted
- **Date:** 2026-11-14
- **Related tickets:** 1209; engine#2220 (hot reload, engine ADR-0071),
  engine#2242 (pictures), 0764 (the writer's preview, which watches its
  folder in every build)
- **Extends:** ADR-0005 and ADR-0105 (every layer an embedded table),
  ADR-0057, ADR-0117

## Context

Engine ADR-0071 gives hot reload: the platform polls the folders a game read
with `read_folder`, puts a changed file's new bytes in its layer and tells
the game's modules (`Module::reloaded`); `cargo shuyi run --watch` starts a
debug build with `SHUYI_WATCH` naming the game's folder (`shuyi.ron`'s
`assets`). A game with its own `main` lays that folder over its files
itself.

This game reads nothing from disk: `build.rs` embeds `assets/`, the
content (`assets-private/content/`, laid over it) and the test files
(`assets-test/`) as tables, without their READMEs (ADR-0105) or the
content's `tests/` (ADR-0118), and the content is loaded once into
`ctx.game.content`. A change to a scene or a palette needed a rebuild.

- Laying the watched `assets/` over everything would hide the content's
  files at the same paths (the content lies over `assets/`), and the
  engine's own `watched_layer()` would also make the backend draw
  `assets/`'s placeholder pictures over the bought art.
- `read_folder` keeps every file of the folder. `lang/README.md` is then a
  file no language has, and the content fails to load.
- Engine ADR-0040 §3: a module's hook changes its own state and the
  context's UI side, never a stage's state, which changes only through
  commands (this game's ADR-0004).

## Decision

1. **Under `run --watch`, in a build with debug tools, the app reads the
   game's folders in place of their tables.** `SHUYI_WATCH` names
   `assets/`; its parent is the repository's root, and each layer of
   `bundle::layers()` but the bought art is read from the folder at its
   shown name (`assets`, `assets-test`, `assets-private/content`) with
   `shuyi_app::read_folder`, before anything reads a file
   (`bundle::read_from_disk`). Each takes its table's place in the order,
   private and test as it was. The bought art stays embedded: pictures
   don't reload yet. Without `SHUYI_WATCH`, or without debug tools, nothing
   is read from disk and the build is what it was.
2. **The game reads a folder as a build embeds it.** `bundle::Folder` keeps
   the layer the platform watches (every file) and a second one of the
   files `build.rs` would embed, which the bundle reads. A change is copied
   from the first to the second (`bundle::take_changes`) before anything
   reloads. This is a workaround: `read_folder` has no way to leave files
   out.
3. **A module reloads the palettes and the scenes.** `trpg_ui::hot`'s
   `HotContent` (installed by `start::start` in a build with debug tools)
   loads the whole content again when a theme's palette or a `dialogue/`
   `.dlg` file changed, so a scene goes through every check it has at
   start, and takes from it the palettes (the one in use repainted from the
   next frame) and the dialogue table, which the game looks a scene up in
   each time it plays one. When the load fails the old ones stay, and its
   errors (file and line) are noted under `content` and drawn over the
   frame until a load succeeds.
4. **Rules data never changes under a running game.** Maps, battles,
   chapters, classes, items and the rest keep what the game started with:
   a battle's state changes only by `Command` (ADR-0004), and its record
   must replay. Swapping the palettes and the dialogue is outside ADR-0040
   §3's letter (they are the game's data, not the module's): they are
   files the game reads, not state a command changes, and no record holds
   them.

## Consequences

- A writer or designer edits a scene or a palette in `assets/`,
  `assets-test/` or the content, and the running debug build shows it on
  the next frame, or the next time the scene plays. A scene already on
  screen keeps the copy it opened with.
- A whole content load per save costs a fraction of a second; only a
  palette or scene change does it.
- The disk layers leak each version of a changed file (engine ADR-0071),
  twice here (the watched layer and the game's view): a debug build only.
- Maps and battle files would reload at most at the next battle, by a
  later ticket; pictures after engine#2242 is adopted.
- A `read_folder` that leaves files out would remove decision 2's copy.

## Alternatives considered

- **`watched_layer()` laid over the whole bundle.** Hides the content and
  draws placeholder pictures over the bought art (Context).
- **Skipping READMEs in the loaders.** ADR-0105 took that knowledge out of
  them; it would come back in every loader that lists a folder.
- **`Hot<PaletteDef>` per theme and a dialogue-only reload.** Lighter, but
  skips the checks across files (a theme's names against the default's, a
  battle naming a scene), which a full load keeps, at a cost too small to
  matter.
- **Reading the folders in every debug build.** `cargo run` would then
  depend on the working copy's files; `run --watch` asks for it.
