---
id: "0061"
title: "Decide: the Playdate version of Chapter 1 (screen, buttons, crank, what's left out)"
type: design-decision
milestone: Design decisions
model: reasoning
effort: medium
status: done
blocked_by: []
nick_input: decision
completed: 2026-10-15
---

# 0061 — Decide: the Playdate version of Chapter 1

## Context

Nick, 2026-10-15: "I wanna rush a playdate port!!! how can we get ch1 on
playdate asap?!" He unparked the port (1025). The technical test (1026)
runs at the same time as this ticket. Neither one waits for the other.

His first thoughts, 2026-10-07 (1025): "as far as controls limitations we
have solutions like chording A+B if we need to or otherwise have A/B off
of a unit act differently like it already does and the rewind mechanic
can use the winding mechanism on the side. […] if we just reuse the
retro glyph stuff it could totally scale down to Playdate I think".

What the Playdate gives us (check against the current SDK docs before
asking):

- **Screen:** 400 × 240 pixels, 1-bit (black or white, no greys, no
  colour), 2.7 inches. Today's game is 100 × 32 cells of 8 × 16 pixels
  (800 × 512). With the same cell size the Playdate fits 50 × 15 cells.
  8 × 8 cells give 50 × 30, and 6 × 8 cells give 66 × 30. So every
  screen needs a smaller layout, not a shrunk one. The battle screen's
  70-cell map plus 30-cell side panel (ADR-0018) cannot both stay on
  screen.
- **No colour:** sides, ready/acted units, move and attack ranges, the
  danger zone, terrain, weapon types and HP are told apart by colour today
  (`docs/design/look-and-feel.md`, *Colour*). Each needs another sign.
- **Buttons:** D-pad, A, B, the crank, and the system Menu button. The
  system menu can hold up to three of the game's own entries. The game
  has 14 actions (`docs/design/controls.md`), but only the cursor,
  Confirm, Cancel and End turn are needed to play. The map menu already
  opens from Cancel with nothing to cancel, or from Confirm on an empty
  tile.
- **Crank:** rewind goes back to any earlier action in the battle
  (`docs/design/death-and-difficulty.md`).
- **Art:** the bought portraits and map sprites (ADR-0040) are colour
  pixel art. The glyph map skin (ADR-0049) is the natural Playdate look.

## Nick input

**Decision**, with the `ask-nick` skill. Nick asked for this now
(2026-10-15), so it does not wait for the Chapter 1 playtest.

Show **rendered 400 × 240 black-and-white mockups** of Chapter 1's first
battle for every visual question. Use the game's real glyph font,
scaled or redrawn per option. Offer several options and expect several
rounds (feedback: visual reviews with Nick). A mockup that uses the
bought art may be shown to Nick but never committed (ADR-0040). Glyph-only
mockups may be committed under `docs/design/playdate/`.

## Scope

**In:** these questions, asked at most three per message, as player
situations with an example:

1. **How big is the text and the map?** Today's tall cells (50 × 15
   cells, a small map window, easy to read) vs square 8 × 8 cells
   (50 × 30, more map) vs small 6 × 8 cells (66 × 30, the most room,
   hardest to read on 2.7 inches). Show the same moment of Chapter 1's
   first battle in each.
2. **How is the battle screen cut down?** For example, the map fills
   the screen and unit info pops up in a corner over the map (Fire
   Emblem on the GBA), vs a thin info strip along one edge, vs info on
   its own screen behind a button. The forecast and the combat picture
   need the same answer.
3. **How are things told apart without colour?** Enemy vs player vs ally
   units, units that have acted, move and attack ranges, the danger
   zone, HP. Options: inverted cells, dither patterns, letter case,
   blinking, underlines, or a mix. Mock up each one on the same map.
4. **Buttons and the crank.** Which actions sit on A, B, the D-pad,
   A+B together and the system menu. What the crank does: rewind
   through the battle's history, as Nick suggested, and anything else
   (scrolling the dialogue log, moving through a menu).
