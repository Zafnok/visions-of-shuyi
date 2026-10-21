---
id: "0469"
title: "Guides show a small picture of what each topic explains"
type: feature
milestone: M3 Battle UI
model: opus-5.5
effort: high
status: todo
blocked_by: []
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
