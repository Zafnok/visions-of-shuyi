---
id: "0812"
title: Chapter title card before a chapter starts
type: feature
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: ["0801"]
nick_input: decision
completed: 2026-11-17
---

# 0812 — Chapter title card

## Context

0801's chapter file has a `title` ("Chapter 1: …"), and 0802's save slots
show it, but no screen ever shows it to the player when a chapter begins.
Fire Emblem games show the chapter number and name on a card before the
chapter's opening scene. Found while listing Chapter 1 gaps (2026-09-29).

It's player-facing UI, so Nick decides whether there is a card and what it
looks like; don't pick one as if it were decided.

## Nick input

**Decision** with the `ask-nick` and `ascii-art` skills. Render mockups at the
game's size in palette D:

- **A. No card:** the chapter starts with its first scene (today's 0801 flow).
- **B. Plain card:** "Chapter 1" and the chapter name, centred on black, for
  a few seconds or until Confirm.
- **C. Card with a border or small glyph ornament** in the style of the
  phase banners (0405).
- **D.** "Describe your own."

## Scope

**In:**
- If Nick picks a card: a `ChapterCardScreen` pushed by `ui::flow` before the
  chapter's `intro_scenes`, showing the chapter file's `title` (with name
  tokens from 0709 if titles use them). It closes after a set time or on
  Confirm; Cancel also closes it.
- Record the choice in `docs/design/look-and-feel.md`.

**Playdate:** if 1044 (the Playdate title screen) is done, give the
card a Playdate paint too, chosen by `ctx`'s `SkinSet` (1037); if not,
1044 does it.

**Out (do not do):**
- Transitions between screens (0813).
- Music on the card (the intro scene's `@music` decides, 0710).

## Implementation steps

1. Run the decision; record it.
2. If a card: add the screen, push it from `ui::flow`, and add a snapshot.

## Acceptance criteria

- [x] Nick's choice is recorded.
- [x] If a card: Harness test New Game → card shows the test chapter's title
      → Confirm → first intro scene; snapshot of the card.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: Harness flow and card snapshot (if a card).

## Completion notes

**Decision** (`docs/design/look-and-feel.md`, *The chapter title card*):
over four rounds of mockups Nick picked **C3** (two rules with a diamond
on black) and asked to try other fonts ("a titlecard is fine to break
from our established one font"). He then picked **Cinzel** for both the
name and the `Chapter 1` line, over Jacquard 24 and IM Fell English SC.

**Built:**

- `ChapterCardScreen` (`crates/ui/src/screens/chapter_card*`): a view
  with the two lines, split from the chapter file's `title` (in the
  player's language) at its first `: `. The PC skin draws each Cinzel
  glyph as a sprite from a baked sheet, painted in the line's colour; the
  Playdate skin uses the cell font in its inks. It closes after 3 s, or on
  Confirm or Cancel.
- `ui::flow` pushes it in `start_chapter`, before the intro scenes (or, on
  a world-map chapter, before the opening scenes): New Game, the next
  chapter, and a loaded chapter save.
- Cinzel (OFL-1.1) baked by a new `cargo xtask lettering` (fontdue) into
  `assets/title/chapter_{name,line}.{png,ron}`, read by
  `trpg_content::lettering` through `title.ron`'s new `lettering` field.
  ADR-0128 records this as a stand-in for the engine's proportional fonts
  (engine 2224, 2334). Credits, `THIRD_PARTY_ASSETS.md`, the notices and
  the licence file are updated.
- Tests: the flow test goes New Game → the card with the test chapter's
  title → Confirm → the intro scene; the card's snapshot; unit tests of
  the split, the timer, the keys, the layout, the fallback to the cell
  font, the Playdate skin, the loader and the bake tool (the committed
  sheets are what the tool makes).
- Pictures: `docs/screenshots/0812-chapter-card.png` and
  `0812-chapter-card-playdate.png` (`cargo shuyi frame-png`).

**The agent's starting rules** (Nick can veto; judged in the Act 1
playtest, 1064):

- The card stays up **3 seconds** unless Confirm or Cancel closes it.
- A world-map chapter shows it before its opening scenes; arriving at a
  battle on the world map, restarting and retrying don't show it.
- The debug Quick Battle shows no card.
- A title without `: ` shows whole as the name; a title with a character
  Cinzel's sheet lacks (beyond ASCII) is drawn in the cell font.
- On the Playdate the card is in the cell font.

**Size:** 449 production lines against an estimate of 450 (1.0×); the
bake is a stand-in for engine 2224 and 2334, logged in
`docs/engine/friction.md`. `pr-size` 1,924 (the two generated sheet
`.ron` files are 419 of it).

**Tests changed:** every test that walks into a chapter from New Game, a
victory or a load now closes the card first (the scene-skipping helpers
close it; a few asserts name it). Two world-map snapshots changed only in
the cursor's pulse colour, which the card's extra frames shift.

**Deviation:** the Cinzel layout spaces glyphs by their ink and a fixed
gap, not by the font's own advances and kerning, so the spacing is a
little wider than in the mockup.
