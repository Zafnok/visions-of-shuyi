---
id: "0048"
title: "Decide: what stars and Visions pay (explore the options first)"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: high
status: todo
blocked_by: ["0046", "0047"]
nick_input: decision
completed:
---

# 0048 — Decide: what stars and Visions pay

## Context

Nick, 2026-10-05
([`docs/design/replayability.md`](../../docs/design/replayability.md),
*Rewards for stars and Visions*):

> "the reward should be something more meaningful than not using rewinds
> but not mandatory to clear the game"
>
> "as for rewards, TBD, we can make a ticket exploring options but since we
> have the split they should pay out slightly differently but not game
> breakingly so"

So the bar is fixed on both sides:

- **Above:** the unused-rewind bonus (7% of a level per unused charge, for
  every deployed unit; `death-and-difficulty.md`, *Unused charges*).
- **Below:** nothing a player needs to finish. Every battle is autobalanced
  for a player who took no reward (`playtest-bots.md`), so a reward that the
  campaign needs would break that.
- **Stars** (story battles, fixed skirmishes, 0046) and **Visions** (random
  skirmishes, 0047) pay **slightly differently**.

Nick asked for the options to be explored before he picks, so this ticket
does a short study first.

## Nick input

**Decision**, after a written comparison. Run with the `ask-nick` skill.

## Scope

**In:**
- A comparison of reward kinds, with how each game used it and what it would
  do to our balance (step 2).
- The questions below; the answers in `replayability.md`; the
  `docs/design/README.md` row; implementation tickets.

**Out (do not do):** any game code; changing the unused-rewind bonus;
anything a player must have to finish the campaign.

## Implementation steps

1. Read `replayability.md`, the answers from 0046 and 0047,
   `death-and-difficulty.md`, `progression.md` (*Reclass*, seals),
   `weapons-and-items.md` (gold, shops, accessories) and `supports.md`.
2. **Write the comparison** in the ticket's completion notes or a section of
   `replayability.md` marked *study, not a rule*. For each reward kind: an
   example from a real game, what it would look like here, and the risk to
   balance. At least:
   - gold (most FE paralogues);
   - rare items: Reclass Seals, unique weapons or accessories (FE
     paralogue rewards);
   - a star currency spent in a special shop (Advance Wars' Battle Maps
     shop; Into the Breach's coins);
   - unit cosmetics: palettes, titles shown on the info screen;
   - unlocks for the run mode (0050) or the randomizer (0051): cards,
     classes, starting units;
   - EXP or class points (the riskiest: it touches every battle's balance);
   - support points;
   - extra lore: a short scene or a codex entry (Dark Souls-style item
     text), which fits the "lore that rewards curiosity" pattern in
     `setting-and-tone.md`.
3. Measure the risk where it can be measured: what the Hardcore bot's win
   rate does on the next story battle if it carries a given reward (0505's
   report), if 0506 is done; otherwise say it can't be measured yet.
4. **Ask Nick**, in this order:
   1. **What stars pay**, from the comparison.
   2. **What Visions pay**, and how that differs from stars (Nick:
      "slightly differently").
   3. **How much in total.** A cap on what a perfect player can earn over an
      act, so it can never be "mandatory".
   4. **Whether the reward is paid on a replay** that beats an earlier
      attempt's stars (only the new stars, or nothing).
5. Record the answers; write the implementation tickets (`write-ticket`) and
   name them in 0046's and 0047's implementation tickets.

## Acceptance criteria

- [ ] The comparison exists and covers every reward kind in step 2.
- [ ] Every question above has Nick's answer, verbatim, in `replayability.md`.
- [ ] The implementation tickets exist; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes
