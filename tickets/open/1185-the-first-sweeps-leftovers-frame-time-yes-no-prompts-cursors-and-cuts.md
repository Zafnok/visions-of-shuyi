---
id: "1185"
title: "The first sweep's leftovers: five frame-time cleanups, four yes/no prompts on Ask, the last hand-written cursors, and cuts and centring on the helpers"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1039"]
nick_input: none
completed:
---

# 1185 — The first sweep's leftovers

## Context

From the retro of the last 50 PRs (2026-10-22). The first friction
sweep (1153, `docs/engine/friction.md`) filed tickets that moved the
game onto new engine pieces: 1160 (list cursor and scroll window), 1163
(the question widget `Ask`), 1165 (drawing helpers), 1166 (frame time).
Each found its targets with a grep, ran once, and some ran in parallel
with a ticket that renamed or moved the code. What they missed:

1. **Five frame-time cleanups.** 1167 (#371) renamed `input.dt` to
   `input.dt()` 37 minutes before 1166 (#373) merged, and 1166's grep
   (`dt.is_finite()`) couldn't match the new spelling. The engine's
   `FrameInput::dt()` is "always finite and at least 0", yet
   `if input.dt().is_finite() { input.dt().max(0.0) } else { 0.0 }` is
   still in `screens/battle/mod.rs`, `screens/results.rs`,
   `screens/title.rs` (twice) and `debug/cinematic.rs`. (1166's notes
   say "the others went with earlier carve tickets"; none had.)
2. **Four yes/no prompts not on `Ask`.** 1159 (#365) made the rewind a
   screen three hours before 1163 merged, and 1163's list didn't have
   it. Still hand-written: the battle's End turn, Restart and Suspend
   prompts (`Mode` variants in `screens/battle/mode.rs`, shown through
   `QuestionView` in `view.rs`, painted by `draw_dialog` in
   `screens/battle/glyph.rs`), and the rewind's "Use 1 of N charges?"
   (`confirming: bool` in `screens/battle/rewind.rs`, painted in
   `glyph/rewind.rs`). The box sized to its text with a double border is
   written twice (`dialog_size`/`draw_dialog_at` in the battle's
   `glyph.rs`, and `screens/class_change/glyph.rs`), beside the engine's
   own Ask painter. The same answer line is in `assets/lang/en/ui.ron`
   three times: `save.overwrite_answers`, `battle.yes_no`,
   `class_change.yes_no`.
3. **The last hand-written cursors.** The dialogue log steps its focus by
   hand (`saturating_sub(1)` and `(focus + 1).min(…)` in
   `screens/dialogue.rs`), which is `ListCursor::step` without wrap.
   `wrap_step` steps through an `ALL` array in `lead_select.rs` and
   `preparations.rs`: the engine's public `stepped`
   (`shuyi_ui::screens::options::stepped`). `preparations.rs` clamps a
   focus after a removal by hand (`ListCursor::set_len`), and
   `debug/portrait_viewer.rs` has its own wrap.
4. **Cuts and centring.** 1165 kept `print_clipped` (a hard cut, waiting
   on 0068's decision) but about nine inline `chars().take(w)` cuts
   bypass it (`screens/battle/progress.rs` five times,
   `screens/battle/stage/glyph.rs` twice, `screens/class_change/glyph.rs`
   once), and `Pen::cut` in `screens/battle/info.rs` is a second
   hard-cut helper. Two placeholders are centred by `label.len()` (bytes,
   not cells: `info.rs` and `progress.rs`), so a non-ASCII translation
   sits off-centre.
5. **Three more, from the game/engine boundary audit of 2026-11-03.**
   The box at the end of the lessons steps its three answers by hand
   (`lessons_end_key` in `screens/battle/mod.rs`, over the plain
   `end_focus` field and `EndAnswer::ALL`, playing the move and select
   sounds itself): the engine's `Ask` with no cancel answer.
   `turn_page` in `screens/battle/info.rs` writes its own wrap
   (`(page + 1) % pages`, `checked_sub(1).unwrap_or(pages - 1)`) beside
   `wrap_step`. And the battle shop's special-sale question
   (`step_special` in `screens/battle/shop.rs`) hands `Ask::handle` a
   throwaway `AudioQueue::new(AUDIO)`, then compares the focus before
   and after to report `ShopEffect::Moved`; the town's and the
   Preparations' shops pass `ctx.audio`.

The battle's five target pickers keep a plain `index` and `wrap_step`
behind 1161's `Picker` trait on purpose (their fields are public to
the battle); they stay.

## Nick input

None. Nothing a player sees changes, except that the two placeholders
are centred by cells (the same in English). Verified by the snapshots.

## Scope

**In:**
- The five frame-time cleanups become `input.dt()`.
- The End turn, Restart and Suspend prompts and the rewind's charge
  prompt held as the engine's `Ask` (`Ask::yes_no`, `view_with_keys`),
  painted by the engine's Ask painter or, where the look must stay (a
  box sized to its text in the map's middle), by one game painter shared
  by the battle and class change. End turn keeps answering yes to the
  End turn action as well as Confirm: handle `Action::EndTurn` before
  passing the key to the `Ask`.
- One `ui.ron` key for the answer line (keep `save.overwrite_answers`
  if the save's wording differs; otherwise one key, the others removed
  from every language pack, `assets-test/lang/test` too).
- The dialogue log on `ListCursor`; `stepped` for the `ALL` arrays;
  `set_len` for the clamps; the portrait viewer's wrap on `wrap_step` or
  `ListCursor`.
- Inline cuts through `print_clipped` (or `cut` where nothing is
  printed), `Pen::cut` on it, and the two placeholders on
  `print_centred`.
- The lessons-end box held as an `Ask<EndAnswer>` (no `cancelled_by`),
  `lessons_end_key` passing the key to `ask.handle(action, &mut
  ctx.audio)`, and `LessonsEndView`'s answers and focus taken from it.
  `Ask` also steps on Left and Right; accept that, or filter the two
  before the `Ask`, and say which in the Completion notes.
- `turn_page` on `wrap_step`.
- The battle shop's special-sale question given the screen's audio
  queue, as the other two shops do, so the throwaway queue and the
  focus comparison go. If the battle shop's step must stay free of
  `ctx`, keep it and say why in the Completion notes.
- Snapshot changes, reviewed (none expected except where a test language
  has non-ASCII text).

**Out (do not do):**
- The pickers' `index` (above).
- Choosing a cut with `…` (0068 decides; this ticket makes the switch one
  place).
- The one-bit skins (1077 and 1184).
- New engine features: if `Ask` can't express a prompt, keep that prompt
  and write it in the Completion notes with the `engine-friction` skill's
  outcome.
- The hand-built boxes beside `GlyphBuffer::panel` and the shop, Guides
  and Preparations list windows (1238).

**Either order with 1238:** if 1238 is done, the cuts sit inside its
panels: change only the cuts. If not, leave the `fill_rect` +
`draw_box` pairs for 1238. **Either order with 1216** (one shop flow):
if 1216 is done, its shared shop picker plays on the queue it is handed
and the throwaway queue is gone: skip that item. If not, change
`step_special` in `screens/battle/shop.rs`; 1216 then keeps it.

## Implementation steps

1. **Read** the engine's `widgets/ask.rs`, `widgets/list.rs`,
   `screens/options.rs` (`stepped`), `frame/draw.rs` (`print_centred`,
   `print_cut`), and the game files named above. Note 1039 (blocked by:
   it rewrites the rewind's skins) and read what it changed.
2. **Frame time**: `grep -rn "is_finite()" crates/ui/src`: every hit on
   a frame time goes.
3. **Prompts**, one at a time, each with its tests green before the
   next: rewind, Suspend, Restart, End turn (it has the extra action).
4. **Cursors**: the dialogue log, the `ALL` arrays, the clamps, the
   viewer.
5. **Cuts and centring**: `grep -rn "chars().take(" crates/ui/src` and
   `grep -rn "len()) / 2\|\.len() / 2" crates/ui/src` for the
   centring; each hit on a label goes through the helper.
6. **Item 5**: the lessons-end box, `turn_page`, the shop's question.
7. **Measure** with `cargo xtask code-lines` (estimate: count the call
   sites changed, not the helpers removed: about 120 added lines, and
   about 20 more for item 5, which removes about 18).

## Acceptance criteria

- [ ] `grep -rn "dt().is_finite()" crates/` finds nothing.
- [ ] The four prompts are `Ask`s (a test per prompt: Confirm says yes,
      Cancel says no, End turn's own action says yes).
- [ ] One painter draws a sized yes/no box in the game, or none.
- [ ] `grep -rn "saturating_sub(1)" crates/ui/src/screens/dialogue.rs`
      finds no cursor step; `lead_select.rs` and `preparations.rs` step
      their `ALL` arrays with `stepped`.
- [ ] `grep -rn "chars().take(" crates/ui/src/screens` finds no label
      cut outside the helpers.
- [ ] A test centres a label with a two-byte character by cells.
- [ ] `grep -rn "end_focus\|unwrap_or(pages - 1)" crates/ui/src/screens/battle`
      and `grep -rn "AudioQueue::new" crates/ui/src/screens` find
      nothing outside tests.
- [ ] A test: the lessons-end box wraps from its last answer to its
      first, and Cancel answers nothing.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each prompt's answers; the centring by cells; the lessons-end
  box; `turn_page` at both ends; the special sale's move sound.
- Snapshot: the battle's prompts, the rewind screen, the dialogue log
  (reviewed, expected unchanged).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
