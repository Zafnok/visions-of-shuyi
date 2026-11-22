---
id: "1314"
title: "Each target has a graphics tier, and every moving look a still fallback"
type: infra
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
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
- [ ] A test: the Playdate target reports `Reduced` (or the chosen name),
      desktop and web `Full`.
- [ ] A test: an animated tile row with no fallback fails the check; one
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

