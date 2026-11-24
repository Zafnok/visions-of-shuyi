---
id: "0236"
title: "Wide glyphs: the first wide font in the atlas, two cells per glyph"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: []
nick_input: none
completed: 2026-11-24
---

# 0236 — Wide glyphs: the first wide font in the atlas, two cells per glyph

## Context

The font is Terminus 8×16 and a cell holds one 8×16 `char` (ADR-0016).
Chinese and Japanese glyphs are 16×16. [ADR-0045 §4](../../docs/adr/0045-languages-text-by-key-and-line-ids.md):
the atlas gains 16×16 glyphs from a second font, and a wide glyph takes
two cells. This ticket makes wide text *drawable*, for whichever
language comes first; 0237 makes it wrap and fit, and 0239 adds what
only Chinese needs.

**The font** (Nick, 2026-11-24, 0045: "lets do chinese first"): the
Chinese default, **Fusion Pixel 12px** (the monospaced `zh_hans` BDF of
release 2026.11.10, its 12×12 glyph drawn in the 16×16 space;
`docs/design/voices-languages-and-script.md`, *Chinese (Simplified)*),
as the wide set `zhhans`. Its licence and placement are in 0239's
context. The Japanese font is 0042's, still open: if 0042 has chosen it
by the time this is worked, add it too, as the set `ja`; if not, 0239
or 0042's follow-up adds it.

**The engine does the drawing: engine#2301** (in
[Shuyi Engine's tickets](https://github.com/Zafnok/shuyi-engine/tree/main/tickets)).
Since the carve (1104, 1108, 1117) the atlas format, the `font-atlas`
tool, `GlyphBuffer`, `text_width`, the renderers and `frame-png` are the
engine's, so 2301 gives them wide glyphs: a wide font beside the narrow
one in the atlas (`font-atlas --wide <font.bdf>`, the characters the
packs use plus a set the project file names), a wide character in two
buffer cells, drawn as one 16×16 glyph, and `text_width` counting it as
two. This ticket is the game's half: the font, its licence, the game's
atlas and the game's own layout code. engine#2301 shipped as engine
#264 and the pin move of 2026-11-04 reached it.

## Nick input

None (the font is 0045's answer; Japanese's is 0042's).

## Scope

**In:** the font's BDF in `assets-src/fonts/` and its licence; the
game's atlas regenerated with it; the set of wide glyphs the game always
needs; the game's layout code measuring with `text_width`.

**Out (do not do):** the engine's half (engine#2301); line breaking and
length limits (0237); translating (0719, 0726); Chinese punctuation,
Chinese line breaking and a set per language for shared characters
(0239); the second Chinese font (0261); changing any Latin glyph. A
font that ships only as TTF is converted to BDF at 16 px here (a one-off, documented in
`assets/fonts/README.md`; the BDF is what is committed).

## Implementation steps

1. ~~Move the pin to an engine `main` that has engine#2301~~ (done by the
   pin move of 2026-11-04). That move kept the game narrow with two
   shims that measure with `CellWidths::NARROW`:
   `trpg_content::lang::text_width` (`crates/content/src/lang.rs`) and
   `trpg_ui::screens::playdate::cut` (`crates/ui/src/screens/playdate.rs`).
   Delete both: the content checks measure with the language's widths,
   the screens pass `buf.widths()` to the engine's `cut`.
2. Add the font's BDF and licence text (`assets-src/fonts/`,
   `assets/fonts/<Name>-LICENSE.txt`), a row in `THIRD_PARTY_ASSETS.md`
   and a section in `assets/fonts/README.md`.
