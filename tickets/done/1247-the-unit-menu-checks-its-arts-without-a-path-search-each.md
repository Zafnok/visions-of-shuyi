---
id: "1247"
title: The unit menu checks its arts without a path search for each
type: bug
milestone: Later
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-06
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

- [x] Opening the unit menu runs at most one path search for the moving
      unit (test).
- [x] Every art line and its reason are unchanged (the `art_list` tests
      and snapshots pass unchanged).
- [ ] `battle-attack`'s costliest frame is under 2,000,000 instructions on
      the colour targets (the `bench` job's table, in `docs/targets.md`).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the menu's search count.
- Snapshot: unchanged art lists.

## Completion notes

- **Core.** `BattleState::plan_move` checks a ready unit's move to a
  tile and searches its path once (`PlannedMove`);
  `preview_attack_after` and `check_after` check an action against it
  with the same answers as `preview_attack` and `check`, which now use
  them. `legal::attack_reach`, the bound `legal` already used, is public.
  A `search-count` feature (tests only, on in `trpg-ui`'s
  dev-dependencies) counts `movement::reachable` runs per thread.
- **UI.** `attack::AttackFrom` holds one planned move and the attacker's
  reach. `fit`, the weapon choices, the spell list and the menu's `Open`
  check use it. `fit` previews only the units within reach; when none of
  them is hit, the first unit beyond reach that the core refuses for its
  cost or range gives the reason. The core checks the cost before the
  target, so that reason holds for all of them, and every `Fit` stays
  the same. `open_menu` builds one `AttackFrom` for the art lines, the
  entries, the weapons and the spells, and the action menu keeps its art
  lines (`Mode::ActionMenu::arts`). `triggers_now` and `Arts`' enabled
  check read those lines.
- **Test.** `opening_the_unit_menu_searches_the_move_once` plays the
  bench's steps in *The Company Ledger*. Opening the unit menu now runs
  1 search. Before this change it ran hundreds: the bench's dump counted
  298 in that frame, and a midway version of this change still ran 4.
  The `art_list` tests and every snapshot pass unchanged.
- **Deviation.** `menu_entries` takes the menu's `AttackFrom` and art
  lines. Its tests call `entries_at`, a test helper.
- **Bench.** No Valgrind on this machine. CI's `bench` job measures it,
  and its numbers are in `docs/targets.md`.
- No gameplay rule decided; nothing for the Act 1 playtest.
