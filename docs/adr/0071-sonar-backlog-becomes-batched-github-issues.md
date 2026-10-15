# ADR-0071: SonarCloud's backlog becomes batched GitHub issues, filtered by a checked-in policy

- **Status:** Accepted
- **Date:** 2026-10-15
- **Related tickets:** 0126 (this), 0104 (SonarCloud in CI)

## Context

SonarCloud runs in `ci.yml`'s coverage job (0104,
[ADR-0008](0008-ci-quality-gates.md)). Its quality gate grades new code
only, so what is already on `main` is never looked at. On 2026-10-15 the
project had 51 open findings, all code smells, across five rules:
`rust:S8863` (30), `rust:S2208` (12), `rust:S3776` (7), `rust:S1612` and
`rust:S8855` (one each). No security hotspots.

SonarCloud cannot file GitHub issues. Nick's other repository,
[`Zafnok/code-equivalency`](https://github.com/Zafnok/code-equivalency),
built a tool for this (its ADR 0016, batching, and ADR 0022, the size cap)
and runs it weekly. Nick asked for the same here.

## Decision

Port it. `tools/sonar-triage/sonar-triage.ps1` reads SonarCloud's
`/api/issues/search` and `/api/hotspots/search` anonymously (the project is
public), drops what `tools/sonar-triage/policy.jsonc` accepts or defers,
groups the rest, and keeps one GitHub issue per batch in sync, labelled
`sonar`. `.github/workflows/sonar-triage.yml` runs it on Mondays at 06:00
UTC with `-Apply`, and from the Actions tab as a dry run unless `dry_run`
is unticked.

**Batching.** A rule found in 3 or more files is one rule-wide batch (one
decision, one PR). Everything else groups by file. A file left with one
finding rolls into a long-tail batch for its top-level folder (`crates`,
`web`, ...), so the queue doesn't fill with one-finding issues.

**Size cap.** No issue lists more than 25 findings. A bigger batch becomes
a parent issue that holds the rule-wide decision and a table of parts, and
each part is its own issue, linked as a GitHub sub-issue. Parts follow the
directory tree (crate, then folder, then file), small neighbours are packed
back together in path order, and a single file over the cap is cut by line,
so fixing one part leaves the others' membership alone.

**State.** Each issue body opens with
`<!-- sonar-triage:v1 key=... -->`. That marker is the only state: a run
re-reads open `sonar` issues and creates, edits or closes to match. A
settled `-Apply` reports `created 0 / edited 0 / linked 0 / closed 0`.

**Policy.** `policy.jsonc` is the one place a finding is argued with:
`verdict` is `fix` (default), `accept` or `defer`, and the last two require
a `reason`, citing an ADR or a ticket when it appeals to a repo-wide
choice. The script refuses to run on a suppression without one. No
`// NOSONAR`, no `#[allow]`.

**A `sonar` issue is its own ticket.** The issue body carries the goal, the
findings and the acceptance criteria. A fix is one PR on a `sonar/<slug>`
branch (which `ticket-lint` ignores), titled like the issue, closing it.
This is the one exception to "work one ticket at a time from
`tickets/open/`" (`CLAUDE.md` hard rule 1). The `sonar-fix` skill is the
checklist; `sonar-triage` covers running the script and deciding verdicts.

**Changes from the source repository:** the project key; rule links go to
the rule's SonarCloud page, which exists for every rule key, external ones
included; acceptance criteria name this
repo's gates (`run-gates`); the step that pinned a Roslyn analyzer's
severity in `.editorconfig` is dropped, because Clippy already runs with
`-D warnings` over `all` and `pedantic`, so a Clippy finding fails CI
before Sonar sees it; and a `@word` in a Sonar message is put in backticks
so the issue never mentions a GitHub user.

## Consequences

- GitHub Issues become a work surface, for `sonar` issues only. Everything
  else stays in `tickets/`.
- The weekly job needs `issues: write` and no secret.
- `-PushResolutions` (mark accepted findings Won't Fix in SonarCloud) needs
  a `SONAR_TOKEN` with issue-admin rights and asks before acting; CI never
  passes it.
- Sonar issue keys change on re-analysis, so batches are keyed by rule or
  path, and a fix session re-checks each finding against `HEAD`.
- The script has no tests, as in its source repository: it is HTTP calls
  and `gh` shell-outs, checked by a dry run. Changes to it are checked the
  same way.
- When the backlog is empty the job files nothing and closes what is left.

## Alternatives considered

- **Rewrite it as a `cargo xtask` command.** Matches this repo's tooling,
  but needs an HTTP crate and a JSON crate in `xtask` and a rewrite of a
  working, idempotent 800-line tool. The PowerShell runs on Nick's Windows
  PowerShell 5.1 and on the runner's `pwsh` unchanged. Revisit if it needs
  tests.
- **Write the batches as ticket files in `tickets/open/`.** Keeps one work
  surface, but a weekly bot would open PRs adding and deleting ticket files,
  and ticket numbers are never reused, so churn burns numbers.
- **`sonar.rust.*` exclusions or removing rules from the quality profile.**
  Hides findings instead of deciding them one by one.
- **A SonarCloud webhook as the trigger.** It sends a quality-gate result,
  not the issue list, and fires on every PR analysis.
- **Per-finding `// NOSONAR`.** Spreads the reason across call sites, where
  nobody reading the dashboard sees it.
