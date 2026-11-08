//! Arts that strike several units (ticket 0327, `combat-arts.md`, *Exact
//! rules*): Wide Swing's cleave, Comet Lance's line, Arrow Rain's burst and
//! Chain Strike's allies. Each is a test art of rank D here, added to
//! [`art_setup`]'s table with the design doc's costs: `wide_swing` (axe,
//! range 1), `comet_lance` (spear, a line of 3), `arrow_rain` (bow, range
//! 2–3) and `chain_strike` (gauntlet).

use super::art::{art_attack, art_setup, artist};
use super::*;
use crate::art::ArtArea;
use crate::battle::arts::area_tiles;

/// The four arts, rank D.
fn several_arts() -> Vec<ArtDef> {
    let art = |id: &str, kind, cost, effect| ArtDef {
        id: ArtId::new(id),
        name: id.into(),
        kind,
        rank: Some(WeaponRank::D),
        cost,
        effect,
    };
    let area = |area, min, max| ArtEffect {
        area: Some(area),
        min_range: Some(min),
        max_range: Some(max),
        ..ArtEffect::default()
    };
    vec![
        art(
            "wide_swing",
            WeaponKind::Axe,
            4,
            area(ArtArea::Cleave, 1, 1),
        ),
        art(
            "comet_lance",
            WeaponKind::Spear,
            8,
            area(ArtArea::Line(3), 1, 3),
        ),
        art("arrow_rain", WeaponKind::Bow, 8, area(ArtArea::Burst, 2, 3)),
        art(
            "chain_strike",
            WeaponKind::Gauntlet,
            6,
            ArtEffect {
                chain: true,
                ..ArtEffect::default()
            },
        ),
    ]
}

fn battle(units: Vec<Unit>) -> BattleState {
    let mut setup = art_setup(units);
    let mut arts = (*setup.arts).clone();
    for def in several_arts() {
        arts.arts.insert(def.id.clone(), def);
    }
    setup.arts = Arc::new(arts);
    // Classes that give unit EXP, for the awards.
    setup.classes = Arc::new(leveling((*setup.classes).clone()));
    start(setup)
}

/// `u` with `hp` HP and this Spd.
fn tuned(u: Unit, hp: StatValue, spd: StatValue) -> Unit {
    Unit {
        stats: Stats::from_growable([hp, 0, 0, 0, spd, 0, 0], 3),
        hp,
        ..u
    }
}

fn enemy(id: u32, pos: Pos) -> Unit {
    unit(id, Faction::Enemy, pos)
}

/// Every combat in `events`: `(attacker, defender, forecast, strikes made)`.
fn combats(events: &[Event]) -> Vec<(u32, u32, Forecast, usize)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::CombatResolved {
                attacker,
                defender,
                forecast,
                outcome,
            } => Some((attacker.0, defender.0, *forecast, outcome.strikes.len())),
            _ => None,
        })
        .collect()
}

/// Who struck whom in `events`, in order.
fn struck(events: &[Event]) -> Vec<(u32, u32)> {
    combats(events).iter().map(|&(a, d, ..)| (a, d)).collect()
}

fn hp(s: &BattleState, id: u32) -> StatValue {
    s.unit(UnitId(id)).map_or(0, |u| u.hp)
}

fn exp_awards(events: &[Event], unit: u32) -> usize {
    let mine = |e: &&Event| matches!(e, Event::ExpGained { unit: u, .. } if *u == UnitId(unit));
    events.iter().filter(mine).count()
}

fn weapon_exp(events: &[Event], unit: u32) -> Vec<u32> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::WeaponExpGained {
                unit: u, amount, ..
            } if *u == UnitId(unit) => Some(*amount),
            _ => None,
        })
        .collect()
}

