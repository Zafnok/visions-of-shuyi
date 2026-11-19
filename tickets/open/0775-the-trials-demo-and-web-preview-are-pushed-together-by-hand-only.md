---
id: "0775"
title: "The trial's demo and web preview are pushed together, by hand only"
type: infra
milestone: M6 Story & dialogue
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0775 — The trial's demo and web preview are pushed together, by hand only

## Context

The writer trial (0766) sends each applicant two password-protected itch.io pages:

- **The Chapter 1 web demo** (0914, 0915).
- **The web script preview**, where they drop their `.dlg` files (0767, the `story-trial` edition of 0771, ADR-0126).

A test audit before the first packets found that the two pages update on different schedules:

- **The preview publishes itself every night.** `.github/workflows/story-kit.yml` runs on `schedule: - cron: "23 4 * * *"` (lines 35-36). Its plan step (lines 86-88) rebuilds when anything under `assets`, `crates/core`, `crates/content`, `crates/ui`, `crates/app`, `crates/story-preview`, `assets-private.rev`, `shuyi-engine.rev`, `Cargo.lock` and more has changed since the last kit. That is nearly every merge. The `story-preview (web)` job then runs `butler push ... "$PROJECT:web"` (lines 269-284) with no human step. Scheduled runs pushed it on 11-15, 11-16, 11-17, 11-18 and 11-19 (run 44533546592).
- **The demo publishes only by hand.** `.github/workflows/demo.yml` pushes only on `workflow_dispatch` with `publish` ticked (line 120). Its last such run was 11-13 (43339431351).

So during a trial week:

- A bad merge reaches every applicant's preview overnight, unannounced.
- The preview and the demo are built from different commits and can disagree, for example on text-box behavior.
- An applicant who reloads the preview gets the new build, with no scenes. The web preview keeps dropped files only in the page's memory (`crates/story-preview/src/main.rs:92-97`), so a reload empties it either way.

The fix: a trial's pages change only when Nick runs one workflow by hand, and both pages come from the same commit.

It doesn't block the writer trial (0766): 0778, a two-line stopgap, makes the preview's push by hand only, so it can't change under the applicants; this ticket then moves it into demo.yml so both pages come from one commit.

Related:
- `docs/releasing.md`: *The demo's itch.io pages* (line 296) and *The script preview's itch.io page* (line 343, whose "Push it" bullet at line 357 describes the nightly push).
- ADR-0117 §4: nothing with the content or the bought art is uploaded as a GitHub artifact, hence one job from the build to itch.io.
- 0175 (open): a PR dry run of story-kit.yml. Either order (see **Out**).

## Nick input

None. When the trial pages are republished is a release-process choice. It changes neither what the game looks like nor how it plays. Verified by the xtask tests below and one by-hand Demo run on the PR's branch with publishing off. Nick's only step afterwards is the same button he already uses for the demo (**Actions → Demo → Run workflow**). The README and `docs/releasing.md` say so.

## Scope

**In:**
- **demo.yml's by-hand run publishes the web preview too.** Besides the demo's web and Windows packages, it builds the `story-trial` web edition, checks it, and pushes it to `ITCH_PREVIEW_PROJECT:web` from the same job layout and the same commit. All three uploads carry the same `+<sha7>` in their user version.
- **Story kit no longer pushes to itch.io.** Its `story-preview (web)` leg stays: it still builds the `story-trial` web edition nightly and checks that it holds no script line, so a break is caught before a packet day. It publishes nothing and says so in a notice. The Windows (and macOS) kit zips and story-check still go to the mirror's release `kit` as today.
- **The script-line check becomes one script.** It moves from story-kit.yml's "The program holds no script line" step (lines 210-232) into `.github/no-script-lines.sh`, used by both workflows.
- xtask tests (below).
- Docs:
  - `docs/releasing.md`'s two itch.io sections.
  - Both workflows' header comments.
  - The writer trial's README line on pushing the preview (`docs/writer-trial/README.md`, if 0766 has merged it; see step 6).

