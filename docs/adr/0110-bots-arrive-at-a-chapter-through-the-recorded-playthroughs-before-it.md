# ADR-0110: The bots arrive at a chapter through the recorded playthroughs before it

- **Status:** Accepted
- **Date:** 2026-11-03
- **Related tickets:** 0520 (this decision), 0848 (found the gap), 0803
  and 0848 (the recorded playthroughs)
- **Extends:** ADR-0033 (playtest bots)

## Context

The playtest and the autobalance played every battle with the characters
of its player slots fresh from the character data and the battle's
default pack. A battle with Preparations has an empty default pack, so the
bots played Chapter 2 at the character data's levels with no consumables,
where a player arrives with Chapter 1's EXP, its unused-rewind bonus and
its gold. Chapter 2 was tuned for a weaker army than any player brings.

Three ways to build the army a player brings were open:

1. **Replay the earlier chapters' recorded playthroughs**: the winning
   player's commands that `trpg-ui`'s `ch0N_winnable` tests already keep
   and check in the real game, applied to the campaign as the game does.
2. **A persona's own run** of the earlier chapters: each persona arrives
   with what its own play left.
3. **A level per chapter in data**, as the reference army of a generated
   skirmish does (ADR-0033's 0510 part).

## Decision

**Option 1.** `trpg_bots::arrival` starts a New Game campaign, wins each
earlier story chapter's battle with its recorded playthrough (the enemy
playing as in the game), applies each result with every rewind charge
unused, adds whoever the scenes recruit by the chapter, and prepares the
battle. The playthroughs are compiled into `trpg-bots`
(`include_str!` of `crates/ui/tests/it/fixtures/<battle>_winning.ron`),
so the crate stays pure and the army needs no file at run time.

The playtest's `stage` (`crates/xtask/src/playtest.rs`) is the one place
that picks the army, for the playtest, the autobalance and the tools'
`bots` and `replay` alike: a battle's `bots_army` first, else the arrival
for a story chapter after the first, else the fresh characters (Chapter
1, the debug Quick Battle, the test battles). Every report names the army
it fielded.

**The pack in Preparations** is a game rule of the bots, not a design
decision: the stock's consumables, dearest first, then the dearest
consumable the chapter's basic shop sells that the gold pays for, until
the pack is full (`trpg_bots::prepare`, *the agent's starting rule* in
`docs/playtesting.md`).

## Consequences

- **Deterministic and cheap.** Replaying Chapter 1 takes a few
  milliseconds; the same content gives the same army for every persona and
  every try, so the playtest and the autobalance agree.
- **Real rules, one real player.** The army is exactly what the rules
  make of a real winning play, not an estimate. It is one player's play:
  Chapter 1's recording gives most of the EXP to Tamsin, where another
  player would spread it. A recording is a fair player, not an average one.
- **A recording is a balance lever.** Re-recording an earlier chapter, or
  a balance change that makes the rules stop winning it, changes the army
  of every later chapter. The playtest stops with an error naming the
  recording when it no longer wins; the playthroughs, the chapters, the
  New Game file and the basic shop are on `.github/playtest-levers.sh`'s
  list.
- **Each chapter needs a recording before the next can be tuned.** A
  chapter's ticket already records one (0803, 0848); the next chapter's
  playtest errors without it.
- Option 2 was not taken: a persona's run of the earlier chapters costs
  minutes per try, loses units in some runs (and then has no army to
  bring), and would make each persona play a different Chapter 2. Option
  3 would estimate what the rules can compute.
