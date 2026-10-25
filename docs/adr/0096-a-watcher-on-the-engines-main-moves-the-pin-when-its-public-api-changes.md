# ADR-0096: A watcher on the engine's `main` files a pin move when it is due, and each move reviews what the engine gives

- **Status:** Accepted
- **Date:** 2026-10-25
- **Related tickets:** 1190 (this), 1177, 1188 and 1189 (the last pin
  tickets), engine#2030 (the `api:` label), engine#2031 (the dispatch on
  push); extends [ADR-0063](0063-an-engine-and-a-game-one-boundary-now-two-repositories-later.md) §8
  and [ADR-0071](0071-sonar-backlog-becomes-batched-github-issues.md)

## Context

The game builds on one Shuyi Engine commit, the pin (`shuyi-engine.rev`,
ADR-0063 §8). Until now it moved only when someone wrote a ticket for it
(1177, 1188, 1189), each naming the few engine tickets someone waited for.
On 2026-10-24 the pin was 25 engine commits behind the engine's `main`,
and nothing in this repository said so. Three engine PRs had moved public
types onto constructors (#88, #90, #91), so whoever moved the pin next
would inherit every break at once. Ready game tickets each had to move
the pin themselves, so two of them in parallel worktrees fought over
`Cargo.lock`. And nobody looked at what the engine shipped that no
ticket asked for: a pin move only checked that the game still built, so
the game kept its own copies of what the engine now gave.

Nick asked for a robust approach (2026-10-24). He turned down a weekly
move as too slow, since the engine was built in about a week, and chose a
token-started watcher over polling (2026-10-25).

## Decision

1. **A watcher the engine starts.** `.github/workflows/engine-watch.yml`
   runs on `workflow_dispatch` with an `engine_sha` input, which the
   engine sends on every push to its `main` (engine#2031) with a
   fine-grained token that can only run workflows here. It also runs
   hourly (`17 * * * *`) as a backstop, and from the Actions tab as a dry
   run (`dry_run`, default true; a run with `engine_sha` applies). Each
   run has two stages:
   - **Stage 1, no checkout and no Rust:** read the engine `main`'s head,
     the pin, and the marker of the newest `engine-pin` issue
     (`<!-- engine-pin:v1 main=… pin=… trigger=… dispatch_seen=…
     dispatched=… -->`). If `main` and the pin are what the open issue's
     marker recorded (or `main` is the pin, with no issue open), stop.
     A scheduled run that finds engine commits over two hours old that no
     dispatch reported, once a dispatch has ever arrived, adds a line to
     the issue: check the token.
   - **Stage 2:** build `trpg-engine-watch` and sync the one open issue
     labelled `engine-pin`: create or edit it when a move is due, close
     it when the pin is at `main` or no move is due any more, and
     otherwise change nothing.
2. **When a move is due.** At least one engine PR past the pin:
   - **changes the public API:** its `api: added` or `api: breaking`
     label (engine#2030); `api: none` doesn't count. Without a label, a
     `+` or `-` line declaring a `pub` item (not `pub(crate)`,
     `pub(super)`, `pub(in …)`) in a `src/` file of a crate in the
     engine's `API_CRATES` (`crates/xtask/src/api.rs` at `main`; engine
     ADR-0038's six when it can't be read), or a file whose patch GitHub
     left out. A `Breaking:` line in the PR or its engine ticket's
     Completion notes counts either way;
   - **or unblocks a game ticket:** it moves an engine ticket into the
     engine's `tickets/done/` that an open game ticket's `blocked_by` or
     an `// engine-friction: engine#NNNN` marker in `crates/` names. A PR
     that only files the ticket unblocks nothing;
   - **or has waited:** it changes code under `crates/` and merged more
     than 48 hours ago.

   A change to tickets, docs or CI alone never files anything.
3. **The report** sorts each engine PR into the first section that fits:
   *Breaking* (a `Breaking:` line or the `api: breaking` label), *Unblocks*,
   *New for games* (a public API change, as in point 2), *Nothing for the
   game* (collapsed). Its title is `Engine pin: N engine PRs behind
   (public API changed in #NN)`, `(unblocks NNNN)` or `(oldest merged
   YYYY-MM-DD)`. It also lists the adoption tickets still open from the
   ledger, with their age. `cargo xtask engine-pin --report` prints it
   locally.
4. **The issue is its own ticket**, the second exception to `CLAUDE.md`
   hard rule 1 after ADR-0071's `sonar` issues. The `engine-pin` skill
   works it in one PR on `engine-pin/<8 hex>` (Opus 5.5, effort medium;
   Nick, 2026-10-25): it pins the engine's `main` as it stands then, fixes
   the breaks on the game side with behaviour unchanged, records the
   adoption decisions, writes the ledger section and closes the issue.
   No more tickets written only to move the pin.
5. **The adoption review.** Each *New for games* PR gets exactly one
   decision: `adopt → NNNN` (a game ticket that moves the game onto the
   engine's piece and deletes its copy, or an open ticket changed to use
   it; `(after engine#MMMM)` when the engine must change first) or `not
   for the game: <why>`. Adopting is ordinary ticket work, never done in
   the pin PR.
6. **The ledger,** `docs/engine/pins.md`: a section per pin move, by the
   watcher's issue or by a ticket, with a row and a decision for every
   *Breaking*, *Unblocks* and *New for games* PR, and a count of the
   rest. `cargo xtask engine-pin --check` (in CI) fails when the last
   section isn't the pin, a row has no decision, or an `adopt → NNNN`
   names no ticket in `tickets/open/` or `tickets/done/`. So a ticket's
   own pin move (`engine-change` §3.6 and §7) writes its review too.
7. **`trpg-engine-watch`** (`crates/engine-watch`) is dev tooling, never
   shipped, like `trpg-bots`. It depends only on `std` and `serde_json`,
   so the watcher builds in seconds without the engine or macroquad. Its
   pure part (the parsing, the sort, the due rule, the rendering, the
   ledger) is unit-tested on fixtures. `fetch.rs` (the `gh` calls and file
   reads) and `main.rs` are left out of mutation testing, as `crates/app`
   is: no test reaches the network.

## Consequences

- The gap between the pin and the engine's `main` is always visible: an
  open `engine-pin` issue says how far behind the game is and why it
  matters.
- Breaks arrive a few PRs at a time, fixed by one session, instead of all
  at once in whichever ticket needs the next engine.
- Every engine feature a move passes gets a decision on the record, and
  the adoption tickets left open stay visible in the next issue.
- A ticket that moves the pin itself has more to do: its ledger section.
  CI makes sure it does.
- The `pub`-line fallback counts too much (a `pub` item in a private
  module), which only files an issue early. engine#2030's label replaces
  it for every PR merged after that ticket.
- Before engine#2031 lands, the hourly run is the whole watcher: up to an
  hour's delay. The missing-dispatch line stays off until the first
  dispatch arrives.
- With no issue open, stage 1 compares `main` with the pin. A gap that
  isn't due (tickets only) runs stage 2 each hour until something makes a
  move due, a few cached seconds of build each time.
- The report reads at most 250 engine commits (GitHub's compare). Past
  that it fails and asks for the pin to be moved in steps.

## Alternatives considered

- **Keep pin tickets.** Each written by hand when someone waits for
  something, while the gap and the unread features grow in between.
- **A weekly move.** Nick: too slow for an engine built in a week. Bug
  fixes and breaks would wait up to seven days.
- **Move the pin on every engine PR.** Ten engine PRs a day would be ten
  pin PRs and gate runs a day, each colliding with game PRs in flight. One
  open issue that gathers PRs until it is worked moves just as fast with
  far less churn.
- **Polling every 15 minutes without a token.** No secret, but up to 15
  minutes of delay and about 100 runs a day. Nick preferred a token; the
  hourly backstop keeps polling's robustness.
- **`repository_dispatch`.** It needs a token with Contents: write here,
  which can push code. `workflow_dispatch` needs only Actions: write.
- **The exact public API diff run here** (engine ADR-0038's `cargo xtask
  api`): a pinned nightly, `cargo-public-api` and `cargo-semver-checks`,
  and minutes per run. The engine computes it on every PR already, so
  engine#2030 publishes the verdict as a label.
- **A bot that opens the pin PR.** A PR opened with the workflow's
  `GITHUB_TOKEN` doesn't start the `pull_request` workflows, so CI
  wouldn't run on it without a personal token in this repository, and
  the adoption review needs judgment a workflow can't give.
- **Adopt in the pin PR.** It would mix a mechanical move with refactors
  of any size. Adoption as tickets keeps both reviewable, one ticket per
  PR.
- **Dependabot or Renovate.** Neither bumps a `rev` across three files
  kept in step (`shuyi-engine.rev`, `Cargo.toml`, `Cargo.lock`), and
  neither reads the engine's PRs.
