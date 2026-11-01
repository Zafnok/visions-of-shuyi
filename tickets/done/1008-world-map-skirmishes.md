---
id: "1008"
title: "World map skirmishes: fixed and random, with markers and level cap"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: done
blocked_by: ["1007", "0501", "0807"]
nick_input: none
completed: 2026-10-31
---

# 1008 — World map skirmishes

## Context

[`docs/design/world-structure.md`](../../docs/design/world-structure.md)
(ticket 0008): chapters on the world map can hold several battles before the
climax. Nick said a chapter may have "many random overworld triggered
skirmishes to train for a climactic battle" (`chapter-1.md`), and in 0008 he
picked a **mix**:

- **Fixed skirmishes:** hand-placed, cleared once. Some guard a path and must
  be won to pass.
- **Random skirmishes:** appear on cleared nodes after story battles. They're
  optional, keep coming, and their enemy level is **capped per act**.

Nick: make sure they're "clearly delineated so that players know which ones
are more important and which ones are just for grinding. Some symbol above
them or a level marker or both." The design uses **both**. 1007 chose
them (`look-and-feel.md`, *The world map*): symbol plates with the level
marker above, a fixed skirmish `!` on a brick red plate and a random one
`?` on a grey plate, beside the story battle's gold `‼` and the side
quest's green `♥`. Add the two marks to `PlaceMark` and both skins
(`crates/ui/src/screens/world_map/`), and to the Find list
(`FindKind`). Travel already goes around an unwon battle and stops at
one in the way (`trpg_core::worldmap`): a guarding fixed skirmish needs
nothing more than being a battle node. A random group must never spawn on
the army's own place (it would be a battle the army is standing on).

Skirmish battle files use the `music: Pool("skirmish")` field that 0807
adds to battle files (added to `blocked_by` 2026-10-01).

## Nick input

None. Nick no longer checks gameplay tickets after merge (Nick,
2026-10-21: "I will save all my checking for either design (i.e. 00
tickets) or for the full act 1 playtest itself"). Verification is by
tests, `cargo shuyi frame-png` pictures in the PR and the playtest bots
(0506, 0509). What the old sign-off asked goes on the checklist of
Nick's next playtest (1064, the Act 1 playtest, if this lands before
it):

> Nick plays the test world map, wins a few random skirmishes and
> a fixed one, and says whether (1) he can tell at a glance which fights matter,
> and (2) the random fights feel like useful training without being grindy.

## Scope

**In:**
- Node kinds `FixedSkirmish` (with a `guards_path` flag) and random skirmish
  groups, drawn with the 1007 symbols and level markers.
- Fixed skirmishes: authored battle data (map, enemies, tier, optional
  scene), cleared once. A guarding one blocks travel past it until it's won.
- Random skirmishes, with all numbers in a data file (*starting values,
  tunable*, from the design): after each story battle is won, **0–2** groups
  spawn on cleared nodes of the current act, with **at most 3** on the map.
  Enemy level cap = latest story battle's level in this act **−1**. Size is
  **4–6 enemies**, the map tier is **Easy**, and rewards are EXP plus
  sometimes a little gold or a common item (never unique items).
- Random skirmish maps: pick from a small pool of authored skirmish maps per
  act, with enemies built from templates at the capped level. (Later, 0510's
  generator can fill this pool with bot-balanced skirmishes; the pool format
  should be plain battle files so it can.) Spawns use the
  campaign's deterministic RNG (0304), so they're saved and reproducible.
- Skirmish music: every skirmish battle file uses `music: Pool("skirmish")`
  (0807; `docs/design/audio.md`: one track at random from the pool).
- The random-skirmish fights are ordinary battles. Death and retreat, rewind
  charges and the unused-charge bonus work as in `death-and-difficulty.md`.
- Test data: two fixed skirmishes (one guarding) and a random pool of two
  maps on the 1007 test world map.

