---
id: "1187"
title: "Engine friction: estimates against a reference, the signals as a checklist in the PR, the platformer test on every Out row of an engine ticket, and acceptance greps that match the habit"
type: infra
milestone: "Engine: carve"
model: implementation
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1187 — Engine friction checks that see the split

## Context

From the retro of the last 50 PRs (2026-10-22). ADR-0092's per-ticket
check (the `engine-friction` skill, steps 1–3) passed on the PR that
wrote the most code that wasn't the game's, and the lessons it logged
weren't fed back:

1. **The estimate came from the ticket's split.** 0130 decided the
   playtest bots go "engine first", but split it as *search → engine,
   the rest → game*, and engine#2126 (written in that session) put
   timing, threads and a run history Out "the game keeps its own".
   0506's estimate (600) was 0130's, and counted the xtask command (120),
   the workflow (40) and the targets (150) as game work. It measured
   1,060: 1.8×, "under 2×". The same day the owner asked for the generic
   parts in the engine, and the sweep found seven engine pieces (3.1× to
   9.2×) and about 1,000 lines for 1180 to remove. Skill step 1 says
   "start from the ticket", which inherits the ticket's split.
2. **The signal that would have caught it wasn't asked for.** 0506 moved
   the pin to an engine that already had `shuyi playtest` and wrote its
   own command instead, because the engine's took a finished report: a
   workaround for an engine assumption, which step 3 says "triggers the
   review at any ratio". The PR body's `Engine friction:` line
   (`work-ticket` §6) has room only for the ratio.
3. **Non-Rust code is free headroom.** `cargo xtask code-lines` counts
   `*.rs` only, so 0506's 94-line workflow sat inside the estimate
   uncounted.
4. **One lesson logged five times.** "Count the lines a refactor rewrites,
   not only the new ones" is in the friction log's rows for 1161, 1162,
   1163, 1165 and 1167 ("the 1161 note, not applied"). Step 1 still
   doesn't say it. (1174 adds net lines for refactors to the measure;
   this ticket fixes the estimate.)
5. **Engine tickets' Out lists pre-commit the game.** engine#2126 ("the
   game keeps its own" history), engine#2128 ("no … danger zones or
   forecasts", reversed by engine#2131 a day later) and engine#2127 ("a
   `shuyi autobalance` contract operation", now engine#2541) each put a
   generic piece Out without the `engine-change` §1 tests.
6. **Acceptance greps match a spelling.** 1166's grep (`dt.is_finite()`)
   missed the five sites 1167 had renamed to `dt().is_finite()` that
   morning; 1163's list missed the rewind prompt 1159 had moved; 1169's
   grep by an error message missed a loader that skipped silently (all
   in 1185 and 1186 now).

## Nick input

None. Process text only.

## Scope

**In:**
- `.claude/skills/engine-friction/SKILL.md`, step 1:
  - the reference is **another implementation of the same job**: an
    engine example (hello-grid's playtest, about 75 lines), another
    engine or framework, or this repository's own similar code. The
    ticket's expected size is a starting point to check against that
    reference, never the reference itself.
  - a move, rename or refactor counts every call site it touches (the
    five rows' lesson), with the 1162 row as the example.
  - non-Rust code (workflows, scripts) the change writes is estimated
    and counted by hand on the same line ("+ 94 YAML").
  - a ticket re-gated mid-work (blocked on a new engine ticket, as 0506
    was on engine#2128) restates its estimate when it resumes.
- The same skill, step 3, and `work-ticket` §6: the PR's `Engine
  friction:` line gains the signals: `signals: workaround no · third copy
  no · budget no` (each "yes" names the review's outcome).
- `.claude/skills/engine-change/SKILL.md` §1: when a game ticket splits
  work between the engine and the game, or an engine ticket written from
  the game lists Out items, run the two tests on **each row** of the
  game's side and on each Out item that is generic (threads, timing,
  history, files, a contract operation, CI): "the game keeps its own" is
  an answer only for a row that fails a test. 0130/engine#2126 as the
  example.
- `work-ticket`, its acceptance step: a grep that proves something is
  gone searches for what the code does, in every spelling (`is_finite()`
  on a frame time, not `dt.is_finite()`), and runs again after merging
  `main` into the branch, before the PR merges.
- One row in `docs/engine/friction.md`'s Sweeps section, under the 0506
  sweep: "the per-ticket check missed it; 1187 changed the estimate
  step".

**Out (do not do):**
- `cargo xtask code-lines` counting YAML (a ticket of its own if wanted).
- 1174's area column, gate check and net-lines measure.
- ADR-0092's rule itself (only the skill's steps change; if the session
  finds the ADR's text contradicts the new steps, add a dated note under
  its *Decision*, not a new ADR).

**Either order with 1174:** if 1174 is done, step 1's refactor rule
points to its net-lines measure as the check; if not, it stands alone
and 1174 links it.

## Implementation steps

1. **Read** ADR-0092, the three skills, `docs/engine/friction.md` (the
   rows named above and both Sweeps sections), 0130 and 0506 in
   `tickets/done/`, and engine#2126, #2127, #2128's Out lists (the
   engine's `tickets/done/`).
2. Write each change as in Scope, each with its example from this
   ticket's Context in one line.
3. `cargo xtask ticket-lint` and the docs checks (`typos`, the links
   check) pass.

Expected size: no Rust; about 40 lines of skill text.

## Acceptance criteria

- [ ] `engine-friction` step 1 names a reference implementation as the
      reference, counts call sites for moves and renames, counts non-Rust
      code by hand, and restates an estimate after a re-gate.
- [ ] `work-ticket` §6's `Engine friction:` line has the signals.
- [ ] `engine-change` §1 runs the tests on each row of a split and each
      generic Out item.
- [ ] `work-ticket` says how an acceptance grep is written and when it
      runs again.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None beyond the docs gates (skill text).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
