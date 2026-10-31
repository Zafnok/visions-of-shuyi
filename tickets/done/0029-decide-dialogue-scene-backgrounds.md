---
id: "0029"
title: "Decide: backgrounds behind dialogue scenes"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0021"]
nick_input: decision
completed: 2026-10-15
---

# 0029 — Decide: backgrounds behind dialogue scenes

## Context

The dialogue screen (0704) draws two portraits, name plates and a text box on
a black background. Fire Emblem games draw a background picture behind story
conversations (a castle hall, a forest, a village). Nothing in
`docs/design/look-and-feel.md` or the backlog says whether ours should. Found
while listing Chapter 1 gaps (2026-09-29).

Constraints:

- Portraits are bought 48×48 Tiny Tales faces by Mega Tiles (0021, 0711). Nick disliked agent-drawn
  portraits, so agent-drawn backgrounds may clash in the same way; say so
  honestly and show them next to the bought portraits.
- Any bought art follows ADR-0032 and the private assets repo (0110).
- **Already bought (2026-10-02):** the Mega Tiles bundle includes
  *Battlebacks Vol.1*: 24 backgrounds, 336×248 pixels, eight outdoor places
  (desert, oasis, two fields, meadows, two mountains, wasteland) each at
  day, dusk and night, and the Tower tileset's three sky pictures (384×512).
  There is nothing indoors and no village or town picture. The tilesets
  (Overworld, Dungeons 1 and 2, Tower) can also be laid out as a room or a
  yard and used as a backdrop. Show both under option C before looking for
  more packs.
- Glyph-drawn art is the game's own style (ADR-0018).

## Nick input

**Decision.** With the `ask-nick` skill, show real mockups of the dialogue
screen with a bought portrait on each option:

- **A. Keep black** (today).
- **B. Glyph-drawn backgrounds** (the `ascii-art` skill), dimmed so the
  portraits stand out.
- **C. Bought background art** matching the portrait style (search for packs
  under allowed licences; name the price and licence). Look first at the
  artist 0021 picked (Mega Tiles, which sells tile and terrain packs), then
  others such as CaptainSkolot, who sells background packs; say honestly
  if the styles clash.
- **D. No picture, just a place line** (for example "The old fort, at dusk") above
  the text box.
- **E.** "Describe your own."

Sub-question: if backgrounds, how many does Chapter 1 need (one per
location in `docs/story/chapters/ch01.md`)?

## Scope

**In:**
- The mockups and the decision, recorded in `docs/design/look-and-feel.md`
  with Nick's words.
- If Nick picks B, C or D: create the implementation ticket (a `@background`
  or `@place` directive for `.dlg` scripts, like 0710's `@music`, plus the
  art or text), with the `write-ticket` skill, and say whether it blocks the
  playtest (0804).

**Out (do not do):**
- Implementing it.
- Buying anything (Nick's call and money).

## Implementation steps

1. Build mockups with real portraits (0711 format).
2. Run the decision; record it.
3. Create the follow-up ticket if needed.

## Acceptance criteria

- [x] `look-and-feel.md` records the choice and Nick's words.
- [x] A follow-up ticket exists if the answer isn't A.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- None (design decision; docs only).

## Nick's idea (playtest round 2, 2026-10-14)

Nick, after playing Chapter 1 (0804):

> "probably the conversations should have some kind of background imagery,
> similar to a cutscene... like even just Dace pacing a castle or
> whatever... and then flip to the conversation portraits with the
> background opacity lessened or otherwise darkened... it would set the
> mood better than portraits against black background"

So this is now his option, and it leads the question: a short scene first
(units or a character moving in a place, like the title cinematic's map
shots), then the portraits over that place, darkened. Mock it up next to
A–D (bought backdrops, a tileset room or yard, the battle map itself) and
ask how many places Chapter 1 needs and whether every conversation opens
with the moving part or only the first in a place.

**The playtest waits for this** (Nick, 2026-10-14: all round 2 feedback is
done before he replays Chapter 1). The follow-up implementation ticket(s)
go in 0804's `blocked_by` and in `docs/ROADMAP.md`'s round 3 list.

**Scene music:** asked about the four scene music picks, Nick said "I want
to see how the cutscenes would play out with this ambience". The picks
(`audio.md`, *Chapter 1's cues*) stay unapproved until he sees the scenes
with their backgrounds; the follow-up's sign-off asks about them.

## Completion notes

**Decided 2026-10-15** (`docs/design/look-and-feel.md`, *Places behind
conversations*, with Nick's words).

- **Mockups:** real frames of the game (`cargo xtask frame-png`, with the
  bought art) of `ch01_intro` (the lead and Hollis) and Tamsin's news,
  with places composited behind at 0.42 brightness: today's black; the
  battle map in glyphs; the *Battlebacks* (fields at dusk, meadows); the
  tilesets' sample maps (a cabin yard, a town); the battle map; and two
  animated openings (the lead and Hollis sparring in the yard, then the
  talk over it darkened; Dace pacing a stone hall). They show bought art,
  so they are not committed (CLAUDE.md, *Bought art*).
- **Found:** option D (a place line) already existed: every Chapter 1
  scene opens with an `@caption` line. It stays.
- **Nick's answers:** E (a moving opening, then the talk over the place,
  darkened), C (a still place) where there's no opening; glyphs when the
  player picks the glyph look and on the Playdate; the opening should be
  about half the mockup's length; the place is the fitting tileset place
  away from a battle and the battle map before, during and after one; an
  opening only where the script asks; in battle, the battle map darkened.
- **Chapter 1 needs** two new places (the yard at Harrowby, the study at
  Veyne Hall) plus the battle map; openings in `ch01_intro` (sparring)
  and `ch01_tbc` (Dace pacing).
- **Follow-up tickets** (all in 0804's `blocked_by` and the roadmap's
  round 3 list; the playtest waits for them): **0732** the `@place`
  directive and the darkened place, the battle map behind battle scenes;
  **0733** Chapter 1's yard and study; **0734** the moving openings, whose
  sign-off asks again about the four scene music picks (`audio.md`).
  1043 (the Playdate's dialogue screen) now says how the place shows
  there.
- **the agent's starting rules** (in the design doc, Nick can veto): the
  place at about 40% brightness; an opening of about 1.5 s of movement
  and 0.5 s of darkening, Confirm skips it; actors are their map sprites
  (glyphs in the glyph look) at the map's walking speed; intro and
  victory scenes use the battle map only when the script names it (the
  letter at dawn gets its own place); a place is lit by its time of day
  where the pictures allow; a scene with no place stays black.
- **Gates:** docs and tickets only; `cargo xtask ticket-lint`, `typos`
  and the `xtask` tests were run (no code changed).
