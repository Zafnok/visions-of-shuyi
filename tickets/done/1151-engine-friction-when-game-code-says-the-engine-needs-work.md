---
id: "1151"
title: "Engine friction: an ADR, a skill and a log for when game code is far bigger than the job"
type: infra
milestone: "Engine: carve"
model: reasoning
effort: high
status: done
blocked_by: ["1118"]
nick_input: none
completed: 2026-10-18
---

# 1151 — Engine friction: when game code says the engine needs work

## Context

ADR-0063 splits the project into Shuyi Engine and the game. Once the
carve is done (1102–1118), every game ticket is also a test of the
engine: if a simple job takes far more game code than it should, the
engine is missing something, or the game is missing a data structure.

Nick, 2026-10-15 (the request behind this ticket):

- "If we find ourselves in the game writing too much code for a
  relatively simple action, consider if this should be an engine feature
  or optimization, or if we need more game-side data structures or
  reorganization."
- Examples he gave: a dialogue box that needs "a lot of hacky things",
  or a scene that takes 700 lines when it "should only take like 30".
- "When we talk about LOC mismatch it should be order of magnitude": if
  we expect 50 lines and it takes 80, that is not worth an engine change;
  3×, 5× or 10× the expected size is.
- "The engine is owned by us, and its primary / launch game is this, so
  we can totally work on it in parallel." Never "we'll wait for the
  engine". It must be easy to decide to iterate on the engine.
- "The engine shouldn't know about game concepts": a data structure made
  for our game in particular does not go into the engine.

This ticket writes that down as a rule sessions follow, so the judgment
is made the same way every time and leaves a trail. It is subjective by
design; the numbers below only decide *when to look*, never what the fix
is.

The ADR-0063 §5 test still decides where code lives: *would a platformer
or a farming game want this unchanged?* → engine. *Would another
grid-tactics game?* → tactics kit. Otherwise → game.

Two-repository mechanics (engine PR, pin bump, `engine#NNNN` ids) are
1152's. This ticket's rule must work in both setups:

- **before 1131**, the engine is in `engine/`, and an engine fix is part
  of the same PR or a ticket in this repository;
- **after 1131**, the engine is in its own repository, and the skill
  points to 1152's `engine-change` skill. Until 1152 is done, write
  "see 1152" in that place.

## Nick input

None. Nick set the rule (quoted above); the rest is technical.

## Scope

