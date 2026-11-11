---
id: "1253"
title: The Playdate build draws with a pattern map instead of inks
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["engine#2388"]
nick_input: none
completed: 2026-11-11
---

# 1253 — The Playdate build draws with a pattern map instead of inks

## Context

The Playdate build draws its one-bit frames with the engine's **inks**
(ADR-0074): `assets/data/palettes/playdate.ron` names an ink for each
colour role, each ink is a reserved RGB code, and
`shuyi_ui::frame::onebit::render`, `render_marked` and `render_changed`
read the codes back as inks (`InkShades`).

The engine has a second, generic way to say the same thing: a **pattern
map** beside the palette (engine ADR-0026, `shuyi_assets::onebit`,
`PatternMap`), drawn by `render_with`, `render_marked_with` and
`render_changed_with`. Engine ticket 2388
([engine ADR-0089](https://github.com/Zafnok/shuyi-engine/blob/main/docs/adr/0089-one-bit-roles-draw-everything-the-inks-do-a-dots-pattern-and-a-roles-edge-and-invert.md))
gave pattern maps the three things only inks could do (a `Dots`
pattern, a role's `edge` and its `invert`) and proved, over random
frames, that this map draws every ink-coloured frame exactly as the
inks do:

| Ink | A role's patterns |
| --- | ----------------- |
| `white` | `(fg: White)` |
| `black` | `(fg: Black)` |
| `black_edged` | `(fg: Black, edge: true)` |
| `grey` | `(fg: Dither50)` |
| `dots` | `(fg: Dots)` |
| `diagonals` | `(fg: DiagonalDown)` |
| `dashes` | `(fg: Checker2)` |
| `invert` | `(fg: Black, invert: true)` |

Engine ticket 2389 then removes the inks (`Ink`, `InkShades`, `ink_of`,
`render`, `render_marked`, `render_changed`) from the engine, and waits
for this ticket. The engine keeps nothing of the game: the inks are
named for this game's uses, which the engine's rules keep out.

## Nick input

None. The Playdate's screens must look exactly as they do now: the
one-bit snapshots and the bench's frames must not change.

## Scope

**In:**
- Move the engine pin past engine#2388 (its PR's merge commit), with
  the migration below (`RolePatterns` is `#[non_exhaustive]` there).
- A pattern map for the Playdate's palette,
  `assets/data/palettes/playdate@onebit.ron`
  (`shuyi_assets::onebit::map_path`): each role of `playdate.ron` gets
  its ink's patterns from the table above (`"cursor": (fg: Black,
  invert: true)`, `"move_range": (fg: Dots)`, …), and the `distinct`
  groups that `crates/xtask/src/game_tools/one_bit.rs` holds today
  (`DISTINCT`).
- The Playdate build loads it beside its palette (`trpg_content::ink`,
  `load_playdate`, or a loader next to it), builds a
  `shuyi_ui::frame::onebit::PatternMap` from the palette and the map
  once, and draws with `render_with`, `render_marked_with` and
  `render_changed_with` and that map in `crates/playdate/src/pd/entry.rs`
  and `crates/playdate/src/pd/bench.rs` (and the reference renderer's
  `_with` functions where the bench uses `reference::render_marked`).
  The start card (`crates/playdate/src/card.rs`, drawn by `render`)
  draws with the same map.
- `check --target playdate` (`crates/xtask/src/game_tools/one_bit.rs`)
  reads the file instead of deriving a map from the inks by "the
  nearest pattern" (`pattern`, `map_of`), so the check sees what the
  build draws.

**Out (do not do):**
- Changing the palette's colours. They stay the inks' codes, so every
  screen and skin draws unchanged; a later ticket may give them real
  RGB values once nothing reads them as inks.
- Text roles in the map (`text: [...]`): roles of one colour must share
  their patterns (`Rule::SharedColour`), and `text` shares the black
  ink's code with roles drawn black on black. Leave `text` empty, as
  today, and keep `the_text_roles_are_solid_black`.
- Removing the inks from the engine: engine ticket 2389.
- Removing the game's own uses of `Ink` (`trpg_content::ink`, the map
  skin's tests that name inks): they stay until engine 2389 removes the
  type, and that pin bump's ticket moves them.

## Implementation steps

1. Move `shuyi-engine.rev` past engine#2388. In
   `crates/xtask/src/game_tools/one_bit.rs`, `RolePatterns { fg, bg:
   None }` becomes `RolePatterns::of(fg)` (until step 4 removes it).
2. Write `assets/data/palettes/playdate@onebit.ron`: every role of
   `playdate.ron`, its ink's patterns from the table, and `distinct`.
   Add a content test that every role of `playdate.ron` has the
   patterns of its ink (read the ink from the colour code with
   `Ink::from_code`, compare with the table), so the two files can't
   drift.
3. Load the map with the palette (`PatternMapRon`, in the bundle like
   the palette), build the `PatternMap` once where the Playdate build
   keeps its font and pictures, and switch every `onebit::render*` call
   of `crates/playdate` to its `_with` form with that map.
4. In `one_bit.rs`, build the `OneBitSkin` from the file's map; delete
   `pattern` and `map_of`, and the comment that says the game draws with
   inks.
5. Run the one-bit snapshots, the bench's frames and `cargo shuyi check
   --target playdate`.

## Acceptance criteria

- [x] `assets/data/palettes/playdate@onebit.ron` gives every role of
      the Playdate's palette its ink's patterns (a content test).
- [x] No call in `crates/playdate` draws with `InkShades`: `rg
      "onebit::render\(|render_marked\(|render_changed\(" crates/playdate`
      finds only `_with` calls.
- [x] Every one-bit snapshot and bench frame is unchanged (no snapshot
      file in the diff).
- [x] `cargo shuyi check --target playdate` reads the map file and
      reports no new problem.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the map gives each role its ink's patterns.
- Property: none (engine 2388 proves the map draws as the inks do).
- Snapshot / integration: the existing one-bit snapshots, unchanged.

## Completion notes

- **Pin:** already past engine#2388 (`edb6c888`, moved by the pin bump
  #624 and after), so step 1 was only the `RolePatterns::of` migration,
  which `main` had too; nothing to move.
- **The map:** `assets/data/palettes/playdate@onebit.ron`, every role of
  `playdate.ron` with its ink's patterns from the table, the palette's
  comments kept, and the two `distinct` groups. `trpg_content::ink`
  loads it (`load_playdate_map`, `PLAYDATE_MAP_PATH`); the content test
  `the_pattern_map_gives_each_role_its_inks_patterns` checks every role
  against its ink, that the map names no other role, that
  `onebit::check` finds nothing, and that the path is `map_path`'s.
- **Drawing:** `trpg_ui::onebit::playdate_map()` builds the
  `PatternMap` from the palette file and the map. The front end builds it
  once with the loading card's font (the card draws with it) and keeps it
  beside the font; the frame loop draws with `render_with` /
  `render_changed_with`, the bench with `render_marked_with` and
  `reference::render_marked_with`. It reads the palette without the
  default theme's names (the card comes before the rest of the data);
  `start.rs` still checks the palette has them when the game loads it.
  The Chapter 1 bundle test reads the map with the card's font, as the
  device does, so the bundle holds no file it never reads.
- **Check and tools:** `check --target playdate` reads the file
  (`one_bit.rs` lost `pattern` and `map_of`); the tools' one-bit pictures
  (`frame-png --playdate`, the bench's Playdate session) draw with the
  same map.
- **Same frames:** no snapshot file changed (the one-bit snapshots are
  glyph-buffer text). As a throwaway check, not committed, every one-bit
  pixel test of `trpg-ui` (2,200+) ran with the map instead of the inks,
  and the Playdate walk-through test (title, menus, a typing scene, the
  battle, the cursor: 60+ frames) compared the map's frame with the
  inks' byte for byte: all the same with the Playdate's skins. Without
  them (the PC's dialogue skin on the Playdate palette, which no build
  does) a darkened place is a 40% grey that is no ink: the inks draw it
  black, the map dithers it. Engine 2389's pin bump should expect that
  if a test draws the PC skins in one bit.
- **Deviation:** `tickets/open/1262-…` was left behind on `main` beside
  `tickets/done/1262-…` (#626 and #627 merged in that order), which
  fails `ticket-lint` on every branch. This PR deletes the stale open
  copy, in its own commit.
- No gameplay rule decided; nothing for the Act 1 playtest.
