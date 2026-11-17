---
id: "0254"
title: "Key caps in every hint: the game turns on the engine's key-cap style, with a bold face and the arrow cluster"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0250"]
nick_input: none
completed:
---

# 0254 — Key caps in every hint

## Context

Nick decided how a key's name looks in hints in 0250 (2026-11-16):
`docs/design/look-and-feel.md`, *Key names in hints*. In short: every
key's name sits in a small rounded box like a keyboard key, in bold,
single letters in capitals (`F`, not `f`), longer names as written
(`Space`, `Shift+Space`), 2 px between the letters and the box on all
four sides, nothing beside a cap hidden under it; the arrow keys as one
picture of the arrow cluster (an upside-down T of four 8 × 8 px caps);
a cursor on other keys as four caps (`W A S D`).

The engine fills key names into text and draws the text, so the engine
builds it first:

- engine#2347: key names as `Key` spans in `StyledText` (`fill_styled`,
  `help_line_styled`, `Ctx::text_styled`, `print_styled`), the engine's
  own screens switched over, and the palette's `key` / `key_bg`;
- engine#3215: the key-cap style a theme turns on (box, bold, capitals,
  even margins, room around a cap; engine PR #463), engine#3216 a bold
  face in the font atlas (`font-atlas --bold`, #468), and engine#3217
  the direction cluster with its picture (`KeyCapStyle::with_arrows`,
  `TextRole::Arrows`, #477). All three are in the pin since the pin
  move of 2026-11-17, which also made `{Cursor}` on other keys read
  `w a s d` (a space between keys).

This ticket is the game's half: move the pin past both, switch the
game's own hints to styled text and turn the style on. The mockups'
colours were a box `#4a402e` with a 1 px edge `#807052`, the name in
`text` (`#d0c4a8`) (*tunable*).

## Nick input

None: decided in 0250. Verified by tests and `cargo shuyi frame-png`
pictures in the PR; what is left to judge by playing goes on the Act 1
playtest's checklist (1064).

## Scope

**In:**
- ~~Move the engine pin past engine#3215 (`engine-change` §7); fix what
  the pin move breaks.~~ Done by the pin move of 2026-11-17. (engine#2347 is in the pin since the pin move of
  2026-11-16, which already switched the game's call sites to its
  `StyledText` with no key roles: `.into()` and `.plain()`.)
- The game's views that name keys hold `StyledText`, built with
  `ctx.text_styled` / `help_line_styled`: the battle's help bars
  (`crates/ui/src/screens/battle/`), the text box's corner (`b log
  f ▼`, the game builds `WaitView` and `SkipView`), tips and lessons
  (`crates/ui/src/tips.rs`, `tips/view.rs`), the shops, Guides, save,
  town and class-change screens' help, every other place
  `key_name` / `all_key_names` / `cursor_keys_name` / `help_line` /
  `text_with` names a key for the player (step 1 lists them). Skins
  paint them with `print_styled`.
- The theme turns on the key-cap style with Nick's settings: bold,
  capitals, 2 px margin, the colours above.
- The bold face: Terminus Bold (`ter-u16b.bdf`, the same family and
  license as the regular face, SIL OFL 1.1) in `assets-src/`, the atlas
  built again with `cargo shuyi font-atlas`; `THIRD_PARTY_ASSETS.md` and
  the credits name it.
- The arrow cluster's picture: a 24 × 16 PNG in `assets/` drawn for
  this (four 8 × 8 caps in an upside-down T, an arrow in each, the
  caps' colours), named in the theme.
- `look-and-feel.md` unchanged unless the build differs from it (then
  say how in the completion notes).
- The `keyboard-input` skill: text that names a key uses the styled
  functions, so the key gets its cap.

**Out (do not do):**
- A controller's buttons (0255) and the Playdate's one-bit look (0256):
  both keep today's text.
- The help bar's layout (0445).
- Changing which keys are bound.

## Implementation steps

1. List every place the game names a key to the player:
   `grep -rn "key_name\|all_key_names\|cursor_keys_name\|help_line\|text_with\|help_keys" crates/ui/src crates/app/src`
   (leave out tests and the debug tools, which 2347 leaves plain too).
2. ~~Move the pin (`engine-change` §7, `engine-pin` skill for the PRs it
   passes); build and fix.~~ Done by the pin move of 2026-11-17.
3. Switch each view from step 1 to `StyledText`; tests that compared a
   hint with a string compare its `plain()`.
4. Add the bold face and build the atlas; add the cluster picture;
   turn the style on in the theme.
5. Pictures: `cargo shuyi frame-png` of the battle (`--battle ch01`),
   the text box (`--scene ch01_prebattle --wait 3`) and a tip
   (`--battle ch01 --tips --wait 1`), in both looks (the picture look
   locally only, ADR-0040), checked against the decision.
6. `cargo shuyi check-keys`, the gates.

**Expected size:** about 600 production lines (the views' hints switched
to styled text, about 15 places at 20–40 lines each, and the theme's
settings), plus tests and snapshots. **Split:** past 2,000 changed
lines, ship the pin move, the theme and the battle's help bars, and move
the other screens to a new ticket blocked by this one.

## Acceptance criteria

- [ ] Every key name a player sees on the PC (both layouts, keyboard)
      is a `Key` span: a test lists each switched view's hint spans and
      finds the keys `key_name` and its siblings give, in order.
- [ ] A help-bar snapshot and `frame-png` pictures show the caps as
      decided: bold capitals, even margins, `.` and `,` after a cap
      whole, the arrow cluster in a tip, four caps for a cursor bound to
      W A S D.
- [ ] With a controller in use, hints read as today (0255).
- [ ] The Playdate build's hints read as today (0256).
- [ ] `cargo shuyi check-keys` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each switched view builds `Key` spans for its keys.
- Snapshot: the battle help bar, the text box corner, a tip, Options;
  each reviewed.
- Integration: a Harness run with the cursor rebound to W A S D shows
  four key spans in a tip.

## Completion notes

*(Filled in by the session that completes the ticket.)*
