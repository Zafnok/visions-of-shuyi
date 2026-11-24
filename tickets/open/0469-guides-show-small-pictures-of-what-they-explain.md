---
id: "0469"
title: "Guides show a small picture of what each topic explains"
type: feature
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: []
blocks: ["1064"]
nick_input: decision
completed:
---

# 0469 — Guides show a small picture of each topic

## Context

Guides (map menu → Guides, 0458) lists every lesson and tip as text.
Chapter 1 playtest round 3 (0804, 2026-10-16):

> "Also it would be good if the guide had rendered mini screens showing
> what does what. This might not be possible for small screen devices like
> playdate"

Fire Emblem Engage's and Three Houses' tutorial lists show a still image
per topic (check before citing). Nick didn't ask for this before the next
playtest, so it's not in the Act 1 playtest's `blocked_by` (1064).

Pictures must follow ADR-0038: a picture is a rendered frame of the real
game's screen (made from a scene by the skins), not hand-drawn cells; and
bought art shows only in builds that have it (ADR-0040).

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

**Decision** (`ask-nick`) with mockups: still pictures vs short loops;
where in the Guides layout; on the Playdate, none or a smaller one-bit
picture.

**Widened 2026-11-22** (Nick, after watching a new player play Chapter 1:
"We should embed images or sprites into the guides (options menu) and
vix tips where appropriate like for example she said "this is your
cursor" why don't we show the blinking cursor (animated) in the tip bar
itself"): ask also which of Vix's tips and lessons show a picture in the
tip box, where in the box, and which pictures move (the cursor's blink
at least). Since then on the Act 1 critical path (in 1064's
`blocked_by`, Nick's pick, 2026-11-22). 1321 builds the tip box's side and waits for this answer;
the Playdate's version follows 1314's graphics tiers if it is done
(either order: if not, record the Playdate answer and 1321 leaves a
`TODO(1314)`).

## Scope

**In:** the question; a way to render a topic's picture from a small
staged battle scene (a unit, an enemy, the cursor) at build time or run
time; pictures for every lesson and tip; Guides showing them.

**Out (do not do):** new topics.

## Implementation steps

1. Mockups; ask Nick.
2. Stage scenes per topic (data, reusing the battle map scene).
3. Render and show them in Guides' view and skin.

## Acceptance criteria

- [ ] Nick picked; recorded in `tutorial.md`.
- [ ] Every topic has its picture; Guides shows it.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: each topic's staged scene builds (read from the scene).
- Snapshot: Guides with a picture.

## Completion notes

*(Filled in by the session that completes the ticket.)*
