---
id: "1077"
title: "One-bit skins: each skin wraps the battle notes itself, lists scroll by the engine's rule, one menu text and the window box on the engine's panel"
type: infra
milestone: Post–Chapter 1
model: implementation
effort: medium
status: todo
blocked_by: ["1039"]
nick_input: none
completed:
---

# 1077 — One-bit skins wrap their own notes and scroll by the engine's rule

## Context

From the retro of the last 50 PRs (2026-10-22). The Playdate's battle
skin (1038, #392) landed three days after 1160 (#363) moved the game's
13 hand-written list cursors and 6 scroll windows onto the engine's
`ListCursor` and `Scroll`, and 1165 (#370) moved the PC skins onto the
engine's drawing helpers. The one-bit skins (1045's and 1038's) missed
both, and one look decision sits in the battle's view:

1. **The view wraps the battle notes at the PC's width.**
   `BattleScreen` puts `notes::note_lines` into the view, wrapped to
   `NOTE_WIDTH` (`crates/ui/src/screens/battle/notes.rs`). The Playdate
   skin then joins them back with `unwrap_notes` and wraps again at its
   own width (`note_lines` in `screens/battle/playdate.rs`). How wide a
   line is belongs to the skin (ADR-0054), and the round trip puts a
   space inside any long word the first wrap split.
2. **A fourth scroll rule.** `pd::first_shown`
   (`crates/ui/src/screens/playdate.rs`) keeps the focus in the middle
   as `focus - room / 2`; the engine's `Scroll::Centred` is `focus -
   (rows - 1) / 2`. With an even window (the Options and Key bindings
   skins show 12 rows) the two differ by a row. 1160 kept it because it
   "rounds the other way", but the middle rule is a *the agent's starting
   rule* (`docs/design/playdate.md`, 1045's rules) that says nothing
   about rounding: nobody decided the difference. Its users: the
   battle's one-bit menus (`shown` in `screens/battle/playdate.rs`, when
   a menu has no window of its own), `options/playdate.rs`,
   `key_bindings/playdate.rs`.
3. **Two copies of a menu item's text**: `item_text`
   (`screens/battle/playdate.rs`) and `menu_text`
   (`screens/playdate.rs`) are the same function.
4. **The window box copies the engine's panel.** `pd::window` fills a
   rectangle, draws a double box and prints a title two cells in: the
   engine's `GlyphBuffer::panel` with `PanelLook::new(BoxStyle::Double,
   …)`.

The larger part, the one-bit skins of the engine's own screens moving
into the engine, is 1184 (after engine#2371); this ticket is what the
game can tidy now.

## Nick input

None. The one visible change (an even scrolled list's window one row
off from today) is in a *the agent's starting rule*; list it in the
Completion notes for the Act 1 playtest's checklist (1064). Verified by
the snapshots.

## Scope

**In:**
- The battle's view carries the notes' **texts** (one string a note,
  in the player's language); each skin wraps them: the PC glyph skin with
  `wrap_notes(texts, NOTE_WIDTH)`, the Playdate skin at its width.
  `unwrap_notes` goes. Its `every_battles_notes_fit_the_map_view` test
  wraps the texts as the PC skin does.
- `pd::first_shown` goes: the one-bit skins ask the engine for the
  window (`ListCursor` with `Scroll::Centred` and a window of `room`, or
  a public engine function that gives the first row; if the engine has
  none public, use the cursor). Where the logic already knows the rows
  (a menu built with `Menu::showing`), the skin reads `first_shown` from
  the view as it does now.
- One menu item text function, in `screens/playdate.rs`, used by both.
- `pd::window` on `GlyphBuffer::panel` if its title sits where
  `window` puts it (two cells in, a space either side, cut to fit);
  otherwise leave `window` and say why in the Completion notes.
- The one-bit skins that 1039 adds (forecast, unit info, level-up,
  rewind) are swept the same way if they carry any of the above.
- Snapshot changes, reviewed.

**Out (do not do):**
- Moving one-bit skins into the engine (1184).
- The yes/no prompts and the PC skins' leftovers (1185).
- Changing the PC look.

## Implementation steps

1. **Read** `notes.rs`, the two `note_lines` calls in
   `screens/battle/mod.rs` (the view and the playback panel), the view
   type that holds them (`NotesView`), `screens/battle/playdate.rs`
   (`note_lines`, `shown`, `item_text`, `paint_menu`),
   `screens/playdate.rs` (`first_shown`, `menu_text`, `window`), and the
   engine's `widgets/list.rs` (`ListCursor`, `Scroll`) and
   `frame/draw.rs` (`panel`, `PanelLook`).
2. **Notes**: change `NotesView.lines` to `texts`; the PC skin wraps
   where it paints; the Playdate skin wraps from the texts. Update the
   fit test.
3. **Scroll**: replace each `pd::first_shown` call; delete it and its
   tests; update the snapshots that move by a row.
4. **Menu text** and **window** as in Scope.
5. Run the PC and Playdate snapshot tests; review every changed
   snapshot (the PC ones should not change).

Expected size: about 40 production lines changed, net fewer (about 60
removed).

## Acceptance criteria

- [ ] `grep -rn "unwrap_notes\|fn first_shown" crates/ui/src` finds
      nothing.
- [ ] A battle note with a word longer than the PC's width wraps on the
      Playdate without a space inserted in it (a test).
- [ ] No PC snapshot changes.
- [ ] One function gives a one-bit menu item's text.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the Playdate skin's note wrapping from texts; the window rule at
  an even and an odd room.
- Snapshot: the one-bit Options, Key bindings and battle menu snapshots
  (reviewed).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
