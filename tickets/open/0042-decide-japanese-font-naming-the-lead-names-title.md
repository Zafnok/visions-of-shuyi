---
id: "0042"
title: "Decide: Japanese — the font, naming the lead, names, the title"
type: design-decision
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0042 — Decide: Japanese — the font, naming the lead, names, the title

## Context

Nick, 2026-10-03 (`docs/design/voices-languages-and-script.md`): add a
machine-translated Japanese option, with a human translation later if the
game earns enough. Not a blocker for Chapter 1 or Act 1. The technical
plan is [ADR-0045](../../docs/adr/0045-languages-text-by-key-and-line-ids.md);
this ticket asks Nick the parts a player sees.

**Chinese went first** (Nick, 2026-11-24: "why should chinese be
blocked by japanese? lets do chinese first."). 0045 asked the two
questions both languages share, and they are not asked again here: a
player picks a language on a first-launch list with the computer's
language highlighted, and in Options; a machine translation says so
beside the language's name (日本語 (機械翻訳)), on the store page and in
the credits (`docs/design/voices-languages-and-script.md`, *Picking a
language and saying it is machine-made*). The Chinese font is Fusion
Pixel 12px, with Unifont 16px in Options: Fusion Pixel has Japanese
shapes too (`ja`), the cheapest way to make the two languages look
alike, so render it among the candidates.

Nobody on the project reads Japanese, Nick included. So he judges the
*look* from rendered screens, and the wording questions are put to him in
English with what Japanese players are used to.

## Nick input

**Decision.** Run with the `ask-nick` skill, one question at a time, each
with options from real games and "describe your own". Expect several
rounds on the font (he picks looks by eye).

## Scope

**In:** the questions below; the answers written into
`docs/design/voices-languages-and-script.md` (a *Japanese* section) and a
row in `docs/design/README.md`; the chosen font's row in
`assets/fonts/README.md`'s candidates table (licence checked at source).

**Out (do not do):** any game code; adding the font to the atlas (0236);
translating anything (0719). No mockup made with bought art is committed
(ADR-0040); mockups use the public placeholder portraits.

## Implementation steps

1. **Find the font candidates.** 16×16 pixel fonts with kana and kanji
   whose licence ADR-0013 allows (OFL-1.1, public domain, MIT…), read at
   the source on the day. Start from: GNU Unifont (dual GPL / OFL-1.1),
   Shinonome 16 (public domain), DotGothic16 (OFL-1.1), Fusion Pixel
   12px `ja` (OFL-1.1; Ark Pixel 16px was retired, 0045), M+ bitmap (its
   permissive notice passed ADR-0013's review in 0045, inside Fusion
   Pixel's Cubic 11 licence). Drop any whose licence fails; say so in
   the ticket.
2. **Render mockups**, real pixels at 2×, of the same dialogue box and the
   same unit-info panel in each font, beside Terminus Latin text (stats
   and numbers stay Latin), and beside the Chinese font 0045 chose, so
   Nick sees the two languages side by side. 0045's throwaway script
   (paint over a real 2× frame cell by cell) is described in its
   completion notes; put the PNGs in `docs/screenshots/0042-*.png`. Use a short Japanese
   sample of a Chapter 1 line (machine-translated for the mockup only).
3. **Ask Nick**, in this order:
   1. **The font**, from the mockups.
   2. ~~Picking a language~~ and ~~saying it is machine-translated~~:
      answered in 0045 for every language (above).
   3. **Naming the lead in Japanese.** (0045's answer for Chinese was
      typing with the system's input method, 0262.) Options: a kana grid picked with
      the cursor (Dragon Quest, Pokémon); Latin letters only (the screen
      as it is); a fixed default name with no entry in Japanese.
   4. **Names.** Katakana for every name (what Fire Emblem's Japanese
      releases do for Western-style names), or Latin names left as they
      are.
   5. **The game's title in Japanese:** keep *Visions of Shuyi* in Latin
      letters, or add a Japanese subtitle. (Feeds 0906's trademark check
      if it changes.)
   6. **Voices with Japanese text** was answered in 0043: Japanese voices (0755).
4. Record each answer verbatim and as a rule. If an answer needs work no
   ticket covers (a kana name-entry grid), write that ticket (`write-ticket`)
   and add it to 0719's `blocked_by` if Chapter 1 in Japanese needs it.
5. Update 0718 and 0825 where an answer changes their text. The
   Japanese font goes in the atlas as the set `ja`: in 0236 or 0239 if
   either is still open (add the step there), otherwise in a ticket of
   its own (`write-ticket`) that 0719 waits for. Both closed before this
   was answered (0239, 2026-11-25), so it is a ticket of its own, and it
   carries 0239's check left waiting: the same character (直) draws from
   the set `ja` with the language `ja` and from `zhhans` with `zhhans`
   (a frame PNG test against two committed references). The engine
   already looks a glyph up in the language's set first (engine#2303).

## Acceptance criteria

- [ ] Every question above has Nick's answer, verbatim, in the design doc.
- [ ] The chosen font's licence is quoted with its source URL and passes ADR-0013.
- [ ] The mockups are in `docs/screenshots/` and contain no bought art.
- [ ] Follow-up tickets exist for anything the answers add; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes

