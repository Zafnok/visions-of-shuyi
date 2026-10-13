---
id: "0247"
title: "Lead select: say what it shows as a view, painted by a skin"
type: feature
milestone: M1 Engine
model: sonnet-5
effort: medium
status: todo
blocked_by: ["0245"]
nick_input: none
completed:
---

# 0247 — Lead select as view + skin

## Context

The rest of 0245, split off as its scope said to: the three screens were
more than one PR, so 0245 did Preparations (the largest) and this ticket
and 0246 do the other two.
[ADR-0054](../../docs/adr/0054-screens-say-what-they-show-as-a-view-a-skin-paints-it.md)
is the pattern. Worked examples: `crates/ui/src/screens/preparations.rs`
with `preparations/view.rs` and `preparations/glyph.rs` (0245), and
`screens/save.rs` with `save/` (0241: a question in a box over the
screen).

## Nick input

None. Nothing the player sees changes.

## Scope

**In:** `crates/ui/src/screens/lead_select.rs`, with its typing box
(`NameBox`) and letter grid (`NameEntry`).

**Out (do not do):**
- Any change to what the screen shows, its text, keys or layout.
- A second skin or a way to choose skins.
- Class change (0246; *either order works*, they share no code).
- The battle screen and the dialogue screen (0242).
- Text literals to the language file (0234). *Either order works*: the
  view always holds finished strings.
- Pictures stay pictures (ADR-0043): the view names *what* (the
  portrait's id, its expression, whether it is the chosen one) and the
  skin places it with `portrait::draw_portrait` and decides how much to
  dim the one not chosen.

## Implementation steps

1. Add `lead_select/view.rs`: a `LeadSelectView` with the heading; the two
   genders in order, each `{ gender: LeadGender, label, portrait:
   PortraitId (LeadProfile::portrait_id), expression, chosen: bool }`;
   which row has the cursor (`Option<Row>`: `None` while the typing box or
   the grid is open, as nothing on the screen behind them is lit then);
   the name row (`label`, `first`, `family`); the Start button's text;
   the typing box while open (`title`, the name so far, the hint); the
   letter grid while open (`title`, the name so far, the rows of cells as
   their labels, the focused `(row, column)`); and the help line. The
   name-so-far is the name only: the `_` cursor after it is the skin's.
2. Add `fn view(&self, ctx: &Ctx) -> LeadSelectView` and
   `lead_select/glyph.rs` with `pub fn paint(ctx, &view, buf)`. Move every
   layout constant (`HEADING_ROW`, `FRAME`, `FRAME_Y`, `FRAME_X`,
   `LABEL_ROW`, `NAME_ROW`, `START_ROW`, `GRID_BOX`, `TYPE_BOX`), box,
   colour, the `[ … ]` around Start and the dimming amount into the skin.
   `draw` becomes `glyph::paint(ctx, &self.view(ctx), buf);`.
3. `LeadSelectScreen` opts into `Screen::as_any`. The game flow hosts it
   (`flow.rs`, `Stage::Lead`): add a `FlowScreen::lead_select()` accessor
   beside `preparations()` if a Harness test needs the hosted screen.
4. Add `("lead_select.rs", include_str!("../lead_select.rs"))` to the grep
   test in `screens/title/tests.rs`
   (`the_logic_modules_name_no_cells_or_colours`).
5. Tests, as 0245: behaviour tests in `lead_select/tests.rs` read the
   view; tests of a look (cells, colours, positions) move to
   `lead_select/glyph/tests.rs`, declared with `#[cfg(test)] mod tests;`
   at the end of `glyph.rs`; no assertion is dropped. Helpers a skin test
   needs from `lead_select/tests.rs` become `pub(super)`. Snapshot tests
   stay in `lead_select/tests.rs` so their `.snap` files keep their names
   and content.
6. Update `crates/ui/README.md` (the *The look is a skin* paragraph: add
   lead select to the converted screens).

## Acceptance criteria

- [ ] `lead_select.rs` doesn't import `UiColor`, `Rect`, `BoxStyle` or
      `Cell`, nor call `draw_portrait`; its `draw` is one call to its skin
      (the grep test is extended).
- [ ] No snapshot file's content changes.
- [ ] The view has a unit test of its content on the first frame, with
      the other gender chosen, with the cursor on each row, with the
      typing box open and with the letter grid open on a cell.
- [ ] `cargo xtask check-keys` and `check-text` pass; the `check-text`
      limit isn't raised.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the view's content; the skin's placement of what the old tests
  checked.
- Property: none.
- Snapshot / integration: the existing snapshots, unchanged.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
