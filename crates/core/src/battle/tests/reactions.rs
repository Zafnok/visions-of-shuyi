//! Stances that react (ticket 0326, `combat-arts.md`, *Exact rules*):
//! Riposte strikes first, Spear Wall counters at 2, Deflect turns the first
//! hit into a miss and a strike back, and Overwatch shoots the first enemy
//! that ends a move in range. Each is a test art of rank D here, added to
//! [`art_setup`]'s table: `vantage` (Riposte's stance, sword), `spear_wall`
//! (spear), `deflect` (gauntlet) and `overwatch` (bow).

use super::art::{art_attack, art_setup, artist};
use super::*;
use crate::combat::{Reactions, StrikeReaction};
use crate::skill::{StanceReaction, TimedEffect};

fn aid(id: &str) -> ArtId {
    ArtId::new(id)
}

/// The four arts, rank D, with these stances.
fn reacting_arts() -> Vec<ArtDef> {
    let stance = |f: fn(&mut TimedMods)| {
        let mut mods = TimedMods::default();
        f(&mut mods);
        ArtEffect {
            stance: Some(mods),
            ..ArtEffect::default()
        }
    };
    let art = |id: &str, kind, cost, effect| ArtDef {
        id: ArtId::new(id),
        name: id.into(),
        kind,
        rank: Some(WeaponRank::D),
        cost,
        effect,
    };
    vec![
        art(
            "vantage",
            WeaponKind::Sword,
            4,
            stance(|m| m.reaction = Some(StanceReaction::StrikesFirst)),
        ),
        art(
            "spear_wall",
            WeaponKind::Spear,
            4,
            stance(|m| m.reaction = Some(StanceReaction::Counters { reach: 2, hit: 15 })),
        ),
        art(
            "deflect",
            WeaponKind::Gauntlet,
            5,
            stance(|m| m.reaction = Some(StanceReaction::Deflect)),
        ),
        art(
            "overwatch",
            WeaponKind::Bow,
            5,
            stance(|m| m.reaction = Some(StanceReaction::Overwatch)),
        ),
    ]
}

fn battle(units: Vec<Unit>) -> BattleState {
    let mut setup = art_setup(units);
    let mut arts = (*setup.arts).clone();
    for def in reacting_arts() {
        arts.arts.insert(def.id.clone(), def);
    }
    setup.arts = Arc::new(arts);
    // Classes that give unit EXP, for the Overwatch shot's award.
    setup.classes = Arc::new(leveling((*setup.classes).clone()));
    start(setup)
}

/// `u` with `hp` HP and these Dex and Spd.
fn tuned(u: Unit, hp: StatValue, dex: StatValue, spd: StatValue) -> Unit {
    Unit {
        stats: Stats::from_growable([hp, 0, 0, dex, spd, 0, 0], 3),
        hp,
        ..u
    }
}

/// An enemy that always hits (Dex 30) with 30 HP.
fn sure(id: u32, pos: Pos) -> Unit {
    tuned(unit(id, Faction::Enemy, pos), 30, 30, 0)
}

/// Every combat in `events`: `(attacker, defender, forecast, outcome)`.
fn combats(events: &[Event]) -> Vec<(UnitId, UnitId, Forecast, CombatOutcome)> {
    events
        .iter()
        .filter_map(|e| match e {
            Event::CombatResolved {
                attacker,
                defender,
                forecast,
                outcome,
            } => Some((*attacker, *defender, *forecast, outcome.clone())),
            _ => None,
        })
        .collect()
}

/// Who struck, and how, in `outcome`.
fn strikes(outcome: &CombatOutcome) -> Vec<(Side, Option<StrikeReaction>)> {
    outcome.strikes.iter().map(|s| (s.by, s.reaction)).collect()
}

fn hp(s: &BattleState, id: u32) -> StatValue {
    s.unit(UnitId(id)).map_or(0, |u| u.hp)
}

fn expired(unit: u32, art: &str) -> Event {
    Event::EffectExpired {
        unit: UnitId(unit),
        source: EffectSource::Art(aid(art)),
    }
}

fn use_art(art: &str) -> UnitAction {
    UnitAction::UseArt {
        art: aid(art),
        slot: 0,
    }
}

// ---- Riposte -----------------------------------------------------------------

