---
id: "0236"
title: "Wide glyphs: the Japanese font in the atlas, two cells per glyph"
type: feature
milestone: M1 Engine
model: reasoning
effort: medium
status: todo
blocked_by: ["0042", "engine#2301"]
nick_input: none
completed:
---

# 0236 — Wide glyphs: the Japanese font in the atlas, two cells per glyph

## Context

The font is Terminus 8×16 and a cell holds one 8×16 `char` (ADR-0016).
Japanese glyphs are 16×16. [ADR-0045 §4](../../docs/adr/0045-languages-text-by-key-and-line-ids.md):
the atlas gains 16×16 glyphs from the font Nick chose in 0042, and a wide
glyph takes two cells. This ticket makes Japanese text *drawable*; 0237
makes it wrap and fit.

**The engine does the drawing: engine#2301** (in
[Shuyi Engine's tickets](https://github.com/Zafnok/shuyi-engine/tree/main/tickets)).
Since the carve (1104, 1108, 1117) the atlas format, the `font-atlas`
tool, `GlyphBuffer`, `text_width`, the renderers and `frame-png` are the
engine's, so 2301 gives them wide glyphs: a wide font beside the narrow
one in the atlas (`font-atlas --wide <font.bdf>`, the characters the
packs use plus a set the project file names), a wide character in two
buffer cells, drawn as one 16×16 glyph, and `text_width` counting it as
two. This ticket is the game's half: the font, its licence, the game's
atlas and the game's own layout code.

## Nick input

None (the font is 0042's answer).

## Scope

**In:** the font's BDF in `assets-src/fonts/` and its licence; the
game's atlas regenerated with it; the set of wide glyphs the game always
needs; the game's layout code measuring with `text_width`; moving the
engine pin to a commit with engine#2301.

**Out (do not do):** the engine's half (engine#2301); line breaking and
length limits (0237); translating (0719); a second wide font for Chinese
and its own shapes for shared characters (0239); changing any Latin
glyph. If 0042 chose a font that ships only as TTF, converting it to BDF
at 16 px is part of this ticket (a one-off, documented in
`assets/fonts/README.md`; the BDF is what is committed).

## Implementation steps

1. Move the pin to an engine `main` that has engine#2301 (`cargo xtask
   engine-pin <commit>`, the `engine-change` skill §7), and fix what the
   new API needs (`text_width` takes the atlas's widths).
2. Add the font's BDF and licence text (`assets-src/fonts/`,
   `assets/fonts/<Name>-LICENSE.txt`), a row in `THIRD_PARTY_ASSETS.md`
   and a section in `assets/fonts/README.md`.
3. `shuyi.ron`'s `font_atlas`: the wide font and the set the game always
   needs whatever the packs say (hiragana, katakana, the Japanese
   punctuation block), so name entry and tests don't depend on a
   translation. Regenerate the atlas with `cargo shuyi font-atlas
   --wide …` and commit it. A test fails when a pack gains a wide
   character the committed atlas lacks (engine#2301's staleness check
   on the game's packs).
4. The game's screens: replace `chars().count()` / `len()` used for
   layout in `crates/ui/src` (centring, right-aligning, clipping) with
   `text_width`, and give the game's buffers the atlas's widths where it
   makes them.
5. Text snapshots: a continuation cell prints as nothing (engine#2301),
   so a snapshot row with Japanese reads naturally.
6. A screenshot for the PR: the test pack (0233) with a few Japanese
   characters on the title screen, via `cargo shuyi frame-png`.

## Acceptance criteria

- [ ] The title screen shows Japanese text from the test pack, two cells
      per character (a game frame PNG test against a committed
      reference, or a snapshot reading cells).
- [ ] The committed atlas has exactly the wide characters the packs use
      plus the game's set, and a test fails when a pack gains a new kanji.
- [ ] Every existing snapshot and the narrow atlas cells are unchanged.
- [ ] The font's licence is shipped and listed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the game's layout helpers with wide text.
- Snapshot / integration: a frame PNG or snapshot with Japanese text;
  the look test reads cells, as ADR-0038 allows for tests of a look.

## Completion notes
