---
id: "1303"
title: "Unnamed enemies show the shared soldier face in unit info"
type: bug
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1303 — Unnamed enemies show the shared soldier face in unit info

## Context

Filed from 0766's test audit of the Chapter 1 web demo that applicant
writers will play (0914, 0915). In unit info (the `Info` action on a
unit in battle), every enemy but Red Harl shows an empty box with the
word `portrait`. A writer reading Chapter 1's enemies sees a stand-in
on almost every one of them.

This comes before the writer trial's first packet (0766): every Red
Company enemy but Harl shows the `portrait` box.

Why: ticket 1295 gave the info screen the unit's face, but only for a
named character. `info::face` (`crates/ui/src/screens/battle/info.rs:39`)
looks the portrait up by `unit.character` (`:43`). A generic unit has
`character: None` (`crates/core/src/unit.rs:163`), so it gets no face,
and the glyph skin draws the placeholder label from `battle.portrait`
(`crates/ui/src/screens/battle/glyph/info.rs:113`;
`assets/lang/en/ui.ron:669`). 1295 kept this on purpose ("generic
soldiers and enemies with no portrait keep the box and label as
today"). Chapter 1's enemies are generic templates (`red_brigand`,
`red_raider`, `red_archer`, `red_rider`, `red_guard` in
`assets-private/content/battles/ch01.ron`); only `red_captain` is a
character.

The face already exists and its use is decided: ticket 0706 (Nick
picked "soldier B") made one shared `soldier` portrait "for unnamed
enemies". `assets/portraits/README.md:20` calls it "the one face of
every unnamed enemy". It is bought art in the private repository
(`assets-private/game/portraits/soldier.ron`, eight expressions,
`neutral` first). Nothing shows it for a unit today.

Related: 1295 (done; the face lookup and the skin's draw this reuses),
1222 (portraits as the engine's picture set) and 1238 (engine panels).
Either order works with those two: if one lands first, use its types for
the same thing; if this lands first, they move this code with the rest.

## Nick input

None. The look is decided: 0706 and `assets/portraits/README.md` make
`soldier` the face of every unnamed enemy, and the info screen draws it
the way 1295 draws a named character's. It is verified by the tests
below and a `cargo shuyi frame-png` picture in the PR. Whether the face
reads well at its size goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- In unit info, a generic unit (`character: None`) of
  `Faction::Enemy` shows the `soldier` portrait in its first expression
  (its neutral one, as 1295 does for characters), in the PC's glyph and
  picture looks (they share this screen's skin).
- When the `soldier` portrait isn't loaded (a build without the private
  assets), the placeholder box and label stay as today.
- The portrait id as one named constant with a doc comment that points
  at 0706 and `assets/portraits/README.md`.
- Module docs of `info.rs` and `glyph/info.rs` updated.

**Out (do not do):**
- Generic units of the Player, Ally or Neutral sides: the decision
  covers enemies only. They keep the placeholder.
- A face per enemy group (Hounds, wardens, privateers) or per class: not
  decided. Every unnamed enemy shares `soldier`, as the README says.
- Monsters: no elemental is placed as an enemy in Act 1's content (only
  in test content such as the Quick Battle's `test_frost_elemental`).
  The rule applies to them as written. The Completion notes say so for
  1064's checklist; don't add a class exception.
- The Playdate's info screen (`playdate/info.rs`): it has no portrait
  slot (1295's notes).
- The side panel, the forecast and the dialogue screen.
- Drawing or editing any art (ADR-0040), or adding a `soldier` file to
  `assets/` or `assets-test/`.

## Implementation steps

Expected size: about 15 production lines (one constant, one fallback
branch in `info::face`), judged by 1295's lookup (`face`, 11 lines).

1. In `crates/ui/src/screens/battle/info.rs`, add
   `pub const UNNAMED_ENEMY_PORTRAIT: &str = "soldier";` beside `face`,
   with a doc comment naming 0706 and `assets/portraits/README.md`. If
   `cargo shuyi check-text` flags the literal, mark it the way the file
   already does (`// check-text: not a data name (a portrait id)`).
2. `info::face` already passes the id to `progress::portrait_of`
   (`crates/ui/src/screens/battle/progress.rs:596`), which takes it as
   `Option<&str>` and returns `None` for `None`. Check that this still
   holds; then, when `unit.character` is `None` and
   `unit.faction == Faction::Enemy`, call it with
   `Some(UNNAMED_ENEMY_PORTRAIT)` rather than duplicating the lookup
   (`lead.portrait_for` leaves any id but the lead's as it is). The
   `FaceView` built after it stays the same, with that id and the
   portrait's first expression. A named character keeps today's path
   unchanged. A missing portrait returns `None`, so the skin draws the
   placeholder as today.
3. No change to `glyph/info.rs`'s drawing is needed: it already draws
   any `view.face` with `draw_portrait_in` and falls back to the label
   when the face's portrait is missing. Update its module doc ("a
   placeholder box and label for a unit with none" becomes "... for a
   unit with no face: a generic unit that isn't an enemy, or a build
   without the private portraits").
4. Update `info.rs`'s module doc and `face`'s doc comment.
5. Run `cargo xtask ticket-lint`.

## Acceptance criteria

- [ ] With a `soldier` portrait in the content, `info::face` on a
      generic enemy returns `soldier` and its first expression
      (`neutral`).
- [ ] Without a `soldier` portrait, `info::face` on a generic enemy
      returns `None`, and the info screen shows the `portrait` box and
      label as before.
- [ ] A generic unit of another side gets `None`; a named character
      still gets its own portrait (the existing
      `the_info_screen_is_the_units_and_closes` test still passes).
- [ ] A look test shows the info screen of a generic enemy drawing the
      portrait's picture, not the label.
- [ ] A `cargo shuyi frame-png` picture of a generic enemy's info screen
      is in the PR, cropped to the screen, test content only (ADR-0118).
- [ ] `cargo xtask ticket-lint` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (`crates/ui/src/screens/battle/view_tests.rs`, beside
  `the_info_screen_is_the_units_and_closes`): `info::face` on a Quick
  Battle `test_brigand` against a portraits map that has a `soldier`
  entry (a copy of `test_knight`'s `Portrait` under that id, built in
  the test: `face` takes the map), against one without it, and on a
  generic unit whose faction is set to `Ally`.
- Snapshot / integration: a text snapshot of a generic enemy's info
  screen with the face drawn (the sprite overlay line names the
  portrait's picture, as 1295's snapshots do), with the `soldier` copy
  added to the test context's content. If the content can't be changed
  in a test, build the view with the face set and the portrait in
  reach the way 1295's look test does.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
