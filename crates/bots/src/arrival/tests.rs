use trpg_content::battle_campaign;
use trpg_core::{CharacterId, Faction, LeadGender, Unit};

use super::*;
use crate::LINES;
use crate::testkit::{TEST_RECRUIT, test_story};

/// The game's content, for the tests of its story (ADR-0118 (a)): they
/// play the recorded playthroughs of its chapters.
fn content() -> Content {
    trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"))
}

fn lead() -> LeadProfile {
    LeadProfile::new("Lead", LeadGender::Male)
}

/// The armies of `battle` in `mode`, a line each.
fn arrive(content: &Content, battle: &str, mode: GameMode) -> Option<Vec<BattleSetup>> {
    let def = &content.battles[battle];
    arrival(content, def, mode, &lead()).unwrap_or_else(|e| panic!("{e}"))
}

/// The armies of `battle` in Classic: there are some.
fn armies(content: &Content, battle: &str) -> Vec<BattleSetup> {
    arrive(content, battle, GameMode::Classic).unwrap_or_else(|| panic!("no army for {battle}"))
}

/// The player's units of `setup`, by character.
fn players(setup: &BattleSetup) -> Vec<(CharacterId, &Unit)> {
    let units = setup.units.iter().filter(|u| u.faction == Faction::Player);
    units
        .filter_map(|u| Some((u.character.clone()?, u)))
        .collect()
}

#[test]
fn nothing_comes_before_the_first_chapter() {
    let content = test_story();
    let first = "test_world_story";
    assert_eq!(arrive(&content, first, GameMode::Classic), None);
    assert_eq!(chapters_before(&content, first).len(), 0);
}

#[test]
fn chapter_2_is_played_with_the_army_chapter_1_leaves() {
    if trpg_content::bundle::skip_without_the_game() {
        return;
    }
    let content = content();
    let before: Vec<&str> = chapters_before(&content, "ch02")
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(before, ["ch01"]);
    let potion = ItemId::new("potion");
    let price = content.items.consumable(&potion).map_or(0, |c| c.price);
    for mode in [GameMode::Classic, GameMode::Casual] {
        let lines = arrive(&content, "ch02", mode).unwrap_or_else(|| panic!("no army"));
        assert_eq!(lines.len(), LINES, "a line each");
        let def = &content.battles["ch02"];
        let fresh =
            battle_campaign(&content, def, mode, lead()).battle_setup(def, &content.tables());
        let fresh = players(&fresh);
        for setup in &lines {
            let army = players(setup);
            assert_eq!(army.len(), 6, "the six of Chapter 1");
            for (who, unit) in &army {
                let (_, data) = fresh
                    .iter()
                    .find(|(c, _)| c == who)
                    .unwrap_or_else(|| panic!("{who:?}"));
                assert!(
                    unit.level > data.level || unit.exp > data.exp,
                    "{who:?} gained nothing in Chapter 1: lv {} exp {}",
                    unit.level,
                    unit.exp
                );
                assert_eq!(unit.hp, unit.stats.hp, "{who:?} rested");
            }
            // Chapter 1's clear gold buys Potions at Chapter 2's shop, until
            // the pack is full or the gold runs out.
            let pack = &setup.pack.items;
            assert!(pack.iter().all(|i| *i == potion), "{pack:?}");
            assert_ne!(pack.len(), 0, "a pack of Potions");
            assert!(pack.len() == setup.pack.cap || setup.gold < price);
        }
    }
}

/// Every line brings its own army (Nick, 2026-11-21: "Several
/// playthroughs"): the lines don't all leave the same levels.
#[test]
fn each_line_brings_its_own_army() {
    if trpg_content::bundle::skip_without_the_game() {
        return;
    }
    let content = content();
    let levels =
        |s: &BattleSetup| -> Vec<u32> { players(s).iter().map(|(_, u)| u.level).collect() };
    let lines: Vec<Vec<u32>> = armies(&content, "ch02").iter().map(levels).collect();
    assert!(lines.iter().any(|l| *l != lines[0]), "{lines:?}");
}

