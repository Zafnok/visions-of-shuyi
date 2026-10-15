---
id: "0058"
title: "Decide: which options belong to a save file"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-14
---

# 0058 — Decide: which options belong to a save file

## Context

Nick, Chapter 1 playtest round 2 (0804, 2026-10-14): "they should probably be
per-save-file. I have never saved and yet the browser cached my options
settings which feels weird to me."

Today (ADR-0050, `crates/ui/src/settings.rs`, `Ctx::change_settings` in
`crates/ui/src/screen.rs`): every option is one record per computer or
browser (storage key `settings`; on the web `localStorage`), written the
moment it changes, whether or not the game was ever saved. Saves
(`slot_01`..`slot_30`, `suspend`; ADR-0039) hold no options, only the
campaign; the game mode is the one thing that is per campaign
(`Campaign.mode`). Key bindings (`keybindings`, ADR-0031) and the hand
layout are separate and needed before any save is loaded (the first-launch
picker). The tips already seen (`tips_seen`) and the title backdrop are also
remembered without a save. ADR-0050 never weighed per-save options.

The options (Options screen, `docs/design/options.md`): text speed,
animation speed, combat animations, enemy phase speed, auto end turn,
fullscreen, cursor style, map look, map zoom, color theme, music and sound
volume, hand layout, key bindings, game mode, reset tips. Coming: language
(0825), voices (0826).

How other games do it, to check before showing: many console tactics games
keep options per save file, while most PC games keep volume, display and
controls per machine.

## Nick input

**Decision**, with the `ask-nick` skill, as player situations with an
example (two players sharing a computer; one player with two campaigns):

- **A. Everything per save**, with the title screen's Options for the
  machine (the title has no save loaded).
- **B. Split**: per machine (display, volume, language, keys, layout) and
  per save (game speed, auto end, cursor, map look/zoom, theme, tips seen).
  Show the exact split as a table.
- **C. Keep per machine**, but nothing is remembered until the player
  saves or finishes the first-launch setup (Nick's "I never saved").
- **D. Player profiles** (a profile picked at launch holds its options and
  its saves).
- **E.** "Describe your own."

Also: the **tutorial level** (ticket 0057, `docs/design/tutorial.md`):
picked at New Game and kept with the campaign (0835). Ask whether the
player can change it later, and where (Options, per save).

Sub-questions: what does a New Game start with (defaults, or the last
used)? What happens to options changed on the title screen?

## Scope

**In:**
- The question and the answer, recorded in `options.md` (a *Where options
  are kept* part).
- An ADR superseding ADR-0050's storage part if the answer changes it
  (`write-adr` skill), written by the follow-up or here if it is only a
  decision record.
- Follow-up implementation ticket(s), added to 0804's `blocked_by`.

**Out (do not do):**
- Implementing it.
- Changing the save format here.

## Implementation steps

1. Ask; record.
2. Write the follow-up ticket(s): what moves into `SaveFile` (ADR-0039;
   bump its version and migrate old saves), what stays in `settings`,
   what the title screen's Options shows, and tests. Add them to 0804's
   `blocked_by` and `docs/ROADMAP.md`.
3. Update tickets that add options (0825, 0826, 0049) so they say where
   their option lives.

## Acceptance criteria

- [x] `options.md` records the answer and Nick's words.
- [x] Follow-up tickets exist; 0804, 0825, 0826 and 0049 name them where
      needed.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None (design decision; docs and tickets only).

## Completion notes

Asked Nick with the question pop-up (two rounds plus one clash check).
He picked **D, player profiles**, then the recommended answer to every
follow-up: a profile holds every option (keys, layout, volume and
fullscreen included), the tips seen and its own 30 slots and suspend save;
the game opens on the last profile used and the title menu switches; the
first profile, `Player 1`, is made without asking; options kept before
any save are fine now that they belong to a profile; the tutorial level
can be changed later in Options; a New Game uses the options last used.

Recorded in `docs/design/options.md` (*Where options are kept*), with
`tutorial.md` and the design README updated.

Follow-up tickets, all added to 0804's `blocked_by` and the roadmap:
- **0836** profiles under the options, keys, tips seen and saves, the
  migration into `Player 1`, and the ADR superseding ADR-0050's storage
  part.
- **0837** the title menu's Profile screen (switch, add, rename, delete),
  after 0836.
- **0838** the tutorial-level row in Options, after 0835.
- **0839** leaving Options asks whether to keep the changes (Nick on the
  PR: overwriting options must be deliberate).

Nick on the PR also said the controller's letter grid stays, but Steam
(and any console) uses the platform's own keyboard: recorded in
`options.md` and in 0903's Deck step.

0825 and 0826 now say their settings belong to the profile (no new
dependency); 0049 says Ironman stays with the campaign; 0835 points at
0838.

*Claude's starting rules* (Nick may veto):
- `Profile` sits below `Options` on the title menu.
- A new profile starts from the default options and runs the hand-layout
  picker.
- Names up to 12 characters (*tunable*), no two alike; up to 8 profiles
  (*tunable*); the only profile can't be deleted. Deleting asks yes/no,
  focus on No.
- Which profile was last used and the title backdrop order stay with the
  computer.
- The tutorial row shows only while a campaign is loaded, not on the
  title's Options, and changes the level at once without asking.
- Leaving Options with changes asks Keep / Discard / Back, cursor on
  Back; backing out of a name box with an unkept name asks the same way.
- The Zoom map key and the hand-layout picker keep saving at once.
