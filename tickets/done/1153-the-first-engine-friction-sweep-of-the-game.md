---
id: "1153"
title: "The first engine-friction sweep: measure the game's biggest jobs against what they should take, and file the fixes"
type: research
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: done
blocked_by: ["1152", "1157"]
nick_input: none
completed: 2026-10-19
---

# 1153 — The first engine-friction sweep of the game

## Context

1151 adds a friction check to every new game ticket: estimate the
size, measure it, and review anything 3× or more over. But the game
already has about 120k lines written before the engine existed. Some of
them are exactly what Nick described (2026-10-15): a simple job that
takes "700 LOC when it should only take like 30", or "a lot of hacky
things" around a dialogue box. New tickets will only find those spots
when they happen to touch them.

This ticket sweeps the existing game once, right after the engine
moves to its own repository (1131) and the two-repository flow exists
(1152), and after the engine parts of open game tickets have moved
there (1157), so it does not file engine work those tickets already
hold. It files what it finds as engine tickets, game-structure
tickets, or "accept" rows. It also sets how later sweeps happen, so
this is not a one-off.

## Nick input

None.

## Scope

**In:**
- **Pick the candidates.** List the game's screens, skins, scenes and
  flows by production size, using `cargo xtask code-lines` against an
  empty tree or a simple line count that skips tests the same way. Take
  the 15 largest, plus these, which are likely spots:
  - the dialogue screen and its skin (`screens/dialogue*`, ADR-0054);
  - the battle map scene and the map skins (ADR-0038);
  - the combat picture stage (ADR-0056);
  - cinematics shot kinds (ADR-0060–0062);
  - the options screen (ADR-0072; it is the reference pattern, so it
    sets the low end);
  - the Playdate front end (`crates/playdate`), for performance
    friction.
- **For each candidate:**
  - name the job it does in one sentence;
  - write the estimate as 1151's skill says, with a reference;
  - measure;
  - read the code for the signals (workarounds, copied engine code, the
    same boilerplate a third time).
  Do not refactor anything here.
- **Review every candidate at 3× or more, or with a signal,** with the
  `engine-friction` skill. Write a row in `docs/engine/friction.md` for
  every candidate, including the ones under 2× (outcome "under 2×"), so
  the next sweep has a baseline.
- **File the findings:**
  - engine tickets in the engine repository (`engine-change` skill:
    ticket-and-follow-up), each with its generic shape and the leak test
    passed;
  - game tickets for game-side structures, and for deleting workarounds
    (blocked by the engine ticket).
  - Group findings that share one engine fix into one engine ticket.
  - Rank them in the log by ratio × how often the pattern recurs
    (the number of screens that would shrink).
- **Later sweeps.** Add a "Sweep" section to the `engine-friction`
  skill. A sweep runs when either of these happens:
  - a milestone in `docs/ROADMAP.md` is closed;
  - the log has three "accept" or 2×–3× rows on the same engine area
    since the last sweep.

  The session that sees the trigger writes the sweep ticket
  (`write-ticket`), named like this one.

**Out (do not do):**
- any refactor or engine change (the filed tickets do them);
- tickets for candidates under 3× with no signal;
- putting any filed ticket on the Chapter 1 critical path (ADR-0063 §7).

## Implementation steps

1. List and measure the candidates; write a scratch table first.
2. Estimate each with a reference, then review the ones that trigger.
3. Write the log rows.
4. File the engine tickets in the engine repository, and the game
   tickets here.
5. Add the Sweep section to the skill.
6. Summarize in the Completion notes: the top five findings with ratio,
   outcome and ticket ids.

## Acceptance criteria

- [x] `docs/engine/friction.md` has a row for every candidate (15
      largest plus the named ones), each with estimate, measured, ratio
      and outcome.
- [x] Every candidate at 3× or more, or with a signal, has outcome 1, 2
      or 3 with a filed ticket, or outcome 4 with a reason.
- [x] Every filed engine ticket passes the leak test (no game ids or
      concepts in its text), and every workaround-removal ticket here
      has its `engine#NNNN` in `blocked_by`.
- [x] The `engine-friction` skill has the Sweep section with its two
      triggers.
