---
id: "0413"
title: Combat scene with bought full-body art of both combatants
type: feature
milestone: M3 Battle UI
model: opus-5.5
effort: high
status: done
blocked_by: ["0404", "0021", "0035", "0110", "0711", "0231"]
nick_input: decision
completed: 2026-10-11
---

# 0413 — Combat scene with bought full-body art

## Context

In ticket 0011 Nick said the combat screen "probably" should show "a full body
rendering of the 2 battling units", rather than portraits
(`docs/design/look-and-feel.md`). 0404 builds combat playback as a text box
with names and HP bars. This ticket adds a scene with the two fighters drawn
full-body while the strikes play out.

**Changed 2026-09-30:** the art is **bought**, not drawn by Claude. Nick
doesn't want Claude-drawn character art (0021). His words: "I guess we need
an itch artist who has a pack with portraits and battle sprites". 0110 keeps
the bought files in the private assets repo, and 0711 draws bought PNG art
as sprite items (0231, ADR-0038; changed 2026-10-01 from per-pixel
overlays). This ticket reuses that drawing for the fighters: each fighter
is one sprite item, moved by changing its `dest`.

**Decided in 0021 (2026-09-30), `look-and-feel.md` § Combat screen:** the
art is Mega Tiles' Tiny Tales **big still battle images** (one picture per
hero or class, shipped at 1×, 2× and 3×; use the 1× file), **moved by the
game**: lunge to strike, flash on a hit, shake, fade when defeated (like the
enemies in Final Fantasy VI or Dragon Quest). The packs' small animated
RPG Maker battle sprites are not used. Nick: "1A seems best". Classes with
no hero art use a still image as a stand-in in Chapter 1: church cleric
(Cleric) and church knight (Guard) from *Vol.5 Faith and Evil*, the orc axe
fighter (Brigand) from *Vol.1 Monstrous Uprising*, and an on-foot lance
fighter for the Rider (nothing mounted exists in the catalogue).
Named characters without their own hero art use their class's picture
recoloured to their colours (Nick: "sure"); a character's face and combat
picture must match (Rue uses the Witch for both). The boss **Harl**'s
picture is decided in ticket 0035 (Nick shops around), which this ticket
waits for since 2026-10-01 (before that, nothing would have put his picture
in once it was chosen). Use 0035's pick; if Nick chose to keep a stand-in,
use the stand-in he named. The picture's weapon must match the unit's
weapon type (weapon types have their own rules).

Nick wants this scene in the Chapter 1 playtest (2026-10-01), so 0804 waits
for this ticket.

**Changed 2026-10-02 (ticket 0038).** Nick bought the whole Mega Tiles
bundle and saw two mockups of this scene made from the real files, with a
bought battle background behind the fighters: G1a, in today's 44-cell box
over the map, art at its own size (a 336×248 background fits the box
almost exactly), and G1b, filling the screen with the art at double size.
His words: "for G1a and G1b I think both are a little off. The main thing
that's off though is that you put the enemy facing backwards... let's
always mirror the image to face the player when battling. Other than
that, I worry G1b won't be able to convey all the info, but maybe it can
work with some additional in-battle overlays or pop up messages".

- **Decided: the two fighters always face each other**; a picture facing
  the wrong way is mirrored (`look-and-feel.md`, *Combat screen*). Every
  still battle picture looked at on the bought files (the heroes and the
  Vol.1, 4 and 5 classes) **faces right as bought**, so the fighter on the
  right is the one to mirror. Check each picture when importing it; the
  sidecar's "which way it faces" field (step 2) is what the game reads.
- **Still open, ask in this ticket:** box or full screen, and if full
  screen, what overlays or pop-up messages carry the information the box
  shows today (names, HP, hit, damage, crit, the strike text); whether a
  bought background is drawn behind the fighters at all. Show both layouts
  again with the facing fixed.
- **What the bundle has for backgrounds:** *Battlebacks Vol.1*, 24
  pictures of 336×248: desert, oasis, two fields, meadows, two mountains
  and wasteland, each at day, dusk and night. Nothing indoors, no forest
  interior, no town, no bridge or water. If backgrounds are in, which one
  a fight uses follows the terrain the defender stands on and the map's
  light (ticket 0438: noon → day, dusk → dusk, night → night); say which
  terrains have no fitting picture and what they fall back to.
- **Sizes found:** hero pictures are 64–128 wide and 72–80 tall; class
  pictures 48–128 by 48–96; the *Gods and Gallants* and *Epic Monsters*
  pictures are much bigger (up to 221×184), so a boss from those towers
  over a hero at the same scale.
- The still battle pictures are sorted in the private assets repository
  (ADR-0040; `cargo xtask private-assets --library`), in
  `assets-private/library/tiny-tales/characters/` (`heroes/<Name>/
  battler.png`, `battler-classes/<pack>/<Name>/battler.png`).

