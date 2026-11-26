---
id: "0831"
title: "Game Over's words as a picture: a language can bring its own"
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: low
status: done
blocked_by: ["0830"]
nick_input: none
completed: 2026-11-26
---

# 0831 — Game Over's words per language

## Context

Ticket 0830 made the words `GAME OVER` on the Game Over screen a picture
in the logo's block lettering (`assets/title/game_over.png`, generated
from `assets-src/title/game_over.txt`; `docs/design/look-and-feel.md`,
*Game Over screen*). A picture reads the same in every language: the
language files' `game_over.heading` is only shown when `title.ron` names
no picture.

English is the only shipped language today, so nothing is wrong yet. The
first translated language (tickets 0042, 0045; ADR-0045) needs a way to
show its own words.

**The engine holds a language's copies** (engine #590, ticket 2278, made
for this ticket and in the pin since the pin move of 2026-11-26). A
language pack's folder may hold `files/<path>`, that language's copy of
the game's file at `<path>` (`assets/lang/<tag>/files/title/game_over.png`),
and `Ctx::localized("lang", path, has)` returns the first copy along the
language's chain, else `path` itself. The game reads its packs with the
engine's `shuyi_assets::lang` (`crates/content/src/lang.rs`), so the
folder loads with no change there; the game writes no loader or field of
its own.

## Nick input

None. (Which words a language shows is that language's translation.)

## Scope

**In:**
- A language pack can name its own Game Over picture, or none: with none,
  that language shows its `game_over.heading` as text in the picture's
  place (the fallback 0830 built).
- The `test` language pack (`assets-test/lang/test/`) uses it, so a test
  covers the switch.

**Out (do not do):**
- Drawing the words for any real language.
- The logo: the game's name is not translated here (0045 asks).

## Implementation steps

1. A language's picture is its pack's copy,
   `lang/<tag>/files/title/game_over.png` (above). Check that the game's
   image table lists it (the engine's lists every PNG of the bundle), and
   that the Playdate's lettering (`LETTERINGS` in
   `crates/content/src/playdate_images.rs`) takes a language's copy too.
2. `game_over/glyph.rs` asks `ctx.localized("lang",
   "title/game_over.png", …)` each frame it draws, so a change of
   language shows at once. When it returns the plain path (no pack along
   the chain has a copy), a language other than the source shows its
   `game_over.heading` as text, as the scope says, never the English
   picture.
3. Document it in `assets/title/README.md` and the language pack's README.

## Acceptance criteria

- [x] In English the screen is as 0830 left it.
- [x] Test: a language with its own picture shows it; one with none shows
      its heading as text.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: which picture or text each language gets (a copy, a copy along
  the chain, none).
- Snapshot / integration: Game Over in the `test` language.

## Completion notes

- `game_over/glyph.rs`: `words_picture` asks `ctx.localized("lang",
  <the shipped picture's path>, …)` each frame. A copy along the
  language's chain is drawn in place of the English picture, at its own
  size, centred on the same row; the source language draws the shipped
  picture; a language with no copy prints its `game_over.heading` as
  text (0830's fallback), never the English picture. English is
  unchanged (its snapshot and look tests are as 0830 left them).
- The image table already lists a pack's PNGs (the engine's lists every
  PNG of the bundle); nothing changed there. The Playdate's `kind_of`
  (`playdate_images.rs`) now reads `lang/<tag>/files/<path>` as of its
  picture's kind, so a language's Game Over words are a lettering too.
  That code carries 1221's marker (engine#2245): a dated line in 1221's
  Context says the engine's bake covers it (a copy gets its own
  `.meta`).
- The `test` pack has `files/title/game_over.png`, a 23×7 `TEST` in the
  same gold, drawn by a one-off script (a test picture, not shipped).
- Tests: which picture or text each language gets (own copy, a copy
  along the chain `pt-BR` → `pt`, none, no picture named, the test pack
  hidden without debug tools); the Game Over screen in the test language
  (snapshot plus its sprite).
- Documented in `assets/title/README.md` and `assets/lang/README.md`
  (*A language's own files*).
- No gameplay rule decided. Nothing to judge by playing.
- Engine friction: estimate 30, measured 31 (1.0×), under 2×. PR size: estimate 250, measured 221.