#[test]
fn riposte_strikes_first_when_attacked_not_when_attacking() {
    let mut s = battle(vec![
        tuned(artist(1, p(0, 0), "blade"), 30, 0, 0),
        sure(3, p(1, 0)),
        sure(4, p(2, 1)),
    ]);
    // Used in an attack, it changes nothing in that combat.
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "vantage"));
    let [(_, _, _, outcome)] = &combats(&events)[..] else {
        panic!("one combat: {events:?}");
    };
    assert_eq!(
        strikes(outcome),
        [(Side::Attacker, None), (Side::Defender, None)]
    );
    // On the enemy's turn the user's counter comes first, in the preview's
    // plan and in the combat.
    end(&mut s);
    let shown = s
        .preview_attack(UnitId(4), p(0, 1), &attack(1))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(shown.forecast.reactions.defender_first);
    assert_eq!(shown.plan.strikes[0].by, Side::Defender);
    let events = act(&mut s, 4, p(0, 1), attack(1));
    let [(_, _, forecast, outcome)] = &combats(&events)[..] else {
        panic!("one combat: {events:?}");
    };
    assert_eq!(
        forecast.reactions,
        Reactions {
            defender_first: true,
            deflects: [false, false],
        }
    );
    assert_eq!(
        strikes(outcome),
        [(Side::Defender, None), (Side::Attacker, None)]
    );
}

#[test]
fn riposte_needs_a_counter() {
    // An archer at 2: the sword can't counter, so nothing changes.
    let mut s = battle(vec![
        tuned(artist(1, p(0, 0), "blade"), 30, 0, 0),
        sure(3, p(1, 0)),
        tuned(
            with_weapon(unit(4, Faction::Enemy, p(4, 0)), "longbow"),
            30,
            30,
            0,
        ),
    ]);
    act(&mut s, 1, p(0, 0), art_attack(3, "vantage"));
    end(&mut s);
    let events = act(&mut s, 4, p(2, 0), attack(1));
    let [(_, _, forecast, outcome)] = &combats(&events)[..] else {
        panic!("one combat: {events:?}");
    };
    assert!(!forecast.reactions.defender_first);
    assert_eq!(strikes(outcome), [(Side::Attacker, None)]);
}

fn with_weapon(u: Unit, id: &str) -> Unit {
    carrying(u, &[item(id)])
}

// ---- Spear Wall --------------------------------------------------------------

#[test]
fn spear_wall_counters_an_archer_at_two_with_hit_plus_fifteen() {
    // The archer has Spd 10: avoid 20, so the pike's counter hits 80, 95
    // with the stance.
    let archer = || {
        tuned(
            with_weapon(unit(4, Faction::Enemy, p(4, 0)), "longbow"),
            30,
            30,
            10,
        )
    };
    let mut plain = battle(vec![
        tuned(artist(1, p(0, 0), "pike"), 30, 0, 0),
        sure(3, p(1, 0)),
        archer(),
    ]);
    act(&mut plain, 1, p(0, 0), art_attack(3, "unhorse"));
    end(&mut plain);
    let before = plain
        .preview_attack(UnitId(4), p(2, 0), &attack(1))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(before.forecast.defender, None);

    let mut s = battle(vec![
        tuned(artist(1, p(0, 0), "pike"), 30, 0, 0),
        sure(3, p(1, 0)),
        archer(),
    ]);
    act(&mut s, 1, p(0, 0), art_attack(3, "spear_wall"));
    end(&mut s);
    let events = act(&mut s, 4, p(2, 0), attack(1));
    let [(_, _, forecast, outcome)] = &combats(&events)[..] else {
        panic!("one combat: {events:?}");
    };
    assert_eq!(forecast.defender.map(|d| d.hit), Some(95));
    assert_eq!(
        strikes(outcome)[..2],
        [(Side::Attacker, None), (Side::Defender, None)]
    );
    // Adjacent, it still counters, with the bonus.
    let shown = s
        .preview_attack(UnitId(3), p(1, 0), &attack(1))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(shown.forecast.defender.map(|d| d.hit), Some(100));
}