**Changed 2026-10-04 (ticket 0040).** Nick chose art for the fighters
Tiny Tales has no picture for; the table is in `look-and-feel.md`,
*Fighters with no Tiny Tales art*. It replaces the stand-ins named above
and settles Harl (0035 is answered there):

- **Brigand, Raider and Harl** use Shironejiya's human bandits (free;
  Harl is the bearded axe bandit) instead of the orc. **Mounted classes**
  use Pixel Flag's Paladin (¥500, **not bought yet**: ask Nick to buy it
  when this ticket needs the file) instead of an on-foot lance fighter.
  Crane, the Brawler and Dace use Shironejiya pictures too; Aske is the
  Archer hero and Maud (the keeper, a Cleric) the Church Cleric.
- **Sizes are no longer whole numbers only.** Nick picked 1.25× for the
  Shironejiya pictures and 1.5× for the Paladin so they stand as tall as a
  Tiny Tales hero, and noticed that Tiny Tales' own class pictures are
  smaller than its heroes. So each picture carries its own size next to
  its facing in the sidecar (step 2), and "how big the fighters are" below
  is asked per source, not as one scale.
- **These files are not Tiny Tales':** bring them in with their own
  credit and licence text (0829 refuses bought files with no credit).
  Shironejiya's terms need its site name, address and a no-reuse line in
  a text shipped with the game. Its pictures are 64×64 PNGs facing left.
  Crane's spectacles are a 21-pixel edit to make at import (the pixels are
  in 0040's Completion notes).

## Nick input

**Decision** (use `ask-nick`, with rendered mockups made from the bought
sprites). Ask only what 0021 and 0040 didn't settle and the packs allow:

- where the scene is shown: the box over the map or the full screen, with
  the facing fixed, and what tells the player the numbers in the
  full-screen version (see *Changed 2026-10-02* above);
- whether a bought battle background is drawn behind the fighters;
- how big the fighters are on screen (the whole-number scales that fit the
  images' real size in the battle screen);
- the motions: how far the lunge goes, flash colour, shake, the miss and
  crit looks (show them as a short GIF or a strip of frames);
- per-class art vs per-character where both exist (the heroes have their
  own still image; generic classes share one);
- how the recolours look for named characters (show two or three);
- on/off setting (0805 already has `combat_animations`).

Record the answers in `look-and-feel.md` before implementing.

## Scope

**In:** the design question above; then a `CombatScene` drawn in the
playback overlay: both fighters' bought still images facing each other,
moved by code, HP bars and numbers under each, strike/miss/crit text, per
the answers. Images for the Chapter 1 classes (lord, Rider, Archer, Cleric,
Guard, Mage, Brigand, Raider) mapped from the bought packs.

**Out (do not do):** the packs' small animated battle sprites; drawing
sprites (only the recolours and small pixel edits 0021 allows); images for
classes after Chapter 1 (content tickets per chapter); sound; changes to
combat rules; buying anything.

## Implementation steps

1. Mock up 2–3 options for Nick with the real bought sprites (from
   `assets-private/library/`, ADR-0040), rendered as in 0011, and record
   his choice.
2. A battler format next to 0711's PNG portraits: a sidecar per class (and
   per hero that has its own image), `assets/battlers/<id>.ron`, naming the
   one PNG and which way it faces (so the game knows when to mirror it).
   The motions are code, timed by constants in `ui`, not per-image data.
   Validate like 0711 (all errors at once, with file names). Write an ADR
   (`write-adr`).
3. `cargo xtask battler-import`, or extend 0711's `portrait-import`, to copy
   a pack's 1× still image into `assets-private/game/battlers/` and write the
   sidecar stub.
4. Public placeholders: a small, plain PNG per class that ships in `assets/`
   so a clone without `assets-private/` builds, tests and runs (ADR-0040).
   Not meant to look good; Nick only sees the bought art.
5. Draw the scene in 0404's playback overlay with sprite items, as 0711
   does (mirror one fighter with `flip_x` so they face each other). If a
   motion Nick picks needs something a sprite item can't do yet (a flash
   to white, a tint), add it to `Sprite` here (the item, `app`'s renderer,
   the snapshot line and, if done, 0232's PNG renderer) and note it in
   ADR-0038's section 2; never fall back to per-pixel rectangles. Honour the
   `combat_animations` setting (off = 0404's plain box). The setting comes
   from the Options menu (0805), which this ticket doesn't wait for: if
   0805 isn't done, keep the value in `Ctx` (as `cursor_style` and
   `text_speed` are today, default on) and add a line to 0805 to move it
   into `Settings`.
6. Name each pack in the bundle's row in `THIRD_PARTY_ASSETS.md`, add
   the new files' folder to the `private` list of the bundle's credit in
   `assets/data/credits.ron` and a new artist to its `author` (ADR-0051:
   `cargo xtask private-assets --pin` refuses bought files without a
   credit), and note the class → image mapping in `look-and-feel.md` or the
   class data.

## Acceptance criteria

- [x] Nick's answers recorded in `look-and-feel.md`.
- [x] Playback shows both fighters; the setting turns it off.
- [x] A clone without `assets-private/` builds and passes every gate with
      the placeholders.
- [x] Snapshots of the scene mid-strike (placeholders), and a rendered PNG
      with the bought sprites sent to Nick.
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Unit: battler sidecar parsing and validation, every error with its
  message; the motion curves (position/flash per frame) are pure functions,
  tested frame by frame.
- Snapshot / integration: Harness playback with the scene on and off.

## Completion notes

**Done (2026-10-11).** While a fight plays with combat animations on, the
map and the side panel make way for a stage: the two fighters' bought
pictures at double size, facing each other in front of a bought backdrop,
moved by the game; a plate per fighter with name, HP bar and the fight's
damage, hit and crit; the result over the target's head.

**Nick's decisions** (three rounds, on mockups and then on renders of the
game; all recorded in `look-and-feel.md`, *Combat screen*):

