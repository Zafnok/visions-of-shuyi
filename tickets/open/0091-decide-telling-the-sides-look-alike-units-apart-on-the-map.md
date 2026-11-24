---
id: "0091"
title: "Decide: telling the two sides' look-alike units apart on the map"
type: design-decision
milestone: Design decisions
model: opus-5.5 / gpt-6.1-sol
effort: medium
status: todo
blocked_by: []
blocks: ["1064"]
nick_input: decision
completed:
---

# 0091 — Decide: telling the two sides' look-alike units apart on the map

## Context

In the picture look, a unit's map sprite comes from its class. Only the
lead has a picture of their own: the private tileset
(`assets-private/game/tilesets/tiny_tales.ron`, `units.characters`)
names only `lead_m` and `lead_f`. In Chapter 1 the player's named
units share a class with the Red Company's soldiers
(`assets-private/content/data/characters.ron`, the `red_*` templates;
`content/battles/ch01.ron`):

| Ours | Theirs | Same map picture |
| ---- | ------ | ---------------- |
| Tamsin (`sergeant`, Rider) | the Red Company's rider | MattWalkden's red lance rider (`units/lance_cavalier.png`) |
| Hollis (`retainer`, Guard) | the Red Company's guard | Church Knight (`units/church_knight.png`) |
| Aske (`poacher`, Archer) | the Red Company's archers | Tiny Tales Archer (`units/archer.png`) |
| Harl (`red_captain`, Brigand, the boss) | the Red Company's brigands | Warrior M1 (`units/warrior_m1.png`) |

