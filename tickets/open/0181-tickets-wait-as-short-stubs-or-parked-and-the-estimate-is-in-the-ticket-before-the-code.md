---
id: "0181"
title: Tickets wait as short stubs or parked, and the estimate is in the ticket before the code
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1256"]
nick_input: none
completed:
---

# 0181 — Tickets wait as short stubs or parked, and the estimate is in the ticket before the code

## Context

The [retro of PRs #616–#735](../../docs/research/retro-2026-11-17-prs-616-735.md)
(T22) found tickets too heavy for what they carry:

- Ticket markdown was 31% of the window's added lines. #723 wrote 16
  build tickets (1091–1099, 1275–1281), +2,939 lines, on four open
  decisions (0085–0088) that wait on 0766; each will be rewritten when
  its decision lands.
- Completion notes restate the PR body: half of done tickets have 37 or
  more non-blank lines of them (1249: 136 lines for a 7-line change).
- The ADR-0092 estimate is often written after the code (#726, #634,
  #676).

So: a ticket that waits on an open decision is a stub, a ticket that waits
on the Act 1 playtest is parked, completion notes hold only what the PR
body doesn't, and the code estimate goes in the ticket's front matter
before the first line of code.

1256 comes first: it adds `ticket-lint`'s warnings and the
`### For the Act 1 playtest (1064)` heading that this ticket leaves out
of the notes count.

## Nick input

None.

## Scope

**In:**
- **Stubs.** `write-ticket`: a ticket whose `blocked_by` chain reaches an
  open `00xx` ticket is a stub of about 15–40 lines: front matter,
  Context (what it builds, which decision shapes it), Scope **In** at the
  level no answer can change, and "Steps: written when 00xx is decided".
  No snapshot of today's code. If Nick asks for full tickets, write them;
  the lint only warns.
- **Parked.** A new status, `parked`, for an open ticket that waits on the
  Act 1 playtest (1064) or on post-launch work. It has a `blocked_by` id
  or a reason naming who unparks it. Picking a ticket skips it.
- **Lean notes.** `TEMPLATE.md`'s Completion notes become three short
  parts: deviations from the steps, follow-up ticket ids, and 1256's
  playtest heading. What was done is in the PR body, not here.
- **Estimate first.** An optional `estimate_lines:` front-matter key (a
  number, or `none` for a ticket with no code). `work-ticket` §3 writes
  it, with the reference in one line of the ticket, in the branch's first
  commit, before any code.
- **Lint (warnings, never errors)**, see the steps.
- `pr-size`'s report adds one line splitting the total into `tickets/`,
  `docs/` and the rest. The 2,000 and 3,000 thresholds are unchanged.
- Park the open tickets whose `blocked_by` chain reaches 1064, in this PR.

**Out (do not do):**
- Rewriting or deleting the #723 tickets or the 7 tickets it amended.
  They are parked here and get rewritten when 0085–0088 are decided.
- Taking `tickets/` out of `pr-size`'s count (ADR-0101 counts it on
  purpose: a session reads those lines too).
- The PR body's closing lines and their check (0151).
- Ticket and ADR numbers across open PRs (0131, 0144).
- Sorting the playtest's starting rules (0183).
- The error for `status: blocked` whose blockers are done (0147); the
  "unpark it" warning here is the same idea for `parked`. Either order.
- Ranking ready work (0182): it counts only `todo` tickets, so parked
  ones drop out of it in either order.

## Implementation steps

1. `crates/xtask/src/tickets.rs`: add `"parked"` to `VALID_STATUSES`
   (line 50). Parse `estimate_lines` beside `status` in `parse_ticket`:
   if present it is a positive integer or `none`, else an error.
2. Add `--base <ref>` to `ticket-lint` (`parse_ticket_lint_args` in
   `crates/xtask/src/main.rs`). With it, list the tickets the PR adds or
   moves (`git diff --name-status <base>` over `tickets/`, as `pr_size.rs`
   uses `code_lines::git`).
3. Warnings, on 1256's channel:
   - a ticket the PR adds to `open/` with more than 50 lines whose
     `blocked_by` chain (followed through open tickets) reaches an open
     `00xx`: "write a stub";
   - the PR's own ticket (`ticket_id_from_branch`, `tickets.rs` line 454) moved to
     `done/` with more than 40 non-blank lines of Completion notes outside
     the playtest heading, or with no `estimate_lines`;
   - a `todo` or `blocked` ticket with a `parked` ticket in `blocked_by`:
     "park it too";
   - a `parked` ticket whose every `blocked_by` id is done: "unpark it".
4. `.github/workflows/ci.yml` line 341: pass `--base HEAD^1` with
   `--pr-branch`.
5. `crates/xtask/src/pr_size.rs`: `report` (line 124) prints the
   `tickets/`, `docs/` and other subtotals. `hook_answer` (line 284): when
   the diff touches `crates/*/src/` and the branch's ticket has no
   `estimate_lines`, warn "write `estimate_lines:` in the ticket before
   more code (ADR-0092)". Make `ticket_id_from_branch` `pub(crate)`.
6. Docs: `tickets/README.md` (status list and *Picking the next ticket*),
   `TEMPLATE.md`, `write-ticket` (stub rule, step 5's ticket-size line),
   `work-ticket` §1 (skip `parked`), §3 (the field) and §5 (lean notes),
   and an amendment line in ADR-0010's status list. Then run
   `pwsh -File scripts/sync-agent-skills.ps1`.
7. Run the new lint, set `status: parked` on the tickets it names (the 15
   with 1064 in `blocked_by` and those behind them), and list them in the
   Completion notes.

Expected size: about 200 lines of Rust and 150 of tests, 120 of docs and
skills, 30–60 one-line status changes. Planned total about 800.

## Acceptance criteria

- [ ] `ticket-lint` accepts `status: parked` and rejects
      `estimate_lines: soon`.
- [ ] With `--base`, a new 80-line ticket blocked by an open `00xx` warns;
      the same ticket on `main` does not.
- [ ] The PR's own done ticket with 60 lines of notes, or without
      `estimate_lines`, warns; the playtest heading's lines don't count.
- [ ] A `todo` ticket behind a parked one warns; a parked ticket with all
      blockers done warns.
- [ ] The `pr-size` hook warns on a branch with code and no estimate.
- [ ] No open `todo` or `blocked` ticket has 1064 in its chain.
- [ ] `pwsh -File scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each warning both ways over small ticket strings; the
  `estimate_lines` parse; the chain walk; `pr-size`'s subtotals and the
  hook's estimate warning.
- Integration: `ticket-lint --base` in the existing `crates/xtask` CLI
  tests over a fixture repository.

## Completion notes

*(Filled in by the session that completes the ticket: deviations,
follow-up ticket ids, and the tickets it parked.)*
