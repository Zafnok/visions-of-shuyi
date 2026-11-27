---
id: "1381"
title: "Every other screen and the dialogue box in the UI kit's frames"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1380"]
nick_input: none
completed:
---

# 1381 — Every other screen and the dialogue box in the UI kit's frames

## Context

1380 builds the picture frames (engine nine-slice frame, `cargo xtask
ui-import`, `assets/ui/frames.ron`, the frame name → `PanelLook`
helper) and puts the battle's windows on them. The look is in
`docs/design/look-and-feel.md`, *Windows (the picture look)*, decided
by 1379. This ticket puts every other window of the PC game on the same
frames. The game will then have one look outside the map.

**What 1379 picked for these screens** (the table and Nick's words in
*Windows (the picture look)*; pictures in the private repository's
`library/`):

| Window | Picture | Slices (l, t, r, b) | Fill |
| ------ | ------- | ------------------- | ---- |
| Menus and other screens' boxes | Pixel Quest `F-UI_Panel_H.png` (as the side panel) | 22, 33, 22, 32 | its plum |
| Full-screen Options | Pixel Quest `F-UI_Panel_H.png` | 22, 33, 22, 32 | **slate grey-blue**: the panel's cool pixels recoloured, the gold kept (1379's mock: hue 215 degrees, saturation x0.45) |
| Dialogue box | Pixel Quest `F-UI_Panel_G.png` | 22, 29, 22, 27 | its plum |
| Dialogue portraits | tiopalada's *Dragon Regalia* `library/tiopalada/ui/dragon-regalia/20240707dragon9SlicesA.png` (CC0): its blue fill made the windows' plum, its top-right 72x72 corner mirrored to the other three | 34, 38, 34, 38 | plum |
| Speaker, portrait and title name plates | Post-Baroque `UI_TitleB1.png`: 16-pixel end caps, a plain body (column 60, rows 8 to 24) as wide as the name | | its dark |

Nick on the portraits: *"dragon regalia 1.5x mirrored as shown but with
the portraits pushed up a bit so they dont overlap the dialogue box. also
ensure the names fit inside the nameplates"*. So: the portrait frames are
drawn at **1.5x the font's pixel** (at window scale 2; the nearest whole
screen pixel at other scales, the font's pixel at scale 1); the portrait
windows move up (1379's mock: 40 screen pixels at scale 2) so their
frames clear the dialogue box; each name and its plate sit inside the
portrait's frame, the name between the plate's caps. Fills are solid.

**The kits are in the private repository from commit `b68863cc` on**
(its PR #42, `library/barely-games/`, `library/tiopalada/`,
`library/opengameart/`, `library/untied/`, `library/hypnobius/`). When
1379 merged, `assets-private.rev` was older (moving it then would have
carried other tickets' unmerged content), and `cargo shuyi private-assets
--library` checks out the pinned commit. So first check that the pin is at
or past `b68863cc`; if not, check out the private repository's `main` in
`assets-private/` and move the pin (`cargo shuyi private-assets --pin`) in
this ticket's PR, once no open public PR still needs an older pin for its
private content.

## Nick input

None (1379 decided the look). Pictures in the PR from `frame-png` with
`private-assets`, kept local; anything left to judge by playing goes on
the Act 1 playtest's checklist (1064).

## Scope

**In:** each screen's boxes on 1380's helper, with the frame 1379 names
for it:
- **Dialogue:** the text box and the speaker's name plate, the two
  portrait frames and their name plates (`screens/dialogue/glyph*`), the
  log (`dialogue/log/glyph.rs`), the guide's tip and lesson box.
- **Menus:** title menu, options (the engine's `screens/options.rs`
  skin: if it can't take a frame name, that is an engine-ergonomics
  item for the `engine-friction` skill), key bindings, files/save/save
  import, language list, mode select, voices question, profile, credits.
- **Between battles:** preparations (its lists and shop), camp, town,
  results, class change, familiarity, lead select, chapter card, game
  over, world map's windows.
- Frames 1379 named that 1380 didn't need, added to
  `assets-src/ui/` and `frames.ron` (re-run `ui-import`, move the pin).

**Out (do not do):**
- Battle windows (1380); the cursor, gauges, icons (1382); the font.
- The Playdate's skins; the title's logo and backdrop (ADR-0059);
  the world map's place icons (1082).
- Any change to a window's contents, size or place.

## Implementation steps

1. Estimate the lines (ADR-0092): about 600 (each screen's look moved
   onto the helper: a few lines per box, about 30 boxes, plus tests).
   Measure with `cargo xtask code-lines`. If past about 2,000 changed
   lines, split by the groups above (`write-ticket`) before coding.
2. Screen by screen, the `glyph.rs` skin's `panel` calls take the
   helper's `PanelLook`. Screens without a box keep none.
3. `frame-png` each screen in both looks and check by eye: text inside
   frames on its cells, nothing cut off, the name plate fits the
   longest name in each language pack (`test` pack included).

## Lesser targets

Playdate: unchanged. `reduced`/`minimal` tiers: as 1380 decided.

## Acceptance criteria

- [ ] Every screen in *In* draws the picture frame in the picture look
      (a Harness test per screen reads its sprite items).
- [ ] Glyph-look and no-art snapshots unchanged.
- [ ] The longest speaker name of every language pack fits its plate
      between the caps (a test).
- [ ] The portrait frames clear the dialogue box (a test of their
      rects).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: one Harness test per screen for the frame's
  sprite items; existing snapshots unchanged in the glyph look.
- Unit: the name plate's width for the longest name per pack.

## Completion notes

*(Filled in by the session that completes the ticket.)*
