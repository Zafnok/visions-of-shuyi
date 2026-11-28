# ADR-0014: CI quality gates, skipping heavy jobs on docs-only PRs

- **Status:** Accepted
- **Date:** 2026-09-25
- **Related tickets:** 0109, 0106
- **Supersedes:** ADR-0008

## Context

ADR-0008 set up the CI gates and ran all of them on every pull request.
Many tickets (all `00xx` design decisions, most `07xx` story work) change only
Markdown, yet each such PR ran tests on three OSes, a WASM build, coverage,
mutation testing and CodeQL (`analyze (rust)` alone takes about 2.5 min). CodeQL
on a PR always analyses the whole codebase; main's baseline is only used to show
*new* alerts, never to skip work.

Two GitHub behaviours constrain the fix:

1. A workflow skipped by an `on: pull_request: paths` / `paths-ignore` filter
   **reports no status at all**. If that check is required (ticket 0106), the PR
   waits forever and cannot be merged. A *job* skipped by `if:` does report
   (as "skipped"), and a skipped check satisfies branch protection.
2. A **matrix** job skipped by `if:` reports one check with the unexpanded name,
   e.g. `test (${{ matrix.os }})`, never the per-leg names such as
   `test (windows-latest)` (seen on the `mutants (full, …)` jobs on
   Zafnok/tactical-rpg#11). A required per-leg check would therefore also wait
   forever.

Everything else in ADR-0008 still holds: free-tier only; CodeQL and secret
scanning are free only because the repo is public. If the repo is ever made
private, remove CodeQL.

## Decision

### Docs-only detection

- `.github/workflows/changes.yml` is a reusable workflow (`on: workflow_call`)
  with one job, `detect`, and one output, `code`.
- On `pull_request`, it checks out the PR merge commit with `fetch-depth: 2` and
  lists `git diff --name-only --no-renames HEAD^1 HEAD` (the first parent of the
  merge commit is the base branch tip). `--no-renames` makes a move such as
  `crates/x.rs → docs/x.md` list both paths, so it counts as code.
- **Docs-only paths:** `*.md` anywhere, `docs/**`, `tickets/**`, `.claude/**`.
  A Markdown file the code reads runs its crates' tests (ADR-0098).
  `.claude/**` counts as docs: it holds agent skills (Markdown) and local agent
  settings, and no CI job reads it. Anything else (including `.github/**`,
  `Cargo.*`, `deny.toml`, `_typos.toml`) is code.
- On **any other event** (`push` to `main`, `schedule`, `workflow_dispatch`)
  `code` is always `true`, so main's CodeQL baseline, the weekly scans and the
  weekly mutation run stay complete.

### Gating

Each heavy workflow calls it as a job named `changes` and heavy jobs get:

```yaml
needs: changes
if: needs.changes.outputs.code == 'true'
```

Never use workflow-level `paths` / `paths-ignore` on a workflow with a required
check.

A workflow with a matrix job also gets an aggregate job (`ci-result`,
`codeql-result`): `if: always()`, `needs:` every other job in the workflow
(including `changes`), fails if any result is `failure` or `cancelled`, passes
on `success`/`skipped`. **The aggregate is the required check**, not the matrix
legs. Any new job added to such a workflow must be added to its aggregate's
`needs`.

### Gates

"Required" means the check is part of branch protection on `main` (ticket 0106).
"Docs-only PR" says what happens on a PR that only touches docs-only paths.

| Gate | Tool | Trigger | Docs-only PR | Required check | Ticket |
| ---- | ---- | ------- | ------------ | -------------- | ------ |
| Formatting | `cargo fmt --check` | PR, push | skipped | via `ci-result` | 0102 |
| Lints | `cargo clippy --all-targets -- -D warnings` (+ selected `pedantic` lints) | PR, push | skipped | via `ci-result` | 0102 |
| Tests (3 OS) | `cargo nextest run --workspace --profile ci` on `windows-latest`, `ubuntu-latest`, `macos-latest`; Ubuntu under `cargo llvm-cov nextest`, then `cargo test --doc` | PR (a draft: Ubuntu only), push | skipped | via `ci-result` | 0102, 0174 |
| WASM build | `cargo shuyi web` (debug; `export` builds the release package) | PR, push | skipped | via `ci-result` | 0102, 0174 |
| Docs build | `cargo doc --no-deps` with `-D warnings` | PR, push | skipped | via `ci-result` | 0102 |
| Coverage + static analysis | `sonar`: SonarCloud reads `test`'s Ubuntu `lcov.info` and `clippy`'s `clippy.json` | PR (not a draft), push | skipped | via `ci-result` | 0104, 0174 |
| Licences (ADR-0013), advisories, bans, sources | `cargo-deny` | PR, push, weekly | skipped | `deny` | 0103 |
| Unused dependencies | `cargo-machete` | PR, push, weekly | skipped | `machete` | 0103 |
| Spelling | `typos` | PR, push, weekly | **runs** (useful for Markdown) | `typos` | 0103 |
| Workflow security | `zizmor` (medium+) | PR, push, weekly | runs when `.github/**` changed (`workflows`), else skipped | `zizmor` | 0103, 0174 |
| Code scanning | CodeQL (Rust + Actions) | PR (not a draft), push, weekly | runs when `.github/**` changed (`workflows`), else skipped | `codeql-result` | 0103, 0174 |
| Mutation testing | `cargo-mutants --in-diff` (PR), full run weekly | PR (not a draft), weekly | skipped | `mutants (diff)` | 0105, 0174 |
| Supply-chain posture | OpenSSF Scorecard | push to main, weekly | n/a | ❌ (report only) | 0103 |
| Dependency updates | Dependabot (cargo + github-actions), grouped weekly | schedule | n/a | n/a | 0103 |
| Secret scanning + push protection | GitHub built-in (repo setting) | always | n/a | n/a | 0103 |
| Ticket hygiene | `cargo xtask ticket-lint` | PR, push | should run (tickets are docs) | `tickets` | 0106 |
| Playtest (`playtest.yml`) | `cargo xtask playtest <battle> --bot all`, every battle | PR, push, "Run workflow" | skipped: plays only when a balance lever changed (`.github/playtest-levers.sh`) | ❌ (advisory: fails on a blocked battle, a band missed only warns) | 0514, 0530, 0173 |

*Note, 2026-11-17 (ticket 0173):* Playtest reuses its reports when its
inputs hash the same (`.github/playtest-inputs.sh`: the toolchain, the lock
file, the engine pin, the rules, content and bots, the playtest command,
the public, test and private content, the workflow). The seeds are fixed,
so the same inputs give the same reports: a PR's later commit takes them
from the cache, and the push to `main` that merges it from the PR's last
run. A reused blocked battle still fails. Playtest stays advisory: making
it required is a ruleset setting, Nick's call.

*Note, 2026-11-27 (ticket 0150):* the aggregates (`ci-result`,
`codeql-result`, `mutants (diff)`) judge their legs with
`scripts/ci-result.sh`. A `cancelled` leg fails one only while its commit
is still the head of the PR (or of the branch it was pushed to): a run a
newer push cancelled (`cancel-in-progress`) passes with a notice, since
the newer run checks the newer commit, and was red on 11 PRs for no
finding. A run cancelled by hand, or a leg past its time limit, on the
head commit stays red, so cancelling never lets a commit merge
unchecked. CI's and the mutants runs' proptests use one fixed seed
(`PROPTEST_RNG_SEED`, the same in `ci.yml` and `mutants.yml`);
`proptest-explore.yml` tries random seeds daily on `main` and files a
`proptest-seed` issue for a failing case.

*Note, 2026-11-28 (ticket 0174):* most CI minutes went to runs the change
couldn't fail (the retro of PRs #616–#735, T11). `changes.yml` now:

- on a **push**, diffs the push's own commits, `github.event.before` to
  `github.sha` (`before` fetched at depth 1), so a docs-only merge to
  `main` skips the heavy jobs as its pull request did. A push with
  `before` all zeros, or one whose `before` can't be fetched, counts as
  code; `schedule` and `workflow_dispatch` still always do, so the weekly
  scans keep main's baselines whole. A push to `main` in `ci.yml`,
  `codeql.yml` and `playtest.yml` is never cancelled by the next one (its
  concurrency group is the commit), since each diffs only its own commits;
- has a second output, **`workflows`**: `true` when any `.github/**` path
  changed. `zizmor` and CodeQL's `analyze` run on `code || workflows`;
- counts as **docs** `LICENSE`, and the workflows that no pull-request
  workflow calls and that are no pull request's check (`release.yml`,
  `story-kit.yml`, `story-mirror.yml`, `playtest-itch.yml`,
  `engine-watch.yml`, `sonar-triage.yml`, `scorecard.yml`,
  `.github/dependabot.yml`): their line adds `xtask` to `doc_crates`, whose
  tests parse every workflow. xtask's
  `light_workflows_are_called_by_no_pull_request_workflow` keeps the list
  true (it dropped the butler action, which `demo.yml` runs on PRs);
