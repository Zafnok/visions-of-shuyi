---
id: "1065"
title: "The act ends: the closing scene, \"Act 1 ends\", and the army carried over"
type: feature
milestone: Post–Chapter 1
model: sonnet-5
effort: medium
status: todo
blocked_by: ["1007"]
nick_input: none
completed:
---

# 1065 — The act ends

## Context

Act 1 ends with Chapter 8, "The Last Watch" (`docs/story/outline.md`):
after the story battle at Saltmere Docks and Hollis's death, a
**closing scene** plays (Varenhall, the king reading to someone the
camera never shows), then "*Act 1 ends.*" Act 2 isn't written yet.

Today a chapter with `next: None` shows "To be continued" and returns
to the title (`crates/ui/src/flow.rs`, `Stage::ToBeContinued`,
`assets/chapters/README.md`). 1007 builds the act change on the world
map ("when an act's final story battle is won, the next act's world map
replaces the current one"), but with no Act 2 there is nothing to load.

`world-structure.md` (*Continents and acts*): "*Claude's default, Nick
may veto:* the whole army, stock, gold and support progress carry over
to the next act, like one continuous campaign." So the player's Act 1
army must survive the end of the act in a save Act 2 can start from.

## Nick input

None. No check after merge (Nick, 2026-10-21: he checks only design and
the full Act 1 playtest, 1064); the end of the act is on 1064's
checklist.

## Scope

**In:**
- A chapter (or the act's world-map data, wherever 1007 put the act's
  final story battle) can name **closing scenes** that play after its
  victory scenes and the save prompt: Chapter 8's closing scene is
  written by 0750; until then the test data uses a stand-in scene from
  `assets-test/`.
- After them, an **"Act 1 ends"** card: today's "To be continued"
  screen with the act's words instead (the text from the language pack,
  `{n:...}` tokens as usual), then the title. The look stays the "To be
  continued" screen's; no new art.
- **The army carried over:** the save written at the end of the act (the
  0802 "Save your progress?" prompt after the final victory) records
  that Act 1 is complete, with the whole army, stock, gold and support
  points (the `Campaign` already holds them; this adds the act-complete
  mark and keeps it through save and load). `Load Game` on such a save
  shows the "Act 1 ends" card and returns to the title until Act 2's
  map exists; once it does (1007's act change), the same save starts
  Act 2's map. Bump `SAVE_VERSION` if the format changes, with a
  migration test.
- The title's `Continue` (if 0802 has one) treats an act-complete save
  the same way.

**Out (do not do):**
- Act 2 (content and its world map).
- New Game+, a post-game, credits changes (the credits screen stays as
  it is; whether it plays after "Act 1 ends" is unchanged from today's
  "To be continued").
- Chapter 8's scripts and battle (0750, 0854).

## Implementation steps

1. Read `flow.rs` (`Stage`, how `next: None` reaches
   `ToBeContinued`), 0802's save format, and how 1007 marks an act's
   final story battle and changes acts.
2. Add closing scenes to the chapter (or act) data and the loader's
   check that each named scene exists; document it in
   `assets/chapters/README.md`.
3. Add the act-complete mark to `Campaign` (via a `Command` → `Event`,
   ADR-0004), saved and loaded.
4. In `flow`: final victory → save prompt → closing scenes → "Act 1
   ends" → title. Loading an act-complete save goes to the card.
5. Test data in `assets-test/`: a two-chapter test act whose last
   chapter has a closing scene.

## Acceptance criteria

- [ ] Winning the test act's final battle plays its victory scenes, the
      save prompt, the closing scene and "Act 1 ends", then the title
      (harness test).
- [ ] The save written at the end keeps every unit, the stock, gold and
      support points, and the act-complete mark (round-trip test).
- [ ] Loading it shows "Act 1 ends" and returns to the title.
- [ ] Chapter 1 still plays exactly as before (existing tests pass).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the act-complete event; the loader's check of closing scenes.
- Integration: the scripted end of the test act; save → load of an
  act-complete save.
- Snapshot: the "Act 1 ends" card.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