/// On the Act 1 map the fixed skirmishes that guard the way to a story
/// battle are won before it, and only those (1071): `aldwater` before
/// `ch04`'s battle, `salt_road` before `ch05`'s; never an optional one. A
/// guard is fought with the army its chapter brings.
#[test]
fn the_guards_on_the_way_are_won_first() {
    if trpg_content::bundle::skip_without_the_game() {
        return;
    }
    let content = content();
    let chapters = story_chapters(&content);
    let guards = guards_by_chapter(&content, &chapters);
    let by_chapter: Vec<(&str, Vec<&str>)> = chapters
        .iter()
        .zip(&guards)
        .map(|(c, g)| (c.id.as_str(), g.iter().map(|(_, b)| b.as_str()).collect()))
        .collect();
    let of = |id: &str| {
        by_chapter
            .iter()
            .find(|(c, _)| *c == id)
            .map(|(_, g)| g.clone())
    };
    assert_eq!(of("ch03"), Some(vec![]));
    assert_eq!(of("ch04"), Some(vec!["aldwater"]));
    assert_eq!(of("ch05"), Some(vec!["salt_road"]));
    let before: Vec<&str> = chapters_before(&content, "salt_road")
        .iter()
        .map(|c| c.id.as_str())
        .collect();
    assert_eq!(before, ["ch01", "ch02", "ch03", "ch04"]);
    assert_eq!(
        chapters_before(&content, "millhaven_road").len(),
        0,
        "optional"
    );
    // The guard's army is Chapter 3's, and Chapter 4's has won it too.
    let bridge = armies(&content, "aldwater");
    let hall = armies(&content, "ch04");
    let levels = |s: &BattleSetup| players(s).iter().map(|(_, u)| u.level).sum::<u32>();
    for (bridge, hall) in bridge.iter().zip(&hall) {
        assert!(levels(hall) > levels(bridge), "EXP from the bridge");
    }
}

#[test]
fn the_army_is_the_same_every_time() {
    if trpg_content::bundle::skip_without_the_game() {
        return;
    }
    let content = content();
    let first = arrive(&content, "ch02", GameMode::Classic);
    assert_eq!(first, arrive(&content, "ch02", GameMode::Classic));
}

#[test]
fn a_battle_no_chapter_fights_has_no_arrival() {
    let content = test_story();
    assert_eq!(chapters_before(&content, "quick").len(), 0);
    assert_eq!(arrive(&content, "quick", GameMode::Classic), None);
}

#[test]
fn the_pack_takes_the_stock_first_then_what_the_gold_buys() {
    let content = test_story();
    let def = &content.battles["test"];
    let mut setup = battle_campaign(&content, def, GameMode::Classic, lead())
        .battle_setup(def, &content.tables());
    setup.stock = trpg_core::Stock::default();
    setup.pack.items.clear();
    setup.stock.add(ItemId::new("elixir"));
    setup.gold = 0;
    let shop = content.basic_shop.shop(2);
    prepare(&mut setup, Some(&shop));
    assert_eq!(
        setup.pack.items,
        [ItemId::new("elixir")],
        "no gold, nothing bought"
    );

    let potion = ItemId::new("potion");
    let price = content.items.consumable(&potion).map_or(0, |c| c.price);
    setup.gold = price * 2 + 1;
    prepare(&mut setup, Some(&shop));
    let pack = &setup.pack.items;
    assert_eq!(
        pack,
        &[ItemId::new("elixir"), potion.clone(), potion],
        "two Potions bought"
    );
    assert_eq!(setup.gold, 1, "paid for");
    assert!(setup.stock.items.is_empty(), "everything packed");
}

#[test]
fn a_full_pack_buys_nothing_more() {
    let content = test_story();
    let def = &content.battles["test"];
    let mut setup = battle_campaign(&content, def, GameMode::Classic, lead())
        .battle_setup(def, &content.tables());
    setup.stock = trpg_core::Stock::default();
    setup.pack.items.clear();
    let potion = ItemId::new("potion");
    let price = content.items.consumable(&potion).map_or(0, |c| c.price);
    let cap = u32::try_from(setup.pack.cap).unwrap_or(0);
    setup.gold = price * (cap + 2);
    prepare(&mut setup, Some(&content.basic_shop.shop(2)));
    assert_eq!(setup.pack.items.len(), setup.pack.cap);
    assert_eq!(setup.gold, price * 2, "only the pack's worth bought");
    assert_eq!(
        setup.stock.count(&potion),
        0,
        "nothing bought for the stock"
    );
}

#[test]
fn whoever_a_scene_has_recruited_by_the_chapter_joins() {
    let content = test_story();
    let mut campaign = new_campaign(&content, GameMode::Classic, lead());
    let recruited = CharacterId(TEST_RECRUIT.into());
    recruit(&mut campaign, &content, "test_wm1");
    assert!(
        campaign.member(&recruited).is_none(),
        "not before the scene"
    );
    recruit(&mut campaign, &content, "test_wm2");
    let joined = campaign
        .member(&recruited)
        .unwrap_or_else(|| panic!("{TEST_RECRUIT} didn't join"));
    assert_eq!(joined.faction, Faction::Player);
    assert_eq!(campaign.roster.len(), content.new_game.roster.len() + 1);
}

