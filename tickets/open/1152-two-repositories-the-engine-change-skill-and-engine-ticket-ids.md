---
id: "1152"
title: "Two repositories: an engine-change skill, engine ticket ids in blocked_by, and the engine side's intake for game requests"
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: high
status: todo
blocked_by: ["1131", "1151"]
nick_input: none
completed:
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
that path a skill a Sonnet session can follow without guessing.

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

- [ ] `cargo xtask ticket-lint` passes with an `engine#NNNN` id in a
      `blocked_by` (unit test with a fixture ticket), and still fails on
      an unknown plain id.
- [ ] `cargo xtask ticket-lint --check-engine` reports an `engine#NNNN`
      that does not exist in the engine repository.
- [ ] `.claude/skills/engine-change/SKILL.md` exists. It covers the
      leak test, fix-now vs ticket-and-follow-up, the local checkout,
      the pin, the cross-links and parallel sessions.
- [ ] The engine repository's intake PR is open or merged, and linked.
- [ ] The no-op walk-through's two PRs are named in the Completion notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `tickets.rs`:
  - `engine#1234` accepted;
  - `engine#12` and `engine#abcd` rejected;
  - an unknown plain id still rejected;
  - the `--check-engine` lookup over a fake directory listing.
- No snapshot tests.

## Completion notes

*(Filled in by the session that completes the ticket.)*
