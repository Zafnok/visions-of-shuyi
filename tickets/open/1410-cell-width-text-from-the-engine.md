---
id: "1410"
title: "Text padded, clipped and wrapped in cells by the engine: pad_cells, clip, wrap_indented and wrap_notes go"
type: feature
milestone: "Engine: carve"
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: blocked
blocked_by: ["engine#2291"]
nick_input: none
completed:
---

# 1410 — Text padded, clipped and wrapped in cells by the engine

## Context

The third engine-friction sweep (1397, `docs/engine/friction.md`) found
cell-width text helpers in the game that any translated game needs,
about 130 production lines in `crates/ui/src`:
- `screens/mod.rs`: `clip` and `pad_cells` (about 30 calls in 16
  files);
- "the widest name, then pad each" written 5–6 times
  (`screens/battle/map_menu.rs`, `objective.rs`, `items.rs`, `magic.rs`,
  `skills.rs`);
- `widgets/mod.rs`: `wrap_indented` and `after_line`;
  `screens/battle/notes.rs`: `wrap_notes`.

Engine ticket 2291 adds `pad_cells`, `pad_cells_left`, `clip_cells`,
`column_width` and `wrap_hanging`. Each item it replaces is marked
`// engine-friction: engine#2291, moves (game 1410 deletes it)`.

The sweep also found `unwrap_notes` (`notes.rs`), which rebuilds the
notes' texts from their wrapped lines so the Playdate skin can wrap them
again at its own width. That is a game-side structure (ADR-0054: the
view carries the texts, each skin wraps), done here too.

## Nick input

None. Nothing on screen changes: the snapshots show it.

## Scope

**In:**
- Move the pin past engine 2291 if it isn't (`engine-change` §7).
- Every call onto the engine's functions; `clip`, `pad_cells`,
  `wrap_indented`, `after_line` and `wrap_notes` go; the widest-then-pad
  copies use `column_width`.
- The notes' view carries the note texts; each skin wraps them with
  `wrap_hanging`; `unwrap_notes` goes.

**Out (do not do):**
- The arts menu's padding by characters (`art_list.rs`): its own bug
  ticket, 1422. Either order: if 1422 is done its `pad_cells` call moves
  here like the rest; if not, 1422 uses the engine's function.
- `wrap_step` and the `cells` copies (other tickets of the sweep).

## Implementation steps

1. Estimate: about 130 lines removed, 30 added.
2. Move the pin if needed.
3. The padding and clipping calls (mechanical), then the wraps, then the
   notes' view, with the snapshot tests after each.

## Acceptance criteria

- [ ] `grep -n "engine#2291" crates/` finds nothing.
- [ ] `grep -rn "fn pad_cells\|fn clip\|fn wrap_indented\|fn wrap_notes\|fn unwrap_notes" crates/ui/src`
      finds nothing.
- [ ] Every snapshot test passes unchanged, the Chinese ones included.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- The existing snapshot and view tests, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket.)*
