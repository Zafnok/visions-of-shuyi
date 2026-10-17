# ADR-0078: Input actions are an open set the game defines; layouts are named data

- **Status:** Accepted
- **Date:** 2026-10-16
- **Related tickets:** 1106, 1107, 1112
- **Amends:** ADR-0015 (*Layouts are data*: the keymap file now names its
  layouts) and ADR-0031 (*Fixed keys live in `Keymap`, not in data*: which
  action a fixed key triggers is now the game's data). Their saved formats
  are unchanged.

## Context

The input mechanism (the keymap file, `Keymap`, held keys and repeat,
`FrameInput`, the player's saved bindings, help text) was built on this
game's closed `Action` enum, and `Layout` was a closed two-variant enum.
That is the biggest coupling between the game and the engine being carved
out of it (`docs/engine/boundary.md`, coupling 1; ADR-0063). Ticket 1106
makes the mechanism generic in place, so that 1107 can move it to the
engine without changing behaviour: rebinding, help text by keymap and the
saved bindings (version 2, ADR-0053) must keep working exactly.

## Decision

1. **`ActionSet`** (`trpg-content`'s `keymap::action_set`): a game's action
   enum implements `ActionSet: Copy + Ord + Eq + Hash + Debug + 'static`
   with `ALL`, `name`, `from_name`, `is_repeatable`, `is_required`,
   `is_rebindable`, an associated `Group` type and `group` (ADR-0068's key
   groups: the engine knows actions come in groups, the game names them),
   `fixed_keys()` and `ui()`. This game's `Action` keeps its inherent
   methods; its `ActionSet` impl delegates to them.
2. **`UiAction`** `{ Up, Down, Left, Right, Confirm, Cancel, Debug }`: what
   the generic parts react to. `ActionSet::ui` maps a game's action onto
   one (this game: the four cursor moves, Confirm, Cancel, Debug).
   `Menu::handle` and the Key bindings screen match on `action.ui()`;
   `Game` opens the debug menu on `UiAction::Debug`; the cursor's help
   name (`cursor_keys_name`, `cursor_buttons_name`) uses the actions mapped
   to the four moves.
3. **Generic types**, each over `A: ActionSet`: `KeymapDef<A>` (the file
   format, `keymap::format`), `Keymap<A>` (`input::keymap`), `InputState<A>`
   (`input::state`), `FrameInput<A>` (`input::frame`), `LayoutBindings<A>`,
   `PadBindings<A>`, `PlayerKeys<A>`, `BindError<A>` (`input::bindings`),
   `HelpKeys<'a, A>` and the name helpers (`input::help_keys`). The game
   gets aliases with its `Action` at the old paths (`trpg_ui::input::Keymap`,
   `trpg_ui::screen::FrameInput`, `trpg_ui::widgets::help::HelpKeys`, …), so
   screens are unchanged. These modules name none of this game's actions:
   1107 moves them.
4. **Fixed keys are data.** `ActionSet::fixed_keys()` lists the chords every
   keymap adds on top of its bindings (this game: plain `Escape` →
   `Cancel`). Plain `Escape` and `Delete` stay reserved by the mechanism
   (`Chord::is_reserved`): the Key bindings screen backs out of a capture
   with one and empties a slot with the other (`input::fixed`).
5. **The battle's own key rules move to the battle**:
   `select_action` and `end_turn_accept_actions` (*Optional split keys*)
   are `screens::battle::split_keys`, not `Keymap` methods.
6. **Layouts are named data.** `keymap.ron`'s `layouts` is a list of
   `(id, name, keys)`: `id` is the name saved settings and the saved
   bindings know it by, `name` the text key the layout picker shows. The
   picker lists the file's layouts in file order. `Layout` is that id, kept
   inline (`Copy`, at most 24 ASCII letters, digits or `_`), ordered by
   name; the game's constants are `RIGHT_HANDED` and `LEFT_HANDED`. A saved
   layout the keymap doesn't list is ignored (the player is asked again),
   as an unknown one was before. The loader refuses a file with no layout
   or a repeated id.
7. **The Key bindings screen takes its rows from the game**:
   `KeyBindingsScreen::with_rows(ctx, rows)` with rows `(action, text key,
   RowGroup)`, where `RowGroup` is `Required`, `Optional` or `Shared(heading
   text key)` (a key group of its own). `new(ctx)` uses the game's
   `input::KEY_ROWS`.

## Consequences

- A second game implements `ActionSet` for its own enum and writes its own
  keymap file; nothing in the mechanism changes. A test does it with a
  three-action set (`input/toy_tests.rs`).
- The saved bindings and settings keep the same names and bytes: a file
  saved by the build before this change loads and saves back identically
  (`input/bindings/saved_before_1106.ron`).
- Adding or renaming a layout is a `keymap.ron` edit plus a language key;
  renaming an `id` loses players' saved choice of it, so ids are never
  renamed.
- The Key bindings and layout picker screens still use this game's `Ctx`;
  making them generic screens is 1112's.

## Alternatives considered

- **Default type parameters** (`Keymap<A = Action>`) instead of aliases —
  the default has to name the game's type in the crate that defines the
  generic one, which can't work once 1107 moves it to the engine.
- **`Layout` as an index into the file's list** — saved files name layouts,
  so every load would need the keymap to turn a name into an index, and a
  reordered file would change what a stored index meant.
- **Leaking the file's layout names to get `&'static str`** — leaks on
  every load (tests load the keymap many times) for no gain over an inline
  name.
- **Keeping `Esc → Cancel` in `Keymap` and mapping it through
  `UiAction::Cancel`** — works for this game, but a game whose Cancel is
  not the action `Escape` should trigger (or that wants another fixed key)
  couldn't say so.
