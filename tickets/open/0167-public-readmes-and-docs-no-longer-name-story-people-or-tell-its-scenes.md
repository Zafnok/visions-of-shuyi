---
id: "0167"
title: Public READMEs and docs no longer name the story's people or tell its scenes
type: infra
milestone: M0 Foundation
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["0160"]
nick_input: none
completed:
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

- [ ] The grep of step 2 finds no display name of the story outside the
      out-of-scope folders (the completion notes list what is left and
      why).
- [ ] No public README or doc in scope tells who dies, joins or leaves.
- [ ] `cargo xtask check-leaks` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None new; the existing suite (a README a test reads keeps passing).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
