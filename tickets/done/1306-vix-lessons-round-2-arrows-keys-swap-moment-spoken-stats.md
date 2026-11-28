---
id: "1306"
title: "Vix's lessons, round 2: arrow symbols, no repeated cursor item, swapping in its moment, stats spoken in full"
type: feature
milestone: M6 Story & dialogue
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1304"]
nick_input: none
completed: 2026-11-20
---

# 1306 — Vix's lessons, round 2: arrow symbols, no repeated cursor item, swapping in its moment, stats spoken in full

## Context

Nick played the re-pushed Chapter 1 demo on 2026-11-20, after 1304, and
gave a second round of feedback on Vix's tutorial (his words):

1. "the arrows tutorial shouldnt say Up Down Left Right but use some kind
   of symbol denoting arrow direction"
2. "the tutorial that pops up after 'played with the keys' also asks to
   move the cursor even tho it just happened in the prev tutorial which is
   dumb."
3. "'this unit has more than one weapon, try changing them' tutorial can
   trigger right after the battle tutorial so the voiceline and tip are no
   longer true about that specific unit, make it so it triggers next time
   conditions met on non exhausted unit"
4. "when I said Vix should say DEF and AVO as full words I meant her
   voiceline, the text can still display as DEF and AVO. So we should have
   script separate from spoken replacements probably."

These are Nick's decisions (`docs/design/tutorial.md`, *Round 2*). The
lessons and tips are in the private repository's `content/data/`
(`content-change` skill); the voice tool's respellings are `say.ron`
beside the clips (`docs/voice.md`). It goes out before the writer trial's
packets (0766).

## Nick input

None: the decisions are above. What's left to judge by playing goes on the
Act 1 playtest's checklist (1064).

## Scope

**In:**
1. A `{CursorUp}`-style placeholder for one cursor direction names its
   arrow (↑ ↓ ← →) while the four directions are on the arrow keys
   (`crates/ui/src/tips.rs`, `key_text`); other keys keep their names.
2. The `keys` lesson drops its "Move the cursor" item (`CursorMoved`).
3. The `weapon_swap` lesson opens only while a forecast that can swap is
   on screen; once open, it stays while a unit left could swap
   (`LessonState::settle_when`, `BattleScreen::settle_lessons`).
4. The tips show DEF, AVO, EXP and Mov again; `say.ron` makes the voice
   say Defense, Avoid, Experience and Movement; their three clips are
   made again.

**Out:** other lessons or tips; renaming keys in the engine's key table
(its names are also the keymap file's).

## Implementation steps

1. Expected size: about 40 production lines (the arrow names, the settle
   flag), data edits in two RON files and `say.ron`, tests.
2. Build points 1 to 4 as above; update `docs/design/tutorial.md`.
3. Tests: arrow names on the arrow keys and not on others; the keys
   lesson at Short closes on Options alone; the swap lesson waiting behind
   another lesson in its forecast doesn't open after it, and opens at the
   next forecast that swaps.
4. `cargo xtask voice generate --scene vix`; `voice status` shows none
   missing. Update `shuyi.ron`'s disclosure lists for the clips.
5. The private PR first; this PR moves `assets-private.rev` to its
   **merge on main** before it merges.

## Acceptance criteria

- [x] `a_cursor_direction_on_the_arrow_keys_is_its_arrow` passes.
- [x] `the_keys_lesson_opens_first_at_short_and_closes_after_options`
      passes with no cursor item.
- [x] `the_swap_lesson_opens_only_in_a_forecast_that_swaps` passes.
- [x] The tips say DEF, AVO, EXP and Mov; the three clips were sent
      Defense, Avoid, Experience and Movement (`sent.ron`).
- [x] `docs/design/tutorial.md` says the new rules.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit and Harness tests (ADR-0007), as in step 3.

## Completion notes

Built as scoped. The arrow names go through the game's placeholder
filling: they show wherever a tip or lesson names one direction's key
(the cursor lesson, the swap lesson's `{CursorLeft} {CursorRight}`), in
key caps too; the whole cursor (`{Cursor}`) keeps its arrow-cluster
picture. The engine's key table is untouched.

Voice: 3 clips made from the full words, 3 removed; `voice audit` flags
none.

### For the Act 1 playtest (1064)

Swapping weapons, *the agent's starting rule*: opened, it stays (holding
the turn) while some unit still to act could swap, as before; only its
opening waits for the forecast.

Left to judge by playing (1064): the arrows' look in the lesson box.
