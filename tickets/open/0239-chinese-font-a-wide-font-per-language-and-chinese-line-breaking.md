---
id: "0239"
title: "Chinese glyphs: a wide font per language, wide punctuation, Chinese line breaking"
type: feature
milestone: M1 Engine
model: opus-5.5
effort: high
status: todo
blocked_by: ["0045", "0236", "0237"]
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

## Nick input

None (the font is 0045's answer).

## Scope

**In:** the Chinese font's BDF in `assets-src/fonts/`; the atlas tool,
format and validation; which wide set a language uses; widths that
depend on the language; the wrapping rules; ADR-0045 amended.

**Out (do not do):** translating (0726); the language pack's text files
other than what a test needs in `assets/lang/test/`; changing any Latin
or Japanese glyph; Traditional Chinese; typing Chinese (0045 decides
whether that is ever built). If 0045 chose one font for both languages
that has a single shape per character (Unifont), steps 2 and 3 shrink to
"one set, shared": do that, say so in the completion notes, and keep
steps 4 to 6.

## Implementation steps

1. Add the font's BDF and licence text (`assets-src/fonts/`,
   `assets/fonts/<Name>-LICENSE.txt`), a row in `THIRD_PARTY_ASSETS.md`
   and a section in `assets/fonts/README.md`, as 0236 did for the
   Japanese font. A TTF-only font is converted to BDF at 16 px the way
   0236 documented.
2. `crates/xtask/src/font_atlas.rs`: `--wide` takes a named set and may
   be given more than once (`--wide ja=<font.bdf> --wide zhhans=<font.bdf>`).
   A set's glyphs are the wide characters used by the packs that name it
   (step 3), plus that set's fixed minimum (Japanese: as 0236; Chinese:
   the CJK punctuation block and the full-width forms `，。！？；：（）`).
   Sets are laid out one after another after the narrow glyphs.
3. `trpg_content::font::FontAtlasDef`: wide glyphs become a map per set
   name. `lang.ron` gains `wide_font: Some("zhhans")` (the Japanese pack
   gets `Some("ja")`; English and `test` keep `None` or name a set a test
   needs). Validation: a pack's `wide_font` names a set the atlas has;
   every wide character of a pack's text is in its set. The stale-atlas
   test passes every set.
4. Looking a wide glyph up: the current language's set first, then any
   other set that has the character. (The Options row shows `日本語` and
   `简体中文` at the same time, whatever the language is.) One function
   in `ui`, used by the renderer in `crates/app/src/render.rs` and by
   `cargo xtask frame-png`.
5. Widths by language: a pack's `lang.ron` gains `wide_punctuation: bool`
   (true for Chinese). When true, `“ ” ‘ ’ … —` are wide and drawn from
   the pack's wide set; otherwise they are narrow as today. This relaxes
   0236's "no char is both narrow and wide" to "except these six, by the
   language's flag". `GlyphBuffer`'s `is_wide` and `text_width` (0236)
   take the width table of the current language; `content`'s length
   checks for a pack use that pack's table.
6. Wrapping (`wrap` in the `ui` text module, 0237): add to the
   never-start-a-line set `，。、；：？！）》〉」』”’…—` and to the
   never-end-a-line set `（《〈「『“‘`; never break between the two halves
   of `……` or `——`. The quote and dash rules apply only where they are
   wide (step 5), so English wrapping is unchanged.
7. Amend ADR-0045 §4 (a dated "as built" paragraph, as 0233 and 0717
   did): a wide set per font, the look-up order, the six
   language-dependent widths.
8. A screenshot for the PR: the test pack with a line that mixes Chinese,
   Japanese and Latin on one screen, via `frame-png`.

## Acceptance criteria

- [ ] The same character (直) draws from the Japanese set with the language `ja` and from the Chinese set with `zhhans` (a frame PNG test against two committed references).
- [ ] With `wide_punctuation`, `“好”` is six cells wide and `……` is four; without it, `“ok”` is four cells (unit tests).
- [ ] A Chinese paragraph wraps within the width with no line starting with `，` or `。`, none ending with `“`, and `……` never split (unit tests with real sentences).
- [ ] A pack that uses a character its set lacks fails validation; the stale-atlas test fails when a pack gains a new character (tests).
- [ ] Every existing snapshot, the narrow atlas cells and the Japanese glyphs are unchanged.
- [ ] The font's licence is shipped and listed.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: set look-up and fall-back; language-dependent widths; each new breaking rule; atlas validation.
- Property: 0237's wrap properties (width, round trip) hold with `wide_punctuation` on.
- Snapshot / integration: the frame PNGs; the look tests read cells, as ADR-0038 allows for tests of a look.

## Completion notes

