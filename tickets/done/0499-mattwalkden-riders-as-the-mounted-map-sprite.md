---
id: "0499"
title: "MattWalkden's riders as the mounted units' map sprite"
type: feature
milestone: M3 Battle UI
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: done
blocked_by: []
blocks: ["1064"]
nick_input: setup
completed: 2026-11-17
---

# 0499 — MattWalkden's riders as the mounted units' map sprite

## Context

In the picture look, every mounted unit (Tamsin, the Hound riders) is drawn
on the map as the Tiny Tales knight **on foot** (`knight_m1`,
`crates/xtask/src/map_sprite_import.rs` ~:105–113: "a knight on foot until
ticket 0040 finds mounted art"). Ticket 0040 chose MattWalkden's free
*Fantasy Battle Pack* riders for the map (`look-and-feel.md`, *Who uses
what*; `tickets/done/0040-…md` ~:189), but nobody imported them.

Ticket 0083 (2026-11-17) made this matter. A rider can now get off the
horse (`terrain.md`, *Getting off and back on a horse*), and Nick picked
MattWalkden's riders for a mounted unit and today's knight for a rider on
foot ("Yes, MattWalkden riders"). So a mounted unit's map sprite becomes
a MattWalkden rider, and the knight stays for a rider on foot. Nick:
"dont need to block mechanic on it", so 0334 doesn't wait for this
ticket and this ticket doesn't wait for 0334.

The pack: <https://mattwalkden.itch.io/fantasy-battle-pack>, pay what you
want, "Royalty free commercial and personal license", "No generative AI
was used". Its store samples show lance, sword and bow riders with idle,
move, attack and death animations in 4 directions.

## Nick input

**Setup** (before this ticket starts): download the pack from the page
above ("Download Now", name your own price, $0 is accepted) and put the
zip in `D:\tactical-rpg\assets-private\library\mattwalkden\`. The agent
doesn't download it: it's a purchase flow on itch.io.

Nothing else is asked. It's verified by `cargo shuyi frame-png` pictures
(the picture look, cropped to a few tiles, never a whole real map:
ADR-0118) and the sprite skin's tests. What's left to judge by playing
goes on the Act 1 playtest's checklist (1064).

## Scope

**In:**
- Read the licence file in the download before using it (ADR-0013; it
  must allow commercial use and edits) and add the pack to
  `THIRD_PARTY_ASSETS.md` and its credit to `assets/data/credits.ron`,
  with its `private` paths (ADR-0051).
- Cut the rider that fits the Rider class (the lance rider) into this
  game's map-sprite sheet format (`look-and-feel.md` ~:868: 16×20
  frames, three walking frames per direction), recoloured like the
  other units, through `map_sprite_import.rs` (a `For::Class("rider")`
  entry, and the other mounted classes if the pack has their weapon).
  The files go to the private repository's `game/units/`
  (`cargo shuyi private-assets --pin`; AGENTS.md, *Bought art*).
- Keep `knight_m1` as the picture of a rider **on foot**. If 0334 is
  done, the sprite skin picks it from the unit's on-foot state in the
  map scene. If 0334 isn't done, leave a name for it in the tileset file
  (e.g. an on-foot picture per class) and add a line to 0334 saying the
  skin must pick it.
- Update `look-and-feel.md`'s *Who uses what* row (drop "not imported
  yet").

**Out (do not do):**
- The combat picture on foot and the glyph look's mark (0089).
- The dismount rules, menu and scene field (0334).
- Any other MattWalkden sprite (foot units keep Tiny Tales).

## Implementation steps

1. Check the zip is there; read its licence; record the credit.
2. Look at the pack's rider sheets; pick the frames that match how the
   other units walk (`look-and-feel.md` ~:415, ~:438–446) and map them
   to the game's sheet layout in `map_sprite_import.rs`.
3. Run the import (`cargo xtask` map-sprite import, as `knight_m1` was
   made), check the result in `assets-private/game/units/`, pin it.
4. `frame-png` the picture look with a Hound rider and Tamsin on the map.

## Acceptance criteria

- [x] Mounted units are drawn on the map as a MattWalkden rider in the
      picture look (`frame-png` picture in the PR, cropped).
- [x] The pack is in `THIRD_PARTY_ASSETS.md` and `credits.ron`, and its
      licence allows commercial use.
- [x] A rider on foot still has the knight (via 0334's state, or a named
      on-foot picture 0334 picks up).
- [x] All gates in the `run-gates` skill pass.

## Tests required

- Snapshot / integration: the sprite skin picks the mounted picture for a
  mounted class (and the on-foot one for a rider on foot, if 0334 is
  done), read from the scene and the tileset, not from pixels.

## Completion notes

- **The pack** (Nick put the zip in the session; *Fantasy Battle Pack
  06-07-25*): its licence file allows use "as a component part of an
  artistic or commercial work" and edits, and forbids handing the files
  out on their own, like our other bought art (ADR-0040). It is in the
  private library, `library/mattwalkden/`, with `terms.txt` (the licence
  in text) and **`INDEX.md`, a catalog of the whole pack**: lance, sword
  and bow riders; lance and axe knights on foot; spear, sword and axe
  fighters, an archer, a thief and a wizard (each in blue, green and
  red, most in three skin tones, with idle, move, attack and death in
  three directions); a 16×16 tileset; cursor, path arrows and range
  tiles; a critical-hit flash. The zip is in `originals/mattwalkden/`.
  `THIRD_PARTY_ASSETS.md` and `credits.ron` have it.
- **What the map shows:** every mounted class is a MattWalkden rider: the
  **red lance rider** for the Rider, Lancer, High Lancer, Iron Rider and
  Juggernaut, the **red bow rider** for the Outrider and Windrunner (the
  pack has no axe rider). `cargo xtask map-sprite-import` cuts them into
  the game's walking sheet: the pack's first three move frames per
  facing; left is the pack's right-facing row mirrored (it has no left);
  its half-transparent drop shadow is dropped, since the skin draws its
  own outline round the shape.
- **Deviation: 24×24 frames, not 16×20.** The riders are about 20 px wide
  with the horse, so a 16×20 cut would lose the lance and the tail, and
  scaling pixel art down ruins it. The tileset format gained
  `frame_px` on a picture (its frames' size, walking sheet included); the
  skin already drew a picture at its own size, centred on its tile. A
  rider is 4 px wider on each side than its tile and about as tall above
  it as a Tiny Tales sprite.
- **On foot:** 0334 isn't done, so the tileset names the Tiny Tales knight
  (`knight_m1`) for each mounted class in a new `units.on_foot` table
  (`Tileset::on_foot_picture`), checked like any picture but not drawn
  yet; 0334's map bullet now says the skin must pick it.
- **Private repository:** the art is on its branch
  `t0499-mattwalkden-riders` (Zafnok/visions-of-shuyi-assets#11), and the
  pin points at that branch's commit for CI. That pull request merges
  first; then `cargo shuyi private-assets --pin` moves the pin to its
  merge on `main` before this one merges (`content-change`).

### For the Act 1 playtest (1064)

- **Starting rules (the agent's), for the Act 1 playtest (1064):**
  - **Red for every rider, ours and theirs**, as bought: the Red
    Company's riders are red in the combat scene (0446) and Tamsin's
    colour is red; the outline still says the side. No pixel recolour.
  - **The light-skinned, short-haired variant** for the class picture.
  - **The move frames for stepping on the spot too** (like every unit):
    a rider that can still act trots on the spot; one that has acted
    stands on the middle move frame.
  - **The Iron Rider and Juggernaut** (plate armour) are the lance rider
    too; nothing in the pack is an armoured horseman.
  - **Every mounted class on foot is the knight**, the bow riders too.
