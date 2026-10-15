---
id: "0455"
title: The guide's face in the tip box (Fox Miko)
type: feature
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: done
blocked_by: []
nick_input: sign-off
completed: 2026-10-15
---

# 0455 — The guide's face in the tip box

## Context

Ticket 0057 decided a tutorial (`docs/design/tutorial.md`, *The guide*). A
new mascot, the **Fox Miko** face from the bought Tiny Tales *Heroes 2:
Rebellious Souls* pack, speaks every tip and lesson. The guide is outside the
story (`docs/story/characters/guide.md`): never in a scene, only in the tip
box.

Today the tip box is `draw_tip` in `crates/ui/src/tips.rs` (ticket 0406),
called from `crates/ui/src/screens/battle/tips.rs`. It has a title
(`Your move`), up to 3 lines of text and a close hint. Nick picked the
mockup "A1": the same box over the map, the guide's face at **twice** the
64×64 bust size (128×128 console px) on the left, the text to its right,
and the guide's name as the box's title.

Bug seen while making the mockups: at the start of Chapter 1, the start tip
("Your move") is drawn **under** Hollis's first-turn scene
(`ch01_first_turn`), so both boxes show at once. `shown_tip` in
`crates/ui/src/screens/battle/mod.rs` waits for `queued_scene()`, but once
the scene is taken from the queue and plays as an overlay it no longer
counts. Fix it here, since this ticket changes how the tip is drawn.

## Nick input

**Sign-off:** after merge, start a New Game and look at the first tip box
(the face, the name, the text fitting).

## Scope

**In:**
- Import the Fox Miko busts as portrait id `guide`:
  `cargo xtask portrait-import <library>/characters/heroes/FoxMiko guide`
  (`ascii-art` skill, *Portraits and battle art*). Use `--shift-x` if the
  head sits off-centre. Then the sidecar `assets-private/game/portraits/guide.ron`.
- Its credit (ADR-0051): the Fox Miko row in `assets/data/credits.ron`
  with `private: ["portraits/guide/", "portraits/guide.ron"]` (the same
  form as the other bought portraits), the pack and the artist.
  Push the private repo, `cargo xtask private-assets --pin`, commit
  `assets-private.rev` in this PR (`CLAUDE.md`, *Bought art*).
- Without the private assets (the public build and the gates) the guide has
  no portrait. The box then draws as it does today, with no face and no
  gap. Don't add a public placeholder picture.
- The tip box with the face (ADR-0054: the tip's view is plain data, the
  skin paints it): the guide's portrait at 2× on the left, inside the box;
  the text to its right. The box title is the guide's name, from the text
  key `guide.name` (English: `Guide` until ticket 0731 names the guide).
  The tip's own title (`Your move`) becomes the first text line, in the
  highlight colour. The box is wide enough for 60-character lines next to
  the face; check it fits the map area at the smallest window
  (`look-and-feel.md`).
- Expression: `smile` by default. `tips.ron` gets an optional
  `face: "<expression>"` per tip (validated against the portrait's
  expressions when the portrait exists).
- The bug: no tip shows while a scene overlay is on screen.

**Out (do not do):**
- New tips, lessons, the Guides list, the familiarity question (0458,
  0456, 0457, 0835).
- Changing any tip's words (0731 rewrites them in the guide's voice).
- Showing the guide anywhere but the tip box.

## Implementation steps

1. Import the portrait and credit it (Scope). Render it in the dialogue
   test screen once and look at it (`ascii-art` skill).
2. Split the tip box into a view (title, face id and expression, lines,
   close hint) and a paint function, following `screens/options.rs`
   (ADR-0054). The face is a sprite item (`draw_portrait` in
   `crates/ui/src/portrait.rs` draws at the 256 px frame; add a size
   argument or a sibling that draws at a given whole scale, ADR-0043).
3. Add `face` to the tips format in `trpg-content` and its validation.
4. Add `guide.name` to the language files (`test` pack too).
5. Fix `shown_tip` so a playing scene overlay hides the tip; a failing
   test first.
6. Render the Chapter 1 first turn with
   `cargo xtask frame-png --features private-assets` and look at it.

## Acceptance criteria

- [x] With the private assets, the first tip in Chapter 1 shows the Fox
      Miko face at 2× left of the text, titled `Guide`.
- [x] Without them, the tip box looks as before (snapshot unchanged except
      the title/first line change).
- [x] The credit lists the new files; `cargo xtask private-assets --pin`
      passes; `assets-private.rev` is committed.
- [x] A test shows the start tip doesn't show while `ch01_first_turn`
      plays, and shows after it closes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the tips format's `face` field and its validation.
- Snapshot / integration: the tip box's view (title, face, lines); the
  tip waiting for the scene overlay (scripted Harness test with tips on).
- Look: the paint test of the box with a face lives with the skin.

## Completion notes

- **Portrait:** the Fox Miko busts imported as `guide` with `cargo xtask
  portrait-import` (no `--shift-x`: the head sits in the middle), all 8
  expressions, the required five mapped as the other heroes (`happy` =
  smile, `angry` = stern...). Pushed to the private repository (9a096e1);
  `assets-private.rev` pinned. Credit: `portraits/guide/` and
  `portraits/guide.ron` added to the Tiny Tales entry (the pack's artists,
  Lunatic Red and Rayane Félix, were already in its `author`), and the
  pack to its row in `THIRD_PARTY_ASSETS.md`.
- **The box** (ADR-0054): `crates/ui/src/tips/view.rs` builds a `TipView`
  (title = the guide's name, `guide.name`; face; heading = the tip's own
  title; lines; close hint), `crates/ui/src/tips/glyph.rs` paints it. The
  face is `draw_portrait_in` in a 16×8-cell area (128×128 px: a 64×64 bust
  at 2×), at the box's top left inside the border, the text 2 cells to its
  right. Without the guide's portrait the box is as before, its title now
  `Guide` and `Your move` its first line, in the highlight colour.
- **Width (for Nick's sign-off):** the map area is 70 cells wide, and the
  face (16) plus a 60-character line plus margins needs 82, so a box with
  long lines can't fit the map area. The box stays centred on the map
  while it fits and otherwise slides left and over the side panel, never
  off the screen. Today's longest tip line (49 characters) makes the
  Chapter 1 start tip 71 cells wide: it covers the side panel's left
  border by one cell.
- **`face`** in `tips.ron`: optional, `smile` if absent, checked against
  the guide's portrait's expressions when that portrait exists (the
  public build has none, so it isn't checked there).
- **The bug:** the battle screen notes when it starts a scene over itself
  and shows no tip until it is updated again (on top once more). Test:
  `the_start_tip_waits_for_the_first_turn_scene` (Chapter 1, tips on).
- **Tooling:** `cargo xtask frame-png --tips` shows the tips (the Harness
  has them off), needed for step 6. The render was looked at: the face, the
  name and the text fit.
- No gameplay rules decided. No follow-up tickets.
