---
id: "0832"
title: A cinematic shot that plays the combat scene
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5
effort: high
status: todo
blocked_by: []
nick_input: sign-off
completed:
---

# 0832 — A cinematic shot that plays the combat scene

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14), on the title
cinematic: "the combat scene (2 battlers) is never shown, only some
in-battle-map fighting which also looks a bit odd."

The cinematic format (`crates/content/src/cinematic.rs`, `Shot`; ticket
0817, ADR in the cinematic tickets 0817/0818/0827) has `MapPan` (with
`Walk` / `Strike` actions on the map, 0827), `Talk`, `Characters` and
`Logo`. There is no shot of the **combat scene**: the two battlers with
their full-body pictures over a backdrop (tickets 0413, 0035; built in
`crates/ui/src/screens/battle/stage.rs`, `stage::view(ctx, &Playback, ground,
light) -> Option<StageView>`, painted by `stage/glyph.rs`). In the game the
combat scene plays every attack when combat animations are on.

0833 uses this shot in the title cinematic.

## Nick input

**Sign-off:** watch the title cinematic once 0833 uses it: the fight looks
like the game's own combat scene.

## Scope

**In:**
- A new `Shot` kind (e.g. `Combat { battle, attacker, target, strikes }`),
  named and documented like the others, read from `.ron`, checked by the
  content checks (the units exist in the battle; at least one strike).
- What it shows: the game's own combat scene for one exchange between two
  units of the battle, built by the same code the battle uses
  (`stage::view` from a `Playback` of a combat worked out by `core`, so the
  numbers are real), with the backdrop of the defender's terrain. Its
  outcome comes from the rules, not from the cinematic file; the shot may
  name the RNG seed or the result to show (`hit`/`miss`, HP left) if the
  rules allow a fixed result (pick the simplest honest way and say so in
  the Completion notes).
- How long it lasts: the combat's own length at normal speed; the cinematic
  cuts to the next shot at its `at` as for any shot.
- The cinematic's tests and a snapshot of a frame of the shot.

**Out (do not do):**
- Changing the title cinematic's shots (0833).
- Changing the combat scene itself.

## Implementation steps

1. Read `cinematic.rs` (format), `crates/ui/src/cinematic/` (the player:
   `talk.rs`, `faces.rs`, `action.rs`) and `stage.rs` / `stage/view.rs`.
2. Add the shot kind and its checks in `trpg-content`.
3. Play it in the cinematic player: build the combat from the battle's
   start state with `core` (a `Command` → `Event`s, ADR-0004), make the
   `Playback`, show `stage::view` at the shot's local time.
4. Tests: content parsing and checks; the player shows the stage view at a
   time inside the shot (read the view: both battlers, the striker); a
   snapshot frame.

## Acceptance criteria

- [ ] A `.ron` with the new shot loads; a shot naming a unit not in the
      battle fails the content check (test).
- [ ] Test: half a second into the shot, the view is the combat stage with
      the two named units.
- [ ] Snapshot of a frame (public placeholder art).
- [ ] The title cinematic plays unchanged (its tests pass).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: parsing and checks.
- Snapshot / integration: the player's view in the shot; one snapshot.

## Completion notes

*(Filled in by the session that completes the ticket.)*
