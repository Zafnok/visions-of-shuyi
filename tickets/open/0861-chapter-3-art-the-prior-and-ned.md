---
id: "0861"
title: "Chapter 3 art: the prior's and Ned Tolley's faces and combat picture"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0849"]
nick_input: setup
completed:
---

# 0861 — Chapter 3 art: the prior's and Ned Tolley's faces and combat picture

## Context

0849 built Chapter 3's battle at Greywater Abbey, its units and scenes,
but not their art, for the same reasons 0848 split Chapter 2's off into
0858: the faces wait on Nick (he builds generator faces himself, 0070),
the combat picture on a download from a site that answered a script
with a placeholder page, and all of it goes through the private
repository in one push (`CLAUDE.md`, *Bought art*; ADR-0040). Until this
lands, Prior Edwin Sallow (`greywater_prior`) and Sergeant Ned Tolley
(`hound_sergeant`) speak with empty portrait frames and are drawn by
their class's map sprite and combat picture (Mage, Swordsman).

0858 does the same for Chapter 2, through the same tools and the same
site: one session may do both, in one push of the private repository.

The picks (`docs/design/look-and-feel.md`, *Faces and combat pictures
for the Act 1 cast*; the sheets in `docs/story/characters/`):

- **The prior**: Shironejiya's dark-robed priest `m_s_mage_dark_m01` at
  1.25× as his combat picture (0070), from
  <http://hi79.web.fc2.com/material/monster_s/>, with that site's credit
  text (`look-and-feel.md`, *Fighters with no Tiny Tales art*). He speaks
  in the battle's scenes, so he gets a generator face (Nick: "Only if
  they speak").
- **Ned**: "a Hound soldier's picture will do" (his sheet): his class's
  combat picture and map sprite stay. He speaks, so he gets a generator
  face.

## Nick input

**Setup** (before the session starts): build the two faces in the
Character Generator EX from these parts lists, and save the eight
expressions of each as the other generator faces were saved
(`assets-private/game/portraits/keeper/` shows the set). Their sheets
use only `neutral`, `angry`, `sad` and `surprised`.

- **The prior** (63, `greywater_prior.md`): a small, round-shouldered
  old man; an elderly body (or the oldest adult one); hair white and
  thin, or bald with a white fringe; clean-shaven, or a short white
  beard; a dark priest's robe (the darkest robe outfit there is), no
  hat; a smudge of soot on a cheek if the generator has one.
- **Ned** (47, `hound_sergeant.md`): an adult man, heavy in the middle;
  short gray hair; gray stubble (the shortest facial hair, gray); a
  soldier's outfit in black leather; a faded blue scarf at the neck if
  one fits; no hat.

If the Shironejiya site doesn't open for the session, also save
`m_s_mage_dark_m01.png` from it into the private repository's
`library/shironejiya/monster_s/`.

Verification after that is by tests and `cargo shuyi frame-png`
pictures (built with `--features private-assets`, screenshots only);
no check after merge (the Act 1 playtest, 1064, covers the look).

## Scope

**In:**
- Both generator faces (`cargo xtask portrait-import --generator`).
- The prior's combat picture (`cargo xtask battler-import`, at 1.25×).
- Credits: each file's path in its credit's `private` list in
  `assets/data/credits.ron`, the pack and artist added (Shironejiya)
  if 0858 hasn't; `THIRD_PARTY_ASSETS.md` rows.
- Push the private repository, `cargo shuyi private-assets --pin`,
  commit `assets-private.rev`.

**Out (do not do):**
- Drawing any face or picture (recolours and small edits only, the
  `ascii-art` skill).
- Anyone else's art; any change to the battle, units or script (0849's).

## Implementation steps

1. Read `look-and-feel.md` (*Faces and combat pictures for the Act 1
   cast*), the `ascii-art` skill, 0858 and how 0706/0413 imported
   Chapter 1's art (`tickets/done/`).
2. `cargo shuyi private-assets --library`.
3. The two faces; the prior's combat picture.
4. Credits and `THIRD_PARTY_ASSETS.md`; push; pin.
5. Render a fight with the prior and Ned's scene with Hollis
   (`ch03_sergeant_engage_retainer`) with `cargo shuyi frame-png` built
   with `--features private-assets`, look at them, and fix what looks
   wrong.

## Acceptance criteria

- [ ] With the private assets, the prior and Ned have faces in every
      Chapter 3 scene they speak in, and the prior his own combat
      picture.
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
