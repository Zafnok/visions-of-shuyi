---
id: "1046"
title: "Playdate: preparations, results, saving, game over and the end screens in one bit"
type: feature
milestone: Post–Chapter 1
model: sonnet-5
effort: high
status: todo
blocked_by: ["1037", "1035", "1040"]
nick_input: sign-off
completed:
---

# 1046 — The Playdate's other Chapter 1 screens

## Context

Second batch of Playdate screen paints (ADR-0054; 1045 did New Game and
Options). Chapter 1's flow (`crates/ui/src/flow.rs`, `Stage`) also goes
through:

| Screen | View today |
| ------ | ---------- |
| Preparations (`screens/preparations*`), if Chapter 1's battle has them | `PreparationsView` |
| Results (`screens/results*`) | `ResultsView` |
| Game over (`screens/game_over*`) | `GameOverView` |
| Save prompt and slot picker (`screens/save*`) | views |
| To be continued (`screens/game_over*`) | view |
| Credits (`screens/credits*`) | `CreditsView` |

## Nick input

**Sign-off:** Nick finishes (or loses) Chapter 1's battle and saves, in
the Simulator or on his Playdate, and comments.

## Scope

**In:**
- A Playdate paint for each screen above, one-bit, 50 × 15, in the
  window style 1038 set, chosen by `SkinSet`.
- Game over's picture (`assets/title/game_over.png`) in one bit with
  1040's tool, if the game over screen shows it on the PC.
- Credits: every credit as on the PC, scrolled to fit, plus the
  Playdate SDK if 1031's row says it needs a credit (it is a tool, not
  an asset: probably not).
- Look tests with each skin; behaviour tests read the views.

**Out (do not do):**
- Screens 1044 and 1045 cover.
- Class change (not in Chapter 1; 0246).
- Any PC change.

**Carve (1112):** either order works.

## Implementation steps

1. Read each screen's view; `flow.rs`.
2. One paint at a time with its look test.
3. `frame-png --playdate` pictures for the PR.
4. Simulator: through preparations, the battle, results, save; lose once
   for game over; load the save from the title.

## Acceptance criteria

- [ ] In the Simulator, Chapter 1 can be finished, saved and loaded, and
      lost, with every screen readable.
- [ ] Pictures of each screen in the PR.
- [ ] No PC snapshot changes.
- [ ] Nick's sign-off comment is in the completion notes (or "no comment
      yet").
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with each skin).
- Integration (harness, Playdate skin set): win → results → save; lose →
  game over → rewind or retry.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
