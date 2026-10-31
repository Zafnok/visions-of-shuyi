---
id: "1067"
title: "Playdate: camp, supports, promotion and the Preparations shop in one bit"
type: feature
milestone: Post–Chapter 1
model: implementation
effort: high
status: todo
blocked_by: ["1003", "0606", "0443", "1046"]
nick_input: none
completed:
---

# 1067 — Playdate: camp, supports, promotion and the Preparations shop

## Context

Between Act 1's battles the PC game gets screens Chapter 1 never had:

- **1003**: Supports (the list of pairs and their conversations) and
  Camp (camp events), offered between chapters and from the world map
  menu.
- **0606**: promote and reclass from the between-battle menu (where it
  lives is 0606's question to Nick).
- **0443**: the Preparations screen's shop tab.

Nick plays Act 1 on his Playdate in 1064, so each needs a one-bit paint
(ADR-0054, ADR-0070), at 50 × 15 cells, in 1038's window style, as
1046 did for Preparations, results, saves and the ending screens. The
support and camp conversations themselves play on the dialogue screen
(1043).

## Nick input

None. No check after merge (Nick, 2026-10-21): `cargo shuyi frame-png
--playdate` pictures go in the PR, and the rest goes on 1064's
checklist.

## Scope

**In:**
- One-bit paints for Supports, Camp, the promote / reclass screens and
  the Preparations shop tab, chosen by `SkinSet`.
- "What's new" marks (1003) told apart without colour (`playdate.md`,
  *Telling things apart without colour*).
- Look tests with the Playdate skin; behaviour tests read the views.

**Out (do not do):**
- The world map, towns and their shop (1066).
- The conversations' dialogue screen (1043).
- Any PC change.

## Implementation steps

1. Read the four screens' views and PC skins, 1046's paints and
   `playdate.md`.
2. Estimate first (ADR-0092): about 250 production lines, by
   comparison with 1046.
3. One paint at a time with its look test.
4. `frame-png --playdate` pictures of each for the PR.
5. Simulator: open Camp and Supports, read a conversation, promote a
   unit (with the debug tools' class change unit if needed), buy in the
   Preparations shop.

## Acceptance criteria

- [ ] In the Simulator each screen opens, reads and works.
- [ ] Pictures of each screen in the PR.
- [ ] No PC snapshot changes.
- [ ] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit (look, Playdate skin): each screen.
- Integration (harness, Playdate skin set): camp → supports → a
  conversation; promote; Preparations shop buy → deploy.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
