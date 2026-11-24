---
id: "1276"
title: "The grinder bot and a shop-only balance check"
type: feature
milestone: Post–Chapter 1
model: opus-5.5 / gpt-6.1-sol
effort: high
status: todo
blocked_by: ["1099", "1098"]
nick_input: answer-first
completed:
---

# 1276 — The grinder bot and a shop-only balance check

## Context

Part of the 2026-11-16 hook chain (legendary beasts, their parts and
forging). Nick on crafting and balance: "if we require a minmax build the
game becomes grindy which stalls the SRPG flow. If we don't require a
minmax build then the game gets bloated numbers if a player does grind",
and "yes we would need a grinder bot". So battles are balanced against
**shop-only gear** (crafting is never required), and a **grinder bot**
farms and forges everything the rules allow, and must stay within a
**margin** Nick sets of the shop-only army. **The rules come from 0087's
answers** in `docs/design/playtest-bots.md` (*The grinder bot*, question
12: for example "at most 10 points more wins for the Casual bot, or at
most one turn faster") and `weapons-and-items.md` (*Forging*); where they
differ from the sketch here, the decision wins.

What it builds on:

- **The bots' army** (`docs/playtesting.md`, *The army*):
  `crates/tools/src/playtest.rs` `stage` builds a `BattleStage` (a
  `BattleSetup` per mode, `crates/bots/src/playtest.rs`) for the playtest
  and the autobalance alike; a story battle's army comes from
  `trpg_bots::arrival` (`crates/bots/src/arrival.rs`: earlier chapters won
  by their recorded playthroughs, then `prepare` fills the pack; weapons
  and armour stay as they are). That army never forges: it **is** the
  shop-only army.
- **Entrants and checks**: `BattleEntrant` (mode, `Player`, `start` picks
  `stage.setup(mode)`), `Targets` from `crates/bots/targets.ron` turned
  into the engine's `Check::Rate` and `Check::Median`
  (`Targets::checks_of`, the `Faster` comparison). The engine has no
  check of the gap between two entrants' rates; its `PlaytestReport`
  keeps every `Tried` with its measures, so the game can compute one.
- **Forging** (1099): `ForgeTable` (`Content::forge`), `Campaign::forge`,
  `ForgeOrder`, the tiers' unlocks, and the farm limit
  (`Campaign::gain_material`) if 0087 set one. **Materials from beast
  parts** (1098): which part drops what, and the rare one.
- CI: `.github/workflows/playtest.yml` plays every battle with `--bot all`,
  one job per battle (0530), when `.github/playtest-levers.sh` names a
  changed lever.

**Either order with 1207** (the engine's playtest command): if 1207 is in
`done/`, the grinder entrants go in `Trpg::playtest_plan`
(`crates/tools/src/game_tools/plays.rs`) and the margin is a summary item
there; if not, they go in `crates/tools/src/playtest.rs` (`Bot`, `stage`,
`render`) and 1207 carries them over with the rest. Either order with 1180
the same way (`Targets` moves there).

## Nick input

**Answer first:** ticket 0087 (*The grinder bot*: the margin, which
personas it applies to, and what a miss does). Never a check after
merge: verified by tests and the bots' reports in the PR.

## Scope

**In:**
- **The grinder's haul**: what the rules let a player gather by a battle
  (one-time sources won by then, plus repeatable sources up to the farm
  limit), computed from the data, not played.
- **The grinder's army**: the shop-only army, then before the battle's
  Preparations every forge the haul, the gold and the reached tiers allow,
  equipped.
- **Grinder entrants** (`casual+grind`, … for the personas the margin
  names) and the **margin check** against the same persona's shop-only
  rate (and turns, if 0087's margin has them), in the report and the CI.
- **The shop-only check**: a test that no story battle's bots army holds a
  forged weapon or forges, so the bands are always met without crafting.
- `docs/playtesting.md`, *The grinder*.

**Out (do not do):**
- Changing battles, recipes or traits to meet the margin (a tuning
  ticket per miss; the autobalance keeps tuning against the shop-only
  army only).
- Playing farm battles, delves (1277) or hunts (1279) for real: the haul
  is an upper bound from data, so no `cargo xtask playtest` stage for a
  whole delve is needed (1277 leaves that call here). Either order with
  1277 and 1279: if their loot is in `done/` data, `haul` reads it up to
  the farm limit; if not, it joins when they land (they hand materials
  over through `Campaign::gain_material`).
- The frost wyrm's lair (1280), which runs this check on its own battle.

## Implementation steps

1. Read *The grinder bot* and *Forging*. The shop-only army stays as it
   is; if 0087 says it also buys the chapter's best shop weapons, add
   that to `prepare` here and note the Act 1 numbers it changes.
2. `crates/bots/src/grinder.rs`: `haul(content, chapter) ->
   BTreeMap<ItemId, u32>`: every material of the story battles,
   guards and side quests won before the battle (`clear_items`, chests'
   `Loot::Item`, beast parts' drops from 1098's data, the rare one
   included), plus each repeatable source up to 0087's farm limit for the
   chapters so far. Document it as an upper bound.
3. `grind(campaign, content, chapter_number)`: adds the haul to the stock,
   then forges greedily and deterministically: units in roster order;
   for each, the reached recipe of the highest tier (then recipe order)
   whose `from` it holds or the stock holds and whose result it can wield
   (`Unit::can_wield`); `Campaign::forge` while materials and gold last;
   then the default equip. Forging pays from the campaign's gold before
   `prepare` buys the pack (*the agent's starting rule*, written in
   `docs/playtesting.md`).
4. `grinder_stage(content, def) -> Option<BattleStage>`: the army `stage`
   builds today (`arrival`, whose recorded playthroughs stay valid since
   nothing changes until the target battle; or a `bots_army`'s reference
   army, ground by its `chapter`), then `grind`, then `prepare`. `None`
   for an army fresh from the character data, and when the grinder's
   setups equal the shop-only ones (no recipe reached or nothing
   forgeable): the check is skipped and the report says so.
5. Entrants: `BattleStage` gets `grinder: Option<(BattleSetup,
   BattleSetup)>` and `BattleEntrant` a `Gear::{ShopOnly, Grinder}` field so
   `start` picks the right setup; the names add `+grind`. Only the
   personas the margin names play it (each persona try is costly:
   `docs/playtesting.md`, *Running it*).
6. The margin: `Targets::grinder` in `targets.ron` (per persona: the most
   rate points and, if 0087 has it, turns the grinder may gain), and
   `grinder_margin(report, targets) -> Vec<MarginVerdict>` computed from
   the report's tries with each persona's success conditions. A miss is
   what 0087 says; if it is silent, report only (✗), like a missed band.
   The generic "gap between two entrants' rates" is engine#2148's part E
   (`Check::Gap`, `Targets::gaps`; Nick asked for it, 2026-11-24). The
   turns margin stays the game's: engine#2148 leaves a median gap out.
   **Either order with engine#2148:** if the pin is past it, each
   persona's rate points are a `GapTarget` (`<persona>+grind` against
   `<persona>`, the persona's success conditions) and `grinder_margin`
   reads its `Found::Gap`; if not, `grinder_margin` computes the rate gap
   here, marked `// engine-friction: engine#2148`, and the game never
   waits for it.
7. The report: an `army · grinder …` line (what was forged, for whom) and
   one margin line per persona; `--grinder` on the playtest command (off
   by default), and CI's playtest job passes it for story battles.
   `assets-private/content/data/forge.ron` is a lever (1099 added it,
   after 0172; check).
8. Tests use the test files, never the private content: `grind` on a
   campaign built in `crates/bots/src/testkit.rs` with 1099's test
   materials and recipes; and a test battle with a `bots_army` (a
   chapter whose number reaches a test recipe) and a test material in
   its haul, for the command and the stage.
9. Expected size: about 350 production lines (`grinder.rs` about 200,
   like `army.rs`'s reference army; entrants and stage about 60; margin
   and report about 90), plus tests and docs: about 1,100 changed lines.
   Measure with `cargo xtask code-lines` (ADR-0092).

## Acceptance criteria

- [ ] `grinder::tests`: the haul by a test chapter counts each one-time
      material once and each repeatable one up to the farm limit; `grind`
      forges the expected weapons for the expected units, the same every
      run.
- [ ] `shop_only_army_never_forges`: for every story battle of the
      content, the bots' army (`stage`) holds no forged weapon.
- [ ] `cargo xtask playtest <test battle> --bot all --grinder` prints the
      grinder army line and a margin line per persona; a test with a
      scripted report shows a pass and a miss.
- [ ] On the real content before any recipe is reached, the report says
      the grinder check was skipped and every number is unchanged.
- [ ] CI's playtest job runs the grinder on story battles and stays
      within its time limit; the PR names the added minutes per battle.
- [ ] `docs/playtesting.md` has *The grinder* (haul, forging order,
      margin, what a miss does).
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the haul, the forging order, the stage with and without
  anything to forge, the margin from a report.
- Property: the grinder army's setup is deterministic (same content,
  same setup); it never holds a weapon of a tier not reached.
- Bots / integration: the playtest command on a test battle with
  `--grinder`; the shop-only test over every story battle.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
