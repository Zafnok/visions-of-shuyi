---
id: "0845"
title: "Chapter 1's objective becomes defeating Red Harl; the rest of the Red Company runs"
type: feature
milestone: M7 Chapter 1 & game flow
model: reasoning
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-10-22
---

# 0845 — Chapter 1's objective: defeat Red Harl

## Context

Chapter 1's objective is Rout: defeat every enemy (`assets/battles/
ch01.ron:49`, `objective: Rout()`; `docs/design/chapter-1.md`,
*Objective and loss*: Nick, Q1, "Let's pick Rout"). Chapter 1 playtest
round 3 (0804, 2026-10-16), Nick changed it:

> "for this first ch1 fight probably the obj should be changed to just
> kill harl rather than all enemies cuz a straggler guard is not that
> important he should just run away after harl dies"

Core already has the kind: `Objective::DefeatUnit { unit, turn_limit }`
(`crates/core/src/battle.rs:448-454`), written
`DefeatUnit(unit: "red_captain")` (`assets/battles/README.md:65`). Harl is
`red_captain` at (20, 2), `ai: Stationary`, `boss: true`
(`ch01.ron:43`).

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Win by defeating Harl with enemies left; does what the
> leftovers do feel right? Also confirm the starting rules below.

## Scope

**In:**
- `ch01.ron`: `objective: DefeatUnit(unit: "red_captain")`. The
  objective page and the battle's opening objective line say so (text
  ids, ADR-0045).
- **The rest run** (*the agent's starting rules*, listed in the Completion notes):
  - when Harl falls, the battle is won at once; the enemies left on the
    map are shown leaving: each walks toward the nearest map edge at map
    speed and fades out, one short moment (≤ 2 s together; Confirm skips),
    then the victory scenes play as today;
  - they give no EXP and drop nothing (they ran);
  - the results screen counts them as fled, not defeated (if the results
    screen lists enemies at all).
- Update `chapter-1.md` (*Objective and loss*) with Nick's words, and the
  battle's beat in `docs/story/chapters/ch01.md` only if it says "rout"
  (a story-text change, so just the objective word).
- The tutorial's lessons still fit (Attacking, Ending the turn, etc.);
  check nothing assumes Rout.
- Re-run the playtest bots (`cargo xtask playtest ch01`,
  `docs/playtesting.md`) and put the win rate and turns before/after in
  the completion notes. The bots' win check must use the objective.
- The title cinematic doesn't show the objective; no change there.

**Out (do not do):** Harl's stats or AI; the script (0737 writes any
line about the Red Company scattering, if it wants one).

## Implementation steps

1. Change the objective; update the objective text.
2. Add the "leftovers leave" moment after a `DefeatUnit` win (a battle
   option, e.g. `leftovers: Flee`, so other battles can choose; say the
   field's name in `assets/battles/README.md`). Map units go through the
   map scene and every skin (ADR-0038); tests read the scene.
3. EXP/results for fled units.
4. Bots and docs.

## Acceptance criteria

- [x] Chapter 1 is won when Harl falls, with enemies left (core test).
- [x] The leftovers walk off and fade in both skins (scene test); Confirm
      skips it.
- [x] No EXP for fled units (test).
- [x] Bot numbers before/after in the completion notes.
- [x] `chapter-1.md` updated.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (core): `DefeatUnit` victory with units left.
- Integration: Chapter 1 in the battle harness, Harl defeated first.
- Snapshot: one frame of the leftovers leaving.

## Completion notes

**Done.** `ch01.ron` is now `objective: DefeatUnit(unit: "red_captain",
leftovers: Flee)`. The objective page already said `Defeat Harl` for a
`DefeatUnit` (`battle.objective.defeat`); there is no other opening
objective line in Chapter 1 (no battle notes, and the tips and lessons
never mention Rout), so no text id changed.

- **Core:** `Objective::DefeatUnit` gained `leftovers: Leftovers` (`Stay`,
  the default, or `Flee`; old saves read as `Stay`). On a won battle with
  `Flee`, the enemies still on the map move into `BattleState::fled()`
  and `Event::Fled { units }` comes just before `BattleEnded`. They are
  not among the fallen, give no EXP and drop nothing. Green units and
  units that are down stay.
- **Screen:** `Event::Fled` queues a `Flight` (`screens/battle/flight.rs`)
  before the victory banner. Until it is on screen (Harl's fall, his
  death lines and the EXP bar play first), the leftovers are drawn
  standing where they were; then each walks straight to its nearest map
  edge at the map skin's walking speed and fades, all together, in 1.5 s.
  Confirm skips it. It is built from `UnitView`s in the map scene (walk
  offset, facing, frame, fade), which every skin already paints
  (ADR-0038).
- **Results:** the results screen lists no enemies, so nothing to show
  there; the state counts the leftovers as fled, not defeated.
- **Bots:** they already read the objective (win check, progress and
  goals: Harl's HP and position for `DefeatUnit`). Before → after
  (`cargo xtask playtest ch01`, 100 tries each, seeds 1–100):

  | Bot | Before (Rout) | After (defeat Harl) |
  | --- | --- | --- |
  | baseline | won 27/100, turns median 7 | won 28/100, turns median 7 |
  | casual | 51/100 succeed (band 30–55% ✓), turns median 28 | 36/100 succeed (✓), turns median 21 |
  | normal | 47/100 succeed (band 50–75% ✗ too hard), won 86/100, turns median 10 | 63/100 succeed (✓), won 90/100, turns median 9 |
  | hardcore | 25/100 succeed (band 50–75% ✗ too hard), won 46/100, turns median 7 | 49/100 succeed (✗ too hard, just), won 81/100, turns median 8 |

  Casual now heads for Harl and wins less often (51 → 36, still in its
  band); Normal moves into its band and Hardcore nearly doubles. Ticket
  0855 (Chapter 1 back in its bands after the armour weights) should
  start from these numbers.
- **Docs:** `chapter-1.md` (*Objective and loss*), `docs/story/chapters/
  ch01.md` (the objective word and why it changed),
  `assets/battles/README.md` (the `leftovers` field).
- **Tests:** core (won with enemies left and they stay by default; with
  `Flee` they leave and `Fled` comes before `BattleEnded`; no EXP for
  them), content (Chapter 1's objective; `leftovers` defaults to `Stay`),
  bots integration (Chapter 1 won on turn 1 by striking Harl first, 13
  fled), screen (the leftovers walk toward the edge and fade in the glyph
  and sprite skins, then the victory banner; Confirm skips; a snapshot of
  one frame).
- **Engine friction:** estimate 160 lines, measured 166 (1.0×), under 2×.

**The script still says Rout.** The pre-battle scene ends with the
Sergeant's "Any of them who gets away comes back tonight with torches. So
it has to be all of them." (`assets/dialogue/ch01.dlg`). Out of scope
here; 0737 (the Chapter 1 script) writes any line about the company
scattering and should change that one.

**the agent's starting rules** (for the Act 1 playtest, 1064):

1. When Harl falls, the battle is won at once, with no extra turn.
2. The rest of the Red Company stands still while Harl's fall, his death
   lines and the EXP bar play, then all of them run together.
3. Each walks in a straight line to its nearest map edge (ignoring
   terrain), at the same speed units walk on the map, fading out over
   1.5 seconds; one already on an edge fades where it stands. Pressing
   Confirm skips it.
4. They give no EXP, drop nothing and count as fled, not defeated.
5. Then the victory banner and the victory scenes, as before.