#[test]
fn area_tiles_follow_the_design_doc() {
    let o = p(4, 2);
    // Wide Swing facing east: front, left (north), right (south).
    assert_eq!(
        area_tiles(ArtArea::Cleave, o, p(5, 2)),
        Some(vec![p(5, 2), p(4, 1), p(4, 3)])
    );
    // Facing west: front, left (south), right (north).
    assert_eq!(
        area_tiles(ArtArea::Cleave, o, p(3, 2)),
        Some(vec![p(3, 2), p(4, 3), p(4, 1)])
    );
    // Only next to the user, never on a diagonal.
    assert_eq!(area_tiles(ArtArea::Cleave, o, p(6, 2)), None);
    assert_eq!(area_tiles(ArtArea::Cleave, o, p(5, 3)), None);
    // Comet Lance: the 3 tiles from the user towards the target.
    assert_eq!(
        area_tiles(ArtArea::Line(3), o, p(4, 0)),
        Some(vec![p(4, 1), p(4, 0), p(4, -1)])
    );
    assert_eq!(area_tiles(ArtArea::Line(3), o, p(5, 3)), None);
    // Arrow Rain: the target, then clockwise from the north.
    assert_eq!(
        area_tiles(ArtArea::Burst, o, p(6, 3)),
        Some(vec![p(6, 3), p(6, 2), p(7, 3), p(6, 4), p(5, 3)])
    );
}

#[test]
fn wide_swing_strikes_front_left_and_right_never_behind() {
    // From (4, 2) facing east: front (5, 2), left (4, 1), right (4, 3);
    // (3, 2) is behind. Spd 10 would double: a cleave never does.
    let units = || {
        vec![
            tuned(artist(1, p(4, 2), "heavy_axe"), 10, 10),
            enemy(3, p(5, 2)),
            enemy(4, p(4, 1)),
            enemy(5, p(4, 3)),
            enemy(6, p(3, 2)),
        ]
    };
    let mut s = battle(units());
    let shown = s
        .preview_attack(UnitId(1), p(4, 2), &art_attack(3, "wide_swing"))
        .unwrap();
    assert_eq!(shown.forecast.defender, None);
    assert_eq!(shown.forecast.attacker.strikes, 1);
    let events = act(&mut s, 1, p(4, 2), art_attack(3, "wide_swing"));
    assert_eq!(struck(&events), [(1, 3), (1, 4), (1, 5)]);
    for (_, _, forecast, strikes) in combats(&events) {
        assert_eq!((forecast.defender, strikes), (None, 1));
    }
    // Might 3 is raised to the axe minimum, 5. Nothing struck back.
    assert_eq!([3, 4, 5, 6].map(|id| hp(&s, id)), [5, 5, 5, 10]);
    assert_eq!(hp(&s, 1), 10);
    // Facing north instead: front (4, 1), left (3, 2), right (5, 2).
    let mut north = battle(units());
    let events = act(&mut north, 1, p(4, 2), art_attack(4, "wide_swing"));
    assert_eq!(struck(&events), [(1, 4), (1, 6), (1, 3)]);
    assert_eq!(hp(&north, 5), 10);
}

#[test]
fn wide_swing_passes_over_allies_and_empty_tiles() {
    let mut s = battle(vec![
        artist(1, p(4, 2), "heavy_axe"),
        unit(2, Faction::Player, p(4, 1)),
        enemy(3, p(5, 2)),
        unit(4, Faction::Ally, p(3, 2)),
    ]);
    let events = act(&mut s, 1, p(4, 2), art_attack(3, "wide_swing"));
    assert_eq!(struck(&events), [(1, 3)]);
    assert_eq!((hp(&s, 2), hp(&s, 4)), (10, 10));
}

