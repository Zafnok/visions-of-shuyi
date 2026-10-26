//! Tests of the village rule (ticket 0313, `turn-structure.md`, *Save the
//! village*): `Evacuate`, raiders burning houses, the loss and the rewards.
//! The houses are at (2,0), (5,0) and (5,4) of the open map. Test units
//! deal their weapon's might with every strike and each side strikes once
//! (see the parent module).

use super::*;

/// The houses of these tests.
const HOUSES: [Pos; 3] = [p(2, 0), p(5, 0), p(5, 4)];

const fn p(x: i32, y: i32) -> Pos {
    Pos { x, y }
}

/// A village of [`HOUSES`] with this minimum and these rewards.
fn goal(minimum: u32, rewards: Vec<Loot>) -> VillageGoal {
    VillageGoal {
        houses: HOUSES.to_vec(),
        minimum,
        rewards,
        burnt: BURNT,
    }
}

/// A rout of `units` with the village `goal`.
fn village(units: Vec<Unit>, goal: VillageGoal) -> BattleState {
    start(BattleSetup {
        village: Some(goal),
        ..setup(units)
    })
}

fn raider(u: Unit) -> Unit {
    Unit { raider: true, ..u }
}

/// Lord 1 at (0,0) and unit 2 at (0,2); raider 3 at (7,0) and enemy 4 at
/// (7,4). Save 1 of 3, no rewards needed for the tests that don't win.
fn villagers() -> Vec<Unit> {
    vec![
        lord(1, p(0, 0)),
        unit(2, Faction::Player, p(0, 2)),
        raider(unit(3, Faction::Enemy, p(7, 0))),
        unit(4, Faction::Enemy, p(7, 4)),
    ]
}

/// Ends the player phase: the enemy phase starts.
fn to_enemy_phase(s: &mut BattleState) {
    end(s);
    assert_eq!(s.phase(), Phase::Enemy);
}

/// The events of unit `unit` burning the house at `pos`, a plain tile of
/// the open map: it burns, then its tile becomes burnt terrain.
fn burned(pos: Pos, unit: u32) -> [Event; 2] {
    [
        Event::HouseBurned {
            pos,
            unit: UnitId(unit),
        },
        Event::TerrainChanged {
            pos,
            from: TerrainId(0),
            to: BURNT,
        },
    ]
}

fn acted(unit: u32) -> Event {
    Event::UnitActed { unit: UnitId(unit) }
}

// ---- Evacuate --------------------------------------------------------------

#[test]
fn evacuate_saves_a_house_for_good() {
    let mut s = village(villagers(), goal(1, vec![]));
    assert!(s.can_evacuate(UnitId(1), p(2, 0)));
    assert!(!s.can_evacuate(UnitId(1), p(1, 0)));
    assert!(
        !s.can_evacuate(UnitId(3), p(5, 0)),
        "enemies never evacuate"
    );
    let events = act(&mut s, 1, p(2, 0), UnitAction::Evacuate);
    assert_eq!(
        events,
        [
            Event::UnitMoved {
                unit: UnitId(1),
                path: vec![p(0, 0), p(1, 0), p(2, 0)],
            },
            Event::HouseSaved {
                pos: p(2, 0),
                unit: Some(UnitId(1)),
            },
            acted(1),
        ]
    );
    let v = s.village().unwrap();
    assert_eq!(v.house(p(2, 0)), Some(HouseState::Saved));
    assert_eq!((v.total(), v.saved(), v.lost(), v.minimum()), (3, 1, 0, 1));
    assert_eq!(v.standing().collect::<Vec<_>>(), [p(5, 0), p(5, 4)]);
    assert_eq!(s.outcome(), None);
}

#[test]
fn evacuate_is_refused_off_a_standing_house_and_for_other_units() {
    let mut s = village(villagers(), goal(1, vec![]));
    refused_act(
        &mut s,
        1,
        p(1, 0),
        UnitAction::Evacuate,
        CommandError::NoHouse(p(1, 0)),
    );
    // Without a village goal there is no house anywhere.
    let mut plain = start(setup(villagers()));
    refused_act(
        &mut plain,
        1,
        p(2, 0),
        UnitAction::Evacuate,
        CommandError::NoHouse(p(2, 0)),
    );
    act(&mut s, 1, p(2, 0), UnitAction::Evacuate);
    to_enemy_phase(&mut s);
    refused_act(
        &mut s,
        3,
        p(5, 0),
        UnitAction::Evacuate,
        CommandError::PlayerOnly(UnitId(3)),
    );
    end(&mut s);
    assert_eq!(s.phase(), Phase::Player);
    refused_act(
        &mut s,
        1,
        p(2, 0),
        UnitAction::Evacuate,
        CommandError::HouseAlreadySaved(p(2, 0)),
    );
}