#[test]
fn spear_wall_bonus_is_for_counters_only() {
    let s = battle(vec![
        tuned(artist(1, p(0, 0), "pike"), 30, 0, 0),
        tuned(unit(3, Faction::Enemy, p(1, 0)), 30, 0, 10),
    ]);
    // Avoid 20: the user's own attack hits 80 with or without the stance.
    let shown = s
        .preview_attack(UnitId(1), p(0, 0), &art_attack(3, "spear_wall"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(shown.forecast.attacker.hit, 80);
}

// ---- Deflect -----------------------------------------------------------------

#[test]
fn deflect_turns_the_first_hit_into_a_miss_and_a_strike_back_once() {
    let mut s = battle(vec![
        tuned(artist(1, p(0, 0), "knuckles"), 30, 0, 0),
        sure(3, p(1, 0)),
        sure(4, p(2, 1)),
    ]);
    // In its own combat: the counter that would hit misses, and the user
    // strikes back at once (3 damage, as its first strike did).
    let shown = s
        .preview_attack(UnitId(1), p(0, 0), &art_attack(3, "deflect"))
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(shown.forecast.reactions.deflects, [true, false]);
    let planned: Vec<_> = shown
        .plan
        .strikes
        .iter()
        .map(|x| (x.by, x.reaction))
        .collect();
    assert_eq!(
        planned,
        [
            (Side::Attacker, None),
            (Side::Defender, Some(StrikeReaction::Deflected)),
            (Side::Attacker, Some(StrikeReaction::StrikeBack)),
        ]
    );
    assert_eq!(shown.plan.attacker_hp, 30);
    assert_eq!(shown.plan.defender_hp, 24);
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "deflect"));
    let [(_, _, _, outcome)] = &combats(&events)[..] else {
        panic!("one combat: {events:?}");
    };
    assert_eq!(
        strikes(outcome),
        [
            (Side::Attacker, None),
            (Side::Defender, Some(StrikeReaction::Deflected)),
            (Side::Attacker, Some(StrikeReaction::StrikeBack)),
        ]
    );
    let deflected = &outcome.strikes[1];
    assert!(!deflected.hit && deflected.damage == 0 && !deflected.crit);
    assert_eq!(outcome.strikes[2].damage, 3);
    assert_eq!((outcome.attacker_hp, outcome.defender_hp), (30, 24));
    // The stance has reacted: it ends after the combat.
    let i = events
        .iter()
        .position(|e| matches!(e, Event::CombatResolved { .. }))
        .unwrap_or(0);
    assert_eq!(events.get(i + 1), Some(&expired(1, "deflect")));
    // The next enemy hits normally.
    end(&mut s);
    let events = act(&mut s, 4, p(0, 1), attack(1));
    let [(_, _, forecast, outcome)] = &combats(&events)[..] else {
        panic!("one combat: {events:?}");
    };
    assert_eq!(forecast.reactions.deflects, [false, false]);
    assert_eq!(
        strikes(outcome),
        [(Side::Attacker, None), (Side::Defender, None)]
    );
    assert_eq!(hp(&s, 1), 27);
}

#[test]
fn deflect_waits_for_a_strike_that_would_hit() {
    // An enemy that can't hit (Dex 0 against the gauntlet's avoid 15 and
    // Spd 50: hit 0) misses on its own; the stance is kept.
    let mut s = battle(vec![
        tuned(artist(1, p(0, 0), "knuckles"), 30, 0, 50),
        tuned(unit(3, Faction::Enemy, p(1, 0)), 30, 0, 0),
    ]);
    let events = act(&mut s, 1, p(0, 0), art_attack(3, "deflect"));
    let [(_, _, _, outcome)] = &combats(&events)[..] else {
        panic!("one combat: {events:?}");
    };
    assert!(outcome.strikes.iter().all(|x| x.reaction.is_none()));
    assert!(!events.contains(&expired(1, "deflect")));
    assert!(
        s.unit(UnitId(1))
            .is_some_and(|u| u.has_effect(&EffectSource::Art(aid("deflect"))))
    );
}

