---
id: "0126"
title: "SonarCloud's backlog becomes batched GitHub issues (sonar-triage)"
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0104"]
nick_input: none
completed: 2026-10-15
---

# 0126 — SonarCloud's backlog becomes batched GitHub issues (sonar-triage)

## Context

0104 put SonarCloud in CI ([ADR-0008](../../docs/adr/0008-ci-quality-gates.md)).
Its quality gate grades only new code, so the findings already on `main`
are never looked at: on 2026-10-15 there were 51 open, all code smells,
across five `rust:*` rules (30 of them `rust:S8863`). SonarCloud cannot file
GitHub issues itself.

Nick's other repository, `Zafnok/code-equivalency`, already solves this with
`sonar-triage.yml` and `tools/sonar-triage/` (its ADRs 0016 and 0022): a
weekly job reads the SonarCloud API, drops findings a checked-in policy
accepts, groups the rest into batches and keeps one GitHub issue per batch
in sync. Nick asked to bring it over. This ticket ports it, adapted from C#
to this repo's Rust and ticket rules. Decision record:
[ADR-0071](../../docs/adr/0071-sonar-backlog-becomes-batched-github-issues.md).

## Nick input

None.

## Scope

**In:**
- `tools/sonar-triage/sonar-triage.ps1`, `policy.jsonc`, `README.md`,
  ported from `code-equivalency` and adapted (project key, Rust rule links,
  `crates/<name>` areas, this repo's gates in the acceptance criteria; the
  `.editorconfig` pinning step dropped).
- `.github/workflows/sonar-triage.yml`: weekly on Mondays, applies; manual
  runs default to a dry run.
- Skills `sonar-triage` (refresh the queue) and `sonar-fix` (fix one batch).
- ADR-0071; `CLAUDE.md` and `tickets/README.md` record that a `sonar`
  GitHub issue is its own ticket.

**Out (do not do):**
- Changing how the scan runs (`ci.yml`'s coverage job) or
  `sonar-project.properties`.
- Fixing any Sonar finding, or filing issues by hand.
- Making the Sonar check required.

## Implementation steps

1. Copy the three `tools/sonar-triage/` files from `code-equivalency` and
   adapt them as listed under **In**.
2. Add `.github/workflows/sonar-triage.yml` with this repo's pinned
   `actions/checkout`, `persist-credentials: false`, and permissions scoped
   to the job (`contents: read`, `issues: write`).
3. Add the two skills under `.claude/skills/`.
4. Write ADR-0071, add it to `docs/adr/README.md`, and update `CLAUDE.md`
   (hard rule 1, repo map, skills list) and `tickets/README.md`.
5. Dry-run the script against the live project.

## Acceptance criteria

- [x] `pwsh ./tools/sonar-triage/sonar-triage.ps1` (dry run) reads the live
      project and prints its batches without writing anything.
- [x] The same dry run passes under Windows PowerShell 5.1 (`powershell`).
- [x] A policy entry with `verdict: accept` and no `reason` makes the
      script stop with an error.
- [x] No issue body puts a bare `@word` on GitHub.
- [x] `sonar-triage.yml` passes zizmor (the `security` workflow).
- [x] All gates in the `run-gates` skill pass. (No Rust changed: fmt, the
      xtask checks, typos, ticket-lint and zizmor ran locally; CI runs the
      rest, since `.github/` and `tools/` count as code.)

## Tests required

- Unit / property: none; the script has no Rust and no test harness, as in
  its source repository (ADR-0071).
- Integration: the dry run above against the live SonarCloud project.

## Completion notes

Ported `sonar-triage.yml`, `tools/sonar-triage/` and the two skills from
`Zafnok/code-equivalency`. Its `sonar.yml` (the scanner) was not ported:
`ci.yml`'s coverage job already runs the SonarCloud scan here.

Changes from the source, all in ADR-0071: project key; rule links go to the
rule's SonarCloud page (the `rules.sonarsource.com` link was C#-only); the
`.editorconfig` severity-pinning step is dropped (Clippy already runs with
`-D warnings` over `all` and `pedantic`); acceptance criteria name
`run-gates` and forbid `#[allow]`/`#[expect]`; a `@word` in a Sonar message
is put in backticks; the workflow scopes its permissions to the job and
checks out with `persist-credentials: false`, as this repo's other
workflows do. I tried long-tail areas per crate (`crates/ui`) and went back
to the source's top-level folder: per crate, today's two one-off findings
became two one-finding issues.

Dry run on 2026-10-15 (pwsh 7 and Windows PowerShell 5.1 agree): 51
findings, 6 issues. `rust:S8863` (30, over the cap) becomes a parent with
two sub-issues (`crates/content/src/` 7, `crates/ui/src/` 23);
`rust:S2208` (12); `rust:S3776` (7); and one long tail for `crates` (2).
A policy entry with `verdict: accept` and a blank reason stops the script
with an error.

The first real filing happens on the first Monday run after merge, or
sooner from the Actions tab (**Sonar triage** → Run workflow, untick
`dry_run`). No follow-up tickets. No gameplay rules decided.