#[test]
fn a_burned_house_cant_be_evacuated() {
    let mut units = villagers();
    units[0].pos = p(4, 1);
    let mut s = village(units, goal(1, vec![]));
    act(&mut s, 1, p(4, 1), UnitAction::Wait);
    to_enemy_phase(&mut s);
    act(&mut s, 3, p(6, 0), UnitAction::Wait);
    end(&mut s);
    refused_act(
        &mut s,
        1,
        p(5, 0),
        UnitAction::Evacuate,
        CommandError::HouseLost(p(5, 0)),
    );
}

// ---- Burning ---------------------------------------------------------------

#[test]
fn a_raider_burns_the_house_it_ends_beside() {
    let mut s = village(villagers(), goal(1, vec![]));
    to_enemy_phase(&mut s);
    let events = act(&mut s, 3, p(6, 0), UnitAction::Wait);
    assert_eq!(
        events,
        [
            Event::UnitMoved {
                unit: UnitId(3),
                path: vec![p(7, 0), p(6, 0)],
            },
            Event::HouseBurned {
                pos: p(5, 0),
                unit: UnitId(3),
            },
            Event::TerrainChanged {
                pos: p(5, 0),
                from: TerrainId(0),
                to: BURNT,
            },
            acted(3),
        ]
    );
    let v = s.village().unwrap();
    assert_eq!(v.house(p(5, 0)), Some(HouseState::Burned));
    // A burned house gives no bonus: its tile is burnt terrain now (Nick).
    assert_eq!(s.map().tiles.get(p(5, 0)), Some(&BURNT));
    assert_eq!((v.saved(), v.lost()), (0, 1));
}

#[test]
fn a_raider_burns_the_house_it_stands_on() {
    let mut s = village(villagers(), goal(1, vec![]));
    to_enemy_phase(&mut s);
    let events = act(&mut s, 3, p(5, 0), UnitAction::Wait);
    assert_eq!(events[1..3], burned(p(5, 0), 3));
    assert_eq!(events[3..], [acted(3)]);
}

#[test]
fn diagonals_and_other_enemies_burn_nothing() {
    let mut s = village(villagers(), goal(1, vec![]));
    to_enemy_phase(&mut s);
    // (6,1) is diagonal to the house at (5,0).
    let events = act(&mut s, 3, p(6, 1), UnitAction::Wait);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::HouseBurned { .. }))
    );
    // Enemy 4 isn't a raider: beside the house at (5,4), nothing burns.
    let events = act(&mut s, 4, p(6, 4), UnitAction::Wait);
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::HouseBurned { .. }))
    );
    assert_eq!(s.village().unwrap().lost(), 0);
}

#[test]
fn a_saved_house_never_burns() {
    let units = vec![lord(1, p(0, 0)), raider(unit(3, Faction::Enemy, p(4, 1)))];
    let mut s = village(units, goal(1, vec![]));
    act(&mut s, 1, p(2, 0), UnitAction::Evacuate);
    to_enemy_phase(&mut s);
    // (3,0) is beside the saved house at (2,0) and two tiles from (5,0).
    let events = act(&mut s, 3, p(3, 0), UnitAction::Wait);
    assert_eq!(events.last(), Some(&acted(3)));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::HouseBurned { .. }))
    );
    assert_eq!(s.village().unwrap().house(p(2, 0)), Some(HouseState::Saved));
}

#[test]
fn a_raider_burns_after_its_attack_unless_it_fell() {
    let units = vec![
        lord(1, p(0, 0)),
        unit(2, Faction::Player, p(6, 1)),
        raider(unit(3, Faction::Enemy, p(7, 0))),
    ];
    let mut s = village(units.clone(), goal(1, vec![]));
    to_enemy_phase(&mut s);
    let events = act(&mut s, 3, p(6, 0), attack(2));
    // After the combat and its weapon EXP.
    let n = events.len();
    assert_eq!(events[n - 3..n - 1], burned(p(5, 0), 3));
    assert_eq!(events[n - 1], acted(3));
    assert!(matches!(events[n - 4], Event::WeaponExpGained { .. }));
    // A raider that falls to the counter burns nothing.
    let mut units = units;
    units[1] = armed(units[1].clone(), 10);
    let mut s = village(units, goal(1, vec![]));
    to_enemy_phase(&mut s);
    let events = act(&mut s, 3, p(6, 0), attack(2));
    assert!(events.contains(&Event::UnitFell { unit: UnitId(3) }));
    assert!(
        !events
            .iter()
            .any(|e| matches!(e, Event::HouseBurned { .. }))
    );
}

// ---- Loss and rewards ------------------------------------------------------

