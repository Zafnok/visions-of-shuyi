---
id: "0049"
title: "Decide: Ironman mode — what a defeat does, saving, switching mode"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0804"]
nick_input: decision
completed:
---

# 0049 — Decide: Ironman mode

## Context

Nick, 2026-10-05
([`docs/design/replayability.md`](../../docs/design/replayability.md),
*Ironman*). Already decided, not asked again:

- **A third mode at New Game**: Classic / Casual / **Ironman**. Nick: "we
  don't need ironvow naming ironman is universal".
- **Ironman hides everything that undoes a result, rewind included**: "I
  guess we need to be consistent... ironman hides everything even rewind".

Today the modes are Classic and Casual (`death-and-difficulty.md`, ticket
0006): picked on a screen after `New Game`, stored in the campaign, and
switched **one way only** (Classic → Casual), from the Options menu (0805).
What a player can undo today: rewind (charges per map tier), Restart battle
(map menu), Retry Battle (Game Over), the one-time Suspend (deleted on load),
loading any of 30 save slots, and soon Replay after a win (0046).

*Claude's reading of "hides everything", to confirm in question 1:* in
Ironman there is no rewind, no Replay, no Restart battle and no Retry
Battle, and fallen units die as in Classic.

## Nick input

**Decision.** Run with the `ask-nick` skill, one question at a time, each
with options from real games and "describe your own".

## Scope

**In:** the questions below; the answers in `replayability.md` (*Ironman*)
and a short pointer in `death-and-difficulty.md` (*Falling units*, the mode
table); the `docs/design/README.md` row; the implementation tickets.

**Out (do not do):** any game code; harder difficulty modes (a separate
question, `death-and-difficulty.md`, *Difficulty*); Visions in Ironman
(asked in 0047).

## Implementation steps

1. Read `replayability.md`, `death-and-difficulty.md` and 0805.
2. **Ask Nick**, in this order:
   1. **Confirm the list** of what Ironman hides (Claude's reading above).
   2. **What a defeat does.** Options: the campaign is over and its save is
      deleted (roguelikes; XCOM's Ironman when the campaign is lost);
      Game Over, then the player may load their last chapter save (softer:
      a retry in disguise); Game Over sends the player back to the start
      of the chapter.
   3. **Saving.** Options: one save the game writes for the player after
      every battle and on quitting, with no slot picker (XCOM's Ironman);
      the normal 30 slots (then loading an older save undoes a battle, which
      question 1 may forbid); one slot per Ironman campaign, plus Suspend.
   4. **Suspend.** Keep the one-time Suspend (it's deleted on load, so it
      undoes nothing) or replace it with the automatic save of question 3.
   5. **Switching mode.** Ironman → Classic allowed one way (as Classic →
      Casual is today), or never.
   6. **The unused-rewind bonus**, since Ironman has no charges. Options: no
      bonus; the bonus as if every charge went unused; a different reward
      for Ironman.
   7. **How Ironman shows.** A mark on the save slot; a mark next to stars
      earned in Ironman (0046); a line on the credits or the ending.
3. Record each answer verbatim and as a rule.
4. Write the implementation tickets (`write-ticket`): the mode in core and
   the save (after 0801, 0802), the New Game screen, hiding rewind, Restart,
   Retry and Replay, and the Options switch. If 0805 is still open, add
   the Ironman switch to it instead of a new ticket.
5. Check `tickets/open/` for tickets that list the modes as "Classic or
   Casual" (0506, 0508, 0715, 0805) and add Ironman where it changes them.

## Acceptance criteria

- [ ] Every question above has Nick's answer, verbatim, in `replayability.md`.
- [ ] `death-and-difficulty.md` points to the Ironman rules.
- [ ] The implementation tickets exist; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes
