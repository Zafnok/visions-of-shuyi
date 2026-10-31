---
id: "0857"
title: "A switch to Casual reaches the battle from one place, and the battle screen's per-battle data is built once"
type: infra
milestone: M7 Chapter 1 & game flow
model: implementation
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0857 — The battle's mode and its own data set in one place

## Context

From the retro of the last 50 PRs (2026-10-22). Ticket 1171 (#376) gave
the campaign's mode, tutorial level and lessons one owner while the
campaign runs: `ctx.game.campaign_mode` (`crates/ui/src/screen.rs`).
Two duplications are left near it:

1. **The switch to Casual is written into the battle being prepared
   twice.** When the Options screen opened from Preparations switches the
   campaign to Casual (0805), the battle being prepared must start in
   Casual. Today two places do it:
   - `PreparationsScreen::uncovered` calls `switch_to_casual`, which
     sets its own `prep.setup.mode`
     (`crates/ui/src/screens/preparations.rs`);
   - the flow's `sync` (after `take_in`) sets `fight.prep.setup.mode`
     (`crates/ui/src/flow.rs`).
   On the normal path the flow's write is then overwritten when
   Preparations pops with its own `prep`. Two writers of one value, each
   kept in step by hand; the next mode-like setting would be a third.
2. **The battle screen's per-battle data is set by a builder chain
   written twice.** 0324 (#404) added `.with_exit_name(…)` to the chain
   `BattleScreen::start/resume(…).with_look(…).in_battle(&id)
   .with_exit_name(…).with_music(…)`, which the flow writes at both
   places it makes a battle screen (a new battle and a resumed one, in
   `flow.rs`). Each new per-battle field gets added twice, and a missed
   one is a resumed battle that differs from a new one.

## Nick input

None. Nothing a player sees changes; verified by the flow tests.

## Scope

**In:**
- One writer of the battle's mode from the campaign's: where the flow
  starts the battle from `fight.prep.setup` (the `BattleState::new`
  call in `flow.rs`), it sets the setup's mode from
  `ctx.game.campaign_mode` (Casual stays Casual: a battle never goes
  back to Classic). `switch_to_casual`, its call in `uncovered`, and the
  write in `sync` go, unless Preparations shows the mode itself; then it
  reads `ctx.game.campaign_mode` when it paints instead of keeping a
  copy.
- One function that dresses a battle screen for a fight
  (`fn battle_screen(def, look, music, screen) -> BattleScreen`, or a
  `BattleScreen` builder taking the `BattleDef`), used by both
  construction sites.

**Out (do not do):**
- Changing what the Options screen offers, or when a switch to Casual
  is allowed (0805, 1171).
- The save format (`Campaign` keeps its mode: it is what a save holds).

## Implementation steps

1. **Read** `flow.rs` (`take_in`, `sync`, `prepared`, the two
   `BattleScreen::start`/`resume` chains), `preparations.rs`
   (`switch_to_casual`, `uncovered`, anything that reads
   `prep.setup.mode`), `screen.rs` (`campaign_mode`, the switch), and the
   flow tests about 0805 (`grep -n "Casual" crates/ui/src/flow/tests.rs`).
2. **A test first** that pins today's behaviour: Options from
   Preparations switches to Casual → the battle that starts is Casual;
   the same with a resumed battle; the campaign saves Casual.
3. Move the write to where the battle starts; delete the other two.
4. The dressing function; both chains call it.
5. Run the flow tests and the Chapter 1 playthrough tests unchanged.

Expected size: about 30 production lines changed, net fewer.

## Acceptance criteria

- [ ] `grep -n "setup.mode = " crates/ui/src` finds one write of the
      battle's mode from the campaign's.
- [ ] The step 2 tests pass, before and after.
- [ ] `with_exit_name` (or whatever carries per-battle data) is called
      from one place in `flow.rs`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: `crates/ui/src/flow/tests.rs` (the switch to Casual on a
  new and a resumed battle).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
