//! The arts that change what units may do (ticket 0328, `combat-arts.md`,
//! *Exact rules*): Disarm (no counters, no weapon attacks), Marked Shot
//! (hit and crit against the target) and Unbroken Stride (act again after
//! a kill). Each is a test art of rank D here, added to [`art_setup`]'s
//! table with the design's numbers.

use super::art::{art_attack, art_setup, artist, combats, landed, preview, seed_where, stats};
use super::*;
use crate::ai::{AiWeights, next_command};
use crate::art::{ArtNote, ArtStatus, OnKill};
use crate::combat::SideForecast;
use crate::skill::TimedEffect;

fn aid(id: &str) -> ArtId {
    ArtId::new(id)
}

/// `disarm` (sword, 5), `marked_shot` (bow, 6) and `unbroken_stride`
/// (sword, 8), rank D.
fn rule_arts() -> Vec<ArtDef> {
    let art = |id: &str, kind, cost, effect| ArtDef {
        id: aid(id),
        name: id.into(),
        kind,
        rank: Some(WeaponRank::D),
        cost,
        effect,
    };
    vec![
        art(
            "disarm",
            WeaponKind::Sword,
            5,
            ArtEffect {
                on_first_hit_status: vec![ArtStatus::Disarmed],
                ..ArtEffect::default()
            },
        ),
        art(
            "marked_shot",
            WeaponKind::Bow,
            6,
            ArtEffect {
                on_first_hit_status: vec![ArtStatus::Marked { hit: 20, crit: 10 }],
                ..ArtEffect::default()
            },
        ),
        art(
            "unbroken_stride",
            WeaponKind::Sword,
            8,
            ArtEffect {
                on_kill: Some(OnKill::ActAgain),
                ..ArtEffect::default()
            },
        ),
    ]
}

fn with_rule_arts(mut setup: BattleSetup) -> BattleSetup {
    let mut arts = (*setup.arts).clone();
    for def in rule_arts() {
        arts.arts.insert(def.id.clone(), def);
    }
    setup.arts = Arc::new(arts);
    setup
}

fn battle(units: Vec<Unit>) -> BattleState {
    start(with_rule_arts(art_setup(units)))
}

fn applied(unit: u32, art: &str, until: Phase) -> Event {
    Event::EffectApplied {
        unit: UnitId(unit),
        source: EffectSource::Art(aid(art)),
        until,
    }
}

fn expired(unit: u32, art: &str) -> Event {
    Event::EffectExpired {
        unit: UnitId(unit),
        source: EffectSource::Art(aid(art)),
    }
}

fn sides(outcome: &CombatOutcome) -> Vec<Side> {
    outcome.strikes.iter().map(|s| s.by).collect()
}

fn act_cmd(id: u32, dest: Pos, action: UnitAction) -> Command {
    Command::Act {
        unit: UnitId(id),
        dest,
        action,
    }
}

fn bolt_on(target: u32) -> UnitAction {
    UnitAction::Cast {
        spell: SpellId::new("bolt"),
        target: CastTarget::Unit(UnitId(target)),
        active: None,
    }
}

/// `u` also knowing `bolt` (range 1–2), with two uses.
fn casting(mut u: Unit) -> Unit {
    let bolt = SpellId::new("bolt");
    u.learned.insert(bolt.clone());
    u.spells.uses_left = BTreeMap::from([(bolt, 2)]);
    u
}

// ---- Disarm --------------------------------------------------------------------

/// A swordsman (Str 5: 8 damage, hit 100) and an armed enemy with 30 HP
/// next to it, which knows `bolt`.
fn duel() -> Vec<Unit> {
    vec![
        stats(artist(1, p(0, 0), "blade"), [30, 5, 0, 0, 0, 0, 0]),
        casting(stats(
            unit(3, Faction::Enemy, p(1, 0)),
            [30, 0, 0, 0, 0, 0, 0],
        )),
    ]
}

#[test]
fn disarm_sure_to_hit_shows_and_gives_no_counter() {
    let mut s = battle(duel());
    assert!(
        preview(&s, 1, p(0, 0), &attack(3))
            .forecast
            .defender
            .is_some()
    );
    let shown = preview(&s, 1, p(0, 0), &art_attack(3, "disarm"));
    assert_eq!(shown.forecast.attacker.hit, 100);
    assert_eq!(shown.forecast.defender, None);
    assert_eq!(shown.notes, [ArtNote::Disarms]);
    assert_eq!(shown.durability, Some((20, 15)));
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "disarm"));
    let (_, forecast, outcome) = &combats(&events)[0];
    assert_eq!(*forecast, shown.forecast);
    assert_eq!(sides(outcome), [Side::Attacker]);
    // Until the end of the enemy's next phase.
    assert!(events.contains(&applied(3, "disarm", Phase::Other)));
    assert!(s.unit(UnitId(3)).unwrap().is_disarmed());
}

