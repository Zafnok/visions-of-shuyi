---
id: "0533"
title: "The bots try player-side changes and report the ones that fit the bands"
type: feature
milestone: M4 Enemy AI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: blocked
blocked_by: ["0527"]
nick_input: none
completed:
---

# 0533 — The bots try player-side changes and report the ones that fit the bands

## Context

After playing Chapters 1–4 (2026-11-08) Nick suggested "more thorough
bots automated balancing (i.e. tweaking not just the enemy but also the
player classes and units and levelups)". Asked in 0080 whether the bots
should report player-side changes, also apply small ones inside limits,
or neither, he picked **"Report only"** (`docs/design/playtest-bots.md`,
*The player's side: proposed, never applied*): the bots try changes to
the player's side and report which ones put each chapter in its band;
Nick picks, and a ticket makes the change (as 0864 asked him about
Tamsin). The autobalancer itself still changes only the battle.

Today the search (`cargo shuyi autobalance`, `crates/bots/src/balance.rs`,
`knobs` ~:112) has only enemy knobs, and the player's army comes from
the content: the arrival army (`crates/bots/src/arrival.rs`, `arrival`)
is built from `Content` (characters, classes, growths) by the recorded
playthroughs. **0527** adds the per-unit report (who can't hurt the
enemies, who falls behind in level or EXP) and its flags; this ticket
starts from those flags.

## Nick input

None. The report goes to Nick only as proposals inside a later balance
ticket's question (`ask-nick`); this ticket builds the tool and runs it
once on Chapters 1–4 for `docs/playtesting.md`.

## Scope

**In:**

- **Player-side knobs**, applied to an in-memory copy of the `Content`
  (never written to a file): for one named unit, a growth **±5 or ±10**
  points, a base stat **±1 or ±2**, its starting weapon one tier up or
  down (as the enemy knobs' `tier`, `balance.rs` ~:93, steps a weapon); for a class, a growth
  ±5 or ±10 points. The steps are *the agent's starting values*, as named
  constants with a comment.
- **A report-only search**: a command (e.g. `cargo shuyi autobalance
  --stage chNN --player-side`, or a `playtest` flag; the session
  decides, and says it in `docs/playtesting.md`) that tries the knobs for
  the units 0527 flags first, then the others, rebuilds the arrival army
  from the changed content for each try (a growth changes every level-up
  from Chapter 1 on, so it is the whole chain, not one battle), plays the
  chapter with the bots, and prints the changes that bring each bot into
  its band, best first, e.g.
  `ch03  poacher str growth 45→55  normal 58→66 ✓  casual 41→44 ✓`.
  It prints the change and its numbers, never applies it, and never
  touches `characters.ron` or `classes.ron`.
- A line per change says which unit it was for and the flag that made
  the search try it (0527's flags).
- `docs/playtesting.md`: how to run and read it, and its first run on
  Chapters 1–4 (with 0333's piercing in, if 0333 is done first).
- `docs/design/playtest-bots.md`, *What the report shows*: one sentence
  pointing at the new report.

**Out (do not do):**
- Applying any change to the player's side, in data or in a battle file.
- Changing the bands, the personas or the enemy knobs.
- **1180** and **1207** (the bots on the engine's playtest pieces, and its
  CLI and history). **Either order:** if they are done first, the
  player-side knobs go where they put the game's own knobs; if not, add
  a line to 1180 naming these knobs so the move carries them.
- Not on the Act 1 playtest's chain (1064): it is a tool for the
  balance tickets, not part of the game.

## Implementation steps

1. Read 0527's report and flags, `balance.rs` (the `Knob` trait,
   `tier`, `enemy_numbers`), `arrival.rs` and how the autobalance
   command prints (`crates/tools/src/game_tools.rs`).
2. Write the content knobs (unit growth, unit base stat, unit weapon tier,
   class growth) as `Knob`s over a `Content` copy, with tests that each
   one changes exactly its number.
3. Write the report-only search: for each knob step, the arrival army
   from the changed content, the bots on the chapter, the band check.
4. Print the table; run it on Chapters 1–4; write `docs/playtesting.md`.

**Expected size** (ADR-0101): about 400 production lines (four knobs, the
search loop over the arrival chain, the table) and 300 of tests, plus
docs. If the search needs an engine change to carry content knobs, follow
`engine-change` (the game never waits: a game-side knob type is fine).

## Acceptance criteria

- [ ] Each knob changes exactly one number of a copy of the content, and
      the files on disk are unchanged after a run (test).
- [ ] A test: raising a flagged unit's Str growth gives that unit a
      higher Str in the arrival army of a later chapter.
- [ ] The command prints, for a chapter outside its band, the changes
      that bring it in, with each bot's rate before and after, and the
      unit and flag each change was tried for (a test of the printing,
      and the output in the PR).
- [ ] The first run on Chapters 1–4 is in `docs/playtesting.md`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each knob; the order the search tries them (flagged units first).
- Integration: a short run on a test battle whose band a known change
  fixes.

## Completion notes

*(Filled in by the session that completes the ticket.)*
