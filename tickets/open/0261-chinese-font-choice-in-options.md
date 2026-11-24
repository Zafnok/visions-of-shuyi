---
id: "0261"
title: "A second Chinese font in Options: GNU Unifont 16px with spaced lines"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0239", "0825"]
nick_input: none
completed:
---

# 0261 — A second Chinese font in Options: GNU Unifont 16px with spaced lines

## Context

Nick and his wife picked the Chinese font in 0045 (2026-11-24,
`docs/design/voices-languages-and-script.md`, *Chinese (Simplified)*,
*The font*): Chinese text starts in **Fusion Pixel 12px** (0239 adds it),
and "B in Options": a font row, shown when the language is Chinese,
switches to **GNU Unifont 16px** with a blank row between lines of text
so the lines don't touch. Both fonts ship. The mockups are
`docs/screenshots/0045-font-compare.png` (B and C) and
`0045-info-unifont.png`.

Unifont's licence is checked: OFL-1.1, or GPL-2+ with the font
exception; we take the OFL (the design doc's *What the font research
found*; `assets/fonts/README.md`, *Chinese (Simplified)*). Unifont has a
16×16 glyph, so it fills the two cells; Fusion Pixel's 12×12 one leaves
space around it, which is why only Unifont needs the spaced lines.

## Nick input

None: 0045 decided both fonts and that B is the Options choice. The
row's wording and where it sits in Options are this ticket's to propose
(0045's starting rules): if they aren't obvious from the Options screen's
other rows, ask through `ask-nick` before building, with a rendered
picture.

## Scope

**In:** Unifont's BDF (or `.hex` converted) in `assets-src/fonts/`, its
licence text, a `THIRD_PARTY_ASSETS.md` row and the credit; a second
Chinese wide set in the atlas; a setting and an Options row that picks
between the two; one blank row between lines wherever Chinese text wraps
over several lines, while Unifont is picked.

**Out (do not do):** the default Chinese font and Chinese line breaking
(0239); translating (0726); a font choice for any other language; a
change to any Latin glyph.

## Implementation steps

1. Download GNU Unifont 18.0.01 (or the release current on the day) from
   https://unifoundry.com/pub/unifont/; read `LICENSE.txt` again and
   quote it in `assets/fonts/README.md` (move the row from *chosen, not
   yet in the atlas* to the font's own section, as 0239 did). Ship the
   OFL text as `assets/fonts/Unifont-LICENSE.txt`, add the
   `THIRD_PARTY_ASSETS.md` row and the credit in
   `assets/data/credits.ron`.
2. A second Chinese set in `shuyi.ron`'s `font_atlas` (e.g. `zhhans16`
   beside 0239's `zhhans`), holding the same characters as the default
   set, so a pack's text can draw from either. Regenerate the atlas. If
   the engine's wide sets (engine#2303) can't switch a language's set at
   run time, follow the `engine-change` skill.
3. `Settings` (0805) gains the Chinese font choice, default the Fusion
   Pixel set; a settings-version bump and migration as 0825 did for the
   language.
4. The Options row, shown only while the language is Chinese. Changing
   it takes effect at once.
5. Spaced lines: while Unifont is picked, every place that draws wrapped
   Chinese text over several lines (the dialogue box, tips, descriptions,
   help) leaves one blank row between lines. A place whose text no
   longer fits (fewer lines fit) is listed by `cargo shuyi lang-status
   zhhans` or a test, and fixed by its layout, not by squeezing the rows.
6. Pictures for the PR (`cargo shuyi frame-png`, public placeholder art
   only, ADR-0040): the dialogue box and unit info in each font.

## Acceptance criteria

- [ ] With Chinese and the Unifont choice, a two-line Chinese dialogue line is drawn with a blank row between its lines; with the default it isn't (Harness tests reading the view).
- [ ] The Options row shows only while the language is Chinese, and the choice survives a restart (Harness test).
- [ ] Old saved settings load with the Fusion Pixel default (unit test).
- [ ] The same character draws from Unifont with the choice on and from Fusion Pixel with it off (a frame PNG test against two references).
- [ ] Unifont's licence is shipped, listed and credited.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: settings migration.
- Snapshot / integration: the Options row; the spaced lines; the frame PNGs (look tests read cells, as ADR-0038 allows).

## Completion notes

