# ADR-0092: Engine friction: game code far bigger than the job triggers a review

- **Status:** Accepted
- **Date:** 2026-10-18
- **Related tickets:** 1151 (this rule), 1152 (two repositories: the
  `engine-change` skill), 1153 (the first sweep of the game)
- **Extends:** ADR-0063 (an engine and a game; its §5 decides where code
  lives)

## Context

ADR-0063 splits the project into Shuyi Engine and the game. With the
carve done (1102–1118), every game ticket is also a test of the engine:
when a simple job takes far more game code than it should, the engine is
missing something, or the game is missing a data structure of its own.

Nick, 2026-10-15:

- "If we find ourselves in the game writing too much code for a
  relatively simple action, consider if this should be an engine feature
  or optimization, or if we need more game-side data structures or
  reorganization." His examples: a dialogue box that needs "a lot of hacky
  things", a scene that takes 700 lines when it "should only take like
  30".
- "When we talk about LOC mismatch it should be order of magnitude": 80
  lines where 50 were expected is not worth an engine change; 3×, 5× or
  10× is.
- "The engine is owned by us, and its primary / launch game is this, so
  we can totally work on it in parallel." Never "we'll wait for the
  engine".
- "The engine shouldn't know about game concepts."

Without a written rule, each session judges "too much code" its own way,
and nothing is left behind for the next one to see a pattern in. The
judgment is subjective by design; this ADR fixes *when to look* and *how
to decide*, never what the fix is.

## Decision

### 1. The estimate, before the first code

