//! Tests of the world map's rules on a small map:
//!
//! ```text
//! camp ── town ── ch1 ── ch2 (final)
//!                  │
//!                quest (opens when ch1 is cleared and `ann` is alive)
//! ```

use proptest::prelude::*;

use super::*;
use crate::battle::GameMode;
use crate::battle::tests::{items_for, lord, p, unit};
use crate::shop::{ShopError, ShopKind};
use crate::item::Stock;
use crate::lead::{LeadGender, LeadProfile};
use crate::unit::{Faction, Unit};

fn node(id: &str, kind: NodeKind, opens_when: Vec<NodeCondition>) -> WorldNode {
    WorldNode {
        id: id.into(),
        level: kind.is_battle().then_some(5),
        kind,
        opens_when,
    }
}

fn story(chapter: &str) -> NodeKind {
    NodeKind::StoryBattle(chapter.into())
}

fn cleared(id: &str) -> NodeCondition {
    NodeCondition::Cleared(id.into())
}

fn town() -> NodeKind {
    NodeKind::Town {
        shops: Vec::new(),
        capital: false,
    }
}

fn road(a: &str, b: &str) -> (String, String) {
    (a.into(), b.into())
}

fn map() -> WorldMap {
    WorldMap {
        id: "act1".into(),
        act: 1,
        levels: (3, 9),
        start: "camp".into(),
        final_node: "ch2".into(),
        next: Some("act2".into()),
        nodes: vec![
            node("camp", NodeKind::Empty, vec![]),
            node("town", town(), vec![]),
            node("ch1", story("ch01"), vec![]),
            node("ch2", story("ch02"), vec![]),
            node(
                "quest",
                NodeKind::SideQuest(SideQuest {
                    battle: "quest".into(),
                    ..SideQuest::default()
                }),
                vec![cleared("ch1"), NodeCondition::Alive(CharacterId("ann".into()))],
            ),
        ],
        roads: vec![
            road("camp", "town"),
            road("town", "ch1"),
            road("ch1", "ch2"),
            road("ch1", "quest"),
        ],
    }
}

fn act2() -> WorldMap {
    WorldMap {
        id: "act2".into(),
        act: 2,
        levels: (10, 15),
        start: "port".into(),
        final_node: "ch3".into(),
        next: None,
        nodes: vec![
            node("port", NodeKind::Empty, vec![]),
            node("ch3", story("ch03"), vec![]),
        ],
        roads: vec![road("port", "ch3")],
    }
}

fn named(u: Unit, id: &str) -> Unit {
    Unit {
        character: Some(CharacterId(id.into())),
        name: id.into(),
        ..u
    }
}

/// A campaign at Chapter 1, on [`map`], with the lord and `ann`.
fn campaign() -> Campaign {
    let roster = vec![
        named(lord(1, p(0, 0)), "lead"),
        named(unit(2, Faction::Player, p(0, 1)), "ann"),
    ];
    let mut c = Campaign::new_game(
        GameMode::Classic,
        LeadProfile::new("Mara", LeadGender::Female),
        "ch01",
        roster,
        0,
        Stock::default(),
    );
    c.enter_world(&map());
    c
}

fn travel(c: &mut Campaign, to: &str) -> Result<Vec<WorldEvent>, WorldError> {
    c.world_command(&map(), &WorldCommand::TravelTo(to.into()))
}

fn at(c: &Campaign) -> &str {
    &c.world.as_ref().expect("on the map").at
}

fn open(c: &Campaign, m: &WorldMap) -> Vec<String> {
    m.open_nodes(c).map(|n| n.id.clone()).collect()
}

fn route(ids: &[&str]) -> WorldEvent {
    WorldEvent::Travelled(ids.iter().map(|s| (*s).to_owned()).collect())
}