Only two things separate them. The 1-pixel outline is the side's
colour: player a deeper blue, enemy red (`look-and-feel.md`, *A sprite
unit's side*, decided 2026-10-02 and in 0436, "The outline alone, no
corner mark"; the deeper blue is from 0447). Harl also has the boss
mark. That is the rule as written (`look-and-feel.md` ~:351: "A sprite
hangs off a class or a named character, not a side: the outline says
whose a unit is. So an enemy Archer looks like ours"). Ticket 0499 added
the riders under the agent's starting rule "Red for every rider, ours
and theirs", meant for the Act 1 playtest's checklist (1064), which
hasn't happened yet.

The combat pictures already tell them apart. Nick decided in 0413 and
0446 that named characters use their class's picture recoloured to
their own colours: Hollis's trim is faded Veyne blue against the Red
Company's red; Aske is the Archer as bought against archers with dark
hair and red clothes; Tamsin keeps the gold trim and lance against
riders in red trim (`look-and-feel.md`, *Who is recoloured in Chapter
1*). The map sprites have no such recolour, and the map-sprite import
(`crates/tools/src/map_sprite_import.rs`) has no recolour step.

A second, related point: a rider's frame is 24x24 on a 16-pixel tile
(0499's deviation). It is drawn at its own size, centred
(`crates/ui/src/map_view/sprite/units.rs`, `unit_dest`), so it reaches
4 pixels into the tiles to its left and right. The clip mask (0039,
0436; `place`) cuts a sprite only under a unit standing on the tile
*above*. So beside another unit, a rider's lance or tail is drawn into
that unit's tile. The whole map scales, so the spill is the same share
of a tile at 1x and 2x.

Why now: a test audit of the Chapter 1 web demo (0914, 0915) found
this. The demo is built with the bought art, and Nick is about to send
it to about five applicant writers (0766). They will play it, often at
1x in a browser window.

This is not a blocker for the writer trial (0766): Question 3 lets Nick
choose whether the change must be in the demo first. The Act 1
playtest (1064) does need the answer.

## Nick input

**Decision** (`ask-nick`), asked with `cargo shuyi frame-png` pictures
of the game's own frames. Each picture is cropped to a few tiles and
staged on a throwaway `zz_` battle file, never a real map (ADR-0118),
and never the raw bought files (ADR-0040). Show each pair (Tamsin
beside a Red Company rider, Hollis beside a Red guard, Aske beside a
Red archer, Harl beside a brigand) at 2x and at 1x, standing and acted
(grey). Expect several rounds of review: Nick asks for changes and
new options before he picks.

**Question 1: how should the player tell their own unit from an enemy
of the same class on the map?** Mixes are fine (e.g. C with B).

- **A. Keep today's rule:** the side's outline alone (0436), the boss
  mark on Harl. Nothing is built.
- **B. Recolour every sprite by side.** In Fire Emblem's GBA games and
  Advance Wars, the player's units are blue, the enemy's red and allies
  green on the map, whatever their class. Here the outline stays as
  well. For the riders, MattWalkden's pack already ships each unit in
  blue, green and red (0499's notes). The Tiny Tales sprites would be
  recoloured by the agent, a small edit the bought-art rules allow
  (`look-and-feel.md`, *Claude may only make small edits*). Tamsin's
  map rider would then be blue, not her red.
- **C. Give named characters a map recolour of their own, matching
  their combat picture** (0413 and 0446 carried onto the map): Hollis
  in faded Veyne blue, Aske as bought against dark-haired archers in
  red, Tamsin's gold trim against red trim, and Harl in his own
  colours. This is how Final Fantasy Tactics and Fire Emblem: Three
  Houses work: named characters look like themselves and generic
  soldiers share a class look. The cost: at 16-24 pixels a trim colour
  is a few pixels, so the pictures must show whether it reads at 1x.
- **D. A stronger side mark than the outline.** For example, a coloured
  ring or base under each unit's feet in its side's colour, as Triangle
  Strategy marks its sides on the ground. Or the 2-pixel outline Nick
  saw in 0447 (he chose 1 pixel then, for the player's blue only), for
  every side. 0436's corner mark was turned down and is not offered
  again.
- **E. Describe your own.**

**Question 2: the rider reaching 4 pixels into the tiles beside it.**

- **A. Keep it.** It already reaches into empty ground, as every head
  reaches into the tile above.
- **B. Shave it beside a unit.** Extend the clip mask sideways: a rider
  is not drawn inside a tile to its left or right where another unit
  stands, as a head is shaved under a unit above (0039, "A is best").
- **C. Neighbours in front.** The unit beside a rider is drawn over the
  rider's spill, so the spill shows only on empty ground.
- **D. Describe your own.** A narrower 16-pixel cut was ruled out in
  0499 (it loses the lance and the tail, and scaling pixel art down
  ruins it), so the pictures don't show it unless Nick asks.

**Question 3: must the change be in the writers' demo before the
packets go out (0766)?** Yes: the build ticket says it ships before
the packets go out. No: it waits for the Act 1 playtest (1064).

## Scope

**In:**
- The three questions above, with the `frame-png` pictures. The mock-up
  plumbing is a throwaway `zz_` battle file plus an env-var mock in the
  sprite skin that `frame-png` renders with, both reverted before the
  PR.
- Nick's answers and words in `docs/design/look-and-feel.md` (*A sprite
  unit's side*, *Which sprite each unit is*, *Who uses what*), replacing
  0499's starting rule "Red for every rider, ours and theirs" if he
  changes it, and a row in `docs/design/README.md`.
- One build ticket (12xx) for what he picks, unless he keeps A for both
  questions. It goes in 1064's `blocked_by`; if Question 3 is yes, its
  Context says it ships before the writers' packets go out.

**Out (do not do):**
- Building any of it: no tileset, import or skin change ships in this
  ticket.
- The glyph look (its units are letters in the side's colour) and the
  Playdate's one-bit map (0067).
- A rider on foot's look (0089) and the combat pictures (0413 and 0446
  already decided them).
- Any new art purchase. If Nick wants one, that is its own setup ticket.

## Implementation steps

1. Read `look-and-feel.md` (*A sprite unit's side*, *Which sprite each
   unit is*, the clip mask rules around ~:440 and ~:462, *Who is
   recoloured in Chapter 1*, the bought-art edit rules), 0436's, 0447's
   and 0499's completion notes, `crates/ui/src/map_view/sprite/units.rs`
   (`outline_color`, `unit_dest`, `place`) and the private tileset's
   `units` table.
2. Run `cargo shuyi private-assets` and stage the four pairs on a `zz_`
   battle file: open ground, units side by side, one acted. Render each
   option of Question 1 with `cargo shuyi frame-png`, at 2x and 1x,
   cropped to the pairs. Recolours for B and C are scratch PNGs in
   the scratchpad, never committed.
3. Render Question 2's options: a rider between two units, at 2x and
   1x.
4. Ask Nick (`ask-nick`, the question pop-up); repeat rounds as he
   asks.
5. Record his answers in `look-and-feel.md` and `docs/design/README.md`;
   revert the mock-up plumbing.
6. Write the build ticket (`write-ticket`, numbered in 12xx: 04xx is
   full). It names the tileset entries
   (e.g. `units.characters` for `sergeant`, `retainer`, `poacher`,
   `red_captain`), the import changes (a recolour step in
   `map_sprite_import.rs` if B or C wins) and the skin change (D, or
   Question 2's B or C). Then add it to 1064's `blocked_by`. If
   Question 3 is yes, the build ticket's Context and this ticket's
   Completion notes say it must ship before the packets go out.

## Acceptance criteria

- [ ] `look-and-feel.md` records Nick's answer to each of the three
      questions, with his words and the date.
- [ ] `docs/design/README.md` has a row for this decision.
- [ ] A build ticket exists in `tickets/open/` for what he picked
      (or the Completion notes say he kept A for both questions),
      and it is in 1064's `blocked_by` (and, if Question 3 is yes,
      its Context says it ships before the packets go out).
- [ ] No mock-up file, `zz_` battle or env-var mock is left in the
      diff (`git diff --stat origin/main` shows only docs and
      tickets).
- [ ] `cargo xtask ticket-lint` passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

None (a decision): `cargo xtask ticket-lint`. The build ticket carries
the tests (the sprite skin's picture pick read from the scene and the
tileset, per ADR-0038 and ADR-0007).

## Completion notes

*(Filled in by the session that completes the ticket: what was done,
deviations, follow-up tickets created, notes for Nick.)*
