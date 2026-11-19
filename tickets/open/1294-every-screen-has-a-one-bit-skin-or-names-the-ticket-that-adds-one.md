---
id: "1294"
title: Every screen has a one-bit skin or names the ticket that adds one, in a table of each screen's looks
type: infra
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
nick_input: none
completed:
---

# 1294 — Every screen has a one-bit skin or names the ticket that adds one

## Context

The friction sweep of 2026-11-17
([report](../../docs/research/friction-2026-11-17.md), "The Playdate gap
isn't visible", bucket B5 with 1180, 1207, 0173, 1170 and 0182) found
that nothing lists which screens the Playdate can draw. A PC screen
lands with a glyph skin only, and the one-bit gap shows up weeks later
as a catch-up ticket (1046, 1058, 1060, 1062, 1067, 1069, 1075, 1076).
About one catch-up PR follows every two or three PC visual PRs.

Re-counted on `main` (`adb04590`) for this ticket: 21 screen modules under
`crates/ui/src/screens/` (`actions.rs`, `files/` and `playdate/` are
shared helpers and the one-bit kit). They hold 24 `impl Screen<Shuyi>
for` types; options, key bindings and the layout picker have none,
since each returns one of the engine's ready-made screens from a
`screen()`, `first()` or `change()` function.
11 draw on the Playdate through a `Skins` (ADR-0091) or the skin set:
battle (with rewind and the shop), dialogue, familiarity (mode select's
skins), key bindings, lead select, mode select, options, profile,
title, town (the battle shop's skin) and the world map. 10 have no
one-bit skin: camp, class change, credits, game over (and To be
continued), layout picker, preparations, results, save, save import
and the voices question. The sweep's "12 of 20" counted folders.

Of those 10, 1046 covers preparations, results, game over, save and
credits, and 1067 covers camp and class change. Save import is never
offered on the Playdate (`ImportScreen::can_open`). The layout picker
and the voices question have no ticket.

Related: ADR-0038 (graphics are a skin), ADR-0054 (view and skin),
ADR-0083 (the skin set), ADR-0091 (the Playdate's screen skins), 1077
(one-bit skins wrap their own notes). An engine-side default one-bit
skin for the engine's widgets was proposed by the sweep, not filed (an
engine ticket needs Nick's OK first).

Whether the Playdate stays on the Act 1 playtest's gate (1064) is Nick's
call, asked through 0184. This ticket doesn't depend on the answer: the
table is useful either way.

## Nick input

None. Tooling and a test; nothing the player sees changes.

## Scope

**In:**
- A table in `crates/ui/src/screens/looks.rs`: one row per screen
  module under `crates/ui/src/screens/` (21 today), listing the
  module's `impl Screen<Shuyi> for` types (none for the three engine
  screens), with three columns:
  - `glyph`: `Own` (the game's `glyph.rs`) or `Engine` (the engine's
    glyph skin, as the layout picker).
  - `picture`: `Glyph` (glyphs in both looks) or `Sprites` (paints
    sprite items from asset files when the picture look is on: map
    skins, faces, backdrops, title art). Filled from the code as it is.
  - `one_bit`: `Own`, `Shared("<module>")` (another screen's skins),
    `Pending("<ticket id>")`, or `NotOnPlaydate("<where the code
    says so>")`.
- A test that a new screen must have a one-bit skin or name the ticket
  that adds one (steps 3–4).
- A dated note in ADR-0091 saying a new screen adds its row.

**Out (do not do):**
- Painting any one-bit skin. 1046 and 1067 own theirs.
- Deciding which screens the Playdate shows. `NotOnPlaydate` only
  records what the code already does; it never becomes a way to skip
  a skin.
- Deciding whether the Playdate is on the 1064 gate (Nick's, 0184).
- An engine default one-bit skin (proposed engine ticket, not filed).

## Implementation steps

Expected size: about 60 lines for the table, 120 for the test, 20 of
docs, and the ticket move: under 300 changed lines.

1. Write `looks.rs` with `pub struct ScreenLooks { module: &'static str,
   types: &'static [&'static str], glyph: Glyph, picture: Picture,
   one_bit: OneBit }` and `pub const SCREEN_LOOKS: &[ScreenLooks]`, one
   row per screen module, named by the module (`"camp"`, types
   `["CampScreen"]`; `"options"`, types `[]`). Where one module's types
   differ (game over and To be continued, save prompt and slot picker),
   they share the row's values. Declare the module in `screens/mod.rs`.
2. Fill the rows from the code on `main`. Today's one-bit values:
   - `Own` or `Shared`: the 11 screens in Context (their modules' extra
     types, such as `RewindScreen`, `ShopScreen` and `GuidesScreen`, go
     in the row's `types`).
   - `Pending("1046")`: preparations, results, game over, To be
     continued, save prompt and slot picker, credits.
   - `Pending("1067")`: camp, class change.
   - `NotOnPlaydate("ImportScreen::can_open")`: save import.
   - Layout picker: `NotOnPlaydate` only if the code never opens it
     without a keyboard (ADR-0091 §6 leaves `Layout` out of Options;
     check the first-launch path too). If it can open, file a ticket
     with `write-ticket` and name it.
   - Voices question: `Pending("1048")` if 1048's scope shows the
     question on the device; if not, file a ticket and name it. Whether
     the Playdate asks it at all follows the voice decisions already in
     `docs/design/`; if they don't say, write the ticket's question as
     open for Nick, never decided here.
3. Test `every_screen_has_a_looks_row` (in `crates/ui/src/screens/looks.rs`'s
   tests, reading the sources with `CARGO_MANIFEST_DIR`): every screen
   module under `src/screens/` (all but `mod.rs`, `actions.rs`, `files`
   and `playdate`) has exactly one row; every `impl Screen<Shuyi> for X`
   is in its own module's `types`; and no row names a module or type
   that is gone. A new module that returns an engine screen gets a row
   too, so the test doesn't rely on the `impl`.
4. Test `one_bit_rows_match_the_code`:
   - An `Own` or `Shared` row's module paints through a `Skins`
     (`SKINS.paint`) or reads `ctx.game.skin_set`; a `Pending` or
     `NotOnPlaydate` row's module doesn't (so a row can't fall behind when a
     skin lands).
   - A `Pending` id is a file in `tickets/open/` (four digits, the
     `NNNN-*.md` form); one only in `tickets/done/` fails with "its
     ticket is done: paint it or name the next ticket".
   - A `NotOnPlaydate` reason names a function that exists in the
     screen's file.
   - Each failure names the screen and what to do.
5. Add the dated note to ADR-0091 (*Consequences*): a new screen adds
   its row; the one-bit column says `Own`, `Shared`, `Pending` with a
   ticket, or `NotOnPlaydate` with the code that says so.

## Acceptance criteria

- [ ] `SCREEN_LOOKS` has one row per screen module on `main`, with the
      values in step 2 (or the tickets filed for the layout picker and
      the voices question, named in the completion notes).
- [ ] Deleting the Playdate painter from `mode_select`'s `SKINS` (tried,
      noted) or adding a screen module or type with no row fails a test that says
      what to do.
- [ ] A row naming a done ticket fails (fixture or tried, noted).
- [ ] ADR-0091 has the note.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `every_screen_has_a_looks_row` and `one_bit_rows_match_the_code`,
  reading sources and `tickets/`; no frame is drawn.

## Completion notes

*(Filled in by the session that completes the ticket: the final table,
tickets filed for screens with none, deviations.)*
