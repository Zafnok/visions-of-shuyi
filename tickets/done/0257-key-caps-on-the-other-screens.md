---
id: "0257"
title: "Key caps on the other screens: shops, Preparations, saves, town, world map, class change and the rest"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0254"]
nick_input: none
completed: 2026-11-17
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

- [x] No view a player sees on the PC names a key in plain text: the grep
      of step 1 finds only `.plain()` for tests, the Playdate's skins and
      the debug tools.
- [x] A test per switched view finds the keys `key_name` gives, in order.
- [x] With a controller in use, hints read as today.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each switched view's key spans.
- Snapshot: Preparations and the world map with key caps.

## Completion notes

Done on 2026-11-17, in one PR (no split: 919 changed lines).

- **Switched to key spans**, each view's help line a `StyledText` built
  with `ctx.text_styled` (or `key_pair`, `text_with_keys`, `join_hints`
  where the line is built from parts) and painted with the engine's
  `print_centred_styled` or `print_styled`: title, mode select,
  familiarity, lead select (and its text box's fixed keys, now the
  engine's styled `text_keys_help`; the game's own copy is gone), the
  voices question, the save question and the save/load slots, the save
  import, game over, "To be continued" and the demo's end, credits, camp,
  the world map (both skins), Preparations, class change, results, the
  dialogue log, and the writer's preview's names page. In the battle: its
  yes/no questions (the battle's dialog boxes now take styled lines) and
  the forecast's `Left/Right weapon`. The rest the ticket listed in the
  battle (the menus' hints, objective page, info screen, EXP pages,
  notes) already show their keys only on the help bar 0254 switched.
  The Profile screen's own lines and the engine's questions (`Ask`) were
  already styled.
- The title's help line fades with the menu, its caps too; over the
  cinematic it keeps no strip behind its words, as before.
- **Playdate skins** read `.plain()`, as the ticket says.
- **Tests:** `widgets::help::{key_spans, spans_of}` (test and harness
  builds only) read a hint's keys and what `key_name` names for a list of
  actions; each switched view has a test that they agree, in order.
  `tests/it/key_caps.rs` adds Preparations and the world map (the test
  world map) with caps, as snapshots, and Preparations on a controller
  (plain text, no key spans).
- **Snapshots:** about 70 changed only in the key cells' colours (the
  palette's `key` and `key_bg`, as in 0254). About ten from tests
  that build the real game's context (key caps on) now draw the caps:
  the save slots and question, the title, the world map's help line.
  Two private ones (`it__world_map__act1_start`,
  `it__title_cinematic__title_cinematic_12_logo`) in
  `Zafnok/visions-of-shuyi-assets`, its PR linked in this PR, branched
  from the current pin (the private `main` has 0490's content, whose game
  PR is still open). One test (`save.rs`) read the slots' frame text and
  now reads the caps.
- **Picture:** `docs/screenshots/0257-preparations.png` (`cargo shuyi
  frame-png --keys "Down f"`, the Quick Battle's test units). The world
  map was not rendered with `frame-png`: its New Game plays the private
  story, whose map and scenes may not be committed (ADR-0118); the
  snapshot `it__key_caps__the_world_map_draws_its_keys_as_caps` shows the
  test world map's help line with caps instead.
- **Left plain:** the debug tools, as the ticket says; the battle's
  `help()` (its words, for tests).

Size: estimate 250 production lines (about 8 a screen for 20 screens and
the battle's two hints, judged by 0254's town and rewind switches);
measured 248 added, 131 removed (`code-lines`), 1.0×. PR size 919
changed lines.

*Claude's starting rules:* none (no gameplay).

For the Act 1 playtest (1064): whether the caps read well on the menus'
centred help lines.
