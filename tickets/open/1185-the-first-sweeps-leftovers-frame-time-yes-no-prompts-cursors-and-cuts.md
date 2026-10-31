---
id: "1185"
title: "The first sweep's leftovers: five frame-time cleanups, four yes/no prompts on Ask, the last hand-written cursors, and cuts and centring on the helpers"
type: infra
milestone: "Engine: carve"
model: implementation
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
6. **Measure** with `cargo xtask code-lines` (estimate: count the call
   sites changed, not the helpers removed: about 120 added lines).

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
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each prompt's answers; the centring by cells.
- Snapshot: the battle's prompts, the rewind screen, the dialogue log
  (reviewed, expected unchanged).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
