---
id: "0179"
title: One written rule says what each public place may show of the story
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0179 — One written rule says what each public place may show of the story

## Context

The retro of PRs #616–#735 (T16 in
[`docs/research/retro-2026-11-17-prs-616-735.md`](../../docs/research/retro-2026-11-17-prs-616-735.md))
found that what counts as secret is decided again in each ticket, and
that the written rule and the practice disagree:

- [ADR-0118](../../docs/adr/0118-tests-of-the-real-content-and-what-a-public-file-may-show.md)
  §2 lets a tracked file name a content id (a character id included) and
  forbids only quoted lines, real maps and plot. It leaves tickets, ADRs
  and design documents outside the rule, and `leaks.rs` skips
  `tickets/` (`UNSEARCHED`, line 43).
- The de-naming tickets went further. 0166 (#658) and 0167 (#677) took
  display names out of code and docs, and #658 even reworded an id
  (`defector` → "the recruit"). Yet `defector`, `hound_lieutenant`,
  `greywater_prior` and other ids stay in 28 places outside `tickets/`,
  among them the demo's canaries (`shuyi.ron` lines 1060–1085, copied
  into the story trial by #708). Under the written rule those are fine.
- Display names still reach public text: one lead's name is in 16 open
  tickets and #731's PR title. Nobody has said whether that matters.
- #663 committed six `docs/img/0495-*.png` frames about 2 hours after
  #656 removed every public picture of a real map. Whether they show a
  real map was not checked.

Nick's aim for the private content is "to limit AI copying capability"
(engine `docs/decisions.md`, 2026-11-12; ADR-0117). Old history can't be
pulled back (0153 §3.5), so any rule is forward-looking only.

## Nick input

**Decision**, asked once before anything is written, with the question
pop-up (options plus "describe your own"). It is his call because it is
about which parts of his story he wants kept out of public view:

> From now on, may the names of the story's people and places appear in
> public text: tickets, PR titles, commit messages, code and docs?
> (Script lines, real maps and plot, meaning who dies, joins or betrays,
> stay private in every case. Old tickets and PRs stay as they are.)
>
> - **A.** Yes, everywhere. Only lines, maps and plot are private.
> - **B.** In tickets and PR titles yes, in code and docs no (today's practice).
> - **C.** Nowhere public. Tickets and PR titles use ids (`retainer`, `ch03`).

Everything else (the surfaces, the threat, ids, checks) is the agent's
call. Record his answer, dated and quoted, in the ADR. Nothing to check
after merge.

## Scope

**In:**
- Amend ADR-0118 with the threat model (step 2). No new ADR number.
- A names check, only if Nick picks B or C (step 3).
- Settle the 0495 pictures (step 4).
- One line in `write-ticket` and `content-change` pointing at the table.

**Out (do not do):**
- Rewording old tickets, done PRs, commit messages or history.
- Renaming content ids, canaries included. If Nick's answer makes a role
  id like `defector` private, write a follow-up ticket instead.
- Caches and artifacts: 0171 owns the caches, and 0178 owns ADR-0117
  §4's `playtest-xtask` artifact. The table only points at them (either
  order: the artifacts row says what holds today and names 0178).
- The writer's mirror (ADR-0122) and the demo's contents (ADR-0119).

## Implementation steps

1. Ask Nick the question above. Stop until he answers.
2. In ADR-0118, add **§5 Who it guards against, and each public place**:
   - The threat: a person or AI tool reading the public repository, its
     PRs and its CI. Name Nick's aim and say what B+ can't protect.
   - A table, with a row for each public place and a column for each
     kind of information: script lines, map rows, plot, display names,
     content ids. Places: tracked code and docs (`shuyi.ron` included);
     tickets, ADRs and design docs; PR titles, bodies and commit
     messages; CI logs; artifacts (ADR-0117 §4); caches (0171); old
     history. Each cell says allowed, not allowed, or held by review, and
     which check holds it (`check-leaks`, the step 3 check, or none).
   - Content ids stay allowed everywhere (§2 as written), canaries too.
     Rewording an id to a role, as #658 did, is not needed.
   - The rule applies to new text only. No session goes back to scrub old
     text unless it is quoting a line or drawing a real map.
   - Update §2's sentence on tickets to point at the table. Add "Amended
     by 0179" to the Status line.
3. Only for B or C: add `cargo xtask check-leaks --names` in
   `crates/xtask/src/leaks.rs`. It reads the display names from
   `trpg_content::names::load()` (the private table when the content is
   checked out). It reports a whole-word match by file and line, never
   the name, because CI logs are public (as `command` does now).
   - B: search tracked files outside `tickets/`, `docs/adr/`,
     `docs/design/` and `docs/research/`. Fail, like the quote check.
   - C: also search the open tickets this PR added or changed and the PR
     title. Those are a warning that never fails (ADR-0101's pattern).
   - Run it in `ci.yml`'s `tickets` job, beside "Check the PR title for
     accidental mentions" (line 349) and `check-leaks` (line 353). Pass
     the title through `env`, as `PR_TITLE` is passed now.
   - Expected size: about 120 production lines, close to `leaks()` and
     `command()`.
4. Open the six `docs/img/0495-*.png` and compare them with a
   `cargo shuyi frame-png` of the test chapter. If they show a real map,
   replace them with test-chapter frames cropped to the panel. Then
   fix the paths in `tickets/done/0495-*.md` lines 179–181.
5. Add one line to `.claude/skills/write-ticket/SKILL.md` and
   `.claude/skills/content-change/SKILL.md`: "What a ticket or PR title
   may name: ADR-0118 §5". Then run
   `pwsh -File scripts/sync-agent-skills.ps1`.

## Acceptance criteria

- [ ] ADR-0118 has §5 with Nick's dated answer, the threat and the
      table. Every cell names its check or says "review".
- [ ] B or C: a unit test in `leaks.rs` finds a fixture name and reports
      only the place. With C, a test shows that the ticket and title
      findings are warnings (exit 0).
- [ ] A: no check is added, and §5 says why.
- [ ] No `docs/img/0495-*.png` shows a real map.
- [ ] `write-ticket` and `content-change` point at §5, and
      `sync-agent-skills.ps1 -Check` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the `--names` search (whole words, place only, warning vs
  failure), if built.
- Integration: none. The CI step is the check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