**Out (do not do):**
- **Keeping dropped `.dlg` files across a reload of the web preview.** They live only in the page's memory today. Whether they should persist is a separate question, not this ticket.
- **Freezing or pinning a trial to a commit by any other means** (a repository variable, a release branch). One by-hand run is the only publisher, which is enough.
- **The PR leg of demo.yml.** It stays the demo's web export without the art. It does not export `story-trial`.
- **0172's `assets-private.rev` in demo.yml's PR paths.** That is 0172's job. Either order: neither edits the other's lines.
- **A pull_request dry run of story-kit.yml** (0175). Either order:
  - If 0175 is done, delete the itch steps it skips on `pull_request` along with the rest (its PR-only `if`s on them go too).
  - If not, 0175 has fewer steps to skip. Add a line to 0175 saying the itch steps are gone.
- **The voice check of a package with the bought art** (0929). Either order: both tickets edit demo.yml's `export` job and its *Check the reports* step. Whichever lands second keeps both changes, and the voice check runs only for `matrix.edition == 'demo'`, because the `story-trial` preview ships no voices (step 4 already sets `voice: false` for it).
- **The full game's itch.io pages** (0901), and the playtest page (`playtest-itch.yml`).
- **Building the packet** (0766).

## Implementation steps

1. **Size first (rule 9).** About 450 changed lines:
   - demo.yml: +45
   - story-kit.yml: -40
   - the script: 40
   - xtask tests: 90
   - docs: 30
   - this ticket: 250

2. **`.github/no-script-lines.sh EXPORT_DIR DIALOGUE_DIR`.** Move the body of story-kit.yml's "The program holds no script line" step (lines 210-232) here unchanged in behavior:
   - Read `folder` from `EXPORT_DIR/export-report.json`.
   - Find `story-preview.exe`, `story-preview.wasm` or `*/MacOS/story-preview`, and `trpg-story-preview.pack`.
   - Take the first long speech line of `ch01`, `ch04` and `ch08` from `DIALOGUE_DIR/<name>.dlg` with the same `sed` pattern and `awk 'NR == 1'` (keep the ticket-0769 comment about `head -1`).
   - Exit 1 with the same `::error::` messages if a line is found, or if the program or a line to look for is missing.
   - `set -euo pipefail`, executable bit set.
   - story-kit.yml's step becomes `.github/no-script-lines.sh "dist/export/$EDITION/$TARGET" assets-private/content/dialogue`.

3. **story-kit.yml.**
   - Delete "Can the web preview publish?", the `butler` step and "Push the web preview to itch.io" (lines 250-284).
   - Add one step on the web leg that prints `::notice::The web preview was built and checked, not published: the Demo workflow's run by hand publishes it with the demo (ticket 0775).`
   - Update the header comment (lines 10-16) and the comment above the `preview` job (lines 139-144).

4. **demo.yml.** Turn the `export` matrix into `include` entries:
   - `{target: web, edition: demo, os: ubuntu-latest, project: ITCH_DEMO_PROJECT}`
   - `{target: windows, edition: demo, os: windows-latest, project: ITCH_DEMO_DESKTOP_PROJECT}`
   - `{target: web, edition: story-trial, os: ubuntu-latest, project: ITCH_PREVIEW_PROJECT}`

   A `pull_request` run keeps only the first entry, for example with two `fromJSON` lists chosen by event, as line 60 does today. Then:
   - **Job name:** `export (${{ matrix.edition }}, ${{ matrix.target }})`.
   - **private-assets inputs:** `voice: ${{ matrix.edition != 'story-trial' }}`. The preview ships no voice clips (0769, story-kit.yml lines 169-172). `art` and `content` stay as they are.
   - **cargo-about manifest:** `crates/story-preview/Cargo.toml` for `story-trial`, `crates/app/Cargo.toml` for `demo` (as story-kit.yml line 202 does).
   - **Export:** `cargo shuyi export "$TARGET" --release --edition "$EDITION" $WITHOUT_ART`.
   - **"Check the reports":** reads `dist/export/$EDITION/$TARGET` and greps `"edition": *"$EDITION"`. Keep the `content_root` grep for `demo` only, unless the `story-trial` exposure report names its own root. Check it, don't guess.
   - **Script-line check:** for `story-trial`, run `.github/no-script-lines.sh` before any push.
   - **"Can it publish?" and "Push to itch.io":** use `vars[matrix.project]`.
   - **Upload checks:** the `story-trial` web upload checks `index.html`, as the demo's web one does.
   - **Upload version:** for `story-trial`, the `trpg-story-preview` package version and `--userversion "$version-preview+${SHA:0:7}"`, as story-kit did. The demo's legs keep `trpg-app` and `-demo+`.
   - **Comments:** update the header comment (lines 3-28) to say the by-hand run is the only thing that publishes the trial's two pages, and why (one commit, nothing changes mid-trial on its own).

