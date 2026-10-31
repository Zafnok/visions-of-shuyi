---
id: "1081"
title: "Vix explains the world map the first time the army reaches it"
type: feature
milestone: Post–Chapter 1
model: reasoning
effort: medium
status: todo
blocked_by: ["1007", "1070"]
nick_input: decision
completed:
---

# 1081 — Vix explains the world map the first time

## Context

Nick, while ticket 1007 built the world map (2026-10-29): "we can also
have Vix give a small tutorial the first time hitting the world map".
Vix is the tutorial guide (`docs/design/tutorial.md`, ticket 0731); she
speaks through one-time tips (`assets/data/tips.ron`, `crates/ui/src/tips.rs`)
and checklist lessons (`assets/data/lessons.ron`, 0456), each shown by the
player's tutorial level.

The world map's controls (1007, `docs/design/controls.md` *World map*):
the cursor's keys jump place to place, Confirm travels (battles start on
arrival, towns open on arrival unless Options says not), Previous/Next unit
go through the places, Your army and Find, Cancel opens the menu (Save,
Camp, Options). The symbol plates and level markers are in
`docs/design/look-and-feel.md` *The world map*.

## Nick input

**Decision, before building:** what Vix teaches and how, shown as a
rendered mockup of the world map with her tip (`ask-nick`): one tip, a
short checklist lesson (travel, a town, Find, the menu), or a tip now and
a lesson for the menu later; and which tutorial levels see it. Write the
lines with the `story-writing` skill in her voice (0731). Nick checks
nothing after merge (2026-10-21): frame-png pictures in the PR, and the
feel goes on 1064's checklist.

## Scope

**In:**
- A tip and/or lesson triggered the first time the world map opens in a
  campaign (a new tip trigger if `TipTrigger` has none that fits), shown
  per the tutorial level like the battle's tips.
- Its text in `assets/data/tips.ron` / `lessons.ron`, keys named through
  `{Action}` placeholders (`keyboard-input` skill), never a key.
- Tests: it shows once, on the first world map, at the tutorial levels
  Nick picks.

**Out (do not do):**
- Changes to the world map's rules or controls (1007, 1008).
- Vix explaining skirmishes (1008 may add its own tip).

## Implementation steps

1. Read `tutorial.md`, `crates/ui/src/tips.rs`, `crates/ui/src/screens/
   world_map.rs` and the battle's tip hooks (`screens/battle/tips.rs`).
2. Ask Nick (Nick input) with a mockup on 1070's real map.
3. Add the trigger and the text; show the tip from `WorldMapScreen` on its
   first update when the campaign hasn't seen it.
4. Tests and frame-png pictures.

Expected size: about 80 production lines (a trigger, a hook, its view in
the existing tip box).

## Acceptance criteria

- [ ] Nick's answer is in `docs/design/tutorial.md`.
- [ ] The tip (or lesson) shows the first time the world map opens, and
      never again in that campaign (a Harness test on the test world map).
- [ ] It names keys only through placeholders (`cargo shuyi check-keys`).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the trigger fires once.
- Integration (Harness): New Game on the test world map shows it; a
  world-map save loaded later doesn't.