#[test]
fn an_attacked_unit_deflects_strikes_back_and_still_counters() {
    let mut s = battle(vec![
        tuned(artist(1, p(0, 0), "knuckles"), 30, 30, 0),
        tuned(unit(3, Faction::Enemy, p(5, 0)), 30, 0, 0),
        sure(4, p(2, 1)),
    ]);
    act(&mut s, 1, p(0, 0), UnitAction::Wait);
    if let Some(u) = s.units.iter_mut().find(|u| u.id == UnitId(1)) {
        u.add_effect(TimedEffect {
            source: EffectSource::Art(aid("deflect")),
            mods: TimedMods {
                reaction: Some(StanceReaction::Deflect),
                ..TimedMods::default()
            },
            until: Phase::Player,
        });
    }
    end(&mut s);
    let events = act(&mut s, 4, p(0, 1), attack(1));
    let [(_, _, forecast, outcome)] = &combats(&events)[..] else {
        panic!("one combat: {events:?}");
    };
    assert_eq!(forecast.reactions.deflects, [false, true]);
    // The strike back doesn't use up the counter.
    let planned: Vec<_> = crate::combat::if_all_hit(
        forecast,
        crate::combat::CombatHp {
            current: 30,
            max: 30,
        },
        crate::combat::CombatHp {
            current: 30,
            max: 30,
        },
    )
    .strikes
    .iter()
    .map(|x| (x.by, x.reaction))
    .collect();
    assert_eq!(
        planned,
        [
            (Side::Attacker, Some(StrikeReaction::Deflected)),
            (Side::Defender, Some(StrikeReaction::StrikeBack)),
            (Side::Defender, None),
        ]
    );
    assert_eq!(
        strikes(outcome),
        [
            (Side::Attacker, Some(StrikeReaction::Deflected)),
            (Side::Defender, Some(StrikeReaction::StrikeBack)),
            (Side::Defender, None),
        ]
    );
    assert_eq!((outcome.attacker_hp, outcome.defender_hp), (24, 30));
}

#[test]
fn a_deflecting_unit_that_cant_counter_only_deflects() {
    let mut s = battle(vec![
        tuned(artist(1, p(0, 0), "knuckles"), 30, 0, 0),
        tuned(unit(3, Faction::Enemy, p(5, 0)), 30, 0, 0),
        tuned(
            with_weapon(unit(4, Faction::Enemy, p(4, 0)), "longbow"),
            30,
            30,
            0,
        ),
    ]);
    act(&mut s, 1, p(0, 0), UnitAction::Wait);
    if let Some(u) = s.units.iter_mut().find(|u| u.id == UnitId(1)) {
        u.add_effect(TimedEffect {
            source: EffectSource::Art(aid("deflect")),
            mods: TimedMods {
                reaction: Some(StanceReaction::Deflect),
                ..TimedMods::default()
            },
            until: Phase::Player,
        });
    }
    end(&mut s);
    let events = act(&mut s, 4, p(2, 0), attack(1));
    let [(_, _, _, outcome)] = &combats(&events)[..] else {
        panic!("one combat: {events:?}");
    };
    assert_eq!(
        strikes(outcome),
        [(Side::Attacker, Some(StrikeReaction::Deflected))]
    );
    assert_eq!(hp(&s, 1), 30);
    assert!(events.contains(&expired(1, "deflect")));
}

// ---- Overwatch ---------------------------------------------------------------

/// An archer on Overwatch at (0,0) (bow range 2), with enemies placed
/// where `enemies` says.
fn on_watch(enemies: Vec<Unit>) -> (BattleState, Vec<Event>) {
    let mut units = vec![tuned(artist(1, p(0, 0), "longbow"), 30, 30, 0)];
    units.extend(enemies);
    let mut s = battle(units);
    let events = act(&mut s, 1, p(0, 0), use_art("overwatch"));
    end(&mut s);
    (s, events)
}

#[test]
fn overwatch_is_an_action_that_pays_and_takes_the_stance() {
    let (s, events) = on_watch(vec![sure(3, p(6, 0))]);
    assert_eq!(
        events,
        [
            Event::ArtUsed {
                unit: UnitId(1),
                art: aid("overwatch"),
                weapon: item("longbow"),
                durability_before: 20,
                durability_after: 15,
            },
            Event::DurabilitySpent {
                unit: UnitId(1),
                slot: 0,
                item: item("longbow"),
                amount: 5,
                left: 15,
            },
            Event::EffectApplied {
                unit: UnitId(1),
                source: EffectSource::Art(aid("overwatch")),
                until: Phase::Player,
            },
            Event::UnitActed { unit: UnitId(1) },
        ]
    );
    assert!(
        s.unit(UnitId(1))
            .is_some_and(|u| u.has_effect(&EffectSource::Art(aid("overwatch"))))
    );
}

#[test]
fn overwatch_is_a_legal_command_for_the_bots() {
    let s = battle(vec![
        tuned(artist(1, p(0, 0), "longbow"), 30, 30, 0),
        sure(3, p(6, 0)),
    ]);
    let watch = Command::Act {
        unit: UnitId(1),
        dest: p(0, 0),
        action: use_art("overwatch"),
    };
    let legal = legal_commands(&s);
    assert!(legal.contains(&watch));
    // Only the action art: an attack art isn't an action.
    assert!(!legal.iter().any(|c| matches!(
        c,
        Command::Act {
            action: UnitAction::UseArt { art, .. },
            ..
        } if *art != aid("overwatch")
    )));
}

