---
id: "0159"
title: The story documents move to the private assets repository
type: infra
milestone: M0 Foundation
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["0156"]
nick_input: none
completed:
---

# 0159 — The story documents move to the private assets repository

## Context

Nick picked B+ in 0153 (`docs/research/0153-keeping-the-story-private.md`
§9): the story and the game's data move into the private assets
repository. This ticket moves the **documents**: `docs/story/` and the
story sections of `docs/design/`. 0160 moves the game's content (the
`.dlg` scripts with it). The documents are a separate step because no
build embeds them; only three tests read them.

What moves (0153 §3.1, counted 2026-11-10):

- `docs/story/`: 39 files, 11,528 lines (beats, bible, outline,
  characters, chapter beat sheets, ledger, names registry, craft notes,
  the writer's guide);
- the story sections of `docs/design/`, found by 0153's verifiers. Each
  line range is read again before moving:
  - `setting-and-tone.md` (all);
  - `title.md` :52-61 "Who Shuyi is in the story";
  - `world-structure.md` :166-181 and :264-276;
  - `audio.md`'s cue notes (:80-91, :187, :197);
  - `battle-scenes-and-recruitment.md` :85-90;
  - `voices-languages-and-script.md` :222-243;
  - `death-and-difficulty.md` :128-142;
  - `turn-structure.md` :367-481 (each chapter's battle rule);
  - `look-and-feel.md` :1035-1044;
  - `combat-arts.md` :542-543;
  - `docs/design/README.md` :13;
  - `docs/design/playdate/` (90 files), which 0153 didn't read for story:
    check it.

  `chapter-1.md` (279 lines) moves too: under B+ Chapter 1's content
  is private as well.
- `assets-src/audio/voice-audition.html`, the page Nick chose the voices
  on (0043): it plays real Chapter 1 lines, so it is a story document too
  (found by 0157's `cargo xtask check-leaks`, which skips it until this
  ticket moves it; drop it from the check's `MOVING` list here). Its row
  in `assets-src/audio/README.md` and the link in
  `docs/design/voices-languages-and-script.md` point at the new place.

Where they go: the assets repository's `story/` (made by 0156), with
`story/design/` for the sections. A design doc that loses a section keeps
a one-line pointer: "The story part of this decision is in the private
assets repository, `story/design/<file>`."

What reads them, and must follow:

- `crates/content/src/names/tests.rs:19`:
  `include_str!("../../../../docs/story/names.md")`, a compile error once
  the file moves;
- `crates/xtask/src/check_script.rs:341`: a test reading
  `docs/story/writers-guide.md`;
- `crates/content/src/audio.rs:240`: a test parsing `audio.md`'s
  "Music cues" table. Only the descriptions that name plot move; the rows
  the test needs stay;
- `.github/workflows/changes.yml:60,62`;
- about 117 lines in 75 files that name `docs/story` in comments, docs and
  skills (0153 B1): `story-writing` (9), `work-ticket`, `AGENTS.md`,
  `README.md`, `_typos.toml`, data comments.

These read the private copy through 0156's layer and follow its rule
without it, or move into a keyed CI job.

## Nick input

None.

## Scope

**In:**
- The documents above into the assets repository, through the
  `content-change` skill (0156): an assets PR, then this repository's PR
  that deletes them here and moves `assets-private.rev`.
- The three tests and `changes.yml`.
- Pointers: every line naming `docs/story/` (outside `tickets/` and
  `docs/research/`) names the new place, `assets-private/story/`.
- `story-writing` skill: where the story lives, and that changes go
  through `content-change`.
- `AGENTS.md`: rule 3's "`docs/design/` … and `docs/story/beats.md`" and
  the map of the repo; `docs/story/README.md`'s pointer stays as a stub
  at `docs/story/README.md` saying where the story is and how to get it
  (`cargo shuyi private-assets`).
- 0153's check (0157), or a grep, shows no story section left in
  `docs/design/`; once 0157 is done (either order works), `cargo xtask
  check-leaks` passes with
  `docs/story/` and the audition page dropped from its `MOVING` list.

**Out (do not do):**
- The `.dlg` scripts and the game's data (0160).
- Rewriting history (0153 §3.5: not recommended).
- Tickets, ADRs and `docs/research/` that describe plot: they stay.
- Changing any document's words beyond the pointers (the writing pause,
  `AGENTS.md` rule 3).

## Implementation steps

1. Write the size estimate first (rule 10). Deletions count: about 12,000
   lines deleted and about 400 changed lines to review (pointers 200,
   tests 60, skills and `AGENTS.md` 60, this ticket 250). It passes the
   3,000-line warning by deletion; say so in the PR (ADR-0101).
2. Read each design section again; list the exact ranges.
3. The assets PR (`story/`, `story/design/`), merged.
4. Here: delete, add the pointers and the stub, fix the tests and
   `changes.yml`, move the pin.

## Acceptance criteria

- [ ] `docs/story/` holds only its stub README; the assets repository's
      `story/` holds the 39 files and `story/design/` the sections.
- [ ] `git grep -n "docs/story/" -- . ':!tickets/' ':!docs/research/'`
      finds only the stub's own lines.
- [ ] The names-registry, writer's-guide and audio-cue tests pass in CI
      on the private copy.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The three moved tests, unchanged in what they check.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
