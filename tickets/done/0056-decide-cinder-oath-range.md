---
id: "0056"
title: "Decide: Cinder Oath's range and what it is"
type: design-decision
milestone: Design decisions
model: reasoning
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-15
---

# 0056 — Decide: Cinder Oath's range

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14): "on special skill Rue's
skill is range 1 really? seems a bit weird what is the flavor here? why
should mage special skill be range 1?" Asked about the signature spells:
"seem ok other than cinder oath concern I raised" (so Maud's Vigil Light
stays as it is).

Today (`assets/data/spells.ron`; `magic.md`, *Personal signature spells*;
starting numbers by 0803, not decided by Nick): Cinder Oath is 9 might, 80
hit, **range 1**, 3 uses a battle, burns a forest tile. Fire is 5 / 90 /
range 1–2 / 10 uses.

Where range 1 came from: Rue's sheet (`docs/story/characters/heretic.md`,
*Signature spells*): "The vow she swore at the Ashfields. A close-range fire
strike (range 1) with high might and few uses." She swore it standing in
the fire with burned hands; her Act 2 spell, **Kept Promise**, is the same
vow sworn anew, "a longer-range fire attack (range 1–3) with lower hit",
so the range growing is meant as her story arc (close and desperate, then
reaching). The game cost is that at range 1 a frail mage takes the
counter-attack. Nobody asked Nick about the range itself.

## Nick input

**Decision**, with the `ask-nick` skill. Explain the flavor above in two
sentences, then ask, with real games for each:

- **A. Keep range 1** with the flavor made visible (she grabs the enemy
  with her burned hands; the description says so): a high-risk close
  spell.
- **B. Range 1–2 like Fire**, stronger and fewer uses.
- **C. Range 1, but no counter-attack** (the enemy can't strike back at
  the oath), keeping the close flavor without punishing her.
- **D. Range 2 only** (she can't use it next to an enemy).
- **E.** "Describe your own."

Sub-question if the range changes: keep Kept Promise (Act 2) at 1–3, or
change it so the two still differ?

Give each option an example from a real game (the `ask-nick` skill wants
them), and check every one before putting it in front of Nick: a wrong
comparison is worse than none.

## Scope

**In:**
- The question and Nick's answer, recorded in `magic.md` (*Personal
  signature spells*) and Rue's sheet.
- The data change in `assets/data/spells.ron` (and a no-counter rule only
  if C is picked and the engine already has one; otherwise a follow-up
  ticket).
- Re-run the Chapter 1 difficulty sim (`cargo xtask playtest ch01 --runs
  200`) and the winning-replay tests; if the replay breaks, re-record it
  as `crates/ui/tests/it/ch01_winnable.rs` says.

**Out (do not do):**
- Vigil Light (Nick is happy with it).
- New engine rules beyond data (a follow-up ticket instead).

## Implementation steps

1. Ask Nick; record his words.
2. Change the data; update the description text if the spell has one.
3. Run the gates, the sim and the replay tests; write the result in the
   Completion notes.
4. If the answer needs code, write the follow-up ticket and add it to
   0804's `blocked_by`.

## Acceptance criteria

- [x] `magic.md` and `heretic.md` record the answer and Nick's words.
- [x] `spells.ron` matches it.
- [x] Sim result written in the Completion notes; replay tests pass.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Existing content tests and the Chapter 1 replay tests.

## Completion notes

Asked Nick (2026-10-15) with the flavor and the four options (Into the
Breach's Flame Mech, GBA Elfire/Bolganone, Three Houses' Windsweep, GBA
bows). Nick: "C, but only because of flavor, and it should get upgraded
later in the game after some personal arc or a support level increase".
On what triggers the upgrade: "we'll iterate it later, it might be a story
beat, or a support level, but until we have ch1 playtest done and other P0
items then this is not a conversation worth having".

- Recorded in `magic.md` (*Personal signature spells* → *Cinder Oath: range
  1, no counter*, and the open-questions list) and in Rue's sheet
  (`heretic.md`): Cinder Oath stays range 1 and the enemy can't counter
  it; Kept Promise (1–3) is its later upgrade, trigger open.
- The engine has no-counter only for arts, not spells, so the rule is
  follow-up **0317** (attack spells the enemy can't counter), added to
  0804's `blocked_by` and the roadmap. `spells.ron` keeps its numbers (no
  field to set yet); only its comment changed.
- **Sim not re-run:** no number in the data changed, so the Chapter 1 sim
  and the winning replay play exactly as before. 0317 re-runs both when
  the rule changes play. The full test suite (including the replay tests)
  passes.
- *the agent's starting rule:* "no counter" covers only Rue's own attack. If
  Cinder Oath is her equipped attack and an enemy attacks her from the
  next tile, she still counters with it, as any attack spell does.
