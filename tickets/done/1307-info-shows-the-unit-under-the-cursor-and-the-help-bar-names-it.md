---
id: "1307"
title: "With a unit selected, Info shows the unit under the cursor, and the help bar names it"
type: bug
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["1305"]
nick_input: none
completed: 2026-11-20
---

# 1307 — With a unit selected, Info shows the unit under the cursor, and the help bar names it

## Context

1305 (PR #777) made Info open the *selected* unit's info while a unit is
selected. Nick played the re-pushed Chapter 1 web demo on 2026-11-20:

> E unit info shouldnt override unit occupied tiles with the selected
> units info, its fine to keep the behavior for blank tiles. and, it
> should probably say E Unit info hint at the bottom like every other
> available key available in the context. Better yet would be (unit
> name)'s info but it's ok to just keep unit info if some unit names can
> be too long.

E is the Info action on the right-handed layout. This comes before the
writer trial's packets go out (0766).

## Nick input

None: the message above settles the behavior and the hint's words.

## Scope

**In:**
- With a unit selected (its move range up), Info on a tile that holds a
  unit (ours, an ally, an enemy) opens **that** unit's info; on an empty
  tile it opens the selected unit's info (1305). Cancel returns to the
  selection either way.
- In the selected unit's action menu, Info keeps showing the selected
  unit (1305).
- The help bar in those two contexts shows the Info key with `<name>'s
  info`, the unit Info would open; when the line with the name would not
  fit the help row (beside the debug hint, if shown), `unit info`. The key
  is named through the player's keymap (`{Info}` in the text), never
  written.

**Out:**
- The browsing help bar (`e info` stays as it is), other modes' help.

## Implementation steps

1. Expected size: about 40 production lines (one helper choosing the
   unit, one help hint), judged by 1305's change.
2. `crates/ui/src/screens/battle/mode.rs`: a `Mode::info_unit(cursor,
   state)` giving the unit Info opens in `Selected` (the unit on the
   cursor's tile, else the selected one) and `ActionMenu` (the selected
   one); `step_with` uses it.
3. `crates/ui/src/screens/battle/mod.rs` `help_parts`: in those modes,
   join a hint `battle.hint.info_of` (`{Info} {name}'s info`) or, when
   it doesn't fit, `battle.hint.unit_info` (`{Info} unit info`) after
   the mode's line. Add both keys to `assets/lang/en/ui.ron` and the
   `test` language pack.
4. Tests: mode tests (Info on an enemy, an ally, an empty tile; Cancel
   back), help tests (the name, the fallback for a name too long), the
   Harness test from 1305 extended.
5. A `cargo shuyi frame-png` of the help bar, cropped (no real map,
   ADR-0118). Run the `run-gates` skill.

## Acceptance criteria

- [x] With a unit selected, Info on another unit's tile opens that unit's
      info, on an empty tile the selected unit's; Cancel returns to the
      selection (tests).
- [x] The help bar shows the Info key with `<name>'s info` while a unit
      is selected or in its action menu, `unit info` when the name doesn't
      fit (tests, picture in the PR).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit tests of the mode step and the help line; the Harness test.

## Completion notes

- `Mode::info_unit(cursor, state)` picks the unit Info opens: with a unit
  selected, the unit on the cursor's tile (ours, an ally or an enemy),
  else the selected one; in its action menu, the selected one. The step
  and the help bar both read it, so the hint always names the unit Info
  would open.
- The help bar in those two modes ends with the Info key and
  `<name>'s info` (`battle.hint.info_of`), or `unit info`
  (`battle.hint.unit_info`) when the line with the name would not fit the
  help row beside the debug hint. Both keys are in English and the `test`
  pack; the key comes from the keymap.
- The hint shows in every form of the selected help (move here, attack,
  cast, heal, plain move), since Info works in all of them.
- About 20 tests compared the old selected and action-menu help lines
  exactly; they now expect the hint (the name tells which unit Info would
  open). `wait_all` checks the line's start, as it walks several units.
- Starting rules: none.
