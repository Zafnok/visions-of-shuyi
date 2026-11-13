---
id: "0495"
title: "Vix's boxes don't take the keys meant for other screens"
type: bug
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0495 — Vix's boxes don't take the keys meant for other screens

## Context

Nick, 2026-11-12, after playing the Chapter 1 demo (0915, PR #657):

> "Vix's tutorial steals priority (D/F confirm / cancel) over other UI
> elements. So when she asks you to go to options for example, several of
> her tutorials then play one after another and D both closes options and
> her tutorial so you're trying to go back out of options to do her
> tutorials and instead just skipping them one after another."

He calls it long-standing, and wants it cleaned up before the demo goes to
the writers applying (0766).

How it works today (read by 0915's session, not yet reproduced):

- **A box takes every key.** `BattleScreen::overlay_key`
  (`crates/ui/src/screens/battle/mod.rs`): while a tip is on screen it
  returns `true` for *every* action, so nothing else on the battle screen
  gets a key; Confirm or Cancel dismiss it once it has been drawn (0467).
- **Opening Options lets the next boxes in.** The `keys` lesson
  (`assets/data/lessons.ron`) ticks "Open Options from the map menu" on
  `DoneWhen::OptionsOpened`. Its last item ticked, the lesson closes, and
  what was waiting comes due: the next lessons, the zoom tip ("the first
  time the player is back on the map with no lesson in play",
  `docs/design/tutorial.md`, Tips), the battle-start tip. The battle is
  covered while Options is open (`covered` / `uncovered`), so they show the
  moment Options closes, one after another.
- **Leaving Options is Cancel; so is dismissing a box.** The player
  pressing Cancel to back out of Options (or a page of it) and then again
  to get back to the map lands on a box that just came up, and closes it
  unread. With several queued, each press of D or F skips the next.

Related: 0406 (tips), 0457 (lessons), 0464 (a lesson only when it can be
done), 0466 (the zoom tip), 0467 (tips lost before they show; a key in the
frame a tip comes due doesn't close it), 0476 (her voice), 0494 (her
missing clips).

## Nick input

**Decision** (asked before building, with the `ask-nick` skill, options
from real games and "describe your own", with mockups of the battle
screen in both looks): how one of Vix's boxes is put away, and what the
player can do while it is up.

- **A. Modal, as now, but safe:** the box takes the keys until it is
  closed, but never a press that began on another screen, and only one
  box comes up at a time (the next waits for the player to do something
  on the map). As *Fire Emblem: Three Houses*' tutorial pop-ups (a press
  of A closes each).
- **B. Out of the way:** the box sits in a corner and the battle takes
  every key under it; it goes when the player does what it talks about,
  or when the next one replaces it. Like the hint boxes many strategy
  games keep at the screen's edge while the game goes on.
- **C. Its own key:** the box stays until a key of its own (not Confirm
  or Cancel) closes it; every other key goes to the battle.

Whatever he picks, the two bugs below are fixed: no press that closed
another screen closes a box, and boxes never come up in a row unread.

## Scope

**In:**
- Reproduce first: a Harness test (`crates/ui/tests/it/`, a module in
  `main.rs`) that plays Chapter 1's turn 1 at Full, opens Options from the
  map menu, backs out with Cancel, and shows that the boxes that came due
  were dismissed without being seen. It fails before the fix.
- A press that closes a screen above the battle (Options, the map menu,
  the Guides list) never dismisses a box: a box that comes up as the
  battle is uncovered takes no Confirm or Cancel until those keys have been
  released and pressed again.
- Boxes don't come up back to back: after one is dismissed, the next
  waits until the player has done something on the map (*the agent's
  starting rule*, unless Nick's decision says otherwise).
- What Nick decides for how a box is put away (A, B, C or his own).
- `docs/design/tutorial.md` records the decision; `lessons.ron`'s and
  `tips.ron`'s header comments say how a box is dismissed.

**Out (do not do):**
- Rewording Vix's lines (dialogue writing is paused, AGENTS.md rule 3).
- Her voice and its clips (0494).
- New lessons or tips, or when each one triggers (beyond waiting its
  turn as above).
- Key names or bindings: help text names keys through the keymap
  (`keyboard-input` skill); no key is hard-coded.

## Implementation steps

1. Ask the decision with `ask-nick` (mockups of the box over a menu and
   on the map, glyphs and pictures, `cargo shuyi frame-png`). Record it in
   `docs/design/tutorial.md`.
2. Write the failing Harness test above.
3. In `crates/ui/src/screens/battle/`: track the Confirm and Cancel held
   when the battle is uncovered (`uncovered` in `mod.rs`), and have
   `overlay_key` ignore them for a box until released (0467's "once
   drawn" check, `tips.rs` `on_screen`, is the pattern). Do the same for
   lessons' boxes and the end-of-lessons box (`lessons_end_key`).
4. Space the boxes: after a dismissal, hold the next tip or lesson until
   the player acts on the map (a cursor move, a unit picked, a menu
   opened). Put it beside the queue in `tips.rs` / `lessons.rs`.
5. Build Nick's choice for how a box is put away; if it is B or C,
   `overlay_key` stops returning `true` for every action.
6. Update the header comments in `assets/data/lessons.ron` and
   `assets/data/tips.ron`, and `docs/design/tutorial.md`.

Expected size: about 120 production lines for steps 3–4 (judged by 0467's
change to `tips.rs` and `overlay_key`), more if Nick picks B or C.

## Acceptance criteria

- [ ] The Harness test (Options opened from the map menu on turn 1 at
      Full, then Cancel back to the map) fails before the fix and passes
      after: each box that came due is on screen until a press made after
      it came up.
- [ ] A test: Cancel held from closing Options doesn't dismiss the box
      that comes up; a fresh Cancel does (or, with B or C, the rule Nick
      picked).
- [ ] A test: two boxes due at once come up one at a time, the second
      only after a map action.
- [ ] `cargo shuyi frame-png` pictures in the PR: a box after leaving
      Options, in both looks.
- [ ] `docs/design/tutorial.md` holds Nick's decision; the agent's
      starting rules are listed in the completion notes for the Act 1
      playtest (1064).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the box's key handling after the battle is uncovered; the spacing
  of queued boxes.
- Integration (Harness): the Options round trip on Chapter 1's turn 1.
- Snapshot: the box after leaving Options, both looks.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
