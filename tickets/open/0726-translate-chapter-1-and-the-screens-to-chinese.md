---
id: "0726"
title: Translate Chapter 1 and the screens to Chinese (Simplified)
type: content
milestone: M6 Story & dialogue
model: fable-5.1
effort: high
status: todo
blocked_by: ["0234", "0239", "0243", "0716", "0725", "0825"]
nick_input: none
completed:
---

# 0726 — Translate Chapter 1 and the screens to Chinese (Simplified)

## Context

The first real use of the Chinese pack (`docs/design/voices-languages-and-script.md`,
[ADR-0045](../../docs/adr/0045-languages-text-by-key-and-line-ids.md)):
everything a player reads through Chapter 1, machine-translated with
0725's pipeline. 0716 finishes Chapter 1's English scenes, so this waits
for it. It is the Chinese twin of 0719 and doesn't depend on it.

## Nick input

None. Nothing waits on a person. If 0045 recorded that Nick's wife plays
Chapter 1 in Chinese, that happens after this merges (on the Pages
build), and what she finds becomes tickets through `write-ticket`.

## Scope

**In:** `assets/lang/zhhans/ui.ron` (every key), `data.ron` (every name,
description and tip the game shows through Chapter 1, and all of
`names.ron`), `dialogue/ch01.ron` (every line of `ch01.dlg`); the
regenerated atlas.

**Out (do not do):** changing English text to make translating easier
(write a ticket if an English line is the problem); translating from the
Japanese pack; test scenes (`test*.dlg`); later chapters (one ticket
each, when written); code.

If it is too big for one PR, split into screens-and-data (first) and
dialogue (second) as two tickets.

## Implementation steps

1. Follow `docs/story/zh/README.md` for every file. Use the glossary's
   term for every game term; add a missing term to the glossary first.
2. Screens and data: translate `ui.ron` and `data.ron`. Where
   `lang-status` reports overflow, shorten the Chinese (it is usually
   shorter than English already); if a place can't fit any honest
   translation, write a ticket for the layout and leave that key in
   English.
3. Dialogue: scene by scene, with the speakers' style sheets open. Keep
   each line one text box, as the English is.
4. Run the back-translation check in a separate session (or a clearly
   separate step with a fresh subagent that never sees the English).
   Fix what it finds; list what was changed in the completion notes.
5. Regenerate the atlas (`cargo xtask font-atlas …`), so every character
   used is in the Chinese set. A character the font lacks is reworded
   around, never borrowed from the Japanese set.
6. Play Chapter 1's scenes in Chinese through the Harness or
   `frame-png` and look at five screens for clipped or overlapping text.

## Acceptance criteria

- [ ] `cargo xtask lang-status zhhans` reports nothing missing, stale or overflowing for `ui.ron`, `names.*`, the data Chapter 1 shows and `ch01`.
- [ ] The back-translation pass has run and its findings are resolved or listed.
- [ ] The committed atlas is current (the stale-atlas test passes).
- [ ] Five `frame-png` screenshots in Chinese are attached to the PR (public placeholder art only, ADR-0040).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The all-assets test validates the pack. A Harness test plays `ch01_intro` in Chinese to its end without a missing glyph.

## Completion notes

