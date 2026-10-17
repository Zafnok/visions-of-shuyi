---
id: "1041"
title: "Playdate: the simple one-bit combat picture"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1039", "1040", "1061", "0062"]
nick_input: sign-off
completed:
---

# 1041 — The Playdate combat picture

## Context

Nick kept "a simple one-bit combat picture" on the Playdate
(`docs/design/playdate.md`, Q5). 0062 decides what "simple" looks like
(read its answer in the design doc's *One-bit pictures* section first;
if it changed this ticket's scope, 0062's session updated it).

The combat scene is already a view painted by a skin
(`crates/ui/src/screens/battle/stage.rs`, `stage/view.rs`, `stage/glyph*`;
ADR-0056, `docs/design/look-and-feel.md`, *Combat screen*): each fighter
is one still, full-body picture moved by the game, over a battle
background. On the Playdate it is a new paint of the same stage view,
with one-bit pictures from 1040. 1039 painted the fight box and the rest
of the fight's windows.

## Nick input

**Sign-off:** Nick watches a fight in the Simulator or on his Playdate
(e.g. the lead against the first brigand) and comments.

## Scope

**In:**
- A Playdate paint of the stage view, chosen by `SkinSet` (1037), at
  400 × 240: the two fighters and background as 0062 decided
  (`playdate.md`, *One-bit pictures*: fighters in style A on plain white
  ground with a 2-pixel white rim, the dithered backdrop only above a
  horizon line; the portrait style setting doesn't change them), the HP
  and strike numbers the PC combat screen shows, the hit, miss and
  crit moments the PC shows, at the PC's timing.
- The pictures it needs converted by 1040's tool (battlers and
  battlebacks of Chapter 1), with the method 0062 chose.
- The Playdate bundle (1036) lists them.
- Look tests with the skin; behaviour tests read the stage view.
- **Measure on the device:** frame time during a fight (the motion
  redraws every frame); the PR lists it.

**Out (do not do):**
- New combat animations or effects the PC doesn't have.
- Changing the PC combat scene.

**Carve:** no carve ticket touches the stage; follow where the frame
went (1108).

## Implementation steps

1. Read 0062's answer, `stage.rs`, `stage/view.rs`, `stage/glyph.rs`.
2. The Playdate paint and its tests.
3. Convert and bundle Chapter 1's battlers and battlebacks.
4. Simulator, then ask Nick to sideload for the frame time.

## Measured on Nick's Playdate

Frame time during a fight (min/avg/max) and the stack high-water mark,
from the Simulator console while the device runs a fight.

## Acceptance criteria

- [ ] A fight in Chapter 1 shows the one-bit combat picture in the
      Simulator, at the PC's timing.
- [ ] Device frame time is in the PR; the fight doesn't stutter (or a
      follow-up ticket says what to cut).
- [ ] No PC snapshot changes.
- [ ] Nick's sign-off comment is in the completion notes (or "no comment
      yet").
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): one frame of a strike.
- Integration (harness, Playdate skin set): a fight plays to its end;
  reads the stage view.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
