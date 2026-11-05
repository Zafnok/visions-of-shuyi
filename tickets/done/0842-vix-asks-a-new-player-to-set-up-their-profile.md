---
id: "0842"
title: "On a new profile, Vix suggests setting up the profile from the title"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-04
---

# 0842 — Vix suggests setting up the profile

## Context

Profiles (0836, 0837): on first launch the game makes "Player 1" without
asking; Title → Profile renames, adds, switches. Chapter 1 playtest round
3 (0804, 2026-10-16), on the Profile screen:

> "I think probably on first launch, we should have the guide kinda pop
> off to the right in its own window and be like "Hey! Did you change your
> profile yet?" or something, just a small clippy style message showing
> only until the player makes a save to their profile (customize options
> or name) or to the game itself."

Vix: face and box from 0455/0459, voice `docs/story/characters/guide.md`
(dry, calm; no exclamation marks, so "Hey!" becomes her voice).

**Waits for the engine carve and the Playdate port** (Nick, 2026-10-16),
so `blocked_by` has 1118 and 1050.

**Re-gated 2026-10-21 (0129):** the carve is done and 1050 was folded
into the Act 1 playtest (1064), so neither 1118 nor 1050 is in
`blocked_by` any more.

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Launch with a fresh browser profile (a private window):
> does the message sit well beside the title, and go away when it should?

## Scope

**In:**
- On the title, after the first press, a small box with Vix's face to
  the right of the menu (*the agent's starting rule*: right of the menu, not
  covering the logo), one or two lines in her voice suggesting Title →
  Profile to name the profile.
- It shows while the profile in use has **never been changed or saved
  to**: never renamed, no option kept (0839's Keep), no game saved. Once
  any of those happens, it's gone for that profile for good.
- It doesn't take input or block the menu; it hides while the cinematic
  plays.
- *the agent's starting rule:* it doesn't depend on the tutorial level
  (there's no campaign yet on the title). List it in the Completion notes (1064's checklist).
- Profiles keep a "set up" flag (or derive it from the profile's data;
  say which).

**Out (do not do):** a pop-up window outside the game (the box is drawn
in the game's frame, "its own window" read as its own box).

## Implementation steps

1. Add the flag or derivation to the profile (0836's storage).
2. Add the box to the title's view and skin (ADR-0054), reusing the
   guide box.
3. Text ids in the language files (ADR-0045).
4. Tests.

## Acceptance criteria

- [x] A fresh profile shows the box on the title (test).
- [x] Renaming, keeping an option, or saving a game removes it for that
      profile (tests per case); other profiles keep theirs.
- [x] The box never covers the menu or the logo (snapshot).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the "never set up" check.
- Integration: title harness with a fresh and a set-up profile.
- Snapshot: the title with the box.

## Completion notes

**Done.** On the title, once the menu shows, Vix's note sits right of the
menu box, level with its top: her face (the neutral one, at 1×, half the
tip box's) and two lines in her voice:

> **Still Player 1?**
> Profile, on this menu, gives you a name of your own. Your options and
> saves stay with it.

(`title.note.heading`, `title.note.text`; `{name}` is the profile's name.)
It takes no keys, isn't there with the "press any key" prompt or while the
cinematic plays alone, and fades with the menu. Without the bought art
it is the same box without the face.

**Flag or derivation: both.** `profiles::set_up` derives it from the
profile's data: its name isn't the number it was made with (`Player N`, as
the engine names it), its saved settings differ from the defaults apart
from the layout, or it has a save slot or a suspend save. The first time
the title sees it set up it keeps a mark under the profile's own
`set_up` key, so it stays set up for good (a suspend save continued, or
options put back, don't bring the note back). The derivation needs no
hook in the engine's Profile and Options screens: the title looks again
each time a screen over it closes (`uncovered`), and at launch.

Layout: `title::glyph::note_layout` (right of the menu with a 2-cell gap,
at most to one cell short of the right edge, text wrapped to fit). The
tip box's frame painter takes the face's size now
(`tips::glyph::paint_frame_sized`).

Tests: unit (`profiles`: each way to be set up, a layout picked alone
isn't, kept for good, per profile; `title`: shown with the menu, not with
the prompt or the cinematic, mark kept on the first update), integration
(`it::profiles`: rename on the Profile screen, Keep in Options, a save,
another profile), the skin's layout in both languages with the widest
menu, and the title snapshots, which now show the note.

**The agent's starting rules** (for the Act 1 playtest, 1064):

- The note sits right of the menu, top level with it, never over the
  logo (the ticket's starting rule).
- It doesn't depend on the tutorial level.
- What counts as an option kept: any saved option that differs from the
  default, whatever saved it: Keep in Options, but also the fullscreen
  shortcut, the map zoom or auto end turn changed in battle, and turning
  voices off at New Game's question. The layout picked on a first launch
  doesn't count.
- A profile added on the Profile screen gets a name typed for it, so it
  counts as set up at once unless it's named exactly `Player N` (its
  number).
- Not on the Playdate: its 50×15 screen has no room beside the menu, so
  its skin leaves the note out.
- Her face at 1× (8×4 cells), not the tip box's 2×, to keep it small.

Playtest checklist (1064): *Launch with a fresh browser profile (a
private window): does the message sit well beside the title, and go
away when it should?*
