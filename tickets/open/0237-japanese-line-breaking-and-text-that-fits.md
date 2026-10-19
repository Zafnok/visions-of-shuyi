---
id: "0237"
title: Japanese line breaking, and screens whose text must fit
type: feature
milestone: M1 Engine
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0235", "0236", "engine#2302"]
nick_input: none
completed:
---

# 0237 — Japanese line breaking, and screens whose text must fit

## Context

After 0236 Japanese glyphs draw, two cells each. But wrapping breaks at
spaces, which Japanese doesn't have, and some layouts were sized for an
English word. [ADR-0045 §4](../../docs/adr/0045-languages-text-by-key-and-line-ids.md)
sets the rule: break between any two characters, except before closing
punctuation and after opening brackets.

**The engine does the breaking: engine#2302** (in
[Shuyi Engine's tickets](https://github.com/Zafnok/shuyi-engine/tree/main/tickets)).
Since the carve (1109, 1112) the one wrapping function (`word_wrap`, the
game's `crate::widgets::word_wrap`), the dialogue text box's paging and
`lang-status` are the engine's. 2302 gives `word_wrap` the Japanese
breaking rules (measured in cells), keeps a key name filled into a
placeholder in one piece, lets the source language's `ui.ron` give a
key the width of its place (`max_cells`), and adds an **overflow** list
to `lang-status`. This ticket is the game's half: the game's own
wrapping routines, the widths of its places and its layouts.

## Nick input

None.

## Scope

**In:** every place the game wraps text using the engine's `word_wrap`;
the widths of the game's fixed-width places in `en/ui.ron`; the game's
length limits for pack text in cells; layouts sized for one English
word; moving the engine pin to a commit with engine#2302.

**Out (do not do):** the engine's half (engine#2302); translating
(0719); new layouts for Japanese (if a place can't fit, report it; a
redesign is its own ticket); furigana; vertical text; Chinese
punctuation and its wide quotes (0239).

## Implementation steps

1. Move the pin to an engine `main` that has engine#2302 (`cargo xtask
   engine-pin <commit>`, the `engine-change` skill §7).
2. Find every wrapping routine the game still has of its own in
   `crates/ui/src` (`screens/battle/glyph/rewind.rs`'s `wrap`,
   `screens/playdate.rs`'s `wrap`, and any other) and replace each with
   `word_wrap`, keeping what it does beyond wrapping (an indent, the
   text's own line breaks).
3. The game's length limits for pack text (0235 step 7) in cells, with
   `text_width`: a dialogue line at most 200 cells, a reply 60, a `lead:`
   line 40, tips 3 lines of 60 (the game's `LIMITS` in
   `crates/content/src/dialogue.rs`, and its tip and lesson limits).
4. Give every `assets/lang/en/ui.ron` key that is drawn in a fixed-width
   place its `max_cells`, set from the layout, so `cargo shuyi
   lang-status <code>` lists the pack entries that overflow.
5. Fix any layout 0234 or 0243 noted as sized for one English word,
   where the fix is measuring instead of a constant.

## Acceptance criteria

- [ ] A Japanese line from the test pack pages in the dialogue box at
      the same 3 lines, with no line starting with `。` or `、` (Harness
      test reading the view).
- [ ] English wrapping is unchanged (existing snapshots).
- [ ] No wrapping routine of the game's own is left in `crates/ui/src`.
- [ ] `cargo shuyi lang-status test` reports an entry made too long on
      purpose as overflow (test).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the replaced routines keep their indents and line breaks.
- Snapshot / integration: the dialogue screen showing a Japanese line
  from the test pack.

## Completion notes