#[test]
fn a_shopper_shot_on_the_way_keeps_its_wounds() {
    // A player unit walks to a vendor within reach of an enemy on
    // Overwatch: the visit doesn't undo the shot.
    let mut map = map(&OPEN);
    map.features = [(
        p(2, 0),
        TileFeature::Shop(crate::shop::Shop {
            kind: crate::shop::ShopKind::Vendor,
            stock: vec![item("potion")],
        }),
    )]
    .into_iter()
    .collect();
    let mut watcher = carrying(unit(3, Faction::Enemy, p(4, 0)), &[weapon(1, 2, 3)]);
    watcher.add_effect(TimedEffect {
        source: EffectSource::Art(aid("overwatch")),
        mods: TimedMods {
            reaction: Some(StanceReaction::Overwatch),
            ..TimedMods::default()
        },
        until: Phase::Enemy,
    });
    let shopper = tuned(unit(1, Faction::Player, p(0, 0)), 30, 0, 0);
    let mut s = start(BattleSetup {
        map,
        gold: 1000,
        ..setup(vec![shopper, tuned(watcher, 30, 30, 0)])
    });
    let events = act(
        &mut s,
        1,
        p(2, 0),
        UnitAction::Shop {
            txns: vec![ShopTxn::Buy {
                item: item("potion"),
            }],
        },
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::OverwatchShot { .. }))
    );
    assert_eq!(hp(&s, 1), 27);
}

#[test]
fn overwatch_is_refused_in_an_attack_and_other_arts_as_an_action() {
    let s = battle(vec![
        tuned(artist(1, p(0, 0), "longbow"), 30, 30, 0),
        sure(3, p(2, 0)),
    ]);
    let refused = |action| {
        s.check(&Command::Act {
            unit: UnitId(1),
            dest: p(0, 0),
            action,
        })
    };
    let not_usable = |art: &str| {
        Err(CommandError::ArtNotUsable {
            unit: UnitId(1),
            art: aid(art),
        })
    };
    assert_eq!(refused(art_attack(3, "overwatch")), not_usable("overwatch"));
    assert_eq!(refused(use_art("pinning_shot")), not_usable("pinning_shot"));
    // A broken bow can't pay for it.
    let mut broken = battle(vec![
        tuned(artist(1, p(0, 0), "longbow"), 30, 30, 0),
        sure(3, p(2, 0)),
    ]);
    if let Some(Some(w)) = broken
        .units
        .iter_mut()
        .find(|u| u.id == UnitId(1))
        .and_then(|u| u.loadout.weapons.first_mut())
    {
        w.durability_left = 0;
    }
    assert!(matches!(
        broken.check(&Command::Act {
            unit: UnitId(1),
            dest: p(0, 0),
            action: use_art("overwatch"),
        }),
        Err(CommandError::CannotPayArt { .. })
    ));
}

#[test]
fn overwatch_shoots_the_first_enemy_ending_a_move_in_range_once() {
    // Enemy 3 stands in range and doesn't move: not shot. Enemy 4 moves
    // to 2 tiles away: shot once before it acts. Enemy 5 moves in after:
    // the stance is gone.
    let (mut s, _) = on_watch(vec![sure(3, p(2, 0)), sure(4, p(4, 1)), sure(5, p(2, 3))]);
    let events = act(&mut s, 3, p(2, 0), UnitAction::Wait);
    assert!(combats(&events).is_empty(), "{events:?}");
    let events = act(&mut s, 4, p(1, 1), UnitAction::Wait);
    let shot = events
        .iter()
        .position(|e| {
            *e == Event::OverwatchShot {
                unit: UnitId(1),
                art: aid("overwatch"),
                target: UnitId(4),
            }
        })
        .unwrap_or_else(|| panic!("no shot: {events:?}"));
    assert!(matches!(events[shot - 1], Event::UnitMoved { .. }));
    let [(attacker, defender, forecast, outcome)] = &combats(&events)[..] else {
        panic!("one combat: {events:?}");
    };
    assert_eq!((*attacker, *defender), (UnitId(1), UnitId(4)));
    assert_eq!(forecast.defender, None);
    assert_eq!(forecast.attacker.strikes, 1);
    assert_eq!(strikes(outcome), [(Side::Attacker, None)]);
    assert_eq!(hp(&s, 4), 27);
    assert!(events.contains(&expired(1, "overwatch")));
    let exp = s.items().rules.weapon_exp(1, 1, 3, true);
    assert!(events.contains(&Event::WeaponExpGained {
        unit: UnitId(1),
        kind: WeaponKind::Bow,
        amount: exp,
    }));
    assert!(
        events
            .iter()
            .any(|e| matches!(e, Event::ExpGained { unit, .. } if *unit == UnitId(1)))
    );
    // It then acts as planned.
    assert_eq!(events.last(), Some(&Event::UnitActed { unit: UnitId(4) }));
    let events = act(&mut s, 5, p(0, 2), UnitAction::Wait);
    assert!(combats(&events).is_empty(), "{events:?}");
}

