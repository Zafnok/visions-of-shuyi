---
id: "1256"
title: What a ticket leaves for the Act 1 playtest goes under one heading the lint reads
type: infra
milestone: "Engine: carve"
model: haiku-4.5 / gpt-6-luna
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1256 — What a ticket leaves for the Act 1 playtest goes under one heading the lint reads

## Context

Gameplay tickets have no check after merge (`tickets/README.md`). What
is left to judge by playing goes to the Act 1 playtest, ticket 1064,
which collects it by grepping done tickets for a starting-rule marker.
The retro of 2026-11-08 found three different markers in use:

- #526 wrote "For 1064:", which 1064's grep would miss;
- #564 (0761) used neither of the phrases 1064 greps for;
- others used "Claude's starting rule" or "starting rules".

So some of what the playtest should check never reaches it.

## Nick input

None.

## Scope

**In:**
- One heading, `### For the Act 1 playtest (1064)`, under a done
  gameplay ticket's `## Completion notes`. `write-ticket`, `work-ticket`
  and `tickets/README.md` name it.
- `ticket-lint` warns on a `feature`, `content`, `tuning` or `bug` ticket
  moved to `done/` without the heading or a line `Nothing for the
  playtest: <why>`.
- `cargo xtask playtest-notes` lists every done ticket's section, and
  1064's text points at it instead of a grep.
- Move the existing notes found under the old markers (`grep -rln
  "starting rule\|For 1064" tickets/done`) under the new heading.

**Out (do not do):**
- The playtest itself (1064).

## Implementation steps

1. Add the warning to `crates/xtask/src/tickets.rs`.
2. Add `playtest-notes` printing ticket id, title and the section.
3. Edit the done tickets found, and 1064's steps.
4. Update the skills, then run `pwsh -File scripts/sync-agent-skills.ps1`.

## Acceptance criteria

- [ ] `cargo xtask playtest-notes` lists 0761's and 0477's notes.
- [ ] A gameplay ticket moved to `done/` with neither line warns.
- [ ] 1064 names the command.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the section reader and the warning.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
