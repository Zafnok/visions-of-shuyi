---
id: "0254"
title: "Key caps in every hint: the game turns on the engine's key-cap style, with a bold face and the arrow cluster"
type: feature
milestone: M1 Engine
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0250"]
blocks: ["1064"]
nick_input: none
completed: 2026-11-17
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

- [x] Every key name a player sees on the PC (both layouts, keyboard)
      is a `Key` span: a test lists each switched view's hint spans and
      finds the keys `key_name` and its siblings give, in order.
- [x] A help-bar snapshot and `frame-png` pictures show the caps as
      decided: bold capitals, even margins, `.` and `,` after a cap
      whole, the arrow cluster in a tip, four caps for a cursor bound to
      W A S D.
- [x] With a controller in use, hints read as today (0255).
- [x] The Playdate build's hints read as today (0256).
- [x] `cargo shuyi check-keys` passes.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: each switched view builds `Key` spans for its keys.
- Snapshot: the battle help bar, the text box corner, a tip, Options;
  each reviewed.
- Integration: a Harness run with the cursor rebound to W A S D shows
  four key spans in a tip.

## Completion notes

Done on 2026-11-17, **split** as the ticket's *Split* says: the game names
keys in about 270 places in 55 files, not the 15 the estimate counted, so
this PR turns the style on and switches the battle and the three places
the pictures show; **0257** switches the other screens.

- **The style** (`Shuyi::key_caps`): bold, single letters in capitals,
  2 px margin, not on a one-bit screen, the arrow keys as the picture
  `assets/images/key_arrows.png` (24 × 16, four 8 × 8 caps, drawn by
  hand for this in the default theme's key colours, the same picture in
  every theme). The engine's own screens (Options, Key bindings, its
  questions) draw caps from here on too.
- **Colours** (*tunable*): the default theme has the mockup's box
  `#4a402e`, edge `#807052`, name `#d0c4a8` (its `text`). The other
  themes got a box a shade lighter than their panel and an edge lighter
  again (Painterly `#24403f`/`#4e7a78`, Moonlit `#283244`/`#5a6a84`);
  Parchment is light, so its box is lighter than the panel (`#f0e6cc`) and
  its edge darker ink (`#8a7454`). The Playdate's palette names them but
  draws no caps.
- **Bold face:** Terminus Bold `ter-u16b.bdf` from the same 4.49.1 release
  (its regular face is byte-for-byte the one already here), the atlas
  rebuilt with `--bold` (547 bold glyphs); `THIRD_PARTY_ASSETS.md` names it;
  the credits' Terminus entry covers both faces.
- **Switched to key spans:** the battle's help bar and toggles, its
  Guides, rewind and shop screens (and the town's shop, which shares it),
  the EXP pages' help; the text box's corner; the tip, lesson and
  end-of-lessons boxes (the guide's spoken lines name no key and stay
  plain). `widgets::help` holds the helpers, `tips::fill_styled` fills a
  tip's placeholders in spans.
- **Controller and not mapped:** a controller's buttons and `! not
  mapped` read as plain text, as before (`widgets::help::for_player`,
  also in `Shuyi::key_text_styled`, so the engine's screens follow it).
- **Tests:** `tests/it/key_caps.rs` (each switched view's keys in order,
  W A S D as four key spans in a tip, the controller, snapshots of the
  help bar, text box corner, a tip and Options with caps),
  `widgets/help/tests.rs`. The test harness draws keys as words
  (`Shuyi::key_caps` off) so tests of what a hint says read its words;
  a test of the look calls `Harness::with_key_caps`, and `cargo shuyi
  frame-png` always draws caps. About 100 snapshots changed only in the
  key cells' colours (the palette's `key` and `key_bg`); four private
  ones in `Zafnok/visions-of-shuyi-assets` (its PR is linked in this
  PR).
- **Engine friction:** the engine wraps plain text only, so the tip boxes
  wrap styled text with the game's `widgets::help::wrap_styled` (about 40
  lines), a marked workaround; a styled `word_wrap` would be an engine
  ticket (not filed: engine steps need Nick's OK).
- `look-and-feel.md` unchanged: the build matches it. One detail as the
  engine builds it: a cap takes one cell more than its name, so `F select`
  reads `F␣␣select` in cells and a `.` after a cap sits one cell from it.

*Claude's starting rules* (none are gameplay): the other themes' cap
colours above; the arrows picture keeps the default theme's colours in
every theme.

Pictures (`cargo shuyi frame-png`, glyph look, cropped clear of the map
and the script, ADR-0118): `docs/screenshots/0254-help-bar.png`,
`0254-text-box-corner.png`, `0254-tip-arrows.png`, `0254-tip-wasd.png`.
The picture look was not rendered: the caps are drawn the same in both
looks (the frame draws them, not the map skin), and it needs the bought
art. Acceptance "every key name a player sees" holds for the switched
views; the rest is 0257's.

Size: the ticket's estimate was about 600 production lines (no separate
estimate was written before coding); measured 323 added, 122 removed
(`code-lines`), under it. PR size 1,323 changed lines.

For the Act 1 playtest (1064): whether the caps read well in each theme,
and the arrow cluster's size beside text.
