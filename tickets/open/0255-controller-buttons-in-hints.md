---
id: "0255"
title: "Decide and build: how a controller's buttons look in hints (coloured buttons or the platform's icons)"
type: feature
milestone: Later
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0254"]
nick_input: decision
completed:
---

# 0255 — How a controller's buttons look in hints

## Context

0250 decided the keyboard's look: key names in hints drawn as key caps
(`docs/design/look-and-feel.md`, *Key names in hints*). With a controller
in use, hints name buttons (`A`, `Options`, `D-pad/L-stick`, from
`PadKind::button_name`, ADR-0053), and 0254 leaves them as today's text.
Nick, 2026-11-16, on the key-cap mockups:

> "this probably will need to be changed on console / playdate (console
> using i.e. colored buttons or official icons, and playdate
> consideration for 1-bit screen) -- these can be followup tickets."

**Much later** (Nick, 2026-11-16): "I only want the web/PC keycaps to
block anything substantial, all other systems designs and
implementations can wait for much later". Nothing waits on this ticket.

## Nick input

**Decision** (`ask-nick`), on rendered mockups of the battle help bar,
the text box corner and a tip, for each pad kind the game knows
(generic, Xbox, PlayStation, Switch). Options to draw, each checked
against real games before citing:

- A. The key cap, holding the button's name (`A`, `Options`).
- B. Round face buttons in the platform's colours (Xbox's green A, red
  B, blue X, yellow Y; PlayStation's shapes), the D-pad and sticks as
  small pictures.
- C. The platform's official button icons, if their terms allow it in a
  game sold on itch.io and Steam (check each platform's terms; say which
  don't, and that a console build has its own rules).
- D. Describe your own.

## Scope

**In:** the question with mockups; the answer in `look-and-feel.md`; the
build: button pictures as sprite items from asset files (ADR-0038), each
new asset's license checked against ADR-0013 and credited; an engine
ticket first if the engine's styled text can't show a button picture
(`engine-change`).

**Out (do not do):** the Playdate's buttons (0256); the keyboard's look
(0254); rebinding.

## Implementation steps

1. Mockups on real frames (`cargo shuyi frame-png`, with a pad:
   `--pad`), both looks; ask Nick.
2. Record the answer; write the engine ticket if needed.
3. Build it on 0254's styled text.

## Acceptance criteria

- [ ] Nick picked; recorded in `look-and-feel.md`.
- [ ] Each pad kind's hints show the picked look (snapshots reviewed).
- [ ] `the_longest_help_bar_fits_on_every_pad` still passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot: help bar, text box corner and a tip for each pad kind.

## Completion notes

*(Filled in by the session that completes the ticket.)*