- Where: the stage, not the box ("1B"), always with a backdrop ("1A or 1B
  not the 2 option"), and over the side panel too ("the side panel is
  somewhat useless in the fight scene").
- Motion: the striker runs up to its target ("2B").
- Brigand = the bearded axe bandit in its second colours, Raider = the
  armoured axe fighter in red ("2A but open to change as we progress").
- Hollis in Veyne blue and the Red Company's guard in red ("1A"); the Red
  Company's archers recoloured so they aren't Aske ("2A"); the Church
  Cleric and the Church Wizard at 1.25× ("3A").
- The Shironejiya pictures may be downloaded; the Paladin isn't needed
  for the Chapter 1 playtest.

**Claude's starting rules** (Nick can veto):

- The player's fighter stands on the left; if neither or both are the
  player's, the attacker does.
- A fighter striking from two tiles or more away (a bow, a spell) only
  steps forward instead of running up.
- A strike that heals its target (Absorb) moves nobody.
- A hit flashes the target white and shakes it; a miss makes the target
  step back; a critical leans back first, flashes the whole stage and
  shakes twice as hard; a fall fades and sinks. (All were in the GIFs
  Nick chose from; the amounts are tunable in `stage/motion.rs` and
  `stage/glyph.rs`.)
- A line spoken inside a fight (a boss's, last words) plays over the
  stage, not over the map.
- Backdrops: a fight on a mountain or a peak is in front of the
  mountains, every other terrain in front of the fields; the terrain is
  the defender's. Day pictures only until maps have a light (0438).
- The generic Mage class is the Church Wizard; Rue keeps the Witch.
- A class's picture is what the enemy's unnamed soldiers look like (the
  Guard class is the red knight); a named character of the player's who
  shares it has their own recolour.
- The stage is the same whichever map look is on, so the side panel is
  hidden in the glyph look too.

**What was built.**

- `trpg_content::battler` (ADR-0056): `assets/battlers/<id>.ron` sidecars
  (image, facing, scale) and `assets/battlebacks/battlebacks.ron`, with
  every error reported at once. Format: `assets/battlers/README.md`.
- Public placeholders: a plain figure per Chapter 1 class, `default`, two
  backdrops (made once; not art).
- `cargo xtask battler-import [--list]`: copies the bought pictures into
  `assets-private/game/`, clears Shironejiya's flat background, recolours
  (hue tints), writes the sidecars.
- `screens/battle/stage`: the view of the playback (ADR-0054), the motion
  functions, the painter. The playback's fighters now carry character,
  class and tile; its combats the forecast numbers.
- Credits: Shironejiya's own entry, with the text its terms ask for; the
  Tiny Tales entry covers its battle pictures and backdrops.

**Deviations from the plan.**

- No new `Sprite` feature was needed (a hit is `Paint::Solid`, a fall is
  `opacity`, the stage's flash the backdrop drawn again in white), so
  ADR-0038 has no new note.
- With combat animations off, fights were already skipped whole (0805),
  not shown in a box; that is unchanged. The box still shows while a
  story line waits inside a skipped fight.
- The Rider has no bought picture: the Paladin isn't bought. It shows the
  placeholder figure (ticket 0446).
- Crane, Dace and the Brawler aren't in Chapter 1: their pictures (and
  Crane's spectacles) come with their chapters. Their Shironejiya files
  are in the private library already.
- The Red Company archer's recolour is weaker in the clothes than hoped:
  the Archer wears mostly brown, so it is the dark hair that tells her
  from Aske.
- The placeholders were drawn once by a script, not by an `xtask`
  generator.

**Follow-up tickets:** 0446 (the riders' Paladin). Lines added to 0438
(dusk and night backdrops) and 0901 (Shironejiya's text in a release).

**For Nick, when playing:** start any fight. To compare, turn *Combat
animations* off in Options. Riders are a plain grey-green figure until
the Paladin is bought.
