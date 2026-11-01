---
id: "0475"
title: Vix's lines come apart from the objectives, with no key in what she says
type: content
milestone: M3 Battle UI
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0043"]
nick_input: none
completed:
---

# 0475 — Vix's lines come apart from the objectives, with no key in what she says

## Context

Vix is voiced (0043, `docs/design/voices-languages-and-script.md`, *Voices*).
Nick, 2026-11-01: "for the button tutorials they don't need voices, but make
sure Vix has lines throughout tutorial, so a clear delineation between <Vix's
line> <tutorial objectives> should be made if it's not already entangled..."

A voice clip is made ahead of time and can't say a key the player rebound. Today
a lesson (`assets/data/lessons.ron`) has `text` and `items`, which is nearly the
split he asks for, but `text` often names keys ("Press {NextUnit} for anyone
who hasn't acted"). A tip (`assets/data/tips.ron`) has one `text` that mixes her
talk and the keys, and no list.

## Nick input

None. The script is the agent's (`story-writing` skill). What is left to judge
by playing goes on the Act 1 playtest's checklist (1064).

## Scope

**In:** the data split for lessons and tips; their new wording, in Vix's voice
card; the screens showing both parts; the check that keeps keys out of her
lines; the *Guides* list (0458) showing the same text.

**Out (do not do):** playing any voice (0476); making clips (0476); new lessons
or tips; changing when any lesson or tip appears.

## Implementation steps

1. Decide the fields by reading `crates/content/src/lesson.rs` and the tips'
   loader. A lesson's `text` becomes her line, and its `items` stay the
   objectives. A tip gains an optional `how` of one or more lines of plain
   instruction, beside its `text`. Follow `shuyi-screen`: the view gives the
   two parts apart, and the skin paints them.
2. Add a content check: a lesson's or tip's `text` may hold no `{Action}`,
   `{Cursor}` or `{Select}` placeholder. Keys go in `items` or `how`.
3. Rewrite every lesson and tip so that each has a line for her to say, which
   teaches something on its own. Keep the limits in `lessons.ron`'s header and
   her voice card (`docs/story/characters/guide.md`): short, dry, says what to
   do and stops. Every rule a line taught must still be taught by the line or
   by the objectives.
4. Update `docs/design/tutorial.md` with the rule and `lessons.ron`'s header
   comment with the new fields.

## Acceptance criteria

- [ ] No lesson or tip `text` in `assets/` holds a key placeholder, and the content check fails on one that does (test).
- [ ] Every lesson and tip has a non-empty `text`.
- [ ] Each key named by the old wording is still named in `items` or `how` (compare in the notes).
- [ ] A frame picture of a lesson and of a tip with `how` is in the PR (`cargo shuyi frame-png`), and neither overflows its box.
- [ ] Existing lesson and tip tests pass, with snapshots updated on purpose.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the content check.
- Snapshot: a lesson box and a tip box with both parts.

## Completion notes

