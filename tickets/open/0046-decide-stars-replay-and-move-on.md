---
id: "0046"
title: "Decide: stars on story battles and fixed skirmishes, Replay, and the \"Move on?\" question"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0804"]
nick_input: decision
completed:
---

# 0046 — Decide: stars, Replay and "Move on?"

## Context

Nick, 2026-10-04 and 2026-10-05
([`docs/design/replayability.md`](../../docs/design/replayability.md),
*Stars* and *Replaying a won map*). Already decided, not asked again:

- **Story battles and fixed skirmishes have fixed stars.** One star is a
  clear; the extra stars are challenges. They never have Visions (random
  skirmishes have Visions instead, 0047).
- After a win the player can **Replay** the map before going on. **A replay
  undoes the whole attempt.**
- **Continue always asks first** ("Move on?" or similar): the player may have
  lost units, used too many consumables or missed stars, and even a
  three-star win can have cost too much.
- **The victory scenes play after Continue.**
- What stars pay is 0048, not this ticket.
- Ironman hides Replay (0049).

Where this sits today: a win shows `VICTORY`, then the results screen (0810:
gold, rewinds left, the bonus EXP bars), then the victory scenes, then in the
linear chapters "Save your progress?" (`death-and-difficulty.md`). Restart
and Retry Battle already put a battle back at its first turn, or at
Preparations if it has one, and refund the rewind charges (0408). The
Hardcore bot already measures each map's median turns (`playtest-bots.md`),
and Nick didn't want to set turn targets by hand.

This waits for the Chapter 1 playtest, like the other post-playtest
decisions.

## Nick input

**Decision.** Run with the `ask-nick` skill, one question at a time, each
with options from real games and "describe your own". Show a rendered
mockup of the end-of-battle screens (results, stars, Replay / Continue,
"Move on?") built from the Chapter 1 map, with no bought art (ADR-0040).

## Scope

**In:** the questions below; the answers written into
`docs/design/replayability.md` (*Stars* and *Replaying a won map*); the
`docs/design/README.md` row; the implementation tickets the answers call for.

**Out (do not do):** any game code; the rewards (0048); Visions (0047);
Ironman (0049); the veteran's ghost (0052).

## Implementation steps

1. Read `replayability.md`, `death-and-difficulty.md` (*Results screen*,
   *Rewind*, *Saving*), `playtest-bots.md` and `chapter-1.md`.
2. Render the mockups (PNGs in `docs/screenshots/0046-*.png`).
3. **Ask Nick**, in this order:
   1. **What the extra stars are.** Options: the same challenges on every
      map (Advance Wars DS ranks every map by speed, power and technique);
      written for each map (Into the Breach's bonus objectives, Super Mario
      3D World's hidden stars); a mix: star 2 is the same everywhere, star 3
      is written for the map.
   2. **How many stars.** Options: 3 (clear plus two challenges); 4 or more
      on big maps; always 3.
   3. **Which challenges are allowed.** Show a list with an example for
      each: nobody falls; win within N turns, with N taken from the
      Hardcore bot so nobody sets it by hand; no rewinds; at most N
      consumables; visit every village or open every chest; defeat the boss
      with a given character; protect a green unit. Say plainly that
      **every star is checked before a map ships**: a bot must win it, or the
      map doesn't pass (the proven win 0052 can show players).
   4. **One attempt or several.** Options: all stars must come from one
      attempt (Into the Breach's per-run objectives); the best of every
      attempt counts, so star 2 and star 3 can come from different replays
      (most mobile star ratings). Replays undo the whole attempt either way.
   5. **Rewinds and Casual.** Does using a rewind lose a star (or only the
      "no rewinds" star)? Does a Casual retreat count as falling for
      "nobody falls"?
   6. **Where the stars show.** Before the battle (objective screen,
      Preparations), during it (map menu), after it only; and on the world
      map's nodes and the save slots.
   7. **The "Move on?" question.** What it lists (units lost, consumables
      used out of those brought, stars got and missed, rewinds used), and
      its wording. Show it on the mockup.
   8. **Where a replay starts.** Options: at Preparations, like Retry
      Battle; at the first turn. And whether the luck is new or the same as
      the first try.
   9. **Replaying later.** Options: only on the end screen (what Nick said);
      also from a menu after moving on (Advance Wars lets you replay a
      cleared campaign map). If later replays exist: what a later replay
      may change (stars only, never the army).
   10. **The order of the screens** in a linear chapter: `VICTORY` →
       results → stars → Replay / Continue → "Move on?" → victory scenes →
       "Save your progress?". Confirm or change it.
4. Record each answer verbatim and as a rule.
5. Write the implementation tickets (`write-ticket`): stars in core (map
   data, star checks, saved best), the end-of-battle screens and "Move on?"
   (after 0810), and a bot check that every star is reachable (after 0506).
   Name 0048 where rewards are paid.
6. If the screen order changes `death-and-difficulty.md`'s *Results screen*
   or *Saving*, update those sections.

## Acceptance criteria

- [ ] Every question above has Nick's answer, verbatim, in `replayability.md`.
- [ ] The mockups are in `docs/screenshots/` and contain no bought art.
- [ ] The implementation tickets exist; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes
