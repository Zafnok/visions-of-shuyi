---
id: "1113"
title: "Cinematics: the timeline, easing and backdrop pan and zoom move to the engine; shot kinds are registered by the game"
type: infra
milestone: "Engine: carve"
model: reasoning
effort: high
status: done
blocked_by: ["1112"]
nick_input: none
completed: 2026-10-17
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

- [x] An engine test plays a toy cinematic with two toy shot kinds.
- [x] The title cinematic's frames are byte-identical before and after.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: timeline math (moved).
- Integration: toy cinematic; title frames compared.

## Completion notes

- **Engine, assets** (`shuyi_assets::cinematic`): the file's frame,
  `CinematicFile<S, P>` (`id`, `length: Music(cue) | Seconds(n)`,
  `shots: [(at, shot)]`, and the optional `actions` lists shots may
  share), `CinematicDef<S>` / `ShotDef<S>`, and `check`, which reports the
  id, the length and the shots' order and times, then hands each shot to
  the game's `ShotCheck<S>` (and `finish` after the last). `parse`,
  `from_source`, `millis`, `seconds` and `MAX_SECONDS` came with it. The
  game's `CinematicDef` and `ShotDef` are aliases over its `Shot`; its
  `ActionLists` is its `ShotCheck` (each shot's own problems, then its
  action's; unused lists at the end), so every message and its order is
  as before.
- **Engine, ui** (`shuyi_ui::cinematic`): `Timeline`, `ShotAt`, a new
  `Moment` (index, progress, time into the shot, its length, the wrapped
  time: all a shot is told), `millis`, `ease_in_out`, a backdrop `Pan`
  (eased centre, `cinema::view` framing) and `CinematicPlayer<S>`: blank
  screen, the shot's frame and paint, then the brightness fade. Each
  shot kind implements `ShotPainter` (its `Game`, a `Shared` value the
  player is drawn with, and a plain-data `Frame`).
- **Game**: every shot kind and its painter stay (`MapPan` with its
  action, `Logo`, `Talk`, `Characters`, `Combat`). `trpg_ui::cinematic::
  CinematicPlayer` wraps the engine's player with its old API (as 1112
  did for the dialogue player); the lead and the logo's backdrop are its
  `Shared` value. The map pan now holds a `Pan`.
- **Tests**: the timeline and easing tests moved to the engine, with new
  ones for `Moment` and `Pan`; the file's frame tests (id, length, order)
  moved to `shuyi_assets` on a toy game's two shot kinds and shared
  lists. `engine/crates/ui/tests/it/cinematic.rs` reads a toy cinematic
  of two toy shot kinds (a letter, a sweeping bar) on the toy's track and
  plays it: frames by time, the loop, drawing over a stale screen, the
  fade.
- **Frames**: `cargo xtask frame-png` of the title at 31 moments (PC at
  0, 1, 5, 10 s of the menu, the fade at 15.5 s, then every shot kind to
  the logo at 120 and 140 s; and seven `--playdate` frames) are
  byte-identical before and after. The ticket's 0, 1, 5 and 10 s are
  still the menu, which the cinematic only replaces after 15 s idle, so
  the later times were added to cover every shot.
- **Deviation**: the ticket sketched `S: DeserializeOwned + ShotCheck`
  on the shot itself. A game's checks need its tables and what earlier
  shots taught them (a shared list checked once per battle), so
  `ShotCheck<S>` is a checker the game makes per file, and the shot type
  is only `DeserializeOwned`.
- 1044 (the Playdate's paint of the title's shots) hasn't landed; nothing
  in `crates/playdate` names the cinematic. The engine crates keep their
  `no-std` builds.
- No gameplay rules decided. No follow-up tickets.
