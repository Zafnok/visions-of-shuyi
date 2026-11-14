---
id: "0167"
title: Public READMEs and docs no longer name the story's people or tell its scenes
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0160"]
nick_input: none
completed: 2026-11-13
---

# 0167 — Public READMEs and docs no longer name the story's people or tell its scenes

## Context

0160 moved the game's content into the private repository's `content/`
(ADR-0117, ADR-0121), and 0166 cleared the code's comments of the
story's display names and plot. ADR-0118 §2 says a public file may name
a content id but never describe the plot; design documents, ADRs and
tickets are outside that rule.

Files that are inside it still name the story's people and places by
display name and tell what happens in scenes. 0160 moved the two
catalogue tables of real files (the places in `assets/maps/README.md`,
the openings in `assets/openings/README.md`) to the private READMEs, and
left the rest here. A grep of the display names in the private
`content/data/names.ron` (people and places) finds hits in, among others:

- `assets/dialogue/README.md` (its examples name the cast; its
  `@if` examples are read by `readme_if_examples_are_valid`, guarded
  since 0160), `assets/battles/README.md`, `assets/cinematics/README.md`,
  `assets/battlers/README.md`, `assets/portraits/README.md`;
- `crates/ui/README.md`, `docs/playtesting.md` (about 45 lines),
  `docs/voice.md`, `docs/session-records.md`, `docs/engine/*.md`,
  `docs/targets.md`;
- `docs/ROADMAP.md` (about 20 lines).

## Nick input

None.

## Scope

**In:**
- Every file above, and any other tracked file outside `tickets/`,
  `docs/adr/`, `docs/design/` and `docs/research/` that the grep finds:
  display names reworded to ids or roles (`the retainer`, `ch03`), and
  any sentence that tells who dies, joins, leaves or what a scene
  reveals reworded to what the file needs (a format, a tool, a test).
- A format README's example that names the cast uses made-up or test
  ids (`test_knight`, `test_voice`) where nothing reads it; where a test
  reads it (`assets/dialogue/README.md`'s `@if` examples), keep the
  test's guard or move it to the test cast with the example.
- `docs/ROADMAP.md`: chapter and ticket ids, not plot.

**Out (do not do):**
- Tickets, ADRs, design documents, research notes (ADR-0118 §2).
- The engine's skill pack (`.claude/skills/shuyi-*`, never edited here).
- Rewriting history.

## Implementation steps

1. List the display names: every value of the private
   `assets-private/content/data/names.ron` that is a person or a place
   (not `Ellery`, `Veyne` or `Vix`, which are public: 0166's notes).
2. `git grep -nw` them over the tracked files, less the out-of-scope
   folders; list each hit, marked "plot" or "name only", in the
   completion notes.
3. Reword each; keep each file saying what it needs to.
4. Run `cargo xtask check-leaks` and the content tests.

Expected size: no production code; about 300–500 changed lines of docs.

## Acceptance criteria

- [x] The grep of step 2 finds no display name of the story outside the
      out-of-scope folders (the completion notes list what is left and
      why).
- [x] No public README or doc in scope tells who dies, joins or leaves.
- [x] `cargo xtask check-leaks` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None new; the existing suite (a README a test reads keeps passing).

## Completion notes

- Step 1: the person and place values of the private `names.ron`
  (91 words once `Ellery`, `Veyne`, `Vix`, the generic minor names
  such as `Messenger` and the factions, gods and terms were set aside).
  Step 2: `git grep -nwF` over the tracked files, less `tickets/`,
  `docs/adr/`, `docs/design/`, `docs/research/` and the engine's
  `shuyi-*` skills: 98 text hits (the rest were binary `.ogg`/`.flac`
  matches, noise). A second grep for plot words (dies, joins, betray,
  confession…) found one more line (`docs/ROADMAP.md`'s "runaway Candle").
- Plot (who dies, joins or leaves, or what a scene holds), reworded to
  what the file needs: `docs/playtesting.md` (the defector "joins after
  Chapter 2", the shieldbearer arriving for Chapter 5), `docs/ROADMAP.md`
  (0069's "Dace fleeing … Hollis's death", 1071 "with Gil", 1009's
  runaway recruit), `assets/dialogue/README.md` (`@dies`/`@joins`'s
  examples named the cast; the `@else` example's knowledge rule told a
  Chapter 7 scene; the `@if` example's `@else` line hinted a death),
  `assets-src/audio/import.py` (the retainer's confession).
- Name only, reworded to ids or roles (`the retainer`, `ch04`,
  `harrowby_yard`): the rest of `docs/playtesting.md` (report samples and
  army lists now print role ids), `docs/ROADMAP.md`, the dialogue README's
  name tables, `.claude/skills/story-writing/SKILL.md`,
  `THIRD_PARTY_ASSETS.md`, `_typos.toml`, `assets-src/tilesets/tiny_tales.ron`,
  `assets/battlers/README.md`, `assets/data/{credits,lighting,names,terrain}.ron`
  (comments), `assets/openings/README.md`, `crates/content/demo/bundle.txt`,
  `crates/content/playdate-music.txt`, `docs/voice.md` (its `say.ron`
  example now respells the public `guide`).
- `readme_names_example_is_valid` (guarded since 0160) no longer checks
  that the README writes the example's output with real names, since the
  README now says what it shows without them; it still checks the
  example against the real table, token for token.
- Left on purpose: `Tor's Bow` (`item/tests.rs`, an item name in a
  guarded test of the real data) and `Haltet das Tor` (German, not a
  name), as in 0166; the lowercase `aske`/`ned` entries of `_typos.toml`
  (a case-sensitive whole-word grep doesn't match them, and typos needs
  them for the tickets and design documents); the map ids
  `harrowby_yard` and `veyne_study` (content ids, ADR-0118 §2).
- The ticket's list named `assets/battles/`, `cinematics/`, `portraits/`,
  `crates/ui/README.md`, `docs/session-records.md`, `docs/engine/` and
  `docs/targets.md`: 0160 and 0166 had already cleared them; the grep
  found no person or place there.
- No follow-up tickets.
