---
id: "1255"
title: One PR moves the pin at a time, and a pin PR pushes once
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1255 — One PR moves the pin at a time, and a pin PR pushes once

## Context

The retro of 2026-11-08 found four things in the game's pin moves:

- **Every pin PR pushes twice.** All 8 of the last 8 (#513, #524, #529,
  #537, #547, #557, #572, #574) pushed a second commit only to write
  their own PR number into the `docs/engine/pins.md` heading. Each
  second push cancels a full CI run (24–140 minutes) and causes the
  false `codeql-result` reds. #547 guessed #541 and had to correct it.
- **Two PRs moved the pin at once.** Ticket 1240's PR #574 moved it in
  its first commit (8af6ff08), 11 minutes after engine-pin PR #572
  opened. It redid all 17 engine PRs: the same bots fold, the same break
  fixes and a ledger of its own. A 99-file merge threw that away. The
  two also wrote the same new ticket as 1252 and 1251.
- **"Unblocks NNNN" without the status change.** #547 wrote "unblocks
  1170" in the ledger and left `status: blocked`. #557 struck 1228's pin
  step and left it blocked. The skill's step 5 asks for the change.
- **A failure that happens at either pin was hidden in a pin PR.** #557's
  commit b8e413d0 narrowed the proptest generator with `prop_filter` to
  get past a flaky print/parse failure "at either pin". Open ticket
  0759's Out forbids exactly that, and 0759 wasn't named.

The retro of PRs #616–#735 (2026-11-17,
`docs/research/retro-2026-11-17-prs-616-735.md`, T6) found the second push
still there (#624, #638, #641, #700, #706, #732: 6 of 10 pin PRs in 7
days) and two more costs:

- **Pins move about once a day.** `engine-watch.yml` runs hourly (`cron:
  "17 * * * *"`) and on every engine `main` push, and files an issue as
  soon as any PR changes the public API. 10 pin PRs in 7 days, each a
  full CI and Playtest run; issue #733 opened 4 minutes after #732
  merged.
- **The ledger is 64 KB.** `docs/engine/pins.md` is 442 lines; 60 rows
  say `nothing to change`, and 42 lines run past 400 characters.

**Amended by the [feedback-latency retro](../../docs/research/feedback-latency-2026-11-24.md) (2026-11-24).** When a feedback item's
engine PR merges, the game PR of that item moves the pin to that merge
commit at once, ahead of the next batched pin PR: about 30–40 h over
five items in two weeks went to waiting for the batch (0254 about 14 h,
0908 about 12 h, 1314 about 7 h, 1343 about 5 h). Reconcile this with
"one PR moves the pin at a time": the item's own PR is that one.

## Nick input

None.

## Scope

**In:**
- `engine-pin --check` accepts a ledger heading without a PR number (or
  a `#this-pr` placeholder that CI fills in on merge), so the first push
  is the only one.
- A CI check that fails a PR changing `shuyi-engine.rev` while another
  open PR also changes it, unless it is based on that PR's branch.
  `work-ticket` and `engine-change` §7 say to stack on an open pin PR,
  never pin again.
- `engine-pin --check` fails a ledger "unblocks NNNN" whose ticket is
  still `blocked`, or still lists the pinned `engine#` id.
- The `engine-pin` skill: a failure that also happens at the old pin is
  not a break. Name its ticket and leave it; never narrow a generator or
  a test in the pin PR. A diff adding `prop_filter` or `prop_assume`
  names a ticket in its commit (a check in the tickets job).
- The watcher batches pins: it files (or reopens) at most one
  `engine-pin` issue a day, sooner only when an engine PR unblocks a
  game ticket. A public API change alone waits for the day's batch. The
  rule goes in `classify` (`crates/engine-watch/src/lib.rs`) beside the
  48-hour wait, whichever trigger ran; the hourly cron stays a backstop.
  ADR-0096 §2 amended (`write-adr`).
- A compact ledger: an engine PR that needs nothing from the game takes
  one short line (or joins the `Nothing for the game: N PRs` count),
  and details are written only where the game acts (`fixed:` with a
  change, `adopt →`, `unblocks`). `engine-pin --check` fails a row over
  about 200 characters in the newest section. The existing sections are
  compacted once, and the format at the top of `pins.md` and the
  `engine-pin` skill's example say so.

**Out (do not do):**
- The watcher's report (1254): its sections and wording. Only the rule
  for when an issue is filed changes here.
- Undoing #557's `prop_filter` (0759 does it).

## Implementation steps

1. Change the heading rule in `crates/xtask/src/engine.rs`'s `--check`,
   and the skill's step 6.
2. Add the two-PRs-move-the-pin check to the tickets job (`gh pr list
   --state open --json number,files,baseRefName`).
3. Add the "unblocks" check over `pins.md`'s newest section.
4. Add the `prop_filter`/`prop_assume` check.
5. Update `engine-pin`, `work-ticket` and `engine-change`, then run
   `pwsh -File scripts/sync-agent-skills.ps1`.
6. Add the once-a-day batch to `classify` (an unblock still files at
   once), with unit tests in `crates/engine-watch/src/tests.rs`, and
   amend ADR-0096 §2.
7. Teach the ledger rules (`crates/engine-watch/src/ledger.rs`) the
   short "needs nothing" line and the row-length cap, compact
   `docs/engine/pins.md` once, and update its format note, ADR-0096 §6
   and the `engine-pin` skill.

## Acceptance criteria

- [ ] A pin PR's ledger section without its number passes `--check` on
      the first push.
- [ ] A second open PR changing `shuyi-engine.rev`, not stacked, fails.
- [ ] "Unblocks 1228" with 1228 still `blocked` fails `--check`.
- [ ] A second public API change within a day of the last `engine-pin`
      issue files nothing; an engine PR that unblocks a game ticket
      files at once (unit tests).
- [ ] `docs/engine/pins.md` keeps no `nothing to change` row with a
      long explanation, and a newest-section row over the cap fails
      `--check`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each check over fixtures.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
