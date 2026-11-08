---
id: "1258"
title: "Three follow-ups left in prose: the options doc, the side-quest validator and the Harness's storage"
type: bug
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1258 — Three follow-ups left in prose: the options doc, the side-quest validator and the Harness's storage

## Context

The retro of 2026-11-08 found three things that PRs noted and left with
no ticket, against `AGENTS.md`'s no-unticketed-work rule.

1. **`docs/design/options.md` contradicts the game.** 0826's completion
   notes (#542) say the doc "still says the log's row 'isn't shown'
   until the game has voices; this ticket's step 3 says the rows stay,
   so they do". The doc's *Voices in the dialogue log* bullet (around
   line 110) still says "until the game has voices there is nothing for
   it to turn off, so it isn't shown".
2. **The validator doesn't know a side quest's scenes can count on their
   character.** 1072's notes (#545) say Tamsin's and Aske's lines before
   and after their own battle sit in `@if` blocks like everyone else's.
   "A small content change could teach it (`Cast` reading the world
   maps' side quests); not done here, since the dialogue is checked
   before the world maps load." The next side quests (1083, 1084) will
   write the same `@if` blocks.
3. **"The engine's `Harness::start` takes no storage."** 1245's PR #569
   noted it: "Small; no ticket." If a test can't start a Harness with a
   given storage, every save-related test works around it. The engine's
   `Harness::into_storage` exists "to start the next launch with it"
   (`crates/platform/src/harness.rs`), so the way in may exist and be
   hard to find.

## Nick input

None.

## Scope

**In:**
- Fix the options doc's sentence to say what the game does, which 0826's
  step 3 decided.
- Teach the dialogue validator a side quest's own character: `Cast`
  reads the world maps' side quests before the dialogue check, or the
  check takes them as an input. Then drop the `@if` blocks the fix makes
  unneeded in Tamsin's and Aske's scenes.
- Check the engine for a way to start a `Harness` with a storage
  (`Start` and `Setup` hold one in `crates/platform/src/app.rs`). If there
  is one, write it into the `shuyi-test` skill's notes for the game. If
  not, file the engine ticket (the `engine-change` skill, "Ticket and
  follow-up") and add its friction row.

**Out (do not do):**
- New options or new side quests.

## Implementation steps

1. Edit `docs/design/options.md`.
2. Find where `Cast` is built for the dialogue check (`grep -rn "Cast"
   crates/content/src/dialogue`) and add the side quests' characters.
   Test with Tamsin's scene.
3. Read `crates/platform/src/harness.rs` and `app.rs` at the pinned
   revision for a storage parameter, then act on the result as above.

## Acceptance criteria

- [ ] The options doc's bullet matches the game (`grep -n "isn't shown"
      docs/design/options.md` finds nothing).
- [ ] A side quest's scene that names its own character without `@if`
      passes the check (a test), and Tamsin's and Aske's scenes are
      simplified.
- [ ] The Harness's storage question is answered in the notes, with a
      skill line or an engine ticket.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the validator with a side quest's own character.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
