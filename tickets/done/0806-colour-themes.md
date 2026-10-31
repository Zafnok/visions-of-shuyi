---
id: "0806"
title: Colour themes option (C, D, E, G palettes)
type: feature
milestone: M7 Chapter 1
model: implementation
effort: medium
status: done
blocked_by: ["0805"]
nick_input: sign-off
completed: 2026-10-12
---

# 0806 — Colour themes option

## Context

In ticket 0011 Nick chose palette **D "Earthy painterly"** as the game's look,
and asked for four palettes to be offered as a player setting: **C Rich &
painterly, D Earthy painterly (default), E War-table parchment, G Moonlit**
(`docs/design/look-and-feel.md`). Their mockup values are in that file, and
screenshots are in `docs/screenshots/0011-theme-{c,d,e,g}.png`. Code refers to
colours only by palette name (ADR-0018), so a theme is a data swap. Options
screen: 0805.

## Nick input

**Sign-off:** Nick switches between the four themes in the options screen and
says whether each still looks like its mockup; tweaks become `tuning` tickets.

## Scope

**In:**
- Three more palette files, e.g. `assets/data/palettes/{painterly,parchment,moonlit}.ron`,
  each with every name in `palette.ron`. D stays `assets/data/palette.ron` (or
  move all four into `palettes/` and update `PALETTE_PATH`: pick one and keep a
  single code path).
- A `theme` setting (`Settings`, 0805), default D, saved like the others.
- Loading the chosen palette at startup and on change (rebuild `Ctx.palette`).

**Out (do not do):**
- Colour-blind palettes (a later ticket).
- Portrait colour variants per theme: portraits keep their own colours.
  The same goes for every image (ADR-0038): a theme recolours glyphs and
  the UI, never a picture or a tileset.

## Implementation steps

1. Fill each theme file from the tables in `look-and-feel.md`. Derive the names
   the tables don't list (`ally`, `neutral`, `heal_range`, `danger_zone`,
   `exp_bar`, `stone*`, `road*`, `path`, `hp_mid`, panel colours) to match that
   theme's feel. E is **light**: keep `black` as its light background only if
   `black` means "console background" everywhere; otherwise add a
   `background` name and switch the renderer/screens to it in this ticket.
2. Content validation: every theme file has exactly the same names as D.
3. `Settings.theme: Theme` (`Painterly | Earthy | Parchment | Moonlit`), an
   options row with Left/Right to change it, applied immediately.
4. Check the dimming code (acted units, listener portraits) lerps toward the
   background colour so it works on the light theme (ADR-0018).

## Acceptance criteria

- [x] All four themes selectable; the choice persists across restart.
- [x] Content test: every theme has the same colour names as D.
- [x] Snapshots (colour names) of the Quick Battle screen in each theme.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: theme file validation; settings round-trip.
- Snapshot / integration: Quick Battle in each theme; Harness test that
  changing the option changes `ctx.palette`.

## Completion notes

- `trpg_content::Theme` (Painterly, Earthy, Parchment, Moonlit) names each
  palette file; D stays `data/palette.ron`, the others are in `data/palettes/`
  (`Theme::path` is the one code path). `Content.themes` holds all four, and
  content validation fails if a theme's colour names differ from D's.
- `Settings.theme` (saved, default Earthy), an Options row **Colors** after
  Map look, applied at once: `Ctx` rebuilds `Ctx.palette` on load and on
  change, and `app` updates the renderer's background every frame.
- `black` is the console background everywhere, so E uses a light `black`
  and no `background` name was added. E's `white` is dark ink (strongest text).
- Dimming: glyph units already fade toward the tile background; the map
  cursor's blink faded toward black and now fades toward `black` (the
  theme's background). Portraits fade by opacity. Pictures are never recoloured
  (sprite "acted" grey-dimming is unchanged).
- Colours the mockups didn't list (ally, neutral, heal_range, danger_zone,
  exp_bar, stone, road, path, hp_mid, effect marks, etc.) are my derivations.
- The agent's starting rules: the row is named "Colors" and the themes are named
  as in Nick's list; Restore defaults puts the theme back to D.
- Tests: content (same names, themes differ), settings round-trip, options
  unit + scripted (palette changes, saved, relaunch, restore), Quick Battle
  snapshots in each theme. Existing Options snapshots gained the new row.
