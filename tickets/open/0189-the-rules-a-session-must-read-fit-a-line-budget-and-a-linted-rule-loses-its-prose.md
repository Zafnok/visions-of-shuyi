---
id: "0189"
title: The rules a session must read fit a line budget, and a linted rule loses its prose
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0145", "0186"]
nick_input: none
completed:
---

# 0189 — The rules a session must read fit a line budget, and a linted rule loses its prose

## Context

The [friction sweep of 2026-11-17](../../docs/research/friction-2026-11-17.md)
(*Processes that slow us down*, "Required rules text doubled in seven
weeks"; *Recommended order* item 5) counted the rules a ticket session
reads before code: `AGENTS.md` 173, `tickets/README.md` 152,
`work-ticket` 267, `run-gates` 131 and `shuyi-gates` 128 lines, 851 in
all. `work-ticket` grew from 84 to 267 lines over 28 commits, and
`run-gates` from 54 to 131 over 33 (`git log` on each `SKILL.md`,
re-counted 2026-11-17). Several rules patch one incident, and some are
now enforced by a lint and still written out in full (the bare-mention
rule in `work-ticket` §6 and `write-ticket` step 3, which `ticket-lint`
and `check-mentions` check).

The same sweep found chapter-battle PRs that also carried a new
mechanic (#554, #588), which makes a battle PR bigger and its test
harder to read (*Too many touchpoints*, the chapter-battle row). That
rule goes into `write-ticket`, and one rule comes out to make room.

Budgets: `work-ticket` at most 150 lines, `run-gates` at most 60. The
retro skill (0186) already says a retro that adds a rule removes or
shortens one, and that a rule which becomes a lint loses its prose; this
ticket does the first cleanup and keeps the budget visible.

Blocked by:

- 0145: its `cargo xtask gates` (`--fast`, `--full`) replaces the hand-typed command list
  in `run-gates` (about 30 commands; 40 of its lines name `cargo`),
  without which 60 lines can't hold it.
- 0186: the retro skill this ticket adds the budgets to.

Either order:

- 0181 rewrites `work-ticket` §1, §3 and §5 (parked tickets, the
  estimate, lean notes). If it lands first, trim its text with the rest.
  If this lands first, 0181 keeps within the budget (0181 says so).
- 0151's closing-line check now folds into 0181, which may shorten §6.

This bucket of the sweep (process and tooling): 0188 (split xtask), this
ticket, 0190 (retire the Sonar issue lane), 0191 (delete the frozen
generators), and the amended 0145, 0147, 0181, with 0151, 1174, 1257 and
1239 folding into 0181.

## Nick input

None. Process rules are technical, so the agent's (`AGENTS.md`, *Who
decides what*); a rule that came from Nick
(for example the estimate before code, ADR-0092, or no checks after merge
on gameplay tickets) stays, shortened if it can be, never removed.

## Scope

**In:**
- Trim `.claude/skills/work-ticket/SKILL.md` to at most 150 lines and
  `.claude/skills/run-gates/SKILL.md` to at most 60, with no rule lost
  that no tool enforces:
  - a rule a lint or hook checks (`ticket-lint`, `check-mentions`,
    `pr-size --hook`, `cargo xtask gates`) becomes a one-line pointer to
    that tool;
  - a rule another skill holds (`shuyi-gates`, `engine-change`,
    `content-change`) becomes a link;
  - incident histories (PR numbers, what happened) move to the ADR or
    ticket they came from, or go.
- `write-ticket`: add "A chapter's new mechanic is its own ticket,
  before the chapter's battle ticket, which is `blocked_by` it." Remove
  one rule to make room: step 3's bare-mention sentences become a
  pointer to `ticket-lint` (which fails on one in an open ticket's
  title). `write-ticket` ends no longer than its 115 lines today.
- The retro skill (`.claude/skills/retro/SKILL.md`, *A rule out for each
  rule in*): name the two budgets.
- A warning, never a failure, when a budgeted skill passes its budget:
  in `scripts/sync-agent-skills.ps1 -Check`, from a small table of skill
  name and line limit at the top of the script.

**Out (do not do):**
- Changing what any rule requires. This is a rewrite for length.
- `AGENTS.md` and `tickets/README.md` (no budget yet; a later retro may
  set one).
- The engine's skill pack (`shuyi-*`), copied at the pin and never
  edited here (ADR-0097).
- The gates runner itself (0145).

## Implementation steps

1. List every rule in `work-ticket` and `run-gates` (one line each) in
   the PR body, marked *kept*, *pointer* (with the tool) or *moved*
   (with where). This is the review: a rule marked *pointer* names the
   check that enforces it.
2. Rewrite the two skills from that list. `run-gates` keeps the
   environment facts this machine needs (MSYS2 on `PATH`, one cargo at a
   time, no local mutants, no local release web export) and names
   `cargo xtask gates --fast` before a push and `--full` to reproduce
   CI.
3. Edit `write-ticket` and the retro skill as in *Scope*.
4. Add the budget table and the warning to `sync-agent-skills.ps1`, then
   run `pwsh -File scripts/sync-agent-skills.ps1` and `-Check`.
5. `wc -l` the five files in the Completion notes, before and after.

Expected size: about 350 changed lines of skills (mostly deletions), 20
of PowerShell.

## Acceptance criteria

- [ ] `wc -l .claude/skills/work-ticket/SKILL.md` is 150 or fewer and
      `run-gates` 60 or fewer.
- [ ] `write-ticket` has the mechanic-before-battle rule and is no
      longer than 115 lines.
- [ ] Every rule removed from the two skills is listed in the PR body
      with the tool or file that now holds it.
- [ ] `sync-agent-skills.ps1 -Check` warns on a skill made one line over
      its budget (tried by hand, noted) and passes as committed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Manual: the budget warning, by adding a line to `run-gates` locally
  and running `-Check`, recorded in the notes.
- The existing `-Check` (loaders current) still passes.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