/// A swordsman with Spd 31 (two strikes) against an armed enemy with avoid
/// 50 (hit 50) on `seed`.
fn chancy_duel(seed: u64) -> BattleState {
    let mut s = battle(vec![
        stats(artist(1, p(0, 0), "blade"), [30, 5, 0, 0, 31, 0, 0]),
        stats(unit(3, Faction::Enemy, p(1, 0)), [100, 0, 0, 0, 25, 0, 0]),
    ]);
    s.rng = SimRng::new(seed);
    s
}

#[test]
fn disarm_cancels_the_counters_left_from_the_first_hit() {
    let shown = preview(&chancy_duel(0), 1, p(0, 0), &art_attack(3, "disarm"));
    let counter = shown.forecast.defender.expect("a counter before a hit");
    assert_eq!(
        (shown.forecast.attacker.hit, shown.forecast.attacker.strikes),
        (50, 2)
    );
    let after = shown.forecast.after_first_hit.expect("numbers after a hit");
    assert_eq!(after.defender, None);
    // A hit first: the counter never comes.
    let seed = seed_where(&[50, 50], |r| r[0] < 100);
    let mut s = chancy_duel(seed);
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "disarm"));
    let (_, _, outcome) = &combats(&events)[0];
    assert_eq!(sides(outcome), [Side::Attacker, Side::Attacker]);
    assert!(events.contains(&applied(3, "disarm", Phase::Other)));
    // A miss first: the counter comes, then the hit disarms.
    let seed = seed_where(&[50, counter.hit.into(), 50], |r| r[0] >= 100 && r[2] < 100);
    let mut s = chancy_duel(seed);
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "disarm"));
    let (_, _, outcome) = &combats(&events)[0];
    assert_eq!(
        sides(outcome),
        [Side::Attacker, Side::Defender, Side::Attacker]
    );
    assert!(landed(outcome)[2]);
    assert!(events.contains(&applied(3, "disarm", Phase::Other)));
    // Two misses: nothing.
    let seed = seed_where(&[50, counter.hit.into(), 50], |r| {
        r[0] >= 100 && r[2] >= 100
    });
    let mut s = chancy_duel(seed);
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "disarm"));
    assert!(!events.contains(&applied(3, "disarm", Phase::Other)));
    assert!(!s.unit(UnitId(3)).unwrap().is_disarmed());
}

#[test]
fn a_disarmed_enemy_casts_but_never_attacks_with_its_weapon() {
    let mut s = battle(duel());
    act(&mut s, 1, p(0, 0), art_attack(3, "disarm"));
    end(&mut s);
    assert_eq!(s.phase(), Phase::Enemy);
    assert_eq!(
        s.check(&act_cmd(3, p(1, 0), attack(1))),
        Err(CommandError::Disarmed(UnitId(3)))
    );
    assert_eq!(s.check(&act_cmd(3, p(1, 0), bolt_on(1))), Ok(()));
    // Its weapons leave its danger zone: only the spell's 1–2 is left.
    let enemy = s.unit(UnitId(3)).unwrap();
    let ranges = enemy.threat_ranges(s.classes(), s.items(), s.spells(), s.arts(), s.skills());
    assert_eq!(ranges, [(1, 2)]);
    // The AI casts instead.
    let Some(Command::Act { unit, action, .. }) = next_command(&s, &AiWeights::STARTING) else {
        panic!("the enemy acts");
    };
    assert_eq!(unit, UnitId(3));
    assert_eq!(action, bolt_on(1));
}

#[test]
fn a_disarmed_unit_doesnt_counter_until_the_end_of_its_next_phase() {
    let mut units = duel();
    units.push(artist(2, p(1, 1), "blade"));
    let mut s = battle(units);
    act(&mut s, 1, p(0, 0), art_attack(3, "disarm"));
    let counter = |s: &BattleState| preview(s, 2, p(1, 1), &attack(3)).forecast.defender;
    assert_eq!(counter(&s), None);
    let events = end(&mut s);
    assert!(!events.contains(&expired(3, "disarm")));
    let mut events = end(&mut s);
    if s.phase() != Phase::Player {
        events.extend(end(&mut s));
    }
    assert!(events.contains(&expired(3, "disarm")));
    assert!(counter(&s).is_some());
}

#[test]
fn disarmed_error_message() {
    assert_eq!(
        CommandError::Disarmed(UnitId(3)).to_string(),
        "unit 3 is disarmed: no weapon attacks"
    );
}

