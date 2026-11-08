---
id: "0484"
title: "A unit's stats are easy to see in battle, as decided in 0078"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["0078"]
nick_input: answer-first
completed:
---

# 0484 — A unit's stats are easy to see in battle

## Context

A friend's playtest of the Act 1 build, passed on by Nick (2026-11-07):
"Make stats easier to see, rather than needing to go to unit info or
leveling up". 0078 asks Nick how; this ticket builds his answer.

The side panel: `crates/ui/src/screens/battle/panel.rs` (`PanelView`,
`PanelUnitView`, `view.rs` ~:151, ~:175), painted by `glyph.rs` (~:277,
~:327) and the Playdate skin (`playdate.rs` ~:193). Stats come from the
unit in core (`Unit`'s stats with the class and equipment, the same
numbers unit info shows: `battle/info.rs`).

## Nick input

**Answer first:** ticket 0078. Nothing to check after merge: verified by
view tests and `frame-png` pictures; anything left to judge by playing
goes on the Act 1 playtest's checklist (1064).

## Scope

**In:** what 0078 records in `look-and-feel.md` (*Side panel*): the view's
new fields (built from the same numbers unit info uses, so they never
disagree), and every skin of it (glyph, picture look, Playdate). A key,
if 0078 picks one: a new `Action` with a default key and a row on the Key
bindings screen (keyboard-input skill: never a hard-coded key).

**Out (do not do):** the forecast (0480); unit info's own screen; panels
on the engine's widgets (1238; if it is done, build inside its panel).

## Implementation steps

1. Read 0078's answer in `look-and-feel.md`; update this ticket's steps if
   0078 didn't.
2. Add the view fields in `panel.rs` / `view.rs`, filled from the unit's
   numbers through the helper unit info uses (share it if it is private).
3. Paint them in each skin; the Playdate skin by its own fit rule.
4. Tests and snapshots.

## Acceptance criteria

- [ ] Hovering a unit gives the panel view the numbers 0078 picked, equal
      to unit info's for that unit (test on an ally and an enemy).
- [ ] Every skin paints them (snapshots; `frame-png` pictures in the PR).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the panel view's numbers against unit info's.
- Snapshot: the side panel, glyph and Playdate.

## Completion notes

*(Filled in by the session that completes the ticket.)*