- counts a **pin move** as docs when `assets-private.rev` is the only
  non-docs path and its old and new commits have the same root tree
  (commit objects fetched with `--filter=tree:0` and the deploy key, which
  the four callers pass as the optional `PRIVATE_ASSETS_KEY` secret). No
  key (a fork) or any error counts as code.

A **draft** pull request (`types: [opened, synchronize, reopened,
ready_for_review]` on `ci.yml`, `codeql.yml`, `mutants.yml`,
`playtest.yml`) runs `test` on Ubuntu only and skips mutants' `plan`,
CodeQL's `analyze`, `sonar` and the playtest; its `--pr-branch` ticket
lint only warns. "Ready for review" starts the full run. The aggregates
already pass on `skipped`. Every `Swatinem/rust-cache` step saves only on
`main` (`save-if`; xtask's `every_rust_cache_step_saves_only_on_main`):
pull requests restore `main`'s caches and save none, which had passed the
10 GB cap and evicted `main`'s.

Do not require checks that are not produced by a job in these workflows (for
example the code-scanning `CodeQL` / `zizmor` result checks, or a SonarCloud app
check): they are not posted when their job is skipped.

Conventions for all workflows (unchanged from ADR-0008):

- Third-party actions are **pinned by commit SHA** with the version in a comment
  (zizmor and Scorecard both check this). Dependabot keeps them updated.
- Default `permissions: contents: read`; grant more per job only when needed.
- Checkouts use `persist-credentials: false`; `${{ }}` expressions go through
  `env:`, never directly into `run:`. `zizmor` reports nothing at medium+.
- Use `Swatinem/rust-cache` for build caching.
- `concurrency` groups cancel superseded runs on the same PR.

## Consequences

- Docs-only PRs finish in well under a minute (`changes`, `typos`, the
  aggregates) instead of several minutes, and are still mergeable with every
  required check green or skipped.
- Every merged code PR has still passed formatting, lints, tests on three OSes,
  a WASM build, security scans, coverage and mutation checks. That is the review.
- A docs-only PR is not scanned by CodeQL; main is, on every push and weekly.
- The docs-only path list lives in one file (`changes.yml`). If a CI job ever
  starts reading Markdown or `.claude/**` (e.g. docs tests, a Markdown linter),
  that job must not be gated, or the path list must change.
- Adding a job to `ci.yml` or `codeql.yml` means adding it to the aggregate's
  `needs`, otherwise it is not enforced.

## Alternatives considered

- **Workflow-level `paths-ignore`** — simplest, but skipped workflows report no
  status and required checks block the merge forever.
- **Skip steps instead of jobs** (`if:` on every step) — per-leg check names keep
  expanding, but each leg still boots a runner (Windows and macOS are slow to
  start), and a green check that did nothing is misleading.
- **`dorny/paths-filter` or `tj-actions/changed-files`** — well known, but a
  third-party action (and `tj-actions/changed-files` was compromised in 2025) for
  what is a five-line `git diff`.
- **Requiring each matrix leg by name** — breaks on docs-only PRs (behaviour 2
  above).