// ---- Marked Shot ---------------------------------------------------------------

/// An archer (range 2, Dex `dex`, Spd 40: two strikes) at (0, 0), a
/// swordsman at (2, 1) and an armed enemy at (2, 0) with avoid 60 (Spd
/// 30) and 100 HP.
fn marking(dex: StatValue, seed: u64) -> BattleState {
    let mut s = battle(vec![
        stats(artist(1, p(0, 0), "longbow"), [30, 0, 0, dex, 40, 0, 0]),
        stats(artist(2, p(2, 1), "blade"), [30, 0, 0, 0, 0, 0, 0]),
        stats(unit(3, Faction::Enemy, p(2, 0)), [100, 0, 0, 0, 30, 0, 0]),
    ]);
    s.rng = SimRng::new(seed);
    s
}

fn hit_crit(f: &SideForecast) -> (u8, u8) {
    (f.hit, f.crit)
}

#[test]
fn a_marked_unit_gives_every_attack_on_it_hit_and_crit_until_its_phase_ends() {
    // Dex 30 makes the archer's first strike sure.
    let mut s = marking(30, 0);
    let sword = |s: &BattleState| hit_crit(&preview(s, 2, p(2, 1), &attack(3)).forecast.attacker);
    let before = sword(&s);
    assert_eq!(before, (40, 0));
    let shown = preview(&s, 1, p(0, 0), &art_attack(3, "marked_shot"));
    assert_eq!(shown.notes, [ArtNote::Marks { hit: 20, crit: 10 }]);
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "marked_shot"));
    assert!(events.contains(&applied(3, "marked_shot", Phase::Other)));
    assert_eq!(sword(&s), (60, 10));
    // On the enemy's phase its own attack is countered with the mark too.
    end(&mut s);
    let attacked = preview(&s, 3, p(2, 0), &attack(2)).forecast;
    let counter = attacked.defender.expect("unit 2 counters");
    assert_eq!(hit_crit(&counter), (60, 10));
    let mut events = end(&mut s);
    if s.phase() != Phase::Player {
        events.extend(end(&mut s));
    }
    assert!(events.contains(&expired(3, "marked_shot")));
    assert_eq!(sword(&s), before);
}

#[test]
fn marked_shot_marks_from_its_first_hit_on() {
    // Hit 40 before the mark, 60 after it.
    let shown = preview(&marking(0, 0), 1, p(0, 0), &art_attack(3, "marked_shot"));
    assert_eq!(shown.forecast.attacker.strikes, 2);
    assert_eq!(hit_crit(&shown.forecast.attacker), (40, 0));
    let after = shown.forecast.after_first_hit.expect("numbers after a hit");
    assert_eq!(hit_crit(&after.attacker), (60, 10));
    // Miss, then a roll that needs 60: still a miss, nothing is marked.
    let seed = seed_where(&[40, 40], |r| r[0] >= 80 && (80..120).contains(&r[1]));
    let mut s = marking(0, seed);
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "marked_shot"));
    let (_, _, outcome) = &combats(&events)[0];
    assert_eq!(landed(outcome), [false, false]);
    assert!(!events.contains(&applied(3, "marked_shot", Phase::Other)));
    // Hit, then the same roll: a hit, from the mark.
    let seed = seed_where(&[40, 60], |r| r[0] < 80 && (80..120).contains(&r[1]));
    let mut s = marking(0, seed);
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "marked_shot"));
    let (_, _, outcome) = &combats(&events)[0];
    assert_eq!(landed(outcome), [true, true]);
}

// ---- Unbroken Stride -----------------------------------------------------------

/// A swordsman (Str 10 with might 10: 20 damage, hit 100, Mov 3) at
/// (0, 0), an enemy with `hp` next to it and a sturdy one at (4, 0).
fn stride(hp: StatValue) -> BattleState {
    battle(vec![
        stats(artist(1, p(0, 0), "big_blade"), [30, 10, 0, 0, 0, 0, 0]),
        stats(unit(3, Faction::Enemy, p(1, 0)), [hp, 0, 0, 0, 0, 0, 0]),
        stats(unit(4, Faction::Enemy, p(4, 0)), [60, 0, 0, 0, 0, 0, 0]),
    ])
}

