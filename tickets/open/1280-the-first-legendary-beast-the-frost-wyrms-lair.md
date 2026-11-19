---
id: "1280"
title: "The first legendary beast: the frost wyrm's lair"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1279", "1281", "1096", "1097", "1098", "1094", "1275", "1276"]
nick_input: answer-first
completed:
---

# 1280 — The first legendary beast: the frost wyrm's lair

## Context

The end of the hook chain Nick settled on 2026-11-16 after five research
rounds: legendary beasts hidden at the bottom of dungeon delves, fought
as a raid (0085–0088, 0772, 1091–1099, 1275–1281). Every system is built
by the tickets this one waits for; this ticket puts the first real beast
in the game, the example used through the whole chain: the **frost
wyrm**. It is the first time all of them meet in one fight, so it is
also their check: the hunt finds it, the delve leads to it, its parts
break and fall as ice, a perfect parry steals the turn, its materials
forge a weapon, and the bots say whether it is fair.

**The rules come from 0087's answers in `docs/design/beasts.md` (and
`weapons-and-items.md`, *Forging*; `playtest-bots.md`, *The grinder
bot*), 0088's in `delves.md`, 0085's in `magic.md` and 0086's in
`timing.md`. Where they differ from the sketch here, the decisions
win.** The story's framing is decided by 0087 and recorded in the
private repository's `story/design/` (ticket 0772); none of it goes in
this public repository.

What it builds on:

- **The hunt and the delve**: 1279 (`Hunt` on the world map, rumors,
  tracks, the column) and 1277 (`DelveDef`, generated floors, an authored
  `last` floor); 1278's dark floors if it is done (it isn't in
  `blocked_by`: if it isn't done, the lair's floors are lit and a line
  goes in 1278 to darken them).
