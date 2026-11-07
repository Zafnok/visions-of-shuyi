---
id: "1248"
title: The Playdate's one-bit drawing within a frame's budget
type: infra
milestone: Later
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1247"]
nick_input: none
completed:
---

# 1248 — The Playdate's one-bit drawing within a frame's budget

## Context

`cargo shuyi bench` (ticket 1244, `docs/targets.md`) plays each scenario
on the Playdate with every frame drawn in one bit, as its build draws it
(the engine's `shuyi_ui::frame::onebit`, ADR-0074). Against the
Playdate's budget of 2 million instructions a frame (provisional, the
engine profile's), PR #558's first run found:

| Scenario | Costliest frame | Mean | Of that, one-bit drawing |
| --- | ---: | ---: | --- |
| battle-idle | 1.89 M | 1.88 M | about 1 M a frame (`onebit::Target::run`, `draw_glyph`, `render_marked_with`) |
| battle-cursor-sweep | 3.11 M | 1.91 M | as idle, plus the cursor's reach (`movement::reachable`, `attack_tiles`) |
| battle-enemy-turn | 4.53 M | 1.97 M | as idle, plus the enemy's choice (`ai::Planner::decide`, `reachable`) |
| dialogue-portrait | 4.74 M | 2.44 M | 2.7 M in `onebit::Columns::bits`, drawing the lead's portrait |
| options-menu | 1.88 M | 0.84 M | (within) |

The colour targets' runs of the same frames are 1.1 M (idle), 2.2 M,
3.7 M and 2.1 M: the one-bit drawing alone takes about half the
Playdate's budget on every frame, and a picture drawn in one bit (the
portrait) takes more than the budget by itself. A count of x86-64
instructions is a proxy for the device's (engine ADR-0081); engine
tickets 3011 and 3013 measure the device and may set another budget, but
a budget only moves with a device measurement.

## Nick input

None.

## Scope

**In:**
- Run the `engine-friction` skill on the finding ("a performance budget
  missed because of the engine"). The drawing is the engine's, so the
  fix is likely an engine change (the `engine-change` skill): a picture's
  one-bit columns made once and kept, not per frame; the unchanged
  rows of a frame not drawn again; or what the review finds.
- The game's own share: the battle's per-frame reach and AI work in the
  cursor-sweep and enemy-turn frames, if it is still over after the
  drawing's fix (a ticket of its own if it is big). The cursor's threat
  arrows, a path search per enemy on each cursor move, are 1251's.
- Every Playdate scenario's costliest frame under 2,000,000 instructions,
  or a ticket for each that isn't, naming what is slow.

**Out (do not do):**
- Raising the budget (only a device measurement changes one).
- The unit menu's spike (1247, first).

## Implementation steps

1. Read the Callgrind dumps of the `bench-report` artifact of the latest
   `bench` run (`runs/<scenario>-playdate/callgrind.out.<frame + 1>`).
2. Run `engine-friction`; follow its outcome (`engine-change` for an
   engine fix, then move the pin).
3. Re-run the bench and record the numbers in `docs/targets.md`.

## Acceptance criteria

- [ ] The `engine-friction` review is recorded (`docs/engine/friction.md`).
- [ ] Every Playdate scenario's costliest frame is under 2,000,000
      instructions, or each one over has a ticket naming what is slow.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- CI: the `bench` job's table.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