#[test]
fn the_army_starts_at_the_start_with_only_the_open_places_shown() {
    let c = campaign();
    assert_eq!(at(&c), "camp");
    // ch2 waits for its chapter; the quest for ch1.
    assert_eq!(open(&c, &map()), ["camp", "town", "ch1"]);
    let roads: Vec<_> = map().open_roads(&c).cloned().collect();
    assert_eq!(roads, [road("camp", "town"), road("town", "ch1")]);
}

#[test]
fn entering_the_same_map_again_keeps_where_the_army_is() {
    let mut c = campaign();
    travel(&mut c, "town").expect("open");
    c.enter_world(&map());
    assert_eq!(at(&c), "town");
}

#[test]
fn travel_follows_open_roads_to_an_open_place() {
    let mut c = campaign();
    assert_eq!(travel(&mut c, "town"), Ok(vec![route(&["camp", "town"])]));
    assert_eq!(at(&c), "town");
}

#[test]
fn locked_unknown_and_same_places_are_refused_and_change_nothing() {
    let mut c = campaign();
    let before = c.clone();
    assert_eq!(travel(&mut c, "ch2"), Err(WorldError::Locked));
    assert_eq!(travel(&mut c, "quest"), Err(WorldError::Locked));
    assert_eq!(travel(&mut c, "nowhere"), Err(WorldError::NoSuchNode));
    assert_eq!(travel(&mut c, "camp"), Err(WorldError::AlreadyThere));
    assert_eq!(c, before);
}

#[test]
fn an_open_place_with_no_open_road_is_refused() {
    let mut m = map();
    m.nodes.push(node("island", NodeKind::Empty, vec![]));
    let mut c = campaign();
    let travel = WorldCommand::TravelTo("island".into());
    assert_eq!(c.world_command(&m, &travel), Err(WorldError::NoRoad));
    assert_eq!(at(&c), "camp");
}

#[test]
fn a_campaign_off_the_map_is_refused() {
    let mut c = campaign();
    c.world = None;
    assert_eq!(travel(&mut c, "town"), Err(WorldError::NotOnMap));
    let mut c = campaign();
    let travel = WorldCommand::TravelTo("ch3".into());
    assert_eq!(c.world_command(&act2(), &travel), Err(WorldError::NotOnMap));
}

#[test]
fn arriving_at_a_battle_not_yet_won_starts_it() {
    let mut c = campaign();
    let events = travel(&mut c, "ch1").expect("open");
    assert_eq!(
        events,
        [
            route(&["camp", "town", "ch1"]),
            WorldEvent::BattleAhead("ch1".into())
        ]
    );
    assert_eq!(at(&c), "ch1");
}

#[test]
fn the_army_stops_at_an_unwon_battle_on_the_way() {
    let mut m = map();
    m.nodes.push(node("beyond", NodeKind::Empty, vec![]));
    m.roads.push(road("ch1", "beyond"));
    let mut c = campaign();
    let events = c
        .world_command(&m, &WorldCommand::TravelTo("beyond".into()))
        .expect("open");
    assert_eq!(
        events,
        [
            route(&["camp", "town", "ch1"]),
            WorldEvent::BattleAhead("ch1".into())
        ]
    );
}

#[test]
fn the_army_goes_round_an_unwon_battle_when_it_can() {
    let mut m = map();
    m.nodes.push(node("beyond", NodeKind::Empty, vec![]));
    m.nodes.push(node("bypass", NodeKind::Empty, vec![]));
    m.roads.extend([
        road("ch1", "beyond"),
        road("town", "bypass"),
        road("bypass", "beyond"),
    ]);
    let mut c = campaign();
    let events = c
        .world_command(&m, &WorldCommand::TravelTo("beyond".into()))
        .expect("open");
    assert_eq!(events, [route(&["camp", "town", "bypass", "beyond"])]);
}

#[test]
fn a_won_battle_is_passed_through() {
    let mut c = campaign();
    c.clear_node(&map(), "ch1", Some("ch02"), None);
    travel(&mut c, "quest").expect("open");
    assert_eq!(travel(&mut c, "camp"), Ok(vec![route(&["quest", "ch1", "town", "camp"])]));
}

#[test]
fn entering_a_town_opens_it_and_an_empty_place_has_nothing() {
    let mut c = campaign();
    let enter = WorldCommand::Enter;
    assert_eq!(c.world_command(&map(), &enter), Err(WorldError::NothingHere));
    travel(&mut c, "town").expect("open");
    assert_eq!(
        c.world_command(&map(), &enter),
        Ok(vec![WorldEvent::Entered("town".into())])
    );
}

#[test]
fn entering_an_unwon_battle_starts_it_and_a_won_one_has_nothing() {
    let mut c = campaign();
    travel(&mut c, "ch1").expect("open");
    let enter = WorldCommand::Enter;
    assert_eq!(
        c.world_command(&map(), &enter),
        Ok(vec![WorldEvent::BattleAhead("ch1".into())])
    );
    c.clear_node(&map(), "ch1", Some("ch02"), None);
    assert_eq!(c.world_command(&map(), &enter), Err(WorldError::NothingHere));
}

#[test]
fn winning_a_story_battle_clears_it_and_opens_what_waited_for_it() {
    let mut c = campaign();
    let events = c.clear_node(&map(), "ch1", Some("ch02"), None);
    assert_eq!(
        events,
        [
            WorldEvent::Cleared("ch1".into()),
            WorldEvent::Opened("ch2".into()),
            WorldEvent::Opened("quest".into()),
        ]
    );
    assert_eq!(c.chapter, "ch02");
    assert!(map().is_cleared("ch1", &c));
    assert_eq!(open(&c, &map()), ["camp", "town", "ch1", "ch2", "quest"]);
    let roads = map().open_roads(&c).count();
    assert_eq!(roads, 4);
}

#[test]
fn a_side_quest_whose_character_is_gone_stays_closed() {
    let mut c = campaign();
    c.roster.retain(|u| u.name != "ann");
    let events = c.clear_node(&map(), "ch1", Some("ch02"), None);
    assert!(!events.contains(&WorldEvent::Opened("quest".into())));
    assert!(!map().is_open("quest", &c));
}

#[test]
fn a_cleared_side_quest_stays_open_once_its_character_is_gone() {
    let mut c = campaign();
    c.clear_node(&map(), "ch1", Some("ch02"), None);
    c.clear_node(&map(), "quest", None, None);
    c.roster.retain(|u| u.name != "ann");
    assert!(map().is_open("quest", &c));
}

#[test]
fn a_flag_opens_a_place() {
    let mut m = map();
    m.nodes.push(node(
        "shrine",
        NodeKind::Empty,
        vec![NodeCondition::Flag("rumour".into())],
    ));
    let mut c = campaign();
    assert!(!m.is_open("shrine", &c));
    c.flags.insert("rumour".into(), true);
    assert!(m.is_open("shrine", &c));
}

#[test]
fn winning_the_final_story_battle_moves_to_the_next_acts_map() {
    let mut c = campaign();
    c.clear_node(&map(), "ch1", Some("ch02"), None);
    let events = c.clear_node(&map(), "ch2", Some("ch03"), Some(&act2()));
    assert_eq!(
        events,
        [
            WorldEvent::Cleared("ch2".into()),
            WorldEvent::NewAct("act2".into())
        ]
    );
    let world = c.world.as_ref().expect("on the next map");
    assert_eq!((world.map.as_str(), world.at.as_str()), ("act2", "port"));
    assert!(world.cleared.is_empty());
    // The old map is gone.
    assert_eq!(travel(&mut c, "town"), Err(WorldError::NotOnMap));
    assert_eq!(open(&c, &act2()), ["port", "ch3"]);
}

#[test]
fn the_last_acts_final_battle_only_clears() {
    let mut c = campaign();
    c.clear_node(&map(), "ch1", Some("ch02"), None);
    let events = c.clear_node(&map(), "ch2", None, None);
    assert_eq!(events, [WorldEvent::Cleared("ch2".into())]);
    assert_eq!(c.world.as_ref().map(|w| w.map.as_str()), Some("act1"));
}

