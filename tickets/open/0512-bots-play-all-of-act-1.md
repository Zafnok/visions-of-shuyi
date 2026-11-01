---
id: "0512"
title: "The bots play all of Act 1: softlocks, balance bands, saves at every node, and a Playdate run"
type: feature
milestone: M4 Enemy AI
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0506", "0509", "0319", "0330", "0521", "1051", "1065", "0743", "0744", "0745", "0746", "0747", "0748", "0749", "0750", "0848", "0849", "0850", "0851", "0852", "0853", "0854", "1070", "1071", "1072", "1083", "1084", "1073", "1066", "1067", "1068", "1069"]
nick_input: setup
completed:
---

# 0512 — The bots play all of Act 1

## Context

Nick no longer checks gameplay tickets one by one (2026-10-21); he plays
Act 1 once, in 1064. Before he does, the bots check the whole act the
way he would hit problems: every chapter played in order, as a
campaign, with the army carried from battle to battle, on the PC and on
the Playdate.

Each chapter ticket already balanced its own battle against a fresh
army (0509's `cargo shuyi autobalance`). What none of them checks:

- **The campaign as a whole:** a Casual, Normal and Hardcore bot
  (0506) playing Chapters 1–8 in a row, with the levels, losses and
  stock they really have by then, visiting towns, fixed skirmishes and
  side quests by a simple route.
- **Softlocks:** a battle no bot can finish, a node that can't be
  entered, a scene that never ends.
- **Saves:** a save and load at every world-map node and after every
  chapter gives the same game (the same state, the same next choice).
- **The Playdate:** memory (1068's margin) and timing (the slowest enemy
  phase, frame times) over Act 1's biggest battles, on the device.

0330 and 0521 are required first: the run must exercise corrected
first-hit combat effects and AI technique reach/Cascade scoring, not
certify the old mechanics. Record the tested game/engine revisions and
rerun the relevant acceptance reports after either fix changes those
revisions. 0753's Classic knowledge repair arrives through 0854's blocker;
dialogue continuity is verified by its rendered-path tests, not inferred
from a persona's win rate.

**Either order with 1180:** if 1180 is done, the campaign entrants use
its engine-backed personas and targets; if not, use the existing game
types and 1180 adapts their shared interfaces when it lands.

**Either order with 1207:** if 1207 is done, the campaign run is
a stage or an option of `cargo shuyi playtest`; if not, of `cargo
xtask playtest`, and 1207 moves it. Either way, write the campaign as
engine#2532's shape already allows (2026-10-22, from the retro of the
last 50 PRs): one try of an entrant is one whole campaign from one seed,
the per-chapter numbers are named measures (engine#2533's summary
lines), and the runner's options, threads and history are not
touched. Then 1207's move is mechanical and the engine needs nothing
new for a campaign.

## Nick input

**Setup** (a measurement, not a check): Nick sideloads the measuring
`.pdx` and pastes the Simulator console, as for 1036 and 1055.

## Scope

**In:**
- `cargo xtask playtest --campaign act1 --bot all` (or the name that
  fits 0505's runner): plays New Game to "Act 1 ends" (1065) with each
  persona, choosing a route on the world map (story battles, the
  guarding skirmishes, and the optional nodes by a fixed rule), with
  the persona's usual rewinds.
- The report: per chapter and persona, win rate, turns, units lost,
  levels and gold on arrival, against the bands in
  `docs/design/playtest-bots.md`; every softlock with its seed and a
  replay; the act's total time per persona.
- **Saves:** at every node visit and every chapter end, save, load, and
  check the loaded state equals the saved one and the next command
  gives the same result (a property-style run over the campaign).
- **The Playdate run:** the slowest phases and frames of Act 1's
  battles in a device build (a debug entry or a bench, as 1026 and 1051
  did), plus the heap per chapter (1068's numbers).
- Fixing what the run finds is **not** in this ticket: each finding
  becomes a ticket (`write-ticket`), added to 1064's `blocked_by` when
  it would spoil the playtest.

**Out (do not do):**
- Tuning battles (the chapter tickets, or new tuning tickets).
- New bot personas, the trained bot (0507), calibrating against Nick's
  play (0508).

## Implementation steps

**Size (0135, ADR-0101):** about 2,000-3,000 changed lines. The natural cut, if it runs over: the
campaign runner, its report and the save check here; the Playdate run in a
new ticket blocked by this one.

1. Read 0505/0506/0509's runner and reports, `playtest-bots.md`, 1007's
   campaign state, 0802's save format, 1068's measurements.
2. The campaign runner and its route rule; the report.
3. The save/load check.
4. The Playdate run; sideload (Nick); record the numbers.
5. File tickets for the findings; put the report in the PR and
   `docs/playtesting.md`.

## Acceptance criteria

- [ ] All three personas reach "Act 1 ends", or each failure is a filed
      ticket in 1064's `blocked_by`.
- [ ] No softlock left unfiled.
- [ ] Campaign reports name the tested revisions, include 0330/0521's
      completed fixes, and are regenerated after a change to those combat
      or AI paths; pre-fix balance results are not used as certification.
- [ ] Save → load at every node and chapter end gives the same state
      (the run's check passes).
- [ ] Device numbers for Act 1 (heap per chapter, slowest phase, frame
      times) are in the PR.
- [ ] The command and how to read its report are in
      `docs/playtesting.md`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the campaign runner on a two-chapter test act in
  `assets-test/` (fast, in the gates); the full Act 1 run is a command,
  not a gate.
- Property: save/load equality over the test act.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