**In:**
- **An ADR** (`write-adr` skill, the next free number): "Engine friction:
  game code far bigger than the job triggers a review". It records:
  - **The estimate.** Before writing code, the session writes down how
    many production lines the change *should* take with a good engine,
    and why. The reference is a comparable piece of code: another engine
    or framework doing the same job (Ren'Py's `say` for a dialogue line,
    an egui window, a Godot scene), or a screen here that already does
    something similar (`screens/options.rs`, ADR-0054). One sentence and
    a number.
  - **The measure.** Production lines added and changed in the game's
    crates, from `cargo xtask code-lines` (below). Tests, snapshots,
    data files and docs do not count.
  - **The thresholds** (ratio = measured ÷ estimate):

    | Ratio | What to do |
    | ----- | ---------- |
    | under 2× | nothing |
    | 2× to 3× | one line in the Completion notes ("friction: 2.4×, accepted because …") |
    | 3× or more | a friction review (the skill) before the PR is opened |
    | 10× or more | a friction review, and the fix comes first unless the review says why not |

  - **Signals that trigger a review at any ratio:**
    - a workaround for an engine assumption (a `cfg`, a copy of engine
      code into the game, reaching into an engine type's internals,
      fighting a trait to make it fit);
    - the same boilerplate written a third time across screens (rule of
      three);
    - a performance budget missed because of how the engine does
      something (the Playdate is about 700× slower than a PC; ADR-0070).
  - **The four outcomes of a review**, exactly one per finding:
    1. **Engine feature**: a generic capability is missing. It passes
       the ADR-0063 §5 test.
    2. **Engine ergonomics or optimization**: the capability exists but
       costs too much code or time to use (a missing builder, a missing
       default, a slow path).
    3. **Game-side structure**: the game needs its own data structure,
       helper or reorganization (for example a game-side dialogue box
       builder over the engine's widgets). This is the answer whenever
       the fix would name a game concept.
    4. **Accept**: the job really is that big. Say why, and correct the
       estimate habit if the estimate was the problem.
  - **The leak test for outcomes 1 and 2.** The engine change must not
    name a class, unit, faction, Combat Art, chapter, character, story
    beat, or any id from the game's data. If the only shape that helps
    is game-shaped, it is outcome 3. If a generic shape exists under the
    game-shaped one (a "dialogue box" is a game idea; "a text panel with
    a portrait slot and paged text" is generic), the engine gets the
    generic shape and the game wraps it.
  - **Never wait on the engine.** A friction finding never blocks game
    work. Exactly one of:
    - **Fix first**, when the engine fix is one PR's worth or less and
      cuts the game change sharply: do the engine change first in the
      same session (before 1131: same PR; after: 1152's flow), then
      finish the game ticket on it.
    - **Ship and follow up**, otherwise: the game ticket lands with the
      workaround marked by a comment `// engine-friction: <ticket id>`,
      plus two tickets. One is the engine or game-structure ticket. The
      other is a game ticket that deletes the workaround, blocked by the
      first.

    The engine and the game are both ours: an engine ticket is normal
    work, picked up like any other, not a request to someone else.
- **The skill** `.claude/skills/engine-friction/SKILL.md`: the steps of
  a review, written for an implementation session.
  1. Write the estimate.
  2. Run `cargo xtask code-lines`.
  3. Check the signals.
  4. If triggered, name the job in one sentence, then find where the
     extra code went: which lines exist only to get around the engine,
     and which are the game's real logic.
  5. Pick one outcome per finding and apply the leak test.
  6. Decide fix-first or ship-and-follow-up, and write the tickets
     (`write-ticket`).
  7. Add a row to the log.

  Include two worked examples, written from the ADR (not from real
  code): a dialogue box that needed hacks (outcome 1 for a generic text
  panel plus outcome 3 for a game-side builder), and a scene at 700
  lines against an estimate of 30 (outcome 2 for a scene builder, about
  23×).
- **The log** `docs/engine/friction.md`: one table, one row per review
  or 2×–3× note. Columns: date, ticket, the job, estimate, measured,
  ratio, outcome, follow-up ticket(s). It is how a later sweep (1153)
  sees three small notes in the same area: three or more rows on the
  same engine area with outcome "accept" also trigger a review.
- **`cargo xtask code-lines [--base <ref>]`** (default base
  `origin/main`): counts production Rust lines added and removed per
  crate between the base and the working tree. It skips files under
  `tests/`, `#[cfg(test)]` modules (from the line of the attribute to the
  module's closing brace), blank lines and comment-only lines. Snapshot,
  `.ron`, `.dlg` and Markdown files do not count. It prints one row per
  crate and a total. Order of magnitude is the goal: brace counting is
  enough, no parser needed.
- **Hook it into the workflow:**
  - `work-ticket`:
    - in **3. Implement**, before the first code: write the estimate;
    - in **4. Verify**: run `code-lines` and follow the thresholds;
    - in the PR body template: one line "Engine friction: estimate N,
      measured M (R×), <outcome or 'under 2×'>".
  - `write-ticket`: a ticket that adds a screen or a scene states the
    expected size in its Implementation steps, so the estimate starts
    from the ticket.
  - `CLAUDE.md`: a short rule 9, "**Code far bigger than the job is a
    signal**", with one sentence and a link to the ADR and the skill.
  - The skill table in the map of the repository: add `engine-friction`.

**Out (do not do):**
- the two-repository workflow, the pin and `engine#NNNN` ids (1152);
- the first sweep of the existing game code (1153);
- any engine or game refactor found while writing this. Write it as a
  ticket.
- a CI gate on the ratio. The rule is a judgment, and a gate would be
  gamed or would block honest big jobs. `code-lines` informs; it never
  fails a build.

## Implementation steps

1. Write the ADR (`write-adr`). Add it to `docs/adr/README.md`. Link it
   from ADR-0063's Related line.
2. Write `crates/xtask/src/code_lines.rs` with a pure function
   `count(diff: &str) -> Vec<CrateLines>` over `git diff --unified=0`
   output plus the files' text for the `#[cfg(test)]` ranges. Wire it in
   `main.rs` with a `USAGE` string like the other commands. Unit-test it.
3. Write `.claude/skills/engine-friction/SKILL.md` with the steps and the
   two worked examples.
4. Create `docs/engine/friction.md` with the table header and a one-line
   explanation. Link it from `docs/engine/README.md`.
5. Edit `work-ticket`, `write-ticket` and `CLAUDE.md` as listed in
   Scope.
6. Run `cargo xtask ticket-lint` and the gates.

## Acceptance criteria

- [x] The ADR exists, is in the index, and states the estimate, the
      measure, the threshold table, the signals, the four outcomes, the
      leak test and the never-wait rule.
- [x] `cargo xtask code-lines --base HEAD~1` on this ticket's own branch
      prints per-crate rows. Its count for `crates/xtask` leaves out
      `code_lines.rs`'s test module (unit test).
- [x] `.claude/skills/engine-friction/SKILL.md` exists with frontmatter
      (`name`, `description` saying when to use it), the steps and both
      worked examples.
- [x] `work-ticket` asks for the estimate before code and the
      measurement in Verify. The PR body template has the friction line.
- [x] `CLAUDE.md` has the rule and lists the skill.
- [x] `docs/engine/friction.md` exists. This ticket's own PR adds its
      first row (estimate vs measured for `code_lines.rs`).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `code_lines::count`:
  - a test module is skipped;
  - a `tests/` file is skipped;
  - blank and comment lines are skipped;
  - lines are grouped per crate;
  - a removed line counts as removed.
- No snapshot or integration tests (docs and a dev tool).

## Completion notes

- **ADR-0092** (*Engine friction: game code far bigger than the job
  triggers a review*), in the index and on ADR-0063's Related line. It
  holds the estimate, the measure, the thresholds, the signals, the four
  outcomes, the leak test, never-wait (fix first, or ship and follow up)
  and the trail (log, PR line, skill).
- **`cargo xtask code-lines [--base <ref>]`** (`crates/xtask/src/code_lines.rs`).
  It diffs the base's merge base with `HEAD` against the working tree
  and counts untracked files too, so a new file counts before its first
  commit. A file's `#[cfg(test)]` ranges are read from its text on each
  side (`git show base:path` for removed lines). It prints one row per
  crate, `total (game)` and, when `engine/` changed, `total (engine)`.
  `--base 4b825dc…` (git's empty tree) counts the whole tree, which 1153
  can use: today that is about 49,500 game lines and 27,000 engine lines.
- **The skill** `.claude/skills/engine-friction/SKILL.md` has the seven
  steps and both worked examples. In the after-1131 places it says "see
  1152".
- **The log** `docs/engine/friction.md`, linked from `docs/engine/README.md`.
  Its first row is this ticket's own estimate against the measurement.
- **Hooks**: `work-ticket` (the estimate in Implement, the measurement
  and thresholds in Verify, the `Engine friction:` line in the PR body),
  `write-ticket` (the expected size of a screen or scene goes in the
  Implementation steps), `CLAUDE.md` rule 9 and the skill table.

Choices the ticket left open:
- Besides `tests/`, the count also leaves out `tests.rs` and `*_tests.rs`
  files. The repository has about 150 of them, each declared
  `#[cfg(test)] mod tests;` by its parent, so they are test code.
- A `#[cfg(test)]` *item* of any kind is skipped, not only a module
  (a test helper `fn` under the attribute too).
- Three or more "accept" rows on one engine area are listed among the
  ADR's signals, as the ticket's log paragraph says.
- In `work-ticket`, a ticket with no Rust code skips the estimate. Its
  PR line says "none: no Rust code".

Engine friction (this ticket's own): estimate 180 (a diff parser, a
brace-counting test-range scanner, a crate key and a table, comparable
to the git-driven `clean_targets.rs` command); measured 312 in
`crates/xtask`; 1.7×, under 2×.

Gates: fmt, clippy, the whole workspace's tests (`--no-fail-fast`, 0
failures), check-keys, check-text, check-engine-boundary, `cargo doc -p
xtask`, deny, machete and typos pass. The WASM and Playdate builds were
not run: this ticket changes no crate they build (`xtask` only).

No gameplay rules were decided. Nothing for Nick to play.
