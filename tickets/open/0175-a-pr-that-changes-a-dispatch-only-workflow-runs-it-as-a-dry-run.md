---
id: "0175"
title: A PR that changes a dispatch-only workflow runs it as a dry run
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0175 — A PR that changes a dispatch-only workflow runs it as a dry run

## Context

Three workflows never run on a pull request: `story-kit.yml` (by hand and
nightly), `playtest-itch.yml` (push to `main` and by hand) and
`release.yml` (a `v*.*.*` tag and by hand). A change to one is merged
without ever running. In the week of the retro
([report, T13](../../docs/research/retro-2026-11-17-prs-616-735.md)),
#694 added story-kit's web leg with no run on its branch, the nightly
failed, and #699 found a second shipped bug in it: `sed | head -1` under
`set -euo pipefail` (`head` closes the pipe, `sed` gets SIGPIPE, the step
fails). The same pipe is still at `story-kit.yml:85`. Pages was red on
20 pushes in a row for the same reason; it is gone now (ADR-0127).

`demo.yml` already shows the pattern: a path-filtered `pull_request`
trigger that builds and checks, and uploads nothing. A second `[[bin]]`
in `trpg-app` is already caught by #690's xtask test
(`every_export_without_a_bin_can_pick_its_program`), so the export
targets are not part of this ticket.

## Nick input

None.

## Scope

**In:**
- `story-kit.yml`, `playtest-itch.yml` and `release.yml` run on a
  `pull_request` that changes them: they build and check, and push,
  upload or attest nothing.
- The `sed | head -1` at `story-kit.yml:85`.
- An xtask test: every workflow runs on a PR that changes it, or is on an
  allowlist with a reason.

**Out (do not do):**
- Exporting every target and edition on PRs (#690's test covers the bin
  case; the retro dropped the rest).
- Running `private_assets::` on PRs that change the content (0177, a
  `ci.yml` job of its own). Here `playtest-itch.yml`'s PR trigger is its
  own files only.
- Linting `$?` and `PIPESTATUS` in run blocks, and checking for a dispatch
  run on a PR's head SHA (dropped by the retro's verifier).
- The `rust-cache` steps' cache settings: 0171 owns them. Either order
  works: if 0171 is done, keep its guard on story-kit's `build` and
  `preview`; if not, leave the steps as they are (0171's table names them).
- The itch.io push in `release.yml` (0901). Either order: 0901's push
  steps key on `dry_run == 'false'`, so a PR run pushes nothing.

## Implementation steps

1. **The pipe.** At `story-kit.yml:85` replace `| head -1` with
   `| awk 'NR == 1'`, as the preview step does at lines 212–216 (ticket
   0769), with the same one-line comment.
2. **story-kit.yml.**
   - `on:` gets `pull_request: paths:` for `.github/workflows/story-kit.yml`
     and the local actions it uses (`.github/actions/private-assets/**`,
     `binaryen/**`, `butler/**`).
   - `plan`: on `pull_request`, skip the mirror query (no
     `STORY_MIRROR_TOKEN` is needed), print `build=true` and the previews
     without macOS. On a fork's PR (`github.event.pull_request.head.repo.fork`)
     print `build=false` with a notice: the content key isn't there.
   - `build`: skip "Upload to the release kit" (line 124) on
     `pull_request`.
   - `preview`: skip "Zip and upload to the release kit" (line 225) and
     the itch steps from line 245 on `pull_request`. "The program holds no
     script line" still runs.
   - `publish` (line 278): `if: github.event_name != 'pull_request'`.
   - `concurrency` (line 41): PR runs get their own group per ref and
     cancel in progress; the nightly and by-hand runs keep `story-kit`
     with `cancel-in-progress: false` (they write to the mirror).
   - Update the header comment: what a PR run does.
3. **playtest-itch.yml.** `on:` gets `pull_request: paths:` for its own
   file and `.github/actions/private-assets/**` and `butler/**`. The job
   is skipped on a fork's PR (job-level `if`). "Can it publish?" prints
   `ready=false` on `pull_request`, so the content test and the web build
   run and nothing is pushed. Concurrency is already per ref. Update the
   header comment.
4. **release.yml.** `on:` gets `pull_request: paths:` for
   `.github/workflows/release.yml`, `scripts/check-release-package.sh` and
   `scripts/upload-debug-file.sh` (not the shared actions: the macOS leg
   costs ten times the minutes). In `check-version` (line 79) a dry
   run is any event but `push`: `if [ "$EVENT_NAME" != push ]`. The rest
   already keys on `dry_run`: the debug-file upload and `attest` are
   skipped, and `private-assets` is not `required`, so a fork builds with
   the public files. Add one line to `docs/releasing.md`, "A dry run": a
   PR that changes the workflow or its scripts runs one.
5. **The test.** In `crates/xtask/src/main.rs`'s tests, beside
   `every_markdown_file_the_code_reads_is_tested` (line 1073, which reads
   a workflow as text the same way), add
   `every_workflow_runs_on_the_prs_that_change_it`. For each
   `.github/workflows/*.yml`: if its `on:` has `pull_request:` with a
   `paths:` list, the list names the file itself; if it has
   `pull_request:` with no `paths:`, it passes; otherwise the file is in
   a `const NOT_ON_PRS: &[(&str, &str)]` of (file, reason). Fill the list
   from today's files with no PR trigger (`changes.yml`, `engine-watch.yml`,
   `scorecard.yml`, `sonar-triage.yml`, `story-mirror.yml`), one reason
   each (for example `changes.yml` is called by `ci.yml`). If one has no
   honest reason, file a ticket for its dry run with `write-ticket` and
   name it in the reason. The failure names the file. A second test feeds
   it two made-up workflows (one dispatch-only, one with a `paths:` list
   that leaves itself out) and expects both to fail.
6. **Run them.** This ticket's PR changes all three files, so each runs.
   Check in each run's log that every upload, push and attest step is
   skipped.

Expected size: about 60 lines of workflow, 70 of test, 5 of docs.

## Acceptance criteria

- [ ] `story-kit.yml` has no `| head -1` after a `sed`.
- [ ] The PR's checks show Story kit, Playtest page and Release runs on
      the `pull_request` event, all green; the Completion notes link them.
- [ ] In those runs no step that uploads to the mirror, pushes to itch.io,
      uploads a debug file or attests ran (named in the notes).
- [ ] `cargo test -p xtask every_workflow_runs_on_the_prs_that_change_it`
      passes, and fails when `pull_request:` is taken off `story-kit.yml`
      (tried, said in the notes).
- [ ] `zizmor` (`security.yml`) passes on the changed workflows.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the workflow test and its made-up workflows (step 5).
- Property: none.
- Snapshot / integration: the three dry runs on the PR (step 6).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
