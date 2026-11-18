---
id: "0171"
title: The game's workflows cache no build that embeds its private content
type: bug
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-17
---

# 0171 — The game's workflows cache no build that embeds its private content

## Context

The game's repository is public; its content (the scripts, chapters,
battles, maps, data) is in the private repository so that it can't be
copied, by AI tools among others (ADR-0117; Nick's aim is in the engine's
`docs/decisions.md`, 2026-11-12, "to limit AI copying capability").
`crates/content/build.rs` embeds `assets-private/content/` with
`include_bytes!` whenever it is checked out, so every build made in a job
that ran `./.github/actions/private-assets` with `content: "true"` holds
the content: `trpg-content`'s rlib, its `OUT_DIR` (`onebit-content/`),
every test binary and program linked with it, and `target/playdate/`.

ADR-0117 §4 keeps those builds out of **artifacts**. It says nothing of
the **Actions cache**. Those jobs then run `Swatinem/rust-cache`, which
saves `target/` on every push to `main`, and a pull request's run may
restore the caches of its base branch, **a fork's pull request too**: its
changed workflow can restore the cache with no key and print or send what
it holds. On 2026-11-17 `gh cache list` showed 35 caches, 23 of them on
`refs/heads/main` (10.5 GB). The repository asks approval only for a
first-time contributor's runs (`fork-pr-contributor-approval`:
`first_time_contributors`), so a second PR runs unasked.

The engine closed this for new games' templates in engine#2596 (engine
PR #397): the template's `rust-cache` step sets
`cache-targets: ${{ hashFiles('.github/actions/private-files/action.yml') == '' }}`,
no `target/` at all when the game has private files. It left the game out
("its own ticket there, if needed"), and none was written; the engine's
retro of 2026-11-17 (PRs #379–#484) found it.

**What is cached today is not settled.** `rust-cache` v2.9.3 (the pin
here), before saving, cleans `target/` (`src/cleanup.ts`,
`cleanTargetDir`): it deletes every file outside a profile's `build/`,
`.fingerprint/` and `deps/`, and in those keeps only the packages whose
`Cargo.toml` is outside the workspace root, because
`cache-workspace-crates` and `cache-all-crates` default to `false`. So by
default `trpg-content` and every workspace crate's build are deleted
before the save, and `target/playdate/` with them. But the cleaning's
errors are swallowed (`catch { core.debug(...) }`), the workspace test is a
`startsWith` on paths (the Windows and macOS `test` legs cache too), and
one input (`cache-workspace-crates: "true"`, `cache-all-crates: "true"`),
an `actions/cache` step on `target/`, or a git dependency whose build
script reads the game's files would put the content in the cache with no
test failing. Step 1 settles what the caches on `main` hold; the guard of
step 3 holds either way.

Jobs that check out the content and then run `rust-cache` (verified at
`origin/main` `e7d193dc`):

| Workflow | Jobs | Events that save to `main`'s scope |
| --- | --- | --- |
| `ci.yml` | `clippy`, `test` (Windows, Linux, macOS), `doc-tests`, `wasm`, `no-std`, `playdate`, `docs`, `coverage`, `tickets`, `bench` (`key: bench`) | push to `main` |
| `playtest.yml` | `build` (`play` has no `rust-cache`) | push to `main`, by hand |
| `mutants.yml` | `baseline`, `pr`, `weekly-baseline`, `weekly` | schedule, by hand |
| `story-kit.yml` | `build` (`key: story-kit`), `preview` (`key: story-kit-preview`; it checks out the **bought art** too, `art` defaults to `"true"`) | schedule, by hand |

Not in it: `fmt`, `export` (`key: export`) and `engine-watch.yml`'s
`watch` run `rust-cache` without the content (ADR-0117 §3, ticket 0158);
`release.yml`, `demo.yml` and `playtest-itch.yml` run no `rust-cache`;
`story-mirror.yml` builds nothing. The game runs its own `playtest.yml`,
not the engine's reusable one (whose `playtest` job already has 2596's
guard).

Artifacts, checked at the same commit: `ci.yml`'s `export` (no content),
`bench` (`dist/bench`, the numbers), `mutants.yml`'s `mutants.out` and
`playtest.yml`'s battle reports hold no build. One does:
`playtest.yml`'s `build` uploads `target/release/xtask`, which links
`trpg-content` and so embeds the content, kept one day. ADR-0117 §4
accepted it knowingly; this ticket leaves it (Out), and its check names
it as the one allowed.

## Nick input

None. Deleting caches needs `actions: write`, which `gh` has on Nick's
machine (`gh cache list` works there). If the session's `gh` is refused,
the step's exact commands go to Nick as a **Setup** in the PR's
description.

