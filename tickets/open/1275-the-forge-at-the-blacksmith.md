---
id: "1275"
title: "The forge at the blacksmith"
type: feature
milestone: Post–Chapter 1
model: sonnet-5 / gpt-6.1-terra
effort: medium
status: todo
blocked_by: ["1099"]
nick_input: answer-first
completed:
---

# 1275 — The forge at the blacksmith

## Context

Part of the 2026-11-16 hook chain (legendary beasts, their parts, and
forging; Nick's search for the game's storefront hook). Back in town, the
blacksmith forges a beast's material into a weapon that *does* something
new, never stronger than the chapter allows. 1099 builds the rules
(`crates/core/src/forge.rs`: `ForgeTable`, `Recipe`, `Tier`,
`Campaign::forge` with `ForgeOrder`/`ForgeAt`, `ForgeError`; materials as
`ItemDef::Material` in the party `Stock`; forged weapons as `WeaponDef`s
with `traits`). This ticket is the screen. **The rules come from 0087's
answers** in `docs/design/weapons-and-items.md` (*Forging*); where they
differ from the sketch here, the decision wins.

What it builds on:

- **The town's shop screen** (`crates/ui/src/screens/town.rs`,
  `TownScreen`, ticket 1007): tabs from `Tab::of(shop.kind)`
  (`crates/ui/src/screens/battle/shop.rs`; a Blacksmith has only
  `Tab::Repair`), one `Deal` per line with `TownTrade`, each line enabled
  by trying the deal on a copy of the campaign (`TownScreen::deals`),
  the special-weapon question as an `Ask<bool>`, and a
  `ShopScreenView` (`screens/battle/view/shop.rs`) painted by the PC skin
  (`screens/battle/glyph/shop.rs`) and the Playdate's
  (`screens/battle/playdate/shop.rs`, ticket 1066) through `SKINS`
  (ADR-0054). The Repair tab's `repairs` (the stock's weapons, then each
  held weapon with its holder's name) is the model for the forge's list.
- **Opening a town** (`crates/ui/src/screens/world_map.rs`,
  `open_town`/`on_town`): one shop opens at once, several are listed by
  `kind_key`.
- **The test world map** (debug menu "Test world map",
  `FlowScreen::test_world_map` in `crates/ui/src/flow.rs`;
  `assets-test/worldmaps/test_world.ron`, whose capital has a
  Blacksmith; its chapters `test_wm1`–`3` are number 4).
- **Text** in `assets/lang/en/ui.ron` (`battle.shop.*`); key names only
  through placeholders (`{Confirm}`, …), never a hard-coded key
  (`keyboard-input` skill).

**Either order with 1216** (one shop flow for the three shops): if 1216
is in `done/`, the forge is a `Trader` row kind on its `ShopPicker` in
`crates/ui/src/screens/shop.rs`; if not, build it in `town.rs` as below
and 1216 moves it with the rest. **Either order with 1202** (town state
in the scene tool): if 1202 is done, add the forge tab's rows to the
town-shop scene data it built; if not, 1202 reads them from the view.

## Nick input

**Answer first:** ticket 0087. Never a check after merge: it is verified
by tests, snapshots and `cargo shuyi frame-png` pictures in the PR. The
agent's starting rules below and how forging feels go in the Completion
notes for the next playtest's checklist (1064's, or the chain's).

## Scope

**In:**
- A `Forge` tab on a **town** Blacksmith, after `Repair`: each weapon the
  party has that a reached recipe forges, one line per (weapon, recipe).
- Each line: the weapon and what it becomes, the materials it takes
  (have / need), the gold, and the result's trait text; dimmed when the
  materials or the gold are short (as an unaffordable Buy line is).