#[test]
fn unbroken_stride_acts_again_after_a_kill_once_a_phase() {
    let mut s = stride(10);
    let shown = preview(&s, 1, p(0, 0), &art_attack(3, "unbroken_stride"));
    assert_eq!(shown.notes, [ArtNote::ActsAgain]);
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "unbroken_stride"));
    assert!(events.contains(&Event::UnitFell { unit: UnitId(3) }));
    assert!(events.ends_with(&[
        applied(1, "unbroken_stride", Phase::Enemy),
        Event::ActsAgain { unit: UnitId(1) },
    ]));
    assert!(!events.contains(&Event::UnitActed { unit: UnitId(1) }));
    assert!(!s.unit(UnitId(1)).unwrap().acted);
    // Its full Mov again, but not the art again this phase.
    assert_eq!(
        s.check(&act_cmd(1, p(3, 0), art_attack(4, "unbroken_stride"))),
        Err(CommandError::ArtNotUsable {
            unit: UnitId(1),
            art: aid("unbroken_stride"),
        })
    );
    let unit = s.unit(UnitId(1)).unwrap();
    let usable = unit.usable_arts(0, s.classes(), s.items(), s.arts());
    assert!(!usable.iter().any(|a| a.id == aid("unbroken_stride")));
    let events = act(&mut s, 1, p(3, 0), attack(4));
    assert!(events.contains(&Event::UnitActed { unit: UnitId(1) }));
    assert!(s.unit(UnitId(1)).unwrap().acted);
    // The next phase clears it.
    let events = end(&mut s);
    assert!(events.contains(&expired(1, "unbroken_stride")));
    let unit = s.unit(UnitId(1)).unwrap();
    let usable = unit.usable_arts(0, s.classes(), s.items(), s.arts());
    assert!(usable.iter().any(|a| a.id == aid("unbroken_stride")));
}

#[test]
fn unbroken_stride_without_a_kill_ends_the_action() {
    let mut s = stride(40);
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "unbroken_stride"));
    assert!(events.contains(&Event::UnitActed { unit: UnitId(1) }));
    assert!(!events.contains(&Event::ActsAgain { unit: UnitId(1) }));
    assert!(!events.contains(&applied(1, "unbroken_stride", Phase::Enemy)));
    assert!(s.unit(UnitId(1)).unwrap().acted);
}

#[test]
fn a_kill_without_unbroken_stride_ends_the_action() {
    let mut s = stride(10);
    let events = act(&mut s, 1, p(0, 0), attack(3));
    assert!(events.contains(&Event::UnitFell { unit: UnitId(3) }));
    assert!(events.contains(&Event::UnitActed { unit: UnitId(1) }));
    assert!(s.unit(UnitId(1)).unwrap().acted);
}

// ---- Property ------------------------------------------------------------------

/// The unit `id`, standing or not.
fn anywhere(s: &BattleState, id: UnitId) -> Option<&Unit> {
    s.units()
        .iter()
        .chain(s.fallen())
        .chain(s.down())
        .chain(s.escaped())
        .find(|u| u.id == id)
}

proptest! {
    #![proptest_config(ProptestConfig {
        max_shrink_iters: 32,
        max_shrink_time: 5_000,
        ..ProptestConfig::with_cases(64)
    })]

    /// Units disarmed from the start (until the Other phase), playing by
    /// the AI and at random: none of them strikes with a weapon.
    #[test]
    fn a_disarmed_unit_never_makes_a_weapon_strike(
        mut setup in arb_setup(),
        mask in any::<u16>(),
        choices in prop::collection::vec(any::<u16>(), 0..60),
    ) {
        for (i, u) in setup.units.iter_mut().enumerate() {
            if mask & (1 << i) != 0 {
                u.add_effect(TimedEffect {
                    source: EffectSource::Art(aid("disarm")),
                    mods: TimedMods { disarmed: true, ..TimedMods::default() },
                    until: Phase::Other,
                });
            }
        }
        let (mut s, _) = BattleState::new(setup);
        for choice in choices {
            if s.outcome().is_some() {
                break;
            }
            let legal = legal_commands(&s);
            let cmd = if choice % 2 == 0 {
                next_command(&s, &AiWeights::STARTING).unwrap_or(Command::EndPhase)
            } else {
                legal[usize::from(choice) % legal.len()].clone()
            };
            let disarmed: Vec<UnitId> =
                s.units().iter().filter(|u| u.is_disarmed()).map(|u| u.id).collect();
            let events = s.apply(&cmd).map_err(|e| TestCaseError::fail(e.to_string()))?;
            for e in &events {
                let Event::CombatResolved { attacker, defender, outcome, .. } = e else {
                    continue;
                };
                for strike in &outcome.strikes {
                    let by = if strike.by == Side::Attacker { *attacker } else { *defender };
                    let weapon = anywhere(&s, by)
                        .is_some_and(|u| matches!(u.loadout.equipped, Some(Equipped::Weapon(_))));
                    prop_assert!(
                        !(disarmed.contains(&by) && weapon),
                        "disarmed unit {:?} struck with a weapon: {:?}", by, cmd
                    );
                }
            }
        }
    }
}