Before writing code, the session writes down how many production lines
the change *should* take with a good engine, and why. The reference is a
comparable piece of code: another engine or framework doing the same job
(Ren'Py's `say` statement for a dialogue line, an egui window, a Godot
scene), or a screen here that already does something similar (the
engine's `screens/options.rs`, ADR-0054). One sentence and a number, for
example: "a yes/no pop-up is one view and one skin, a fraction of the
options screen: 40 lines".

A ticket that adds a screen or a scene states the expected size in its
Implementation steps (`write-ticket`), so the estimate starts from the
ticket.

### 2. The measure

`cargo xtask code-lines [--base <ref>]` (default base `origin/main`, from
its merge base with `HEAD`) counts production Rust lines added and removed
per crate up to the working tree, untracked files included. It leaves out
files under `tests/`, `tests.rs` and `*_tests.rs` modules, `#[cfg(test)]`
items (from the attribute to the item's closing brace), blank lines and
comment-only lines; snapshots, `.ron`, `.dlg`, Markdown and other
non-Rust files do not count. Brace counting is enough: the goal is order
of magnitude, not a parser.

The measured size is the **added** column of the game's crates (`total
(game)`): a changed line shows as one removed and one added line, so
"added" is lines added plus lines changed. Engine crates (`engine/`,
before 1131) get their own total, for a fix made in the same PR.

`code-lines` informs; it never fails a build. There is no CI gate on the
ratio.

### 3. The thresholds

Ratio = measured ÷ estimate.

| Ratio | What to do |
| ----- | ---------- |
| under 2× | nothing |
| 2× to 3× | one line in the ticket's Completion notes ("friction: 2.4×, accepted because …") and a row in the log |
| 3× or more | a friction review (the `engine-friction` skill) before the PR is opened |
| 10× or more | a friction review, and the fix comes first unless the review says why not |

### 4. Signals that trigger a review at any ratio

- A workaround for an engine assumption: a `cfg`, a copy of engine code
  into the game, reaching into an engine type's internals, fighting a
  trait to make it fit.
- The same boilerplate written a third time across screens (rule of
  three).
- A performance budget missed because of how the engine does something
  (the Playdate is about 700× slower than a PC; ADR-0070).
- Three or more rows in the log on the same engine area with outcome
  "accept" (the small notes add up).

### 5. The four outcomes of a review

A review names the job in one sentence, finds where the extra code went
(which lines exist only to get around the engine, which are the game's
real logic), and gives each finding exactly one outcome:

1. **Engine feature**: a generic capability is missing. It passes the
   ADR-0063 §5 test (*would a platformer or a farming game want this
   unchanged?* → engine; *another grid-tactics game?* → tactics kit).
2. **Engine ergonomics or optimization**: the capability exists but costs
   too much code or time to use (a missing builder, a missing default, a
   slow path).
3. **Game-side structure**: the game needs its own data structure, helper
   or reorganization (for example a game-side dialogue box builder over
   the engine's widgets). This is the answer whenever the fix would name
   a game concept.
4. **Accept**: the job really is that big. Say why, and correct the
   estimate habit if the estimate was the problem.

### 6. The leak test, for outcomes 1 and 2

The engine change must not name a class, unit, faction, Combat Art,
chapter, character, story beat, or any id from the game's data. If the
only shape that helps is game-shaped, it is outcome 3. If a generic shape
exists under the game-shaped one (a "dialogue box" is a game idea; "a text
panel with a portrait slot and paged text" is generic), the engine gets
the generic shape and the game wraps it.

### 7. Never wait on the engine

A friction finding never blocks game work. Exactly one of:

- **Fix first**, when the engine fix is one PR's worth or less and cuts
  the game change sharply: do the engine change first in the same
  session, then finish the game ticket on it. Before 1131 the engine is in
  `engine/`, and the fix goes in the same PR (or its own ticket in this
  repository, done first). After 1131 the engine has its own repository,
  and the fix follows 1152's `engine-change` skill.
- **Ship and follow up**, otherwise: the game ticket lands with the
  workaround marked by a comment `// engine-friction: <ticket id>`, plus
  two tickets: the engine or game-structure ticket, and a game ticket that
  deletes the workaround, blocked by the first.

The engine and the game are both ours: an engine ticket is normal work,
picked up like any other, not a request to someone else.

### 8. The trail

- **The log**, `docs/engine/friction.md`: one row per review or 2×–3×
  note (date, ticket, the job, estimate, measured, ratio, outcome,
  follow-up tickets). It is how a later sweep (1153) sees three small
  notes in the same area.
- **The PR body** carries one line: "Engine friction: estimate N,
  measured M (R×), <outcome or 'under 2×'>" (`work-ticket`).
- **The skill**, `.claude/skills/engine-friction/SKILL.md`, has the
  steps of a review and two worked examples.

*Note, 2026-11-01 (ticket 1187):* the estimate no longer "starts from the
ticket" (§1): a ticket's size is a starting point to check against the
reference, because it inherits the ticket's split. Non-Rust code is
counted by hand beside `code-lines` (§2), and the PR line (§8) also
answers the signals. The skill has the steps.

## Consequences

- Every ticket costs one sentence before the code and one command after
  it. In exchange, "this took far too much code" is noticed when it
  happens, decided the same way each time, and leaves a row to find.
- The engine grows from real friction in the launch game, not from
  guesses, and the leak test keeps game ideas out of it.
- Estimates will be wrong at first. A bad estimate shows up as outcome
  4 ("accept"), and the review says what to estimate differently, so the
  habit corrects itself.
- Line counts are crude: brace counting can be fooled by a raw string
  with an unbalanced brace, and a line of dense code counts the same as a
  line of glue. That is fine at the order of magnitude this rule works
  at, and the review is a judgment, not the number.
- Shipping with a marked workaround leaves debt, but it is ticketed twice
  (the fix and the cleanup), so it cannot be forgotten.

## Alternatives considered

- **A CI gate on the ratio** — the estimate is the session's own number,
  so a gate would be gamed by padding it, or would block honest big jobs.
  The rule is a judgment; the tool only informs.
- **A fixed size budget per screen or file** (e.g. 300 lines) — the
  right size depends on the job; a budget catches big honest screens and
  misses a 30-line job done in 200.
- **Only reviewing during periodic sweeps** (1153) — friction is
  cheapest to fix while the code is being written, and sweeps alone would
  find it weeks later. Sweeps stay, for what tickets miss.
- **Counting with an existing tool** (`tokei`, `cloc`) — they count
  files, not a diff against a base, and don't know `#[cfg(test)]`
  modules; a small xtask command does exactly the measure above with no
  new dependency.
- **Waiting for the engine when it lacks something** — rejected by Nick:
  the engine is ours and moves in parallel; the game never stops for it.
