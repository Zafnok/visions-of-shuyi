---
id: "1043"
title: "Playdate: the dialogue screen's skin (text box, speaker, one-bit portraits, choices, the dialogue log)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1037", "1042", "0249", "1044"]
nick_input: none
completed: 2026-11-09
---

# 1043 — The Playdate dialogue screen

## Context

Chapter 1 is told in dialogue scenes (`crates/ui/src/screens/dialogue.rs`,
the dialogue log in `dialogue/log*`). Ticket 0249 (split off from 0242)
turns the dialogue screen into a view painted by a skin (ADR-0054). **This
ticket needs 0249 done.** 1042 made the one-bit
faces; 1037 added the `SkinSet` choice.

The Playdate screen is 50 × 15 cells (400 × 240 px). Nick kept portraits
and the dialogue log (its system menu entry, 1035; no map menu entry,
Nick 2026-10-17: the log only opens in a conversation).

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick reads Chapter 1's opening scene in the Simulator or on
> his Playdate and comments on readability (text size is today's 8 × 16
> font, decided in 0061).

## Scope

**In:**
- A Playdate paint of the dialogue view, chosen by `SkinSet`: a text box
  along the bottom (as many lines as fit with the face beside or above
  it), the speaker's name, the one-bit portrait as a sprite item, the
  reveal of text as on the PC, choices, the skip question, captions.
  Layout: *the agent's starting rule (Nick can veto)*: the PC's arrangement
  (face beside the text box) scaled to the 50 × 15 screen; if it doesn't
  fit 0062's portrait size, the face sits above the box.
- The dialogue log: a full-screen page, scrolled with the cursor.
- Scene places (0029, `look-and-feel.md`, *Places behind conversations*;
  built by 0732–0734): on the Playdate the place is drawn in the glyph
  look, darkened, as the battle map is. If 0732 isn't done, leave it out
  and add it to 0732's follow-ups.
- Text that names a button goes through the keymap (`keyboard-input`
  skill).
- Look tests with the skin; behaviour tests read the view.
- **The title cinematic's talk and faces shots** (ticket 1044 left them
  to this ticket): `crates/ui/src/cinematic/talk.rs` paints a text box
  with `dialogue::glyph::paint` and `cinematic/faces.rs` the faces in
  the PC's 34 × 18 frames, so on the Playdate both are off the 50 × 15
  screen today (the talk shots blank, the faces shot two frame edges).
  Paint them with this ticket's Playdate paint (the talk shot through the
  dialogue's skins; the faces in the Playdate's portrait frames), and
  add a check to the Playdate title test in `tests/it/playdate.rs`
  (`the_title_cinematic_plays_to_its_menu_on_the_playdate`).

**Out (do not do):**
- Voices (1048).
- Changing the PC dialogue screen or any script text.

**Carve (1105):** either order works. If 1105 moved the dialogue format to
the engine, the view and skins stay the game's.

## Implementation steps

1. Read 0242's (c) ticket's view, `dialogue.rs`, `dialogue/log.rs`,
   0062's portrait size.
2. The Playdate paint and its tests.
3. `frame-png --playdate` pictures of a scene (with the placeholder
   faces; never the bought ones) for the PR.
4. Simulator: Chapter 1's opening scene.

## Acceptance criteria

- [x] Chapter 1's scenes play in the Simulator with faces, names, text,
      choices and the log.
- [x] Pictures in the PR (placeholder faces only).
- [x] No PC snapshot changes.
- [x] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit (look, with the skin): text box with face, a choice, the log.
- Integration (harness, Playdate skin set): a scene plays to its end.

## Completion notes

**What was done.**

- **The Playdate paint of the dialogue view**
  (`crates/ui/src/screens/dialogue/playdate.rs`, chosen by the screen's
  new `SKINS`): the two faces at 2× in 18 × 10 frames along the top
  (whoever speaks in a double frame, the other single and faded), each
  name in the bottom of its frame, the caption on the top row, and a
  3-line text box of 47 letters in a window across the bottom. The
  reveal is the PC's; the key that goes on and the blinking `▼` are in
  the bottom of the box's frame; a choice grows the box upward over the
  faces with the focused reply the bar; the skip question is asked in
  the box.
