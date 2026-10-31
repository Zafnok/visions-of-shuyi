---
id: "1160"
title: "The game's lists on the engine's list cursor and scroll window: 13 hand-written cursors and 6 scroll windows go"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["engine#2307"]
nick_input: none
completed: 2026-10-19
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
      tests. Two are left, both in `screens/profile.rs`, which
      the Out list leaves to 1164.
- [x] Every snapshot is unchanged.
- [x] `cargo xtask code-lines` shows `trpg-ui` fell.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- The existing tests and snapshots of each screen, unchanged.

## Completion notes

Estimate: about 30 lines added (the ticket's figure). Measured with
`cargo xtask code-lines`: 82 added, 130 removed in `trpg-ui`. Friction:
2.7×, accepted: a changed line counts as added, and most are a field
read such as `self.focus` becoming `self.cursor.focus()`; nothing new
was built around the cursor (row in `docs/engine/friction.md`).

What was done:

- The pin moved to the engine's `main` (ed61c0e), past 2307. Its
  `MenuView` gained `first_shown` and `shown`; the views built by hand
  in tests (and two skins' test views) fill them in.
- **A `ListCursor` held by the screen:** the save picker (its focus and
  its window of 20 slots, the same keep-in-view rule), class change (its
  column and its window of 4 columns) and the Guides list.
- **Lists that keep their focus as a plain index** (the battle's target
  cycles in attack, items, magic and skills, the weapon swap, the Info
  and Talk cycles in `mode.rs`, the five Preparations lists and its
  tabs, the lead select rows): they step through `widgets::wrap_step`,
  one `ListCursor` step, instead of `(i + n - 1) % n`. Their indices
  are public fields read across the battle, so holding a cursor in each
  would have rippled far past this ticket for no player-visible change.
  Preparations' `lists_wrap` test moved with the helper
  (`widgets::tests::a_step_wraps_round_the_ends`).
- **Scroll windows on the cursor's window:** Preparations'
  `window_start` and the Guides skin's `first_shown` (a fresh
  keep-in-view cursor: the focus at the bottom edge once it passes it,
  as before), the rewind list (`window_by_heights`, rows of different
  heights). Their own unit tests are unchanged and pass.

Deviations (kept as they are, so nothing the player sees changes):

- **The Playdate's `first_shown`** (`screens/playdate.rs`, also used by
  its options and key-bindings skins) puts the focus `room / 2` rows
  down; the engine's `Scroll::Centred` uses `(room - 1) / 2`, one row
  higher on an even window. Moving it would shift every even-height
  Playdate list by a row, a look change that needs Nick.
- **The dialogue log's window** shows up to a few rows after the focus,
  then fills before it, then after: no `Scroll` rule does that.
- **The credits** roll by themselves with no focus: a scroll offset,
  not a list cursor.
- The profile screen still has two `(… + n - 1) % n`: out of scope
  (1164 replaces the screen).

No follow-up tickets: the three kept windows are small, and changing
any of them is a look change. No gameplay rules decided.