/// Each earlier chapter's battle is played with whoever had joined by
/// then (0850): a recruit from an earlier chapter's scene fights the
/// recorded playthroughs after their join on the way to Chapter 4, which
/// fail without them.
#[test]
fn a_recruit_fights_the_chapters_after_their_join_on_the_way() {
    if trpg_content::bundle::skip_without_the_game() {
        return;
    }
    let content = content();
    let defector = CharacterId("defector".into());
    for setup in armies(&content, "ch04") {
        let army = players(&setup);
        assert!(army.iter().any(|(who, _)| *who == defector), "defector");
        assert_eq!(army.len(), 7, "the New Game's six and the recruit");
    }
}

/// How far apart the two halves' mean levels may be (ADR-0130), a unit's:
/// the first recording's widest gap was 1.25 levels (the poacher, Chapter
/// 4).
const UNIT_MARGIN: f64 = 2.0;
/// The same for the whole army (every unit's level, averaged): the first
/// recording's widest was 0.43 (Chapter 5).
const ARMY_MARGIN: f64 = 0.75;

/// The tolerance check (0531, ADR-0130): the odd lines and the even ones,
/// recorded with different seeds, bring every later battle armies alike,
/// each unit's mean level within [`UNIT_MARGIN`] and the army's within
/// [`ARMY_MARGIN`]. A set of lines that hangs on one recording's luck again
/// (fewer lines, or lines that all start from one) fails it.
#[test]
fn the_lines_halves_bring_armies_alike() {
    if trpg_content::bundle::skip_without_the_game() {
        return;
    }
    let content = content();
    let mut battles = won_on_the_way(&content);
    battles.extend(story_chapters(&content).last().map(|c| c.battle.clone()));
    let mut checked = 0;
    for battle in &battles {
        let Some(lines) = arrive(&content, battle, GameMode::Classic) else {
            continue;
        };
        checked += 1;
        let mean = |odd: usize| -> Vec<(CharacterId, f64)> {
            let half: Vec<&BattleSetup> = lines.iter().skip(odd).step_by(2).collect();
            let who = players(half[0]);
            who.iter()
                .map(|(c, _)| {
                    let levels = half.iter().filter_map(|s| {
                        let army = players(s);
                        army.iter()
                            .find(|(d, _)| d == c)
                            .map(|(_, u)| f64::from(u.level))
                    });
                    let levels: Vec<f64> = levels.collect();
                    let n = f64::from(u32::try_from(levels.len()).unwrap_or(1));
                    (c.clone(), levels.iter().sum::<f64>() / n)
                })
                .collect()
        };
        let (even, odd) = (mean(0), mean(1));
        let mut gaps = Vec::new();
        for ((who, a), (_, b)) in even.iter().zip(&odd) {
            gaps.push((who.0.clone(), (a - b).abs()));
        }
        let army = |m: &[(CharacterId, f64)]| {
            m.iter().map(|(_, l)| l).sum::<f64>() / f64::from(u32::try_from(m.len()).unwrap_or(1))
        };
        let army_gap = (army(&even) - army(&odd)).abs();
        assert!(
            army_gap <= ARMY_MARGIN,
            "{battle}: the army's mean level {army_gap:.2} apart"
        );
        for (who, gap) in gaps {
            assert!(
                gap <= UNIT_MARGIN,
                "{battle}: {who}'s mean level {gap:.2} apart"
            );
        }
    }
    assert!(checked >= 5, "every later battle: {checked}");
}

/// On the test story, with two lines of the first battle's wins: the next
/// battle has an army per line, line 1's first, each the campaign after
/// its own line's win; with no wins, the error names the battle to record.
#[test]
fn a_later_battle_has_an_army_per_line() {
    let content = test_story();
    let on_the_way = won_on_the_way(&content);
    assert_eq!(on_the_way, ["test_world_story", "test"]);
    let first = &content.battles["test_world_story"];
    let mut wins = Wins::default();
    for line in 0..2 {
        let start = start_of(&content, first, GameMode::Classic, lead(), &wins, line);
        let start = BattleState::new(start.unwrap().unwrap()).0;
        let seed = u64::try_from(line).unwrap() + 1;
        let won = crate::record(start, &content, seed, &|_| true).unwrap();
        wins.set("test_world_story", line, won.commands());
    }
    let next = &content.battles["test"];
    let armies = arrival_with(&content, next, GameMode::Classic, &lead(), &wins);
    let armies = armies.unwrap().unwrap();
    assert_eq!(armies.len(), 2);
    for (line, army) in armies.iter().enumerate() {
        let start = start_of(&content, next, GameMode::Classic, lead(), &wins, line);
        assert_eq!(Some(army), start.unwrap().as_ref(), "line {}", line + 1);
        assert_ne!(players(army).len(), 0);
    }
    let none = arrival_with(&content, next, GameMode::Classic, &lead(), &Wins::default());
    let error = none.unwrap_err();
    assert!(
        error.contains("no recorded wins of \"test_world_story\""),
        "{error}"
    );
}