3. `shuyi.ron`'s `font_atlas`: the wide font and the set the game always
   needs whatever the packs say (the CJK punctuation block and the
   full-width forms; hiragana and katakana with a Japanese font), so
   tests don't depend on a translation. Regenerate the atlas with `cargo shuyi font-atlas
   --wide …` and commit it. A test fails when a pack gains a wide
   character the committed atlas lacks (engine#2301's staleness check
   on the game's packs).
4. The game's screens: replace `chars().count()` / `len()` used for
   layout in `crates/ui/src` (centring, right-aligning, clipping) with
   `text_width`, and give the game's buffers the atlas's widths where it
   makes them.
5. Text snapshots: a continuation cell prints as nothing (engine#2301),
   so a snapshot row with wide text reads naturally.
6. A screenshot for the PR: the test pack (0233) with a few Chinese
   characters on the title screen, via `cargo shuyi frame-png`.

## Acceptance criteria

- [x] The title screen shows Chinese text from the test pack, two cells
      per character (a game frame PNG test against a committed
      reference, or a snapshot reading cells).
- [x] The committed atlas has exactly the wide characters the packs use
      plus the game's set, and a test fails when a pack gains a new one.
- [x] Every existing snapshot and the narrow atlas cells are unchanged.
- [x] The font's licence is shipped and listed.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the game's layout helpers with wide text.
- Snapshot / integration: a frame PNG or snapshot with Chinese text;
  the look test reads cells, as ADR-0038 allows for tests of a look.

## Completion notes

**What was done.**
- **The font:** Fusion Pixel 12px, the release 2026.11.10's monospaced
  `zh_hans` BDF, in `assets-src/fonts/` with one line changed: its
  `FONTBOUNDINGBOX` is `16 16 -2 -4` (was `12 12 0 -2`), so `font-atlas
  --wide` reads it at the wide cell's size (it requires twice Terminus's
  8×16) and places each 12×12 glyph 2 px from the left, on Terminus's
  baseline, 2 px clear above and below: mockup C's placement. No glyph
  changed; a comment at the top of the file says so. Licence:
  `assets/fonts/FusionPixel-LICENSE.txt` (the release's `OFL.txt` and its
  `LICENSES/` texts), a row in `THIRD_PARTY_ASSETS.md`, `NOTICES-game.txt`
  and `shuyi.ron`'s `notices`, a credit in `assets/data/credits.ron`, a
  section in `assets/fonts/README.md`.
- **The atlas:** the named wide set `zhhans` (engine#2303 was already at
  the pin), regenerated with `--wide zhhans=…`. `shuyi.ron`'s
  `font_atlas.wide_set_glyphs.zhhans` keeps the CJK punctuation block
  (U+3000–U+3029, U+3030–U+303F; not its combining tone marks) and the
  full-width forms (U+FF01–U+FF60, U+FFE0–U+FFE6) whatever the packs say:
  174 wide glyphs with the test pack's. The narrow glyphs keep their
  cells; the bold face's glyphs moved after the wide set (same pixels,
  new cell numbers, as the tool lays them out). The stale-atlas test
  (`committed_atlas_is_up_to_date`) builds with the wide set, so a pack
  that gains a character the committed atlas lacks fails it.
- **The test pack** names the set (`wide_font: Some("zhhans")`, else the
  tool keeps its characters in no set) and its `title.crashed` is in
  Chinese, the one line of it not in English capitals; no test read that
  line before.
- **Step 1, the shims.** `trpg_ui::screens::playdate::cut` is the
  engine's `cut` re-exported, every caller passing `buf.widths()`; the
  Playdate's `cells` takes the widths too. `trpg_content::lang::text_width`
  is gone: the content checks (tips', lessons' and weapons' short names'
  lengths, the packs' `PackRules`, the tips' `TipRules`, the cinematics'
  text-box check) measure with the embedded atlas's widths,
  `trpg_content::font::widths()` / `font::text_width`, read once
  (`font::wide_chars`, a `OnceBox`). `lang::RULES` (a `const`, which a
  `CellWidths` can't be) became `lang::rules(widths)` and
  `lang::VARIANTS`.
- **Step 4, the screens.** Every skin's layout count in `crates/ui/src`
  (centring, right-aligning, clipping, box sizes) measures in cells:
  `buf.text_cells`, `text_width(…, buf.widths())`, or `ctx.widths()`
  where a size is worked out before there is a buffer; `chars().take(n)`
  clipping became `screens::clip` (cells, never half a wide character, no
  `…`); the per-skin `cells`/`text_cells`/`width` helpers take the widths.
  `game::new`, `start`, `start_on_scene` give the `Game` the atlas's wide
  characters (`CellWidths::wide`), so a game a test drives prints wide as
  the window does (the platform gives the window `CellWidths::of` the
  atlas when the game loads).
- **Tests:** `title::chinese_text_takes_two_cells_a_character` (the
  title in the test pack with its Chinese notice: two cells a character,
  the band centred on its 30 cells, a snapshot of the row; continuation
  cells print as nothing); unit tests of `clip`, `place_line`,
  `weapon_name` and `font::text_width` with wide text.
- **The picture:** `docs/screenshots/0236-title-chinese.png`.

**Deviations.**
- Step 6 says `cargo shuyi frame-png`; it can't set the language or a
  crash, so the picture was drawn by the same painter (`Painter`, the
  game's `painted`) from the Harness of the test above, in a throwaway
  test not committed.
- The games the constructors make get `CellWidths::wide` (the wide
  characters), not `CellWidths::of` (which carries the atlas's ink boxes
  too): with the inks the Harness placed key caps differently and six key
  cap snapshots changed. The window still gets `of` from the platform.
- The content checks measure with the atlas's widths, not each
  language's: the pack's `data_problem` is a plain `fn(key, text)` with
  no pack, and no pack yet has wide punctuation. A language's
  punctuation widths are 0239's.
- Left for other tickets: the wrapping calls that still pass
  `CellWidths::NARROW` (`word_wrap` in the credits, the dialogue box and
  its log, the battle notes, the Playdate's `wrap`, the cinematic's talk;
  `rewind.rs`'s and `preparations/glyph.rs`'s own wraps) are 0237's,
  which now says so; the battle menus that pad a name column with
  `format!` (items, magic, skills, the unit list, the objective's labels)
  are **0263** (new). Counts that aren't layout stay character counts:
  the typewriter's reveal and the dialogue's timing, a stored name's
  length (`save_files/import.rs`), the writer's preview's token column.

- **Workaround: the Playdate prints narrow.** With the wide set, the
  engine's `GlyphBuffer` looks every printed character up in it
  (`CellWidths::is_wide`, a `BTreeSet`): CI's bench put the Playdate's
  `battle-cursor-sweep` frame at 2,069,579 instructions, over its
  2,000,000 budget (1,983,326 on `main`). `game::with_atlas_widths` skips
  the Playdate's console, which keeps it as it was on `main` (it never got
  widths: it boots without the platform's `App`). The fix is an engine
  fast path (`is_wide` answering a character below the set's first at
  once); its engine ticket waits for Nick's OK. No Chinese is shown on the
  Playdate yet.

**Friction:** 2.4× (estimate 200 production lines, measured 477),
accepted: most of it is rustfmt rewrapping about 90 calls that gained a
widths argument (a row in `docs/engine/friction.md`).

**Follow-up tickets:** 0263 (battle menus pad their name columns in
cells); 0237 gained the narrow `word_wrap` calls.

**For the Act 1 playtest (1064):** nothing; no player sees Chinese yet.

