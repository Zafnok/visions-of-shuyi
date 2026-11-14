---
id: "1254"
title: "The pin review reads behaviour lines, and flags idle adoptions and workarounds the pin ends"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-14
---

# 1254 — The pin review reads behaviour lines, and flags idle adoptions and workarounds the pin ends

## Context

The engine watcher (`.github/workflows/engine-watch.yml`) files an
`engine-pin` issue, and the `engine-pin` skill reviews each engine PR the
move passes. The retro of 2026-11-08 found the loop works: every
`Breaking:` line up to engine #302 reached the game within 37 hours. It
also found three gaps.

- **Behaviour changes have no line to read.**
  - Engine 2350 loosened `.dlg` validation and said "nothing in the game
    must change". Pin PR #547 had to add about 45 lines (`AsciiDlg`) to
    keep game ADR-0045.
  - Engine 2315 and 3210 changed the game's tests and snapshots in #572.
  - The engine is adding a `Behaviour:` line (engine#2066).
- **Adopter tickets sit idle after the pin brings their engine piece:**
  - 1209 for 7 days, 1057 for 14, and 1087, 1252, 0863 and 1068;
  - 1180 has had all 7 engine blockers done since 2026-10-22–24, while
    #520, #535, #538 and #544 kept editing the code it replaces.
  
  The issue's "Adoption tickets still open" list also repeats tickets
  (1170 and 0910 twice).
- **A workaround outlives the pin that ends it.** #563 shipped a
  0.08 s-per-character workaround on top of pin #572, which already had
  engine 2255's end-of-clip report.

**Either order with engine#2066:** if engine#2066 is done, read its
`Behaviour:` lines as below; if not, build the rest and leave the
`Behaviour:` part for a follow-up that engine#2066's completion names.
engine#2066 is done (engine PR #374, in the pin since the pin move of
2026-11-11): the first branch.

## Nick input

None.

## Scope

**In:**
- The watcher's issue and `cargo xtask engine-pin --report` list each
  engine PR's `Behaviour:` lines in the *Breaking* table, with
  `Breaking:` lines, so each gets a decision in the ledger.
  `engine-pin --check` requires one.
- The issue's adoption list is de-duplicated, and marks an adopter
  ticket whose engine piece has been inside the pin for more than 5 days
  with its age.
- The report lists every `engine-friction: engine#NNNN` marker in
  `crates/` whose engine ticket the move passes, and the open PRs that
  add one. The pin PR removes each marker or names the ticket that does.

**Out (do not do):**
- Doing the adoptions (each adopter ticket does).
- The pin PR's own mechanics (1255).

## Implementation steps

1. Read how `crates/xtask/src/engine.rs` (the `engine-pin --report`
   code) reads `Breaking:` lines from the engine's done tickets, and
   read `Behaviour:` the same way.
2. Change the issue body built in `engine-watch.yml` (or the xtask it
   calls) for the de-duplicated, aged adoption list.
3. Add the marker scan (`grep`-like over `crates/`, and `gh pr list
   --json files` for open PRs).
4. Update the `engine-pin` skill, then run `pwsh -File
   scripts/sync-agent-skills.ps1`.

## Acceptance criteria

- [x] `engine-pin --report` lists a passed engine PR's `Behaviour:`
      lines, once engine#2066 is done (or the follow-up is named).
- [x] The adoption list has no repeated ticket, and shows ages.
- [x] A marker whose engine ticket the move passes is listed.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the report over fixture engine tickets and a fixture `crates/`
  tree with a marker.

## Completion notes

What was done (engine#2066 is done and pinned, so the first branch):

- **`Behaviour:` lines.** `trpg-engine-watch` reads them from each engine
  PR's body and its done tickets' notes, as it reads `Breaking:` lines
  (`behaviour_lines`; `Behavior:` counts too), and lists them in the
  report's *Breaking* table after the PR's `Breaking:` lines, each
  prefixed `Behaviour:`. The column is "Breaking or behaviour". A PR with
  only `Behaviour:` lines goes in that table; it doesn't by itself make a
  move due.
- **Wrapped lines.** Both kinds of line are now read as a Markdown
  paragraph: a line the engine's notes wrap at 72 columns goes on to the
  blank line, bullet, heading, fence or next mark. Before, a wrapped
  `Breaking:` line was cut at its first line break (the live report
  showed it on engine #410). One existing test fixture relied on the old
  line-only reading and gained the paragraph break it needs.
- **`engine-pin --check` requires a row.** For the ledger's last section
  it lists the engine PRs between its `from` and `to` (GitHub's compare),
  fetches those without a row, and fails for one with a `Breaking:` or
  `Behaviour:` line (`ledger::missing_rows`, `fetch::unrowed_marked`).
  The current ledger passes.
- **Adoption list.** Each ticket once, from the first pin move whose row
  names it; `unblocks NNNN` rows count as well as `adopt → NNNN` (1180
  was only ever unblocked); each with its age, and `**idle**` past 5
  days.
- **Workarounds the pin ends.** A new section of the issue and the
  report: each `engine-friction: engine#NNNN` marker in `crates/` (file
  and line) whose engine ticket the move passes, and each open game PR
  whose diff adds a marker for such a ticket, or for one an earlier move
  already pinned (by the ledger's Ticket column: the #563 case). The
  watcher's job gains `pull-requests: read` for `gh pr list` and `gh pr
  diff`; a failure there becomes a note, not a failed run.
- `docs/engine/pins.md`'s header and the `engine-pin` skill say all this
  (how to decide a `Behaviour:` row, idle adopters named in the PR body,
  markers removed or their ticket named); loaders synced.

Deviations:

- The open-PR marker list also flags a marker for an engine ticket
  already in the pin, not only one this move passes: that is the #563
  case the ticket's context names, which the narrower reading would miss.
- The skill has the pin PR name an open PR that adds a marker in its
  body, rather than comment on that PR.
- No ADR: this extends ADR-0096's report and check as its ledger header
  and skill describe them.

Measured: the estimate before coding was about 250 production lines (by
the watcher's own `breaking_lines` and the ledger's checks, the code it
extends); `code-lines` added 348 in `total (game)` (1.4×, under 2×). PR
size: estimate 700, measured 869.

Gates: fmt, workspace clippy, the tests of `trpg-engine-watch` and
`xtask` (the only crates changed; nothing else depends on them), their
docs, `engine-pin --check`, `ticket-lint`, `check-leaks`,
`check-skills`, the skill loaders' check and `cargo machete` pass. The
platform builds and the other crates' tests weren't run: no code they
build changed.

Follow-ups: none.

Nick input: none.
