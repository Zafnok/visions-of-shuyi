---
id: "0235"
title: "Language packs cover data names, tips and dialogue lines"
type: feature
milestone: M1 Engine
model: opus-5.5
effort: high
status: done
blocked_by: ["0233", "0717"]
nick_input: none
completed: 2026-10-04
---

# 0235 — Language packs cover data names, tips and dialogue lines

## Context

After 0233/0234 a language pack can replace screen text. The rest of what
a player reads comes from data: `name` and description fields in
`assets/data/*.ron`, the names table, tips, captions and dialogue.
[ADR-0045 §2](../../docs/adr/0045-languages-text-by-key-and-line-ids.md)
gives each a key; 0717 gave dialogue lines ids. English stays in the data
files; a pack overlays it.

## Nick input

None.

## Scope

**In:** `data.ron` and `dialogue/<file>.ron` in a pack; lookups for every
data text; the dialogue screen and tips showing the pack's text; the lead
tokens in another language; the test pack extended.

**Out (do not do):** any Japanese text (0719); wide glyphs and Japanese
wrapping (0236, 0237); the credits' work titles and authors (names of
works stay as they are); changing any English.

## Implementation steps

1. List every player-facing text field in `crates/content/src` (items,
   classes, skills, spells, arts, terrain, characters' titles if any,
   tips, `names.ron`, battle and chapter titles, captions). Put the list
   in `assets/lang/README.md` with each one's key form:
   `<file>.<id>.<field>`, `names.<id>`, `tips.<id>.title|text`,
   dialogue line id, `caption.<scene>_<8 hex>` for `@caption` text
   (hash as in 0717).
2. `trpg_content::lang`: load `data.ron` and `dialogue/*.ron` entries;
   the same unknown-key, stale and missing rules as 0233. `Lang::status`
   includes them.
3. Lookups in `content`, taking a `LangCode`: one function per kind
   (`lang.item_name(code, &ItemDef)`, `lang.line(code, &LineId)`, …) so
   callers can't build keys by hand. `ui` reaches them through `Ctx`
   helpers next to `Ctx::text`.
4. Replace every place `ui` reads `.name` (and descriptions) directly for
   display with the helper. `core` keeps using ids only; where `core`
   returns a name for display today, return the id and let `ui` look it
   up (ADR-0004: `core` knows no language).
5. **Name tokens** in a translated line resolve against the pack's
   `names.<id>` (falling back to English). `{N:…}` capitalises only when
   the result starts with a Latin letter.
6. **Lead tokens.** `{lead}` works in any language. A translated line may
   not need `{they}`-style tokens; they stay allowed (they give the
   English words). For languages where the lead's gender changes more
   than a pronoun, an entry may carry `text_m` and `text_f` instead of
   `text`; the dialogue screen picks by the lead's gender. Validate that
   an entry has either `text` or both.
7. Validation of a pack's dialogue text: the same tokens rule as English
   (only known tokens; every `{n:…}` id exists), and the length limits
   counted in cells via one `text_width` function (today 1 per char;
   0236 makes it glyph-aware). The plain-ASCII and no-written-names
   rules apply to English only.
8. Extend the test pack with a few data names, a tip and one scene of
   `assets/dialogue/test.dlg`; Harness tests show them.
9. `cargo xtask lang-status` covers the new kinds and, for an orphaned
   dialogue entry, prints the new line in the same scene whose text is
   nearest, with the old translation.

## Acceptance criteria

- [x] In the test pack's language, an item name, a class name, a tip and a dialogue line show translated; everything else shows English (Harness tests).
- [x] A name token inside a translated line uses the pack's name (test).
- [x] An entry with `text_m`/`text_f` shows the right one for each lead (test).
- [x] Rewording an English line makes its entry an orphan that `lang-status` pairs with the new line (test).
- [x] No `ui` code reads a data `name` field for display except through the helpers (a `check-text` rule or a grep test).
- [x] English snapshots are unchanged.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: loading and validation per kind; token resolution; gender variants.
- Snapshot / integration: dialogue and an info screen in the test pack.

## Completion notes

**Done.** A language pack can now replace everything a player reads that
comes from data: names of classes, items, spells, skills, Combat Arts and
terrain, the names table, tips, chapter titles, battle notes, and every
dialogue line, reply and caption. `assets/lang/README.md` lists every key.

- `trpg_content::lang` loads a pack's optional `data.ron` and
  `dialogue/*.ron`, with one lookup per kind (`lang.item_name`,
  `lang.line`, …). Each lookup is handed today's English and gives the
  pack's text only when its entry was made from exactly that, so stale
  text shows in English.
- `ui` shows data text through `ctx.words()` (`crates/ui/src/words.rs`);
  no screen reads a `name` field any more, and `cargo xtask check-text`
  fails on one that does.
- Name tokens in a translated line use the pack's names; `{N:…}` only
  capitalises a name that starts with a Latin letter. A dialogue entry may
  have `text_m` and `text_f` instead of `text`.
- `cargo xtask lang-status` covers the new kinds and lists orphaned
  dialogue entries beside the nearest untranslated line of their scene.
- The test pack (`assets/lang/test/`) has a few names, a tip, a chapter
  title, a battle note and some lines of the test scene, with a stale
  entry and an orphan kept on purpose.

**Deviations from the plan**

- *Step 3:* an item's lookup takes its id and name (`ItemDef` has no id);
  terrain and battle notes are found by their English, because a battle
  in play knows neither the terrain's string id nor which file it came
  from.
- *Step 4:* `core` keeps its English `name` fields (units, classes,
  items…) and saves keep them: taking them out would have changed the save
  format and every bot and play record. `core` still decides nothing about
  what is shown; `ui` looks the name up by id (a named character by
  `names.<id>`, a generic unit by its class, the lead by the player).
- The battle's own menus (weapons, arts, spells, skills, items, units)
  are still built in English by the mode machine, which has no language
  and some 250 direct test calls; the screen paints them in the player's
  language (`Mode::told`). A command applied between frames (a combat's
  fighters, a level-up's skills, the rewind list) uses the language of the
  last frame, which the battle screen keeps.
- *Step 7:* a pack's text keeps the limits of text boxes (200 cells) and
  replies (60). English's 40-character limit on the lead's own lines is
  not applied to packs: it is a rule for writing English.
- An unknown key in `data.ron` is an error (as for `ui.ron`); an unknown
  line id in `dialogue/` is an orphan and never an error, so rewording
  English can't fail a gate.
- ADR-0045 was annotated in place (*0235* notes), as 0233 and 0717 did.

**Not translated (out of scope, listed in the README):** the two-letter
map labels, the credits, text still written as literals in `ui` (0234),
and voice clips, which stay in the language of their manifest.

**Follow-up ticket:** 0243 (`lang-status` leaves out test-only text and
can be narrowed to one `.dlg` file; 0719 now waits for it).

**For Nick:** nothing to decide and nothing changes in English. No
gameplay rule was decided here.
