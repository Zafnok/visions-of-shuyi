---
id: "1421"
title: "The game calls the engine pieces it copied: Ask, text_cells, window_by_heights, pcm16_wav, the dice picks, panel and pool"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1421 — The game calls the engine pieces it copied

## Context

The third engine-friction sweep (1397, 2026-11-27,
`docs/research/2026-11-27-engine-shaped-code-audit.md` §B) found game
code that does what an engine function at the pin (230080d) already
does. These need no engine work. Each item below names the game copy
and the engine function that replaces it. The audit checked the engine
side of every item.

## Nick input

None. Nothing the player sees changes. The HP bar's rounding would
change, so it is under **Out**.

## Scope

**In:**
- **Questions on the engine's `Ask`** (`shuyi_ui::widgets::Ask`:
  `yes_no`, `new`, `handle`, `view_with_keys`; the game already uses it
  in `screens/battle/shop.rs`):
  - the end-turn, restart and suspend prompts (`Mode::EndTurnPrompt`,
    `RestartPrompt`, `SuspendPrompt` in `screens/battle/mode.rs`);
  - the `question` closure and `QuestionView`
    (`screens/battle/mod.rs`, `screens/battle/view.rs:282`) and their
    two skins' painters;
  - the rewind list's `confirming` prompt (`screens/battle/rewind.rs`);
  - the lessons' end question (`end_focus`, `lessons_end_key` in
    `screens/battle/guide.rs`).

  The box still sits over the map where it does today. Leave
  `class_change/view.rs`'s own `QuestionView` alone unless it fits
  the same way.
- **`GlyphBuffer::text_cells`** in place of the private `cells` /
  `text_cells` copies:
  - `screens/battle/glyph/info.rs:44`
  - `screens/battle/glyph/shop.rs:216`
  - `screens/lead_select/glyph.rs:44`
  - `screens/save/glyph.rs:48`
  - `screens/save_import/glyph.rs:46`
  - the `class_change` copy, if it has one

  `screens/playdate.rs:79` takes the widths without a buffer: keep it
  if no buffer is at hand.
- **`ListCursor::window_by_heights`** in place of
  `stock_window_start` (`screens/preparations/glyph.rs:212`), as the
  rewind list does (`battle/glyph/rewind.rs`).
- **`shuyi_assets::audio::pcm16_wav(1, SAMPLE_RATE, s)`** in place of
  `sfx::wav_bytes` (`crates/tools/src/sfx.rs:743`) and its callers in
  `voice/synth.rs` and `voice_test.rs`. `cargo shuyi sfx --check` must
  still pass: the bytes must match the committed WAVs.
- **`RandomSource::index`/`pick`/`weighted_index`**:
  - in place of `roll_below(len) as usize` at
    `crates/core/src/worldmap/skirmish.rs:92,95,100`;
  - in place of the weighted growth pick in `progression.rs` `level_up`.

  These make the same draws (one `roll_below` each, the same
  cumulative rule). The replay and determinism tests are the proof.
- **`GlyphBuffer::panel(rect, &look, Some(title))`** in place of
  `paint_frame_sized`'s fill, box and title (`crates/ui/src/tips/glyph.rs:236`).
- **`shuyi_cli::pool::run`** for `bot_wins`' one thread per line
  (`crates/tools/src/bot_wins.rs:135`), with `pool::cores()` threads.
- **`content::font::text_width`** (`font.rs:76`) as a thin call of
  `shuyi_assets::lang::text_width`, if the widths it needs are at hand.
  Keep its signature, which has 53 callers.
- **`movement::area_tiles`** (`crates/core/src/movement.rs:484`) on
  `shuyi_core::threat::strike_area_from` (the centre removed).
  **Either order with 1180:** if 1180 is done, put it next to that
  ticket's adapters; if not, change only `area_tiles` and leave
  `attack_tiles` to 1180.

**Out (do not do):**
- `units.rs`'s `hp_fill`/`lit_fill` on the engine's `meter_fill`: the
  engine keeps one empty cell until full, so 99/100 HP would look
  different. That is a look change, not a refactor.
- Anything that needs an engine change (the sweep's other tickets,
  1400–1420).

## Implementation steps

1. Estimate first: about 250 production lines removed and 80 added
   (`engine-friction` step 1; the reference is the sweep's line counts).
2. Make each change above in its own commit, running the touched
   crate's tests after each.
3. Measure with `cargo xtask code-lines`.

## Acceptance criteria

- [ ] `grep -rn "fn cells(text" crates/ui/src/screens` finds nothing
      except `screens/playdate.rs`, if kept.
- [ ] `stock_window_start`, `wav_bytes`, `paint_frame_sized`'s hand
      box, `end_focus` and the battle's `QuestionView` are gone.
- [ ] The battle prompts behave as before: the existing battle tests
      and snapshots pass, and changed snapshots are explained in the PR.
- [ ] `cargo shuyi sfx --check` passes.
- [ ] The replay and determinism tests pass unchanged (skirmish and
      level-up draws).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing unit, snapshot and replay tests cover these changes. Add
  a unit test only where a copy had a test of its own that the engine's
  function doesn't keep.

## Completion notes

*(Filled in by the session that completes the ticket.)*
