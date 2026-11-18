---
id: "0183"
title: The Act 1 playtest's starting rules are sorted by area and by whether play shows them
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1256"]
nick_input: none
completed:
---

# 0183 — The Act 1 playtest's starting rules are sorted by area and by whether play shows them

## Context

From the retro of PRs #616–#735
([T23](../../docs/research/retro-2026-11-17-prs-616-735.md#t23-a-starting-rules-ledger-triaged-before-the-act-1-playtest)). Gameplay
tickets have no check after merge. Each one leaves its *agent's starting
rules* for the Act 1 playtest (1064), which builds Nick's checklist from
every done ticket. The rules are recorded; the problem is volume. 244
done tickets mention starting rules, and 1064 has 161 ids in
`blocked_by`. One night's PRs (#717, #719, #720, #721) added 22 rules.
Many of them can't be seen in play (a log line, a file's name, a key
reused on a screen with no conflict). A checklist of several hundred
items, most invisible, will not get real answers.

The verifier's corrections stand: #725's Playdate name cut and the key
reuse in #643/#648 were already listed as starting rules, and #624's
volume 80 → 97 is the same loudness on the new curve (`options.md`
says so). So this ticket is about sorting, not about missed items.

1256 comes first: it puts every ticket's notes under one heading,
`### For the Act 1 playtest (1064)`, moves the old markers there, and
adds `cargo xtask playtest-notes`. This ticket sorts what that command
lists.

## Nick input

None. Nick still sees every rule on 1064's page; this ticket only
orders them. What is left to judge by playing stays on 1064's
checklist.

## Scope

**In:**
- Two labels inside a ticket's `### For the Act 1 playtest (1064)`
  section: `Seen in play:` and `Not seen in play:`, each followed by
  its list. A section with one kind needs one label.
- An optional `Area: docs/design/<doc>.md` line in the section. Without
  it, the area is the first `docs/design/*.md` the ticket links, else
  `other`.
- `cargo xtask playtest-notes --by-area`: rules grouped by area, then
  by label, with a count per area and per label, and a list of sections
  with unlabeled items.
- A `ticket-lint` warning (never an error) on a done gameplay ticket
  whose section has items under neither label.
- One pass over `tickets/done/` that adds the labels to every section
  1256 moved.
- `work-ticket` and `write-ticket` say to use the labels.

**Out (do not do):**
- A generated ledger file checked into the repository. Every gameplay
  PR would edit it, and parallel PRs would conflict on it. The command
  prints it instead.
- The heading, the old-marker migration and `playtest-notes` itself
  (1256).
- A "settled" mark. 1064 already records Nick's answers in the design
  doc that holds the rule.
- A rule for flagging a departure from a mockup Nick saw. The verifier
  found those are already listed.
- Changing any rule, or deciding one. Rules stay the agent's starting
  rules until Nick answers.
- Running the playtest (1064). 1064 builds its checklist from
  `--by-area` (its text says so once this lands).
- Lean completion notes, stubs and `parked` (0181). Either order: both
  edit the Completion notes text of `TEMPLATE.md` and the two skills;
  whichever lands second keeps the other's lines, and the labels go
  inside 1256's heading either way.

## Implementation steps

1. Read the module 1256 added for `playtest-notes` (in
   `crates/xtask/src/`, beside `tickets.rs`). Extend its section reader
   to return items with their label (`SeenInPlay`, `NotSeen`,
   `Unlabeled`) and the section's area.
2. Area: the `Area:` line if present, else the first match of
   `docs/design/[a-z0-9-]+\.md` in the whole ticket file, else `other`.
   Use the file stem (`options`, `controls`) as the group name.
3. Add `--by-area` to `playtest-notes` in `crates/xtask/src/main.rs`
   (dispatch near line 103, help text near line 54). Print a summary
   table first (area, seen, not seen, unlabeled), then each area's
   items with the ticket id.
4. In `crates/xtask/src/tickets.rs`, beside 1256's missing-heading
   warning, warn on a section with unlabeled items. Keep it a warning:
   process rules nudge, they never fail CI.
5. Label the done tickets. For each section, decide whether a player
   can see the rule on screen, hear it or feel it in play. When unsure,
   use `Seen in play:`. Add `Area:` where the inferred area is wrong.
6. Update `.claude/skills/work-ticket/SKILL.md` (the Completion notes
   step near line 218) and `.claude/skills/write-ticket/SKILL.md` to
   name the labels. Run `pwsh -File scripts/sync-agent-skills.ps1`.
7. Paste the summary table from step 3 into the Completion notes.

Expected size: about 200 lines of Rust and 120 of tests; the labeling
pass about 600–1,000 changed lines in `tickets/done/`. Planned total
about 1,600. If the labeling pass runs past about 1,500, split it by
ticket block into a second ticket blocked by this one.

## Acceptance criteria

- [ ] `cargo xtask playtest-notes --by-area` prints counts per area and
      label, and lists no unlabeled section.
- [ ] A done gameplay ticket with an unlabeled item makes `cargo xtask
      ticket-lint` warn and still pass.
- [ ] 0498's rule on which weapon name the Playdate cuts (#725) is
      listed under `Seen in play:`.
- [ ] The work-ticket and write-ticket skills name both labels, and
      `pwsh -File scripts/sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the label and area reader (an `Area:` line, a linked design
  doc, neither; both labels; unlabeled items) and the lint warning.
- Integration: `playtest-notes --by-area` on a fixture folder of three
  done tickets prints the expected table.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
