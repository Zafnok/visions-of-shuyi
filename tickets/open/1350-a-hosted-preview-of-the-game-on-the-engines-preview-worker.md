---
id: "1350"
title: "Decide: a hosted preview of the game, played a frame an action, beside the web demo"
type: design-decision
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: blocked
blocked_by: ["engine#2611"]
nick_input: decision
completed:
---

# 1350 — Decide: a hosted preview of the game

## Context

The engine pin move of 2026-11-23 passed engine PRs #546 (engine ticket
2602, engine ADR-0141) and #547 (2609). A game's tool program now has
`preview-worker`, which runs one visitor's session in a process of its
own from the game's start options, answers each action (`{"action":
"<name>"}`) with the frame after it as a PNG and its text, refuses the
developer's debug tools (`Session::for_visitor`), and ends when its input
closes; and `preview`, the HTTP API over it (sessions, one frame an
action, deadlines). The quotas (engine 2610) and the page, its load test
and a month's cost for the engine's owner to decide on (engine 2611) are
not built.

The game gets both commands at the pin (its tool program runs the
engine's `shuyi-cli`), but runs neither. It already has a web demo and
the writer trial's web preview (0914, 0771), which run the whole game in
the visitor's browser.

## Nick input

**Decision**, once engine 2611 has the page and its cost: does the game
offer a hosted preview (for example a store page or a press kit, where a
visitor plays a few turns without loading the web build), and where? With
the cost per month and what a visitor would see, beside the web demo.

## Scope

**In:**
- The question to Nick (`ask-nick`), with engine 2611's page shown on
  the game and its cost.
- If yes: a follow-up ticket for the game's half (its start options for
  a visitor, which actions, the debug tools refused, a test that
  `preview-worker` answers a frame of the title and of a battle).
- If no: the reason in `docs/design/`, and this ticket closed.

**Out (do not do):**
- Deploying or paying for anything.
- Changing the web demo or the writer trial.

## Implementation steps

1. Once engine 2611 is done, run `cargo shuyi preview-worker` on the
   game locally and take a few frames.
2. Ask Nick with those frames and the cost (`ask-nick`).
3. Record the answer; write the follow-up ticket if yes.

## Acceptance criteria

- [ ] Nick's answer recorded in `docs/design/`.
- [ ] A follow-up ticket, or the reason there is none.

## Tests required

None (a decision).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