5. **What is left out of the Playdate version, at least at first?**
   Portraits (or 1-bit versions of them), the combat picture (or a
   simpler one), voices, music, Japanese and Chinese text, the title
   cinematic. For each, say what the player sees instead.
6. **Where is it sold?** itch.io beside the Windows build (players
   sideload it), Panic's Catalog store (curated, needs an application),
   or both.

Record the answers in `docs/design/playdate.md` and its row in
`docs/design/README.md`. Update 1025's steps if the answers change what
its build tickets must cover.

**Out (do not do):** any Playdate code or skins (1025's build tickets);
changing the desktop or web game's look or controls; the technical test
(1026); 1-bit art made from the bought art before Nick picks it in Q5
(and then check that ADR-0040's license notes allow the edit).

## Implementation steps

1. Read `look-and-feel.md` (*Colour*, *Units on the map*, *Cursor and
   selection*, *Attack forecast*), `controls.md`,
   `death-and-difficulty.md` (rewind), `chapter-1.md`, ADR-0018,
   ADR-0049 and 1025.
2. Look up real Playdate games that show a grid map or a lot of text, to
   use as examples next to the usual tactics games (Fire Emblem GBA,
   Advance Wars). Only name games you have checked exist and play the
   way you describe them.
3. Render the mockups. A script in the scratchpad that draws glyphs from
   `assets/fonts/` into a 400 × 240 1-bit PNG is enough. Show them inline
   or as an Artifact page.
4. Ask questions 1–3 first. They decide the look, and 4–6 depend less
   on them. Then ask 4–6.
5. Record the answers (ask-nick *Recording the answer*). Archive this
   ticket as in the `work-ticket` skill.

## Acceptance criteria

- [x] `docs/design/playdate.md` exists with `Decided:`, Nick's words, and
      a rule for each of questions 1–6 (or the question marked deferred,
      in Nick's words).
- [x] `docs/design/README.md` lists it.
- [x] 1025 is updated if the answers change its build tickets.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None (design decision).

## Completion notes

- Asked in three rounds with rendered 400 × 240 one-bit mockups of
  Harrowby Ford (Q1–Q3, then a second Q3 round, then Q4–Q6, then two
  follow-ups on voices and languages). Recorded in
  `docs/design/playdate.md`; every option shown is in
  `docs/design/playdate/` (glyphs only, no bought art).
- Answers: today's 8×16 cells; FE-GBA corner pop-ups; your units solid
  black, enemies grey, acted units outlined, ranges as dots / diagonal
  lines / dashed danger edge; A confirm, B cancel, A+B ends the turn,
  hold B + left/right cycles ready units, the crank winds back to rewind;
  everything kept (music, sounds, 1-bit portraits, a simple combat
  picture, the title cinematic, voices, Japanese and Chinese), but one
  build per language with one purchase covering all; itch.io first, then
  Catalog.
- Added a fourth size option (6×12, Terminus' smaller size) beside the
  three the ticket listed. The small-size mockups used unscii-8 (public
  domain) and Spleen 5×8 (BSD-2) as scratch fonts; none ships.
- Playdate facts checked against the SDK docs: three custom system-menu
  entries, the six buttons, the crank's change and docked calls.
- 1025's scope now lists the voices, 1-bit portraits, combat picture,
  title cinematic, one build per language and the itch.io-then-Catalog
  order.
- *the agent's starting rules* (Nick can veto): a white ground; corner
  pop-ups sit at the top and move down when the cursor comes near; the
  side panel's other contents live in Unit info and the map menu; the
  cursor's corner marks flip the pixels under them; winding the crank
  forwards steps back towards the present while choosing a rewind point;
  the crank does nothing else on the map; the system menu holds Danger
  zone, Dialogue log and Options; allies and neutrals wait for a later
  chapter.
