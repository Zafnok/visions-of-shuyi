---
id: "1068"
title: "Playdate: load one chapter's assets at a time, so all eight chapters fit"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: []
nick_input: setup
completed:
---

# 1068 — Playdate: load one chapter's assets at a time

## Context

The Playdate has 16 MB of RAM for the game's code and heap (ADR-0070).
Today the Playdate build loads all of its content at start-up: 1036
left out what the Playdate never uses, but noted "**No lazy loading.**
Dialogue, the biggest file, is checked at load against the battles,
chapters, supports, openings and cinematics, so loading it per scene is
not a small change". With Chapter 1 alone that fits. Act 1 has eight
chapters' maps, battles, scripts, faces and pictures, plus the world
map, supports and camp events, so all of it at once may not.

**Measure first:** the first step finds out how much Act 1's content
would take, and whether lazy loading is needed at all. If everything
fits with a safe margin, the ticket ends with the measurement and a
short ADR note, and nothing else changes.

## Nick input

**Setup** (a measurement, not a check): Nick sideloads the measuring
build (Simulator, *Device → Upload Game to Device*) and pastes the
Simulator console, as for 1036 and 1055.

## Scope

**In:**
- **Measure:** the heap the content takes on the device today
  (Chapter 1), per kind of file (dialogue, maps, battles, pictures,
  sound), and a projection for Act 1: Chapter 1's numbers times the
  chapters, or better, a generated stand-in content set of eight
  chapters' size in `assets-test/` (not real content). Record the
  numbers in the PR and `crates/playdate/README.md`.
- **If it doesn't fit with a margin** (decide the margin from the
  measurement and the stack and frame budgets, and say it in the ADR):
  load what every chapter needs at start-up (data tables, the font,
  the UI) and each chapter's own files (its map, battle, script, faces,
  pictures, music) when the chapter starts, dropping the last chapter's.
  The content checks that run at load (dialogue against battles,
  chapters, supports...) move to build time for the Playdate bundle
  (`cargo xtask playdate` checks the whole set once, as the gates do on
  the PC), so the device never needs everything at once.
- An ADR (`write-adr` skill) for the loading scheme, or a note in
  ADR-0070's follow-ups if nothing changes.
- The loading card (1036) shows between chapters if a load takes long.

**Out (do not do):**
- Any change to what the PC or web builds load.
- A binary content format (a separate ticket if the measurement says
  parsing time, not memory, is the problem).
- Real Act 1 content (the chapter tickets).

## Implementation steps

1. Read ADR-0070, 1036's completion notes, `crates/playdate/README.md`
   and how the content loads (`trpg_content::load_embedded`, the
   engine's registry).
2. Add heap measurement to the Playdate build's log (per file kind).
3. Build the stand-in eight-chapter content set; sideload (Nick) and
   measure.
4. Decide: fits, or lazy loading. Write the ADR either way.
5. If lazy loading: implement it, with a harness test that plays the
   test act's chapters in order and checks no chapter's files outlive
   it; measure again on the device.

## Acceptance criteria

- [ ] The device numbers (today, projected Act 1, and after the change
      if any) are in the PR and the README.
- [ ] The ADR says what was decided and why.
- [ ] If lazy loading was built: the test act plays through in the
      Simulator, and the heap stays under the margin on the device.
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit: which files a chapter needs (from its chapter file).
- Integration: the stand-in act played chapter by chapter; the build-time
  check of the whole set fails on a broken reference.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
