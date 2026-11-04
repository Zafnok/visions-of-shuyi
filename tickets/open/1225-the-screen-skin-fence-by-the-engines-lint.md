---
id: "1225"
title: "Check the screens' logic and skins with the engine's lint, not a hand-kept list"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: blocked
blocked_by: ["engine#2579"]
nick_input: none
completed:
---

# 1225 — The screen-skin fence by the engine's lint

## Context

From the game/engine boundary audit of 2026-11-03. ADR-0054 (and
`AGENTS.md` rule 8) say a screen's logic builds a view and its skin
paints it. The test that holds the game to it,
`the_logic_modules_name_no_cells_or_colours` in
`crates/ui/src/screens/title/tests.rs` (lines 594–671, with `names_word`
and its own test), reads an `include_str!` list of 14 screen files and 5
battle files with no screen. Screens added since are not on it:
`town.rs`, `camp.rs`, `world_map.rs` and `battle/shop.rs`. And
`world_map.rs`'s `draw` (line 909) calls a bare `paint(ctx,
&self.view(ctx), buf)`, which the test's string match would reject.

[engine#2579](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/2579-a-lint-that-a-screens-draw-only-paints-its-view.md)
makes the rule a `cargo shuyi lint` check (`SH2105`) that finds screens
by `impl Screen<…> for`, configured by `shuyi.ron`'s `screen_skin`.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine#2579 (`engine-change` skill §7, `engine-pin`
  skill), unless an open `engine-pin` PR has already done so.
- `shuyi.ron`'s `screen_skin` part, then whatever the lint finds fixed
  in the screens.
- The hand-kept test and `names_word` deleted.
- **Either order with 1211:** if 1211 is done, the test's list no
  longer names `credits.rs`: delete the test as it is; if not, delete it
  with `credits.rs` on the list, and 1211 has no list left to edit.

**Out (do not do):**
- Changing what a screen shows or how a skin paints it, beyond what a
  finding needs.
- The debug screens (`crates/ui/src/debug/`) and `flow.rs`, which are
  outside the screens' folder.

## Implementation steps

1. **Pin** (as above), then read the engine's `screen_skin.rs` and the
   `ScreenSkin` fields.
2. `shuyi.ron`, after `check_text`, with a comment naming ADR-0054 and
   this ticket:
   `screen_skin: (dir: "crates/ui/src/screens", logic: [the five
   battle files the test calls drawless: battle/mode.rs,
   battle/forecast.rs, battle/info.rs, battle/playback.rs,
   battle/progress.rs], names: ["UiColor", "Rect", "BoxStyle", "Cell",
   "draw_portrait"], hint: "build it in the view and paint it in the
   screen's skin (ADR-0054)")`, with paths as the engine's field docs
   want them.
3. Run `cargo shuyi lint`. Fix each `SH2105` finding in the screen's
   code (move a colour or a rectangle into its skin, or a view field).
   The audit expects none in the four new screens; say in the Completion
   notes what was found.
4. Delete `the_logic_modules_name_no_cells_or_colours`, `names_word` and
   `names_word_finds_whole_identifiers_only` from
   `crates/ui/src/screens/title/tests.rs`.
5. **Measure** with `cargo xtask code-lines`: no production lines unless
   a finding needs some; about 75 test lines removed; about 12 lines of
   `shuyi.ron`.

## Acceptance criteria

- [ ] `shuyi-engine.rev` is at or past the commit that closed
      engine#2579.
- [ ] `cargo shuyi lint` passes with `screen_skin` set, and reports no
      limit for it.
- [ ] Adding `let _ = Rect::new(0, 0, 1, 1);` to `town.rs` makes `cargo
      shuyi lint` fail with `SH2105` (checked by hand, not committed).
- [ ] `grep -rn "names_word" crates` finds nothing.
- [ ] All gates in the `run-gates` skill pass, including
      `cargo xtask ticket-lint --check-engine`.

## Tests required

- Gate: `cargo shuyi lint` (already in `run-gates` and CI). The engine's
  tests (engine#2579) cover the rule.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
