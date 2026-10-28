---
id: "0070"
title: "Decide: faces and combat pictures for Act 1's new cast, from the bought art"
type: design-decision
milestone: Design decisions
model: opus-5.5
effort: medium
status: todo
blocked_by: []
nick_input: decision
completed:
---

# 0070 — Decide: faces and combat pictures for Act 1's new cast

## Context

Act 1 (`docs/story/outline.md`) brings people Chapter 1 never showed,
and each needs a face for dialogue and, if they fight, a combat picture
and a map sprite:

| Who | Id (`docs/story/names.md`) | Class (outline) | First seen |
| --- | --- | --- | --- |
| Joss Pellam | `defector` | Swordsman | Chapter 2 (recruit) |
| Gil Parrow | `prizefighter` | Brawler | Chapter 4 (skirmish recruit) |
| Hedda Ravn | `shieldbearer` | Raider | Chapter 5 (joins) |
| Ragna Holt | `envoy` | NPC | Chapter 5 |
| Oriel Mast | `battlemage` | Sorcerer, tier 2 | Chapter 6 (joins) |
| Dace Marr, in person | `rival` | boss | Chapter 4, 8 (0735 picks his map picture for Chapter 1's opening) |
| Keeper Absalom Crane | `vowmaster` | NPC | Chapter 7 |
| The bosses | `hound_lieutenant`, `hound_sergeant`, `undercroft_warden` (0743) | per chapter | Chapters 2, 3, 7 (Crane's champion) |
| Prior Edwin Sallow | `greywater_prior` | NPC | Chapter 3 |

0743 wrote the sheets before this ticket ran, so their looks are briefs
from the outline, to be rewritten to fit whatever is picked here
(`docs/story/characters/<id>.md`, *Portrait brief* or *Look*):

- **Joss** (`defector`): 18, tall and gangly, unruly brown hair, a
  Hound's black coat a size too big with the badge cut off; a plain
  sword.
- **Gil** (`prizefighter`): 34, huge, broken nose, cropped dark hair and
  a neat square beard, bare forearms and wrapped fists, a fairground
  sash with six ribbons; grave rather than grinning.
- **Hedda** (`shieldbearer`): 44, tall and broad, ash-blond braids going
  gray, Brennish mail and sheepskin, round shield and axe.
- **Ragna** (`envoy`): 38, slim, severe dark-blond hair, a long dark
  northern coat and an envoy's chain; tired eyes.
- **Oriel** (`battlemage`): 41, thin and stooped, graying black hair,
  unshaven, a crown battle-mage's coat with the insignia torn off.
- **The bosses:** Silas Hendry (`hound_lieutenant`, Chapter 2): a
  fair-haired Hound officer in Brennish colors with an axe. Ned Tolley
  (`hound_sergeant`, Chapter 3): an old soldier in Hound leathers with a
  blue scarf, sword and shield. Constance Hobb (`undercroft_warden`,
  Chapter 7): a tall, heavy woman in a warden's gray mail and hood, keys
  at her belt. And one NPC who doesn't fight: Prior Edwin Sallow
  (`greywater_prior`, Chapter 3), a small old keeper in a gray habit
  (a face at most).

Nick bought Mega Tiles' whole bundle (`look-and-feel.md`, *Portraits
and battle art (bought)*): heroes with faces, busts, still battle
images and map sprites, and still battler packs **without** faces. Art
comes from that bundle only (no commissions, never drawn by Claude;
ADR-0040: the files and any picture pasted from them stay out of this
repository).

## Nick input

**Decision**, with the `ask-nick` skill (the question pop-up), on
mockups: each candidate in our dialogue box and our combat scene, at the
game's size, in the scratchpad only (never committed: ADR-0040). Three to
five candidates per character, each with the pack and sheet name, plus
"describe your own" (another sheet from the bundle). Expect several
rounds; a few characters per message.

## Questions to ask

For each person in the table:

1. **Which face** (and so which bust and, for heroes, which battle image
   and map sprite come with it)?
2. **Which combat picture**, if they fight and their face has none (still
   battler packs), and whether its weapon matches their class.
3. For the bosses: a face of their own, or a still battler only (a boss
   who never speaks outside battle may not need a face; Chapter 1's Harl
   has one).

## Scope

**In:**
- Searching the bundle (`assets-private/library/tiny-tales/`, `cargo
  shuyi private-assets --library`) for candidates that fit each
  character sheet (`docs/story/characters/`; 0743 writes the new ones; if
  0743 isn't done, use the outline's descriptions and tell 0743 the
  looks Nick picked).
- Mockups, the questions, and recording the picks in `look-and-feel.md`
  (*Portraits and battle art (bought)*: one row per character with the
  pack, sheet, and the edits allowed there) with Nick's words.
- Telling each chapter ticket (0848–0854) and 0743 what was picked: a
  line in each ticket's Context.

**Out (do not do):**
- Importing the files into the game (each chapter ticket imports its
  chapter's new faces and pictures, 0848–0854; the one-bit Playdate
  versions are 1069).
- Buying anything, commissions, drawing art.
- Dace's Chapter 1 map picture (0735).

## Implementation steps

1. Read `look-and-feel.md` (*Portraits and battle art*), ADR-0040, the
   character sheets and the outline.
2. Check out the library (`cargo shuyi private-assets --library`) and
   shortlist candidates per character.
3. Render mockups in the scratchpad (the dialogue screen with
   `cargo shuyi frame-png --features private-assets` and the combat
   scene the same way, or the store-preview method 0021 used).
4. Ask in rounds; record the answers; note the picks in 0743 and
   0848–0854.

## Acceptance criteria

- [ ] Every person in the table has a recorded face (or "no face" by
      Nick's choice) and, if they fight, a combat picture.
- [ ] Nick's words are in `look-and-feel.md`.
- [ ] No bought file or mockup made from one is committed.
- [ ] `cargo xtask ticket-lint` passes.

## Tests required

- None (decision).

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
