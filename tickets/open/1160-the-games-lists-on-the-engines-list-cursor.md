---
id: "1160"
title: "The game's lists on the engine's list cursor and scroll window: 13 hand-written cursors and 6 scroll windows go"
type: feature
milestone: "Engine: carve"
model: sonnet-5
effort: medium
status: todo
blocked_by: ["engine#2307"]
nick_input: none
completed:
---

# 1160 — The game's lists on the engine's list cursor

## Context

The first engine-friction sweep (1153, `docs/engine/friction.md`,
ranked first) found the same list cursor written by hand 13 times (8 to
12 lines each, around `(i + n - 1) % n` and a move sound) and the same
scroll window 6 times with four different rules. Engine ticket 2307
(`Zafnok/shuyi-engine`) adds `ListCursor` with wrap, disabled rows, a
clamp after a removal and a scroll window, and moves `Menu` onto it.

The copies (paths under `crates/ui/src/`):

- cursors: `screens/preparations.rs` (`step`, used by five lists, and
  `moved`), `screens/save.rs` (the slot picker), `screens/lead_select.rs`,
  `screens/class_change.rs`, `screens/battle/attack.rs` (two),
  `screens/battle/items.rs`, `screens/battle/magic.rs`,
  `screens/battle/skills.rs`, `screens/battle/guides.rs`,
  `screens/battle/mode.rs` (two);
- scroll windows: `screens/save.rs`, `screens/class_change.rs`,
  `screens/preparations/glyph.rs` (`window_start`),
  `screens/playdate.rs` (`first_shown`), `screens/credits.rs`,
  `screens/dialogue/log/glyph.rs`, `screens/battle/glyph/rewind.rs`,
  `screens/battle/glyph/guides.rs`.

(Search again: `grep -rn "+ n - 1) % n\|first_shown\|window_start" crates/ui/src`.)

## Nick input

None.

## Scope

**In:**
- Move the pin past 2307 (`engine-change` skill, step 7).
- Each cursor above becomes a `ListCursor` (or a `Menu`, where the list
  is already one in all but name).
- Each scroll window uses the cursor's window **with the rule it has
  today**: the PC's lists keep the focus in view, the Playdate's centre
  it (`docs/design/playdate.md`), the dialogue log and rewind use rows
  of different heights. Nothing the player sees changes.

**Out (do not do):**
- The profile screen (1164 replaces it whole).
- The battle menus' values and labels (1162), and folding the battle's
  list modes (1161). If 1161 is done first, its one list mode has one
  cursor to move; if not, move each.
- Changing any list's scroll rule or wrap: a change the player sees
  needs Nick.

## Implementation steps

1. Write the estimate: about 150 lines removed, 30 added.
2. One screen at a time, tests green after each.
3. Snapshots must not change. If one does, the rule was not kept: fix
   it, never accept the snapshot.

## Acceptance criteria

- [ ] `grep -rn "+ n - 1) % n" crates/ui/src` finds nothing outside
      tests.
- [ ] Every snapshot is unchanged.
- [ ] `cargo xtask code-lines` shows `trpg-ui` fell.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing tests and snapshots of each screen, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
