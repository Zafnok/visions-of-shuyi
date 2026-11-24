---
id: "0881"
title: "Chapter 2 art: Hendry's combat picture looked at again, then his face, map sprite and combat picture"
type: content
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: ["0858"]
nick_input: decision
completed:
---

# 0881 — Chapter 2 art: Hendry's combat picture looked at again, then his face, map sprite and combat picture

## Context

0858 was Chapter 2's art for Joss and Silas Hendry (`hound_lieutenant`).
Joss's landed there. Hendry's was split off on 2026-11-23: shown the pick
of 0076 (Stupid Creations' **TTMini: Bandit Brigand**,
<https://stupidcreations.itch.io/ttmini-bandit-brigand>, $2.99,
AI-assisted, in its three-quarter pose) beside Joss's art, Nick said: "hendry
I wanna double check on available options for his battle sprite bc its
ugly". The pack isn't bought yet, and his generator face isn't built.

Until this lands, Hendry speaks with an empty portrait frame in Chapter 2
and is drawn by his class's map sprite and combat picture.

Who he is (say it before asking, `look-and-feel.md`, *Lesson for these
questions*): the Hound lieutenant who leads the ford in Chapter 2, its
boss, an axe fighter; his sheet is
`assets-private/story/characters/hound_lieutenant.md` (its look follows
the Brigand today and changes with the pick).

What was already searched and judged (`docs/design/look-and-feel.md`,
*Act 1's new cast* and *Hedda and the Hound lieutenant*, tickets 0070 and
0076): Mega Tiles, Sagak Art and Shironejiya had nothing new; itch.io,
GameDev Market, CraftPix, OpenGameArt, Unity, BOOTH, DLsite and the
Japanese free-art sites were searched in any style; elie automata's pack
characters were "not convinced"; **cogabushi's red-armoured Captain**
(*Pixel art-like style Enemies Pack 300+*, $28.80) is the fallback Nick
already called "ok" as sold, not recoloured. Shironejiya's own bandits
(`library/shironejiya/monster_s/`) are the generic Brigand, Raider and
red captain, so not his.

## Nick input

**Decision** (before building): a new round of candidates for Hendry's
combat picture, with the `ask-nick` skill. Show each as the store shows it
(cut-outs of store previews left artifacts in 0076) and, where the file
allows, on the fields backdrop facing Joss's combat picture at the game's
size, with the Bandit Brigand and cogabushi's Captain among the options so
he compares against them. Include packs released since 2026-10-28 and
side or three-quarter poses that show the face. AI-assisted art only
sparingly (Nick accepted it for this character).

**Setup** (after the pick):

1. Buy the chosen pack if it costs money (Nick decides each purchase) and
   put the zip in the private repository's `originals/`.
2. Build Hendry's face in the Character Generator EX from a parts list
   matched to the chosen picture (hair, facial hair, outfit, colours; the
   generator's `body5` to `body7` have facial hair), and save its eight
   expressions as `assets-private/game/portraits/keeper/` shows. Nick builds
   generator faces himself; Claude's pasted generator mockups are
   unreliable (0070).

## Scope

**In:**
- The candidate round and its answer in `docs/design/look-and-feel.md`
  (under *Hedda and the Hound lieutenant*, with Nick's words).
- Hendry's generator face (`cargo xtask portrait-import --generator`,
  then a sidecar like `keeper.ron`).
- His map sprite (`crates/tools/src/map_sprite_import.rs`'s `SPRITES`,
  `For::Character("hound_lieutenant")`, then `cargo xtask
  map-sprite-import` and `cargo xtask tileset-import`) from the chosen
  pack if it has one, else a Tiny Tales sprite that matches it.
- His combat picture (`crates/tools/src/battler_import.rs`'s `BATTLERS`,
  then `cargo xtask battler-import`), recoloured only if Nick asks.
- Credits: the files' paths in their credit's `private` list in
  `assets/data/credits.ron`, a new credit for a new artist (the test
  `every_bought_file_has_a_credit` lists the Art group's titles);
  `THIRD_PARTY_ASSETS.md` rows (AI-assisted noted, for the Steam
  disclosure, 0903).
- The Chapter 1 demo leaves him out, as 0858 did Joss: `art` lines in
  `crates/content/demo/bundle.txt` for his portrait, battler, map sheet
  (if only he uses it) and an `art swap` for his tileset entry, a `swap`
  of his paths in `credits.ron`, and no `hound_lieutenant` in
  `THIRD_PARTY_ASSETS.md` (the demo ships it).
- `("hound_lieutenant", "<his class>", "units/<sheet>.png")` added to
  `named_characters_resolve_their_own_face_map_sprite_and_battler` in
  `crates/content/tests/it/private_assets.rs`.
- The private repository's pull request, then `cargo shuyi private-assets
  --pin` and `assets-private.rev` (the `content-change` skill, *The game's
  CI reads the new content before either merges*, for the pin while both
  are open).

**Out (do not do):**
- Drawing any face or picture (ADR-0018: recolours and small edits only).
- Joss's art (0858), the Hound sergeant's (0849), anyone else's.
- Any change to the battle, units or script.

## Implementation steps

1. Read `look-and-feel.md` (*Act 1's new cast*, *Hedda and the Hound
   lieutenant*, *Fighters with no Tiny Tales art*), the `ask-nick` and
   `ascii-art` skills, and 0858's completion notes (how Joss's art went
   in).
2. Search, render the candidates, ask Nick; record the answer.
3. After Nick's setup: `cargo shuyi private-assets --library`, then the
   face, map sprite and combat picture as in Scope.
4. Credits, `THIRD_PARTY_ASSETS.md`, the test; the private pull request;
   the pin.
5. Render `ch02_refusal` (`cargo run -p trpg-tools --features
   private-assets -- shuyi frame-png <out> --scene ch02_refusal --wait 2`)
   and a fight with Hendry (a throwaway `assets/battles/zz_*.ron`, never
   committed; crop to the combat stage, no map: ADR-0118), look at them,
   and fix what looks wrong.

## Acceptance criteria

- [ ] Nick's pick and his words are in `look-and-feel.md`.
- [ ] With the private assets, Hendry has a face in every Chapter 2 scene,
      and his own map sprite and combat picture.
- [ ] `named_characters_resolve_their_own_face_map_sprite_and_battler`
      covers him; substituting a class default or omitting a portrait
      fails. Ordinary gates remain independent of the bought files.
- [ ] Completion notes link screenshots of the running refusal scene and
      Hendry's fight from the pinned private build, with neither raw
      bought files nor extracted sprite sheets committed or attached.
- [ ] `cargo shuyi private-assets --pin` passes (every new file has a
      credit).
- [ ] All gates in the `run-gates` skill pass, the private-assets tests
      included.

## Tests required

- The private-assets tests (`crates/content/tests/it/private_assets.rs`)
  load Hendry's portrait, battler and tileset entry.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
