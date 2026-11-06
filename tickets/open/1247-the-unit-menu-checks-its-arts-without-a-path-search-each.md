---
id: "1247"
title: The unit menu checks its arts without a path search for each
type: bug
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1247 — The unit menu checks its arts without a path search for each

## Context

`cargo shuyi bench` (ticket 1244, `docs/targets.md`) found the costliest
frame of the `battle-attack` scenario: **16.8 million instructions on the
desktop and 17.0 million on the Playdate, against the Playdate's budget
of 2 million**. It's the frame the unit menu opens, after the rider walks
onto the bridge. Its Callgrind dump (the `bench-report` artifact of PR
#558's `bench` job) shows where they go:

- `trpg_ui::screens::battle::art_list::art_lines` calls
  [`fit`](../../crates/ui/src/screens/battle/art_list.rs) for every
  technique and weapon slot;
- `fit` calls `BattleState::preview_attack` against **every unit on the
  map**, here 22, until one hits;
- each `preview_attack` runs `BattleState::plan`, which runs a full
  `trpg_core::movement::reachable` search for the move to `dest`. That
  makes 297 previews and 298 searches in one frame: 11.7 million
  instructions in `reachable` and its heap;
- `art_lines` runs three times in that frame: from
  `BattleScreen::triggers_now`, `ListKind::of` (the menu's Arts entry,
  whether it is enabled) and `mode::menu_entries`. Choosing a weapon
  (`attack::weapon_choices`, `attack::reach`) adds 3.3 million more.

The move to `dest` is the same for every preview. One plan of it, and
one list of the art lines per menu opening, should take the frame down
to a few hundred thousand.

## Nick input

None. No rule or look changes: the same lines, dimmed for the same
reasons.

## Scope

**In:**
- Plan the move once for a menu opening: give `preview_attack` (or a
  sibling in `trpg_core`) the planned path, or check the range without
  re-planning, so `fit` doesn't search per target.
- Only units that could be in range are previewed (the technique's range
  from `dest`), not every unit on the map.
- `art_lines` is computed once per menu opening, and the three callers
  share it.
- The `battle-attack` scenario's costliest frame drops under 2 million
  instructions on the desktop (`cargo shuyi bench`; the colour targets'
  run).

**Out (do not do):**
- The Playdate's one-bit drawing cost (1248).
- Changing which arts are offered or why one is dimmed.

## Implementation steps

1. Write a test that counts `reachable` calls (or `plan` calls) when the
   unit menu opens in the `side_company_ledger` battle (the
   `battle-attack` scenario's steps), and fails today.
2. Change `fit`/`preview_attack` as above, keeping every `Fit` the same
   (the existing `art_list` tests and snapshots must not change).
3. Cache the menu's `art_lines` for the menu's life in the battle screen's
   mode, and use it in `triggers_now`, `ListKind::of` and
   `menu_entries`.
4. Run `cargo shuyi bench --scenario battle-attack` in CI (or WSL with
   Valgrind) and put the new numbers in `docs/targets.md`.

## Acceptance criteria

- [ ] Opening the unit menu runs at most one path search for the moving
      unit (test).
- [ ] Every art line and its reason are unchanged (the `art_list` tests
      and snapshots pass unchanged).
- [ ] `battle-attack`'s costliest frame is under 2,000,000 instructions on
      the colour targets (the `bench` job's table, in `docs/targets.md`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the menu's search count.
- Snapshot: unchanged art lists.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
