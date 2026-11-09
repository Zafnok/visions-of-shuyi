---
id: "0866"
title: "Chapter 5 art: Hedda's face and combat picture, Ragna's face"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0851"]
nick_input: setup
completed:
---

# 0866 — Chapter 5 art: Hedda's face and combat picture, Ragna's face

## Context

0851 built Chapter 5's story battle at the Envoy's House and made Hedda
Ravn (`shieldbearer`) a unit, but not her art or Ragna's, for the
reasons 0848 to 0850 split theirs off into 0858, 0861 and 0862: both
faces wait on Nick (he builds generator faces himself, 0070), and all of
it goes through the private repository in one push (`CLAUDE.md`,
*Bought art*; ADR-0040). Until this lands, Hedda and Ragna speak with an
empty portrait frame, and Hedda is drawn by the Raider's map sprite and
combat picture.

The picks (`docs/design/look-and-feel.md`, *Faces and combat pictures
for the Act 1 cast*; the sheets in `docs/story/characters/`):

- **Hedda**: a generator face to match her picture; combat picture elie
  automata's **Battle Axemaiden**
  (<https://elie-automata.itch.io/battle-axemaiden-animated-character-story-art>,
  free, AI-assisted, so the Steam page discloses it, 0903; the licence
  asks for the credit "elie automata"), in her three-quarter pose (0076).
- **Ragna**: a generator face, no combat picture (she never fights).

0858, 0861 and 0862 go through the same tools; one session may do
several of them in one push of the private repository. The Hound
officer and the magistrate (no names of their own, like the herald)
have no picks; they are out of scope.

## Nick input

**Setup** (before the session starts):

1. **Download** the Battle Axemaiden (free, the link above) and put it in
   the private repository's `library/` (any folder name; say which).
2. **Build the two faces** in the Character Generator EX from these
   parts lists, and save the eight expressions as the other generator
   faces were saved (`assets-private/game/portraits/keeper/` shows the
   set). Their scenes use all five of `neutral`, `happy`, `angry`, `sad`
   and `surprised`.
   - **Hedda** (32, `shieldbearer.md`): a big, broad adult woman; long
     red hair in a ponytail; weathered, a little sunburnt; no hat; dark
     armour or the darkest plain outfit, with a red cloak or collar if
     one fits (to match the Axemaiden).
   - **Ragna** (about 38, `envoy.md`): an adult woman; dark-blonde hair
     in a bun; tired eyes; a plain traveller's coat; no hat, no jewellery.

Verification after that is by tests and `cargo shuyi frame-png`
pictures (built with `--features private-assets`, screenshots only);
no check after merge (the Act 1 playtest, 1064, covers the look).

## Scope

**In:**
- The two generator faces (`cargo xtask portrait-import --generator`).
- Hedda's combat picture (`cargo xtask battler-import`), and her map
  picture if the pack has one that fits (`map-sprite-import`).
- Credits: each file's path in its credit's `private` list in
  `assets/data/credits.ron`, the pack and artist (elie automata) added;
  `THIRD_PARTY_ASSETS.md` rows.
- Push the private repository, `cargo shuyi private-assets --pin`,
  commit `assets-private.rev`.

**Out (do not do):**
- Drawing any face or picture (recolours and small edits only, the
  `ascii-art` skill).
- The officer's and the magistrate's faces; anyone else's art; any
  change to the battle, units or script (0851's).

## Implementation steps

1. Read `look-and-feel.md` (*Faces and combat pictures for the Act 1
   cast*), the `ascii-art` skill, 0858, 0861 and 0862, and how 0706/0413
   imported Chapter 1's art (`tickets/done/`).
2. `cargo shuyi private-assets --library`.
3. The two faces; Hedda's combat picture.
4. Credits and `THIRD_PARTY_ASSETS.md`; push; pin.
5. Render a fight with Hedda at the Envoy's House and the attack scene
   (`ch05_prebattle`) with `cargo shuyi frame-png` built with
   `--features private-assets`, look at them, and fix what looks wrong.

## Acceptance criteria

- [ ] With the private assets, Hedda and Ragna have a face in every
      scene they speak in, and Hedda her own combat picture.
- [ ] The private-assets tests resolve their character-specific
      portrait and battler ids; substituting a class default or omitting
      a portrait fails. Ordinary gates stay independent of the bought
      files.
- [ ] `cargo shuyi private-assets --pin` passes (every new file has a
      credit).
- [ ] All gates in the `run-gates` skill pass, the private-assets tests
      included.

## Tests required

- The private-assets tests (`crates/content/tests/it/private_assets.rs`)
  load the new portraits and battler.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
