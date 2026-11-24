---
id: "0193"
title: "Code waiting to move to the engine is marked, and a PR that edits it warns until the engine ticket is checked"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0193 — Code waiting to move to the engine is marked, and an edit to it warns

## Context

Nick, 2026-11-24, after 1180 stalled: "how do we ensure such drift
doesn't happen again".

**What happened:**
- Engine#2132 built `TacticsOrder` from the persona bots' `order`
  (`crates/bots/src/persona.rs`) as it stood on 2026-10-24. The pin
  passed it on 10-25, and 1180, which moves the game onto it, was ready.
- Over the next month seven game tickets changed that same code:
  - 0517 (10-27): no wandering once no foe is left;
  - 0510 (11-01): skirmish bands in `targets.ron`;
  - 0523 (11-03): one strike per unit first;
  - 0849 (11-04): seize;
  - 0526 (11-08): covers in a survive;
  - 0851 (11-08): hold bands and no median check on a hold;
  - 0529 (11-10): holds in a survive.
- None of the seven names 1180 or an engine ticket
  (`grep -cE "1180|2132|TacticsOrder|engine#21" tickets/done/<each>`
  gives 0).
- Three audits re-checked 1180 (11-01, 11-03, 11-17). Each looked only at
  `blocked_by` and the pin, never at what had changed in the code since
  the engine ticket.
- Starting 1180 on 11-24 found that `TacticsOrder` couldn't express two
  of the new rules, so the bots' seeded traces couldn't stay equal.
  Engine#2148 was filed for them, and 1180 waits on it again: a month
  lost, plus a second engine PR.

**Why nobody saw it.** The `// engine-friction: engine#NNNN` marker
(`engine-friction` skill §6, `engine-change` §2) flags a *workaround*.
The engine watcher reads it (`trpg_engine_watch::fetch::friction_markers`),
and 1239 will flag one that has gone stale. Code that an engine ticket
will *replace* (the 0506 sweep's seven pieces, a carve's leftovers) gets
no marker. So a session editing it sees nothing, and the engine ticket
and the game ticket that adopts it both go stale.

**The fix, in four parts:**
1. Mark code that is waiting to move.
2. Warn when a PR edits marked code. The warning never fails a check
   (Nick's rule for process checks: `pr-size`, ADR-0101).
3. The editing PR leaves a dated line on the adopting ticket.
4. An audit that re-checks a ready ticket also looks at what changed in
   the marked code.

## Nick input

None.

## Scope

**In:**
- **The marker.** The same comment, with what replaces the code:
  `// engine-friction: engine#NNNN, moves (game MMMM deletes it)`.
  - The watcher's parser (`trpg_engine_watch::friction_markers`,
    `crates/engine-watch/src/lib.rs:463`) reads only `engine#` and four
    digits, so the extra words change nothing for it, nor for 1239.
  - A marker covers its own line through the end of the item after it:
    the first later line at the marker's indentation that is `}` or ends
    with `;`.
- **The warning**, in `cargo xtask pr-size` (`crates/xtask/src/pr_size.rs`),
  on both its outputs: the Claude Code hook (`--hook`) and the CI step
  (`ci.yml`, the `::warning` line).
  - It fires when the branch's diff against its base (`git diff -U0`)
    changes a line covered by a marker.
  - Its text, one line per marker: `<path>:<line> moves to the engine
    with engine#NNNN (game MMMM deletes it): does engine#NNNN still cover
    this change? If not, add it to the engine ticket now (engine-change),
    and add a dated line to MMMM`.
  - A PR that only adds or removes markers doesn't warn.