#[test]
fn comet_lance_skips_allies_and_strikes_nearest_first() {
    // A line east of (0, 0): an ally at (1, 0), enemies at (2, 0) and
    // (3, 0), and one past the line at (4, 0). Aimed at the farther one.
    let mut s = battle(vec![
        artist(1, p(0, 0), "pike"),
        unit(2, Faction::Player, p(1, 0)),
        enemy(3, p(2, 0)),
        enemy(4, p(3, 0)),
        enemy(5, p(4, 0)),
    ]);
    let events = act(&mut s, 1, p(0, 0), art_attack(4, "comet_lance"));
    assert_eq!(struck(&events), [(1, 3), (1, 4)]);
    assert_eq!([2, 3, 4, 5].map(|id| hp(&s, id)), [10, 7, 7, 10]);
    assert_eq!(hp(&s, 1), 10);
}

#[test]
fn comet_lance_only_goes_in_rows_or_columns() {
    let s = battle(vec![
        artist(1, p(0, 0), "pike"),
        enemy(3, p(1, 1)),
        enemy(4, p(0, 3)),
    ]);
    // A diagonal target is out of the art's reach.
    assert_eq!(
        s.preview_attack(UnitId(1), p(0, 0), &art_attack(3, "comet_lance")),
        Err(CommandError::OutOfRange {
            target: UnitId(3),
            distance: 2,
        })
    );
    // Down a column, 3 tiles.
    let shown = s
        .preview_attack(UnitId(1), p(0, 0), &art_attack(4, "comet_lance"))
        .unwrap();
    let rows: Vec<_> = shown.strikes.iter().map(|r| (r.at, r.hp)).collect();
    assert_eq!(rows, [(UnitId(4), (10, 7))]);
}

#[test]
fn arrow_rain_hits_the_target_and_its_neighbours_that_are_enemies() {
    // The target at (3, 0), 3 from the archer: (4, 0) and (2, 0) are
    // enemies, (3, 1) an ally; (3, 2) is two away.
    let mut s = battle(vec![
        artist(1, p(0, 0), "longbow"),
        unit(2, Faction::Player, p(3, 1)),
        enemy(3, p(3, 0)),
        enemy(4, p(4, 0)),
        enemy(5, p(2, 0)),
        enemy(6, p(3, 2)),
    ]);
    let shown = s
        .preview_attack(UnitId(1), p(0, 0), &art_attack(3, "arrow_rain"))
        .unwrap();
    let rows: Vec<_> = shown
        .strikes
        .iter()
        .map(|r| (r.by.0, r.at.0, r.numbers.hit, r.hp))
        .collect();
    assert_eq!(
        rows,
        [
            (1, 3, 100, (10, 7)),
            (1, 4, 100, (10, 7)),
            (1, 5, 100, (10, 7))
        ]
    );
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "arrow_rain"));
    // Target first, then clockwise from the north: east, then west.
    assert_eq!(struck(&events), [(1, 3), (1, 4), (1, 5)]);
    assert_eq!([2, 6].map(|id| hp(&s, id)), [10, 10]);
}

#[test]
fn arrow_rain_aims_at_two_or_three_tiles() {
    let s = battle(vec![
        artist(1, p(0, 0), "longbow"),
        enemy(3, p(1, 0)),
        enemy(4, p(0, 2)),
        enemy(5, p(4, 0)),
    ]);
    let aim = |id| s.preview_attack(UnitId(1), p(0, 0), &art_attack(id, "arrow_rain"));
    assert!(matches!(aim(3), Err(CommandError::OutOfRange { .. })));
    assert!(aim(4).is_ok());
    assert!(matches!(aim(5), Err(CommandError::OutOfRange { .. })));
}

#[test]
fn each_struck_enemy_is_its_own_combat_and_weapon_exp_is_one_award() {
    let mut s = battle(vec![
        artist(1, p(4, 2), "heavy_axe"),
        enemy(3, p(5, 2)),
        enemy(4, p(4, 1)),
        enemy(5, p(4, 3)),
    ]);
    let events = act(&mut s, 1, p(4, 2), art_attack(3, "wide_swing"));
    // Three combats' unit EXP, for the user only.
    assert_eq!(exp_awards(&events, 1), 3);
    assert_eq!([3, 4, 5].map(|id| exp_awards(&events, id)), [0, 0, 0]);
    // One weapon EXP award: base 4 (an art), dealt 3 × 5 = 15 → +3.
    assert_eq!(weapon_exp(&events, 1), [7]);
}

