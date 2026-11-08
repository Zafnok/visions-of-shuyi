---
id: "0147"
title: "Ticket lint reads done boxes, stale blockers and follow-ups left in prose"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0147 — Ticket lint reads done boxes, stale blockers and follow-ups left in prose

## Context

The retro of game PRs #513–#580 (2026-11-08) found four things the
game's ticket lint (`crates/xtask/src/tickets.rs`) doesn't check. The
engine's lint checks the first already (`check_done_boxes`, engine
ticket 2023).

- **Unticked boxes in a done ticket**, while the PR body shows them
  ticked:
  - every box unticked: 1072 (#545, 6 boxes), 0738 (#555, 6), 1042 (#556,
    5), 0761 (#564, 3) and 0843 (#570, 7);
  - one box each: 1247 and 0826.
  
  The archived ticket, the record a later session reads, says the work
  wasn't done. The reverse happened too. #545 reworded "bots land in
  their bands" so it could tick it, with three of six values out, and
  #535 ticked a box whose check failed (Hardcore-faster ✗).
- **`status: blocked` after the blockers landed:**
  - 1228 is still blocked on engine#2109, done since 2026-11-05 and inside
    the pin since #557, which struck 1228's pin step;
  - 1170 stayed stale the same way for about 49 hours;
  - pin PR #572 left 1240 `blocked` with an empty `blocked_by`.
  
  The retro's PR sets 1228 to `todo`.
- **Follow-ups promised in prose** (`AGENTS.md`'s no-unticketed-work
  rule):
  - #545's side-quest validator gap and #542's `options.md`
    contradiction (both now game#1258);
  - #525's "a voice clip needs two lines in the disclosure list" (now
    0911);
  - #521's "0903 must check `steamworks` on GNU" (the retro's PR adds it
    to 0903);
  - #569's "`Harness::start` takes no storage. Small; no ticket."
- **`nick_input: none` on tickets that got Nick's input:** 520 (bands
  decided mid-ticket), 524 (two listening rounds that amended an ADR)
  and 546.

The engine's twin is engine#2059.

## Nick input

None.

## Scope

**In:**
- An error for an unticked `- [ ]` box in a `done/` ticket's acceptance
  criteria, unless the line holds a miss in the form the engine uses:
  `- [x] ~~…~~ (not met: why, see NNNN)`. A ticked box whose line says
  "✗", "not met" or "misses" is an error too.
- An error for a ticket that is `status: blocked` when every
  `blocked_by` id is done, and for engine ids, also inside the pinned
  revision (`--check-engine`, which reads the engine). Also an error for
  `blocked` with an empty `blocked_by`.
- A warning for a phrase in a done ticket's completion notes that
  promises later work with no ticket id on the line: "not done here",
  "must check", "will need", "no ticket", "for now", "follow-up",
  "worth a ticket".
- A warning when a done ticket's completion notes quote Nick (`Nick,
  20`) and its `nick_input` is `none`.
- Fix the done tickets the box check finds, in the same PR: tick them
  where the PR body shows the box met, or write the miss form.

**Out (do not do):**
- The engine's lint (engine#2059).

## Implementation steps

1. Port `check_done_boxes` from the engine's `crates/xtask/src/tickets.rs`.
2. Add the stale-blocked check beside `check_engine_ids`. It reuses the
   pinned engine checkout `cargo xtask engine` makes, or `gh api` for
   the engine's `tickets/done` at `shuyi-engine.rev`.
3. Add the two warnings, run only on tickets the PR moves to `done/`.
4. Run the lint over `tickets/done` and fix what it finds.

## Acceptance criteria

- [ ] A done ticket with `- [ ]` and no miss form fails, naming the line.
- [ ] A `blocked` ticket whose engine blocker is done and pinned fails.
- [ ] A done ticket's "not done here" with no id warns.
- [ ] `cargo xtask ticket-lint` passes on `tickets/done` after the
      fixes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each check over small ticket strings, both ways.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