- A yes/no question before forging (forging can't be undone), cursor on
  No, as the special-weapon sale asks.
- The PC skin and the Playdate one-bit skin.

**Out (do not do):**
- The rules, the data and the checks (1099). The grinder bot (1276).
- Forging at an on-map Blacksmith in battle or in Preparations (unless
  0087 asks; then a follow-up ticket).
- Showing recipes not reached yet, or a weapon tree view, unless 0087
  asks for them (then they are in scope here).
- New Actions or default keys.

## Implementation steps

1. Read 0087's *Forging*. Anything about the screen it doesn't decide is
   *the agent's starting rule*, named in the Completion notes.
2. `Tab::Forge` with its key `battle.shop.forge`. A town Blacksmith's tabs
   are `[Repair, Forge]`; the battle's Blacksmith keeps `[Repair]`:
   `TownScreen::tabs()` adds `Forge` for a Blacksmith rather than
   changing `Tab::of`.
3. `TownScreen::forges(ctx)`: the chapter number from
   `ctx.game.content.chapters.get(&campaign.chapter)` (`ChapterDef::number`;
   no number, no forge lines), then for each stock weapon copy and each
   held weapon (holder named, as `repairs` does) every recipe of
   `ctx.game.content.forge.recipes_for(weapon, chapter)`. A `Deal` gets a
   forge trade (`TownTrade::Forge(ForgeOrder)`, or a separate enum if
   1099 kept `forge` apart from `trade`), enabled by trying
   `Campaign::forge` on a copy as the other tabs do.
4. The line: name `Iron Spear → Rime Lance`, gold the forge's price,
   detail the materials (`Rime Shard 2/2`); the focused line's trait text
   (`Frostbite: a hit leaves frost on the target's tile`) in a new
   `ShopScreenView::note: Option<String>` (`None` on every other tab).
   Paint the note in `glyph/shop.rs` under the list and in
   `playdate/shop.rs` wrapped by the engine's rule (1077); every other
   shop's view sets `None`, so their snapshots don't change.
5. Confirm on an enabled line opens an `Ask<bool>` (`battle.shop.forge_ask`:
   "Forge {item} into {result}? It can't be undone."), as `special` does
   for a sale; Yes forges through `Campaign::forge`, plays the confirm
   sound, and the list rebuilds (the copy is now the forged weapon). The
   help line while asking is `battle.shop.help_special`.
6. Text keys in `assets/lang/en/ui.ron`, American English; key names as
   placeholders; the `test` language pack gets them too.
7. Test data: give the test world map's campaign
   (`FlowScreen::test_world_map`) the test materials and a test weapon of
   1099's test recipes: one recipe forgeable (its tier reached by
   chapter 4), one short of a material, and none of a tier past chapter 4.
8. Expected size: about 200 production lines (the forge list about 80,
   like `repairs`; the question and the forge call about 50; the view
   field and two skins about 70). Measure with `cargo xtask code-lines`
   (ADR-0092).

## Acceptance criteria

- [ ] `town::tests`: a town Blacksmith shows `Repair` and `Forge`; an
      Armory, a Tailor and the battle's Blacksmith don't show `Forge`.
- [ ] The Forge tab lists a stock weapon and a held weapon (holder
      named) for each reached recipe, and nothing for a recipe whose tier
      the chapter hasn't reached.
- [ ] A line short of a material or of gold is dimmed and Confirm does
      nothing.
- [ ] Confirm, Yes: the weapon becomes the forged one, the materials and
      gold are taken, and the campaign the screen pops with has them;
      Confirm, No (or Cancel) changes nothing.
- [ ] Snapshots of the Forge tab and of its question (PC); the other
      shops' snapshots are unchanged.
- [ ] `cargo shuyi frame-png` pictures in the PR of the Forge tab, PC and
      `--playdate`, from the debug menu's Test world map (as 1007's
      pictures: `--keys "F2"` then the menu's keys to the town).
- [ ] `cargo shuyi check-keys` and `cargo shuyi check-text` pass.
- [ ] All gates in the `run-gates` skill pass.

## Tests required

- Unit: the tabs per shop, the forge lines per chapter, enabling, the
  question, the campaign after forging.
- Snapshot / integration: the Forge tab and its question; a Harness
  script from the test world map: town → Blacksmith → Forge → Yes →
  leave → the stock and gold changed on the map's campaign.

## Completion notes

*(Filled in by the session that completes the ticket: what was done, deviations,
follow-up tickets created, notes for Nick.)*
