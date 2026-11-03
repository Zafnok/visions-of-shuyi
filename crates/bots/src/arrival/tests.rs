use trpg_content::battle_campaign;
use trpg_core::{CharacterId, Faction, LeadGender, Unit};

use super::*;

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
fn nothing_comes_before_chapter_1() {
    let content = content();
    assert_eq!(arrive(&content, "ch01", GameMode::Classic), None);
    assert_eq!(chapters_before(&content, "ch01").len(), 0);
}

#[test]
fn chapter_2_is_played_with_the_army_chapter_1_leaves() {
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

#[test]
fn the_army_is_the_same_every_time() {
    let content = content();
    let first = arrive(&content, "ch02", GameMode::Classic);
    assert_eq!(first, arrive(&content, "ch02", GameMode::Classic));
}

#[test]
fn a_battle_no_chapter_fights_has_no_arrival() {
    let content = content();
    let quick = content
        .battles
        .keys()
        .find(|id| chapters_before(&content, id).is_empty() && !id.starts_with("ch0"));
    let quick = quick.unwrap_or_else(|| panic!("no battle outside the story"));
    assert_eq!(arrive(&content, quick, GameMode::Classic), None);
}

#[test]
fn the_pack_takes_the_stock_first_then_what_the_gold_buys() {
    let content = content();
    let def = &content.battles["ch02"];
    let mut setup = battle_campaign(&content, def, GameMode::Classic, lead())
        .battle_setup(def, &content.tables());
    setup.stock = trpg_core::Stock::default();
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
