---
id: "0239"
title: "Chinese glyphs: a wide font per language, wide punctuation, Chinese line breaking"
type: feature
milestone: M1 Engine
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0045", "0236", "0237", "engine#2303"]
nick_input: none
completed:
---

# 0239 — Chinese glyphs: a wide font per language, wide punctuation, Chinese line breaking

## Context

0236 puts one 16×16 font in the atlas for Japanese and 0237 wraps
Japanese text. Simplified Chinese (Nick, 2026-10-04,
`docs/design/voices-languages-and-script.md`) needs three more things
([ADR-0045 §4](../../docs/adr/0045-languages-text-by-key-and-line-ids.md)):

- **The same character is drawn differently in the two languages.**
  Unicode gives Japanese and Chinese one code point for a shared
  character (直, 骨, 角, 海…), and each language expects its own shape.
  A Japanese font also lacks most Simplified-only characters. So the
  atlas can't key a wide glyph by `char` alone: it needs a set of wide
  glyphs per font, and a language says which set it uses.
- **Some punctuation is narrow in English and wide in Chinese** with the
  same code point: `“ ” ‘ ’ … —`. Chinese text uses `“”` for speech and
  the doubled `……` and `——`.
- **Chinese has its own no-break punctuation**, and never splits `……`
  or `——`.

**The engine builds all three: engine#2303** (in
[Shuyi Engine's tickets](https://github.com/Zafnok/shuyi-engine/tree/main/tickets)).
The atlas format, the `font-atlas` tool, a pack's `lang.ron`, the
renderers and `word_wrap` are the engine's since the carve. 2303 gives
them named wide sets (`font-atlas --wide ja=<font.bdf> --wide
zhhans=<font.bdf>`), a pack's `wide_font` (the set it uses) and
`wide_punctuation` (whether `“ ” ‘ ’ … —` are wide in it), looking a
glyph up in the current language's set first and then in any other set
that has it, widths by language, and the Chinese breaking rules. This
ticket is the game's half: the Chinese font, the game's atlas and packs,
and ADR-0045.

## Nick input

None (the font is 0045's answer).

## Scope

**In:** the Chinese font's BDF in `assets-src/fonts/` and its licence;
the game's atlas with both sets; the packs' `lang.ron`; ADR-0045
amended; moving the engine pin to a commit with engine#2303.

**Out (do not do):** the engine's half (engine#2303); translating
(0726); the language pack's text files other than what a test needs in
`assets-test/lang/test/`; changing any Latin or Japanese glyph;
Traditional Chinese; typing Chinese (0045 decides whether that is ever
built). If 0045 chose one font for both languages that has a single
shape per character (Unifont), step 3 shrinks to "one set, shared": do
that, say so in the completion notes, and keep the other steps.

## Implementation steps

1. Move the pin to an engine `main` that has engine#2303 (`cargo xtask
   engine-pin <commit>`, the `engine-change` skill §7).
2. Add the font's BDF and licence text (`assets-src/fonts/`,
   `assets/fonts/<Name>-LICENSE.txt`), a row in `THIRD_PARTY_ASSETS.md`
   and a section in `assets/fonts/README.md`, as 0236 did for the
   Japanese font. A TTF-only font is converted to BDF at 16 px the way
   0236 documented.
3. `shuyi.ron`'s `font_atlas`: two named sets, `ja` (0236's font and its
   fixed set) and `zhhans` (this font, with the CJK punctuation block and
   the full-width forms `，。！？；：（）` always in it). Regenerate the
   atlas with `cargo shuyi font-atlas` and commit it.
4. The packs' `lang.ron`: the Japanese pack gets `wide_font: Some("ja")`,
   the Chinese pack `wide_font: Some("zhhans")` and `wide_punctuation:
   true`; English and `test` keep `None`, or name a set a test needs.
5. Amend ADR-0045 §4 (a dated "as built" paragraph, as 0233 and 0717
   did): a wide set per font, the look-up order, the six
   language-dependent widths, and that the engine builds them
   (engine#2303).
6. A screenshot for the PR: the test pack with a line that mixes
   Chinese, Japanese and Latin on one screen, via `cargo shuyi
   frame-png`.

## Acceptance criteria

- [ ] The same character (直) draws from the Japanese set with the
      language `ja` and from the Chinese set with `zhhans` in the game's
      atlas (a frame PNG test against two committed references).
- [ ] A Chinese line from the test pack shows `“好”` six cells wide, and
      a Chinese paragraph in the dialogue box has no line starting with
      `，` or `。` (Harness tests reading the view or the cells).
- [ ] Every existing snapshot, the narrow atlas cells and the Japanese
      glyphs are unchanged.
- [ ] The font's licence is shipped and listed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the packs' `lang.ron` load with their sets.
- Snapshot / integration: the frame PNGs; the look tests read cells, as
  ADR-0038 allows for tests of a look.

## Completion notes
