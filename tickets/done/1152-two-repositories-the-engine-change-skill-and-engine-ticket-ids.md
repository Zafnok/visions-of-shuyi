---
id: "1152"
title: "Two repositories: an engine-change skill, engine ticket ids in blocked_by, and the engine side's intake for game requests"
type: infra
milestone: "Engine: carve"
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1131", "1151"]
nick_input: none
completed: 2026-10-19
---

# 1152 — Two repositories: the engine-change skill

## Context

After 1131 the engine's code lives in
[`Zafnok/shuyi-engine`](https://github.com/Zafnok/shuyi-engine). The
game depends on it by a pinned revision (`shuyi-engine.rev`, `cargo
xtask engine-pin`), and maybe through a local checkout with a Cargo
`[patch]` (ADR-0063 §8). 1131 adds one line to `CLAUDE.md` and
`work-ticket`: "a ticket that needs an engine change is an engine PR
first, then a game PR that moves the pin".

That one line is not enough to keep two repositories sustainable. Nick,
2026-10-15: "we will be juggling 2 repos. To make this sustainable, we
should develop some skills", and engine work must be easy to start in
parallel, never a reason to wait. 1151's friction rule sends sessions to
the engine often ("fix first", "ship and follow up"). This ticket makes
that path a skill an implementation session can follow without guessing.

What it needs to cover, from both sides:

- **From the game:** how to make an engine change while working a game
  ticket, and land both.
- **From the engine:** how a request from the game arrives (the engine
  repository's tickets already name game tickets as `game#NNNN`, and its
  `work-ticket` checks them with `gh api`). The game needs the mirror
  image, `engine#NNNN`.
- **Ticket links both ways:** `cargo xtask ticket-lint` today rejects any
  `blocked_by` id it does not find here (`check_blocked_by_exists` in
  `crates/xtask/src/tickets.rs`).

## Nick input

None.

## Scope

**In:**
- **`engine#NNNN` in `blocked_by`.**
  - `ticket-lint` accepts an id of the form `engine#NNNN` without
    looking it up by default. With `--check-engine` it looks the id up
    in `shuyi-engine/tickets/` when the local checkout exists. Otherwise
    it uses `gh api repos/Zafnok/shuyi-engine/contents/tickets/{open,done}`.
    CI does not pass the flag (no network dependency in the gates).
  - `work-ticket` §1: a ticket is ready only when each `engine#NNNN` is
    in the engine's `tickets/done/` **and** the game's pin is at or past
    the commit that closed it. It names the check command.
  - `tickets/README.md`: the `engine#NNNN` form in the Frontmatter
    section.
- **The skill** `.claude/skills/engine-change/SKILL.md`, for a game
  session that needs the engine changed. Steps:
  1. Confirm the change passes 1151's leak test and the ADR-0063 §5
     test. If not, it is a game change; stop.
  2. Decide the size. One engine PR's worth or less → do it now, in this
     session. Larger → write an engine ticket in the engine repository
     (its `write-ticket` skill, with `game#NNNN` naming this ticket),
     ship the game ticket with the marked workaround (1151), and write
     the game follow-up ticket blocked by `engine#NNNN`.
  3. Doing it now:
     - work in the local checkout (`cargo xtask engine`; create it if
       missing);
     - branch there as the engine's `work-ticket` says, and if the
       change has no ticket yet, write it there first;
     - develop both sides against the `[patch]`;
     - run the engine's gates, then push and open the engine PR, with a
       body that links the game ticket;
     - when it merges, `cargo xtask engine-pin`, and the game PR moves
       `shuyi-engine.rev`. The game PR is never merged with the
       `[patch]` in place; CI catches this because it never uses it.
  4. Breaking changes: once the engine has a crates.io release, follow
     its `CHANGELOG.md` and version rule (its `work-ticket`). Before
     that, a breaking change is fine if the same session moves the
     game's pin and fixes the game.
  5. Write the PR bodies so each names the other (`Zafnok/shuyi-engine#N`,
     `Zafnok/visions-of-shuyi#N`). Keep the bare-`@` rule from
     `CLAUDE.md`.
- **The engine side's intake**, as a PR in the engine repository (this
  ticket opens it):
  - its `write-ticket` skill gets a section "A request from the game".
    It says what the ticket must hold:
    - the game ticket id;
    - the job in one sentence;
    - the friction numbers from the game's `docs/engine/friction.md` row;
    - the generic shape (not the game's);
    - an example a non-tactics game would also write.
  - its `work-ticket` skill: a ticket with a `game#` link also lists the
    game PR that will move the pin, in its Completion notes.
- **Parallel sessions.** The skill says when to start the engine work in
  its own session instead of this one (the desktop app's
  `start_session`): when the engine change is larger than the game
  ticket, or touches engine files another open engine PR touches. The
  game session carries on with the marked workaround.
- **Docs:** `CLAUDE.md` (replace 1131's line with a pointer to the
  skill; add the skill to the skill list), `docs/engine/README.md` (a
  "Changing the engine from the game" section in five lines with a link
  to the skill), and 1151's skill (replace "see 1152" with a link to
  this skill).

**Out (do not do):**
- moving the code or the pin mechanics themselves (1131);
- the engine's crates.io release process (an engine-repository ticket);
- the friction sweep (1153);
- making `ticket-lint` reach the network in CI.

