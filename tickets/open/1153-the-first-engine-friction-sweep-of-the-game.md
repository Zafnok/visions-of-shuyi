---
id: "1153"
title: "The first engine-friction sweep: measure the game's biggest jobs against what they should take, and file the fixes"
type: research
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1152", "1157"]
nick_input: none
completed:
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

- [ ] `docs/engine/friction.md` has a row for every candidate (15
      largest plus the named ones), each with estimate, measured, ratio
      and outcome.
- [ ] Every candidate at 3× or more, or with a signal, has outcome 1, 2
      or 3 with a filed ticket, or outcome 4 with a reason.
- [ ] Every filed engine ticket passes the leak test (no game ids or
      concepts in its text), and every workaround-removal ticket here
      has its `engine#NNNN` in `blocked_by`.
- [ ] The `engine-friction` skill has the Sweep section with its two
      triggers.
- [ ] `cargo xtask ticket-lint` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None (research: docs and tickets only).

## Completion notes

*(Filled in by the session that completes the ticket.)*
