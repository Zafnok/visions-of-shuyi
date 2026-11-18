---
id: "0773"
title: The writer-tooling lane waits for the pick, and the story import has run once
type: infra
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0773 — The writer-tooling lane waits for the pick, and the story import has run once

## Context

The retro of PRs #616–#735 ([T24](../../docs/research/retro-2026-11-17-prs-616-735.md#t24-close-the-writer-tooling-lane-at-0766-and-prove-story-import-end-to-end-before-the-trial))
found two things in Lane J (`docs/ROADMAP.md`, *J. A human writer*):

- **The lane kept growing before a writer exists.** Seven PRs added about
  6,500 lines of writer tooling from 2026-11-11 to 11-16 (#678, #682,
  #708, #694, #690, #679, #693) while 0766 was still open. Each shipped a
  gap its ticket never listed and needed a fix PR (#690 for #682's export
  break, #699 for #694's voice clips, #707 and #708 for the trial page).
  The same week only two PRs touched the Act 1 critical path. #616
  remade 41 Chapter 1 clips for lines the writer may rewrite.
- **The approved import has never run.** #679 (ticket 0168) could test
  only the refusal: "GitHub doesn't allow approving your own pull
  request". `cargo xtask story-import` (`crates/xtask/src/story_import.rs`,
  `check_pull` at line 166 wants `OWNER`'s latest review to be
  `APPROVED`) has never pushed an assets branch, and the sync's step that
  closes the mirror's pull request (`.github/workflows/story-mirror.yml`,
  *Close the pull requests now in the game*, lines 141–165) has never
  closed one. The hired writer's first change would be the first run.

The trial itself needs no import: entries come back as files (0766,
draft #696). The import matters from the picked writer's first change,
so this ticket must close before 0766's Setup step 5 adds them to the
mirror.

2026-11-17 friction sweep ([report](../../docs/research/friction-2026-11-17.md),
bucket B2 with 0184): 13 open tickets wait on 0766. Eight list it in
`blocked_by` (0085–0088, 0772, 1073, 1083, 1084) and five say not to
start before the writer's rewrite (0719, 0726, 0755, 0756, 0757). 1064
lists 1073, 1083 and 1084, so the playtest waits on the hire too. The
hire's date (or playing without it) is question 1 of 0184's first
bundle; this ticket's lane rule holds until 0766 closes.

## Nick input

**Decision** (asked with the question pop-up, one question at a time,
options plus "describe your own"; no technical questions):

1. *Should new writer tools wait until you pick a writer?* Cost to put
   in front of him: the numbers above (about 6,500 lines and four fix
   PRs in five days; two PRs on Act 1's path that week; 13 open tickets
   already waiting on the pick). Options:
   **close the lane** (new writer tools, demo polish and voice clips for
   lines the writer may rewrite are filed only when the hired writer or
   Nick asks; fixes for a bug an applicant reports still go ahead);
   **keep it open** as now.
2. *How do we test a writer's change landing once, before the real
   writer's?* Someone other than Nick must open the test change, and a
   second account of his own isn't an option (GitHub's terms allow one
   free personal account per person). Options: **a person he trusts**
   with a GitHub account (about ten minutes of their time, then removed
   from the mirror); **skip the test**: the hired writer's first change
   is the test, and anything broken waits on a fix while they wait.

**Setup** if he picks a person: mirror → **Settings** →
**Collaborators** → **Add people** → their username; they accept the
email. After the run, the same page, **Remove**. Nick approves the test
pull request: **Files changed** → **Review changes** → **Approve** →
**Submit review**.

## Scope

**In:**
- Ask both questions; record the answers (Nick's words) in the
  completion notes.
- If he closes the lane: the rule, as a short paragraph at the end of
  Lane J in `docs/ROADMAP.md`.
- If he picks a person: one real run, mirror pull request → assets pull
  request → game pull request moving `assets-private.rev` → the sync
  closes the mirror's pull request. Fixes it finds, in this ticket's PR
  if small (`story_import.rs`, `story-mirror.yml`, the `story-import`
  skill), otherwise a new ticket.

**Out (do not do):**
- Any change to dialogue words: the test change is one `.dlg` comment
  line (`assets/dialogue/README.md`: `# text` is ignored). Writing pause
  (0766, `AGENTS.md` rule 3).
- A bot-opened test pull request: its author is an app, never a writer,
  so it tests a different path.
- Parking or editing other tickets: no open writer-tooling ticket is
  left (0152, 0168, 0764, 0765, 0914, 0915 are done); 0751 reads scripts
  and goes ahead.
- The retro's "What ships, to whom" template section and the
  `--scene` / `story-preview` overlap (T24 parts 3 and 4).
- Ranking work against 1064 (0182).

## Implementation steps

1. Ask question 1; if "close", write the Lane J paragraph.
2. Ask question 2. If "skip", record it, add a line to the
   `story-import` skill's §2 ("the first real run: expect fixes"), and go
   to step 7.
3. Give the helper the change: on the mirror, edit `dialogue/ch01.dlg`
   in the browser, add one comment line at the top
   (`# Import test, ticket 0773.`), "Create a new branch and start a
   pull request". Wait for the mirror's `check.yml` to pass, then for
   Nick's approval.
4. Follow the `story-import` skill exactly: `--dry-run`, then the real
   run, then the game's pull request (`[story#<n>] …`, `Story-PR: <n>`
   trailer). Check `cargo xtask lines` lists the same line ids before
   and after the pin move (save both outputs and diff them).
5. After Nick merges both, confirm the sync closed the mirror's pull
   request (`gh pr view <n> -R Zafnok/visions-of-shuyi-story --json
   state`) and that the commit author in the assets repository is the
   helper.
6. Each failure: fix it here with a test, or file a ticket.
7. Remind Nick to remove the helper. Completion notes: answers, the
   three pull request links, every fix.

## Acceptance criteria

- [ ] Both answers are in the completion notes, quoted.
- [ ] If the lane closed: `docs/ROADMAP.md` Lane J states the rule.
- [ ] If a person helped: the mirror pull request is closed by the sync,
      its change is on the assets repository's `main`, and the pin
      includes it; the helper is no longer a collaborator.
- [ ] Every fix has a test in `story_import.rs`'s tests or a ticket.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: one per bug the run finds, in `story_import.rs`'s `mod tests`.
- Integration: the run itself, recorded in the completion notes.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
