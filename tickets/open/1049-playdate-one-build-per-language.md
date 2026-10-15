---
id: "1049"
title: "Playdate: one build per language (English, Japanese, Chinese)"
type: feature
milestone: Post–Chapter 1
model: opus-5.5
effort: high
status: todo
blocked_by: ["1036", "1045", "0719", "0726"]
nick_input: sign-off
completed:
---

# 1049 — One Playdate build per language

## Context

Nick: Japanese and Chinese are kept on the Playdate, but **each language
is its own build** ("it is a limited device on purpose"); one purchase
gets every language's build (`docs/design/playdate.md`, Q5, Q6).

On the PC every language pack is in one game (`assets/lang/`; Japanese
0719, Chinese 0726, their fonts 0236/0239 with wide glyphs: two cells per
glyph). 1036 made the Playdate bundle carry one language pack; 1032 made
the 1-bit font from the atlas.

## Nick input

**Sign-off:** Nick opens the Japanese and Chinese builds on his Playdate
and checks the text shows (no missing-glyph boxes) and fits, especially
the battle pop-ups and the dialogue box at 50 × 15 cells. Checking the
wording is the translation tickets' job (0719, 0726; who checks the
Chinese is 0045's question), not this one's.

## Scope

**In:**
- `cargo xtask playdate --lang <en|ja|zh>` (default `en`): the bundle
  holds that language's pack and font only; the `.pdx` is named per
  language (e.g. `VisionsOfShuyi-ja.pdx`) and its `pdxinfo` has a
  bundle ID per language so all three can sit on one device side by side
  (and their saves stay apart). The game starts in that language and
  hides the language option (1045 hid it for one-language builds).
- **Wide glyphs in one bit:** the 1-bit font tool (1032) also packs the
  wide fonts (0236, 0239), and the one-bit renderer draws two-cell
  glyphs. Look tests for a Japanese and a Chinese line.
- **Text that doesn't fit:** Japanese and Chinese lines are checked to fit
  the Playdate's pop-ups, menus and dialogue box with the same line
  breaking as the PC (0237, 0239). Fix overflows in the Playdate skins
  (e.g. wrap or scroll), never by changing the translations; list any
  that can't be fixed as a follow-up ticket.
- The CI `playdate` job builds all three languages.
- If 1052 (selling on itch.io) is done, add the `playdate-ja` and
  `playdate-zh` builds and butler channels to its release jobs; if not,
  1052 does it.
- Voices per language when they exist (1048 converted `en`; add the
  others if their clips exist; otherwise silent).

**Out (do not do):**
- Translating anything.
- Selling and the store page (1052).

**Carve (1105):** either order works. If languages moved to the engine,
the per-language bundle uses the engine's language layers.

## Implementation steps

1. Read `docs/design/playdate.md`, ADR-0070, 0236/0239's fonts, 0237's
   line breaking, 1036's bundle list.
2. `--lang`, bundle IDs, start language.
3. Wide glyphs in the 1-bit font and renderer, with look tests.
4. Walk each screen in Japanese and Chinese in the Simulator; fix
   overflows.
5. CI for three languages; ask Nick to sideload.

## Acceptance criteria

- [ ] Three `.pdx` builds, one per language, install side by side.
- [ ] Each starts in its language; Chapter 1 plays through in the
      Simulator in each.
- [ ] Look tests for wide glyphs pass.
- [ ] Nick's comment in the completion notes (or "no comment yet").
- [ ] All gates in the `run-gates` skill pass, and the CI `playdate` job.

## Tests required

- Unit (look): wide glyphs in one bit.
- Integration (harness, Playdate skin set): Chapter 1's first scene and
  turn in Japanese and Chinese.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