#[test]
fn clearing_off_the_map_does_nothing() {
    let mut c = campaign();
    c.world = None;
    let before = c.clone();
    assert!(c.clear_node(&map(), "ch1", Some("ch02"), None).is_empty());
    assert_eq!(c, before);
}

#[test]
fn the_world_state_round_trips_through_serde_and_is_left_out_while_none() {
    let mut c = campaign();
    c.clear_node(&map(), "ch1", Some("ch02"), None);
    let text = ron::to_string(&c).expect("serializes");
    assert_eq!(ron::from_str::<Campaign>(&text).expect("parses"), c);
    c.world = None;
    let text = ron::to_string(&c).expect("serializes");
    assert!(!text.contains("world"), "{text}");
}

fn smith() -> Shop {
    Shop {
        kind: ShopKind::Blacksmith,
        stock: vec![],
    }
}

#[test]
fn a_towns_blacksmith_repairs_a_weapon_the_army_holds() {
    let mut c = campaign();
    let items = items_for(&c.roster);
    let copy = c.roster[0].loadout.weapons[0].as_mut().expect("armed");
    let full = copy.durability_left;
    copy.durability_left = 1;
    let held = TownTrade::RepairHeld { unit: 0, slot: 0 };
    assert_eq!(c.trade(&smith(), &items, &held), Ok(0));
    let copy = c.roster[0].loadout.weapon(0).expect("armed");
    assert_eq!(copy.durability_left, full);
    // Nothing more to repair, and no such weapon: refused, nothing changes.
    let before = c.clone();
    assert!(c.trade(&smith(), &items, &held).is_err());
    let none = TownTrade::RepairHeld { unit: 0, slot: 3 };
    assert_eq!(c.trade(&smith(), &items, &none), Err(ShopError::NoItem));
    assert_eq!(c, before);
}

#[test]
fn a_towns_shop_buys_into_the_stock_and_sells_from_it() {
    let mut c = campaign();
    let items = items_for(&c.roster);
    let weapon = c.roster[0].loadout.weapon(0).expect("armed").def.clone();
    let armoury = Shop {
        kind: ShopKind::Armoury,
        stock: vec![weapon.clone()],
    };
    assert_eq!(c.trade(&armoury, &items, &TownTrade::Buy(weapon.clone())), Ok(0));
    assert_eq!(c.stock.weapons.len(), 1);
    let sell = TownTrade::Sell(StockItem::Weapon(0));
    assert_eq!(c.trade(&armoury, &items, &sell), Ok(0));
    assert!(c.stock.weapons.is_empty());
    // A Blacksmith neither sells nor buys.
    assert!(c.trade(&smith(), &items, &TownTrade::Buy(weapon)).is_err());
    assert!(c.trade(&smith(), &items, &TownTrade::RepairStock(0)).is_err());
}

/// Whether `to` can be reached from the map's start along open roads.
fn reachable(m: &WorldMap, c: &Campaign, to: &str) -> bool {
    m.search(&m.start, to, c, |_| true).is_some()
}

proptest! {
    /// After any sequence of travels (and of battles won on the way), the
    /// army is always on an open place it can reach.
    #[test]
    fn the_army_is_always_on_an_open_reachable_place(
        steps in prop::collection::vec((0usize..5, any::<bool>()), 0..30),
    ) {
        let m = map();
        let mut c = campaign();
        for (pick, win) in steps {
            let to = m.nodes[pick].id.clone();
            let events = c.world_command(&m, &WorldCommand::TravelTo(to)).unwrap_or_default();
            if win && let Some(WorldEvent::BattleAhead(here)) = events.last() {
                let next = if here == "ch1" { Some("ch02") } else { None };
                c.clear_node(&m, here, next, None);
            }
            let here = at(&c).to_owned();
            prop_assert!(m.is_open(&here, &c), "{here} isn't open");
            prop_assert!(reachable(&m, &c, &here), "{here} can't be reached");
        }
    }
}
