---
id: "1197"
title: The pin review asks what each engine PR gives the game, not only what it replaces
type: infra
milestone: "Engine: carve"
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1197 — The pin review asks what each engine PR gives the game

## Context

Nick (2026-10-28): the pin moves "haven't really been ambitious" about
using the engine's new features. An audit of all seven sections in
`docs/engine/pins.md` (game PR #465) agrees. Three things make the
review miss them:

1. **The watcher puts a PR in one section only** (`classify` in
   `crates/engine-watch/src/lib.rs`: Breaking, then Unblocks, then New
   for games, the first that fits). A PR that breaks the API and also
   adds a feature only gets a *Breaking* row, and the skill says to fix
   those "with behaviour unchanged". So 13 of the ledger's 18 *Breaking*
   rows are `fixed: nothing to change`, which is a compile check rather
   than a review. Features missed this way: #152's stamped records
   (caught late, 1195), #139's `NOTICES.txt` (caught, 0909), #105's
   playtest history and #117/#119's live link and perf traces.
2. **`api: none` PRs are never read.** A PR labeled `api: none`, or
   with no `pub` line in its diff, goes to *Nothing for the game*,
   which is collapsed and has no row in the ledger. But behavior and
   test suites change there too. #153 added `power_cut_suite` for every
   storage backend, and the game's Playdate saves can lose a save to a
   power cut (1196). Its label said `api: none`.
3. **The skill's adoption test only looks for replacements.** Step 4
   asks "Does the game already have code doing this job? Does an open
   ticket plan to?". When the answer is no, the default is `not for the
   game`, even when the game would gain from the feature, because there
   is nothing to replace. Its examples ("the game has no modules")
   teach dismissing. Decisions like `not for the game: tooling it gets
   at the pin` treat "available" as "used".

## Nick input

None.

## Scope

**In:**
- `crates/engine-watch`: a PR that breaks the API and also adds public
  items appears under *Breaking* and under *New for games*. Its
  *Nothing for the game* rows show each PR's labels and are listed
  open, not collapsed.
- `.claude/skills/engine-pin/SKILL.md` step 4 asks about gains first.
- `cargo xtask engine-pin --check` (`crates/engine-watch/src/ledger.rs`):
  a *Breaking* row whose PR is also *New for games* needs an adoption
  decision as well as its `fixed:`.

**Out (do not do):**
- Going back over the old ledger sections (the audit in #465 did that:
  1195, 1196).
- Changing how the engine labels its PRs (an engine ticket, if needed:
  `engine-change`).

## Implementation steps

1. `classify`: return the kinds that apply. A PR with `Breaking:` lines
   and added public items (`declares_public` on a `+pub` line, or the
   `api: added` label) is both `Breaking` and `NewForGames`. `render.rs`
   lists it in both tables. Update `tests.rs`.
2. `render.rs`: *Nothing for the game* becomes a plain table with a
   Labels column, not `<details>`.
3. `ledger.rs`: a row may hold `fixed: …; adopt → NNNN` or
   `fixed: …; not for the game: …`. `--check` requires the second part
   for a PR the report lists in both sections. Update the header of
   `docs/engine/pins.md` to say so.
4. `SKILL.md` step 4 is rewritten as a review of gains:
   - For every *New for games* row, and every *Breaking* row that
     adds something: **what does this let the game do, check or delete?**
     Name the game's files or system that would use it (saves, the
     Playdate build, the bots and playtest, tools and CI, screens).
   - Then the two existing questions (existing code, open tickets)
     decide between a new ticket and changing an open one.
   - `not for the game` needs a reason the game *can't* use it (a
     part of the engine it doesn't use, like modules or the short path;
     a platform it doesn't ship on). "Nothing to replace" and "tooling
     it gets at the pin" are not reasons. When it's unclear, write the
     ticket: a ticket costs little and a missed gain costs a lot.
   - Read each *Nothing for the game* title. A PR that changes behavior
     or adds a suite or check in a crate the game links (storage, saves,
     the renderer, input, the tools) gets a row with a decision even
     when it's labeled `api: none`.
   - Examples: adopt with nothing to replace (#152: the bots' records
     get stamps, 1195; #153: the Playdate's storage runs the power-cut
     suite, 1196) next to a real `not for the game` (#95: the game has
     no modules).
5. Keep the pin PR mechanical as before. Adoption is still a ticket.
   Only the review gets wider.

Expected size: about 60 production lines in `crates/engine-watch`, and
the skill's step 4 rewritten.

## Acceptance criteria

- [ ] A test in `crates/engine-watch/src/tests.rs`: a PR with a
      `Breaking:` line and a `+pub fn` is in both sections of the report.
- [ ] `engine-pin --check` fails on a ledger whose row for such a PR has
      only `fixed:`, and passes with `fixed: …; adopt → NNNN`.
- [ ] The skill's step 4 asks about gains first and has the examples
      above.
- [ ] `cargo shuyi check-skills .claude/skills` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `classify`, `render` and the ledger check, in
  `crates/engine-watch/src/tests.rs` and `ledger.rs`'s tests.

## Completion notes

*(Filled in by the session that completes the ticket.)*
