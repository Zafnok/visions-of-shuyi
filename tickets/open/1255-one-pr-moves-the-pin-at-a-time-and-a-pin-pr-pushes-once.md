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

**Out (do not do):**
- The watcher's report (1254).
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

## Acceptance criteria

- [ ] A pin PR's ledger section without its number passes `--check` on
      the first push.
- [ ] A second open PR changing `shuyi-engine.rev`, not stacked, fails.
- [ ] "Unblocks 1228" with 1228 still `blocked` fails `--check`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each check over fixtures.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
