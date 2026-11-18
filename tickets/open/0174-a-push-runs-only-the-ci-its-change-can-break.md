---
id: "0174"
title: A push runs only the CI its change can break
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0174 — A push runs only the CI its change can break

## Context

The retro of PRs #616–#735 ([T11](../../docs/research/retro-2026-11-17-prs-616-735.md#t11-cheaper-ci-per-push-finer-change-classes-drafts-re-pins-nextest-one-release-wasm-build-pr-caches-restore-only))
found that most CI minutes go to runs the change can't fail:
`changes.yml` counts every push as code (13 docs-only merges ran the full
matrix on `main`; #699, only `.github/` and tickets, ran it 4 times);
drafts run everything (#696: 1,960 job-min); a re-pin of the private
content to the same tree reruns all (#725, #728, #731); the suite runs
twice (`test`, then `coverage` under `llvm-cov`); the web target is built
for release twice (`wasm` and `export`); and pull requests save caches
that evict `main`'s (10.52 GB, over the 10 GB cap).

This ticket amends ADR-0014 (docs-only detection) and ADR-0073 (nextest
only in the mutation gate).

## Nick input

None.

## Scope

**In:**
- `changes.yml`: push events diff their own commits; a `workflows`
  output; a list of workflows no pull-request workflow calls; `LICENSE`
  as docs; a pin move to a private commit with the same tree as docs.
- Drafts skip the heavy jobs; `ready_for_review` starts a full run.
- `test` runs nextest; its Ubuntu leg collects coverage; a small `sonar`
  job replaces `coverage`.
- `wasm` builds a debug package, so the release web build runs once.
- Every `rust-cache` step saves only on `main` (`save-if`).

**Out (do not do):**
- A class for `crates/xtask/**`: `wasm`, `export` and `bench` run
  `cargo shuyi`, which is xtask.
- The playtest's levers and its pin-move paths, `demo.yml`'s paths
  (0172), repeat plays and slow battles (0173). Here `playtest.yml` gets
  only the draft skip. Either order: whichever lands second keeps the
  other's `detect` lines.
- Dry runs of push-only and dispatch-only workflows (0175). Either order:
  see step 1's list.
- What the caches hold and `cache-targets` (0171), the bench and its web
  build (0185), the `ci-result` aggregator's cancelled runs (0150), the
  mutants runs (0150, 0155), the `main-red` issue (0176).

## Implementation steps

1. **`changes.yml` classes** (the `classify` step, lines 44–71):
   - On `push`, diff `github.event.before` to `github.sha` (fetch
     `before` with `git fetch --depth=1`). If `before` is all zeros or the
     fetch fails, `code=true`. `schedule` and `workflow_dispatch` stay
     `code=true`.
   - Add a `workflows` output: `true` when any `.github/**` path changed.
   - Add a case for workflows that no pull-request workflow calls and
     that aren't themselves the PR's checks: `release.yml`,
     `story-kit.yml`, `story-mirror.yml`, `playtest-itch.yml`,
     `engine-watch.yml`, `sonar-triage.yml`, `scorecard.yml`,
     `.github/dependabot.yml` and `.github/actions/butler/*`. They echo
     `workflow, tested by xtask: $path` and add `xtask` to `doc_crates`.
     Don't write `read by`: `every_markdown_file_the_code_reads_is_tested`
     (`crates/xtask/src/main.rs:1073`) parses those lines. If 0175 is
     done, three of them run their own dry run on a PR that changes them;
     they stay on the list.
   - `LICENSE` counts as docs (nothing in `crates/` reads it).
   - **Pin moves.** If `assets-private.rev` is the only non-docs path,
     fetch its old and new commits from `Zafnok/visions-of-shuyi-assets`,
     commit objects only (`git fetch --depth=1 --filter=tree:0`), with the
     deploy key, and compare `git log -1 --format=%T`. Equal root trees
     are docs; no key (a fork) or any error is code. Declare the key as an
     optional `workflow_call` secret, `PRIVATE_ASSETS_KEY`; the four
     callers (`ci`, `codeql`, `security`, `mutants`) pass it by name, not
     `secrets: inherit` (zizmor flags it). 0172 may reuse this fetch.
2. **Who reads `workflows`.** `codeql.yml`'s `analyze` and
   `security.yml`'s `zizmor` run on `code == 'true' || workflows ==
   'true'`.
3. **Push runs on `main` aren't cancelled**, since each diffs only its
   own commits: `ci.yml` (lines 11–13) and `codeql.yml` (13–15) copy
   `playtest.yml`'s concurrency group (keyed by `github.sha` on `push`,
   ticket 0515; 0176's Out points here).
4. **Drafts.**
   - `pull_request` in `ci.yml`, `codeql.yml`, `mutants.yml` and
     `playtest.yml` gets `types: [opened, synchronize, reopened,
     ready_for_review]`.
   - The `test` matrix becomes
     `${{ github.event.pull_request.draft && fromJSON('["ubuntu-latest"]') || fromJSON('["windows-latest","ubuntu-latest","macos-latest"]') }}`.
   - `&& !github.event.pull_request.draft` goes on the `if:` of
     `mutants.yml`'s `plan`, `codeql.yml`'s `analyze`, the new `sonar`
     job and `playtest.yml`'s `detect`.
   - In the `tickets` job's ticket-lint step (lines 337–342), a draft runs
     `ticket-lint` without `--pr-branch`, then with it as a
     `::warning::` only. The aggregators already pass on `skipped`.
5. **nextest.**
   - `.config/nextest.toml` gets a `[profile.ci]` with `fail-fast =
     false`, and its header comment says so.
   - `test` installs `cargo-nextest` with the pinned
     `taiki-e/install-action` (as `mutants.yml` line 116 does).
   - Windows and macOS run `cargo nextest run --workspace --locked
     --profile ci`. Ubuntu runs `cargo llvm-cov nextest --workspace
     --locked --profile ci --exclude-from-report trpg-app
     --exclude-from-report trpg-story-preview --lcov --output-path
     lcov.info`, then `cargo test --doc --workspace --locked` (nextest
     skips doctests), and uploads `lcov.info` (retention 1 day; paths and
     counts, no source text, ADR-0117 §4).
   - The `mutants` baseline already runs nextest on every crate but
     `trpg-app` and `trpg-story-preview`: check those two first, and put
     tests that share a file on disk in a nextest `test-group`.
6. **Sonar.**
   - `clippy` (its run line, line 58) writes `--message-format=json` to
     `clippy.json`, prints the `rendered` messages with `jq`, still fails
     on a warning (capture the status with `if !`, not `$?`, under
     `bash -e`), and uploads `clippy.json`.
   - Replace `coverage` (lines 264–302) with `sonar`: `needs: [clippy,
     test]`, `fetch-depth: 0`, no private checkout; it downloads both
     files and runs the pinned scan. In `ci-result`'s `needs` (line 500),
     `coverage` becomes `sonar`.
7. **One release web build.** `wasm` (line 141) runs `cargo shuyi web`
   without `--release` and keeps the content. `export` stays. If 0185 is
   done, `export`'s steps are in `bench`; that job is then the one.
8. **Restore-only caches.** Every `Swatinem/rust-cache` step (12 in
   `ci.yml`, 4 in `mutants.yml`, 1 each in `playtest.yml`, `release.yml`
   and `engine-watch.yml`, 2 in `story-kit.yml`) gets `save-if: ${{
   github.ref == 'refs/heads/main' }}`. If 0171 is done, keep its lines.
9. **Checks** in `crates/xtask/src/main.rs`'s tests, reading workflows
   as text:
   - `every_rust_cache_step_saves_only_on_main`;
   - `light_workflows_are_called_by_no_pull_request_workflow`: no
     workflow with a `pull_request` trigger `uses:` a workflow or action
     on step 1's list, and a listed workflow's own `pull_request` trigger,
     if any, has `paths` naming only its own files (0175's dry run).
10. **ADRs.** ADR-0014 gets an amendment for steps 1–4 and an updated
    gate table; ADR-0073 a line saying nextest runs the CI tests too.

## Acceptance criteria

- [ ] A throwaway PR changing one comment in `scorecard.yml` (closed
      after) logs `code=false workflows=true`; only `changes`, `tickets`,
      `doc-tests` (xtask), `typos`, `zizmor` and `analyze` run, 15
      job-minutes or fewer (summed in the notes).
- [ ] A throwaway draft PR with a one-line Rust change: `test` runs on
      Ubuntu only; `plan`, `analyze`, `sonar` and the playtest are
      skipped; a `t0174-…` branch with the ticket still open warns, not
      fails; "Ready for review" starts a full run.
- [ ] Two throwaway PRs changing only `assets-private.rev`: one points at
      a scratch private commit with the pin's tree (`git commit-tree
      <pin>^{tree} -p <pin>`, branch `tmp/0174-same-tree`, deleted after)
      and gives `code=false`; one at a different tree gives `code=true`.
- [ ] With `tmp/0174-push` added for a while to `ci.yml`'s push branches,
      a docs-only commit pushed after a code commit logs the
      `before..sha` diff and skips the jobs. The trigger is gone from the
      merged tree.
- [ ] Every `test` leg runs nextest; no `coverage` job; the PR's
      SonarCloud analysis shows coverage (link in the notes).
- [ ] Each test leg's median of 3 runs, before and after, is in the
      notes; the Windows leg is faster (aim: under 10 min).
- [ ] One job builds the release web target.
- [ ] `gh cache list --ref refs/pull/<this PR>/merge` is empty after the
      PR's runs.
- [ ] `cargo test -p xtask every_rust_cache_step_saves_only_on_main` and
      `light_workflows_are_called_by_no_pull_request_workflow` pass, and
      each fails when its rule is broken once (said in the notes).
- [ ] `zizmor` passes on the changed workflows.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the two xtask workflow checks (step 9), each on one passing and
  one failing made-up workflow.
- Property: none.
- Snapshot / integration: the throwaway PR runs above, linked in the
  notes.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
