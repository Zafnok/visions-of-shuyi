---
id: "0045"
title: "Decide: Chinese (Simplified) — the font, the title and the name Shuyi, names, naming the lead, who checks it"
type: design-decision
milestone: M1 Engine
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0042"]
nick_input: decision
completed:
---

# 0045 — Decide: Chinese (Simplified) — the font, the title and the name Shuyi, names, naming the lead, who checks it

## Context

Nick, 2026-10-04 (`docs/design/voices-languages-and-script.md`): add a
machine-translated Chinese option (Mandarin, Simplified characters)
beside the Japanese one. His wife is Chinese and the game is named after
her. Like Japanese, it is not a blocker for Chapter 1 or Act 1. The
technical plan is the same one
([ADR-0045](../../docs/adr/0045-languages-text-by-key-and-line-ids.md));
this ticket asks Nick the parts a player sees that are *different* for
Chinese.

0042 comes first because two of its answers are shared and are not asked
again here: how a player picks a language, and how a machine translation
is labelled. Its font answer also matters: a font family that has both
Japanese and Simplified Chinese shapes is the cheapest way to make the
two languages look alike.

Two things differ from Japanese:

- **Somebody close to the project reads Chinese.** Whether Nick's wife
  looks at any of it is his and her call; nothing may assume she will.
- **"Shuyi" is a real name with real characters.** A machine can't know
  which characters write it (many pairs are read *shūyí*); only Nick can
  say.

## Nick input

**Decision.** Run with the `ask-nick` skill, one question at a time, each
with options from real games and "describe your own". Expect several
rounds on the font (he picks looks by eye).

## Scope

**In:** the questions below; the answers written into
`docs/design/voices-languages-and-script.md` (a *Chinese* section) and
the row in `docs/design/README.md`; the chosen font's row in
`assets/fonts/README.md`'s candidates table (licence checked at source).

**Out (do not do):** any game code; adding the font to the atlas (0239);
translating anything (0726); Traditional Chinese (Nick asked for
Simplified; a ticket of its own if he later wants Taiwan and Hong Kong
covered); Cantonese. No mockup made with bought art is committed
(ADR-0040); mockups use the public placeholder portraits.

## Implementation steps

1. **Find the font candidates.** 16×16 pixel fonts with Simplified
   Chinese shapes whose licence ADR-0013 allows, read at the source on
   the day. Start from: the font 0042 chose, if its family has a
   Simplified Chinese variant (Ark Pixel 16px has `zh_cn`; GNU Unifont
   covers both, with one shape per character); Fusion Pixel (OFL-1.1);
   WenQuanYi Bitmap Song (GPL with a font exception: needs ADR-0013's
   review, likely fails); Zpix (free for non-commercial use only: fails
   unless its terms changed). For each, count how many of the 3,755
   common characters (GB 2312 level 1) it has: a font with gaps can't
   carry a whole game's text. Drop any that fail the licence or the
   coverage; say so in the ticket.
2. **Render mockups**, real pixels at 2×, like 0042's: the same dialogue
   box and unit-info panel per font, beside Terminus Latin text and
   beside the Japanese font 0042 chose (so Nick sees the two languages
   side by side). PNGs in `docs/screenshots/0045-*.png`. Use a short
   Chinese sample of a Chapter 1 line (machine-translated for the mockup
   only).
3. **Ask Nick**, in this order:
   1. **Which characters write "Shuyi".** Free answer (he may need to
      ask his wife). Also: whether the game may use those exact
      characters, or should use a different pair with the same sound.
   2. **The game's title in Chinese.** Options: a Chinese title using
      those characters (games sold in China nearly always show a Chinese
      title; offer three machine-made candidates with their literal
      meanings in English); *Visions of Shuyi* in Latin letters only; the
      Latin title with a Chinese subtitle. Feeds 0906's trademark check
      if it changes.
   3. **Who checks the Chinese.** Options: nobody, it is machine-made
      like the Japanese; his wife reads the short lists only (the title,
      the names, the glossary of game terms: about an hour); she also
      plays Chapter 1 in Chinese and says what reads wrong. If she
      checks anything: whether she is credited, and under what name.
      Say plainly that "nobody" is a fine answer and that the tickets
      work either way.
   4. **The font**, from the mockups.
   5. **Names.** Options: every name written in Chinese characters
      chosen for their sound (what Fire Emblem's official Chinese
      releases do); Latin names left as they are. If characters: whether
      whoever checks the Chinese (question 3) sees the names list before
      it is used, since a sound-alike name can carry an unlucky meaning.
   6. **Naming the lead in Chinese.** Typing Chinese characters needs
      the system's input method, which the game doesn't have. Options:
      Latin letters only (the screen as it is); a fixed default Chinese
      name, no entry; picking from a short list of Chinese names; typing
      Chinese with the system keyboard (say plainly: a large separate
      job, not Chapter 1).
   7. **The machine-translation label for Chinese.** 0042 chose the
      form; show him the Chinese wording ("简体中文 (机器翻译)"). Only if
      question 3 has his wife checking: whether the label says so
      ("machine translation, checked by a native reader") or stays as it
      is.
   8. **Voices with Chinese text** is asked in 0043, not here.
4. Record each answer verbatim and as a rule. If an answer needs work no
   ticket covers (a name list, system text input, a review step), write
   that ticket (`write-ticket`) and add it to 0726's `blocked_by` if
   Chapter 1 in Chinese needs it.
5. Update 0239, 0725, 0726 and 0907 where an answer changes their text.

## Acceptance criteria

- [ ] Every question above has Nick's answer, verbatim, in the design doc.
- [ ] The characters for "Shuyi" are written in the design doc exactly as Nick gave them.
- [ ] The chosen font's licence is quoted with its source URL and passes ADR-0013; its coverage count is recorded.
- [ ] The mockups are in `docs/screenshots/` and contain no bought art.
- [ ] Follow-up tickets exist for anything the answers add; `cargo xtask ticket-lint` passes.

## Tests required

None (docs only).

## Completion notes

