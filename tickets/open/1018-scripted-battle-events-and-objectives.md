---
id: "1018"
title: "Scripted battle events and objectives for mods"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1017"]
nick_input: none
completed:
---

# 1018 — Scripted battle events and objectives

## Context

A battle file (`assets/battles/README.md`) already holds data for what can
happen: objectives (Rout, Seize, Defeat boss, Survive N turns, within N
turns; `turn-structure.md`), reinforcements on set turns, and dialogue
triggers at moments (turn start, a unit ending its move in an area, a
boss's lines; ADR-0030). Mod makers will want more: "the bridge collapses
on turn 5", "win when both villages are visited", "reinforcements come
when the boss falls below half HP".

1017 gives `core` a deterministic, read-only script runtime. This ticket
lets a battle file name scripts that run **at moments the battle already
has** (phase start, after an action) and return **things the battle
already knows how to do**: spawn listed reinforcements now, play a named
scene, change a tile's terrain (as terrain magic already does), declare a
win or a loss. Exact list: 1011's ADR.

## Nick input

None.

## Scope

**In:**
- Battle files may list `scripts: [(moment: PhaseStart | AfterAction,
  script: "...")]`.
- Each script sees the same read-only view as 1017 plus the battle's
  objective state, and returns a list of outcomes from a closed set.
- Outcomes become ordinary `Event`s through existing code paths, so
  playback, rewind and saves need nothing new.
- A custom objective: `objective: Script("...")` whose script returns
  win / lose / not yet, checked where objectives are checked today.

**Out (do not do):** new kinds of outcome that need new rules (each is its
own ticket); scripts in the shipped game's battles; the map editor's UI
for scripts (1020 only names them).

## Implementation steps

1. Read ADR-0030, ADR-0035, `turn-structure.md`, 1017's code and the
   battle's objective checks in `crates/core/src/battle*`.
2. Add the battle-file fields to `trpg-content`'s battle loader, with
   validation (script exists, compiles, moment is known).
3. Run the scripts at their moments in `core`, turning outcomes into
   events. Order: objectives as today, then scripts in file order (write it
   in the ADR's table if it isn't there).
4. Test mod battles: a bridge that turns to water on turn 3, and a "visit
   both villages" objective.

## Acceptance criteria

- [ ] The bridge test battle: on turn 3 the bridge tile becomes water and a
      unit on it is handled as the existing terrain rules say (test).
- [ ] The two-villages objective wins exactly when the second village is
      visited (test).
- [ ] Rewinding past a scripted outcome undoes it; replaying the same
      commands repeats it (test).
- [ ] Bad scripts or moments are content errors with file and line (test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: outcome → event conversion.
- Integration: the test battles, with rewind and save/load.

## Completion notes
