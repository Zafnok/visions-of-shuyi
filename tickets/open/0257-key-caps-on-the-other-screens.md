---
id: "0257"
title: "Key caps on the other screens: shops, Preparations, saves, town, world map, class change and the rest"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0254"]
nick_input: none
completed:
---

# 0257 — Key caps on the other screens

## Context

0254 turned on the key-cap style Nick decided in 0250
(`docs/design/look-and-feel.md`, *Key names in hints*) and switched the
battle's help bars and toggles (with its Guides, rewind and shop
screens, which the town shares), the text box's corner, the tip, lesson
and end-of-lessons boxes to styled text. The engine's own screens
(Options, Key bindings, its questions) already draw caps. 0254 split
here, as its *Split* said, because the game names keys in about 270
places across 55 files, far over its estimate of 15.

The tools are in place: `widgets::help::{action_key, key_pair,
help_line_styled, join_hints, text_with_keys, frame_widths,
wrap_styled}`, `tips::fill_styled`, `ctx.text_styled`, and the
`keyboard-input` skill's *Key caps*.

## Nick input

None: decided in 0250.

## Scope

**In:** every other view whose text names a key for the player switched
to `StyledText` and painted with `print_styled`:
- Preparations and its shop (`screens/preparations*`), the world map
  (`screens/world_map*`), camp, the save and load screens and the save
  import (`screens/save*`, `save_import.rs`), class change
  (`screens/class_change*`, with `progress::help`), results, game over,
  title, lead select, mode select, familiarity, credits, profiles, the
  voices question, the dialogue log;
- in the battle: the forecast's `Left/Right weapon` hint
  (`ForecastView::swap`), the art, skill, magic, item and map menus'
  hints, the objective page, the info screen and the EXP pages
  (`progress.rs`), the battle notes;
- each view's test lists its key spans (as `tests/it/key_caps.rs`
  does), and a snapshot of a few with `Harness::with_key_caps`.

**Out:** the debug tools (plain, as the engine leaves them); a
controller's buttons (0255) and the Playdate (0256); the engine's
styled word wrap (`widgets::help::wrap_styled` is the game's workaround
until the engine has one).

## Implementation steps

1. `grep -rn "key_name\|all_key_names\|cursor_keys_name\|help_line\|text_with" crates/ui/src`
   (not tests or `debug/`), less what 0254 switched.
2. Switch each view, its skin and its tests (`plain()` where a test
   compares words).
3. Snapshots, `cargo shuyi frame-png` of Preparations and the world map,
   the gates.

**Expected size:** about 700 production lines, plus tests and snapshots.
**Split:** past 2,000 changed lines, ship Preparations, the world map and
saves, and move the rest to a new ticket.

## Acceptance criteria

- [ ] No view a player sees on the PC names a key in plain text: the grep
      of step 1 finds only `.plain()` for tests, the Playdate's skins and
      the debug tools.
- [ ] A test per switched view finds the keys `key_name` gives, in order.
- [ ] With a controller in use, hints read as today.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each switched view's key spans.
- Snapshot: Preparations and the world map with key caps.

## Completion notes

*(Filled in by the session that completes the ticket.)*
