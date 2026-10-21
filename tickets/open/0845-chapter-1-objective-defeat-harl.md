---
id: "0845"
title: "Chapter 1's objective becomes defeating Red Harl; the rest of the Red Company runs"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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
- **The rest run** (*Claude's starting rules*, listed in the Completion notes):
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

- [ ] Chapter 1 is won when Harl falls, with enemies left (core test).
- [ ] The leftovers walk off and fade in both skins (scene test); Confirm
      skips it.
- [ ] No EXP for fled units (test).
- [ ] Bot numbers before/after in the completion notes.
- [ ] `chapter-1.md` updated.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (core): `DefeatUnit` victory with units left.
- Integration: Chapter 1 in the battle harness, Harl defeated first.
- Snapshot: one frame of the leftovers leaving.

## Completion notes

*(Filled in by the session that completes the ticket.)*