#[test]
fn an_enemy_out_of_range_isnt_shot_and_the_stance_waits() {
    let (mut s, _) = on_watch(vec![sure(3, p(6, 0)), sure(4, p(2, 2))]);
    // To 4 tiles away: out of the bow's range.
    let events = act(&mut s, 3, p(4, 0), UnitAction::Wait);
    assert!(combats(&events).is_empty(), "{events:?}");
    // An enemy next to it: out of a range-2 bow's range too.
    let events = act(&mut s, 4, p(1, 0), UnitAction::Wait);
    assert!(combats(&events).is_empty(), "{events:?}");
    // The stance ends at the start of the user's next phase.
    let events = end(&mut s);
    assert!(events.contains(&expired(1, "overwatch")), "{events:?}");
}

#[test]
fn an_enemy_shot_down_by_overwatch_doesnt_act() {
    let weak = tuned(unit(3, Faction::Enemy, p(3, 1)), 3, 30, 0);
    let (mut s, _) = on_watch(vec![weak]);
    let hp_before = hp(&s, 1);
    // It means to attack the archer from next to it, ending its move at 1:
    // out of range. From (1,1) it is 2 away: shot first.
    let events = act(&mut s, 3, p(1, 1), UnitAction::Wait);
    assert!(events.contains(&Event::UnitFell { unit: UnitId(3) }));
    assert!(!events.contains(&Event::UnitActed { unit: UnitId(3) }));
    assert_eq!(hp(&s, 1), hp_before);
    // It was the last enemy: the battle is won.
    assert_eq!(s.outcome(), Some(Outcome::Victory));
}

#[test]
fn a_shot_down_attacker_never_strikes() {
    // A brigand with a range-2 weapon walks to 2 tiles away to attack the
    // archer; the shot fells it first.
    let brigand = tuned(
        carrying(unit(3, Faction::Enemy, p(4, 0)), &[weapon(1, 2, 3)]),
        3,
        30,
        0,
    );
    let (mut s, _) = on_watch(vec![brigand, sure(4, p(7, 4))]);
    let events = act(&mut s, 3, p(2, 0), attack(1));
    let fights = combats(&events);
    assert_eq!(fights.len(), 1, "{events:?}");
    assert_eq!(fights[0].0, UnitId(1));
    assert!(events.contains(&Event::UnitFell { unit: UnitId(3) }));
    assert_eq!(hp(&s, 1), 30);
}

// ---- Expiry --------------------------------------------------------------------

#[test]
fn each_stance_ends_at_the_start_of_the_users_next_phase() {
    for (weapon, art) in [
        ("blade", "vantage"),
        ("pike", "spear_wall"),
        ("knuckles", "deflect"),
    ] {
        // The target has no weapon: nothing reacts.
        let target = carrying(unit(3, Faction::Enemy, p(1, 0)), &[]);
        let mut s = battle(vec![
            tuned(artist(1, p(0, 0), weapon), 30, 0, 0),
            tuned(target, 30, 0, 0),
        ]);
        act(&mut s, 1, p(0, 0), art_attack(3, art));
        assert!(
            s.unit(UnitId(1))
                .is_some_and(|u| u.has_effect(&EffectSource::Art(aid(art)))),
            "{art}"
        );
        let enemy_phase = end(&mut s);
        assert!(!enemy_phase.contains(&expired(1, art)), "{art}");
        let player_phase = end(&mut s);
        assert!(player_phase.contains(&expired(1, art)), "{art}");
    }
}
