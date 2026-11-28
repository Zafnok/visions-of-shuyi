---
id: "1418"
title: "The game's one-question screens and its first-launch language list on the engine's ready-made screens"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#3257"]
nick_input: none
completed:
---

# 1418 — Question and language screens from the engine

## Context

The third engine-friction sweep (1397) found that five game screens each
ask one question on a screen of their own:
- the voices question;
- the mode select;
- the familiarity question;
- the save prompt;
- game over.

They share a `pick_one` frame helper (`screens/mod.rs`) and borrow the
save prompt's `SavePromptView` and painter. The first-launch language
list (`language_list.rs`) copies the engine's lookup of the language in
use. It only builds its own names so that a machine-made pack shows its
note, and options builds its own Language row for the same reason.

That is about 220 code lines against about 75. Engine ticket 3257 adds
a full-screen `QuestionScreen<T>`, a `LanguageScreen`, and the note on
the engine's Language row.

## Nick input

None.

## Scope

**In:**
- Move the pin past engine 3257 (`engine-change` skill, §7) if it isn't
  there yet.
- `voices_question.rs` becomes the engine's `QuestionScreen<bool>` with
  the game's keys.
- `mode_select.rs`, `familiarity.rs`, the save prompt in `save.rs` and
  `game_over.rs` use `QuestionScreen` where all they do is ask a
  question. Each keeps whatever else it shows, such as game over's text.
- `language_list.rs` keeps `asks`, or uses the engine's if it is the
  same rule. Its screen, `names` and `in_use` go.
- `options.rs`: `Row::Language` uses the engine's `OptionRow::language()`.
- `pick_one` in `screens/mod.rs` goes once nothing calls it, and so does
  `SavePromptView`/`paint_prompt`.
- The one-bit skins stay the game's, passed with `.painted_by`.

**Out (do not do):**
- Any change the player sees: same text, same answers, same focus, same
  look, PC and Playdate.

## Implementation steps

1. Write the engine-friction estimate first: about 200 lines removed and
   50 added.
2. Move the pin and replace the screens one at a time, each with its
   snapshots unchanged.
3. Delete the marked code (`// engine-friction: engine#3257`).

## Acceptance criteria

- [ ] `grep -rn "engine#3257" crates` finds nothing.
- [ ] Every snapshot of these six screens, PC and Playdate, is unchanged.
- [ ] Each question still pops with the same value for each answer; the
      existing tests prove it.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot/unit: the existing ones.

## Completion notes

*(Filled in by the session that completes the ticket.)*