5. **Tests** in `crates/xtask/src/main.rs`'s tests, beside `no_job_with_private_files_caches_or_uploads_a_build`, reading the workflows as text as `workflow_guard` does:
   - `the_trial_pages_are_pushed_only_from_the_demos_run_by_hand`: across `.github/workflows/*.yml`, every file that names `ITCH_DEMO_PROJECT`, `ITCH_DEMO_DESKTOP_PROJECT` or `ITCH_PREVIEW_PROJECT` is `demo.yml`. In `demo.yml`, the step that runs `butler push` has an `if:` that depends on a step whose `if:` contains `github.event_name == 'workflow_dispatch'`. The failure names the file.
   - `no_script_lines_passes_a_clean_program` and `no_script_lines_fails_on_a_script_line` (`#[cfg(unix)]`, like `levers_output`): write a temp folder holding `export-report.json` with a `folder`, `<folder>/story-preview.wasm` and a fixture `dialogue/` with `ch01.dlg`, `ch04.dlg` and `ch08.dlg`, each with one long speech line. Run `bash .github/no-script-lines.sh <export> <dialogue>`. Exit 0 when the Wasm holds none of the lines; exit non-zero, with `::error::` naming the `.dlg`, when it holds one.

6. **Docs.**
   - `docs/releasing.md`: *The script preview's itch.io page* "Push it" (line 357) now says **Actions → Demo → Run workflow** pushes it with the demo, and Story kit only builds and checks it. *The demo's itch.io pages* "Push them" (line 313) names the third upload. Add one sentence: during a trial nothing republishes either page on its own; a by-hand run changes both for every applicant at once.
   - If 0766 has merged `docs/writer-trial/README.md`, change its line on pushing the preview ("Actions → Story kit → Run workflow pushes it") to the Demo run. If not, add that line to 0766's ticket.

7. **Run it.** On the PR's branch: `gh workflow run demo.yml --ref <branch> -f publish=false`. All three export jobs must be green, and the `story-trial` job's log must show "None of 3 script lines". Run `zizmor` (`security.yml`) on the two changed workflows. After merge, Nick runs the publishing Demo run (**Actions → Demo → Run workflow**, `publish` ticked), the same button he already uses, so the trial's first publish follows this ticket.

## Acceptance criteria

- [ ] `story-kit.yml` contains no `butler` and no `ITCH_PREVIEW_PROJECT` (`grep -c` prints 0 for each).
- [ ] `cargo test -p xtask the_trial_pages_are_pushed_only_from_the_demos_run_by_hand` passes. It fails when the deleted push step is put back into story-kit.yml (tried, said in the Completion notes).
- [ ] `no_script_lines_passes_a_clean_program` and `no_script_lines_fails_on_a_script_line` pass in CI's Linux test job.
- [ ] A by-hand Demo run on the PR's branch with publishing unticked shows `export (demo, web)`, `export (demo, windows)` and `export (story-trial, web)` green. The last one's log has the script-line check's "None of 3 script lines" line, and no push step ran. The run is linked in the Completion notes.
- [ ] The PR's own `pull_request` Demo run shows only `export (demo, web)`.
- [ ] `docs/releasing.md`'s preview section names the Demo workflow's run by hand as the way the web preview is published.
- [ ] `zizmor` passes on `demo.yml` and `story-kit.yml`.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- **Unit (xtask):**
  - The workflow-text test that only demo.yml's by-hand path pushes the trial pages.
  - The two script tests of `.github/no-script-lines.sh` on fixtures.
- **Integration:** the by-hand Demo run on the branch with `publish=false` (CI evidence, linked).
- **Property / snapshot:** none (no rules, no look).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations, follow-up tickets created, notes for Nick.)*
