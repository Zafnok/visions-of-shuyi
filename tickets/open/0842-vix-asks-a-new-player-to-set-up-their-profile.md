---
id: "0842"
title: "On a new profile, Vix suggests setting up the profile from the title"
type: feature
milestone: M7 Chapter 1 & game flow
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
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
  the right of the menu (*Claude's starting rule*: right of the menu, not
  covering the logo), one or two lines in her voice suggesting Title →
  Profile to name the profile.
- It shows while the profile in use has **never been changed or saved
  to**: never renamed, no option kept (0839's Keep), no game saved. Once
  any of those happens, it's gone for that profile for good.
- It doesn't take input or block the menu; it hides while the cinematic
  plays.
- *Claude's starting rule:* it doesn't depend on the tutorial level
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

- [ ] A fresh profile shows the box on the title (test).
- [ ] Renaming, keeping an option, or saving a game removes it for that
      profile (tests per case); other profiles keep theirs.
- [ ] The box never covers the menu or the logo (snapshot).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the "never set up" check.
- Integration: title harness with a fresh and a set-up profile.
- Snapshot: the title with the box.

## Completion notes

*(Filled in by the session that completes the ticket.)*
