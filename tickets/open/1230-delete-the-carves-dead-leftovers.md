---
id: "1230"
title: "Delete the carve's dead leftovers: forwarding wrappers, the serde mirrors, unused loaders and re-exports"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1230 — Delete the carve's dead leftovers

## Context

From the game/engine boundary audit of 2026-11-03. The carve
(1102–1118) moved code into Shuyi Engine and left the game's files
re-exporting or forwarding to it, so call sites didn't change
(`docs/engine/boundary.md`). Some of those leftovers now have no user,
or repeat what the engine or `trpg-core` already does:

- **content:** `lang::pack_from_sources` (`lang.rs:134`) and
  `Lang::new` (`lang.rs:81`) have no caller; `dialogue::load`
  (`dialogue.rs:290`) is called only by two content tests
  (`lib.rs:1191`, `support.rs:235`). `dialogue::parse_dlg`,
  `voice::clip_file` and `voice::from_source` are one-line forwards to
  engine functions of the same signature. `keymap.rs` writes `Action`'s
  `name`, `from_name`, `is_repeatable`, `is_required`, `is_rebindable`
  and `group` inherently, then forwards each again in
  `impl ActionSet for Action` (`from_name` is the trait's default).
- **`enums.rs`** mirrors seven core enums for serde because "core has no
  serde dependency". It has (`core/Cargo.toml:18`), and six of the seven
  derive `Deserialize`; only `ArmourWeight` (`core/src/class.rs:75`)
  doesn't. Users: `class.rs`, `character.rs`, `item.rs`.
- **`ron_loader::load_ron`** (`ron_loader.rs:11`) has no caller outside
  its own two tests.
- **`lead::longest_len`** (`core/src/lead.rs:202`) is used only by its
  tests; the content checks use the engine's `longest_width` through the
  `Tokens::longest` hook.
- **ui:** `cinema.rs` (`pub mod cinema`, `lib.rs:23`) and the
  `map_view::corners` re-export have no user; `map_view::path` is used
  only by `map_view/playdate/tests.rs:17`; `screens/battle/camera.rs` is
  one `pub use` (users: `battle/mod.rs:83`, `battle/rewind.rs:18`,
  `map_view/place.rs:15`). Unused re-exported names: `fit_whole_scale`
  (`portrait.rs:6`), `tile_sprite` (`map_view/sprite.rs:44`),
  `MAX_FRAMES` (`harness.rs:41`), `ease_in_out` and `ShotAt`
  (`cinematic.rs:49`), `BLINK_PERIOD` (`battle/cursor.rs:13`).
  `ModeSelectScreen::menu()` (`mode_select.rs:70`) has no caller.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- Delete `pack_from_sources`, `Lang::new`, `dialogue::load` (its two
  tests call `load_scripts(..).map(|s| s.table)`), `load_ron` and its
  tests, `longest_len`, `cinema.rs`, `battle/camera.rs`, the `corners`
  and `path` re-exports, the unused names above and `menu()`.
- The three forwards become `pub use` of the engine's functions.
- `Action`'s five bodies move into `impl ActionSet`; callers import the
  trait. The enum and its `NAMES` table stay (engine ADR-0036 §9: no
  derive yet).
- `ArmourWeight` derives `Serialize, Deserialize`; the raw types of
  `class.rs`, `character.rs` and `item.rs` use the core enums; `enums.rs`
  and its tests go, with the `.into()` conversions.
- `docs/engine/boundary.md` rows 166, 181, 187 and 206, and
  `crates/content/README.md` and `crates/ui/README.md` where they name a
  deleted item.

**Out (do not do):**
- Dropping the `ron_loader`/`error` shim modules or the `terrain.rs`
  `line_of` re-export, or the facade files (`glyph_buffer.rs`,
  `widgets/menu.rs`, …): import churn that saves no lines.
- `dialogue::from_sources` and `lang::pack_from_files`: test helpers
  that other crates' tests call; they stay.
- `map::print_map` and the bots' `map_text` (leave both).
- `path::steer` and trimming the cursor's and steer's tests.
- `formats::load`'s not-found pre-check (an engine wording question).

**Either order with 1217** (the battle's cursor, camera and walks on the
kit's viewport): if 1217 is done, `battle/camera.rs` and `cursor.rs` may
be gone: delete what is left. If not, do it here; 1217 then imports
`shuyi_ui::grid::Camera` itself.
**Either order with 1233** (the boundary record): if 1233 is done, fix
only the rows this ticket makes wrong; if not, edit the four rows here.
**Either order with 1234** (one choice screen): if 1234 is done,
`mode_select.rs` is gone with `menu()`; if not, delete `menu()` here.
**Either order with 1044** (the Playdate title): if 1044 is done and
imports `crate::cinema`, point it at `shuyi_ui::frame::cinema`; if not,
1044 imports the engine's module.
**Either order with 1223** (content loaders on the engine's error
helpers): if 1223 is done, `class.rs`, `character.rs` and `item.rs`
fetch and report through the engine's helpers: change their raw types
there; if not, nothing to do (1223 works on the core enums).
**Either order with 1231** (content tests that copy the engine's): if
1231 is done, `dialogue::load`'s two tests may already be gone: delete
what is left; if not, switch them as above, and 1231 keeps
`lang::pack_from_files` and `dialogue::from_sources`, which stay here.

## Implementation steps

1. Re-grep each name across `crates/` (brace imports too) before
   deleting it; keep any that gained a user since 2026-11-03 and say so
   in the Completion notes.
2. content: the wrappers, the forwards, `keymap.rs`, then `enums.rs`.
   RON names are unchanged (no serde renames on the core enums); the
   content tests prove it.
3. core: `longest_len`. Keep `longest_len_bounds_substitution`'s check
   that no token is left; bound the length by summing `longest()` inline.
4. ui: the modules and names; `Camera` from `shuyi_ui::grid::Camera`.
5. Docs as in Scope. Measure with `cargo xtask code-lines`.

Expected size: about 95 production lines removed and 15 added; about 60
test lines removed.

## Acceptance criteria

- [ ] `grep -rn "pack_from_sources\|fn load_ron\|longest_len\|BLINK_PERIOD\|mod enums\|mod cinema\|mod camera" crates/`
      finds no game definition or use.
- [ ] `grep -rn "Action::name(self)" crates/content/src/keymap.rs` finds nothing.
- [ ] Every content test passes unchanged except the removed ones.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the existing content, core and ui tests; the reworked
  substitution proptest.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
