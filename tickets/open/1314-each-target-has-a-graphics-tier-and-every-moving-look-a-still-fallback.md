---
id: "1314"
title: "Each target has a graphics tier, and every moving look a still fallback"
type: infra
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: in-progress
blocked_by: []
nick_input: none
completed:
---

# 1314 — Each target has a graphics tier, and every moving look a still fallback

## Context

Nick (2026-11-22), with the tileset overhaul (0096):

> "we must consider our less powerful build targets. I want to deprio
> playdate / gba in general but not forget they exist later. So what this
> means is to build out a fallback / graphics prio system, where we have
> certain toggles for what builds into what. Obviously playdate and
> probably gba cant handle this many animations. So we need some engine
> flags for how to deal w this. And as we build for first class like PC
> (native) and web (itch / playtesting) then we use animated tiles and we
> have the fallback in place already for lesser build targets so the work
> is not massive later [...] The fallbacks dont need to be implemented
> fully just scaffolded wit ha TODO for the asset part"

Today there is no graphics setting per target. The engine's
`PlatformFacts` (shuyi-engine `crates/ui/src/platform_facts.rs`) has the
target, windowing and input facts but nothing about graphics. The game
picks skins with `SkinSet::{Glyph, Playdate}` (`crates/ui/src/screen.rs`,
`Ctx::game.skin_set`, ADR-0083); the player picks `MapLook` (Pictures or
Glyphs, `crates/ui/src/settings.rs`). Targets are listed in `shuyi.ron`
(desktop, web, playdate) and the engine's target profiles
(`targets/*.ron`, e.g. `display.color: OneBit`); budgets are in
`docs/targets.md`. There is no GBA target (the engine has a parked
3022 about measuring one). `ctx.idle_motion` (On/Off) already freezes
looping motion for tests (0093).

This ticket builds the switch before the moving looks exist (1315,
1316), so they are written against it from the start. Nick asked for
engine flags here, which is his OK for this ticket's engine PR (memory:
engine steps need Nick's OK).

## Nick input

None (technical; Nick gave the goal above). Record it as an ADR.

## Scope

**In:**
- **Engine** (`engine-change` skill, an engine ticket closed in the same
  engine PR): a graphics tier or capability set in each target profile
  and in `PlatformFacts`, e.g. `GraphicsTier::{Full, Reduced, Minimal}`
  with what each allows (animated tiles, ambient particles and their
  count, screen effects). The agent decides the shape; names are generic
  (the engine never names a game concept).
- **Game:** desktop and web are `Full`; Playdate `Reduced` (stills;
  maybe a few cheap moving tiles later); a `Minimal` row for a future GBA
  with `TODO(gba)` and no target.
- A rule, checked by a test or lint: every moving look the game ships
  (an animated tile, a particle effect) names its still fallback (frame
  0, or none) for each lower tier. A missing fallback asset is allowed as
  a `TODO` that the check lists, never a crash.
- A debug override to preview a tier on desktop (debug menu or env var),
  so lesser-target looks can be checked with `frame-png` without the
  device.
- A new ADR (the next free number): graphics tiers and fallbacks (which crate decides, how a
  skin asks, how the fallback check runs).

**Out (do not do):** any moving tile or effect itself (1315, 1316); a GBA
target or port; changing the Playdate's look.

## Implementation steps

1. Read the engine's parked tickets 2359, 2360, 2370 and 2103 and fold
   the tier into the one that ships first, or a new engine ticket.
2. Engine PR: the tier in target profiles and `PlatformFacts`, with
   tests; then the game PR moves the pin (`engine-change` §7).
3. Game: set the tier per target; expose it to skins through `Ctx`.
4. The fallback check: a test in `crates/ui` (or a `cargo shuyi lint`
   rule, if the engine offers one) over the tileset's animated rows and
   effect data.
5. The debug override; a `frame-png` of the map at each tier (identical
   today).
6. The ADR with the `write-adr` skill.

## Acceptance criteria

- [ ] Engine ticket closed and the pin moved past it.
- [x] A test: the Playdate target reports `Reduced` (or the chosen name),
      desktop and web `Full`.
- [x] A test: an animated tile row with no fallback fails the check; one
      with `fallback: todo` is listed, not failed.
- [ ] The ADR is merged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: tier per target; the fallback check.
- Engine: the profile field parses; `PlatformFacts` carries it.

## Lesser targets

This ticket is the lesser-targets plan; 1317 makes every later ticket
say how it uses it.

## Completion notes

- **Engine** (engine ticket 3347, its PR Zafnok/shuyi-engine#535):
  `GraphicsTier::{Minimal, Reduced, Full}` (ordered, `Full` the default)
  in each target profile (`graphics:`) and in `PlatformFacts::graphics`,
  with `with_graphics`; the engine's Playdate profile is `Reduced`. A
  debug build of `shuyi-app` shows another tier when `SHUYI_GRAPHICS`
  names one.
- **Game:** both copies of the Playdate's profile say `graphics:
  Reduced`; the desktop and the web take the engine's (`Full`). A skin
  asks `ctx.platform.graphics` (`trpg_ui` re-exports `GraphicsTier`).
  No target is `Minimal`: a `TODO(gba)` beside the tier test, and a row
  in `docs/targets.md`.
- **The fallback:** `trpg_content::motion`: a `Motion` (`lowest`, a
  `stills` entry per lower tier: `show: FirstFrame | Nothing`, and a
  `todo` for a picture still to make), `Motion::drawn(tier)` for the
  skins (a still left out draws frame 0, never a crash), and
  `motion::check`. The content test `every_moving_look_names_its_stills`
  runs it over `motion::moving_looks`, which finds nothing yet
  (`TODO(1315)`, `TODO(1316)`), fails on a problem and prints each to-do.
- **The override:** `cargo shuyi frame-png --graphics reduced|minimal`,
  and `SHUYI_GRAPHICS` for a debug build of the game. A test draws the
  test battle at each tier: the same picture today.
- **ADR-0131.**
- Deviations: the fallback is written `(show: FirstFrame, todo: "…")`
  rather than `fallback: todo`, so a still to make still says what
  stands in for it. The check is a content test, not a `cargo shuyi lint`
  rule (the engine's lint has no game data rules). The tier is not
  folded into a parked engine ticket: 2359, 2360, 2103 and 2370 are all
  blocked, so engine ticket 3347 is new. The override is a variable and a
  `frame-png` option, not a debug-menu row (ADR-0131, Alternatives).
- Handoffs written into 1315, 1316 and 1317 ("From 1314 (done)").
- The pin: the engine PR merges first; moving the pin to it, with its
  section in `docs/engine/pins.md`, is the last step of this PR, in a
  session of its own after Nick merges the engine's.
- The engine's `main` is past the old pin: `crates/app/src/main.rs`'s
  log line now inlines its arguments (macroquad's logging is the `log`
  crate since engine #522, and clippy reads it). The move also takes the
  engine's pack format to version 21 (3347): a pack baked before is
  refused, and the game bakes no pack in the repository.
- Friction: estimate 110 production lines (the `frame-png` `--light`
  option and the tileset's optional fields, ticket 1314's own sizes),
  measured 100 (0.9×), under 2×.
- No gameplay rule was decided: nothing moves yet, and the Playdate's
  look is unchanged.

