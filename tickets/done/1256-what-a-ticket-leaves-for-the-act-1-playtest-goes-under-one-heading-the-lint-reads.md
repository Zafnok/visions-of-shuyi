---
id: "1256"
title: What a ticket leaves for the Act 1 playtest goes under one heading the lint reads
type: infra
milestone: "Engine: carve"
model: haiku-4.5 / gpt-6-luna
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-28
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
  gameplay ticket's `## Completion notes

- **The heading** `### For the Act 1 playtest (1064)`, under `## Completion
  notes`, or a line `Nothing for the playtest: <why>`. `tickets/README.md`
  (*No checks after merge on gameplay tickets*), `work-ticket` (§5 and §5b)
  and `write-ticket` (*Nick input*) name it; loaders synced.
- **The reader** (`crates/xtask/src/playtest_notes.rs`): `section` reads the
  text under the heading inside the Completion notes, up to the next `##`
  or `###` heading outside a code fence; `says_nothing` finds the line with
  a reason; `missing` is a `feature`, `content`, `tuning` or `bug` ticket
  with neither. 0183 extends this module for `--by-area`.
- **The warning** (`tickets.rs`, `playtest_warning`): `ticket-lint` prints
  `ticket-lint: warning: …` and still passes. It applies to done tickets
  with `completed:` on or after 2026-11-28 (`PLAYTEST_RULE_START`), so the
  older tickets that had nothing to move under it don't warn. `run` keeps
  its errors-only answer for the tests; `lint` returns errors and warnings.
- **`cargo xtask playtest-notes`** prints each done ticket's id and title
  as a `##` heading, then its section, and a count: 246 tickets today, 0477
  and 0761 among them. 1064's checklist step names it.
- **The move** (a script, not kept): in the 246 done `feature`, `content`,
  `tuning`, `bug` and `infra` tickets whose Completion notes carried an old
  marker ("starting rule", "For 1064", "For the Act 1 playtest", "(1064)"),
  each block with one (a bullet, a bold paragraph with the list it
  introduces, an old `###` section) moved, unchanged, to the new section at
  the end of the notes; an old marker heading became the new one. A check
  over every changed file found no line added or lost and one heading
  each. A block that only says there is none ("starting rules: none")
  stayed in place. `design-decision`, `research` and `playtest` tickets
  were left alone: their mentions are Nick's answers, not notes for play.
- Friction: estimate 120 production lines (the size of `mentions.rs` and
  its wiring, a reader and a command), measured 193 (1.6×), under 2×.
- PR size: estimate 2,700, measured 2,881 (2,300 of them the move in
  `tickets/done/`; it would split cleanly from the code if needed).
- Nothing for the playtest: process tooling only, no change to play.
