---
id: "0858"
title: "Chapter 2 art: Joss's and Hendry's faces, map sprites and combat pictures"
type: content
milestone: Post–Chapter 1
model: opus-5.5
effort: medium
status: todo
blocked_by: ["0848"]
nick_input: setup
completed:
---

# 0858 — Chapter 2 art: Joss's and Hendry's faces, map sprites and combat pictures

## Context

0848 built Chapter 2's battle, units and scenes, but not their art: it
split the art off because half of it waits on Nick and the other half
on a download, and both go through the private repository in one push
(`CLAUDE.md`, *Bought art*; ADR-0040). Until this lands, Joss
(`defector`) and Silas Hendry (`hound_lieutenant`) speak with empty
portrait frames and are drawn by their class's map sprite and combat
picture, as the Chapter 1 cast is in a build without the private
assets.

The picks are 0070's and 0076's (`docs/design/look-and-feel.md`,
*Faces and combat pictures for the Act 1 cast*):

- **Joss**: the Tiny Tales **Thief** hero's face, bust and map sprite
  (bought, in `assets-private/library/tiny-tales/characters/heroes/Thief/`),
  and Shironejiya's `m_s_hero07` combat picture at 1.25×, recoloured
  (brown hair, dark headband, the red cloth turned Hound black). The
  Shironejiya pictures come from
  <http://hi79.web.fc2.com/material/monster_s/> and carry that site's
  credit text (`look-and-feel.md`, *Fighters with no Tiny Tales art*).
  On 2026-10-29 that address answered with a placeholder page to a
  script (no image list); a browser may still reach it, or Nick can save
  the picture.
- **Hendry**: Stupid Creations' **TTMini: Bandit Brigand**
  (<https://stupidcreations.itch.io/ttmini-bandit-brigand>, $2.99;
  AI-assisted, so the Steam page discloses it, 0903) in his
  three-quarter pose, as his map sprite and combat picture, and a
  Character Generator face to match, since he speaks.

## Nick input

**Setup** (before the session starts):

1. Buy TTMini: Bandit Brigand on itch.io and put the zip in the private
   repository's `originals/` (as the other bought packs).
2. Build Hendry's face in the Character Generator EX from this parts
   list (Nick, 0070: he builds generator faces himself), and save its
   eight expressions as the other generator faces were saved
   (`assets-private/game/portraits/keeper/` shows the set):
   - an adult man's body with facial hair (the generator's `body5` to
     `body7` bodies have it);
   - hair: a shaggy medium or long cut (`hair_medium_*` or
     `hair_long_*`), **red**;
   - facial hair: a full beard (`facialhair_full_*`), **red**;
   - eyes: any plain pair, not bright;
   - outfit: a soldier's (`soldier_*`), leather and mail, a red cloak if
     one fits, to match the brigand picture;
   - no hat.
3. If the Shironejiya site doesn't open for the session, save
   `m_s_hero07.png` from it into the private repository's
   `library/shironejiya/monster_s/`.

Verification after that is by tests and `cargo shuyi frame-png`
pictures (built with `--features private-assets`, screenshots only);
no check after merge (the Act 1 playtest, 1064, covers the look).

## Scope

**In:**
- Joss's portrait (`cargo xtask portrait-import <Thief folder> defector`),
  map sprite (`crates/xtask/src/map_sprite_import.rs`'s table, then
  `cargo xtask map-sprite-import` and `cargo xtask tileset-import`) and
  combat picture (`cargo xtask battler-import`, recoloured as above).
- Hendry's generator face (`cargo xtask portrait-import --generator`),
  map sprite and combat picture from Bandit Brigand.
- Credits: each file's path in its credit's `private` list in
  `assets/data/credits.ron`, the packs and artists added (Stupid
  Creations, Shironejiya); `THIRD_PARTY_ASSETS.md` rows.
- Push the private repository, `cargo shuyi private-assets --pin`,
  commit `assets-private.rev`.

**Out (do not do):**
- Drawing any face or picture (ADR-0018 skill rules: recolours and
  small edits only).
- The Hound sergeant's art (0849) and anyone else's.
- Any change to the battle, units or script (0848's).

## Implementation steps

1. Read `look-and-feel.md` (*Faces and combat pictures for the Act 1
   cast*), the `ascii-art` skill and how 0706/0413 imported Chapter 1's
   art (`tickets/done/`).
2. `cargo shuyi private-assets --library`.
3. Joss: portrait, map sprite, combat picture (recolour), as in Scope.
4. Hendry: face, map sprite, combat picture.
5. Credits and `THIRD_PARTY_ASSETS.md`; push; pin.
6. Render Chapter 2's refusal scene and a fight with Hendry with
   `cargo shuyi frame-png` built with `--features private-assets`, look
   at them, and fix what looks wrong.

## Acceptance criteria

- [ ] With the private assets, Joss and Hendry have faces in every
      Chapter 2 scene, and their own map sprites and combat pictures.
- [ ] `cargo shuyi private-assets --pin` passes (every new file has a
      credit).
- [ ] All gates in the `run-gates` skill pass, the private-assets tests
      included.

## Tests required

- The private-assets tests (`crates/content/tests/it/private_assets.rs`)
  load the new portraits, battlers and tileset entries.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
