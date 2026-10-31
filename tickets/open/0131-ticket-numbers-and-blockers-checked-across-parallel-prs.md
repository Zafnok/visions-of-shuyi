---
id: "0131"
title: "Ticket numbers checked against open PRs, and no ticket in done/ with an open blocker"
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0131 — Ticket numbers and blockers checked across parallel PRs

## Context

From the retro of the last 50 PRs (2026-10-22, #344 → #410). Two
sessions built #401 (0513) and #402 (0321) side by side from the same
`main`, and each filed a new ticket **0855**. Both PRs were green on
their own and merged 20 seconds apart; the next CI run on `main`
failed ticket-lint (`duplicate ticket id 0855`) on all three OSes, and
`main` stayed red about 30 minutes until #405 renumbered one to 0856.

ADR-0042 keeps PRs merging without being up to date with `main` (the
owner weighed it on 2026-10-02: serial merging cost more than the rare
red `main`). That stays. The gap is narrower: **write-ticket picks "the
next unused number" from the local checkout only**, so two sessions
with open PRs pick the same one. A check against open PRs at the moment
a number is picked would have stopped this one.

The same retro found a second, smaller drift: ticket 0740 had
`blocked_by: ["1177"]` (the engine pin move) and merged while 1177 was
still open, moving the pin itself; 1177 then ended a no-op, and 0449
needed engine#2304 with an empty `blocked_by`. Nothing checks that a
ticket in `tickets/done/` has its blockers done too.

## Nick input

None. Tooling only; nothing a player sees changes.

## Scope

**In:**
- `cargo xtask ticket-next <block>` (e.g. `ticket-next 03`): prints the
  next free number in the block across `tickets/open/`, `tickets/done/`
  **and** the ticket files added by open pull requests (read with `gh pr
  list --state open --json number,files`). Without `gh` or the network
  it prints the local answer with a warning line saying open PRs were
  not checked.
- The `write-ticket` skill's step 2 uses it instead of the `ls | grep`
  line.
- `cargo xtask ticket-lint`: a ticket in `tickets/done/` whose
  `blocked_by` names a ticket that is in `tickets/open/` is an error
  ("`<path>`: done, but its blocker `NNNN` is still open; drop it from
  `blocked_by` if it was not needed, or finish it first").
- Clear the two existing cases so the lint passes: 0804 and 1050 were
  folded into 1064 (their Completion notes say their open blockers carry
  over), so set their `blocked_by` to `[]` and add one line to each
  Completion notes: "`blocked_by` emptied by 0131; the list carried over
  to 1064".
- The `work-ticket` skill §6: before opening the PR, run
  `cargo xtask ticket-lint` (it already runs in the gates) and, for each
  new ticket the branch adds, `cargo xtask ticket-next` to confirm its
  number is still free; renumber on the branch if not.

**Out (do not do):**
- Turning on "require branches to be up to date" or a merge queue
  (ADR-0042 keeps them off; revisit only if `main` goes red often).
- Checking engine ticket numbers (the engine has its own `write-ticket`).
- A CI job that queries other PRs (it would need a token and be flaky;
  the check belongs where the number is picked).

## Implementation steps

1. **Read** `crates/xtask/src/tickets.rs` (`check_unique_ids`,
   `check_blocked_by_exists`, the `Folder` enum, the tests) and
   `crates/xtask/src/main.rs` (how commands are dispatched and listed in
   the usage text).
2. **`ticket-next`** in `crates/xtask/src/tickets.rs`:
   - a pure function `next_free(block: &str, taken: &BTreeSet<String>) ->
     Option<String>` (the lowest unused `BBNN` above the highest taken in
     the block, `None` when the block is full);
   - the command gathers `taken` from both folders and from
     `gh pr list --state open --json number,files --limit 100`, keeping
     paths that match `tickets/(open|done)/NNNN-*.md`; it prints the
     number and, on a following line, which open PR holds any number in
     the block above the local highest (`0855 is taken by PR #402`).
   - `gh` missing or failing: print the local number and
     `warning: open PRs not checked (<reason>)`, exit 0.
3. **The lint rule**: `check_done_blockers(tickets)` beside
   `check_blocked_by_exists`: for each ticket in `Folder::Done`, each
   `blocked_by` id (not `engine#…`) whose ticket is in `Folder::Open` is
   an error. Wire it into the lint's list of checks.
4. **Fix 0804 and 1050** as in Scope.
5. **Skills**: `write-ticket` step 2 and `work-ticket` §6 as in Scope.
6. **Usage text** in `crates/xtask/src/main.rs`.

Expected size: about 80 production lines (the command about 50, the
rule about 20, wiring 10).

## Acceptance criteria

- [ ] `next_free("03", …)` returns the number after the highest taken,
      and `None` for a full block (unit tests).
- [ ] With a fake PR list holding `tickets/open/0855-x.md` and a local
      highest of 0854 in block 08, `ticket-next 08` answers `0856` and
      names the PR (a test with the `gh` output injected).
- [ ] Without `gh` on `PATH`, `ticket-next` prints the local answer and
      the warning, exit 0.
- [ ] ticket-lint fails on a done ticket blocked by an open one (unit
      test) and passes on the real repository after 0804 and 1050 are
      fixed.
- [ ] `write-ticket` and `work-ticket` name `cargo xtask ticket-next`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `next_free`, the PR-list parser, `check_done_blockers`.
- Integration: `crates/xtask` CLI test for `ticket-next` with a fixture
  PR list (no network in the gates).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