- **The backfill.** Mark the code of every open game ticket that waits on
  an `engine#` ticket to delete or replace game code (`grep -l "engine#"
  tickets/open/*.md`, read each one's Scope). At least:
  - 1180's: `persona.rs`'s `order`, `split_attacks`, `covers`, `hold`,
    `advance`, `once_per_action`, `Tally::score`, `Goals`, `danger`,
    `threats`, `seize_distance`; `playtest.rs`'s `Targets` and its
    `checks*`; `movement.rs`'s `attack_tiles`, `threat_area`,
    `threat_tiles`, `area_of`, `zone_of`, `danger_zone`, `threats_at`;
    `ai.rs`'s hostile-target `flow_field`. They are marked for engine#2132,
    engine#2131, engine#2129 or engine#2148 as each one's ticket says.
  - 1207's xtask playtest wrapper.

  Name each in the completion notes. A workaround already marked stays as
  it is.
- **Skills** (the shared sources in `.claude/skills/`, then `pwsh -File
  scripts/sync-agent-skills.ps1`):
  - `engine-change`, *Ticket and follow-up*: the PR that files an engine
    ticket to replace game code marks that code.
  - `engine-friction`, *How a sweep runs*: each filed row marks its code.
  - `work-ticket` §3: when the `pr-size` warning names a marker, check
    the engine ticket covers the change. If it doesn't, add the gap to
    that ticket by `engine-change`, with Nick's OK. Then add a dated
    line to the adopting ticket's Context.
  - `work-ticket` §1: an engine-blocked ticket is ready only when the
    marked code hasn't changed since its engine ticket was written, or
    every change is in its Context (`git log --since <engine ticket's
    date> -- <marked files>`).

**Out (do not do):**
- Failing any check on a marker. This is a warning, like `pr-size`.
- 1239's stale-marker check. Either order with 1239: if 1239 is done, a
  "moves" marker whose engine ticket is done at the pin passes its check
  while the adopting ticket is open (it names `engine#NNNN`), so nothing
  to do; if not, nothing to do (1239 builds the check as written).
- 0181's one PR-checks CI job. Either order with 0181: if 0181 is done,
  the warning comes through `pr-size` wherever that job runs it, so check
  it does; if not, nothing to do (`ci.yml`'s `pr-size` step prints it).
- Adopting any engine piece (1180, 1207 and the rest are their own
  tickets).

## Implementation steps

1. `crates/engine-watch/src/lib.rs`: next to `friction_markers`, a pure
   `marker_spans(source) -> Vec<(usize, usize, String)>` (first line,
   last line, engine id) with the covering rule above.
2. `crates/xtask/src/pr_size.rs`:
   - a pure `changed_lines(diff_u0)` that reads the `@@ -a,b +c,d @@`
     hunks of `git diff -U0 <base>...HEAD`, on the new side;
   - `moved_warnings(spans, changed)`: one line per marker, in path and
     line order;
   - both added to `output` and `hook_answer` beside the size warning.
     The hook keeps its JSON shape.
3. The backfill (Scope). Run `cargo xtask pr-size` on a scratch edit
   inside a marked `fn` and check it warns. Then drop the edit.
4. The skill texts, and sync them.
5. `docs/engine/README.md`, where the watcher's markers are described
   (around line 37): the second use, one paragraph.

Expected size: about 150 production lines and 150 of tests, plus the
skills, the backfill markers (about 40 lines) and this ticket: under 600.

## Acceptance criteria

- [ ] `marker_spans` covers a marked `fn`, `struct`, `impl` and a
      one-line item, and nothing past the item's end (unit tests).
- [ ] `changed_lines` reads hunks with and without counts, and a
      deletion (`+c,0`) (unit tests).
- [ ] `cargo xtask pr-size` and `--hook` print the marker line for a diff
      inside a marked item, and nothing for a diff outside one or one
      that only adds a marker (tests on fixture diffs).
- [ ] Every open ticket that waits on an `engine#` ticket to delete game
      code has its code marked. The completion notes list them, with
      1180's and 1207's.
- [ ] `engine-change`, `engine-friction` and `work-ticket` carry the
      steps above; `scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] The engine watcher's tests still pass: a "moves" marker parses as
      today's.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `marker_spans`, `changed_lines` and `moved_warnings`, in
  `crates/engine-watch` and `crates/xtask`.
- Integration: `pr-size`'s output and hook answer on fixture diffs, in
  `pr_size.rs`'s own `mod tests` (beside the hook's existing tests).
  Add no new file under `crates/xtask/tests/`.

## Completion notes

*(Filled in by the session that completes the ticket: the markers added,
deviations, follow-ups.)*
