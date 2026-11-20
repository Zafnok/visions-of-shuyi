---
id: "0778"
title: "The web preview is pushed to itch.io only by hand"
type: infra
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0778 — The web preview is pushed to itch.io only by hand

## Context

The writer trial (0766) sends applicants to two itch.io pages: the Chapter 1
web demo and the web script preview. The demo is pushed only when Nick runs
**Actions → Demo → Run workflow** (`.github/workflows/demo.yml`, the push is
gated on `workflow_dispatch` and `publish`). The preview is pushed by
`.github/workflows/story-kit.yml` on every run that builds the kit,
including the nightly `schedule` (`cron: "23 4 * * *"`, line 36): its
*Push the web preview to itch.io* step (line ~270) is gated only on
`steps.itch.outputs.ready == 'true'`.

So during the trial, any merge to `main` reaches every applicant's preview
overnight, unannounced, on a different commit from the demo they play.
Nick, 2026-11-19: get the trial into the writers' hands as soon as
possible; only blockers that can be done in under a day.

This ticket is the stopgap: the preview is pushed only when someone runs
Story kit by hand. 0775 (pushing the demo and the preview together from one
Demo run) stays the full fix and stops blocking the trial.

## Nick input

None. Verified by the test below and by the next scheduled Story kit run
(its log shows the push step skipped).

## Scope

**In:**
- `story-kit.yml`: the itch.io push steps (the butler install and *Push the
  web preview to itch.io*, both gated on `steps.itch.outputs.ready`) also
  require `github.event_name == 'workflow_dispatch'`. The nightly run still
  builds and checks the web preview; it only stops publishing it.
- `docs/releasing.md`, *The script preview's itch.io page*: the "Push it"
  bullet says the push happens only on **Actions → Story kit → Run
  workflow**, and that a trial keeps the page still between Nick's runs.
- The trial's `docs/writer-trial/README.md` (if it is on `main`; otherwise
  0766's branch has it): run Story kit by hand right after the Demo run, on
  the same day, before sending packets.

**Out:**
- Moving the preview push into demo.yml, one commit for both pages (0775).
- Any change to the kit's release zip or the nightly build itself.

## Implementation steps

1. Expected size: about 4 changed lines of workflow, 10 of docs, 15 of test.
2. Add `&& github.event_name == 'workflow_dispatch'` to the `if:` of the two
   itch steps in `story-kit.yml`.
3. A test in the xtask workflow guard (`crates/xtask/src/workflow_guard.rs`
   or beside it): every step of `story-kit.yml` whose `run` calls
   `butler push` has an `if:` naming `workflow_dispatch`.
4. Update `docs/releasing.md` as in Scope, and 0775's Context with one line:
   "0778 made the push by hand only; this ticket moves it into demo.yml."
5. Run the `run-gates` skill.

## Acceptance criteria

- [ ] `story-kit.yml`'s itch.io push steps run only on `workflow_dispatch`.
- [ ] The test from step 3 passes, and fails if the condition is removed.
- [ ] `docs/releasing.md` says the preview is pushed only by hand.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (xtask): the push steps' condition names `workflow_dispatch`.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
