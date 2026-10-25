---
id: "1190"
title: "The engine pin moves every week: a scheduled engine-pin issue, an adoption review of each engine PR it passes, and a ledger CI checks"
type: infra
milestone: "Engine: carve"
model: sonnet-5
effort: high
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1190 — The engine pin moves every week, with an adoption review and a ledger

## Context

The game builds on one Shuyi Engine commit, the pin (`shuyi-engine.rev`,
ADR-0063 §8, `docs/engine/README.md`). Today it moves only when someone
writes a ticket for it: 1177 and 1188 (both 2026-10-22) and 1189 (open),
each hand-written, each naming the few engine tickets someone was waiting
for. Nick (2026-10-24) asked whether that is the right approach and asked
for a robust one: the pin should move at a controlled, visible rate, and
each move should take up what the engine now gives, so the game writes
less code of its own ("if the engine implements AI behavior, we use its
AI primitives").

What the ad hoc moves cost, measured on 2026-10-24:

- **The gap grows unseen.** The pin (`3f2256d4`, 1188) is 25 engine
  commits behind the engine's `main` (`2659928c`), merged over
  2026-10-23 to 10-25. Nothing in this repository says so.
- **Breaks pile up.** Engine 2013, 2024 and 2025 (PRs #88, #90, #91)
  moved public types onto constructors and `#[non_exhaustive]`
  (`Window`, `AppConfig`, `BoxPlace`, `MapScene`, the game tools'
  types). The game builds some of them by literal, so the next move,
  whoever makes it, inherits every break at once.
- **Ready tickets each move the pin themselves.** Engine 3056 (one-bit
  sprites drawn fast) unblocks 1078; 2131 (threat areas), 2132 (the
  candidate order) and 2533 (playtest history) are three of 1180's
  engine tickets; 2545 and 1134 are 1189's. By `engine-change` §7, the
  first step of each is its own pin move, so two of them in parallel
  worktrees fight over `Cargo.lock`.
- **What nobody asked for is never looked at.** The engine ships things
  the game could use that no game ticket names: 2511 (engine defaults
  for every `GameData` part, a prelude, `run_game`), 2358, 2372 and 2003
  (modules, and a module's debug menu entries), 2012 (the pinned Rust
  kept in step). A pin move today checks only that the game still
  builds; no one decides "use it" or "not for us", so the game keeps
  its own copies. The friction log (`docs/engine/friction.md`) catches
  the cases where game code came out big, not the cases where the
  engine quietly caught up.

### The decision (this ticket's ADR records it)

1. **A schedule.** `.github/workflows/engine-pin.yml` runs every Monday
   at 07:00 UTC (after `sonar-triage` at 06:00) and from the Actions tab
   as a dry run. It runs `cargo xtask engine-pin --report` and keeps
   **one** open GitHub issue labelled `engine-pin` in sync: it creates
   it, edits it to the newest engine `main` on later runs while it is
   still open, and closes it when the pin is at `main`. Weekly bounds the
   gap to about a week of engine work and its review to one session;
   daily would be a pin PR a day of gates for little; monthly would let
   breaks and unread features pile up as they do now.
2. **The issue is its own ticket**, like a `sonar` issue (ADR-0071):
   the second exception to `CLAUDE.md` hard rule 1. A new
   `engine-pin` skill works it: one PR on a branch
   `engine-pin/<new pin's first 8 hex>` that moves the pin to the
   engine's `main`, fixes the breaks game side, writes the ledger
   section, writes the adoption tickets, and closes the issue. No more
   hand-written "move the pin" tickets.
3. **The report sorts every engine PR the move passes** into:
   - **Breaking:** a `Breaking:` line in the PR body or in its engine
     ticket's Completion notes (the engine requires one for every API
     break, engine ADR-0038). Fixed in the pin PR, game side, behaviour
     unchanged.
   - **Unblocks:** its engine ticket is in an open game ticket's
     `blocked_by`, or in an `// engine-friction: engine#NNNN` marker in
     `crates/`. Those game tickets become ready; the pin PR does
     nothing else for them.
   - **New for games:** it changes code under `crates/` and is neither
     of the above. Each one gets a decision in the review.
   - **Nothing for the game:** no change under `crates/` (tickets,
     docs, CI, templates, examples, skills). Listed, collapsed.
4. **The adoption review** ("use what the engine gives"). For each
   *New for games* row, the session asks two questions: does the game
   have code doing this job (search `crates/` for it, the friction
   log's rows for that area, `docs/engine/boundary.md`), and does an
   open ticket plan to write code for it (`grep -rn` in
   `tickets/open`)? Then exactly one decision:
   - **adopt → NNNN**: a game ticket (`write-ticket`) that moves the
     game onto it and deletes the game's copy, with the lines it
     expects to remove; or an existing open ticket's text changed to
     use the engine's piece instead of writing its own;
   - **not for the game:** one line saying why ("templates for new
     games", "the game has no modules").
   The pin PR itself stays mechanical: the pin, the break fixes, the
   ledger, the tickets. Adoption is ordinary ticket work, one ticket
   per PR, so a pin move never turns into a refactor.
5. **The ledger**, `docs/engine/pins.md`: one section per pin move,
   scheduled or not, with a row and a decision for every *Breaking*,
   *Unblocks* and *New for games* PR, and a count of the rest.
   `cargo xtask engine-pin --check` (already in CI) fails when the
   ledger's last section isn't the current pin, when a row has no
   decision, or when an `adopt → NNNN` names a ticket that doesn't
   exist. So every pin move, including a ticket's own (`engine-change`
   §3.6 and §7), writes its review, and nothing the engine ships passes
   unread. The weekly report also lists adoption tickets still open,
   with their age, so the cleanup stays visible until it is done.

A ticket that needs an engine commit past the pin still moves the pin
itself (`engine-change` §3.6, §7; the engine never makes the game
wait). It follows the `engine-pin` skill's review for the PRs it
passes; the next weekly report then starts from its pin.

## Nick input

None. Nick asked for the change (2026-10-24); how it works is Claude's
call. Nick sees it as a weekly `engine-pin` issue on GitHub and a PR
that closes it.

## Scope

**In:**
- ADR-0096 (or the next free number): the decision above, with the
  alternatives below.
- `cargo xtask engine-pin --report` in `crates/xtask/src/engine.rs` (or
  a new `engine_report.rs` beside it): the report as Markdown on stdout,
  and `--json` with the counts the workflow needs.
- `.github/workflows/engine-pin.yml`: schedule, dry-run dispatch, issue
  sync.
- `docs/engine/pins.md` with its first section, and its check in
  `engine-pin --check`.
- `.claude/skills/engine-pin/SKILL.md`.
- Text: `CLAUDE.md` hard rule 1 and the skills list, `tickets/README.md`
  (*The one exception* becomes two), `docs/engine/README.md`,
  `engine-change` §3.6 and §7, `work-ticket` §1, `write-ticket` (no
  "move the pin" tickets), `docs/ROADMAP.md` lane F, `docs/adr/README.md`.
- The `engine-pin` label on the repository (`gh label create`).

**Out (do not do):**
- Moving the pin. The first issue the workflow files does that (after
  merging, run the workflow from the Actions tab with `dry_run` off, or
  wait for Monday). 1189 is either order: if it is still open when this
  lands, it is worked as a pin move under the new rules (it writes the
  ledger's next section, and the issue's next sync starts from its pin);
  if it merges first, the ledger's first section is its pin.
- Adopting any engine feature (1180, 1078 and the rest are their own
  tickets).
- Engine changes. The engine already writes `Breaking:` lines and names
  `game#NNNN` follow-ups; the report reads what is there.
- Working the issue automatically (a scheduled Claude session). The
  issue is worked like a `sonar` issue, by a session someone starts.

## Implementation steps

1. **The report, pure part** (`engine_report.rs`, unit-tested with
   fixtures, no network):
   - `EnginePr { number, title, merged_at, merge_sha, ticket: Option<String>, body, files: Vec<String>, ticket_notes: Option<String> }`.
     The engine ticket comes from the title: `[NNNN] …` or
     `Ticket NNNN: …` / `Tickets NNNN, …` (ticket-only PRs).
   - `breaking_lines(pr) -> Vec<String>`: lines starting `Breaking:` in
     the body or the ticket's notes; `No break`/`No breaking` lines are
     not breaks.
   - `touches_code(pr)`: any file under `crates/` that is not under a
     `tests/` folder.
   - `classify(prs, open_tickets, markers, ledger) -> Report`, where
     `open_tickets` is `(id, blocked_by)` from `tickets/open/` (reuse
     `tickets.rs`'s parser) and `markers` the `engine#NNNN` ids in
     `// engine-friction:` comments under `crates/`. A PR goes in the
     first section that fits, in the order *Breaking*, *Unblocks*, *New
     for games*, *Nothing for the game*.
   - `render(report) -> String`: the issue body. It opens with
     `<!-- engine-pin:v1 -->`, then: the pin, `main`, the number of
     engine PRs between, and the merge date of the oldest; the four
     sections as tables (PR link, engine ticket, title, and for
     *Breaking* its lines, for *Unblocks* the game tickets);
     *Nothing for the game* inside `<details>`; *Adoption tickets still
     open* (each `adopt → NNNN` in the ledger whose ticket is in
     `tickets/open/`, with the ledger date); and a checklist of the
     work, pointing at the `engine-pin` skill. Put any `@word` from an
     engine title in backticks (`CLAUDE.md`).
   - The title: `Engine pin: N engine PRs behind (oldest merged YYYY-MM-DD)`.
2. **The report, network part** (thin; `gh` shell-outs as
   `tickets.rs`'s `--check-engine` does): `gh api
   repos/Zafnok/shuyi-engine/compare/<pin>...main` for the commits (fail
   with a clear message past 250 commits: "move the pin in steps");
   the PR number from each commit title's `(#NN)`; `gh pr view NN
   --repo Zafnok/shuyi-engine --json title,body,files,mergedAt`; the
   ticket's notes from `tickets/done/<NNNN>-*.md` at the merge commit
   when the title names a ticket. Exit 0 with an empty report when the
   pin is at `main`.
3. **The ledger and its check.** `docs/engine/pins.md`: a short header
   (what it is, ADR-0096), then sections, newest last, headed
   `## YYYY-MM-DD: <from 8 hex> → <to 8 hex> (N engine PRs)` with the
   game PR once it exists, a table `| Engine PR | Ticket | What |
   Decision |`, and a line `Nothing for the game: N PRs (#…, #…)`.
   The first section is the baseline: `## <today>: baseline → <pin 8
   hex>`, no rows, one line saying the pins before it were moved by
   tickets (1177, 1188, and 1189 if merged) without this review. In
   `engine.rs`, `problems` (or a sibling `ledger_problems`, called from
   `check`) adds: the last heading's `to` isn't a prefix of
   `shuyi-engine.rev`; a row's Decision cell is empty; an `adopt →
   NNNN` names no file in `tickets/open/` or `tickets/done/`. Tests
   beside the existing ones (`the_real_repo_agrees_with_itself` must
   still pass on the real files).
4. **The workflow** `.github/workflows/engine-pin.yml`, modelled on
   `sonar-triage.yml`: `schedule: cron "0 7 * * 1"` and
   `workflow_dispatch` with `dry_run` (default true); `permissions: {}`
   at the top and `contents: read`, `issues: write` on the job; actions
   pinned by SHA as in the other workflows; set up Rust and the system
   packages `xtask` needs to build as `ci.yml` does. Then, in a script
   reading inputs through `env:` (never `${{ }}` inside `run:`):
   `cargo xtask engine-pin --report --json`; if nothing is behind, close
   the open `engine-pin` issue with a comment and stop; else find the
   open issue with the label (`gh issue list --label engine-pin --state
   open`), edit its title and body, or create it. A dry run prints the
   title and body and changes nothing.
5. **The skill** `.claude/skills/engine-pin/SKILL.md`, in the style of
   `sonar-fix`: the issue is the ticket; re-run `--report` (the
   engine's `main` may have moved: pin the newest); branch
   `engine-pin/<8 hex>`; `cargo xtask engine --off`; `cargo xtask
   engine-pin <main>`; build and gates (`run-gates`), fixing *Breaking*
   rows game side with behaviour unchanged and any snapshot change
   explained by its engine PR; `cargo shuyi mcp --ready`; the adoption
   review of each *New for games* row (the two questions and the two
   decisions of the Context, with an example of each); open tickets in
   *Unblocks* whose text says "the first step moves the pin" get that
   step struck ("done by the pin move of <date>"); the ledger section;
   the PR titled `Engine pin: <old 8> → <new 8> (N engine PRs)` with
   `Closes #<issue>`, the ledger section and the tickets written in its
   body. Model `sonnet-5`, effort `medium`; a review that would need a
   design call goes to a ticket, never decided in the pin PR. Also: a
   game ticket that moves the pin itself (`engine-change` §3.6, §7)
   follows the skill's review and ledger steps for the PRs it passes.
   Two pin moves open at once: the second rebases, pins `main` again and
   extends its section.
6. **Text.**
   - `CLAUDE.md` hard rule 1: "Two exceptions: a GitHub issue labelled
     `sonar` (…) and one labelled `engine-pin`, filed weekly when the
     engine's `main` is past the pin and worked with the `engine-pin`
     skill (ADR-0096)." Add `engine-pin` to the skills list.
   - `tickets/README.md`: *The one exception* covers both issue kinds.
   - `docs/engine/README.md`, *Working with two repositories*: the
     weekly move, the ledger, the skill.
   - `engine-change` §3.6 and §7: after `engine-pin`, follow the
     `engine-pin` skill's review and ledger steps for the engine PRs
     passed.
   - `work-ticket` §1: a ticket whose `engine#NNNN` is done but past the
     pin may also wait for the open `engine-pin` issue's PR instead of
     moving the pin itself, when that PR is already open.
   - `write-ticket`: don't write a ticket only to move the pin; the
     weekly issue does it. A ticket needing a newer engine says its
     first step moves the pin (`engine-change` §7).
   - `docs/ROADMAP.md` lane F: the weekly pin move replaces pin tickets
     (1177, 1188, 1189 were the last); 1190 sets it up.
   - `docs/adr/README.md`: the new ADR in the index.
7. **The label:** `gh label create engine-pin --description "Weekly move
   of the engine pin (ADR-0096)" --color 5319e7`.
8. **Dry run after merge.** Say in the Completion notes what a
   `workflow_dispatch` dry run on `main` printed (or, before merge,
   what `cargo xtask engine-pin --report` printed locally against the
   day's gap).

Expected size: about 350 production lines (the report's types,
classifier and renderer ~220, its `gh` calls ~60, the ledger check
~50, wiring ~20), judged against `tickets.rs`'s `--check-engine` (a
`gh` lookup plus a pure check) and `sonar-triage.ps1`'s issue sync; the
workflow is about 60 lines of YAML, not counted by `code-lines`.

### Alternatives considered (for the ADR)

- **Keep pin tickets.** The status quo: each one written by hand when
  someone waits for something; the gap and the unread features grow in
  between (the Context's numbers).
- **Move the pin in every game PR that builds.** Spreads engine breaks
  across unrelated PRs and makes every PR's diff carry engine churn.
- **A bot that opens the pin PR itself.** A PR opened with the
  workflow's `GITHUB_TOKEN` doesn't start the `pull_request`
  workflows, so CI wouldn't run on it without a personal token in the
  repository; and the adoption review needs judgment a workflow can't
  give. The issue carries the report; a session does the rest.
- **Adopt in the pin PR.** One PR would mix a mechanical move with
  refactors of any size; adoption as tickets keeps both reviewable and
  one ticket per PR.
- **Dependabot or Renovate for the git dependency.** Neither bumps a
  `rev` across three files kept in step (`shuyi-engine.rev`,
  `Cargo.toml`, `Cargo.lock`), and neither reads the engine's PRs.
- **Daily or monthly.** Daily costs a gate run and a session a day;
  monthly brings back the pile-up.

## Acceptance criteria

- [ ] `cargo xtask engine-pin --report` on the day it is worked prints
      a report whose four sections hold every engine PR between the pin
      and `main`; the engine PRs #88, #90 or #91 sort into *Breaking*
      if their engine notes have a `Breaking:` line (or the Completion
      notes say what they hold), and #102 (engine 2131) into *Unblocks*
      with 1180 — or, if the pin has moved past them by then, the
      report against that day's gap is in the Completion notes.
- [ ] Unit tests cover: a ticket id from each title form; a
      `Breaking:` line found in the body and in the ticket notes, and a
      "No break" line ignored; a PR that touches only `tickets/` and
      `docs/` is *Nothing for the game*; a PR whose ticket is in an open
      ticket's `blocked_by` is *Unblocks*; a `Breaking:` PR that also
      unblocks a ticket is *Breaking*; an `@word` in a title comes out
      in backticks; an empty gap renders no issue.
- [ ] `cargo xtask engine-pin --check` fails, with a message naming the
      problem, on a ledger whose last section isn't the pin, on a row
      with an empty decision, and on `adopt → NNNN` with no such ticket
      (tests), and passes on the real repository.
- [ ] `.github/workflows/engine-pin.yml` exists, passes the `zizmor`
      job of `security.yml` on the PR, and a dry run changes no issue.
- [ ] `.claude/skills/engine-pin/SKILL.md` exists, and `CLAUDE.md`,
      `tickets/README.md`, `docs/engine/README.md`, `engine-change`,
      `work-ticket`, `write-ticket` and `docs/ROADMAP.md` say what step
      6 says.
- [ ] ADR-0096 (or the next number) is in `docs/adr/` and its index.
- [ ] The `engine-pin` label exists on the repository.
- [ ] `cargo xtask ticket-lint` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the report's parsing, classifying and rendering on fixture PRs;
  the ledger check (step 3).
- Integration: none against the network; `the_real_repo_agrees_with_itself`
  keeps checking the real ledger.
- The workflow: a dry run (step 8), as ADR-0071 checks the sonar job.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
