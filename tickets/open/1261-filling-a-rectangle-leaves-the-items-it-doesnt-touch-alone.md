---
id: "1261"
title: Filling a rectangle leaves the items it doesn't touch alone
type: bug
milestone: Later
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1261 — Filling a rectangle leaves the items it doesn't touch alone

## Context

1259 looked at `bench/battle-attack.bench.ron`'s frame 43, where the
rider's unit menu opens over *The Company Ledger*: its costliest frame
on the colour targets (2,305,584 instructions in CI's `bench` job on
`main` at 2f10f225; the frame before, 1,205,126). Callgrind's dump of
that frame (the `bench-report` artifact, `runs/battle-attack-colour/
callgrind.out.44`):

| Where | Instructions (inclusive) |
| --- | ---: |
| The battle screen's `draw` | 1,814,071 |
| `GlyphBuffer::fill_rect` (engine, `shuyi_ui::frame::glyph_buffer`) | 1,027,449 |
| of which `GlyphBuffer::cut_items` | 913,299 |
| the menu widget's paint (`shuyi_ui::widgets::menu::glyph::paint`) | 639,317 |
| the map skin's paint | 567,487 |
| opening the menu (`mode::open_menu`: the attack preview, `art_list::fit`) | 310,621 |

The threat arrows are not among them. `fill_rect`'s callers: the menu's
paint (487,970), `GlyphBuffer::panel` (211,291), the battle screen's
paint (100,542), `paint_hp_bar` (97,749). Every frame with the menu open
costs about 2,013,000 on the colour targets (frames 44 on), 800,000 more
than the frame before it; on the Playdate, frame 43 is 2,471,816 and the
next ones about 2,146,000, over its 2,000,000 (with 1248's one-bit cost).

`cut_items` rebuilds `self.items` on every `fill_rect`: for each item,
`subtract(item.visible(), hole)` returns a new `Vec` and the item is
rebuilt with `with_visible`, also when the item doesn't meet the
rectangle at all. Each row the menu fills walks, and reallocates, every
item the map has drawn.

The fix is in Shuyi Engine (`crates/ui/src/frame/glyph_buffer.rs`), so
this ticket follows the `engine-change` skill: an engine ticket and PR
in `Zafnok/shuyi-engine`, then this game's PR that moves the pin and
measures.

## Nick input

None. Nothing a player sees changes.

## Scope

**In:**
- In the engine: `cut_items` keeps an item whose visible part (and
  overhang) doesn't meet the hole as it is, without `subtract`'s `Vec`
  or a rebuild, and does nothing when no item meets the hole. Its
  existing tests and property tests (`fill_rect_only_touches_rect`) pass
  unchanged; a test counts that an item away from the rectangle is kept
  untouched.
- In the game: ~~move the pin~~ (done by the pin move of 2026-11-10, engine #353), and put CI's `bench` numbers in
  `docs/targets.md`.
- If frame 43 is still the costliest frame after it, a Callgrind look at
  it again, named in the completion notes, with a ticket for what is
  left.

**Out (do not do):**
- The Playdate's one-bit drawing (1248).
- Changing how the menu looks, or drawing it into fewer rectangles in
  the game to dodge the cost.
- Raising a budget.

## Implementation steps

1. `engine-change` skill: write the engine ticket, then change
   `GlyphBuffer::cut_items` (`crates/ui/src/frame/glyph_buffer.rs`):
   skip the items that don't meet the hole (`Rect::intersect` on the
   visible part and the overhang), and keep the `Vec` when none do.
2. The engine's tests; its PR, merged.
3. In the game: ~~move the pin (`cargo xtask engine-pin`)~~ (done by the pin move of 2026-11-10, engine #353), run the gates,
   push, and read the `bench` table; update `docs/targets.md` (*The
   numbers*, *Over budget*).

## Acceptance criteria

- [ ] An item away from a filled rectangle is kept as it was (engine
      test), and every existing `GlyphBuffer` test passes unchanged.
- [ ] `battle-attack`'s frame 43 on the colour targets is under
      2,000,000 instructions in CI's `bench` job, or the completion notes
      name what is left and its ticket.
- [ ] Every snapshot passes unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit (engine): the item kept as it was.
- Property (engine): the existing `fill_rect` property tests.
- Snapshot: unchanged frames.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
