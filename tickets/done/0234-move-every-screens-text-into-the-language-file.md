---
id: "0234"
title: Move every screen's text into the language file
type: feature
milestone: M1 Engine
model: sonnet-5
effort: medium
status: done
blocked_by: ["0233"]
nick_input: none
completed: 2026-10-04
---

# 0234 — Move every screen's text into the language file

## Context

0233 built screen text by key ([ADR-0045](../../docs/adr/0045-languages-text-by-key-and-line-ids.md))
and converted the title screen. About 200 player-facing literals remain in
`crates/ui/src`. This ticket moves them, following the title screen's
pattern, so a language pack can cover the whole game.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:** every literal `cargo xtask check-text` lists, in every screen and
widget under `crates/ui/src` (battle, dialogue, results, save, class
change, key bindings, layout picker, lead select, mode select, game over,
credits, the flow screen, help bars, banners such as `PLAYER PHASE`).

**Out (do not do):** the debug menu and debug tools (English only, for
developers); names and descriptions from data files, tips and dialogue
(0235); rewording anything; layout changes. If moving a literal shows a
layout that only works for that English word's length, leave the layout
and note it for 0237.

If this is too big for one PR (over ~600 changed lines besides
`ui.ron`), split by screen group into two PRs of this same ticket's
pattern: write the second as a new ticket and lower the `check-text`
number in each.

## Implementation steps

1. Run `cargo xtask check-text` for the list.
2. Per screen: replace each literal with `ctx.text("<screen>.<thing>")`
   or `ctx.text_with(...)` for text with numbers or names in it, and add
   the key to `assets/lang/en/ui.ron`, grouped by screen with a comment
   line per group. One key per meaning: don't share a key between two
   places just because the English matches (`Cancel` the button and
   `Cancel` the action name may differ in Japanese).
3. Text built by joining pieces (`"Lv " + n`) becomes one key with a
   placeholder (`"Lv {level}"`), because word order differs by language.
4. Screens where new screens were added since 0233 (check `git log` for
   `crates/ui/src/screens`) are covered too.
5. Set the number in `check-text` to 0. What it lists that is not text
   for a translator (a picture drawn from rows of `X` and `.`, a
   controller button's name) gets a `// check-text: not player text`
   comment instead of a key.
6. Add a line to `crates/ui/src/screens/README` or the `work-ticket`
   skill's checklist, wherever new-screen rules live: new screens take
   their text from `ctx.text`.

## Acceptance criteria

- [ ] `cargo xtask check-text` reports 0 and its limit is 0. **Not met:** split as the Scope allows; it reports 123 and its limit is 123. Ticket 0243 brings it to 0.
- [x] No snapshot changes.
- [x] `cargo xtask lang-status test` still works (the test pack stays partial on purpose).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: existing snapshots unchanged is the test.

## Completion notes

Split in two, by the Scope's own rule: this ticket did every screen
outside the battle, and **0243** does the battle screen, the class change
screen and the words inside key names. Nothing a player sees changes; no
gameplay rule was decided here.

- Moved to `assets/lang/en/ui.ron` (about 150 new keys, grouped by
  screen): the layout picker, mode select, lead select and its name grid,
  the text box's help, the dialogue's skip question, the results tally,
  "Save your progress?", the save slots, game over, "To be continued",
  the credits, and the rest of the Key bindings screen.
- `check-text` went from 220 to 123; its limit is 123. All 123 are in
  `screens/battle/`, `screens/class_change.rs` (it shares its stat rows
  and result pages with the battle) and `widgets/help.rs` (`! not
  mapped`).
- `check-text` sees only about half of the text: it misses text built
  with `format!` first and printed later, help labels kept in a variable,
  and words returned from a `match`. In the screens done here those were
  moved too (the save errors, `1 unit` / `3 units`, `(cleared)`, the
  layout picker's legend, the credits' `"Title" by Author`, …). 0243
  lists what is left of that kind and teaches `check-text` to see it.
- Each help line is one key (`"{Cursor} choose · {Confirm} select"`), so
  a translation can reorder it. The question boxes' `f yes / d no` lines
  are their own keys, no longer a help line with its separator swapped.
- Marked `// check-text: not player text`: the pad's `D-pad` / `L-stick`
  / `R-stick`, the name grid's letter rows, and the portrait expression
  id `neutral`.
- Step 6: the new-screen rule was already in `crates/ui/README.md` (0233
  wrote it); nothing to add.

Deviations and details settled:

- Screens whose menu is built once take the `Ctx` in `new` (as the title
  does): `ModeSelectScreen`, `GameOverScreen`, `SavePromptScreen`, and so
  `FlowScreen::new_game`. Their `Default` impls are gone.
- `SaveError::text(ctx)` is what the player is told; its `Display` stays,
  in English, for the log.
- `SlotSummary` gained `cleared`, and `next_chapter_title` returns the
  title and that flag, so `(cleared)` is added in the player's language
  when the slot is drawn.
- English plurals are two keys (`save.army.one`, `save.army.many`).
- The Key bindings screen names keys from the keymap it was opened with,
  so it fills its help lines with `tips::fill_text` and that keymap
  rather than `ctx.text_with`.
- The test pack (`assets/lang/test/ui.ron`) got an entry for every new
  key: two tests pin it to exactly one missing and one stale entry.
- `check-keys` read `Press a key…` in the language file as an order to
  press the `a` key. It now lets `a key` and `a button` through (the
  article), and still flags `press a`, `press a to attack` and `Press A
  button`. The test pack writes `PRESS a KEY…` for the same reason.

Layouts sized for the English words, left as they are, for 0237:

- Results: the `Lv {level}`, `EXP` and `LEVEL UP` columns (`COLS`).
- Save slots: the `Mode`, `Army` and `Time` columns are fixed; a chapter
  title is cut at 48 cells.
- Key bindings: a slot is `SLOT_W` cells, which `Press a button…` just
  fits; the `Change` / `Clear` box is `CHOICE_W` wide.
- Lead select: the name grid's box fits `Blank`, `Delete`, `Done`.
- Layout picker: the legend's words have a fixed column.

Follow-up tickets: **0243**. 0719 and 0726 now wait for it too; 0445,
0826, 0241, 0237 and the roadmap point at it.
