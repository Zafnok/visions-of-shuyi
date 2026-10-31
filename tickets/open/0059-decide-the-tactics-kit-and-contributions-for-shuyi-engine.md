---
id: "0059"
title: "Decide: is the grid-tactics kit open in Shuyi Engine, and who may contribute code to it?"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0059 — Decide: the tactics kit, and who may contribute

## Context

Ticket 1101 studied splitting an open-source engine out of the game
(`docs/engine/feasibility.md`, ADR-0063). On 2026-10-14 Nick answered
most of what this ticket first asked. His words are recorded in the
engine repository's [`docs/decisions.md`](https://github.com/Zafnok/shuyi-engine/blob/main/docs/decisions.md):

- **Plan:** B.
- **Name:** **Shuyi Engine**. It is not registered as a trademark.
- **Repository:** `github.com/Zafnok/shuyi-engine`, **public** now.
- **License:** **MIT OR Apache-2.0**.

Two questions are still open. They decide what the carve (1103–1118)
moves and what the engine repository's `CONTRIBUTING.md` says before the
first public release.

## Nick input

**Decision.** Run with the `ask-nick` skill. Options come from real
projects, with a recommendation and "describe your own". Say plainly
that these are licensing and community choices, not game design.

## Scope

**In:** the two questions below. The answers are recorded in the engine
repository's `docs/decisions.md` (Nick's words, then the rule), through a
small PR there.

**Out (do not do):**
- editing this repository's `LICENSE`;
- any code.

## Implementation steps

1. Read `docs/engine/feasibility.md`, ADR-0063, `docs/engine/boundary.md`
   (what the tactics kit is), and the engine repository's
   `CONTRIBUTING.md` and engine ADR-0002.
2. Ask:
   1. **The tactics kit.** These parts would be MIT OR Apache-2.0 in the
      engine:
      - the grid and pathfinding;
      - movement and attack ranges;
      - the `TurnRules` trait;
      - the battle cursor;
      - the grid map scene and its skins.

      This game's rules (stats, Combat Arts, classes, magic, AI weights)
      stay the game's either way. Options:
      - **A. Open, in the engine** — like Halley, the open engine under
        the tactics game Wargroove. Other tactics games can start from
        it.
      - **B. Kept in the game** — the engine has no grid-tactics parts.
        Other tactics developers build their own.

      Recommendation: **A**. Grid tactics is the engine's best first
      showcase, and the parts are mechanics no one can own. The game's
      look and rules stay closed either way.
   2. **Outside code.** Options:
      - **A. Issues and discussions only, until the first public
        release** — SQLite works this way.
      - **B. Pull requests welcome under a Developer Certificate of
        Origin** — a sign-off line on each commit, as Linux and GitLab
        ask.
      - **C. Pull requests under a Contributor License Agreement** —
        contributors grant Nick the right to relicense; Qt asks this.

      Recommendation: **A** now, then **B** at the first public release
      (engine ticket 1132).
3. Record the answers in the engine repository's `docs/decisions.md`.
   If the answer to question 1 is B, update 1114–1116 here: the kit's
   parts stay in the game's crates.

## Acceptance criteria

- [ ] Both answers are in the engine repository's `docs/decisions.md`,
      verbatim.
- [ ] 1114–1116 match the first answer; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes

*(Filled in by the session that completes the ticket.)*
