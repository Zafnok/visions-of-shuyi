---
id: "1316"
title: "Drifting petals and leaves over the map"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: blocked
blocked_by: ["0096", "1314"]
nick_input: answer-first
completed:
---

# 1316 — Drifting petals and leaves over the map

## Context

Nick (2026-11-22): "that effect Stardew Valley has for like petals
flowing on screen, we should have some effects similar." 0096 decides
which effects, where, and how dense; 1314 gives targets a graphics tier.

The engine has no particles yet: parked engine tickets 2103 (particles:
emitters as data, seed plus time) and 2370 (weather ambiences: particles,
an overlay and a loop sound). Motion is computed from the time alone
(`shuyi-animate` skill); the rules never see it (`core` knows nothing of
the look, ADR-0038), so play records and bots are unchanged.

**From 1314 (done, ADR-0131):** a skin reads the tier from
`ctx.platform.graphics` (`GraphicsTier`; the Playdate is `Reduced`).
Each effect carries a `trpg_content::motion::Motion`: `lowest: Full` and
`stills: {Reduced: (show: Nothing), Minimal: (show: Nothing)}`, with
`todo: "..."` where 0096 wants a lighter Playdate version. Hand the
effects to `motion::check` in the content test
`every_moving_look_names_its_stills` (its `TODO(1316)`), beside 1315's
tile rows. The particle cap
per tier, if one is needed, is a field of the effect's data (ADR-0131
Consequences), not a fourth tier. Preview with `cargo shuyi frame-png
--graphics reduced`.

## Nick input

**Answer first:** 0096. Nothing to check after merge: verified by tests,
`frame-png` pictures and the bench; what's left to judge in play goes on
1064's checklist.

## Scope

**In:**
- Engine (`engine-change`): particles as data, each particle's place a
  pure function of seed and time, drawn as sprite items from an asset
  file; a cap on count from the target's tier (1314).
- Game: an effect file per kind 0096 chose (petals, leaves…) in
  `assets/` (with the art from the bundle or a credited free pack, never
  drawn per pixel), and a map's chapter data naming its effect
  (`content-change` for the private chapters).
- The map skin draws it over the terrain and under the UI; it pauses with
  idle motion off.
- Fallback: none on `Reduced` and lower tiers, with a `TODO` if 0096
  wants a lighter Playdate version.

**Out (do not do):** weather that changes rules (rain lowering hit);
sounds (1332); effects on screens 0096 didn't name; bursts on hits
(1333) and touches and creatures on the map (1334), which use this
ticket's engine particles; each chapter's pick (1335–1342).

## Implementation steps

1. Engine particles (or confirm at the pin), with tests at fixed times.
2. Effect data format in the game; one effect per 0096's answer.
3. Chapter data's `effect` field; Chapter 1's set per 0096.
4. Skin drawing; tier cap; idle motion.
5. `frame-png` strips; `cargo shuyi bench` on desktop and web.

## Acceptance criteria

- [ ] A test: the same seed and time give the same particles.
- [ ] A test: the Playdate tier draws no particles.
- [ ] Chapter 1's map shows its effect in `frame-png` strips in the PR.
- [ ] `cargo shuyi bench` within budget on desktop and web.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: particle positions from seed and time; effect data loads.
- Look: the skin's sprite items at fixed times.

## Lesser targets

Off below `Full` (1314). GBA: no target.

## Completion notes

