---
name: work-ticket
description: Pick up, implement, verify, archive and open a PR for exactly one ticket from tickets/open/. Use whenever asked to "work ticket NNNN", "do the next ticket", or start any implementation work in this repo. Not for a sonar or engine-pin issue (sonar-fix, engine-pin) or for writing a new ticket (write-ticket).
metadata:
  version: "1.0"
---

# Work a ticket

One ticket = one branch = one PR. Do **only** what the ticket says. This repo's
owner (Nick) does not review code; CI and this checklist are the review, so
discipline here is the whole quality system.

## 0. Orient (every session, ~2 minutes)

1. Read `AGENTS.md`.
2. Read `tickets/README.md`.
3. Read the ticket file fully, then every ADR and design doc it links.

## 1. Choose the ticket

- If the user named a ticket, use it.
- Otherwise list candidates: tickets in `tickets/open/` with `status: todo` whose
  every `blocked_by` id exists in `tickets/done/`. Prefer the lowest number on
  the Act 1 critical path in `docs/ROADMAP.md`. Tell the user which one you
  picked and why in one sentence.
- **An `engine#NNNN` in `blocked_by`** is a Shuyi Engine ticket. It is done
  only when it is in the engine's `tickets/done/` **and** the game's pin is
  at or past the commit that closed it. Check both, for a ticket you picked
  or one you were given:

  ```bash
  gh api repos/Zafnok/shuyi-engine/contents/tickets/done --jq '.[].name' | grep '^NNNN-'
  gh api "repos/Zafnok/shuyi-engine/commits?path=tickets/done/<that file>" --jq '.[-1].sha'
  gh api "repos/Zafnok/shuyi-engine/compare/<that sha>...$(cat shuyi-engine.rev)" --jq .status
  ```

  `ahead` or `identical`: the pin has it. `behind` or `diverged`: the
  engine ticket is done but the pin is not past it, so the ticket is ready
  and its first step moves the pin (`engine-change` skill, step 7, with
  the `engine-pin` skill's review and ledger section for the engine PRs
  the move passes). When a PR for an `engine-pin` issue is already open
  (`gh pr list --search "head:engine-pin/"`), the ticket may wait for it
  to merge instead of moving the pin itself. Not in `done/`: blocked. `cargo xtask ticket-lint --check-engine` reports an
  `engine#NNNN` that doesn't exist at all.
- If the ticket's **Nick input** says *Answer first* and the design doc it needs
  doesn't exist yet in `docs/design/`, stop and tell the user which `00xx`
  ticket must be answered first. Do not guess game-design answers.

## 1b. Get the design unknowns out first (Nick asked for this)

Nick's turns are expensive; a question answered late costs a full
implement → test → gates → push cycle. Before writing code:

1. List every gameplay rule the ticket needs that `docs/design/` doesn't
   settle, and every conflict you can see between the ticket, the design
   docs and the data (e.g. a skill locked to a weapon its class can't
   wield). Check new answers against the data the same way.
2. Ask them all at once with the `ask-nick` skill, scoped to this ticket,
   before implementing. Only a rule too small to matter goes in as a
   *the agent's starting rule*.
3. When Nick's reply leaves a conflict or ambiguity, ask about it **before**
   making the rest of the changes, not after the gates have run.

## 2. Start

```bash
git switch main && git pull --ff-only
git switch -c t<NNNN>-<slug>
```

Then free disk space:

```bash
cargo xtask clean-merged-targets
```

It deletes the `target/` build folder of every other worktree whose PR has
merged (never a worktree or a branch). If this command fails, carry on: it
never blocks the ticket.

Set `status: in-progress` in the ticket frontmatter.

## 3. Implement

- **Before the first code, write the estimate** (ADR-0092; the
  `engine-friction` skill, step 1): one sentence and a number, how many
  production lines this change *should* take with a good engine, and the
  comparable code you judged it by (another engine doing the same job, or
  a similar piece of this repository). A size the ticket states is a
  starting point to check against that reference, never the reference
  itself. A ticket with no code (docs, data, a decision) skips this.