**Out (do not do):**
- Real Act 1 skirmish maps and enemy lists (content tickets after 0701).
- FFT-style random encounters while travelling (Nick didn't pick them).
- The grinding tower (deferred).
- Visions on random skirmishes and stars on fixed ones (decided in 0046 and
  0047; their own implementation tickets build them on top of this one).
- Retuning support thresholds (a tuning ticket after the playtest).

## Implementation steps

Session estimate: about 650 production lines, using 1007's side-quest flow,
battle loader and saved world state; about 2,000 total changed lines including
tests, fixtures, documentation and the archived ticket (ADR-0092, ADR-0101).

**Size (0135, ADR-0101):** about 2,500-3,300 changed lines, a coin flip. Keep the tests lean. If it
passes 2,500, stop and split: core, content and enemy building stay here
(with the property and determinism tests); `PlaceMark`, both skins,
`FindKind`, hover and the snapshot go to a new ticket blocked by this one.

1. Read `world-structure.md`, `death-and-difficulty.md`, `supports.md`
   (*Gaining points*, pacing note), the 1007 world-map code and 0304's RNG.
2. `core::worldmap`: add the skirmish node kinds, spawn rules and cleared
   state as commands/events. Spawning happens as an event when a story battle
   is won, and it uses the campaign RNG.
3. Add a data file for the random-skirmish settings (spawn count, cap,
   offset, size, rewards) and the per-act map pool. Validate it with content
   validation.
4. Enemy building for random groups: templates + capped level + default
   loadouts (as in the chapter format).
5. UI: markers and hover info for both kinds (symbol + `Lv N`). A guarded
   path shows as blocked.
6. Tests, then a playtest build for Nick.

## Acceptance criteria

- [x] Random groups never exceed the act's level cap or 3 on the map (property test).
- [x] A guarding fixed skirmish blocks travel until it's won; others don't (tests).
- [x] The same save + same actions spawns the same groups (determinism test).
- [x] Fixed and random nodes use different symbols and both show a level marker (snapshot).
- [x] Starting rules and anything left to judge by playing are in the
      Completion notes (they go on 1064's checklist).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: spawn rules, level cap, guard blocking, reward rules.
- Property: over random story-battle win sequences, spawn count and levels stay within limits.
- Snapshot: world map with both skirmish kinds and hover info.
- Integration: win a story battle → groups appear → fight one → back on the map with it cleared.

## Completion notes

Implemented fixed skirmishes with battle/scenes data and a guard flag, and
saved random groups over cleared battle places. The campaign PCG stream,
group levels, counts, seeds and rewards survive saves, suspends and retries.
A group on an old story node never advances the story again. Only guarding
fixed fights block through travel; random groups and optional fixed fights
can be passed. Both skins share the red `!` / grey `?` plates, level labels,
Find rows and hover text; a guard's panel explains its blocked path.

Per-map `skirmishes/<world-map-id>.ron` files hold every tunable number and
the ordinary battle-file pool. Content validates settings, references,
Easy rout templates, default loadouts, music and rewards; it rebuilds each
generic at the saved level using the ordinary battle loader. ADR-0104 and
`assets/worldmaps/README.md` document the format and save behavior.

The test world map has two fixed fights and two random maps. Its debug
entry enables their test flag. Win Old Camp before Stone Mill to provide
a cleared place for a random group; the test story battle is now level 5,
so its training cap is 4. Demo random enemies are spaced, stationary
brigands; the test fights have 20-turn bounds. No real Act 1 content was
added, and no support threshold or ordinary battle/death/rewind rule changed.

**Starting rules for 1064's checklist:**
- The configured starting values are 0–2 groups per story win, at most 3,
  4–6 enemies, Easy tier and a level offset of 1. A group uses the cap itself
  (also bounded by the act's maximum), rather than rolling a lower level.
- *The agent's starting rule:* rewards are 50% no clear loot, 25% 100 gold,
  and 25% one potion, plus the ordinary combat and unused-rewind EXP.
- *The agent's starting rule:* choose unused cleared battle places, excluding
  the army; use the first N authored enemy entries, with template loadouts.
  If no valid level remains, nothing spawns. If a later story level lowers
  the cap, existing groups are lowered to it, or leave below the act minimum.
- *The agent's starting rule:* the campaign RNG starts at seed 0 on first
  use and continues across acts; each saved group keeps its rolled seed.
- What remains for the Act 1 playtest: whether the marks make fight importance
  obvious, and whether the real training content feels useful without grinding.
  Real maps/enemies and balance remain 1070/1071; Visions and stars remain out.

**Verification:** property tests cover changing story levels, count, size,
cleared places and the army's exclusion; save/action determinism and guard
travel have unit tests. The flow test wins a story battle, generates a group,
suspends/continues/retries its ordinary battle and returns to the map after
winning. A regression covers training on a cleared story node. The view
snapshot and skin assertions cover both symbols, levels, Find and hover.
Rendered frames are `docs/img/1008-fixed-skirmish.png` and
`docs/img/1008-world-map.png` (earned through the normal UI).

Personas, seeds 1–20 on each authored demo map (120 fights total): meadow
wins Casual 20/20, Normal 19/20, Hardcore 20/20; wood 18/20, 19/20, 20/20.
No-death/item success is 20/20, 19/20, 16/20 on meadow and 18/20, 18/20,
19/20 on wood. Median turns are 5/3/3 and 5/4/4. Both commands pass, with
some too-easy warnings; these are mechanical fixtures, not Act 1 balance.

Every command in the game's `run-gates` list passed: formatting, clippy,
workspace tests/doc tests, keys/text, pin/skill pack/skills, tickets/size,
warning-free docs, web build, three no-std builds, both Playdate SDK clippy
builds, dependency audit, machete and typos. The tooling test's temporary
Git repository and cargo-deny's database lock required filesystem access;
both passed on rerun. A native GNU playtest build was also produced.

The gate checkbox remains open for the engine pack's supplemental target
and fence checks: no targets or rules package are declared in this existing
project file. `check --target all` reports SH3003; lint lists the missing
fence as a limit. 0138 tracks that configuration work; the feature adds no
new gate exemption or suppression.

Engine friction: estimate 650, measured 428 added production lines (0.66×),
under 2×. PR size: estimate 2,000, measured 1,651 changed lines.
No engine change was needed. Follow-up 0138 records the existing missing
target-profile settings and rules-package lint wiring in shuyi.ron; the
supplemental target check reports SH3003 and lint reports that fence limit. The full UI fit within
1008's split threshold, so no split was necessary.