#[test]
fn the_battle_is_lost_when_the_minimum_cant_be_met() {
    // Save 2 of 3: one house may burn.
    let mut s = village(villagers(), goal(2, vec![]));
    to_enemy_phase(&mut s);
    act(&mut s, 3, p(6, 0), UnitAction::Wait);
    assert_eq!(s.outcome(), None);
    // Save 3 of 3: the first house lost loses the battle.
    let mut s = village(villagers(), goal(3, vec![]));
    to_enemy_phase(&mut s);
    let events = act(&mut s, 3, p(6, 0), UnitAction::Wait);
    assert_eq!(events[1..3], burned(p(5, 0), 3));
    assert_eq!(events[3..], [acted(3), ended(Outcome::Defeat)]);
    assert!(s.village().unwrap().is_failed());
    refused(&mut s, &Command::EndPhase, CommandError::BattleOver);
}

#[test]
fn a_win_saves_the_standing_houses_and_gives_the_rewards_past_the_minimum() {
    let units = vec![
        armed(lord(1, p(0, 0)), 10),
        unit(3, Faction::Enemy, p(1, 1)),
    ];
    let rewards = vec![Loot::Gold(100), Loot::Item(item("tonic"))];
    let mut s = village(units, goal(1, rewards));
    let events = act(&mut s, 1, p(1, 0), attack(3));
    let saved = events
        .iter()
        .position(|e| matches!(e, Event::HouseSaved { .. }))
        .unwrap();
    let house = |pos| Event::HouseSaved { pos, unit: None };
    assert_eq!(
        events[saved..],
        [
            house(p(2, 0)),
            house(p(5, 0)),
            house(p(5, 4)),
            Event::HouseReward {
                loot: Loot::Gold(100),
            },
            Event::GoldChanged { gold: 100 },
            Event::HouseReward {
                loot: Loot::Item(item("tonic")),
            },
            ended(Outcome::Victory),
        ]
    );
    assert_eq!(s.gold(), 100);
    assert_eq!(s.pack().items.last(), Some(&item("tonic")));
    assert_eq!(s.village().unwrap().saved(), 3);
}

#[test]
fn a_burned_house_gives_no_reward_and_equipment_goes_to_the_stock() {
    let units = vec![
        armed(lord(1, p(3, 2)), 10),
        raider(unit(3, Faction::Enemy, p(6, 1))),
    ];
    let rewards = vec![Loot::Item(item("iron")), Loot::Gold(100)];
    let mut s = village(units, goal(1, rewards));
    act(&mut s, 1, p(3, 2), UnitAction::Wait);
    to_enemy_phase(&mut s);
    act(&mut s, 3, p(5, 1), UnitAction::Wait);
    end(&mut s);
    let events = act(&mut s, 1, p(4, 1), attack(3));
    assert_eq!(events.last(), Some(&ended(Outcome::Victory)));
    let rewards: Vec<&Event> = events
        .iter()
        .filter(|e| matches!(e, Event::HouseReward { .. }))
        .collect();
    // Two stand, one past the minimum: the first reward only.
    assert_eq!(
        rewards,
        [&Event::HouseReward {
            loot: Loot::Item(item("iron")),
        }]
    );
    assert!(s.stock().weapons.iter().any(|w| w.def == item("iron")));
    assert_eq!(s.gold(), 0);
    let v = s.village().unwrap();
    assert_eq!((v.saved(), v.lost()), (2, 1));
}

#[test]
fn a_defeat_gives_nothing_and_leaves_the_houses_as_they_are() {
    let units = vec![
        lord(1, p(6, 1)),
        armed(raider(unit(3, Faction::Enemy, p(7, 0))), 10),
    ];
    let mut s = village(units, goal(1, vec![Loot::Gold(100), Loot::Gold(50)]));
    to_enemy_phase(&mut s);
    let events = act(&mut s, 3, p(6, 0), attack(1));
    assert_eq!(events.last(), Some(&ended(Outcome::Defeat)));
    assert!(!events.iter().any(|e| matches!(e, Event::HouseSaved { .. })));
    assert_eq!(s.gold(), 0);
}

// ---- Saving ----------------------------------------------------------------

#[test]
fn houses_survive_a_save() {
    let mut s = village(villagers(), goal(1, vec![]));
    act(&mut s, 1, p(2, 0), UnitAction::Evacuate);
    to_enemy_phase(&mut s);
    act(&mut s, 3, p(6, 0), UnitAction::Wait);
    let text = ron::to_string(&s).unwrap();
    let mut back: BattleState = ron::from_str(&text).unwrap();
    back.restore_tables(&crate::GameTables {
        terrain: Arc::new(terrain()),
        classes: Arc::new(classes()),
        items: Arc::new(items_for(&villagers())),
        spells: Arc::new(spells()),
        skills: Arc::new(skills()),
        arts: Arc::new(test_arts()),
        supports: Arc::default(),
    });
    assert_eq!(back, s);
    let v = back.village().unwrap();
    assert_eq!((v.saved(), v.lost()), (1, 1));
    // A battle without a village, and a unit that isn't a raider, write
    // neither.
    let plain = ron::to_string(&start(setup(cast()))).unwrap();
    assert!(!plain.contains("village"), "{plain}");
    assert!(!plain.contains("raider"), "{plain}");
}
