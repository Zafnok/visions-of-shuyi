---
id: "0858"
title: "Chapter 2 art: Joss's face, map sprite and combat picture"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: done
blocked_by: ["0848"]
nick_input: setup
completed: 2026-11-23
---

# 0858 — Chapter 2 art: Joss's face, map sprite and combat picture

*Split on 2026-11-23: Hendry's half is 0881 (Nick wants a second look
at his combat picture first). The text below is the plan as written for
both.*

## Context

0848 built Chapter 2's battle, units and scenes, but not their art: it
split the art off because half of it waits on Nick and the other half
on a download, and both go through the private repository in one push
(`CLAUDE.md`, *Bought art*; ADR-0040). Until this lands, Joss
(`defector`) and Silas Hendry (`hound_lieutenant`) speak with empty
portrait frames and are drawn by their class's map sprite and combat
picture, as the Chapter 1 cast is in a build without the private
assets.

The 2026-11-01 audit of
[PR 481](https://github.com/Zafnok/visions-of-shuyi/pull/481) confirmed
this is still deferred acceptance work. Keep it on the Act 1 gate (1064),
and distinguish a private-assets character image from a successful class
fallback: a populated frame alone does not prove the chosen art loaded.

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
  map sprite (`crates/tools/src/map_sprite_import.rs`'s table, then
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

- [x] With the private assets, Joss has a face in every Chapter 2 scene,
      and his own map sprite and combat picture. (Hendry: 0881.)
- [x] The private-assets tests resolve the character-specific portrait,
      map and battler ids for Joss; substituting a class default or
      omitting a portrait fails. Ordinary gates remain independent of the
      bought files. (Hendry joins the same test in 0881.)
- [x] Completion notes link screenshots of the running refusal scene and
      a fight with Joss from the private build, with neither raw bought
      files nor extracted sprite sheets committed or attached. (Hendry's
      fight: 0881.)
- [x] `cargo shuyi private-assets --pin` passes (every new file has a
      credit), at the private pull request's merge on `main`.
- [x] All gates in the `run-gates` skill pass, the private-assets tests
      included.

## Tests required

- The private-assets tests (`crates/content/tests/it/private_assets.rs`)
  load the new portraits, battlers and tileset entries.

## Completion notes

**Split.** Shown the picks side by side (a scratch mockup, not
committed), Nick said "import joss but hendry I wanna double check on
available options for his battle sprite bc its ugly". Joss's art landed
here; Hendry's (a new candidate round, then the pack, his generator face,
map sprite and combat picture) is **0881**, now in 1064's and 1069's
`blocked_by` and named in 0184 and 0878.

**What was done (Joss, `defector`):**

- Face: the Tiny Tales **Thief** hero's eight busts, cut by `cargo xtask
  portrait-import` into `assets-private/game/portraits/defector/`; the
  sidecar maps the dialogue expressions as the other heroes' do
  (`happy` → smile, `angry` → stern, `surprised` → surprise).
- Map sprite: the Thief's sheet, `units/thief.png`, a character row in
  `map_sprite_import.rs`'s `SPRITES`; `tileset-import` named it.
- Combat picture: Shironejiya's `m_s_hero07` at 1.25×, recoloured on
  import by two `Tint`s (`HOUND_BLACK` in `battler_import.rs`): the blond
  hair brown, and the red headband, cape lining and trim black. The
  skin and the orange leather and shield rim stay. The site served the
  file to a plain download, so Nick didn't need to save it; it is in
  the private library beside the other Shironejiya pictures.
- Credits: Tiny Tales' `private` list gains `portraits/defector` and
  `units/thief.`, Shironejiya's `battlers/defector.`; both
  `THIRD_PARTY_ASSETS.md` rows updated.
- **The Chapter 1 demo leaves Joss out** (the first art of a later
  character; the demo's export refuses a later id, `defector` among its
  canaries). Its list, `crates/content/demo/bundle.txt`, swaps his two
  paths out of its copy of `credits.ron`, and a new `art` line form names
  bought-art files: his portrait, battler and map sheet are left out of
  the demo's art table and his tileset entry is cut, checked only by a
  build that has the art (`demo_bundle.rs`; test
  `art_lines_are_checked_only_where_the_art_is`). `THIRD_PARTY_ASSETS.md`,
  which the demo ships, says "a Chapter 2 character" instead of his id.
  0881 adds Hendry's files the same way. The map-sprite table's test now
  accepts any character of the game, not only the lead's pictures.
- Test: `named_characters_resolve_their_own_face_map_sprite_and_battler`
  (`private_assets.rs`), a table of (character, class, map sheet) that
  0881 extends with Hendry.
- Private repository: `Zafnok/visions-of-shuyi-assets` pull request
  `t0858-joss-art`; the pin moves to its merge on `main`.

**Screenshots** (the running game, private build):
[the refusal scene](../../docs/screenshots/0858-joss-refusal.png) (Joss's
face; Hendry's frame still empty until 0881) and
[Joss in a fight](../../docs/screenshots/0858-joss-fight.png) (a throwaway
staged battle, the stage only; the Swordsman class has no bought picture
yet, 0878). Joss isn't on Chapter 2's map (he joins after it), so his map
sprite was checked in a staged battle and first shows in Chapter 3.

**For the Act 1 playtest (1064):**

- The recolour also turned his olive trousers dark brown (same hues as
  the hair, so one tint takes both). Nick's "Hound black" was about the
  cloth; the brown trousers sit well with it, but say if they should be
  olive.
- The face (the Thief bust) has a dark headband and brown vest; the combat
  picture is armoured with a shield. That gap was accepted in 0070
  ("mostly like the thief hero", then "F Hound black").

**Deviations:** `cargo shuyi private-assets --library` and the importers
ran from the built tool program while the session's `shuyi` MCP server
held it; no other change.

**Engine friction:** estimate 40 production lines (two table rows, one
tint); measured about 75, with the demo list's `art` lines (1.9×, under
2×).
