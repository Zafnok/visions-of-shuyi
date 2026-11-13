---
id: "0917"
title: "A release's debug files are kept somewhere private"
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0158"]
nick_input: setup
completed: 2026-11-12
---

# 0917 — A release's debug files are kept somewhere private

## Context

`cargo shuyi export <windows|linux> --release` writes `<package>.debug`
beside the package: the program before `objcopy --strip-all`, with its
symbols and line tables, which turns a player's crash report into file
and line numbers (`docs/releasing.md`, "What a release holds"). It is the
whole program, so it embeds everything the game embeds, the game's
private content included (ADR-0117).

Until 0158, `release.yml` attached it to the public GitHub Release. 0158
stopped that: a public repository's releases and workflow artifacts are
anyone's (ADR-0117 §4), so the debug file now stays in its job and is
lost when the job ends. The first crash report from a player who bought
the game (0901) can't be read without it.

## Nick input

**Setup**, only if the chosen place needs one (a new secret or a new
itch.io project): the session gives the exact clicks and waits for
"done". Nothing to decide.

## Scope

**In:**
- Pick a private home for the debug files and record why in
  `docs/releasing.md` (an ADR if it adds a new secret or service).
  Candidates to compare: a release in the private assets repository
  `Zafnok/visions-of-shuyi-assets` (needs a token that can write there,
  as a secret of this repository); a restricted itch.io project of its
  own, never the store's page (butler and `BUTLER_API_KEY` are there
  already, `.github/actions/butler`); something else that is private
  and costs nothing.
- `release.yml`'s package jobs put the Windows and Linux `.debug` there
  on a real release, named `visions-of-shuyi-<version>-<release_name>.debug`,
  straight from the job (never a workflow artifact). A dry run builds
  and checks it and uploads nothing.
- `docs/releasing.md`: where the debug file is and how to fetch one for a
  crash report.

**Out (do not do):**
- Putting a debug file, a package or anything with the content on a
  public release or a workflow artifact.
- The store page and the packages' pushes (0901). Either order works: 0901 can sell before this lands, and this adds its upload beside 0901's push.

## Implementation steps

1. Compare the candidates above (cost, who can read, what secret it
   needs); pick one.
2. Give Nick the setup if any; wait for "done".
3. Add the upload to `release.yml`'s package jobs, after "Check the
   package", only when `needs.check-version.outputs.dry_run == 'false'`.
4. Test it without cutting a release: a by-hand run with a temporary
   input that uploads under a throwaway name, then delete that upload.
5. `docs/releasing.md`.

## Acceptance criteria

- [x] A test run puts a `.debug` in the private place (run linked), and
      nothing with the content is a workflow artifact or on a GitHub
      Release.
- [x] `docs/releasing.md` says where the debug files are and how to
      fetch one.
- [x] zizmor passes; all gates in the `run-gates` skill pass.

## Tests required

- None in Rust. The workflow is checked by its run (linked) and zizmor.

## Completion notes

**Done.**
- **Where:** a release `debug-v<version>` of the private assets repository
  `Zafnok/visions-of-shuyi-assets`, picked over a restricted itch.io
  project (both free): itch.io keeps one build per channel and butler has
  no documented download command, so each version would need its own
  channel and a crash report a visit to the page; the private repository
  already holds what the file embeds, keeps a release per version, and
  `gh release download` fetches one. The comparison is in
  `docs/releasing.md`, "The debug files". No ADR: the secret is a token to
  a repository the project already uses, and the choice is recorded there.
- **Setup (Nick):** the secret `DEBUG_FILES_TOKEN`, a fine-grained token of
  Zafnok's with Contents: Read and write on that repository only (added
  2026-11-12; no expiry date given).
- `scripts/upload-debug-file.sh` (curl and jq, so it runs in the sniper
  container too, which now installs jq): finds or makes the release (the
  two jobs may race: the loser looks again), replaces an asset of the same
  name on a rerun, and checks the uploaded size. No token on a real
  release: it fails.
- `release.yml`: step **Keep the debug file private** after "Check the
  package", Windows and Linux, only when `dry_run == 'false'`, names the
  file `visions-of-shuyi-<version>-<release_name>.debug`.
- **Test run** (a temporary `debug_test_tag` input, removed before merge):
  https://github.com/Zafnok/visions-of-shuyi/actions/runs/43360103728 put
  `visions-of-shuyi-0.3.0-linux-x64.debug` (68.6 MB) and
  `…-windows-x64.debug` (70.7 MB) on the throwaway release
  `debug-test-0917` there. The run's artifacts are `licenses` and the four
  `kept-<target>` (11–12 KB each, the reports); no GitHub Release was made
  (`release` skipped). `gh release download` fetched the Linux one (seven
  `.debug_*` sections); then the release and its tag were deleted.
- zizmor: no new finding (its two on `release.yml` were there before: the
  self-repository syntax and `action-gh-release`).

**Found: today's crash reports can't use the debug file.** The engine
writes a report's `backtrace` with `Backtrace::to_string()`, the short
form: in a stripped build, `<unknown>` frames with no addresses, and no
load address. `docs/releasing.md` says so; follow-up **0918** (an engine
change, which needs Nick's OK before its engine ticket and PR).

**Gates:** no Rust changed; ran the skills sync check, ticket-lint,
check-mentions, check-leaks, pr-size and typos locally; CI runs the rest.
shellcheck isn't installed here.