- [x] `cargo xtask ticket-lint` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None (research: docs and tickets only).

## Completion notes

**What was done.** Measured every production Rust file of the game
with `code-lines`' rules (about 50k lines) and summed them per screen
(logic, view, skins). Took the 15 largest screens, skins, scenes and
flows plus the combat picture stage and the Playdate front end (the
dialogue screen, the map view, the cinematics and the options screen
were already in the 15): 17 candidates. Each was read for its job, an
estimate with a reference, and the signals (five readers in parallel,
their claims checked against the code before filing). Every candidate
has a row in `docs/engine/friction.md`, and a new *Sweeps* section
ranks the findings by ratio × how many screens would shrink.

**Top five findings:**

| Rank | Finding | Ratio | Outcome | Tickets |
| ---- | ------- | ----- | ------- | ------- |
| 1 | A list cursor and scroll window: 13 hand-written cursors, 6 scroll windows by 4 rules, across 15 screens | 1.8× (class change; 1.2×–1.8× across them) | 1: engine | engine#2307, 1160 |
| 2 | The Playdate front end, 1735 lines against 100: the backend is mostly held by engine 3003 and 3010; the game's own stack and the audio player were in no ticket | 17× | 1: engine (not fixed first: several PRs, already on the engine's roadmap) | engine#3028, engine#3029, notes on 3003, 3010, 3011; 1170 |
| 3 | A screen's result on pop, and covered/uncovered hooks: the flow hosts its 11 children by hand, 8 screens keep a result only for it, the battle's pop-ups are fields, the title's `stale` | 1.6× | 1: engine | engine#2306, 1158, 1159 |
| 4 | A frame time promised finite and positive (13 cleanups) and a clock a held key hurries (3 copies) | 1.6× | 2: engine | engine#2312, 1166 |
| 5 | A folder loader on the bundle (the same loop in 8 content files) | 1.5× | 1: engine | engine#2208, 1169 |

The biggest single ratio after the Playdate is the profile screen
(8.3×): it names nothing of the game and moves to the engine
(engine#2310, 1164).

**Filed.** Engine (in
[Zafnok/shuyi-engine#12](https://github.com/Zafnok/shuyi-engine/pull/12),
one PR for all, merged with a merge commit): 2306–2314, 2208, 2209,
3028, 3029, plus notes on 3003, 3010 and 3011 and a table in its
roadmap. Game: 1158–1170 (each removal blocked by its `engine#NNNN`;
1161 needs no engine), 1171 (game-side: one owner for the campaign
settings, the debug unit out of the class change screen), 1172
(`code-lines --tree`, so later sweeps don't need a throwaway script),
and 0470 (found on the way: the forecast's kill mark is drawn pixel by
pixel, against ADR-0038). None is on the Chapter 1 critical path.

**Accepted (outcome 4), with reasons in the log:** named action lists
in cinematics (one user), the whole-map repaint of an action shot
(unmeasured; parked 1120 holds the engine fix), sprite outlines as four
items (PC skin only, no budget missed), the conversation screen's
newtype (cheaper kept), the title's song clock (the audio backend can't
seek), a save-slots module (one user), choosing a skin by target (1 to
4 lines a screen; the one-bit skins are the game's look; revisit with
engine 3001 and 3010).

**Deviations.**
- Only two candidates reached 3× (the Playdate front end and the profile
  screen): the game's screens are mostly as big
  as their jobs. Nearly all had a signal, though (copied engine code,
  a third copy of the same lines), which is what sent them to review.
- The estimates were written by readers who had seen the code, so they
  lean toward it; the log says to read 1.5× in a sweep as "about right".
- The Playdate count leaves out `chapter_1.rs` (a test module that
  `code-lines` would count because of its name) and the C shim.
- I first planned an engine ticket for per-target skin sets and
  one-bit skins of the engine's screens; on a closer look it is 1 to 4
  lines a screen, so it is an "accept" row instead.
- `cargo xtask ticket-lint --check-engine` will report the new
  `engine#NNNN` ids as missing until the engine PR merges (the plain
  lint passes).

**Engine friction:** none: no Rust code.

**Nick input:** none. Nothing changes in the game.
