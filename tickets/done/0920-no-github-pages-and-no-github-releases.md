---
id: "0920"
title: No GitHub Pages site and no GitHub Releases
type: infra
milestone: M8 Release
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: []
nick_input: none
completed: 2026-11-16
---

# 0920 — No GitHub Pages site and no GitHub Releases

## Context

The first official release of Chapter 1 is out on a password-gated
itch.io page, and the NDA process with outside readers is starting. Nick
(2026-11-16): remove every GitHub Release and stop publishing GitHub
Pages. Until now `pages.yml` put a public web build (public files, test
game, debug tools; ticket 0158, 0170) on
`https://zafnok.github.io/visions-of-shuyi/` after every push to `main`,
and `release.yml` made a public GitHub Release for each `vX.Y.Z` tag with
the notes, each package's reports and `SHA256SUMS` (ADR-0009, ticket
0107).

Done by hand on GitHub before this ticket, on Nick's word: the releases
v0.1.0, v0.2.0 and v0.3.0 and their tags are deleted, the Pages site is
unpublished, the `github-pages` environment is deleted and the Pages
workflow is disabled. This ticket makes the repository match, so nothing
publishes again.

## Nick input

None. The decision is Nick's request above; verification is by the gates
and the GitHub checks listed below.

## Scope

**In:**
- Delete `.github/workflows/pages.yml`.
- `release.yml` makes no GitHub Release: the `release` job keeps only the
  attestation of `SHA256SUMS` (no `contents: write`), and the reports and
  `SHA256SUMS` stay as the run's artifacts.
- The README, `docs/releasing.md`, the comments of `playtest-itch.yml`
  name no Pages site and no GitHub Release.
- A new ADR (0127) amends ADR-0009 and ADR-0023: no Pages, no GitHub
  Releases (accepted ADRs are not edited).
- Open tickets that send Nick to "the Pages build" (0508, 0726) point at
  the restricted playtest page instead (it has the debug tools).
- The repository's website field no longer names the Pages URL.

**Out (do not do):**
- The store's itch.io page and pushing packages to it (0901).
- The private assets repository's `debug-v<version>` releases (private,
  ticket 0917) and the writer's mirror `kit` release (private, ticket
  0152): they stay.
- Making the repository private (ticket 0153 decided against it).

## Implementation steps

1. `git rm .github/workflows/pages.yml`; lower the `every_workflow_file_parses`
   floor in `crates/xtask/src/main.rs` only if it drops below the count.
2. In `release.yml`, rename the `release` job to `attest`, drop the
   `softprops/action-gh-release` step and `contents: write`, and rewrite
   the header comment.
3. Rewrite the README's top link and "What you can play today" lead,
   `docs/releasing.md` (step 3, "Where each build goes", "What a release
   holds"), `playtest-itch.yml`'s comment, ADR-0127 and its index rows,
   0508/0726, and the comments that call the `debug-tools` web build "the
   Pages build".
4. `gh repo edit Zafnok/visions-of-shuyi --homepage ""`.

## Acceptance criteria

- [x] `.github/workflows/pages.yml` is gone and no workflow uses
      `actions/deploy-pages` or `softprops/action-gh-release`.
- [x] `grep -rn "github.io/visions-of-shuyi" README.md docs/releasing.md .github`
      finds nothing.
- [x] `gh release list -R Zafnok/visions-of-shuyi` is empty and
      `gh api repos/Zafnok/visions-of-shuyi/pages` answers 404.
- [x] All gates in the `run-gates` skill pass (ticket-lint, check-mentions, pr-size and xtask tests locally; no Rust code changed, so the workspace build and tests are left to CI).

## Tests required

- Unit: `every_workflow_file_parses` still passes over the remaining
  workflow files.
- Property / snapshot: none (no game code changes).

## Completion notes

- On GitHub, on Nick's word (2026-11-16): deleted the releases v0.1.0,
  v0.2.0 and v0.3.0 with their tags, unpublished the Pages site, deleted
  the `github-pages` environment, disabled the Pages workflow and cleared
  the repository's website field.
- Deleted `pages.yml`. `release.yml`'s `release` job is now `attest`: it
  downloads `SHA256SUMS` and attests it, with no `contents: write` and no
  `softprops/action-gh-release`; reports and sums stay as run artifacts.
- ADR-0127 records it and amends ADR-0009 and ADR-0023 (the template says
  accepted ADRs are superseded, not edited, so a new ADR replaced the
  "amendment line" step).
- README, `docs/releasing.md`, `docs/session-records.md`,
  `docs/targets.md`, the `debug-tools`/`private-assets` comments in three
  `Cargo.toml`s and `shuyi.ron`, and tickets 0508 and 0726 now name the
  playtest page instead of the Pages build. History (done tickets, older
  ADRs, research, the ROADMAP's done M0 row) is left as written.
- The writer's mirror `kit` release and the private assets repository's
  `debug-v*` releases are private and untouched.
- No gameplay rule decided.
