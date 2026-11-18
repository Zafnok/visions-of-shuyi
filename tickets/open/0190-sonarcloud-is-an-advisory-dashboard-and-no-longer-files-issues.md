---
id: "0190"
title: SonarCloud is an advisory dashboard and no longer files issues
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0190 — SonarCloud is an advisory dashboard and no longer files issues

## Context

[ADR-0071](../../docs/adr/0071-sonar-backlog-becomes-batched-github-issues.md)
(ticket 0126, PR #280) made SonarCloud's backlog into batched GitHub
issues labelled `sonar`, filed weekly by `tools/sonar-triage/` and each
fixed in its own PR with the `sonar-fix` skill. The
[friction sweep of 2026-11-17](../../docs/research/friction-2026-11-17.md)
(*Processes that slow us down*, "The Sonar lane produces no fixes")
found it makes no fixes. Re-checked on 2026-11-17:

- `gh issue list --label sonar --state all`: 15 issues, 7 open (#347,
  #351, #352, #355, #426, #427, #711), 8 closed. The closed ones were
  closed by the triage script itself (for example #601's closing
  comment), 5 of them (#350, #353, #354, #356, #357) because the code
  moved to the engine's repository (#358), not because anyone fixed
  them.
- `gh pr list --state all --search "head:sonar/"`: 0 PRs. The only PRs
  with "sonar" in the title are #278 and #280, which built the lane.
- Rule S8863 (redundant `'static`) was filed 8 times as code moved:
  #347, #348, #349, #350, #426, #427, #601 and #711. Clippy does not
  replace it: `clippy::redundant_static_lifetimes` covers only `const`
  and `static` items, and CI already runs clippy with `-D warnings`.
- Upkeep: `tools/sonar-triage/sonar-triage.ps1` (787 lines), its
  `README.md` (96) and `policy.jsonc` (27), the `sonar-fix` (88) and
  `sonar-triage` (86) skills: 1,084 lines, plus the weekly
  `.github/workflows/sonar-triage.yml` (cron Monday 06:00), an exception
  in `AGENTS.md` hard rule 1, one in `tickets/README.md`, and the
  `sonar/` exemption in `crates/xtask/src/pr_size.rs` (line 46).

So: keep SonarCloud as a dashboard anyone can look at (the scan in
`ci.yml`, `sonar-project.properties`), and retire the issue lane.

The per-push cost of the scan's coverage run is 0174's (it replaces the
`coverage` job with a smaller `sonar` job). Either order: this ticket
touches neither job.

This bucket of the sweep (process and tooling): 0188 (split xtask), 0189
(a rules-text budget), this ticket, 0191 (delete the frozen generators),
and the amended 0145, 0147, 0181, with 0151, 1174, 1257 and 1239 folding
into 0181.

Other open tickets that name the lane, either order:

- 0175 lists `sonar-triage.yml` among the dispatch-only workflows. If
  this lands first, 0175 drops it from that list.
- 0143 keeps `sonar(…)` PR titles' treatment. After this there are none;
  0143 may drop the case.

## Nick input

None.

## Scope

**In:**
- A new ADR with the `write-adr` skill (the next free number) that
  supersedes ADR-0071: SonarCloud stays as an advisory dashboard, no
  issues are filed, no PR is a "sonar" PR, and a finding worth fixing
  becomes an ordinary ticket. Mark ADR-0071 *Superseded* in its status
  line and in `docs/adr/README.md`.
- Delete `.github/workflows/sonar-triage.yml`, `tools/sonar-triage/`,
  `.claude/skills/sonar-triage/` and `.claude/skills/sonar-fix/`, and
  their `.agents/skills/` loaders (`pwsh -File
  scripts/sync-agent-skills.ps1` removes stale ones, or delete them by
  hand and run `-Check`).
- `AGENTS.md`: hard rule 1 loses the `sonar` exception (it keeps
  `engine-pin` and `story-import`), rule 10 loses "and `sonar`", the map
  loses the `tools/sonar-triage/` row, and the skills list loses
  `sonar-triage` and `sonar-fix`.
- `tickets/README.md`, *Working a ticket*: "The two exceptions" becomes
  the one (`engine-pin`).
- `crates/xtask/src/pr_size.rs`: drop the `sonar/` exemption and its
  tests (`main.rs` line 1294, `pr_size.rs` lines 347, 397-399, 421 and
  440), and the comment at `ci.yml` line 377.
- Close the 7 open `sonar` issues with a comment naming the new ADR, and
  leave the `sonar` label in place for the closed history.

**Out (do not do):**
- The SonarCloud scan in `ci.yml` and its coverage run (0174).
- `sonar-project.properties` and the `SONAR_TOKEN` secret, which the
  dashboard still needs.
- Fixing any Sonar finding.
- The engine's own triage workflow, which is the engine's to keep or
  retire (an engine step needs Nick's OK first).

## Implementation steps

1. Write the ADR and mark ADR-0071 superseded.
2. `git rm -r` the workflow, the tool folder and the two skills; run
   `pwsh -File scripts/sync-agent-skills.ps1`, then `-Check`.
3. Edit `AGENTS.md`, `tickets/README.md`, `pr_size.rs` and its tests, and
   the `ci.yml` comment.
4. `grep -rn -i "sonar-triage\|sonar-fix\|sonar/" --exclude-dir=target
   --exclude-dir=tickets .` finds nothing but the ADRs and the
   SonarCloud scan; fix any other hit (for example ADR-0117's list of
   workflows that load no content).
5. Close the open issues as in *Scope*, and list their numbers in
   the Completion notes.

Expected size: about 1,150 lines deleted, about 80 added (ADR and edits).

## Acceptance criteria

- [ ] `tools/sonar-triage/`, `sonar-triage.yml` and both skills are gone,
      and `sync-agent-skills.ps1 -Check` passes.
- [ ] The grep of step 4 finds only the ADRs and the scan.
- [ ] `cargo xtask pr-size --branch sonar/x` warns like any other
      branch.
- [ ] ADR-0071 says *Superseded* by the new ADR.
- [ ] `gh issue list --label sonar --state open` is empty.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `pr_size.rs`'s verdict tests updated (a `sonar/` branch is no
  longer exempt).
- The existing `cargo xtask ticket-lint` and workflow guard tests pass.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