#[test]
fn chain_strike_brings_in_every_ally_next_to_the_target() {
    // The target at (1, 1): a player ally north (1, 0), a green one east
    // (2, 1), a non-combat green one south (1, 2); the user west (0, 1).
    let mut s = battle(vec![
        artist(1, p(0, 1), "knuckles"),
        unit(2, Faction::Player, p(1, 0)),
        tuned(enemy(3, p(1, 1)), 30, 0),
        unit(4, Faction::Ally, p(2, 1)),
        Unit {
            role: Role::Noncombatant,
            ..unit(5, Faction::Ally, p(1, 2))
        },
    ]);
    let shown = s
        .preview_attack(UnitId(1), p(0, 1), &art_attack(3, "chain_strike"))
        .unwrap();
    let rows: Vec<_> = shown.strikes.iter().map(|r| (r.by.0, r.hp)).collect();
    // From the target's HP after the combat if every strike hit.
    assert_eq!(rows, [(2, (27, 24)), (4, (24, 21))]);
    let events = act(&mut s, 1, p(0, 1), art_attack(3, "chain_strike"));
    assert_eq!(struck(&events), [(1, 3), (2, 3), (4, 3)]);
    for (attacker, _, forecast, strikes) in &combats(&events)[1..] {
        assert_eq!((forecast.defender, *strikes), (None, 1), "{attacker}");
    }
    assert_eq!(hp(&s, 3), 21);
    // The allies lose no durability and gain no EXP.
    let durability = |id| {
        s.unit(UnitId(id)).unwrap().loadout.weapons[0]
            .as_ref()
            .unwrap()
            .durability_left
    };
    assert_eq!((durability(2), durability(4)), (20, 20));
    assert_eq!(exp_awards(&events, 2), 0);
    assert_eq!(weapon_exp(&events, 2), [] as [u32; 0]);
    // The user's weapon EXP counts their damage: base 4, dealt 9 → +1.
    assert_eq!(weapon_exp(&events, 1), [5]);
}

#[test]
fn chain_strike_stops_once_the_target_falls() {
    let units = |target_hp| {
        vec![
            artist(1, p(0, 1), "knuckles"),
            unit(2, Faction::Player, p(1, 0)),
            tuned(enemy(3, p(1, 1)), target_hp, 0),
            unit(4, Faction::Player, p(2, 1)),
        ]
    };
    // The first ally's strike fells it: the second doesn't strike.
    let mut s = battle(units(6));
    let events = act(&mut s, 1, p(0, 1), art_attack(3, "chain_strike"));
    assert_eq!(struck(&events), [(1, 3), (2, 3)]);
    assert!(events.contains(&Event::UnitFell { unit: UnitId(3) }));
    // The user's own combat fells it: no chain at all.
    let mut s = battle(units(3));
    let events = act(&mut s, 1, p(0, 1), art_attack(3, "chain_strike"));
    assert_eq!(struck(&events), [(1, 3)]);
}

#[test]
fn several_strikes_roll_the_same_way_every_time() {
    // Spd 15: avoid 30, so the 2RN rolls decide; the same seed and the
    // same strike order give the same events.
    let units = vec![
        artist(1, p(0, 0), "longbow"),
        tuned(enemy(3, p(3, 0)), 10, 15),
        tuned(enemy(4, p(4, 0)), 10, 15),
        tuned(enemy(5, p(2, 0)), 10, 15),
        tuned(enemy(6, p(3, 1)), 10, 15),
    ];
    let run = || {
        let mut s = battle(units.clone());
        act(&mut s, 1, p(0, 0), art_attack(3, "arrow_rain"))
    };
    let events = run();
    assert_eq!(struck(&events), [(1, 3), (1, 4), (1, 6), (1, 5)]);
    assert_eq!(events, run());
}

