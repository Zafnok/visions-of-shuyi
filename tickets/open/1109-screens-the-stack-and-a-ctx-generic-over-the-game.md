---
id: "1109"
title: "Screens, the screen stack and the frame driver move to the engine, with a Ctx generic over the game"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: xhigh
status: todo
blocked_by: ["1108"]
nick_input: none
completed:
---

# 1109 — Screens, the stack, the driver; `Ctx<G>`

## Context

`docs/engine/boundary.md`, couplings 2 and 3. `Screen`, `Transition`,
`FrameInput` and `ScreenStack` (`ui/src/screen.rs`, ADR-0017) are
generic in shape, but every screen takes a `Ctx` that holds this game's
`Content`, campaign mode, map skin and lead (`screen.rs:224-327`), and
`Game` (`ui/src/game.rs`) builds this game's title screen, opens the
layout picker on the first key and the debug menu on `Action::Debug`.

ADR-0063 §6: game code keeps its names through aliases, so 40,000 lines
of game screens change field paths, not signatures.

## Nick input

None.

## Scope

**In:**
- `engine_ui::Ctx<G>`: the engine's services (text by key and language,
  keys and help names, settings the engine owns, the audio queue and
  music clock, voice, storage) plus `pub game: G`. `trpg-ui` defines
  `pub struct Shuyi { content, campaign_mode, mode_switch, map_skin,
  lead, … }` and `pub type Ctx = engine_ui::Ctx<Shuyi>`; it implements
  `Deref`-free accessors only where many call sites need them.
- `engine_ui::screen`: `Screen<G, A>` (or `Screen<C>` with `C = Ctx<G>`),
  `Transition`, `FrameInput<A>`, `ScreenStack`.
- `engine_ui::Game<G, A>`: the frame loop and `RawInputEvent` /
  `FrameOutput`; the game gives a root screen and a `GameHooks` value
  (first-key hook, debug hook, fade time). `trpg-ui` keeps `start()` and
  `start_on_scene()` as functions that build the hooks and root.
- The engine-owned part of settings (volumes, fullscreen, text speed,
  theme name, layout name) moves; enemy-phase speed, auto end turn,
  cursor style, map look and zoom and combat animations stay the game's
  (ADR-0050's saved record keeps its format: test with a fixture).

**Out (do not do):** the Harness (1111); widgets and generic screens
(1112); `app` (1110).

## Implementation steps

1. Introduce `Ctx<G>` in place with the alias; move game fields into
   `Shuyi`; fix call sites (mechanical: `ctx.content` → `ctx.game.content`).
2. Make `Game` take the root screen and hooks.
3. Split settings; check the saved settings fixture loads unchanged.
4. Move `screen.rs` (generic part) and `game.rs` (loop) to `engine-ui`.
5. A test in `engine-ui` runs a two-screen toy game (push, pop, quit)
   with a toy `G` and a toy action set.

## Acceptance criteria

- [ ] The toy two-screen test passes in `engine-ui` alone.
- [ ] No snapshot changes; every test passes.
- [ ] A settings file saved by the current `main` loads unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: stack behavior with the toy game.
- Integration: all game Harness tests unchanged.

## Completion notes
