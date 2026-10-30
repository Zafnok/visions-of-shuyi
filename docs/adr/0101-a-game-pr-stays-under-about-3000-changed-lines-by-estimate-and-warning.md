# ADR-0101: A game PR stays under about 3,000 changed lines, by estimate and warning

- **Status:** Accepted
- **Date:** 2026-10-30
- **Related tickets:** 0135 (this rule), 1007 (the PR that prompted it),
  1072 → 1083 → 1084 and 1020 → 1085 (the first splits)
- **Extends:** ADR-0010 (one ticket, one PR), ADR-0092 (estimate before
  coding)

## Context

PR #483 (ticket 1007, the world map) changed about 7,250 lines, and the
session working it ran out of context even with a 1M window. Nick,
2026-10-29: "limit our game PRs to approx 3000 LOC or less". The size that
matters is what a session has to read and write, so that it can finish well.

`write-ticket` said "roughly ≤ 600 changed lines", the engine's number.
Nothing followed it: 30 of the last 80 merged PRs passed 600, and they went
fine. Nick: 600 is "too aggressive", "3000 LOC seems about the limit without
worrying about ctx window". With generated files left out, only 0506 (about
5,300) and 1007 passed 3,000 in those 80.

On enforcing it, Nick: "idk if we need outright fail, the engine mostly
handles by estimation and periodic checks. warning is good tho", with a nudge
while working ("hey we're a bit over can we split cleanly?").

## Decision

### 1. The numbers

A PR passes **about 3,000 changed lines** only with a reason. A ticket is
**planned at about 2,000 or fewer**, because work runs over its plan and the
ticket file itself accounts for 250–400 of them.

### 2. What counts

`cargo xtask pr-size [--base <ref>] [--branch <name>] [--hook]`: lines added
plus removed, as GitHub counts them, from the base's merge base with `HEAD`
to the working tree, untracked files included. Code, tests, docs, tickets and
hand-written data (`.dlg` scripts, `.ron`, maps) all count: a session reads
and writes them all.

Left out: binary files, and files marked `linguist-generated` in
`.gitattributes` (snapshots, `Cargo.lock`, the font atlas, the generated test
tilesets, vendored font sources, the engine's skill pack). That one list also
folds them in GitHub's diff. A new generated file gets a line there.

### 3. Three moments, none of them a failure

- **Before coding** (the main defense): `write-ticket` keeps a ticket's plan
  to about 2,000 lines and `work-ticket` estimates the PR's size beside
  ADR-0092's line estimate. A plan past it is split with `write-ticket`
  first: the first piece keeps the number, so existing chains hold.
- **While working**: a Claude Code hook (`.claude/settings.json`) runs
  `pr-size --hook` before `git commit`, `git push` and `gh pr create`. Past
  2,000 it tells the session to note where the work would split; past 3,000
  it asks whether it can split cleanly. It never blocks the command.
- **On the PR**: CI's `tickets` job runs it and shows a warning annotation
  past 3,000. The job stays green and the check is not required.

### 4. Past the limit

If it splits cleanly, split it: move what can wait to a new ticket, and stack
the PRs when the second needs the first. If it doesn't, ship it and say why in
the PR body's `PR size:` line.

### 5. Exempt: `engine-pin/` and `sonar/` branches

Their size comes from the engine's changes or from the batch of findings, not
from a plan. They get the number and no warning.

## Consequences

- Big tickets get split before a session starts them, which is when a split
  costs nothing.
- A PR can still go over: the rule depends on sessions acting on a warning.
  If PRs keep passing 3,000 without a reason, a later ticket can reconsider.
- The hook runs `cargo xtask`, which rebuilds when a crate xtask depends on
  has changed. After the gates this is only a link; a cold build can use up
  the hook's 180 seconds, and then it just doesn't warn that time.
- Hooks load when a session starts, so an edit to them shows up in the next
  session, not the current one. Tested in a fresh headless session, alone
  and chained (`git status && git commit`): the session gets the warning.

## Alternatives considered

- **A failing CI check at 3,000** — Nick didn't want one, and a red check
  after the work is done forces the most expensive kind of split.
- **Keeping 600** — Nick called it too aggressive, and sessions weren't
  following it anyway.
- **Extending `code-lines`** — it measures production Rust for a different
  question (ADR-0092). A session's context also goes to tests, docs and data.
- **Counting added lines only** — close enough either way; added plus removed
  is the number GitHub shows Nick on the PR.
- **Listing exclusions in the code** — `.gitattributes` is the standard place,
  and GitHub reads it too.
- **The hook as a small shell script, to avoid the cargo build** — a second
  implementation that would drift, with no tests.
