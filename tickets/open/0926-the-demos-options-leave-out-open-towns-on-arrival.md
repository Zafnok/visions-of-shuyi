---
id: "0926"
title: "The demo's Options leave out Open towns on arrival, which has no world map to act on"
type: bug
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0926 — The demo's Options leave out Open towns on arrival, which has no world map to act on

## Context

A test audit of the Chapter 1 web demo (ticket 0914, `docs/design/demo.md`), done before the demo goes to applicant writers (0766, on the password page from 0915), found that its Options screen shows `Open towns on arrival` (`assets/lang/en/ui.ron`: `"options.row.towns": "Open towns on arrival"`). The demo has no towns and no world map: `crates/content/demo/bundle.txt` cuts `skirmishes/` and `worldmaps/` ("The world map and its skirmishes come after Chapter 1."). So the row is a setting that does nothing.

The Options screen (`crates/ui/src/screens/options.rs`, ticket 0805) already leaves out rows that mean nothing where it runs. `PC_ONLY` drops `Row::Towns` on the Playdate ("it has no world map yet (1007; the row comes with it)"), and `shown_rows` shows `Map look` only when the game has pictures (`has_pictures`). This ticket applies the same rule to the world map: the row shows only when the game's content has a world map. That condition also covers the demo. What the setting does is unchanged (`docs/design/options.md`, *Open towns on arrival*, ticket 1007).

A writer in the trial (0766) would see the row, but it is harmless, so this ticket does not block 0766. If a session is free, land it before the packets go out.

## Nick input

None. Leaving out a row that has nothing to act on follows the screen's existing rule (Playdate rows, Map look). No new look or rule is decided. Verified by the tests below and a `cargo shuyi frame-png` picture of the demo's Options in the PR. Nothing is left to judge by playing.

## Scope

**In:**
- `shown_rows` in `crates/ui/src/screens/options.rs` leaves out `Row::Towns` when `ctx.game.content.worldmaps` is empty.
- The module and function doc comments say so.
- Tests: a unit test and one assertion in the demo's integration test.

**Out (do not do):**
- Changing the setting, its default or its text, or hiding it in the full game before the world map first appears (the full game has a world map after Chapter 1, so the row stays there).
- The Playdate's `PC_ONLY` list (keep `Row::Towns` in it; 1007 brings it back with the Playdate's world map).
- The demo's Game mode row and its message (0927, a decision for Nick).
- Any other row.

## Implementation steps

Expected size: about 10 production lines (one filter and its doc lines), the same shape as the `MapLook` filter.

1. In `crates/ui/src/screens/options.rs`, `shown_rows`: add `let world = !ctx.game.content.worldmaps.is_empty();` and `.filter(|&r| r != Row::Towns || world)` beside the `MapLook` filter.
2. Update the doc comment of `rows` ("The rows shown, as the engine's screen takes them: ...") with "`Open towns on arrival` only when the game has a world map (not in the Chapter 1 demo)", and the module doc's paragraph on the Playdate if it lists hidden rows.
3. Check `Restore defaults` and `every_setting_row_changes_its_own_setting` still pass. The setting itself stays in `Settings`; it is only not shown.
4. Add the tests below. Run `cargo shuyi frame-png` on the demo's title → Options for the PR description.

## Acceptance criteria

- [ ] With `ctx.game.content.worldmaps` empty, `shown_rows` has no `Row::Towns`. With a world map loaded it has it, on the PC (unit test `the_towns_row_shows_only_with_a_world_map` passes).
- [ ] Against the demo's bundle (`bundle::demo_layers()`), the title's Options screen lists no `Open towns on arrival` row (a new assertion in `crates/ui/tests/it/demo.rs` passes).
- [ ] The full game's Options, with the test content's world map, still shows the row: the existing `game_mode_and_tutorial_show_only_with_a_campaign` and `the_playdate_leaves_out_the_rows_that_mean_nothing_there` pass, updated only if their expected lists change.
- [ ] The PR shows a `cargo shuyi frame-png` picture of the demo's Options without the row.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: in `crates/ui/src/screens/options/tests.rs`, `the_towns_row_shows_only_with_a_world_map`: test `ctx()` (which has the test world map) shows `Row::Towns`, and after `c.game.content.worldmaps.clear()` it doesn't. The Playdate still never shows it.
- Property: none (a single condition).
- Snapshot / integration: in `crates/ui/tests/it/demo.rs`, open Options from the demo's title and assert the row labels hold no `Open towns on arrival`. Update an existing Options snapshot only if one is taken against content without a world map.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations, follow-up tickets created, notes for Nick.)*
