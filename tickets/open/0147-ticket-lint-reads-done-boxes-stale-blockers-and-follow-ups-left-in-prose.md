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

2026-11-17 friction sweep
([report](../../docs/research/friction-2026-11-17.md), *Should be engine
features*, "Ticket lint and pr-size are forked between the repos"): the
engine's lint at the pin (e738ffc, `crates/xtask/src/tickets.rs`, whose
header says it was ported from the game's lint) already has three of
these checks, from engine 2023 and 2059, both done:
`check_done_boxes`, `check_blocked_with_done_blockers` and
`check_prose_follow_ups` (with its `FOLLOW_UP_PHRASES` list and a `No
ticket: <why>` escape). This ticket ports those three with their tests
instead of designing new ones, adapting `game#` ids to `engine#` ids;
only the Nick-quote warning is the game's own. One difference to keep
from the engine: its stale-blocked check passes a `blocked` ticket with
an empty `blocked_by`, which matches `tickets/README.md` (a written
reason that names who unblocks it), so this ticket drops its own
"empty `blocked_by` is an error". Bucket siblings (process and
tooling): 0181 ports the engine's `Parked:` and size checks the same
way; 0188 may move `tickets.rs` into a light crate (either order: the
port goes wherever the file is).

**Amended by the [feedback-latency retro](../../docs/research/feedback-latency-2026-11-24.md) (2026-11-24).** Do the "status: blocked
with every blocker done" error next: today it would have caught 0610 and
1317 (both fixed by hand in the retro's PR).

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
  revision (`--check-engine`, which reads the engine). A `blocked`
  ticket with an empty `blocked_by` passes, as in the engine.
- A warning for a phrase in a done ticket's completion notes that
  promises later work with no ticket id on the line: the engine's
  `FOLLOW_UP_PHRASES`, plus any of "not done here", "must check", "will
  need", "no ticket", "for now", "follow-up", "worth a ticket" it lacks.
- A warning when a done ticket's completion notes quote Nick (`Nick,
  20`) and its `nick_input` is `none`.
- Fix the done tickets the box check finds, in the same PR: tick them
  where the PR body shows the box met, or write the miss form.

**Out (do not do):**
- Changing the engine's lint.
- Making the lint an engine command any game can run (an engine step,
  only with Nick's OK).

**Either order with 1423** (the engine's redundancy sweep of 2026-11-28): if 1423 is done, the engine's
`shuyi-process` already does this ticket's Rust part with the game's
config. Do only its skill and doc edits, or move it to `done/` as 1423
says. If not, port as planned, and 1423 deletes the port when the engine
ships its crate (engine#3448 and #3449, if Nick says yes).

## Implementation steps

1. Port `check_done_boxes` from the engine's `crates/xtask/src/tickets.rs`
   at the pin, with its tests.
2. Port `check_blocked_with_done_blockers` beside `check_engine_ids`,
   with its tests. For `engine#` ids it reuses the pinned engine checkout
   `cargo xtask engine` makes, or `gh api` for the engine's
   `tickets/done` at `shuyi-engine.rev`.
3. Port `check_prose_follow_ups` (and `notes_units`, `FOLLOW_UP_PHRASES`)
   and add the Nick-quote warning, both run only on tickets the PR moves
   to `done/`.
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
