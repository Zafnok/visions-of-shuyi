---
id: "0924"
title: "The Playdate's title: the campfire, its crackle, or neither"
type: feature
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0922"]
nick_input: decision
completed:
---

# 0924 — The Playdate's title: the campfire, its crackle, or neither

## Context

Ticket 0922 gave the PC and web title a campfire in the bottom-left
corner, embers rising from it and its crackle about 15 dB under the title
song, all fading with the picture when the title goes idle into the
cinematic (`docs/design/look-and-feel.md`, *Title screen*, *The campfire,
its embers and its crackle*). Asked what the Playdate's title gets (crackle
only; the fire as a dithered one-bit picture with the crackle; nothing),
Nick answered: "decide later, followup ticket".

So today the Playdate's bundle leaves `title/campfire.png` out
(`crates/content/playdate-bundle.txt`), its title art loads with no
campfire, and a title with no campfire plays no crackle
(`TitleScreen::crackle`, `crates/ui/src/screens/title.rs`). Its title
skin is `crates/ui/src/screens/title/playdate.rs` (ticket 1044): one bit,
no fades (things cut in or out halfway), no embers (dots are lost in the
backdrop's dither). Its one-bit pictures are made ahead of time
(`trpg_content::playdate_images`, ticket 1040); its sounds are its own
copies, converted and packed in the `.pdx` (ticket 1047, ADR-0084).

## Nick input

**Decision** (`ask-nick`, before building), with `cargo shuyi frame-png
--playdate` renders of the Playdate's title and, for the sound, the
Playdate's own converted copy of `campfire_02.ogg`:

- A **crackle only**: no picture, the same crackle under the song, cut
  off when the picture cuts to black.
- B **fire and crackle**: the campfire as a dithered black-and-white
  picture in the corner (2× or 3×, as fits 400×240), its four frames
  turning, with the crackle; no embers.
- C **nothing**: the Playdate's title stays as it is.
- Or his own. Name a real Playdate game with a fire or ambience on its
  title or menu for each, honestly.

## Scope

**In:** the pick, built: for B the one-bit campfire through
`playdate_images` and the Playdate's title skin (ADR-0038, ADR-0054: the
view says what, the skin paints it), for A or B the crackle in the
Playdate's sound pack; `look-and-feel.md` and `playdate.md` (*The title
cinematic*) updated with Nick's words.

**Out (do not do):** the PC and web title (0922); embers on the Playdate;
a new sound.

## Implementation steps

1. Render the options and ask Nick (throwaway switches, never committed).
2. B: make the fire's one-bit frames where the Playdate's other title
   pictures are made (`crates/content/src/playdate_images.rs`), take
   `title/campfire.png` off `playdate-bundle.txt` (or load the one-bit
   copy in its place), and paint it in `title/playdate.rs`, shown while
   the title's picture is (`view.picture` at least half), as the backdrop.
3. A or B: make sure the Playdate's sound pack carries `campfire_02`
   (ticket 1047's conversion) and that `TitleScreen::crackle` plays it
   there (today it needs a campfire picture; for A, give the title art a
   way to have the crackle without the picture).
4. Record the pick in the design docs.

Expected size: about 80 production lines for B (one picture in the
one-bit maker and one sprite in the Playdate skin), about 20 for A,
judged by 1044's title skin.

## Acceptance criteria

- [ ] Nick picked; recorded in `look-and-feel.md` and `playdate.md`.
- [ ] The Playdate's title shows and plays what he picked
      (`cargo shuyi frame-png --playdate` pictures in the PR).
- [ ] All gates in the `run-gates` skill pass, the Playdate build and
      its bench included.

## Tests required

- Unit: the one-bit fire's frames (B); the crackle asked for on the
  Playdate's title, and not with C.
- Snapshot: the Playdate's title at a fixed frame.

## Completion notes

*(Filled in by the session that completes the ticket.)*
