---
id: "0237"
title: "Line breaking for Chinese and Japanese, and screens whose text must fit"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0235", "0236"]
nick_input: none
completed: 2026-11-25
---

# 0237 — Line breaking for Chinese and Japanese, and screens whose text must fit

## Context

After 0236 wide glyphs draw, two cells each (Chinese first, then
Japanese). But wrapping breaks at spaces, which neither language has, and some layouts were sized for an
English word. [ADR-0045 §4](../../docs/adr/0045-languages-text-by-key-and-line-ids.md)
sets the rule: break between any two characters, except before closing
punctuation and after opening brackets.

**The engine does the breaking: engine#2302** (in
[Shuyi Engine's tickets](https://github.com/Zafnok/shuyi-engine/tree/main/tickets)).
Since the carve (1109, 1112) the one wrapping function (`word_wrap`, the
game's `crate::widgets::word_wrap`), the dialogue text box's paging and
`lang-status` are the engine's. 2302 gives `word_wrap` the breaking
rules for text without spaces (measured in cells), keeps a key name filled into a
placeholder in one piece, lets the source language's `ui.ron` give a
key the width of its place (`max_cells`), and adds an **overflow** list
to `lang-status`. This ticket is the game's half: the game's own
wrapping routines, the widths of its places and its layouts.

2026-11-17 friction sweep
([`docs/research/friction-2026-11-17.md`](../../docs/research/friction-2026-11-17.md),
bucket B1): engine#2302 is in the engine's `tickets/done/` at the
game's pin (`e738ffc`), so it is off `blocked_by`. What still blocks
this is 0236, which no longer waits on Nick's 0042: since 0045
(2026-11-24, Chinese first) its first wide font can be the Chinese
one.

## Nick input

None.

## Scope

**In:** every place the game wraps text using the engine's `word_wrap`;
the widths of the game's fixed-width places in `en/ui.ron`; the game's
length limits for pack text in cells; layouts sized for one English
word; ~~moving the engine pin to a commit with engine#2302~~ (done by the pin move of 2026-11-05).

**Out (do not do):** the engine's half (engine#2302); translating
(0719, 0726); new layouts for either language (if a place can't fit,
report it; a redesign is its own ticket); furigana; vertical text;
Chinese punctuation, its wide quotes and its own no-break rules
(0239).

## Implementation steps

1. ~~Move the pin to an engine `main` that has engine#2302 (`cargo xtask
   engine-pin <commit>`, the `engine-change` skill §7).~~ (done by the pin move of 2026-11-05).
2. Find every wrapping routine the game still has of its own in
   `crates/ui/src` (`screens/battle/glyph/rewind.rs`'s `wrap`,
   `screens/playdate.rs`'s `wrap`, and any other) and replace each with
   `word_wrap`, keeping what it does beyond wrapping (an indent, the
   text's own line breaks). Every `word_wrap` call passes the widths it
   draws with (`buf.widths()`, or `ctx.widths()` where there is no
   buffer), not `CellWidths::NARROW`: 0236 left the wrapping calls narrow
   for this ticket (`grep -rn "CellWidths::NARROW" crates/ui/src`, the
   tests aside).
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

- [x] A Chinese line from the test pack (in 0236's wide font) pages in
      the dialogue box at the same 3 lines, with no line starting with
      `。` or `、` (Harness test reading the view).
- [x] English wrapping is unchanged (existing snapshots).
- [x] No wrapping routine of the game's own is left in `crates/ui/src`.
- [x] `cargo shuyi lang-status test` reports an entry made too long on
      purpose as overflow (test).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the replaced routines keep their indents and line breaks.
- Snapshot / integration: the dialogue screen showing a Chinese line
  from the test pack.

## Completion notes

Done 2026-11-25.

**Wrapping.** Every wrap in `crates/ui/src` now goes through the
engine's `word_wrap` (engine#2302's rules: Chinese and Japanese break
between characters, never before `。` `、` `，` and the like, never after
an opening bracket), measured in the language's cells:

- `widgets::wrap_indented` (new): `word_wrap` with every line after the
  first indented. The rewind list's `wrap`/`wrap_to` (its own space
  splitter until now) is built on it.
- `widgets::help::wrap_styled` (the tips' and hints' styled wrap, its
  own space splitter) now hands `word_wrap` the plain text with each key
  span as an unbreakable run of its width (a cap's extra cells
  included), then puts the spans back. One change: a key wider than the
  row still has a row of its own; a plain word wider than the row is now
  cut to the row, as `word_wrap` cuts it, where it ran past before. The
  rewind list's over-long word likewise.
- Measured with the real widths, not `CellWidths::NARROW`: the dialogue
  screen's pages (`Ctx::widths`, kept on the screen and set by
  `wrapped_for`), the cinematic talk box, the dialogue log (both
  skins), the battle notes (both skins), and the Playdate's `wrap` and
  every caller (the battle's terrain preview, notes and warnings, the
  question window, the mode select). The Playdate's `unwrap_notes`
  rejoins two wrapped lines with no space where both sides are wide, so
  a Chinese note doesn't gain one.
- Left as it is: the credits list still wraps narrow. The code moves to
  the engine (engine#3201, game 1211); a dated line in 1211's Context
  says the engine screen should take `ctx.widths()`.

**Limits in cells.** 0235 had already moved the dialogue, reply, lead
line, tip and lesson limits to cells (the engine's `DialogueLimits`
checks and `crate::font::text_width`). Left were the battle notes: the
120 limit is now in cells, for the battle files and for a pack's
`battles.<id>.note_<n>` entries.

**`max_cells`.** Given from the layout to the `en/ui.ron` keys drawn in a
fixed width (with a note for translators): `results.exp` (3),
`results.level_up` (9), `save.column.slot`/`chapter`/`mode` (5/48/9),
`stat.hp` (2: the forecast's column) and the other stat names (3: the
info screen's column, before the value), `battle.forecast.hit`/`crit`/
`no_counter`/`total` (3/4/11/5). Menus, help bars and panels sized to
their text aren't fixed places and have none. The test pack's
`results.exp` is `EXPERIENCE` on purpose: `lang-status test` reports it
too long (10 cells; the place has 3).

**Layouts from 0234 and 0243's lists.** Fixed where measuring was the
fix: the three labels centered by their length in bytes (the info
screen's and the level-up page's portrait placeholders, the info
screen's spell uses) now use cells. The rest are columns, not
measurements, and stay as they are, now covered by `max_cells` where
they hold one key: the forecast's and results' columns, the info
screen's stat column, the save list's columns. Not done (a redesign,
not measuring): the key bindings' `SLOT_W`/`CHOICE_W`, lead select's
name-grid box, the layout picker's legend column, the class change
screen's columns, the list lines with numbers glued to labels, the help
bar sharing its row with the toggles.

**The test line.** The test pack's Chinese narration
(`test_d7fb4eb3`, "The knight stamps off into the rain.") is made of the
only Chinese characters the wide font has (`title.crashed`'s), so it
reads as nonsense; adding characters means rebuilding the atlas. Cut
every 46 characters, its second and third lines would start with `。`;
the box shows `告。` there instead.

No gameplay rule was decided; nothing for Nick to check.
