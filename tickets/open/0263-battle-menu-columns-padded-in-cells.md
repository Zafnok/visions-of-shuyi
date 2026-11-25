---
id: "0263"
title: "Battle menus pad their name columns in cells, not characters"
type: feature
milestone: M1 Engine
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 0263 — Battle menus pad their name columns in cells, not characters

## Context

0236 put the first wide font in the atlas (Fusion Pixel 12px, the wide
set `zhhans`): a Chinese character takes two cells, and the screens'
skins now measure text with the buffer's widths (`buf.text_cells`,
`shuyi_assets::lang::text_width`, the engine's `cut`). Some battle menus
are built as text by the screen logic instead, with their name column
padded by `format!` (`{name:<w$}`), which counts characters; with wide
names the columns after it slip by one cell per wide character:

- `crates/ui/src/screens/battle/items.rs`: `pack_menu` (`name_w`,
  `format!("{:<w$} ×{}  {effect}")`) and `equip_menu` / `equip_label`;
- `crates/ui/src/screens/battle/magic.rs`: `spell_label` (`name_w`);
- `crates/ui/src/screens/battle/skills.rs`: the `Skill` menu's line
  (`format!("{name:<name_w$}  {text:>6}{weapon}")`);
- `crates/ui/src/screens/battle/map_menu.rs`: the unit list
  (`format!("{:name_w$}", …)`);
- `crates/ui/src/screens/battle/objective.rs`: `label_width` and
  `padded`.

Found while working 0236 (ADR-0045 §4).

## Nick input

None.

## Scope

**In:** the menus above measure their names in cells and pad with the
spaces the cells need, with the language's widths (`ctx.widths()`).

**Out (do not do):** wrapping (0237); the engine's widgets; translating;
any change to what an English menu shows (its snapshots stay as they
are).

## Implementation steps

1. Add `pad_cells(text, w, widths) -> String` next to `clip` in
   `crates/ui/src/screens/mod.rs`: `text` and as many spaces as make it
   `w` cells (`text_width`), none if it is wider.
2. Each builder above takes `&CellWidths` (its callers have a `Ctx`:
   pass `ctx.widths()`), measures `name_w` with `text_width`, and pads
   with `pad_cells` in place of the `format!` width.
3. Tests that call the builders pass `&CellWidths::NARROW`.

## Acceptance criteria

- [ ] `pad_cells` pads by cells: a unit test with `CellWidths::wide`.
- [ ] Each menu above with a wide name lines its next column up with an
      English one's (a unit test per builder, reading the menu's
      lines in cells).
- [ ] Every existing snapshot is unchanged.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `pad_cells`; each builder with a wide name.

## Completion notes
