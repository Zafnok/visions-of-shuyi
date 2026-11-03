---
id: "1211"
title: "Use the engine's credits screen"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3201"]
nick_input: none
completed:
---

# 1211 — Use the engine's credits screen

## Context

The game/engine boundary audit of 2026-11-03 found that the credits
screen (0808; `crates/ui/src/screens/credits.rs`, `credits/view.rs`,
`credits/glyph.rs`, about 270 production lines, with 643 lines of unit
tests) names no game concept. It reads the engine's
`shuyi_assets::credits::Credits`. Its only game inputs are
`TITLE_MUSIC` and the wrap widths 91/89/87, hard-coded for the PC
console (`credits.rs:60-66`). `docs/engine/boundary.md` lists
`screens/credits` as game code with no reason given (line 208). Engine
2223 had left an engine credits screen out "until a game asks".
[engine#3201](https://github.com/Zafnok/shuyi-engine/blob/main/tickets/open/3201-a-credits-screen-the-rolling-list-of-a-games-credits.md)
ports it to `shuyi_ui::screens::credits`, using the same text keys and
the same rolling (Nick, PR 148: it auto-scrolls, and manual scrolling
can be toggled on). This ticket deletes the game's copy
(`engine-change` skill §7; ADR-0092).

## Nick input

None. The screen must look and behave as it does now. The glyph
snapshots in `crates/ui/tests/it/snapshots/it__credits__*.snap` show
this and must not change.

## Scope

**In:**
- Move the pin past engine#3201's merge (`engine-change` §7, the
  `engine-pin` skill's review and ledger), unless an open `engine-pin`
  PR has done so.
- `crates/ui/src/screens/credits.rs` becomes an opener of about 15
  lines: `pub fn screen(ctx: &Ctx) -> shuyi_ui::screens::credits::CreditsScreen<Shuyi>`,
  that is `CreditsScreen::new(ctx, &ctx.game.content.credits).with_music(TITLE_MUSIC)`.
  Delete `credits/view.rs`, `credits/glyph.rs`, `credits/tests.rs` and
  `credits/glyph/tests.rs`.
- Callers: `title.rs:531` pushes `credits::screen(ctx)`. `screens/mod.rs:29`
  re-exports the engine's `CreditsScreen`. `title/tests.rs:444` uses
  `credits::screen(&c)`.
- `screens/actions.rs`: drop `CREDITS` and its row in `every_screen`'s
  own table (25 becomes 24). Add
  `ScreenActions::declared(CreditsScreen::NAME, …)` with the engine's
  declared actions beside `OptionsScreen`'s. The total stays 32.
- `crates/ui/tests/it/credits.rs`: keep the game's tests
  (`the_title_menu_opens_the_credits`, `a_scrolled_page`,
  `cancel_returns_to_the_title`, `every_credit_can_be_scrolled_to`,
  `every_glyph_drawn_is_in_the_font`, `sounds_and_music`). Move
  `the_games_credits_show_title_author_license_and_link` here from the
  unit tests. Delete the tests of the engine's rolling and scrolling
  (`the_list_rolls_by_itself`, `confirm_stops_and_restarts_the_rolling`,
  `after_the_last_row_the_list_starts_again_from_the_top`,
  `scrolling_stops_at_both_ends`). engine#3201 tests those.
- `docs/engine/boundary.md`: take `credits` out of the game row at line
  208, and add a row saying it moved to `shuyi_ui::screens::credits`
  (engine#3201, this ticket).

**Either order with 1046:** if 1046 is done, its Playdate credits
painter becomes the engine screen's `painted_by` painter, with a
`CreditsLayout` for 50 × 15 (or it is deleted if engine#2371 gave the
screen a one-bit painter); if not, nothing to do here (1046 paints the
engine's `CreditsView`, or uses engine#2371's painter).

**Either order with 1225:** if 1225 is done, the hand-kept list in
`screens/title/tests.rs` is gone and the engine's lint checks the
screens: nothing to do; if not, take `credits.rs` off the list in
`the_logic_modules_name_no_cells_or_colours`, since its draw is gone.

**Either order with 1214:** if 1214 is done, `tests/it/credits.rs`'s
font check and snapshot split read through the engine's readers: keep
them so as you trim the file; if not, trim it as listed, and 1214
converts what is left.

**Either order with 1238:** if 1238 is done, `credits/glyph.rs`'s box
is on `GlyphBuffer::panel`: delete the file as it is; if not, delete it,
and 1238 skips it.

**Either order with 1233:** if 1233 is done, its row for
`screens/credits` says "moves with 1211": write it as moved; if not,
change line 208 as above, and 1233 leaves the row.

**Out (do not do):**
- The credits file, its loading and the license lists
  (`crates/content/src/credits.rs`, engine 2223).
- Changing `credits.*` English in `assets/lang/en/ui.ron`: the engine
  uses the same keys, and the game's words stay.

## Implementation steps

1. Move the pin past engine#3201 (`cargo xtask engine-pin <commit>`).
2. Replace `credits.rs` with the opener. Delete the four files and
   fix the callers and `actions.rs`.
3. Trim `crates/ui/tests/it/credits.rs` as listed above. Run it: both
   snapshots must be unchanged. A change to a color only (the engine
   paints with `EngineColor` roles) is fixed in the game's palette
   names, never by accepting the snapshot.
4. Update `boundary.md`. Measure with `cargo xtask code-lines`.
   Expected: about 255 production lines and about 740 test lines
   removed, about 20 added.

## Acceptance criteria

- [ ] `crates/ui/src/screens/credits/` is gone and `credits.rs` is
      under 25 lines.
- [ ] `it__credits__the_title_menu_opens_the_credits.snap` and
      `it__credits__a_scrolled_page.snap` are unchanged.
- [ ] `sounds_and_music` still asks for `"title"` music once.
- [ ] `every_screen_lists_each_screen_once_with_its_actions` passes
      with 32 screens.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the kept `tests/it/credits.rs` tests through the
  harness, on the game's real credits.
- Unit: `actions.rs`'s listing test.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
