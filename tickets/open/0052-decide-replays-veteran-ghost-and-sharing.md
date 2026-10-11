---
id: "0052"
title: "Decide: replays, the veteran's ghost, and sharing a battle"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0046"]
nick_input: decision
completed:
---

# 0052 — Decide: replays, the veteran's ghost, sharing a battle

## Context

Nick, 2026-10-04
([`docs/design/replayability.md`](../../docs/design/replayability.md),
*Replays, the veteran's ghost and sharing*):

> "#5 is ok, if we have a seeded ghost or something that is proven to win"
> "#7 sure"

(#5: the Hardcore bot's turns shown as "the veteran's time" to beat, and a
stuck player watching it play. #7: shared seeds, replays and daily maps.
A daily run was "maybe, but later" and is not part of this ticket.)

Why it's cheap: the core is deterministic and luck is part of the battle
state (ADR-0019, `death-and-difficulty.md`), so a whole battle is its
start plus the list of commands. The bots already play every map many times
and keep each try (`playtest-bots.md`, 0505), and every star is checked by
a bot win before a map ships (0046), so a **proven win exists for every
map**. 0046 decides whether turn targets come from the bot.

## Nick input

**Decision.** Run with the `ask-nick` skill, one question at a time, each
with options from real games and "describe your own".

## Scope

**In:** the questions below; the answers in `replayability.md`; the
`docs/design/README.md` row; the implementation tickets.

**Out (do not do):** any game code; a daily run and leaderboards (later,
not ticketed); multiplayer (0053).

## Implementation steps

1. Read `replayability.md`, 0046's answers, `playtest-bots.md` and
   `death-and-difficulty.md` (*Rewind*).
2. **Ask Nick**, in this order:
   1. **What the ghost is.** Options: the veteran's time only (a number on
      the stars or results screen); the bot's whole winning battle, played
      back like a replay; a hint: the bot plays one turn for you to watch,
      then you take over (an idea from puzzle games' hint buttons).
   2. **When it's available.** Options: after a win only (no spoilers);
      after a defeat; any time from the map menu. And whether watching it
      costs anything (a star can't be earned in that attempt).
   3. **The player's own replays.** Options: none; the last win of each map
      kept and watchable; any battle saved as a replay file on request.
   4. **Sharing a battle.** Options: a code or link that opens a battle at
      its start with the same luck (skirmishes and run-mode fights only, or
      story maps too: spoilers); a code that opens a position mid-battle
      ("can you win from here?"); none. The web build can open a link
      straight into the game.
   5. **Leaderboards later** (Steam: fewest turns per map): wanted or not.
      Only recorded now; built after launch.
3. Record each answer verbatim and as a rule.
4. Write the implementation tickets (`write-ticket`): a replay file and
   player (core already replays commands; the screen is new), the ghost
   (after 0506 and 0046's bot check), share codes (with the web build,
   0206). Leaderboards only as a note in 0903 or a post-launch ticket.

## Acceptance criteria

- [ ] Every question above has Nick's answer, verbatim, in `replayability.md`.
- [ ] The implementation tickets exist; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes
