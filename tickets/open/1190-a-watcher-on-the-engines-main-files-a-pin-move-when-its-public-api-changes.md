---
id: "1190"
title: "A watcher on the engine's main files a pin move when its public API changes, with an adoption review of each engine PR and a ledger CI checks"
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1190 — A watcher on the engine's main files a pin move when its public API changes

## Context

The game builds on one Shuyi Engine commit, the pin (`shuyi-engine.rev`,
ADR-0063 §8, `docs/engine/README.md`). Today it moves only when someone
writes a ticket for it: 1177 and 1188 (both 2026-10-22) and 1189 (open),
each hand-written, each naming the few engine tickets someone was
waiting for. Nick (2026-10-24) asked whether that is the right approach
and asked for a robust one. The pin should move at a controlled rate
that everyone can see, and each move should take up what the engine now
gives, so the game writes less code of its own ("if the engine
implements AI behavior, we use its AI primitives"). A weekly schedule
was the first answer; Nick (2026-10-25) turned it down as too slow: the
engine was built in about a week and the game in two or three. Instead,
watch the engine's `main` and act when its public API changes.

What the ad hoc moves cost, measured on 2026-10-24:

- **Nothing shows the gap.** The pin (`3f2256d4`, 1188) is 25 engine
  commits behind the engine's `main` (`2659928c`), merged over
  2026-10-23 to 10-25. Nothing in this repository says so.
- **Breaking changes pile up.** Engine 2013, 2024 and 2025 (PRs #88,
  #90, #91) moved public types onto constructors and
  `#[non_exhaustive]` (`Window`, `AppConfig`, `BoxPlace`, `MapScene`,
  the game tools' types). The game builds some of them by literal, so
  the next move, whoever makes it, inherits every break at once.
- **Ready tickets each move the pin themselves.** Engine 3056 (one-bit
  sprites drawn fast) unblocks 1078. 2131 (threat areas), 2132 (the
  candidate order) and 2533 (playtest history) are three of the engine
  tickets 1180 waits on. 2545 and 1134 are 1189's. By `engine-change`
  §7 the first step of each is its own pin move, so two of them in
  parallel worktrees fight over `Cargo.lock`.
- **Nobody looks at what nobody asked for.** The engine ships things the
  game could use that no game ticket names: 2511 (engine defaults for
  every `GameData` part, a prelude, `run_game`), 2358, 2372 and 2003
  (modules, and a module's debug menu entries), 2012 (the pinned Rust
  kept in step). A pin move today only checks that the game still
  builds. Nobody decides "use it" or "not for us", so the game keeps
  its own copies. The friction log (`docs/engine/friction.md`) catches
  the cases where game code came out big, not the cases where the
  engine quietly caught up.

### The decision (this ticket's ADR records it)

1. **A watcher the engine starts.** `.github/workflows/engine-watch.yml`
   runs on `workflow_dispatch` with an `engine_sha` input. The engine
   starts it on every push to its `main` (engine#2031), using a
   fine-grained token that can only run workflows here (Actions:
   write), so the watcher runs within a minute of each engine merge.
   Nick (2026-10-25) chose a token over polling. The same workflow
   also runs hourly as a backstop (cron `17 * * * *`, off the hour,
   when GitHub's schedules run late), for a dispatch that never came:
   an expired token, a GitHub outage, or before engine#2031 lands. It
   also runs from the Actions tab as a dry run (`dry_run`, default
   true there; a run the engine starts passes `engine_sha` and no
   `dry_run`, so it applies). Each run does two stages:
   - **A cheap check, seconds and no Rust.** Read the engine `main`'s
     head (`gh api repos/Zafnok/shuyi-engine/commits/main --jq .sha`)
     and the `main=<sha>` that the open `engine-pin` issue's marker
     recorded (or the pin, when no issue is open). If they are equal,
     stop. A backstop run that finds engine commits more than two hours
     old that no dispatch reported adds a line to the issue: "The
     engine's dispatch didn't reach the game: check the token (engine
     2031's *Nick input*)". It knows by the `trigger=` the marker
     records (`dispatch` or `schedule`).
     **Either order with engine#2031:** if engine#2031 is done, dispatches
     arrive and the hourly run only backs them up. If not, nothing to
     do: the hourly run is the watcher until dispatches arrive, and the
     missing-dispatch line stays off until the first dispatch has been
     seen (`dispatch_seen` in the marker).
   - **The report**, only when the engine has new commits.
     `cargo run -p engine-watch --locked` lists every engine PR between
     the pin and `main`, sorts them (point 3), and says whether a pin
     move is due (point 2). If one is due, it creates or edits the one
     open issue labelled `engine-pin`. If not, nothing changes on
     GitHub. When the pin is at `main`, it closes the issue.

   An engine PR shows up as an issue within a minute or two, and one
   open issue at a time keeps it controlled: the issue gathers the
   engine's PRs until someone works it, and that session pins the
   engine's `main` as it stands then.
2. **When a move is due.** At least one engine PR past the pin:
   - **changes the public API.** The engine's own verdict comes first:
     an `api: added` or `api: breaking` label on the PR counts, and
     `api: none` doesn't (engine#2030 has the engine's `api` job set
     these from engine ADR-0038's listing diff and semver check). A PR
     with no `api:` label falls back to a test of its patch. It counts
     when an added or removed line declares a public item (`pub fn`,
     `pub struct`, `pub enum`, `pub trait`, `pub type`, `pub const`,
     `pub static`, `pub mod`, `pub use`, a `pub` field or variant; not
     `pub(crate)`, `pub(super)` or `pub(in …)`) in a `src/` file of an
     engine crate the engine counts as public. That list is
     `API_CRATES` in the engine's `crates/xtask/src/api.rs`, read at
     `main`. Either way, it also counts when the PR or its engine ticket
     has a `Breaking:` line. The fallback counts too much (a `pub` item
     in a private module), which costs only an early issue.
     **Either order with engine#2030:** if engine#2030 is done, PRs
     merged after it carry the label and the fallback serves only older
     or unlabelled PRs. If not, nothing to do: the fallback is the
     whole test until labels appear, and the code already prefers them;
   - **or unblocks a game ticket**: its engine ticket is in an open
     game ticket's `blocked_by` or in an `// engine-friction:
     engine#NNNN` marker in `crates/`;
   - **or has waited long enough**: it changes code under `crates/`
     and merged more than 48 hours ago, so internal fixes and speed-ups
     reach the game even when nothing public moves.

   Otherwise the new PRs wait for the next move that is due: a change
   to tickets, docs or CI alone never files anything.
3. **The report sorts every engine PR the move passes** into:
   - **Breaking:** a `Breaking:` line in the PR body or in its engine
     ticket's Completion notes (the engine requires one for every API
     break, engine ADR-0038). Fixed in the pin PR, on the game side,
     with behaviour unchanged.
   - **Unblocks:** as in point 2. Those game tickets become ready; the
     pin PR does nothing else for them.
   - **New for games:** it changes the public API (point 2) and is
     neither of the above. Each one gets a decision in the review.
   - **Nothing for the game:** the rest (internal code, tickets, docs,
     CI, templates, examples, skills). Listed, collapsed.
4. **The issue is its own ticket**, like a `sonar` issue (ADR-0071):
   the second exception to `CLAUDE.md` hard rule 1. A new `engine-pin`
   skill works it: one PR on a branch `engine-pin/<new pin's first 8
   hex>`. The PR moves the pin to the engine's `main`, fixes the breaks
   on the game side, writes the ledger section and the adoption
   tickets, and closes the issue. No more hand-written "move the pin"
   tickets.
5. **The adoption review** ("use what the engine gives"). For each
   *New for games* row, the session asks two questions:
   - Does the game already have code doing this job? Search `crates/`,
     the friction log's rows for that area, and
     `docs/engine/boundary.md`.
   - Does an open ticket plan to write code for it? `grep -rn` in
     `tickets/open`.

   Then it records exactly one decision:
   - **adopt → NNNN**: a game ticket (`write-ticket`) that moves the
     game onto the engine's piece and deletes the game's copy, stating
     the lines it expects to remove; or an open ticket's text changed
     to use the engine's piece instead of writing its own;
   - **not for the game:** one line saying why ("templates for new
     games", "the game has no modules").

   When adopting needs an engine change too (the engine's piece almost
   fits, or a break can't be fixed on the game side), the session files
   an engine ticket the same way: in the engine's repository, by its
   `write-ticket` and the game's `engine-change` skill ("ticket and
   follow-up"). The adoption ticket is then `blocked_by:
   ["engine#NNNN"]`, and the ledger row says `adopt → NNNN (after
   engine#MMMM)`.

   The pin PR itself stays mechanical: the pin, the break fixes, the
   ledger, the tickets. Adoption is ordinary ticket work, one ticket
   per PR, so a pin move never turns into a refactor. With moves this
   frequent, each review covers a few PRs, not a week's worth.
6. **The ledger**, `docs/engine/pins.md`. It has one section per pin
   move, whether the watcher filed it or a ticket made it, with a row
   and a decision for every *Breaking*, *Unblocks* and *New for games*
   PR, and a count of the rest. `cargo xtask engine-pin --check`
   (already in CI) fails when:
   - the ledger's last section isn't the current pin;
   - a row has no decision;
   - an `adopt → NNNN` names a ticket that doesn't exist.

   So every pin move writes its review, including a ticket's own move
   (`engine-change` §3.6 and §7), and nothing the engine ships passes
   unread. The issue also lists the adoption tickets still open, with
   their age, so unfinished cleanup stays visible.

A ticket that needs an engine commit past the pin still moves the pin
itself (`engine-change` §3.6, §7; the engine never makes the game
wait). It follows the `engine-pin` skill's review for the PRs it
passes; the watcher's next report then starts from its pin.

## Nick input

None here. Nick asked for the change (2026-10-24), for the watcher
instead of a weekly move, and for a token instead of polling
(2026-10-25); how it works is Claude's call. The token is engine#2031's
one-time setup (its *Nick input*), not this ticket's. Nick sees an
`engine-pin` issue on GitHub soon after the engine's API changes, and a
PR that closes it.

## Scope

**In:**
- ADR-0096 (or the next free number): the decision above, with the
  alternatives below.
- A new crate `crates/engine-watch` (`trpg-engine-watch`, a library and
  a binary; dev tooling, never shipped, like `bots`). It depends only on
  `std` and `serde_json`, which is already in `Cargo.lock`, so the
  watcher builds in seconds without the engine or macroquad. It holds
  the report, the decision whether a move is due, and the ledger
  parser.
- `cargo xtask engine-pin --report` (runs the crate locally) and the
  ledger check in `engine-pin --check` (calls the crate's parser).
- `.github/workflows/engine-watch.yml`: the schedule, the dry-run
  dispatch, the two stages, the issue sync.
- `docs/engine/pins.md` with its first section.
- `.claude/skills/engine-pin/SKILL.md`.
- Text: `CLAUDE.md` (hard rule 1, the skills list, the crates row of
  the repo map), `tickets/README.md` (*The one exception* becomes two),
  `docs/engine/README.md`, `engine-change` §3.6 and §7, `work-ticket`
  §1, `write-ticket` (no "move the pin" tickets), `docs/ROADMAP.md`
  lane F, `docs/adr/README.md`, and ADR-0004's crate list if it names
  every crate.
- The `engine-pin` label on the repository (`gh label create`).

**Out (do not do):**
- Moving the pin. The first issue the watcher files does that, within
  the hour after this merges if the engine's API has moved. 1189 can
  land in either order. If it is still open when this lands, it is
  worked as a pin move under the new rules: it writes the ledger's next
  section, and the watcher's next report starts from its pin. If it
  merges first, the ledger's first section is its pin.
- Adopting any engine feature (1180, 1078 and the rest are their own
  tickets).
- Engine changes. The exact public API label is engine#2030 and the
  dispatch on push to `main` is engine#2031. Both land in either order
  (points 1 and 2), and their PRs are the engine's.
- Working the issue automatically (a Claude session started by the
  issue). The issue is worked like a `sonar` issue, by a session someone
  starts; automating that is its own decision.

## Implementation steps

1. **The crate, pure part** (`crates/engine-watch/src/lib.rs` and
   modules, unit-tested with fixtures, no network):
   - `EnginePr { number, title, merged_at, merge_sha, ticket: Option<String>, body, labels: Vec<String>, files: Vec<PrFile>, ticket_notes: Option<String> }`,
     `PrFile { path, patch: Option<String> }` (the `patch` of `gh api
     repos/…/pulls/N/files`). The engine ticket comes from the title:
     `[NNNN] …`, or `Ticket NNNN: …` and `Tickets NNNN, …` for
     ticket-only PRs.
   - `api_crates(api_rs: &str) -> Vec<String>`: the names in `pub const
     API_CRATES`, mapped to their folders (`shuyi-core` →
     `crates/core/`). If the const can't be read, fall back to the six
     of engine ADR-0038 and say so in the report.
   - `changes_public_api(pr, api_dirs) -> bool`: the PR's `api:` label
     when it has one (`added` and `breaking` are true, `none` is false;
     engine#2030). Otherwise, a `+` or `-` line in a patch of a
     `crates/<api dir>/src/` file that declares a public item
     (point 2). A file whose `patch` is missing (GitHub drops very large
     patches) counts as a change. An `api: breaking` label also puts the
     PR in *Breaking*, even without a `Breaking:` line.
   - `breaking_lines(pr) -> Vec<String>`: lines starting `Breaking:` in
     the body or the ticket's notes. `No break` and `No breaking`
     lines are not breaks.
   - `classify(prs, api_dirs, open_tickets, markers, now) -> Report`:
     each PR in the first section that fits (*Breaking*, *Unblocks*,
     *New for games*, *Nothing for the game*), plus `due: Option<Why>`
     by point 2. `open_tickets` is `(id, blocked_by)` from
     `tickets/open/` (frontmatter only; a small parser here, or move
     `tickets.rs`'s into this crate and use it from `xtask`).
     `markers` are the `engine#NNNN` ids in `// engine-friction:`
     comments under `crates/`.
   - `render(report, ledger) -> (title, body)`. The body opens with
     `<!-- engine-pin:v1 main=<sha> -->`, then gives:
     - the pin, `main`, the number of engine PRs between them, and why
       a move is due;
     - the four sections as tables (PR link, engine ticket, title; the
       lines for *Breaking*; the game tickets for *Unblocks*), with
       *Nothing for the game* inside `<details>`;
     - *Adoption tickets still open*: each `adopt → NNNN` in the ledger
       whose ticket is in `tickets/open/`, with the ledger date;
     - a checklist of the work, pointing at the `engine-pin` skill.

     Put any `@word` from an engine title in backticks (`CLAUDE.md`).
     The title is `Engine pin: N engine PRs behind (public API changed
     in #NN)`, or `(unblocks NNNN)`, or `(oldest merged YYYY-MM-DD)`.
   - `ledger::parse(text) -> Result<Vec<Section>, String>` and
     `ledger::problems(sections, pin, ticket_ids) -> Vec<String>`
     (step 3).
2. **The crate, network part** (`main.rs`, thin, `gh` shell-outs as
   `xtask`'s `tickets.rs` does):
   - Get the commits with `gh api
     repos/Zafnok/shuyi-engine/compare/<pin>...main`. Past 250 commits,
     fail with a clear message: "move the pin in steps".
   - Take the PR number from each commit title's `(#NN)`.
   - Get each PR with `gh pr view NN --repo Zafnok/shuyi-engine --json
     title,body,mergedAt,labels` and `gh api
     repos/Zafnok/shuyi-engine/pulls/NN/files --paginate`.
   - Read `crates/xtask/src/api.rs` at `main`, and, when the title names
     a ticket, the ticket's notes from `tickets/done/<NNNN>-*.md` at the
     merge commit.

   Options: `--json` (title, body, `due`, `main`, behind count) for the
   workflow, and Markdown by default. Exit 0 with nothing behind when
   the pin is at `main`.
3. **The ledger and its check.** `docs/engine/pins.md` opens with a
   short header (what it is, ADR-0096). Sections follow, newest last,
   each headed `## YYYY-MM-DD: <from 8 hex> → <to 8 hex> (N engine
   PRs)`, plus the game PR once it exists. Each holds a table `| Engine
   PR | Ticket | What | Decision |` and a line `Nothing for the game: N
   PRs (#…, #…)`. The first section is the baseline: `## <today>:
   baseline → <pin 8 hex>`, with no rows and one line saying the pins
   before it were moved by tickets (1177, 1188, and 1189 if merged)
   without this review. `xtask`'s `engine::check` calls
   `ledger::problems`, which reports:
   - the last heading's `to` isn't a prefix of `shuyi-engine.rev`;
   - a row's Decision cell is empty;
   - an `adopt → NNNN` names no file in `tickets/open/` or
     `tickets/done/`.

   Keep `the_real_repo_agrees_with_itself` passing on the real files.
4. **The workflow** `.github/workflows/engine-watch.yml`, modelled on
   `sonar-triage.yml`:
   - Triggers: `workflow_dispatch` with inputs `engine_sha` (string,
     optional) and `dry_run` (boolean, default true), and `schedule:
     cron "17 * * * *"`. A run with `engine_sha` set is the engine's
     dispatch (engine#2031) and applies, whatever `dry_run` says. A
     run started by hand without it obeys `dry_run`. A scheduled run
     applies. Write the trigger and `dispatch_seen` into the issue
     marker (point 1).
   - `concurrency: engine-watch` with `cancel-in-progress: false`.
   - `permissions: {}` at the top and `contents: read`, `issues: write`
     on the job; actions pinned by SHA as in the other workflows.
   - Stage 1 is a `gh` step with no checkout. If the engine `main`'s sha
     equals the open issue's `main=` marker (or, with no open issue,
     `shuyi-engine.rev` read by `gh api`), it stops the job.
   - Stage 2: checkout, Rust as `ci.yml` sets it up with a cache, then
     `cargo run -p trpg-engine-watch --locked -- --json`.
     - Nothing behind: close the open `engine-pin` issue with a comment.
     - `due` set: edit the open issue's title and body (`gh issue list
       --label engine-pin --state open`), or create one.
     - `due` empty: do nothing.

   Read inputs through `env:` (never `${{ }}` inside `run:`). A dry run
   prints the title and body and changes nothing.
5. **The skill** `.claude/skills/engine-pin/SKILL.md`, in the style of
   `sonar-fix`:
   - The issue is the ticket. Re-run `cargo xtask engine-pin --report`:
     the engine's `main` may have moved, so pin the newest.
   - Branch `engine-pin/<8 hex>`; run `cargo xtask engine --off`, then
     `cargo xtask engine-pin <main>`.
   - Build and run the gates (`run-gates`). Fix *Breaking* rows on the
     game side with behaviour unchanged, and explain any snapshot change
     by its engine PR. Then run `cargo shuyi mcp --ready`.
   - Do the adoption review of each *New for games* row: the two
     questions and the two decisions of the Context, with an example of
     each. Where adoption or a break needs an engine change, file the
     engine ticket the same way (`engine-change`, "ticket and
     follow-up") and block the adoption ticket on it.
   - In open tickets under *Unblocks* whose text says "the first step
     moves the pin", strike that step ("done by the pin move of
     <date>").
   - Write the ledger section.
   - Open the PR titled `Engine pin: <old 8> → <new 8> (N engine PRs)`,
     with `Closes #<issue>`, the ledger section and the tickets written
     in its body.

   Model `sonnet-5`, effort `medium`. A review that would need a design
   call goes to a ticket, never decided in the pin PR. A game ticket
   that moves the pin itself (`engine-change` §3.6, §7) follows the
   skill's review and ledger steps for the PRs it passes. If two pin
   moves are open at once, the second rebases, pins `main` again and
   extends its section.
6. **Text.**
   - `CLAUDE.md` hard rule 1: "Two exceptions: a GitHub issue labelled
     `sonar` (…), and one labelled `engine-pin`, filed by a watcher on
     the engine's `main` when its public API changes and worked with
     the `engine-pin` skill (ADR-0096)." Add `engine-pin` to the skills
     list and `engine-watch` to the crates row.
   - `tickets/README.md`: *The one exception* covers both issue kinds.
   - `docs/engine/README.md`, *Working with two repositories*: the
     watcher, when a move is due, the ledger, the skill.
   - `engine-change` §3.6 and §7: after `engine-pin`, follow the
     `engine-pin` skill's review and ledger steps for the engine PRs
     passed.
   - `work-ticket` §1: a ticket whose `engine#NNNN` is done but past the
     pin may wait for the open `engine-pin` PR instead of moving the pin
     itself, when that PR is already open.
   - `write-ticket`: don't write a ticket only to move the pin; the
     watcher files it. A ticket that needs a newer engine says its
     first step moves the pin (`engine-change` §7).
   - `docs/ROADMAP.md` lane F: the watcher replaces pin tickets (1177,
     1188, 1189 were the last); 1190 sets it up.
   - `docs/adr/README.md`: the new ADR in the index.
7. **The label:** `gh label create engine-pin --description "Move the
   engine pin: the engine's public API changed (ADR-0096)" --color
   5319e7`.
8. **Dry run.** In the Completion notes, say what `cargo xtask
   engine-pin --report` printed locally against the day's gap, and what
   a `workflow_dispatch` dry run printed once the workflow is on `main`
   (or that it is left for the first scheduled run, and why).

Expected size: about 400 production lines. The crate's types,
classifier, API-line test and renderer are about 230; the ledger parser
and check about 60; the `gh` calls about 70; the `xtask` wiring about
20. They are judged against `tickets.rs`'s `--check-engine` (a `gh`
lookup plus a pure check) and `sonar-triage.ps1`'s issue sync. The
workflow is about 80 lines of YAML, which `code-lines` doesn't count.

### Alternatives considered (for the ADR)

- **Keep pin tickets.** This is the status quo: each one is written by
  hand when someone waits for something, and the gap and the unread
  features grow in between (the Context's numbers).
- **A weekly move.** The first version of this ticket. Nick: too slow
  for an engine built in a week. Bug fixes and breaks would wait up to
  seven days.
- **Move the pin on every engine PR.** Ten engine PRs a day would mean
  ten pin PRs and gate runs a day, each colliding with game PRs in
  flight. One open issue that gathers PRs until it is worked moves just
  as fast with a fraction of the churn.
- **Polling every 15 minutes without a token.** The second version of
  this ticket. It needs no secret, but it adds up to 15 minutes of delay
  and about 100 runs a day. Nick (2026-10-25) preferred a token. The
  hourly backstop keeps polling's robustness for when a dispatch is
  missed.
- **`repository_dispatch` instead of `workflow_dispatch`.** It needs a
  token with Contents: write here, which can push code.
  `workflow_dispatch` needs only Actions: write (engine#2031).
- **The exact public API diff** (engine ADR-0038's `cargo xtask api`)
  run here: it needs a pinned nightly, `cargo-public-api` and
  `cargo-semver-checks`, and minutes per run. The engine already
  computes it on every PR, so engine#2030 publishes its verdict as a
  label instead. The `pub`-line fallback errs toward filing early,
  which is the cheap direction.
- **A bot that opens the pin PR itself.** A PR opened with the
  workflow's `GITHUB_TOKEN` doesn't start the `pull_request`
  workflows, so CI wouldn't run on it without a personal token in the
  repository. The adoption review also needs judgment that a workflow
  can't give.
- **Adopt in the pin PR.** It would mix a mechanical move with
  refactors of any size. Adoption as tickets keeps both reviewable and
  keeps one ticket per PR.
- **Dependabot or Renovate for the git dependency.** Neither bumps a
  `rev` across three files kept in step (`shuyi-engine.rev`,
  `Cargo.toml`, `Cargo.lock`), and neither reads the engine's PRs.

## Acceptance criteria

- [ ] `cargo xtask engine-pin --report` on the day the ticket is worked
      prints a report whose four sections hold every engine PR between
      the pin and `main`, and says whether a move is due and why. If the
      pin is still at `3f2256d4`:
      - #88 (engine 2013) is in *Breaking* or *New for games*, depending
        on whether its notes have a `Breaking:` line;
      - #102 (engine 2131) is in *Unblocks* with 1180;
      - #75 (tickets only) is in *Nothing for the game*.

      If the pin has moved past them, the report against that day's gap
      is in the Completion notes instead.
- [ ] Unit tests cover:
  - [ ] a ticket id from each title form;
  - [ ] `API_CRATES` read from a fixture of the engine's `api.rs`, and
        the fallback;
  - [ ] an `api: none` label wins over a `pub` line in the patch, an
        `api: added` label counts with no `pub` line, and `api:
        breaking` puts the PR in *Breaking*;
  - [ ] a `+pub fn` in `crates/core/src/` is a public API change, while
        `pub(crate) fn`, a change under `crates/core/tests/`, and a
        change in a crate outside `API_CRATES` are not;
  - [ ] a `Breaking:` line is found in the body and in the ticket
        notes, and a "No break" line is ignored;
  - [ ] a `Breaking:` PR that also unblocks a ticket is in *Breaking*;
  - [ ] a move is due for a public API change, for an unblock, and for
        code merged 49 hours before `now`, and not for code merged 47
        hours before or for tickets-only PRs;
  - [ ] an `@word` in a title comes out in backticks;
  - [ ] an empty gap renders no issue.
- [ ] `cargo xtask engine-pin --check` fails, with a message naming the
      problem, on each of these (tests):
      - a ledger whose last section isn't the pin;
      - a row with an empty decision;
      - an `adopt → NNNN` with no such ticket.

      It passes on the real repository.
- [ ] `.github/workflows/engine-watch.yml` exists and passes the
      `zizmor` job of `security.yml` on the PR. Stage 1 stops the job
      without checking out when the engine's `main` hasn't moved, and a
      dry run changes no issue.
- [ ] `gh workflow run engine-watch.yml -f engine_sha=<engine main>`
      (what engine#2031 sends) starts a run that applies. If
      engine#2031 is done by then, the Completion notes link the first
      run the engine started (`gh run list --workflow engine-watch.yml
      --event workflow_dispatch`).
- [ ] `trpg-engine-watch` builds without `macroquad` or any `shuyi-*`
      crate (`cargo tree -p trpg-engine-watch` shows neither).
- [ ] `.claude/skills/engine-pin/SKILL.md` exists, and `CLAUDE.md`,
      `tickets/README.md`, `docs/engine/README.md`, `engine-change`,
      `work-ticket`, `write-ticket` and `docs/ROADMAP.md` say what step
      6 says.
- [ ] ADR-0096 (or the next number) is in `docs/adr/` and its index.
- [ ] The `engine-pin` label exists on the repository.
- [ ] `cargo xtask ticket-lint` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the crate's parsing, API-line test, classifying, due rule and
  rendering on fixture PRs; the ledger check (step 3).
- Integration: none against the network. `the_real_repo_agrees_with_itself`
  keeps checking the real ledger.
- The workflow: a dry run (step 8), the way ADR-0071 checks the sonar
  job.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
