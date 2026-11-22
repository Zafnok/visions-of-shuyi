---
id: "0098"
title: "Decide: a mode easier than Casual for players new to games"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0098 — Decide: a mode easier than Casual for players new to games

## Context

Nick (2026-11-22), after watching a new player play Chapter 1:

> "Probably should have a difficulty even below Casual since [the new player] who
> is barely a gamer struggled w it. This would need a new bot profile to
> pair I suppose."

What exists (`docs/design/death-and-difficulty.md`, "One difficulty,
tuned well", around line 234):

- **Game mode** (`GameMode`, `crates/core/src/campaign.rs`): Classic (a
  fallen unit dies) and Casual (a fallen unit retreats and is back next
  battle at full HP). The lead falling is a game over in both. The only
  switch is Classic → Casual at Preparations. Ironman is decided as a
  third mode, still open (0049).
- **Map tier** (`Difficulty`: Easy, Normal, Hard, Finale) sets only the
  rewind charges (2, 3, 5, 8); Chapter 1 is Normal.
- There is no difficulty picker; harder modes may come later.
- Bot personas (`crates/bots/personas.ron`, `docs/design/playtest-bots.md`):
  casual (Casual mode, only needs to win), normal, hardcore; bands per
  tier in `crates/bots/targets.ron` (Normal tier: casual wins 30–55%).
  So the casual bot, by design, loses about half its Chapter 1 runs.

Related: 0927 (a Classic player of the Chapter 1 demo can't reach
Preparations to switch to Casual).

## Nick input

**Decision** (`ask-nick`, worded as player situations with an example,
per the memory on plain rules). Options to combine (check each game
before citing):

- **A. Fallen units come back next turn** (Fire Emblem Fates' Phoenix
  mode), the lead too, so no game over except the objective failing.
- **B. Weaker enemies**: lower enemy stats or fewer enemies (Fire Emblem
  Awakening's Normal vs Hard; Triangle Strategy's Mild/"Story" setting;
  XCOM's Rookie).
- **C. More rewinds**, or rewinds without limit (Fire Emblem Three
  Houses' Divine Pulse counts change by difficulty).
- **D. Help in battle**: units heal a little each turn, or the forecast
  marks risky moves (an enemy that can kill this unit).
- **E. Your own.**

And: its name; where it's picked (New Game, Preparations, Options) and
whether a player can switch up or down during the game; whether it's
offered to a player who loses Chapter 1 twice; and the band the new bot
should hit (e.g. "a bot that plays like a newcomer wins Chapter 1 at
least 80% of the time").

Answers go to `docs/design/death-and-difficulty.md` and
`docs/design/playtest-bots.md`.

## Scope

**In:** the questions, the answers recorded, 0880 and 0536 adjusted.

**Out (do not do):** code.

## Implementation steps

1. Read the design docs above and the bots' Chapter 1 results
   (`crates/bots/tests/ch01_difficulty.rs`) for numbers to show Nick.
2. Ask with `ask-nick`; record; adjust 0880 and 0536.

## Acceptance criteria

- [ ] The design docs say what the new mode changes, its name, where
      it's picked, and the new bot's band.
- [ ] 0880 and 0536 match.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None (a design document).

## Completion notes

