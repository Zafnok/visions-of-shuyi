---
id: "0319"
title: "Core rules: an escape objective, a fleeing boss, a boss who can't die, and a scripted death"
type: feature
milestone: M2 Core rules
model: opus-5.5
effort: high
status: todo
blocked_by: ["0069"]
nick_input: answer-first
completed:
---

# 0319 — Escape, a fleeing boss, a boss who can't die, a scripted death

## Context

Act 1's story battles need four rules `core` doesn't have
(`docs/story/outline.md`):

- **Chapter 6, the Ashfields:** reach the Brennish lines (an escape
  objective).
- **Chapter 4, Veyne Hall:** Dace "escapes at low HP".
- **Chapter 8, Saltmere Docks:** Dace "can't be killed; he retreats".
- **Chapter 8:** Hollis dies holding the gangway, in every mode; in
  Classic, if he already fell, the lead tells Dace instead.

0069 asks Nick how each works and records the answers in the design
docs. This ticket builds them in `core`, as data the battle files can
use, so the chapter tickets (0850, 0852, 0854) only write data.

`core` today has the Rout, DefeatUnit, Seize and Survive objectives,
reinforcements, battle scene triggers (turn start, map areas, falls,
talk), and Classic/Casual death and retreat (`death-and-difficulty.md`).

## Nick input

**Answer first:** 0069. Don't build a rule its answers don't settle; if
one is left open, ask about it with `ask-nick` before writing code.

## Scope

**In** (each as 0069 decided; the names below are placeholders):
- An **escape objective** (`Objective::Escape { area, .. }` or as 0069
  shapes it): which units must reach the area, what happens to a unit
  that reaches it (leaves the map, safe) and to one that doesn't, any
  turn limit; win and loss checks; the objective line in the battle UI
  and the battle notes says it.
- A **fleeing boss**: a unit setting that makes it leave the map under
  0069's condition (an HP threshold, or the hit that would kill it), with
  its line (a scene trigger, like a fall line), and how that counts for
  the objective, EXP and the results screen.
- **A unit that can't die**: a unit setting under 0069's rule (HP floor,
  untargetable, or leaving at low HP); the forecast shows what 0069
  decided.
- **A scripted death**: a scene can remove a named player unit for good
  (in every mode, as 0069 decided), and a chapter's scenes can branch on
  whether that unit was still in the army (0715's lines by who is in the
  army may already cover the branch: use them if so).
- The AI respects them: an escaping side moves toward the area if the
  enemy escapes (only if 0069 needs it), a boss that flees or can't die
  still fights by the boss AI.
- The bots (0504's legal moves) and play records handle the new rules
  (an escaping unit's leaving is a recorded event).
- Content validation: an escape area exists on the map; a fleeing or
  unkillable unit is an enemy (or as 0069 says).
- Test battles in `assets-test/` for each rule.

**Out (do not do):**
- The chapters' real battles (0850, 0852, 0854) and scripts (0746,
  0748, 0750).
- Changing how Rout, Seize, Survive or DefeatUnit work.
- New art or UI beyond the objective line, the forecast and the
  results.

## Implementation steps

1. Read 0069's answers (`turn-structure.md`, `death-and-difficulty.md`,
   wherever it recorded them) and `core`'s objectives, falls, scene
   triggers and AI.
2. Rules in `core`, through `Command` → `Event` (ADR-0004); unit
   settings and objectives in the battle file format, documented in
   `assets/battles/README.md`.
3. The objective line, forecast and results in `ui` (follow ADR-0038 for
   anything shown on the map: the escape area goes in the map scene and
   every map skin paints it).
4. Test battles and tests; the bots run them (`cargo xtask playtest
   <test battle> --bot all`) without a softlock.

## Acceptance criteria

- [ ] Each rule works as 0069 recorded it (unit and integration tests
      named after the rule).
- [ ] The forecast against an unkillable boss shows 0069's choice
      (snapshot).
- [ ] The bots finish each test battle; play records replay identically.
- [ ] Chapter 1 plays exactly as before.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: escape win/loss, fleeing at the threshold, the HP floor, the
  scripted removal in Classic and Casual.
- Property: no sequence of legal commands kills an unkillable unit; an
  escaped unit never acts again in that battle.
- Snapshot / integration: the objective line, the forecast, a scripted
  test battle for each rule.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
