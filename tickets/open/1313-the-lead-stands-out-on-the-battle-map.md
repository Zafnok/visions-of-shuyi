---
id: "1313"
title: "The lead stands out on the battle map"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["0095"]
nick_input: answer-first
completed:
---

# 1313 — The lead stands out on the battle map

## Context

Nick (2026-11-22): the lead's map sprite is bland next to Tamsin's
mounted knight. 0095 decides how the lead stands out (a new sprite, a
mark, a ring, or both). Sprite lookup: `crates/ui/src/map_view/sprite/units.rs`
(around line 192, `lead_m`/`lead_f` first, then the class picture); map
skins paint the map scene (ADR-0038); `is_lord` is in core's unit data.

## Nick input

**Answer first:** 0095. Nothing to check after merge: verified by tests
and `frame-png` pictures; what's left to judge in play goes on 1064's
checklist.

## Scope

**In (per 0095):**
- A new sprite: the `lead_m` and `lead_f` rows in
  `assets-src/tilesets/tiny_tales.ron` (private), re-packed with `cargo
  xtask tileset-import`; credits row if from a new pack.
- A mark or ring: a `lord` flag on the unit in the map scene
  (`crates/ui/src/map_view/scene.rs`, from `is_lord`), drawn by every map
  skin — the sprite skin as a sprite item from an asset file, the glyph
  skin as a glyph or colour, the Playdate skin one-bit.
- Every Act 1 lord gets it (the lead is the only one today).

**Out (do not do):** the combat (duel) picture; boss marks unless 0095
asks for them (else file a ticket).

## Implementation steps

1. The scene field (`lord: bool`) and its test.
2. Each skin's drawing; the asset row for the mark or sprite.
3. `frame-png` pictures of Chapter 1's opening for the PR, all skins.

## Acceptance criteria

- [ ] A scene test: the lead's unit in the map scene is marked as the
      lord; Tamsin's is not.
- [ ] `frame-png` pictures of Chapter 1's opening in the picture look,
      the glyph look and the Playdate skin show the lead as 0095 decided.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the map scene's lord flag. Look tests (cells/sprite items) only
  for each skin's mark.

## Lesser targets

The Playdate skin draws it one-bit (or leaves it out if 0095 says so,
with a `TODO(0095)` naming the asset still needed). No GBA target yet.

## Completion notes

