---
id: "0181"
title: Tickets wait as short stubs or parked, and the estimate is in the ticket before the code
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: high
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

2026-11-17 friction sweep
([report](../../docs/research/friction-2026-11-17.md), *Processes that
slow us down*, "The PR closing lines and the tickets that police them",
and *Should be engine features*, "Ticket lint and pr-size are forked"):

- **Port, don't design.** The engine's lint at the pin (e738ffc,
  `crates/xtask/src/tickets.rs`, ported from the game's) already has a
  parked state and a size estimate: a `Parked:` title prefix with
  `status: blocked` and a `> **Parked**` note under the heading
  (`check_parked`), and an `**Estimated size:** about N lines` line, with
  a `**Split:**` line over 1,000 (`tickets/size.rs`). This ticket ports
  those, not a new `parked` status or an `estimate_lines:` key, so the
  two repositories keep one design. 0147 ports the engine's other checks
  the same way.
- **One job for the closing-line checks.** Of the last 300 merged PRs,
  271 carry `Engine friction:`, 103 of them "none: no code", and 3 of the
  164 with a ratio were 3x or more. Six open tickets (891 lines) police
  these lines. The checks of 0151 (the closing lines), 1174
  (`friction-check`), 1257 (`check-friction` and the warning on a diff
  into code an engine ticket replaces) and 1239 (stale engine-friction
  markers) fold into one command and one CI job here; those tickets
  keep their non-check parts. The estimate stays: it is Nick's rule
  (ADR-0092, `AGENTS.md` rule 9). The `Engine friction:` line is left
  out when the diff changes no production Rust.
- **The rules budget.** 0189 holds `work-ticket` to 150 lines. Either
  order: whichever lands second keeps this ticket's `work-ticket` text
  within it.

Bucket siblings (process and tooling): 0188, 0189, 0190, 0191, 0145,
0147, and the folded 0151, 1174, 1257, 1239.

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
- **Parked**, ported from the engine: a `Parked:` title prefix,
  `status: blocked` and a `> **Parked** (who, date): why` note right
  under the heading, for an open ticket that waits on the Act 1 playtest
  (1064) or on post-launch work. It has a `blocked_by` id or a reason
  naming who unparks it. Picking a ticket already skips `blocked`.
- **Lean notes.** `TEMPLATE.md`'s Completion notes become three short
  parts: deviations from the steps, follow-up ticket ids, and 1256's
  playtest heading. What was done is in the PR body, not here.
- **Estimate first**, ported from the engine's `tickets/size.rs`: an
  `**Estimated size:** about N lines` line in the ticket (with the
  reference it is judged by), and a `**Split:**` line over the game's
  planning size (about 2,000 lines, ADR-0101; the engine's `SPLIT_OVER`
  is 1,000, so make it a constant set to the game's number).
  `work-ticket` §3 writes it in the branch's first commit, before any
  code. The engine's rule (open `feature` or `infra` tickets of effort
  `high` and above) applies to open tickets, and the PR's own ticket
  needs it whenever the diff has production Rust.
- **One PR-checks command and job.** `cargo xtask check-pr` runs, in one
  CI job, the checks folded in from:
  - 0151: the closing lines (`PR size:` present with a measured number
    within 10% of `pr-size`'s count; `Engine friction:` with `estimate
    N`, or absent when `code-lines` counts no production Rust; `Nick
    input:` present), read from `$PR_BODY` through `env:`;
  - 1257: every `docs/engine/friction.md` row has an estimate and a
    measured number, and a warning when the diff touches a file an open
    `engine#`-blocked ticket names as the code it replaces;
  - 1174: `friction-check`, failing when a sweep is due and none is
    filed (with only the milestone trigger if 1174's `Area` column isn't
    there yet; 1174 adds the area count to this command with the column);
  - 1239: the stale-marker check, by running `engine-pin --check` (not
    `--offline`) in the same job; 1239 adds `stale_markers` to it.
- **Lint (warnings, never errors)**, see the steps.
- `pr-size`'s report adds one line splitting the total into `tickets/`,
  `docs/` and the rest. The 2,000 and 3,000 thresholds are unchanged.
- Park the open tickets whose `blocked_by` chain reaches 1064, in this PR.

**Out (do not do):**
- Rewriting or deleting the #723 tickets or the 7 tickets it amended.
  They are parked here and get rewritten when 0085–0088 are decided.
- Taking `tickets/` out of `pr-size`'s count (ADR-0101 counts it on
  purpose: a session reads those lines too).
- 0151's PR template rewrite, 1174's columns, refactor rule, ADR-0092
  amendment and sweep ticket, 1257's `engine-pin` and 1233 text, and
  1239's `stale_markers` function: those tickets keep them.
- Making the lint an engine command for every game (an engine step, only
  with Nick's OK).
- Ticket and ADR numbers across open PRs (0131, 0144).
- Sorting the playtest's starting rules (0183).
- The error for `status: blocked` whose blockers are done (0147); the
  "unpark it" warning here is the same idea for `Parked:`. Either order.
- Ranking ready work (0182): it counts only `todo` tickets, so parked
  ones (`status: blocked`) drop out of it in either order.

## Implementation steps

1. `crates/xtask/src/tickets.rs`: port `check_parked` and the
   `Parked:` / `> **Parked**` constants, and `tickets/size.rs` (the
   estimate and split lines, `leading_count`, the `UNESTIMATED` list for
   open tickets written before the rule), with their tests, from the
   engine at the pin.
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
     the playtest heading, or with no `**Estimated size:**` line when
     the diff has production Rust;
   - a `todo` or `blocked` ticket with a `Parked:` ticket in
     `blocked_by`: "park it too";
   - a `Parked:` ticket whose every `blocked_by` id is done: "unpark
     it".
4. `.github/workflows/ci.yml`, the `ticket-lint` step (line 387 on
   2026-11-17): pass `--base HEAD^1` with `--pr-branch`.
5. `crates/xtask/src/pr_size.rs`: `report` (line 124) prints the
   `tickets/`, `docs/` and other subtotals. `hook_answer` (line 284): when
   the diff touches `crates/*/src/` and the branch's ticket has no
   `**Estimated size:**` line, warn "write the estimate in the ticket
   before more code (ADR-0092)". Make `ticket_id_from_branch` `pub(crate)`.
6. Docs: `tickets/README.md` (a *Parked tickets* section, as the
   engine's, and *Picking the next ticket*), `TEMPLATE.md` (the
   estimate line), `write-ticket` (stub rule, step 5's ticket-size line),
   `work-ticket` §1 (a parked ticket is `blocked`), §3 (the estimate
   line) and §5 (lean notes),
   and an amendment line in ADR-0010's status list. Then run
   `pwsh -File scripts/sync-agent-skills.ps1`.
7. Run the new lint, park the tickets it names (`Parked:` title,
   `status: blocked`, the note; the 15 with 1064 in `blocked_by` and those
   behind them), and list them in the Completion notes.
8. `crates/xtask/src/check_pr.rs`: the four checks of *Scope*, one
   function each, over plain inputs (the body, `friction.md`'s text, the
   diff's paths, the open tickets) so each is unit-tested without git.
   Wire `check-pr` into `main.rs`. In `.github/workflows/ci.yml`, one
   `pr-checks` job (with `edited` in the `pull_request` types, so a body
   fix re-runs it) runs `check-pr` and `engine-pin --check`.
9. `work-ticket` §6: the `Engine friction:` line is left out when the PR
   changes no production Rust (drop "none: no code").

Expected size: steps 1–7 about 200 lines of Rust (less where the
engine's code ports as is) and 150 of tests, 120 of docs and skills,
30–60 one-line status changes; steps 8–9 about 450 of Rust and 400 of
tests. Planned total about 1,800. If the session's estimate passes about
2,000, split steps 8–9 into their own ticket blocked by this one before
coding (`write-ticket` step 5), and name it in 0151, 1174, 1257 and
1239's dated lines.

## Acceptance criteria

- [ ] `ticket-lint` accepts a `Parked:` ticket with `status: blocked`
      and its note, and rejects one without the note; it rejects
      `**Estimated size:** soon`.
- [ ] With `--base`, a new 80-line ticket blocked by an open `00xx` warns;
      the same ticket on `main` does not.
- [ ] The PR's own done ticket with 60 lines of notes, or without an
      estimate on a diff with production Rust, warns; the playtest
      heading's lines don't count.
- [ ] A `todo` ticket behind a parked one warns; a parked ticket with all
      blockers done warns.
- [ ] The `pr-size` hook warns on a branch with code and no estimate.
- [ ] No open ticket that isn't parked has 1064 in its chain.
- [ ] `check-pr` fails a ticket PR body without `PR size:`, one 30% off
      `pr-size`'s count, and a friction row with no measured number; it
      passes a body with no `Engine friction:` line on a docs-only diff.
- [ ] CI's `pr-checks` job runs `check-pr` and `engine-pin --check`.
- [ ] `pwsh -File scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each warning both ways over small ticket strings; the ported
  parked and size checks (with the engine's tests); the chain walk;
  `pr-size`'s subtotals and the hook's estimate warning; each `check-pr`
  check over fixture bodies, friction tables and path lists.
- Integration: `ticket-lint --base` in the existing `crates/xtask` CLI
  tests over a fixture repository.

## Completion notes

*(Filled in by the session that completes the ticket: deviations,
follow-up ticket ids, and the tickets it parked.)*
