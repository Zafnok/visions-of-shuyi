---
id: "0847"
title: "A better default keyboard key for the dialogue log"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5
effort: low
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0847 — A better default key for the dialogue log

## Context

The dialogue log (0729) opens with `DialogueLog`: `b` in the right-handed
layout, `n` left-handed, the left face button on a controller
(`assets/data/keymap.ron`; `docs/design/controls.md`, *Dialogue log*).
The keys were *Claude's starting rules*; the controller button came from
a PS4 guide for Persona 5. `DialogueLog` is in its own key group, so it
may share a key with an action of the rest.

Chapter 1 playtest round 3 (0804, 2026-10-16):

> "the log is well implemented overall. I just don't like the default key
> on keyboard. Let's see what visual novels or persona use."

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision** (`ask-nick`). Research first, then options that fit both
layouts:

1. What PC visual novels and Persona use for the backlog by default
   (check each: e.g. Ren'Py's default, Steins;Gate, Danganronpa, the
   Ace Attorney trilogy on PC, Persona 4 Golden / 5 Royal on PC, and
   mouse-wheel-up, which many VNs use). List each with its key.
2. For each candidate, whether it's free in both layouts (right-handed:
   arrows + A S D F row; left-handed: WASD + J K L ; row) or would share
   with another action (allowed, since `DialogueLog` is its own group,
   but say what it shares with).
3. Ask Nick to pick per layout. Also ask: should the mouse wheel open it
   on builds with a mouse? (The game is keyboard-first; a yes is a new
   input, so it would be its own ticket.)

## Scope

**In:** the research; the question; changing `keymap.ron`'s defaults;
`controls.md`; the Key bindings screen's defaults test.

**Out (do not do):** the controller button (Nick only raised the
keyboard); mouse input.

## Acceptance criteria

- [ ] The research table and Nick's pick are in `controls.md`.
- [ ] `keymap.ron` has the new defaults; the keymap tests and
      `cargo xtask check-keys` pass.
- [ ] Players who already rebound the log keep their binding (the
      profile's saved keys win).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the default keymap has the new chord in each layout, no clash
  inside its key group.

## Completion notes

*(Filled in by the session that completes the ticket.)*
