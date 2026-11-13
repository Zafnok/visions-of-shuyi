use trpg_content::battle_campaign;
use trpg_core::{CharacterId, Faction, LeadGender, Unit};

use super::*;
use crate::testkit::{TEST_RECRUIT, test_story};

/// The game's content, for the tests of its story (ADR-0118 (a)): they
/// play the recorded playthroughs of its chapters.
fn content() -> Content {
    trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"))
}

fn lead() -> LeadProfile {
    LeadProfile::new("Lead", LeadGender::Male)
}

fn arrive(content: &Content, battle: &str, mode: GameMode) -> Option<BattleSetup> {
    let def = &content.battles[battle];
    arrival(content, def, mode, lead()).unwrap_or_else(|e| panic!("{e}"))
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
    for mode in [GameMode::Classic, GameMode::Casual] {
        let setup = arrive(&content, "ch02", mode).unwrap_or_else(|| panic!("no army"));
        let def = &content.battles["ch02"];
        let fresh =
            battle_campaign(&content, def, mode, lead()).battle_setup(def, &content.tables());
        let fresh = players(&fresh);
        let army = players(&setup);
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
        // Chapter 1's playthrough drinks its three Potions; its 1000 gold
        // buys new ones at Chapter 2's shop.
        let potion = ItemId::new("potion");
        let price = content.items.consumable(&potion).map_or(0, |c| c.price);
        let bought = (1000 / price).min(u32::try_from(def.pack_cap).unwrap_or(0));
        let potions = vec![potion; bought as usize];
        assert_eq!(setup.pack.items, potions, "a pack of Potions");
        assert_ne!(potions.len(), 0);
        assert_eq!(setup.gold, 1000 - bought * price);
    }
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
    let bridge = arrive(&content, "aldwater", GameMode::Classic);
    let hall = arrive(&content, "ch04", GameMode::Classic);
    let bridge = bridge.unwrap_or_else(|| panic!("no army"));
    let hall = hall.unwrap_or_else(|| panic!("no army"));
    let levels = |s: &BattleSetup| players(s).iter().map(|(_, u)| u.level).sum::<u32>();
    assert!(levels(&hall) > levels(&bridge), "EXP from the bridge");
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
    let setup = arrive(&content, "ch04", GameMode::Classic);
    let setup = setup.unwrap_or_else(|| panic!("Chapter 4 has an arrival"));
    let defector = CharacterId("defector".into());
    let army = players(&setup);
    assert!(army.iter().any(|(who, _)| *who == defector), "defector");
    assert_eq!(army.len(), 7, "the New Game's six and the recruit");
}
