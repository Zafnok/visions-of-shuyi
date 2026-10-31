---
id: "1161"
title: "One list mode and one target picker in the battle's modes, instead of five list modes and three near-copy pickers"
type: feature
milestone: "Engine: carve"
model: reasoning
effort: high
status: done
blocked_by: []
nick_input: none
completed: 2026-10-19
---

# 1161 — One list mode and one target picker in the battle's modes

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`)
measured `crates/ui/src/screens/battle/mode.rs` at 1243 production
lines against an estimate of 925 (1.3×). The extra is a game-side
structure missing (outcome 3), not an engine gap:

- **Five list menus written as five modes.** `Mode::ArtsMenu`,
  `SpellMenu`, `SkillMenu`, `ItemMenu` and `EquipMenu` each have an
  `open_*` and a `step_*` (`open_arts`/`step_arts`, `open_spells`/
  `step_spells`, `open_skills`/`step_skills`, `open_pack`/`step_pack`,
  `open_equip`/`step_equip`), and each step repacks its variant in three
  match arms; `back_to_entry(sel, state` appears 14 times. The same six
  variants are listed by hand in eight more places (`mod.rs`'s help,
  view and scene builders, `sounds.rs`, `told()`).
- **Three near-copy target pickers.** `step_cast_target`,
  `step_skill_target` and `step_item_target` differ in what is aimed
  and what Confirm does; the line `Action::CursorRight |
  Action::CursorDown | Action::NextUnit` appears five times.

About 275 lines are this repetition.

## Nick input

None.

## Scope

**In:**
- One `Mode::List { sel, menu, kind: ListKind }` (`ListKind` = Arts,
  Spells, Skills, Pack, Equip, carrying what each needs: its choices or
  lines), one `open_list(kind, sel, state)` and one `step_list`, with a
  `match kind` only where the five really differ (what Confirm does).
- One target picker: a `Targeting` shape (or a small trait) for what is
  aimed and what Confirm does, and one `step_target`.
- The eight places that list the variants by hand match `Mode::List`
  once.

**Out (do not do):**
- Items that carry a value and labels translated at paint time (1162,
  which needs engine 2308). This ticket keeps today's `Menu` and
  `told()`.
- The list cursor (1160). Either order works: if 1160 is done first,
  `step_list` has one cursor to keep; if not, 1160 moves this one.
- Any change the player can see: every key, sound, help line and menu
  stays the same.

## Implementation steps

1. Write the estimate before coding: about 250 lines removed, 60 added
   (`cargo xtask code-lines` after).
2. Add `ListKind` and `Mode::List`; move one menu (Arts) onto it with
   its tests green; then the other four, one at a time.
3. Fold the three pickers the same way.
4. Remove the hand-listed variants in `mod.rs` and `sounds.rs`.

## Acceptance criteria

- [x] `grep -n "ArtsMenu\|SpellMenu\|SkillMenu\|ItemMenu\|EquipMenu" crates/ui/src`
      finds nothing outside tests' names.
- [x] `grep -c "Action::CursorRight | Action::CursorDown | Action::NextUnit" crates/ui/src/screens/battle/mode.rs`
      is 1.
- [x] Every battle test, snapshot and bot playtest passes unchanged.
- [x] `cargo xtask code-lines` shows `trpg-ui` fell by at least 150.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The existing mode tests (they drive keys and check modes and effects),
  unchanged in what they check; names may follow the new variant.

## Completion notes

- **One list mode.** `Mode::ArtsMenu`, `SpellMenu`, `SkillMenu`,
  `ItemMenu` and `EquipMenu` are now `Mode::List { sel, menu, kind }`,
  with `ListKind::{Arts, Spells, Skills, Pack, Equip}` carrying each
  list's lines. `ListKind::of` (the list an action-menu entry opens),
  `entry`, `can_choose` and `menu` are the only places the five differ,
  besides `choose_line` (what Confirm does). One `open_list` and one
  `step_list` replace the ten `open_*`/`step_*` functions, and
  `MenuEntry::enabled` and `told()` use `ListKind` too.
- **One target picker.** A private `Picker` trait (cycle, the cursor's
  tile, Confirm, Cancel's way back, the mode) is implemented by
  `Targeting`, `CastTargeting`, `SkillTargeting`, `ItemTargeting` and a
  new `TalkTargeting`. One `step_target` steps all five. The attack
  forecast's left/right weapon swap stays a guard in front of it.
  `Mode::TalkTarget` now holds a `TalkTargeting`, the same shape as the
  other pickers.
- **The hand-listed variants** in `mod.rs` (help, tips, lesson marks,
  threat arrows, cursor, menu view and header) and `sounds.rs` match
  `Mode::List` once. `drawn_pos` now reuses `selection()`, and
  `back_to_entry` is shorter.
- **Deviation, no change the player can see:** the talk picker re-aims
  the cursor at its target on any key it doesn't use, and plays the move
  sound. The other pickers do nothing on such a key. A test pins that
  difference, so `Picker::other_key` keeps it (a default that does
  nothing, overridden by talk).
- **Tests:** only patterns and constructions changed (to
  `Mode::List { kind: ListKind::…, .. }` and `Mode::TalkTarget(_)`). Each
  test still checks which list is open.
- **Measured:** `cargo xtask code-lines` shows `crates/ui` +313 / −464
  (production lines fell by 151). Estimate written before coding: 100
  added. 3.1×, reviewed with `engine-friction`: outcome 4 (accept). No
  engine glue; a changed line counts as added, and most added lines are
  match arms rewritten for `Mode::List`. Row in `docs/engine/friction.md`.
- No follow-up tickets. No gameplay rules decided (nothing the player
  sees changed).