## Implementation steps

1. `crates/xtask/src/tickets.rs`: accept `engine#NNNN` in
   `check_blocked_by_exists` (a regex or prefix check, four digits);
   add `--check-engine` with the lookup. Unit tests for both.
2. Write `.claude/skills/engine-change/SKILL.md`.
3. Update `work-ticket` §1, `tickets/README.md`, `CLAUDE.md`,
   `docs/engine/README.md` and 1151's skill.
4. In a checkout of the engine repository, update its `write-ticket` and
   `work-ticket` skills. Open that PR and link it from this PR's body.
5. Walk the skill once for real with a no-op: a one-line doc change in
   the engine (for example a typo or a missing doc link). Open the
   engine PR, move the pin after it merges, and note both PRs in the
   Completion notes. If the walk-through finds a step that is wrong,
   fix the skill.

## Acceptance criteria

- [x] `cargo xtask ticket-lint` passes with an `engine#NNNN` id in a
      `blocked_by` (unit test with a fixture ticket), and still fails on
      an unknown plain id.
- [x] `cargo xtask ticket-lint --check-engine` reports an `engine#NNNN`
      that does not exist in the engine repository.
- [x] `.claude/skills/engine-change/SKILL.md` exists. It covers the
      leak test, fix-now vs ticket-and-follow-up, the local checkout,
      the pin, the cross-links and parallel sessions.
- [x] The engine repository's intake PR is open or merged, and linked.
- [x] The no-op walk-through's two PRs are named in the Completion notes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `tickets.rs`:
  - `engine#1234` accepted;
  - `engine#12` and `engine#abcd` rejected;
  - an unknown plain id still rejected;
  - the `--check-engine` lookup over a fake directory listing.
- No snapshot tests.

## Completion notes

Done 2026-10-19 (Opus 5.5, high).

- **`engine#NNNN` in `blocked_by`** (`crates/xtask/src/tickets.rs`):
  `ticket-lint` accepts `engine#` plus four digits without a lookup and
  rejects any other `engine#…` form; a plain id still has to exist here.
  `cargo xtask ticket-lint --check-engine` looks each one up in the local
  checkout's `shuyi-engine/tickets/{open,done}` (its working tree, so a
  ticket written on an engine branch counts) or, without a checkout,
  through `gh api` on the engine's `main`. CI doesn't pass the flag. Tried
  for real on both paths with a temporary `engine#2999`: both report it.
- **The skill** `.claude/skills/engine-change/SKILL.md`: the two tests
  (ADR-0063 §5 and the leak test), now vs. ticket-and-follow-up, the
  checkout and branch, both sides against the patch, the engine's gates,
  the engine PR, moving the pin to the engine's `main` after the merge,
  breaking changes, cross-links, a session of its own, and a game ticket
  blocked by an engine ticket.
- **`work-ticket` §1** has the readiness check for `engine#NNNN` (in the
  engine's `done/` and the pin at or past the commit that closed it, with
  three `gh api` commands; tried on engine ticket 2001: `ahead`). Its §3
  line and `CLAUDE.md` rule 1 point at the skill; the skill is in
  `CLAUDE.md`'s skill list; `tickets/README.md` has the `engine#NNNN`
  form; `docs/engine/README.md` has "Changing the engine from the game";
  `engine-friction` links the skill instead of "1152".
- **The engine side's intake:** [Zafnok/shuyi-engine#7](https://github.com/Zafnok/shuyi-engine/pull/7)
  (merged): `write-ticket`'s "A request from the game", `work-ticket`'s
  Completion notes name the game PR that moves the pin, and `AGENTS.md`
  names `engine#NNNN` and this skill.
- **The no-op walk-through:** engine PR
  [Zafnok/shuyi-engine#9](https://github.com/Zafnok/shuyi-engine/pull/9)
  (the README's map names `CONTRIBUTING.md`; merged), and this ticket's
  game PR, which moves the pin from `5aa0f666` to `8ce963aa` (the
  engine's `main` after #9 and #7). The game built on the checkout
  before the merge (`cargo check --workspace --all-targets`) and passes
  its gates on the new pin.

**What the walk-through changed in the skill:**

1. Writing an engine ticket for a one-line fix is all overhead: a change
   of a few lines with nothing to decide uses the game ticket instead
   (branch `game-NNNN-<slug>`, PR `Game NNNN: …`, as the engine's PRs 6, 7
   and 9 did). Added to the skill and, in #7, to the engine's
   `write-ticket`.
2. `cd shuyi-engine && gh pr create` moves the session's working folder:
   the skill now runs `gh pr create --repo … --head …` from the game's
   folder, and the engine's gates in a subshell (`(cd shuyi-engine &&
   …)`), which also makes Cargo read the engine's `.cargo/`, not the
   game's.
3. #7 was merged with "Squash and merge", not a merge commit. Nothing
   pinned its branch, so no harm, and the skill pins the engine's `main`
   after the merge, which works either way. The engine repository allows
   all three merge buttons; turning off squash and rebase there
   (Settings → General → Pull Requests) would make the rule automatic.
   That is Nick's setting, so it is only a suggestion.

No gameplay rules decided. No follow-up tickets.
