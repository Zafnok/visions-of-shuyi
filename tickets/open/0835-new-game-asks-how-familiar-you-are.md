---
id: "0835"
title: New Game asks how familiar you are (the tutorial level)
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: sign-off
completed:
---

# 0835 — New Game asks how familiar you are

## Context

Ticket 0057 (`docs/design/tutorial.md`, *How familiar are you?*). Nick:
"Let's let player pick familiarity. Not familiar --> full tutorial, familiar
with other games --> small tutorial, familiar with this game in particular
--> no tutorial whatsoever."

| Answer | Level |
| ------ | ----- |
| Not familiar | `Full`: basic-controls lessons, *different here* lessons, every tip |
| Familiar with other strategy RPGs | `Short`: only the *different here* lessons |
| Familiar with this game | `None`: no lessons, no tips |

New Game today: title → `mode_select` (Classic/Casual,
`crates/ui/src/screens/mode_select.rs`, ticket 0801) → `lead_select` → the
story. The game mode is kept per campaign (`Campaign.mode` in
`crates/core/src/campaign.rs`, saved with it). Tips are switched on by
`Ctx::tips_enabled` (`crates/ui/src/screen.rs`; `app` turns it on) and
remembered per computer (`tips_seen`).

## Nick input

**Sign-off:** after merge, start a New Game and try each answer.

## Scope

**In:**
- A new screen right after `mode_select` and before `lead_select`
  (*Claude's starting rule*): a heading and three choices, each with one
  line explaining it, like `mode_select`. The cursor starts on
  "Not familiar". Back returns to `mode_select`. Words in the language
  files (Claude writes them; American English; `test` pack too).
  ADR-0054: the screen builds a view, a skin paints it (copy
  `mode_select`'s `view.rs` / `glyph.rs` split).
- `TutorialLevel { Full, Short, None }` kept with the campaign, next to
  `mode` (`Campaign.tutorial`), saved and loaded with it. Saves made before
  this ticket load as `Full` (`#[serde(default)]`), as the game behaves
  today. *Claude's starting rule.*
- Tips show only at `Full` (and still only when `tips_enabled`). At
  `Short` and `None`, no tip shows. The Quick Battle and the test chapter
  use `Full`.
- The save summary (load screen) doesn't need to show the level.

**Out (do not do):**
- Lessons (0456, 0457): this ticket only stores the level and gates tips.
- Changing the level after New Game: 0058 put it in Options; that row is
  **0838** (was: part of 0058's decision; if 0058 is
  done first and puts it in Options, add the row as it says).
- The guide's face (0455), the Guides list (0458).

## Implementation steps

1. `TutorialLevel` in `trpg-core` next to `GameMode` (plain data), the
   field on `Campaign` with a serde default, and a round-trip save test.
2. The familiarity screen in `crates/ui/src/screens/` and its place in the
   New Game flow (`crates/ui/src/flow.rs`, wherever `ModeSelectScreen`
   hands over to `lead_select` now).
3. `Ctx` exposes the running campaign's level (as `campaign_mode` is
   exposed); the battle's tip check (`crates/ui/src/screens/battle/tips.rs`)
   shows tips only at `Full`.
4. Update every test helper that walks New Game (`crates/ui/tests/it/flow.rs`
   `new_game`, the `xtask` `frame-png` docs if they name the steps, the
   bots if they start a New Game through the screens) for the new screen.

## Acceptance criteria

- [ ] New Game shows the familiarity screen after the mode, before the
      lead; Back returns to the mode.
- [ ] The chosen level is saved with the campaign and loaded back; an old
      save loads as `Full`.
- [ ] With tips on: at `Full` the first-turn tip shows; at `Short` and
      `None` it doesn't.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `Campaign` with `tutorial` round-trips; an old save without it
  loads as `Full`.
- Snapshot / integration: the screen's view; a scripted New Game with each
  answer, checking the screen order and whether the start tip shows.

## Completion notes

*(Filled in by the session that completes the ticket.)*