## Scope

**In:**
- Step 1: find out whether any cache on `main` holds a workspace crate's
  build or a content file, on every OS that caches.
- If one does (case A): every job in the table sets
  `cache-targets: false` (the registry and git caches stay), and every
  cache that holds it is deleted.
- If none does (case B): the target caches stay; the jobs in the table
  state the cleaning they rely on.
- In both cases: a test that fails a workflow job that checks out private
  files and caches or uploads a build in a way that keeps them, and
  ADR-0117 §4 says "no artifact **and no cache**".

**Out (do not do):**
- The engine and its templates (engine#2596 did them).
- `playtest.yml`'s `playtest-xtask` artifact (ADR-0117 §4 accepted it; a
  ticket of its own if it should change).
- Logs that quote a line of the content (ADR-0117 §4 accepted them).
- The fork approval setting (`first_time_contributors`): a repository
  setting is Nick's.
- Caching `target/` some other way (an encrypted cache, `sccache` with a
  secret): only if case A's cost (step 2) is judged too high, then a
  ticket of its own.

## Implementation steps

1. **What the caches hold.** On the ticket's branch, add a temporary
   workflow, `.github/workflows/cache-audit.yml`, on `pull_request`, with
   `permissions: actions: read, contents: read`. Its jobs restore main's
   caches as a fork's run would, with no key, and print **counts only**,
   never a file's name or contents:
   - one job per cache to check, on the OS that made it (at least
     `clippy`, `test` on each of the three, `wasm`, `playdate`, `bench`,
     `mutants.yml`'s `baseline`, `story-kit-preview` if on `main`);
   - each uses `actions/cache/restore` with the cache's exact `key`
     (`gh cache list --ref refs/heads/main --json key`) and the same
     `path` list `rust-cache` saved (its run log prints it under "Cache
     Configuration": the registry, git and `target` paths, in that
     order; the cache's version is a hash of the paths, so they must
     match), with `fail-on-cache-miss: true`;
   - then counts `find target \( -name 'trpg_*' -o -name 'trpg-*' -o
     -name 'libtrpg_*' -o -name 'visions-of-shuyi*' -o -name 'xtask*' \)`
     and the files under any `out/onebit-content`;
   - and looks for the content's bytes: it runs
     `./.github/actions/private-assets` (`art: "false"`, `content:
     "true"`, `required: "true"`), takes a needle from the content at run
     time (`needle=$(head -c 64 "$(ls assets-private/content/battles/*.ron | head -1)")`;
     never written into the workflow, which is public) and prints only
     `grep -rqaF -- "$needle" target && echo found || echo absent`.

   Record each cache's counts in the Completion notes. Delete the
   workflow before the PR is ready (it must not reach `main`). Any count
   above 0, or "found", is case A; all 0 and "absent" is case B.
2. **Case A.**
   - In every job of the table, the `rust-cache` step gets
     `cache-targets: false` and a comment: the job checks out the
     private content (ADR-0117), a build here embeds it, and a fork's
     pull request can restore `main`'s cache (this ticket).
   - Delete the caches that hold it, after the PR merges (a run of the
     old workflow would save them again before):
     ```bash
     gh cache list --repo Zafnok/visions-of-shuyi --ref refs/heads/main --limit 100 --json id,key \
       --jq '.[] | select(.key | test("^v0-rust-(clippy|test|doc-tests|wasm|no-std|playdate|docs|coverage|tickets|bench|build|baseline|pr|weekly-baseline|weekly|story-kit)")) | .id' \
       | xargs -r -n1 gh cache delete --repo Zafnok/visions-of-shuyi
     ```
     and once more for every `refs/pull/*/merge` cache made before the
     merge (`--ref refs/pull/<n>/merge`), or `gh cache delete --all
     --repo Zafnok/visions-of-shuyi` if simpler (every cache rebuilds).
   - Measure the cost: each job's duration on `main` in the three runs
     before the merge and the three after
     (`gh run list --workflow ci.yml --branch main --json databaseId`,
     then `gh run view <id> --json jobs`); the table goes in the
     Completion notes and, in one line each, in the comments of the
     slowest jobs.
3. **Case B.** Each `rust-cache` step in the table gets
   `cache-workspace-crates: "false"` and `cache-all-crates: "false"`
   written out (the defaults, now stated), with a comment: the job
   checks out the private content (ADR-0117); `rust-cache` deletes the
   workspace crates' builds, which embed it, before saving, and a fork's
   pull request can restore `main`'s cache (this ticket). No cache is
   deleted.
4. **The check.** A unit test in `crates/xtask/src/main.rs`'s tests,
   beside `every_markdown_file_the_code_reads_is_tested` (which reads
   `changes.yml` the same way, as text), named
   `no_job_with_private_files_caches_or_uploads_a_build`:
   - for every `.github/workflows/*.yml`, split the `jobs:` map into jobs
     by their two-space-indented keys;
   - a job **has private files** when it contains
     `uses: ./.github/actions/private-assets`;
   - for each such job, every `Swatinem/rust-cache` step's `with:` must
     have `cache-targets: false` (case A) or both stated defaults (case
     B), and neither `cache-workspace-crates: "true"` nor
     `cache-all-crates: "true"`;
   - it has no `actions/cache` step (or `actions/cache/save`) whose
     `path` names `target`;
   - it has no `actions/upload-artifact` step whose `path` is under
     `target/` or `dist/`, except the one ADR-0117 §4 allows,
     `playtest.yml` job `build`, path `target/release/xtask`, named in a
     constant with that reference;
   - the failure names the workflow, the job and the step.

   Then a second test feeding it three small made-up workflows (one
   passing, one with an unguarded `rust-cache`, one uploading
   `target/release/x`) so the check itself is tested.
5. **ADR-0117 §4.** Add, as an amendment dated with the PR: the cache
   counts as public as an artifact does; what each case keeps; this
   ticket and engine#2596. Update ADR-0117 §3's job list if it changed.
   In `.claude/skills/content-change/SKILL.md` or the game's
   `run-gates` rules (whichever lists CI's rules on the content), one
   line: a new job that checks out the content follows the test of
   step 4.

## Acceptance criteria

- [x] The Completion notes list, for each cache checked in step 1, its
      key, OS and counts, and say which case followed.
- [x] `.github/workflows/cache-audit.yml` is not in the merged tree.
- [ ] Case A (after the merge: the deletion): every job in the table has `cache-targets: false`; `gh
      cache list --ref refs/heads/main` after the merge and the deletion
      shows no `v0-rust-` cache older than the merge commit for those
      jobs; the duration table is in the Completion notes.
- [ ] Case B: every job in the table states both defaults with the
      comment.
- [x] `cargo test -p xtask no_job_with_private_files` passes, and fails
      when the guard is taken off one job (tried, and said in the notes).
- [x] ADR-0117 has the amendment.
- [x] `zizmor` (security.yml) still passes on the changed workflows.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the workflow check and its made-up workflows (step 4).
- Property: none.
- Snapshot / integration: step 1's audit run, once, on the PR.

## Completion notes

**Step 1, the audit (case A).** A temporary `cache-audit.yml` (run
44205119959 on PR 741, then deleted in the same PR) restored each of
`main`'s rust-cache caches by its exact key, with the paths rust-cache
logged (`~/.cargo/{bin,.crates.toml,.crates2.json,registry,git}` and
`target`), and printed counts only. The needles were one line each, of 30
bytes or more and in no public file, from the first file of the content's
`battles/`, `dialogue/`, `maps/` and `chapters/` (one line, not 64 bytes:
`grep -F` reads each line of a multi-line needle as its own pattern, and
the public files are embedded in every build).

| Key (`…-7b3c7a25`) | OS | files in `target` | workspace names | `out/onebit-content` | `target/playdate` | needles |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `v0-rust-clippy-Linux-x64-cdee7d24` | Linux | 1507 | 0 | 0 | 0 | absent ×4 |
| `v0-rust-test-Linux-x64-cdee7d24` | Linux | 1530 | 0 | 0 | 0 | absent ×4 |
| `v0-rust-test-Windows_NT-x64-6e22454c` | Windows | 1655 | 0 | 0 | 0 | absent ×4 |
| `v0-rust-test-Darwin-arm64-a38dd7d7` | macOS | 1448 | 0 | 0 | 0 | absent ×4 |
| `v0-rust-wasm-Linux-x64-cdee7d24` | Linux | 1877 | 0 | 0 | 0 | absent ×4 |
| `v0-rust-no-std-Linux-x64-cdee7d24` | Linux | 626 | 0 | 0 | 0 | absent ×4 |
| `v0-rust-playdate-Linux-x64-cdee7d24` | Linux | 2015 | 0 | 0 | **363** | absent ×4 |
| `v0-rust-docs-Linux-x64-0c88e82e` | Linux | 1368 | **24** | 0 | 0 | absent ×4 |
| `v0-rust-coverage-Linux-x64-cdee7d24` | Linux | 2638 | 0 | 0 | 0 | absent ×4 |
| `v0-rust-tickets-Linux-x64-cdee7d24` | Linux | 1014 | 0 | 0 | 0 | absent ×4 |
| `v0-rust-bench-bench-Linux-x64-cdee7d24` | Linux | 2982 | 0 | 0 | 0 | absent ×4 |
| `v0-rust-build-Linux-x64-a74c6f26` (playtest) | Linux | 986 | 0 | 0 | 0 | absent ×4 |

rust-cache's cleaning does delete the workspace crates' builds in the
profile folders, but it leaves folders of `target/` that are no profile:
`playdate`'s cache holds `target/playdate/` (the packed device game,
`pdex.bin` linked with `trpg-content` and the converted pictures and
sounds; pdc compresses it, which is why the needles miss) and `docs`'s
holds `target/doc/` (the workspace's rustdoc pages). Counts above 0, so
**case A**. Not on `main` to check: `mutants.yml`'s and `story-kit.yml`'s
caches (none listed on 2026-11-17; their scheduled runs' caches had been
evicted). They get the same guard.

**Step 2.** Every job of the table's `rust-cache` step has
`cache-targets: false` with the comment (17 steps: `ci.yml` 10,
`playtest.yml` 1, `mutants.yml` 4, `story-kit.yml` 2). The registry, git
and installed tools stay cached. The caches are deleted after the merge
(the commands are in the PR's description, "After merging").

**Step 4.** `crates/xtask/src/workflow_guard.rs` (test-only) holds the
check; `no_job_with_private_files_caches_or_uploads_a_build` in
`main.rs`'s tests runs it over every workflow, and three made-up workflows
test the check itself (passing; unguarded `rust-cache` and
`cache-all-crates: "true"`; an upload of `target/release/x` and
`dist/export/web` and an `actions/cache` of `target`). Taking
`cache-targets: false` off `playtest.yml`'s `build` failed it with
`playtest.yml: job `build`: step `Swatinem/rust-cache@…`: the job checks
out the private files: …`; put back, it passes.

**Deviation:** the check allows three uploads, not one. Besides
`playtest.yml` `build` `target/release/xtask` (ADR-0117 §4), `ci.yml`
`bench` uploads `dist/bench` (the numbers and table) and `release.yml`
`package` uploads `dist/release/kept/*` (the reports and SHA-256, which
its own step checks): both under `dist/`, neither a build. Each is in
`ALLOWED_UPLOADS` with its reason, and the test checks each is still
there.

**Step 5.** ADR-0117 §4 has the amendment (its title now "…artifact or
cache"; the ADR index row says so); §3's job list did not change. The
`content-change` skill's "never in public" says a new job that checks out
the content sets `cache-targets: false`, as the test checks.

**Cost** (minutes, each job on `main`'s last three successful CI pushes
before the change, and this PR's first run without the target cache,
which also restored no registry: an upper bound; the merged runs keep the
registry):

| Job | before (3 runs) | mean | after (PR 741) | change |
| --- | --- | ---: | ---: | ---: |
| clippy | 1.3 / 1.0 / 1.6 | 1.3 | 1.9 | +0.6 |
| test (Windows) | 14.6 / 14.0 / 10.3 | 13.0 | 18.5 | +5.5 |
| test (Linux) | 10.9 / 8.2 / 10.7 | 9.9 | 13.5 | +3.6 |
| test (macOS) | 9.2 / 10.6 / 8.9 | 9.6 | 15.6 | +6.0 |
| wasm | 2.6 / 2.2 / 2.8 | 2.5 | 4.0 | +1.5 |
| no-std | 1.3 / 1.0 / 1.6 | 1.3 | 2.0 | +0.7 |
| playdate | 4.7 / 4.1 / 6.4 | 5.1 | 4.8 | -0.3 |
| docs | 0.8 / 0.7 / 1.3 | 0.9 | 1.2 | +0.3 |
| coverage | 13.5 / 9.9 / 9.3 | 10.9 | 12.4 | +1.5 |
| tickets | 1.7 / 1.6 / 2.8 | 2.0 | 2.7 | +0.7 |
| bench | 7.9 / 6.8 / 8.1 | 7.6 | 11.2 | +3.6 |
| playtest `build` | 3.2 / 5.1 / 3.2 | 3.8 | 4.1 | +0.3 |

About +1 to +6 minutes a job (the slowest: macOS `test` +6.0, Windows `test` +5.5, `bench` +3.6); `playdate` and `coverage` barely move, as most of their time is not the dependencies' build. The "after" runs `doc-tests` did not run (no docs-only change). Measure the merged runs again if the minutes matter: they keep the registry cache.

Engine friction: estimate 200 lines (a text check of the workflows and its
tests, like `mentions.rs`), measured 192 (1.0×), under 2×.
PR size: estimate 500, measured 485.