- **Estimate the PR's size too** (ADR-0101): all the changed lines `cargo
  xtask pr-size` will count (code, tests, docs, the ticket file, hand-written
  data), not only the production lines above. Judge it by a done ticket of
  the same kind. **Past about 2,000, split the ticket first** with
  `write-ticket`, then work the first piece: a session that plans 3,000 and
  runs over runs out of context.
- **While you work:** a hook runs `pr-size` before each commit, push and PR
  and tells you when the branch passes 2,000 (note where it would split) or
  3,000 (can it split cleanly?). Take it seriously: finish the part that
  works alone, move the rest to a new ticket (`write-ticket`), and stack the
  PRs if the second needs the first. If it can't split cleanly, ship it and
  say why. The warning never blocks.
- Follow the ticket's **Implementation steps** in order. If a step turns out to
  be wrong, deviate minimally and explain it in the Completion notes.
- Respect crate boundaries (ADR-0004): no I/O, clock or macroquad outside `app`;
  rules only in `core`; state changes only via `Command`s.
- Touching input, controls, help bars, tips or any text that names a key:
  follow the `keyboard-input` skill (never hard-code a key).
- Drawing a picture, or anything on the battle map: follow ADR-0038
  (graphics are a skin). Pictures are sprite items from asset files; map
  things go in the map scene and are painted by each map skin; nothing
  about the look goes in `core`, the bots or play records; a test of what
  happened reads the scene or the state, not cells or colours.
- Write the tests the ticket lists (ADR-0007): unit + property for `core`,
  snapshot + scripted integration for screens.
- A test that checks what happened reads the scene or the state. Only a
  test of a look reads cells, colours or items, and it lives with the
  skin (`crates/ui/src/map_view/glyph*`). Harness shortcuts:
  `cursor_tile()`, `unit_at(pos)`, `tints_at(pos)`, `path()`
  (`crates/ui/README.md`).
- New integration tests go in the crate's `tests/it/` as a module listed in
  `main.rs`; never add a file directly under `tests/` (each one is a
  separate ~100 MB test program; an `xtask` test fails on a second one).
- **The ticket needs an engine change:** follow the `engine-change` skill:
  an engine PR first, in `Zafnok/shuyi-engine`, then this ticket's PR moves
  the pin; or, when the change is bigger than one PR, an engine ticket, a
  marked workaround here and a follow-up game ticket.
- **Scope creep rule:** if you notice something else worth doing (a bug,
  refactor, missing feature), do NOT do it. Create a new ticket with the
  `write-ticket` skill and mention it in the PR description.
- **Adding a dependency or third-party asset** (crate, font, JS file, image,
  sound, SDK): check its license against ADR-0013 *before* using it. Only the
  allowed permissive licenses; no GPL/LGPL/MPL/copyleft, no non-commercial, no
  "GPL or buy a commercial license", nothing requiring payment. For crates,
  `cargo deny check licenses` must pass. For anything else, add a row to
  `THIRD_PARTY_ASSETS.md` and commit its license text next to it. If unsure,
  don't add it: pick another or write the small piece yourself.
- **Every third-party asset gets its credit in the same PR** (Nick:
  "credits should always update when we introduce an asset"): music and
  sounds in `assets/audio/audio.ron`, anything else in
  `assets/data/credits.ron`, with the row's source link. **Bought art**
  (ADR-0051): add the new files' path in `assets-private/game/` to its
  credit's `private` list, the pack to its row and the artist to the
  credit's `author`. `cargo shuyi private-assets --pin` refuses files
  without a credit; don't widen an old path just to get past it.
- A new architectural choice (new dependency with wide impact, new pattern,
  new file format) needs an ADR in the same PR (`write-adr` skill). Adding a
  small, well-known crate for a local need does not.

## 4. Verify

Run the `run-gates` skill. Everything must pass locally before you push.

A change a player sees can be checked through the MCP server's session
tools (`start`, `step`, `scene`, `frame_png`; `.mcp.json`) besides
`cargo shuyi frame-png`.

Measure the change against the estimate: `cargo xtask code-lines`, the
**added** column of `total (game)` (ADR-0092). Ratio = measured ÷
estimate:

- under 2×: nothing;
- 2× to 3×: one line in the Completion notes ("friction: 2.4×, accepted
  because …") and a row in `docs/engine/friction.md`;
- 3× or more, or any signal (a workaround for an engine assumption, the
  third copy of the same boilerplate, a performance budget missed because
  of the engine): run the `engine-friction` skill before opening the PR.
  At 10× or more the fix comes first unless the review says why not.

A finding never blocks the ticket: fix first, or ship and follow up.

Run `cargo xtask pr-size` and compare it with your estimate (ADR-0101).
Walk the ticket's **Acceptance criteria** and tick each box honestly. If one
cannot be met, say so in the Completion notes and in the PR; don't hide it.

A grep that proves something is gone searches for what the code does, in
every spelling (`is_finite()` on a frame time, not `dt.is_finite()`, which
missed the five sites 1167 had renamed that morning; a loader that skips a
file silently has no error message to search for). It runs again after
merging `main` into the branch, before the PR merges (§6).

## 5. Archive the ticket (same PR)

1. Fill in the ticket's `## Completion notes`: what was done, deviations from the
   plan, follow-up tickets created, anything Nick should know when playing.
   List every **gameplay-affecting rule you had to decide** where the design
   docs were silent (mark them *the agent's starting rule*), in plain words.
   Repeat that list in the PR body and in your final message to Nick, so he
   can agree or veto (Nick asked for this).
2. Set `status: done` and `completed: YYYY-MM-DD` in frontmatter.
3. `git mv tickets/open/<file>.md tickets/done/<file>.md`

## 5b. No checks after merge on gameplay tickets

Nick, 2026-10-21: "any tickets that say 'check after merge' aren't gonna happen -- I
will save all my checking for either design (i.e. 00 tickets) or for
the full act 1 playtest itself. I don't wanna spot verify any gameplay
tickets anymore."

- A `feature`, `bug`, `tuning` or `content` ticket is verified by its
  tests, `cargo shuyi frame-png` pictures in the PR and, where it
  touches play, the bots (`cargo xtask playtest`, `autobalance`).
- Whatever is still left to judge by playing (a starting rule, a feel,
  a look) goes in the Completion notes; the Act 1 playtest (1064)
  collects it. Don't send it to Nick.
- If an older ticket still says "Sign-off" after merge, treat it
  this way and say so in the PR.
- Design questions are still asked, **before** building (§1b), with
  mockups when Nick judges pictures.

## 6. Commit and PR

- Commit messages: `[NNNN] imperative summary`.
- Push and open a PR:
  - Title: `[NNNN] <ticket title>`
  - **No bare `@word` in a PR title, PR body, commit message or comment.**
    GitHub reads it as a mention of the user with that name: the release
    notes are generated from PR titles, and v0.1.0 listed the stranger
    `github.com/music` as a contributor because PR #157 was titled
    "...: @music" after the dialogue directive. Put backticks around it
    (`` `@music` ``) or leave the `@` out ("the music directive"). If the
    ticket's own title has one, change it for the PR title. `cargo xtask
    ticket-lint` checks open tickets' titles and CI checks the PR title
    (`cargo xtask check-mentions`) on each push; the rest is on you.
  - Body: summary bullets, list of acceptance criteria (checked), follow-up
    tickets created, an `Engine friction:` line ("Engine friction:
    estimate N, measured M (R×), <outcome or 'under 2×'>; signals:
    workaround no · third copy no · budget no", each "yes" naming the
    review's outcome; "none: no code" when there is none), a `PR size:` line ("PR size: estimate N,
    measured M", with the reason if M passes about 3,000), and `Nick input:` line (e.g. "None: verified by tests and the
    pictures above; starting rules listed for the Act 1 playtest
    (1064)"). Never ask Nick to check a gameplay ticket after merge
    (§5b).
- Don't run mutation testing locally before pushing: CI runs it on the PR
  (see `run-gates`). Push, open the PR, and watch CI once.
- Wait for CI. If a check fails, fix it on the same branch. Never disable a
  gate, lower a threshold, or add `#[mutants::skip]`/`#[allow]` just to pass —
  if a gate is genuinely wrong, write a ticket about it and explain in the PR.
- A PR can merge when its checks are green and GitHub reports no conflict; it
  need not contain the latest `main` (ADR-0042). Merge `main` into the branch
  (`git fetch origin && git merge origin/main`, or the ccd_host
  `sync_with_base_branch` tool in an app worktree) only for a conflict or when
  the PR needs something that landed on `main`. An acceptance grep that
  proves something is gone (§4) needs it: merge, run the grep again, then
  merge the PR.

## Don'ts

- Don't work two tickets in one PR.
- Don't ask Nick technical questions. Decide, and write an ADR if it matters.
- Don't invent game-design answers (stats, magic, story) — those come from
  `docs/design/` and `docs/story/beats.md`.
- Don't edit accepted ADRs; supersede them.
