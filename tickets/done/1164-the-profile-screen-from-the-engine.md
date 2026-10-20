---
id: "1164"
title: "The profile screen and the name entry widget from the engine: the game keeps an opener and its one-bit skins"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: done
blocked_by: ["engine#2310"]
nick_input: none
completed: 2026-10-19
---

# 1164 — The profile screen from the engine

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`)
measured the profile screen (`crates/ui/src/screens/profile*`) at 831
production lines against an estimate of 100 (8.3×): it names nothing of
the game, so it belongs in the engine beside options and Key bindings.
Engine ticket 2310 (`Zafnok/shuyi-engine`) moves it and the name entry
widget (`crates/ui/src/widgets/name_entry*`) into the engine, painted by
a painter the game can replace.

## Nick input

None.

## Scope

**In:**
- Move the pin past 2310 (`engine-change` skill, step 7).
- `screens/profile.rs` becomes a short opener over the engine's screen,
  like `screens/key_bindings.rs` (27 lines) over the engine's Key
  bindings; its view and glyph skin go.
- `widgets/name_entry.rs` and its glyph skin go; the lead's name box
  (`lead_select`) uses the engine's widget.
- The one-bit skins (`profile/playdate.rs`, `name_entry/playdate.rs`)
  stay, painting the engine's views, passed with `painted_by` (or the
  engine's equivalent) on the Playdate.
- The game's text keys the engine now owns move out of the game's text
  tables, unless the game changes their wording on purpose.

**Out (do not do):**
- Any change the player sees on the PC or the Playdate.

## Implementation steps

1. Write the estimate: about 700 lines removed, 40 added.
2. Move the pin; open the engine's screen from where the game opens
   its own today.
3. Delete the game's copies; the one-bit skins on the engine's views.

## Acceptance criteria

- [x] `crates/ui/src/screens/profile.rs` is under 60 production lines
      (25) and `crates/ui/src/widgets/name_entry.rs` is gone.
- [x] The profile and name entry snapshots, PC and Playdate, are
      unchanged.
- [x] `cargo xtask code-lines` shows `trpg-ui` fell by at least 600
      (116 added, 1093 removed: 977 down).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the existing ones (the engine now holds the screen's unit
  tests).

## Completion notes

- **Done:** the pin is at engine `main` 939f023 (2310).
  `screens/profile.rs` is a 25-line opener: `profile::screen(ctx)` is the
  engine's `ProfilesScreen`, opening the first-launch layout picker after
  an add (`opening_after_add`) and painted through `SKINS`
  (`painted_by`). Its view, glyph skin and unit tests are gone (the
  engine holds them); its three snapshots stay, unchanged.
  `widgets/name_entry.rs` and its glyph skin are gone: `widgets::name_entry`
  re-exports the engine's, with the game's one-bit skin
  (`widgets/name_entry/playdate.rs`) kept; the lead screen uses it.
  The profile one-bit skin paints the engine's `ProfilesView`
  (`actions.menu`, `leaving` and `deleting`).
- **Deviations:**
  - **The pin move brought four other engine changes** (2201, 2308,
    2309, 2311, all merged before 2310), and 2308 and 2309 break the
    game. The smallest fix to compile, not those tickets' work: a menu's
    choice is read from its focus (`MenuEvent::Chosen(())`), the engine's
    menu and text box skins take `ctx`, the Playdate's menu helpers word
    `Label`s, one Playdate helper `ask_view` paints an engine `AskView`
    (options, Key bindings, profiles), and tests read menu labels through
    a `LabelText` trait (`widgets/menu.rs`) until 1162 keys the game's
    menus. 1162, 1163, 1165 and 1173 no longer need to move the pin; the
    rest of each is unchanged.
  - The screen's name is now the engine's `profiles` (was `profile`):
    tests only, the player sees no change.
  - Text keys: the engine ships no text table for games (its `en.ron`
    is its toy's), so the game's `profile.*` and `name_entry.*` keys stay
    in `assets/lang/*/ui.ron`, as the options and Key bindings keys do.
- **Friction:** estimate 40 added, measured 116 (2.9×), accepted: about
  80 of the 116 are the 2308/2309 adaptation above, which this ticket
  had to carry because the pin moves forward only; the profile work
  itself is about 35.
- **Follow-up tickets:** none.
- **For Nick:** nothing changes on screen; the Profile screen and the
  lead's name box look and work as before, on the PC and the Playdate.