/// The weapon the user of art `art` carries.
fn weapon_for(art: &str) -> &'static str {
    match art {
        "wide_swing" => "heavy_axe",
        "comet_lance" => "pike",
        "arrow_rain" => "longbow",
        _ => "knuckles",
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(96))]

    /// Whatever stands around, an art strikes only units hostile to its
    /// user, and only its user and its allies strike.
    #[test]
    fn no_art_strikes_a_unit_friendly_to_its_user(
        art in prop::sample::select(vec!["wide_swing", "comet_lance", "arrow_rain", "chain_strike"]),
        others in prop::collection::vec((0..8i32, 0..5i32, 0..3u8), 1..10),
    ) {
        let user = artist(1, p(4, 2), weapon_for(art));
        let mut units = vec![user];
        for (id, (x, y, side)) in (2..).zip(others) {
            if units.iter().any(|u| u.pos == p(x, y)) {
                continue;
            }
            let faction = [Faction::Player, Faction::Enemy, Faction::Ally][usize::from(side)];
            units.push(unit(id, faction, p(x, y)));
        }
        let s = battle(units);
        let targets: Vec<UnitId> = s
            .units()
            .iter()
            .filter(|u| u.faction == Faction::Enemy)
            .map(|u| u.id)
            .collect();
        for target in targets {
            let action = art_attack(target.0, art);
            let cmd = Command::Act { unit: UnitId(1), dest: p(4, 2), action };
            if s.check(&cmd).is_err() {
                continue;
            }
            let mut copy = s.clone();
            let events = copy.apply(&cmd).unwrap();
            for (attacker, defender, ..) in combats(&events) {
                let faction = |id| s.unit(UnitId(id)).unwrap().faction;
                prop_assert!(!Faction::Player.is_hostile_to(faction(attacker)));
                prop_assert!(Faction::Player.is_hostile_to(faction(defender)));
            }
        }
    }
}

#[test]
fn the_ai_counts_only_the_users_strikes_at_other_units() {
    // Wide Swing: the target's own strike is the forecast, not one of the
    // others.
    let s = battle(vec![
        artist(1, p(4, 2), "heavy_axe"),
        enemy(3, p(5, 2)),
        enemy(4, p(4, 1)),
        enemy(5, p(4, 3)),
    ]);
    let user = s.unit(UnitId(1)).unwrap();
    let action = art_attack(3, "wide_swing");
    let outlook = s
        .attack_forecast(user, p(4, 2), &[p(4, 2)], &action)
        .unwrap();
    let others: Vec<UnitId> = outlook.others.iter().map(|(id, _)| *id).collect();
    assert_eq!(others, [UnitId(4), UnitId(5)]);
    // Chain Strike: the allies' strikes aren't the user's.
    let s = battle(vec![
        artist(1, p(0, 1), "knuckles"),
        unit(2, Faction::Player, p(1, 0)),
        tuned(enemy(3, p(1, 1)), 30, 0),
    ]);
    let user = s.unit(UnitId(1)).unwrap();
    let action = art_attack(3, "chain_strike");
    let outlook = s
        .attack_forecast(user, p(0, 1), &[p(0, 1)], &action)
        .unwrap();
    assert_eq!(outlook.others, []);
}

#[test]
fn chain_strike_goes_clockwise_from_the_north() {
    // The user north of the target at (2, 2); allies west, south and east.
    let mut s = battle(vec![
        artist(1, p(2, 1), "knuckles"),
        unit(2, Faction::Player, p(1, 2)),
        unit(4, Faction::Player, p(2, 3)),
        unit(5, Faction::Player, p(3, 2)),
        tuned(enemy(3, p(2, 2)), 30, 0),
    ]);
    let events = act(&mut s, 1, p(2, 1), art_attack(3, "chain_strike"));
    assert_eq!(struck(&events), [(1, 3), (5, 3), (4, 3), (2, 3)]);
}
