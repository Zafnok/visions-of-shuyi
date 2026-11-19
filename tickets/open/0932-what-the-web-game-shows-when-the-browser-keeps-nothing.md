---
id: "0932"
title: "What the web game shows when the browser keeps nothing: Nick picks, then it is built"
type: feature
milestone: M8 Release
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0932 — What the web game shows when the browser keeps nothing: Nick picks, then it is built

## Context

Filed from the writer trial's test audit (0766, 2026-11-19).

When the browser refuses the game its storage, nothing the player does is
kept, and the player isn't told until a save they asked for fails.
1266 (PR #671) walked the game with storage blocked and listed what
happens:

- **Said:** the map menu's Suspend shows `Save storage error:
  localStorage is turned off or blocked in this browser`. Since 0445
  this shows in a warning box over the map's bottom left. Changing an
  Option shows `Couldn't save: the change lasts until you quit`.
- **Silent, on purpose (1266's call, not Nick's):**
  - the layout picker (`crates/ui/src/game.rs` pushes
    `layout_picker::first` while no layout is kept);
  - the profile's set-up mark (`profiles::keep_set_up`,
    `crates/ui/src/profiles.rs`);
  - the title's backdrop;
  - the tips seen;
  - the familiarity answer.
  1266: "With storage blocked, the layout picker and \"How familiar
  are you?\" come back each launch: that is what a game that can't
  remember does, not a bug."

So on every visit the player:

- picks a layout again;
- finds no save, so starts a New Game;
- answers "How familiar are you?" again (it belongs to the campaign,
  `docs/design/tutorial.md`, so it returns because the campaign is
  lost).

What a player is told is look and feel and player text, so it is
Nick's (`AGENTS.md`, *Who decides what*).

Why now: the writer trial plays the demo and the script preview in
itch.io's page frame, from `html-classic.itch.zone`
(`docs/releasing.md`). A private or Incognito window, or blocked
third-party cookies, can refuse that frame its storage. The trial brief
tells applicants to Suspend a battle to keep it
(`docs/writer-trial/brief.md` on the 0766 branch). Such an applicant
learns only when pressing Suspend that nothing is kept, and a reload has
already lost the chapter so far. The public itch.io page (0901) frames
the game the same way.

This is not a blocker for the writer trial (0766): the brief and the
packets can go out with today's behavior, and this ticket improves it
whenever it lands.