- **The beast**: 1095 (the Beast army and its Beast Phase), 1096 (a
  beast on several tiles, each body tile a part with its HP and its
  attacks), 1097 (aiming at a part: the part bar in the forecast, the
  glowing part, the combat frame), 1098 (a broken part falls as terrain,
  an ice claw's shards freezing river tiles, or is carved into a
  material), 1091 (element states on tiles, and the wyrm's innate Ice).
- **Steal the Turn** (1094) and timed presses (1092, 1093): a part
  broken during a stolen move gives the rare material.
- **Materials and forging** (1099, 1275): the wyrm's materials feed a
  weapon tree such as Iron Spear → Steel Spear → Rime Lance, gated by
  chapter. The grinder bot and the shop-only balance check are 1276.
- **Where the game's content lives**: the acts' world maps, battles,
  maps and data are in the private repository's `content/` (ADR-0117),
  changed with the `content-change` skill (a pull request there, then
  this repository's pin `assets-private.rev`). Tests of the real content
  follow ADR-0118.
- **The art**: Nick bought the whole Tiny Tales bundle (ADR-0040). Its
  *Epic Monsters* pack (artist Aekashics, named in the bundle's row of
  `THIRD_PARTY_ASSETS.md`) is sorted in
  `assets-private/library/tiny-tales/characters/battler-classes/epic-monsters/`:
  each monster has a `battler.png` (about 130–170 × 100–110 px, inside
  the combat scene's `MAX_SCALED_PX` of 240 × 144,
  `crates/content/src/battler.rs`) and a `map_sprite.png` (a 48 × 80
  sheet of 16 × 20 frames, as every bought map sprite). Combat pictures
  come in through the table `BATTLERS` of `cargo xtask battler-import`
  (`crates/tools/src/battler_import.rs`) and map sprites through
  `SPRITES` of `cargo xtask map-sprite-import`. ADR-0051: the credit in
  `assets/data/credits.ron` names the new private files with a narrow
  `private` path, or `cargo shuyi private-assets --pin` refuses the pin.
  How a beast on several tiles is drawn on the map is 1096's rule; this
  ticket only gives it the wyrm's picture.

## Nick input

**Answer first:** ticket 0087 (which act and region the first beast is
in, its parts and attacks, its materials and traits, how long before it
retreats deeper, the grinder bot's margin) and 0088 (its delve's
floors). **Before building**, if 0087 didn't settle it: which *Epic
Monsters* picture is the frost wyrm. Show Nick three or four of the
pack's dragons as the game draws them (combat scene and map, both looks,
by `cargo shuyi frame-png` from a local build with the private assets);
the pictures stay local, never committed (ADR-0040). Nothing else is
asked. Verified by tests, frame-png pictures and the bots; what is left
to judge by playing goes in the Completion notes for a later playtest.

## Scope

**In (in the private repository's `content/`, by `content-change`,
unless said otherwise):**
- The frost wyrm's data: its parts, their HP and attacks, its innate
  element, its materials (common and rare), its retreat rule's numbers,
  in the formats 1095–1098 made.
- Its lair: the delve's data (floors, levels, the Way Out), and the
  authored last floor (map and battle) with river tiles for its broken
  ice claw to freeze, as 0087 places it.
- Its hunt on the act's world map 0087 names: the rumor's town, the
  track places, the column.
- The weapons its materials forge, in 1099's recipe data, within the
  chapter's tier.
- The art (bought, `assets-private/game/`): its combat picture and map
  sprite through the two import tables, its credit in `credits.ron`, and
  the bundle's row in `THIRD_PARTY_ASSETS.md` naming *Epic Monsters* as
  used (this repository).
- Balance: the lair in its bands for the three personas, and the
  grinder bot inside Nick's margin (1276).

**Out (do not do):**
- Any new rule or screen: if the fight needs one, it is a bug or a gap
  in 1091–1099 or 1275–1279; write a ticket (`write-ticket`) and name it
  in the Completion notes.
- Dialogue, scenes or battle notes for the hunt or the lair: dialogue
  writing is paused (0766); they are a writer's ticket after 0772.
- Other beasts (content tickets after this one).
- Drawing or editing the wyrm's art beyond the import tool's recolours
  (`ascii-art`: character art is bought, never drawn).
- Its one-bit Playdate picture by hand: the Playdate build converts every
  PNG under `assets-private/game/` (`crates/content/build.rs`,
  `cargo xtask playdate-images`); if 1069 is done, look at the conversion
  as it says; if not, add the wyrm to 1069's list.

## Implementation steps

**Size (ADR-0092, ADR-0101):** about 100 production lines here (the two
import tables, the credit, a content test) and about 400 lines of
hand-written data in the private repository; about 1,200–1,600 changed
lines in all with tests, the bots' report and the archived ticket
(comparable: a chapter's battle, about 2,250, which has dialogue this
one doesn't).

1. Read the five design docs above, 1095–1099, 1275–1279's code and
   data formats, `content-change`, and ADR-0040, 0051, 0117 and 0118.
   `cargo shuyi private-assets --library` for the bought packs.
2. **The picture**: Nick's pick (Nick input), added to `BATTLERS` as
   `frost_wyrm` (its facing and scale) and to `SPRITES`; run both import
   tools; push in `assets-private/`; add the credit's `private` paths
   (`battlers/frost_wyrm.`, `units/frost_wyrm.`) and the row's words.
3. **The beast** in the private `content/`: the wyrm's data and its
   materials, then the forge recipes. Run content validation.
4. **The lair**: the delve's data and the last floor's map and battle;
   then the hunt on the act's world map. A content test (ADR-0118: it
   reads the real content when it is checked out) checks that the hunt
   leads to the delve, the delve's `last` is the lair, and the lair
   holds the wyrm in the Beast army.
5. **Balance**: `cargo xtask playtest <lair> --bot all` with the army
   that arrives there (0520's arrival army);
   `cargo shuyi autobalance --stage <lair>` on the lair's knobs until the personas land in their bands (0509;
   the wyrm's part numbers are its knobs if 0087 allows); then 1276's
   grinder check on the lair and the story battles after it. Put the
   tables in `docs/playtesting.md` and the PR.
6. **Pictures**: frame-png of the hunt's marks on the act's map, the
   lair with the wyrm, the part bar in the forecast, a broken claw's ice
   on the river and the combat scene, in both looks and on the Playdate.
   Screenshots of the running game with the bought art may be committed
   (Nick, 2026-10-05); pictures pasted from the files may not.
7. Both pull requests as `content-change` says: the private one first,
   then this one moving `assets-private.rev`.

## Acceptance criteria

- [ ] The content test above passes with the private content checked
      out, and content validation passes.
- [ ] `cargo shuyi private-assets --pin` accepts the pin (every new
      bought file has a credit).
- [ ] The three personas are inside their bands on the lair, and the
      grinder bot is inside Nick's margin (reports in the PR).
- [ ] An integration test (private content checked out) plays the hunt to
      the lair with the bots and breaks the ice claw: its tiles freeze
      and its material is carved, as 1098 does.
- [ ] frame-png pictures listed in step 6 are in the PR.
- [ ] No story text beyond what public tickets already say is in this
      repository (`cargo xtask check-leaks`, the `content-change` skill).
- [ ] What is left to judge by playing is in the Completion notes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: none new unless a data check is missing (then in the format's
  own tests in `trpg-content`).
- Content: the hunt-to-lair test; content validation of every new file.
- Integration / bots: the hunt played to the lair; the personas' bands
  and the grinder bot's margin.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
