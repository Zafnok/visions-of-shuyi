---
id: "0878"
title: Every class placed in Act 1 has its art, or a named wait
type: content
milestone: M7 Chapter 1 & game flow
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: []
blocks: ["1064"]
nick_input: answer-first
completed:
---

# 0878 — Every class placed in Act 1 has its art, or a named wait

## Context

The friction sweep of 2026-11-17
([`docs/research/friction-2026-11-17.md`](../../docs/research/friction-2026-11-17.md),
*Built but not backfilled*, "Three enemy classes have no art; indoor
fights show an open field") found three classes placed in Act 1's
battles with no tileset unit and no battle picture: **Swordsman**,
**Brawler** and **Shadowblade**. Re-checked on 2026-11-17 in a current
private checkout: none of the three is in `game/tilesets/tiny_tales.ron`'s
`units.classes` or in `game/battlers/` (16 sidecars). Mapped through
`characters.ron`, about 54 of about 250 placements are these classes
(the sweep's skeptic count): they draw the tileset's `fallback` on the
map and the default picture in combat, and the all-assets test accepts
the fallback, so nothing flags it.

- **Brawler** is decided: its combat picture is Shironejiya's fist
  fighter (`martial03`) at 1.25× (`docs/design/look-and-feel.md`,
  *Fighters with no Tiny Tales art*, the *Who uses what* table, ticket
  0040), and the file is already in the private repository at
  `library/shironejiya/monster_s/m_s_martial03.png`. It was never
  imported. Its map sprite is "a Tiny Tales human" in that table, and
  *Which sprite each unit is* (ticket 0436) makes map sprites Claude's
  starting picks for Nick to change. Gil (`prizefighter`) is a Brawler
  with his own pictures (0862); the class picture is for the others.
- **Swordsman**, the most-placed of the three, is not in the *Who uses
  what* table at all: its art is Nick's call.
- **Shadowblade**: the only Shadowblade placed in Act 1 is Dace
  (`rival`), who gets his own combat picture in 0862 and map picture in
  0735. Whether the class needs a picture of its own is Nick's call.
- **Indoor, fort and bridge backdrops**: `battlebacks.ron` keys
  backdrops only on `mountain` and `peak`, so the 4 maps with indoor
  tiles (`envoys_house`, `greywater_kitchen`, `veyne_guardroom`,
  `veyne_study`) fight in front of `fields.png`, and so does every
  fight on a fort or a bridge tile, Chapter 1's boss fight included
  (Harl stands on the fort). The bundle holds no interior set: which
  pictures to use is Nick's call.

The three calls for Nick are asked in 0184's batch. This ticket doesn't
wait for them: it imports what is decided and adds the check, with each
undecided class as a named exception that points at the decision.

Bucket B1 of the sweep (Act 1 completeness). Its siblings: 0877 (the
per-chapter coverage report, whose battle-art and map-art columns show
the same gap), and the amendments to 1064, 0757, 0755, 0756, 0237,
0239, 0606, 1067, 1090 and 0522.

## Nick input

**Answer first, for two parts only:** Swordsman's art (combat picture
and map sprite), whether the Shadowblade class needs art of its own
besides Dace's, and the indoor, fort and bridge backdrops. They are asked
in 0184's batch, with options from the sources look-and-feel.md already
lists. If 0184 has recorded the answers when this ticket starts, do
those parts too. If not, leave them as named exceptions (step 4) and,
when the answers land, the session that records them writes the import
ticket (`write-ticket`).

The Brawler part needs no answer. Verified by the check's test and
`cargo shuyi frame-png` pictures built with `--features private-assets`
(screenshots only, ADR-0040); what is left to judge by playing goes on
1064's checklist.

## Scope

**In:**
- Brawler's combat picture: a row in `BATTLERS`
  (`crates/tools/src/battler_import.rs`) with the `shironejiya` helper,
  `m_s_martial03.png` at 1.25×, and `cargo xtask battler-import`.
- Brawler's map sprite: a starting pick from the bundle's Human NPC
  Advanced sprites in `crates/tools/src/map_sprite_import.rs`, and its
  row in `look-and-feel.md`'s *Which sprite each unit is* table marked as
  a starting pick for Nick to change, as the others are.
- Credits: the new files in the `shironejiya` and Tiny Tales credits'
  `private` lists in `assets/data/credits.ron`; `THIRD_PARTY_ASSETS.md`
  only if a row changes.
- A check in `crates/content/tests/it/private_assets.rs` (the one test
  that reads the bought art): every class and character placed in any
  battle, skirmish or side quest has a battle picture and a tileset unit
  of its own or of its class, or is on a short exception list where each
  entry names the ticket or decision it waits on.
- Push the private repository, `cargo shuyi private-assets --pin`,
  commit `assets-private.rev`.

**Out (do not do):**
- Choosing Swordsman's art, a Shadowblade class picture or the indoor,
  fort and bridge backdrops: Nick's, through 0184.
- Named characters' art: 0858, 0881, 0861, 0862, 0866, 0735.
- Promoted classes' art: they are not placed in Act 1, and whether Act 1
  promotes is asked in 0184 (see 0606).
- One-bit (Playdate) pictures: 1069.
- Moving the importers' tables into a data file (the sweep ranked it
  low).
- The coverage report itself: 0877. Either order: if 0877 is done, its
  art columns show the Brawler fixed; if not, nothing to do there.

## Implementation steps

Expected size: about 40 production lines (two table rows and the check)
and about 60 lines of test, judged by #718's importer share (33 lines).

1. `cargo shuyi private-assets --library` so `library/shironejiya/` is
   there.
2. Add the Brawler row to `BATTLERS` (the array's length goes from 16 to
   17) and run `cargo xtask battler-import`.
3. Pick a Brawler map sprite, add it to the sprite table of
   `map_sprite_import.rs`, run `cargo xtask map-sprite-import`, and add
   the row to `look-and-feel.md`.
4. Write the check in `private_assets.rs`: load the content, collect each
   placement's class through `characters.ron`, and fail with the list of
   classes and characters that draw the fallback and are not excepted.
   Exceptions today: `swordsman` (0184), `shadowblade` (0862 and 0735
   cover Dace; the class itself is 0184's). Run `cargo xtask check-leaks`;
   if it flags the ids in a public file, keep the list in the private
   repository's `content/tests/` (ADR-0118) instead.
5. If 0184 has answers: import those parts the same way and drop their
   exceptions.
6. Run `cargo test -p trpg-content --features private-assets --test it
   private_assets::` and take `frame-png` pictures of a Brawler on the
   map and in combat for the PR.
7. Push the private repository, pin, commit.

## Acceptance criteria

- [ ] A Brawler (not Gil) draws `martial03` in combat and its own sprite
      on the map in a `--features private-assets` build (pictures in the
      PR).
- [ ] The new private-assets test fails when a placed class has no art
      and is not on the exception list (shown by the test's own case),
      and passes on today's content.
- [ ] Each exception names the ticket or decision it waits on.
- [ ] `look-and-feel.md` has the Brawler's map sprite row.
- [ ] `assets-private.rev` is moved and the credits test passes.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the check's class collection on a small content set (a placed
  template whose class has no art, one excepted).
- Integration: the private-assets test above.
- Pictures: `frame-png` of the Brawler on the map and in combat.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
