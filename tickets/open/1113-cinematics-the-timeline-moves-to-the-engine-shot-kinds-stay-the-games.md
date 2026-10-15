---
id: "1113"
title: "Cinematics: the timeline, easing and backdrop pan and zoom move to the engine; shot kinds are registered by the game"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1112"]
nick_input: none
completed:
---

# 1113 — Cinematics: the timeline moves; shot kinds stay the game's

## Context

ADR-0060: a cinematic is a file of timed shots, each frame drawn from
the time alone. The timeline (shots at milliseconds, a length from a
music cue or seconds, `Timeline::at_ms`, easing) is generic, but the
shot kinds name battles, characters and the combat rules (`MapPan`,
`Characters`, `Talk`, `Combat`, `Logo`; `content/src/cinematic.rs:73-158`,
`ui/src/cinematic*`). `docs/engine/boundary.md`, *content* and *ui*.

## Nick input

None. The title cinematic must play identically (compare frames).

## Scope

**In:**
- `shuyi_assets::cinematic`: the file's frame (`id`, `length`, `shots:
  [(at, shot)]`, ordering checks) generic over a shot type `S:
  DeserializeOwned + ShotCheck`, so the game's `Shot` enum plugs in and
  keeps its RON format.
- `shuyi_ui::cinematic`: `Timeline`, `ShotAt`, easing, the player
  skeleton (`CinematicPlayer<S>` calling a `ShotPainter` trait the game
  implements), brightness fades, `cinema::view` framing.
- The backdrop pan and zoom (ADR-0048) are already in the frame (1108).
- The game keeps every shot kind and its painter.

**Playdate:** the Playdate port (1025's tickets, 1027–1053) builds
`core`, `content` and `ui` without `std` and adds a second front end,
`crates/playdate`. If those tickets have landed, code this ticket moves
keeps its default `std` feature and stays in the `no-std` CI build, and
`cargo xtask playdate` keeps building (ADR-0070 §6). 1044's Playdate paint of the title cinematic's shots stays the game's.

**Out (do not do):** new shot kinds; `MapPan` into the tactics kit
(stays the game's until 1115 offers a generic grid map; then it may move,
in 1115's discretion).

## Implementation steps

1. Make the timeline and player generic in place; the game's `Shot`
   implements the traits.
2. Move.
3. Render the title cinematic's frames at 0, 1, 5, 10 s with `frame-png`
   before and after; they must be byte-identical.

## Acceptance criteria

- [ ] An engine test plays a toy cinematic with two toy shot kinds.
- [ ] The title cinematic's frames are byte-identical before and after.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: timeline math (moved).
- Integration: toy cinematic; title frames compared.

## Completion notes
