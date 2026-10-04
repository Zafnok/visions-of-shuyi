---
id: "0445"
title: "Battle help bar: the danger-zone hint on the left, auto-end in a column with end turn"
type: feature
milestone: M3 Battle UI
model: sonnet-5
effort: medium
status: todo
blocked_by: []
nick_input: sign-off
completed:
---

# 0445 — Battle help bar: the toggles line up with the key help

## Context

The battle screen's bottom two rows (`HELP_BAR`, rows 30 and 31) are today:

```
                                                     w danger zone: OFF · Shift+Space auto-end: OFF
 f select · e info · s next unit · r rewind · d menu · Space end turn                      F2 debug
```

Row 30 is the *toggles line* (`BattleScreen::status`), right-aligned; a
message (a toast, a combat or cast preview: `BattleScreen::message`) is
printed at its left when there is one. Row 31 is the key help
(`BattleScreen::help`), from column 1.

Nick, looking at a battle frame (2026-10-04, ticket 0437): "I wonder if we
can left align the w danger zone hint and align shift+space auto-end to a
column with space end turn to make it more visually cohesive". So:

```
 w danger zone: OFF                                    Shift+Space auto-end: OFF
 f select · e info · s next unit · r rewind · d menu · Space end turn                      F2 debug
```

The danger-zone hint starts where the key help starts, and the auto-end
hint starts in the column where the end-turn hint starts below it: the two
things about ending a turn read as one column.

Two things his sentence doesn't settle, which need his eye on a rendered
frame (`docs/design/look-and-feel.md`, *Nick's words*: he judges pictures):

1. **Messages.** A message is printed at column 1 of row 30 today, where
   the danger-zone hint would now be.
2. **When the end-turn hint moves or isn't there.** `Space end turn` is
   the last hint of the *browsing* help only, and the browsing help has
   four forms of different length (over a ready unit of ours, over an
   enemy, over another unit, over an empty tile: `help_idle`). With a unit
   selected, a menu open, a prompt up or the enemy phase running there is
   no end-turn hint at all.

Code:

- `crates/ui/src/screens/battle/mod.rs`: `status` (the toggles line),
  `help` and `help_idle` (the key help), `message`, and the end of `draw`
  (fills `HELP_BAR`, prints the status right-aligned at
  `HELP_BAR.w - 1 - w`, the message at column 1, the help on `HELP_ROW`,
  then `draw_debug_hint`).
- `crates/ui/src/screens/battle/layout.rs`: `HELP_BAR`, `HELP_ROW`.
- `crates/ui/src/widgets/help.rs`: `help_line`, `SEPARATOR`, `key_name`.

Keys are never hard-coded (`keyboard-input` skill): every key name comes
from the player's keymap, differs between the two layouts, can be rebound
and is a button name on a controller. So **columns are worked out from the
text each frame**, never constants.

## Nick input

**Sign-off, with options first.** Before building, render the options of
step 1 and ask Nick to pick (`ask-nick`; frames from
`cargo xtask frame-png`, which need no bought art: the bar is the same on
the glyph look). After the build he looks at it on Pages.

## Scope

**In:**
- Where the two toggle hints are printed on the battle screen, as Nick
  picks in step 1.
- Where a message goes on that row, as Nick picks.
- `docs/design/look-and-feel.md` (*Screen layout*) records it.

**Out (do not do):**
- The words of the hints, their colours, or which hints the key help
  lists.
- Any other screen's help line.
- The zoom toggle's hint (0439): **either order works.** If 0439 is done
  and added a hint to the toggles line, place it by the rule Nick picks
  here and say so in the notes; if not, add a line to 0439 naming this
  ticket's rule.
- Moving the hints' text into the language file (0234): **either order
  works.** Use whatever `status` uses for its words when this is built.
- Saving the auto-end setting (0805).

## Implementation steps

1. **Options for Nick** (don't build before he answers). Render, on the
   Quick Battle, each open point as two or three real frames:
   - *Messages:* (a) a message takes the row and the toggle hints are
     hidden while it shows; (b) the message is printed right of the
     danger-zone hint; (c) the message keeps column 1 and the danger-zone
     hint shows only when there is none. Show a toast and a cast preview.
   - *Auto-end's column:* (a) it follows the end-turn hint: above it
     wherever it is, and where there is no end-turn hint it sits one
     separator after the danger-zone hint; (b) the browsing help always
     puts `end turn` in one column (padded to the longest of its four
     forms), auto-end is always above that column, and it stays there in
     other modes; (c) as (b), but both toggle hints are hidden when the
     key help has no end-turn hint. Show browsing over a unit of ours,
     over an empty tile, and a unit selected.
   - Also show one frame with the left-handed layout and one with a
     controller (`frame-png --layout left`, `--pad`), where the key names
     are other lengths.
   These are Claude's suggestions, not decisions: Nick may want another.
2. Record his answers in `docs/design/look-and-feel.md` (*Screen layout*)
   with the date and his words, and remove the item from *Open
   sub-questions*.
3. **Build.** In `mod.rs`:
   - Split `status` into the two hints (`danger_hint`, `auto_end_hint`),
     each still built with `key_name` from `ctx.help_keys()`.
   - Get the end-turn hint's column from the help line's own parts, not by
     searching the string: e.g. have `help_idle` build its hints as a
     `Vec` and add a helper in `widgets/help.rs`,
     `hint_column(hints, index) -> usize` (the characters of
     `help_line(&hints[..index])` plus `SEPARATOR` when `index > 0`), so a
     label that happens to contain "end turn" can't confuse it. Count
     characters, not bytes.
   - Print the danger-zone hint at column 1 of `HELP_BAR.y`, the auto-end
     hint at `1 + column`, and the message as Nick picked.
   - Nothing may overlap or run off the row: if the auto-end hint would
     pass column `HELP_BAR.w - 1`, or start before the danger-zone hint
     ends plus a separator, fall back to printing it one separator after
     the danger-zone hint.
4. Update `crates/ui/README.md` if it describes the bar, and the snapshots
   that show the bar (most battle snapshots): **read** the changed ones;
   only rows 30 and 31 may differ.

## Acceptance criteria

- [ ] Browsing over a ready unit of ours with the right-handed layout: row
      30 has `w danger zone: OFF` from column 1 and
      `Shift+Space auto-end: OFF` starting in the column where
      `Space end turn` starts on row 31 (a test names both columns from
      the frame, not from the code under test).
- [ ] The same holds with the left-handed layout, with End turn rebound to
      a longer key, and with a controller.
- [ ] Each case Nick decided in step 1 (a message showing; no end-turn
      hint) has a test of what the row shows.
- [ ] For every mode of the battle screen in the existing snapshot tests,
      the two hints, the message and the debug hint never overlap and stay
      inside the row.
- [ ] `cargo xtask check-keys` and `check-text` pass; no key is written as
      a literal.
- [ ] Only rows 30 and 31 changed in the snapshots.
- [ ] `look-and-feel.md` records Nick's answers.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: `hint_column` (first hint, later hints, wide characters); the two
  hints' columns for both layouts, a rebound key and a controller; the
  fallback when the hints would overlap.
- Property: for any key names up to the longest a keymap allows, the row's
  parts don't overlap and stay inside 100 columns.
- Snapshot / integration: the battle snapshots, reviewed; a Harness test
  that toggles the danger zone and auto-end and reads the row.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
