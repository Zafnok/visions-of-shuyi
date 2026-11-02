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
use crate::item::Stock;
use crate::lead::{LeadGender, LeadProfile};
use crate::shop::{ShopError, ShopKind};
use crate::unit::{Faction, Unit};

fn node(id: &str, kind: NodeKind, opens_when: Vec<NodeCondition>) -> WorldNode {
    WorldNode {
        id: id.into(),
        level: kind.is_battle().then_some(5),
        kind,
        opens_when,
        hidden: false,
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
        skirmishes: None,
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
                vec![
                    cleared("ch1"),
                    NodeCondition::Alive(CharacterId("ann".into())),
                ],
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
        skirmishes: None,
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

/// Places one road from an open one show what they are, not open; a
/// hidden one stays out of sight until it opens (Nick, 1007).
#[test]
fn places_one_road_from_an_open_one_show_unless_hidden() {
    let c = campaign();
    let shown: Vec<String> = map().shown_nodes(&c).map(|n| n.id.clone()).collect();
    assert_eq!(shown, ["camp", "town", "ch1", "ch2", "quest"]);
    assert!(!map().is_open("ch2", &c));
    let locked: Vec<_> = map().locked_roads(&c).cloned().collect();
    assert_eq!(locked, [road("ch1", "ch2"), road("ch1", "quest")]);
    let mut m = map();
    m.nodes[4].hidden = true;
    assert!(!m.is_shown("quest", &c));
    assert_eq!(m.locked_roads(&c).count(), 1);
    // Two roads away: not shown.
    m.nodes
        .push(node("far", NodeKind::Empty, vec![cleared("ch2")]));
    m.roads.push(road("ch2", "far"));
    assert!(!m.is_shown("far", &c));
    // Once open, a hidden place shows.
    let mut c = c;
    m.nodes[4].hidden = true;
    c.clear_node(&m, "ch1", Some("ch02"), None);
    assert!(m.is_shown("quest", &c));
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
    assert_eq!(
        travel(&mut c, "camp"),
        Ok(vec![route(&["quest", "ch1", "town", "camp"])])
    );
}

#[test]
fn entering_a_town_opens_it_and_an_empty_place_has_nothing() {
    let mut c = campaign();
    let enter = WorldCommand::Enter;
    assert_eq!(
        c.world_command(&map(), &enter),
        Err(WorldError::NothingHere)
    );
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
    assert_eq!(
        c.world_command(&map(), &enter),
        Err(WorldError::NothingHere)
    );
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
    assert_eq!(c.clear_node(&map(), "ch1", Some("ch02"), None), []);
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
    assert_eq!(
        c.trade(&armoury, &items, &TownTrade::Buy(weapon.clone())),
        Ok(0)
    );
    assert_eq!(c.stock.weapons.len(), 1);
    let sell = TownTrade::Sell(StockItem::Weapon(0));
    assert_eq!(c.trade(&armoury, &items, &sell), Ok(0));
    assert_eq!(c.stock.weapons.len(), 0);
    // A Blacksmith neither sells nor buys.
    assert!(c.trade(&smith(), &items, &TownTrade::Buy(weapon)).is_err());
    assert!(
        c.trade(&smith(), &items, &TownTrade::RepairStock(0))
            .is_err()
    );
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

fn skirmish_map() -> WorldMap {
    let mut m = map();
    m.levels = (1, 50);
    m.skirmishes = Some(SkirmishRules {
        spawn: (2, 2),
        maximum: 3,
        level_offset: 1,
        size: (4, 6),
        pool: vec!["meadow".into(), "wood".into()],
        rewards: vec![
            None,
            Some(crate::Loot::Gold(100)),
            Some(crate::Loot::Item(ItemId("potion".into()))),
        ],
    });
    for i in 0..12 {
        let mut n = node(&format!("fight{i}"), story("ch01"), vec![]);
        n.level = Some(3 + i);
        m.nodes.push(n);
    }
    m
}

#[test]
fn only_guarding_fixed_skirmishes_stop_through_travel() {
    for guard in [false, true] {
        let mut m = map();
        m.nodes[1].kind = NodeKind::FixedSkirmish {
            battle: SideQuest::default(),
            guards_path: guard,
        };
        let mut c = campaign();
        let route = m.route("camp", "ch1", &c).expect("route");
        assert_eq!(
            route.last().map(String::as_str),
            Some(if guard { "town" } else { "ch1" })
        );
        assert!(
            m.is_unwon_battle("town", &c),
            "arriving still starts either fixed fight"
        );
        c.clear_node(&m, "town", None, None);
        assert!(!m.blocks_path("town", &c));
        assert_eq!(
            m.route("camp", "ch1", &c)
                .expect("route")
                .last()
                .map(String::as_str),
            Some("ch1")
        );
        assert_eq!(c.clear_node(&m, "town", None, None), []);
    }
}

#[test]
fn groups_spawn_only_on_other_cleared_battles_and_wins_remove_them_once() {
    let m = skirmish_map();
    let mut c = campaign();
    let events = c.clear_node(&m, "fight0", None, None);
    assert!(events.iter().any(|e| matches!(e, WorldEvent::SkirmishSpawned { node, group } if node == "fight0" && group.level == 2)));
    let before = c.clone();
    assert_eq!(c.clear_node(&m, "absent", None, None), []);
    assert_eq!(c.clear_node(&m, "camp", None, None), []);
    assert_eq!(c, before);
    // Arriving at a group starts it, but passing it en route doesn't.
    assert!(m.is_unwon_battle("fight0", &c));
    assert!(!m.blocks_path("fight0", &c));
    let chapter = c.chapter.clone();
    let rng = c.rng.clone();
    assert_eq!(
        c.clear_node(&m, "fight0", None, None),
        [WorldEvent::Cleared("fight0".into())]
    );
    assert!(m.is_cleared("fight0", &c));
    assert!(!m.is_unwon_battle("fight0", &c));
    assert_eq!(c.chapter, chapter);
    assert_eq!(c.rng, rng, "a training victory never rolls more groups");
    assert_eq!(c.clear_node(&m, "fight0", None, None), []);
}

#[test]
fn an_occupied_place_or_a_cap_below_the_acts_minimum_spawns_nothing() {
    let mut m = skirmish_map();
    let mut c = campaign();
    c.world.as_mut().expect("world").at = "fight0".into();
    c.clear_node(&m, "fight0", None, None);
    assert!(c.world.as_ref().expect("world").skirmishes.is_empty());
    m.nodes.last_mut().expect("story").level = Some(1);
    c.clear_node(&m, "fight11", None, None);
    assert!(c.world.as_ref().expect("world").skirmishes.is_empty());
}

#[test]
fn saved_rng_and_groups_resume_the_same_sequence_and_rewards_cover_the_data() {
    let m = skirmish_map();
    let mut seen = [false; 3];
    let mut pools = BTreeSet::new();
    let mut sizes = BTreeSet::new();
    for seed in 0..64 {
        let mut c = campaign();
        c.rng = Some(crate::SimRng::new(seed));
        c.clear_node(&m, "fight0", None, None);
        let save = ron::to_string(&c).expect("save");
        let mut loaded: Campaign = ron::from_str(&save).expect("load");
        assert_eq!(
            c.clear_node(&m, "fight1", None, None),
            loaded.clear_node(&m, "fight1", None, None)
        );
        assert_eq!(c, loaded);
        for group in c.world.as_ref().expect("world").skirmishes.values() {
            pools.insert(group.battle.clone());
            sizes.insert(group.size);
            seen[match group.reward {
                None => 0,
                Some(crate::Loot::Gold(100)) => 1,
                Some(crate::Loot::Item(_)) => 2,
                _ => panic!("unexpected reward"),
            }] = true;
        }
    }
    assert_eq!(seen, [true; 3]);
    assert_eq!(pools.len(), 2);
    assert_eq!(sizes, BTreeSet::from([4, 5, 6]));
}

proptest! {
    #[test]
    fn story_victory_sequences_respect_cap_capacity_and_occupied_place(
        seed in any::<u64>(), clear in prop::collection::vec((any::<bool>(), 1u32..70), 1..12),
    ) {
        let mut m = skirmish_map();
        let mut c = campaign();
        c.rng = Some(crate::SimRng::new(seed));
        for (i, (win_training, level)) in clear.iter().enumerate() {
            let node = format!("fight{i}");
            m.nodes.iter_mut().find(|n| n.id == node).expect("story").level = Some(*level);
            c.world.as_mut().expect("world").at = node.clone();
            c.clear_node(&m, &node, None, None);
            let w = c.world.as_ref().expect("world");
            prop_assert!(w.skirmishes.len() <= 3);
            prop_assert!(!w.skirmishes.contains_key(&w.at));
            for (place, g) in &w.skirmishes {
                prop_assert!(w.cleared.contains(place));
                prop_assert!(g.level >= 1 && g.level <= level.saturating_sub(1).min(50));
                prop_assert!((4..=6).contains(&g.size));
            }
            if *win_training && let Some(place) = w.skirmishes.keys().next().cloned() {
                c.clear_node(&m, &place, None, None);
            }
        }
    }
}

#[test]
fn lower_story_caps_also_limit_existing_groups_and_zero_rolls_are_possible() {
    let mut m = skirmish_map();
    let mut c = campaign();
    c.clear_node(&m, "fight0", None, None);
    m.nodes
        .iter_mut()
        .find(|n| n.id == "fight1")
        .expect("story")
        .level = Some(2);
    c.world.as_mut().expect("world").at = "fight1".into();
    assert!(
        c.clear_node(&m, "fight1", None, None)
            .contains(&WorldEvent::SkirmishesCapped(1))
    );
    assert_eq!(m.random_at("fight0", &c).expect("group").level, 1);
    m.nodes
        .iter_mut()
        .find(|n| n.id == "fight2")
        .expect("story")
        .level = Some(1);
    assert!(
        c.clear_node(&m, "fight2", None, None)
            .contains(&WorldEvent::SkirmishesCapped(0))
    );
    assert_eq!(c.world.as_ref().expect("world").skirmishes.len(), 0);
    m.skirmishes.as_mut().expect("rules").spawn = (0, 2);
    let mut counts = BTreeSet::new();
    for seed in 0..64 {
        let mut c = campaign();
        c.rng = Some(crate::SimRng::new(seed));
        c.world
            .as_mut()
            .expect("world")
            .cleared
            .extend((0..8).map(|i| format!("fight{i}")));
        c.clear_node(&m, "fight8", None, None);
        counts.insert(c.world.as_ref().expect("world").skirmishes.len());
    }
    assert_eq!(counts, BTreeSet::from([0, 1, 2]));
}

#[test]
fn a_story_at_the_same_cap_keeps_existing_groups_without_a_cap_change_event() {
    let mut m = skirmish_map();
    let mut c = campaign();
    c.clear_node(&m, "fight0", None, None);
    let group = m.random_at("fight0", &c).expect("spawned").clone();
    assert_eq!(group.level, 2);
    m.skirmishes.as_mut().expect("rules").spawn = (0, 0);
    m.nodes
        .iter_mut()
        .find(|n| n.id == "fight1")
        .expect("story")
        .level = Some(3);
    c.world.as_mut().expect("world").at = "fight1".into();
    assert_eq!(
        c.clear_node(&m, "fight1", None, None),
        [WorldEvent::Cleared("fight1".into())]
    );
    assert_eq!(m.random_at("fight0", &c), Some(&group));
}

#[test]
fn a_place_for_some_chapters_opens_only_in_them_and_stays_open_once_cleared() {
    let mut map = map();
    map.nodes.push(node(
        "fight",
        NodeKind::FixedSkirmish {
            battle: SideQuest::default(),
            guards_path: false,
        },
        vec![NodeCondition::Chapter(vec!["ch05".into()])],
    ));
    let mut campaign = campaign();
    campaign.enter_world(&map);
    campaign.chapter = "ch04".into();
    assert!(!map.is_open("fight", &campaign));
    campaign.chapter = "ch05".into();
    assert!(map.is_open("fight", &campaign));
    campaign.clear_node(&map, "fight", None, None);
    campaign.chapter = "ch06".into();
    assert!(map.is_open("fight", &campaign));
}
