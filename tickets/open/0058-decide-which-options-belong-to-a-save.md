---
id: "0058"
title: "Decide: which options belong to a save file"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
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

- [ ] `options.md` records the answer and Nick's words.
- [ ] Follow-up tickets exist; 0804, 0825, 0826 and 0049 name them where
      needed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- None (design decision; docs and tickets only).

## Completion notes

*(Filled in by the session that completes the ticket.)*