- **Pages of 47 letters on the Playdate.** The screen wraps its pages as
  wide as the skin set shows them (`text_w`, `DialogueScreen::wrapped_for`,
  used by `told` and the debug scene preview), so a box turns about
  twice as many pages there. No script text changed.
- **The place behind the talk** (0732–0734, 0738): the Playdate's map
  skin, darkened in one bit by turning its white ground to light dots
  (`place_ground` in the Playdate palette). A battle's scene darkens the
  battle below it the same way. The engine's fade turns every one-bit
  colour black at 40%, so it can't be used.
- **The dialogue log** (`dialogue/log/playdate.rs`): a window over the
  whole screen, its title and help in the frame, a row per line (name,
  then the text two cells in), the cursor's row the bar, lines seen
  ahead in grey dots, scrolling with the cursor as the PC's (its
  placing is shared with the PC's look). Opened from the system menu
  (1035).
- **The title cinematic's talk and faces shots** paint through the
  dialogue's skins: the talk shot wraps to 47 letters and its box grows
  upward to hold the whole text; the faces shot uses the Playdate's
  frames, half as far apart (`FaceLook`).
- **Tests.** Look tests with the skin (`dialogue/playdate/tests.rs`:
  faces, names, box, narration, mirroring, caption, a choice, growth,
  the wait key, the skip question, the place dotted, an opening's cut,
  a battle's scene; `log/playdate/tests.rs`: rows, bar, ahead, back to
  latest, scrolling, wrap width). Behaviour: pages are 47 letters on
  the Playdate's skins (`dialogue/tests.rs`). Integration
  (`tests/it/playdate.rs`): Chapter 1's scenes before its first battle
  played to their end with A on the Playdate's skins (every line ≤ 47,
  faces at 2×, the log from the system menu and B), frames drawn one-bit
  as the front end draws them; the title cinematic test now checks the
  talk shot's box and the faces in the Playdate's frames.
- **Pictures:** `docs/design/playdate/1043-*.png` (frame-png, the public
  placeholder faces), listed in `playdate.md`, *The dialogue screen
  (1043)*.

**Deviations.**

- `cargo shuyi frame-png --playdate` pressed its `--pad` buttons as a
  generic controller, so the pictures named Xbox buttons ("B ▼" for A).
  One line in `crates/xtask/src/game_tools.rs` tells its harness the pad
  is the Playdate's; the game itself already named A.
- A face's name is in the bottom of its frame, not on a row under it:
  the 15 rows hold the 10-row frames and the 5-row box, nothing more.

**the agent's starting rules** (in `docs/design/playdate.md`, *The
dialogue screen (1043)*; for 1064's checklist, Nick can veto):

- Names in the bottom of the faces' frames; the caption on the top row,
  across the frames' tops.
- A page is 3 lines of 47 letters (twice as many pages as the PC).
- Narration in the same black letters as speech (the PC's is dimmer).
- The `A ▼` in the bottom of the box's frame.
- A place behind the talk is darkened by dotting its white ground; the
  study's "darkened less" looks like the rest (one bit has no
  brightness); an opening cuts to the darkened look halfway.
- No faces in the log (a 2× face would take 8 of its 13 rows).

**Left to judge by playing (1064):** Nick reads Chapter 1's opening
scene in the Simulator or on his Playdate and comments on readability
(today's 8 × 16 font, 0061); whether names in the frames' bottoms are
easy to find; whether the dotted place is too busy beside the faces;
whether twice as many pages feels slow; whether the log without faces
is easy to follow.

**Simulator** (`cargo xtask playdate --release --both`, public
placeholder faces, driven by window messages to the Simulator's
window): New Game into Chapter 1's opening scene: the faces, names,
caption and the dotted place, pages of 47 letters with `A` in the box's
frame; the choice answered; the system menu's Dialogue log opened (the
reply in it) and closed with B; A played the scenes through to the
first battle.

**Engine friction:** estimate 380 production lines (one Playdate paint
of an existing view and its log, judged by 1044's title paint, 238, and
1045's lead screen), measured 312 (0.8×). No signals.

**Follow-up tickets:** none.
