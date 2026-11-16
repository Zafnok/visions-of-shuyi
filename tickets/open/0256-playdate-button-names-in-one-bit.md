---
id: "0256"
title: "Decide and build: how the Playdate's button names stand out in its one-bit hints"
type: feature
milestone: Later
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0254"]
nick_input: decision
completed:
---

# 0256 — The Playdate's button names in one bit

## Context

0250 decided the keyboard's look: key names in hints drawn as key caps
(`docs/design/look-and-feel.md`, *Key names in hints*). The Playdate
shows hints in black and white on a 50 × 15 console (ADR-0070, ADR-0079:
`A`, `B`, `A+B`, the crank), and 0254 leaves them as today's text. Nick,
2026-11-16:

> "this probably will need to be changed on console / playdate (console
> using i.e. colored buttons or official icons, and playdate
> consideration for 1-bit screen) -- these can be followup tickets."

**Much later** (Nick, 2026-11-16): "I only want the web/PC keycaps to
block anything substantial, all other systems designs and
implementations can wait for much later". Nothing waits on this ticket.

## Nick input

**Decision** (`ask-nick`), on `cargo shuyi frame-png --playdate`
mockups of the battle's help, the text box corner and a tip. Options:

- A. The key cap in one bit: a black box, the name in white.
- B. Inverted letters with no box.
- C. Brackets (`[A] select`), which take no extra pixels.
- D. Describe your own.

Check each against the Playdate's 50-cell rows first: a cap takes more
cells than its name (engine#3215), so say which hints no longer fit.

## Scope

**In:** the question with mockups; the answer in `look-and-feel.md`; the
build through the engine's one-bit setting of the key-cap style
(engine#3215) or an engine ticket if it lacks what Nick picks.

**Out (do not do):** the PC keyboard (0254) and controllers (0255).

## Implementation steps

1. Mockups with `--playdate`; ask Nick.
2. Record the answer; build it; check every Playdate hint still fits.

## Acceptance criteria

- [ ] Nick picked; recorded in `look-and-feel.md`.
- [ ] The Playdate's hints show the picked look (`frame-png --playdate`
      pictures in the PR) and every help line fits its row.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: the Playdate's battle help, text box corner and a tip.

## Completion notes

*(Filled in by the session that completes the ticket.)*
