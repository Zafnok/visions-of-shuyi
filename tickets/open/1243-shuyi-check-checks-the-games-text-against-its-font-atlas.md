---
id: "1243"
title: "`shuyi check` checks the game's text against its font atlas"
type: infra
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: low
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1243 — `shuyi check` checks the game's text against its font atlas

## Context

Engine #256 (engine ticket 2350, reached by the pin move of 2026-11-04)
gives `shuyi check` a check of every text of every language against the
font atlas (`font::undrawn`): one error per language and character the
atlas can't draw, on the file of its first use. It runs only when the
game's tools offer the atlas (`GameTools::font_atlas`, default `None`),
and the game's `Trpg` tools (`crates/xtask/src/game_tools.rs`) don't yet,
so `check` says in its limits that the text isn't checked. The same PR
makes `cargo shuyi font-atlas`, run in the game's tool program, keep the
characters the text uses (`AtlasChars`) as well as the ranges and the
required glyphs.

Today a character a translation or a `.dlg` line uses that Terminus
lacks shows as the fallback box, and nothing fails. This ticket turns
the engine's check on; the game writes no check of its own. About 15
production lines added, none removed (the game has no copy of this
check: `REQUIRED_GLYPHS` is the glyphs the game's own drawing needs,
which stays).

## Nick input

None.

## Scope

**In:**
- `Trpg::font_atlas` in `crates/xtask/src/game_tools.rs`: the game's
  atlas (`trpg_content::font::load`).
- `cargo shuyi check` on the repository passes with no limit about the
  atlas. A character it names that the game's text really uses (say a
  typographic quote) is fixed in the text or by regenerating the atlas,
  whichever the font allows.
- `cargo shuyi font-atlas` run again with the text's characters, and the
  atlas committed if it changes. `committed_atlas_is_up_to_date`
  (`crates/xtask/src/game_tools/tests.rs`) builds with the text's
  `AtlasChars` in place of `AtlasChars::default()` (the pin move's
  call, the atlas of before).

**Out (do not do):**
- Wide glyphs and the Japanese font (0236), the Chinese font (0239).
- Text length checks with the widths (engine 2252).

## Implementation steps

1. Offer the atlas in `Trpg`'s `GameTools`.
2. Run `cargo shuyi check`; fix what it names.
3. Run `cargo shuyi font-atlas` with the game's fonts; commit what it
   changes; move the up-to-date test onto the text's characters.

## Acceptance criteria

- [ ] `cargo shuyi check` passes with no limit saying the text isn't
      checked against the atlas.
- [ ] A test pack line with a character the atlas lacks fails `check`
      (a test of the game's tools, on the test language pack).
- [ ] `committed_atlas_is_up_to_date` builds the atlas from the text's
      characters and passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Integration: the game's tools' `check` with a test pack line the atlas
  can't draw fails naming the character; with the real packs it passes.

## Completion notes
