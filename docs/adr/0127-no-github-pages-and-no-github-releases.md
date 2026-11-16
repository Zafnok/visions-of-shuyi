# ADR-0127: No GitHub Pages site and no GitHub Releases

- **Status:** Accepted
- **Date:** 2026-11-16
- **Related tickets:** 0920, 0158, 0170
- **Amends:** ADR-0009's targets table (the "GitHub Releases" and "GitHub
  Pages" destinations, and the rule that a tag "creates a GitHub Release"),
  and ADR-0023's Context (the `debug-tools` build is now the playtest page's,
  not a Pages site's). Their other parts stand.

## Context

ADR-0009 put every package on a GitHub Release and the web build on GitHub
Pages after each push to `main`. Ticket 0158 already took everything
private off both: Pages served only the public files and the test game, and
a release held only the notes, each package's reports and `SHA256SUMS`.

The first official Chapter 1 build now goes out on a password-protected
itch.io page, and the NDA process with outside readers is starting. Nick
(2026-11-16) asked for every GitHub Release to be removed and for Pages to
stop publishing. The releases v0.1.0 to v0.3.0, their tags, the Pages site
and the `github-pages` environment were deleted by hand that day.

## Decision

- No workflow publishes to GitHub Pages: `pages.yml` is deleted, and the
  repository's Pages site and website field stay empty.
- `release.yml` makes no GitHub Release. A tag still builds and checks every
  package and attests `SHA256SUMS` (its `attest` job, with no `contents:
  write`); the reports and `SHA256SUMS` stay as the run's artifacts.
- Every build anyone plays goes to a restricted, password-protected itch.io
  page (the playtest page, the demo pages, the writer's preview), as
  `docs/releasing.md` lists. The store page (ticket 0901) takes the
  packages when it exists.
- The private repositories' releases (the debug files, ticket 0917; the
  writer's kit, ticket 0152) are not public and stay.

## Consequences

- There is no public link to play the game; the README names none.
- Nick's and testers' web play, and the debug tools with it (ADR-0023), are
  on the playtest page only.
- A version tag leaves no page on GitHub; its notes, if any are wanted, go
  with the itch.io upload.

## Alternatives considered

- **Keep a public Pages build of the public files only** (ticket 0170's
  state) — it showed the game's systems and test chapter to anyone during
  an NDA period, against Nick's request.
- **Make the repository private** — ticket 0153 priced private CI far above
  the budget.