The check ticket filed with this one (0931, "The web demo and the
script preview, framed on another site with third-party storage
blocked, start, play and say when a save fails") finds which browsers
refuse. **Either order works**: if 0931 is done, put its findings in
the question to Nick (who is affected); if not, render the frames with
a refusing storage anyway and say the browsers are not yet confirmed.

What a reload or a closed window does to a battle that isn't suspended
is decided by 0092. **Either order works**: if 0092 is done and picked
an automatic suspend, a browser that keeps nothing defeats that too, so
the draft notice shown to Nick for Q1 (b) and (c) says progress isn't
kept even when the game suspends on its own. If 0092 isn't done or kept
Suspend by hand only, the notice says nothing about it; if 0092 is
still open, add a line to it that an automatic suspend can't work in a
browser that keeps nothing (this ticket's notice).

## Nick input

**Decision, before building**, with the `ask-nick` skill and the
question pop-up. Render real frames of each option on the title, in both
looks (glyphs and pictures), with a storage that refuses everything.
These are the agent's suggestions; Nick may want another. No check after
merge. It is verified by tests and `cargo shuyi frame-png` pictures in
the PR, and what is left to judge by playing goes on 1064's checklist.

**Q1. What does the game say when it can't keep anything?**
- (a) As today: nothing until a save the player asked for fails (the
  Suspend warning box, the Options line).
- (b) A notice once, at start, before the title, closed with one
  press. It works like the warning PlayStation 2 and GameCube games gave
  at boot when no memory card was in: progress won't be saved, carry on
  anyway. Draft words for Nick to change: "This browser won't let the
  game keep anything. You can play, but your progress, options and
  layout last only until you close this page."
- (c) No prompt, but a quiet line that stays under the title's menu
  while it lasts, e.g. "This browser isn't keeping your progress".
- (d) Both (b) and (c).
- (e) Describe your own.

**Q2. The layout picker, while nothing can be kept:**
- (a) As today: it asks on every visit.
- (b) It isn't shown. The game starts right-handed and Options changes
  the layout for this visit. If Q1 shows a notice, it says so.
- (c) Describe your own.

## Scope

**In:**
- Rendering the options (above) and asking Nick.
- `docs/design/options.md`: a new section, *When the browser keeps
  nothing*, with Nick's answers, the date and his words.
- Knowing at start that storage is refused, and building what Nick
  picked for Q1 and Q2.
- The new text in `assets/lang/en/ui.ron` (and whatever `cargo shuyi
  check-text` requires of the `test` pack).
- Updating 1266's refusing-storage test
  (`a_storage_that_refuses_everything_still_plays_and_says_why_it_cant_save`,
  `crates/ui/tests/it/save.rs`) for any new screen in its walk.

**Out (do not do):**
- The wording of the existing errors (`save.error.storage`, the Options
  line) and the warning box's look (0445's).
- Asking the familiarity question differently: it belongs to the
  campaign (`docs/design/tutorial.md`).
- Checking real browsers (0931).
- What a reload or a closed window does to an unsuspended battle (0092;
  either order, see Context).
- The trial brief and `start-here.txt` (0766's).
- Saving progress anywhere else (a download, a code to copy): if Nick
  asks for that, file it as its own ticket (`write-ticket`).

## Implementation steps

1. **Frames for Nick.** Use a `Storage` whose every call answers
   `StorageError::Backend` with the engine's text (as 1266's test
   does). Render the title with each Q1 option and the start with each
   Q2 option, in the glyph look and the picture look. The notice is
   mocked up as plain view data, not built. Ask Nick (`ask-nick`).
   Build nothing before he answers.
2. Record his answers in `docs/design/options.md` (*When the browser
   keeps nothing*).
3. **Know it at start.** Set a flag in the game's context when the
   start-up's storage write fails: `profiles::keep_set_up` as the title
   shows, or a write and delete of a key of the game's own at start (the
   agent picks). Read it from `Ctx`, never from the platform: it is the
   storage's answer, so it works the same on every target. *Starting
   rule (the agent's, listed in the notes):* a storage that is full at
   start counts the same as one that refuses.
4. **Build Q1** as Nick picked. A notice is a screen whose logic builds
   a view and whose skin paints it (ADR-0054), with its text by key; a
   line on the title goes in the title's view (`crates/ui/src/screens/title.rs`).
   Any key it names comes from the player's keymap (`keyboard-input`
   skill). Expected size: about 120 production lines, judged by a
   one-question screen such as `screens/voices_question.rs` plus the
   flag.
5. **Build Q2** in `crates/ui/src/game.rs` where `layout_picker::first`
   is pushed, as Nick picked.
6. Update 1266's test and the title snapshots, and read the changed
   ones.

## Acceptance criteria

- [ ] `docs/design/options.md` records Nick's answers to Q1 and Q2,
      dated, with his words.
- [ ] A Harness test: a game started over a storage whose every call
      answers `StorageError::Backend` shows what Nick picked for Q1. The
      test reads the notice's or the title's view text, not cells. A game
      over a working `MemoryStorage` shows none of it.
- [ ] A Harness test: with that refusing storage and a keyboard present,
      the screen stack after the first key press is what Nick picked for
      Q2 (with or without `layout_picker`). With a working storage it is
      unchanged.
- [ ] A unit test: the flag is set when the start-up write is refused or
      full, and not set when it succeeds.
- [ ] `a_storage_that_refuses_everything_still_plays_and_says_why_it_cant_save`
      passes (updated only for the new screen, if there is one).
- [ ] `cargo shuyi check-text` and `cargo shuyi check-keys` pass. No key
      is written as a literal.
- [ ] The PR has `cargo shuyi frame-png` pictures of the result with a
      refusing storage, in both looks.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the start-up flag (refused, full, working).
- Property: none (no input space beyond the three storage answers).
- Snapshot / integration:
  - Harness tests of Q1 and Q2 over a refusing and a working storage;
  - 1266's walk updated;
  - a snapshot of the notice or the title line in each skin, read.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
