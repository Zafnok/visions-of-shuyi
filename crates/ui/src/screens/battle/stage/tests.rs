//! What the combat stage shows through a fight: read from the view
//! (ADR-0054), never from cells.

use trpg_core::{
    CharacterId, ClassId, CombatOutcome, Event, Forecast, Pos, StatValue, Unit, UnitId,
};

use super::*;
use crate::screen::tests::ctx;
use crate::screens::battle::playback::{TIMINGS, Timings};
use crate::screens::battle::quick_battle;
use crate::words::Words;

const T: Timings = TIMINGS;
const LORD: UnitId = UnitId(1);
const BRIGAND: UnitId = UnitId(4);

fn strike(by: Side, hit: bool, crit: bool, damage: StatValue, after: StatValue) -> Strike {
    Strike {
        by,
        hit,
        crit,
        damage,
        healed: false,
        target_hp_after: after,
    }
}

fn side_numbers(damage: StatValue, hit: u8, crit: u8) -> SideForecast {
    SideForecast {
        damage,
        followup_damage: damage,
        hit,
        crit,
        strikes: 1,
        effective: false,
        broken: false,
        affinity: None,
    }
}

/// The Quick Battle's units: the lord is unit 1 (19 HP, at (3, 5)), the
/// first brigand unit 4 (20 HP, at (8, 2)).
fn units() -> Vec<Unit> {
    quick_battle(&ctx().content).unwrap().units().to_vec()
}

/// `attacker` walks to `to`, then fights `defender` with `strikes`; the
/// defender counters if `counter`.
fn fight(
    (attacker, to): (UnitId, Pos),
    defender: UnitId,
    strikes: Vec<Strike>,
    counter: bool,
) -> Vec<Event> {
    vec![
        Event::UnitMoved {
            unit: attacker,
            path: vec![Pos::new(5, 2), to],
        },
        Event::CombatResolved {
            attacker,
            defender,
            forecast: Forecast {
                attacker: side_numbers(8, 96, 3),
                defender: counter.then(|| side_numbers(9, 65, 0)),
            },
            outcome: CombatOutcome {
                strikes,
                attacker_hp: 19,
                defender_hp: 0,
            },
        },
    ]
}

/// The lord steps next to the brigand and hits for 8, the brigand misses,
/// the lord crits for 27; the brigand falls.
///
/// On the clock: intro to 0.3; strike 1 runs up to 0.6, lands, gap from
/// 1.1; strike 2 runs up from 1.35, lands at 1.65, gap from 2.15; strike
/// 3 runs up from 2.4, lands at 2.7; the fall from 3.2; the outro from
/// 3.7 to 4.1.
fn kill() -> Playback {
    let before = units();
    let strikes = vec![
        strike(Side::Attacker, true, false, 8, 12),
        strike(Side::Defender, false, false, 0, 19),
        strike(Side::Attacker, true, true, 27, 0),
    ];
    let mut events = fight((LORD, Pos::new(7, 2)), BRIGAND, strikes, true);
    events.push(Event::UnitFell { unit: BRIGAND });
    let mut fallen = before[3].clone();
    fallen.hp = 0;
    Playback::new(Words::ENGLISH, &events, &before, &[fallen], T).unwrap()
}

/// The stage `t` seconds into `pb`, on unknown ground.
fn at(pb: &Playback, t: f32) -> StageView {
    let mut pb = pb.clone();
    pb.tick(t, false);
    view(&ctx(), &pb, None).unwrap()
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 1e-3
}

#[test]
fn the_timeline_these_tests_count_on() {
    let pb = kill();
    let starts: Vec<f32> = pb.steps().iter().map(|s| s.start).collect();
    let expected = [0.0, 0.3, 0.6, 1.1, 1.35, 1.65, 2.15, 2.4, 2.7, 3.2, 3.7];
    assert_eq!(starts.len(), expected.len());
    for (s, e) in starts.iter().zip(expected) {
        assert!(close(*s, e), "{starts:?}");
    }
}

#[test]
fn before_the_first_strike_both_stand_still_with_their_numbers() {
    let v = at(&kill(), 0.1);
    let [a, d] = &v.fighters;
    assert_eq!((a.name.as_str(), d.name.as_str()), ("Test Lord", "Brigand"));
    assert_eq!((a.faction, d.faction), (Faction::Player, Faction::Enemy));
    assert_eq!((a.hp, a.max, d.hp, d.max), (19, 19, 20, 20));
    assert_eq!(
        a.numbers,
        Some(Numbers {
            damage: 8,
            heals: false,
            hit: 96,
            crit: 3
        })
    );
    assert_eq!(
        d.numbers,
        Some(Numbers {
            damage: 9,
            heals: false,
            hit: 65,
            crit: 0
        })
    );
    assert_eq!((a.pose, d.pose), (Pose::default(), Pose::default()));
    assert!(!a.lit && !d.lit);
    // Their classes' pictures (the lord has none of his own).
    let id = |f: &FighterView| f.picture.as_ref().map(|p| p.id.clone());
    assert_eq!(id(a).as_deref(), Some("exile"));
    assert_eq!(id(d).as_deref(), Some("brigand"));
    // The player's fighter on the left; side by side after the walk.
    assert_eq!(v.left, 0);
    assert!(v.melee);
    assert_eq!((v.striker, &v.result, v.veil), (None, &None, false));
    assert_eq!(v.banner, None);
    assert_eq!(
        v.backdrop.map(trpg_content::ImageId::path),
        Some("battlebacks/fields.png")
    );
}

#[test]
fn a_striker_runs_up_with_its_name_lit_on_and_off() {
    let pb = kill();
    // Just into the run-up: lit; a blink later: not.
    let v = at(&pb, 0.33);
    assert_eq!(v.striker, Some(0));
    assert!(v.fighters[0].lit && !v.fighters[1].lit);
    assert!(close(v.fighters[0].pose.lunge, motion::run_up(0.1, false)));
    assert_eq!(v.fighters[1].pose, Pose::default());
    let v = at(&pb, 0.4);
    assert!(!v.fighters[0].lit && !v.fighters[1].lit);
    // Half way through the run-up: three quarters of the way there.
    let v = at(&pb, 0.45);
    assert!(close(v.fighters[0].pose.lunge, 0.75));
    assert_eq!((v.result, v.veil), (None, false));
    // The defender's turn: it is the one that runs, lit.
    let v = at(&pb, 1.38);
    assert_eq!(v.striker, Some(1));
    assert!(v.fighters[1].lit && !v.fighters[0].lit);
    assert!(v.fighters[1].pose.lunge > 0.0);
    assert_eq!(v.fighters[0].pose, Pose::default());
}

#[test]
fn a_hit_flashes_and_shakes_its_target_while_the_striker_goes_back() {
    let pb = kill();
    let v = at(&pb, 0.65);
    assert_eq!(v.striker, Some(0));
    assert_eq!(v.result, Some((1, "HIT -8".to_owned())));
    // Still at the target; the brigand white, shaken toward on step 2.
    assert!(close(v.fighters[0].pose.lunge, 1.0));
    let hit = v.fighters[1].pose;
    assert!(hit.flash);
    assert!(close(hit.shake, -(1.0 - 0.05 / motion::SHAKE_S)));
    assert_eq!((hit.lunge, hit.dodge, hit.fade), (0.0, 0.0, 0.0));
    assert!(!v.veil);
    // One HP gone after 0.05 s at 30 HP a second.
    assert_eq!(v.fighters[1].hp, 19);
    // Later in the result: going home, the flash over.
    let v = at(&pb, 0.83);
    assert!(close(v.fighters[0].pose.lunge, motion::go_back(0.23)));
    assert!(v.fighters[0].pose.lunge > 0.0 && v.fighters[0].pose.lunge < 1.0);
    assert!(!v.fighters[1].pose.flash);
    // The gap: all still, the result still up, the HP drained.
    let v = at(&pb, 1.2);
    assert_eq!(v.striker, None);
    assert_eq!(v.result, Some((1, "HIT -8".to_owned())));
    assert_eq!(
        (v.fighters[0].pose, v.fighters[1].pose),
        (Pose::default(), Pose::default())
    );
    assert_eq!(v.fighters[1].hp, 12);
}

#[test]
fn a_miss_makes_its_target_step_back() {
    let v = at(&kill(), 1.8);
    assert_eq!(v.striker, Some(1));
    assert_eq!(v.result, Some((0, "MISS".to_owned())));
    let lord = v.fighters[0].pose;
    assert!(close(lord.dodge, 1.0));
    assert!(!lord.flash && lord.shake == 0.0);
    assert!(close(v.fighters[1].pose.lunge, motion::go_back(0.15)));
    assert_eq!(v.fighters[0].hp, 19);
}

#[test]
fn a_critical_winds_up_flashes_the_stage_and_shakes_harder() {
    let pb = kill();
    // 0.12 s of a 0.3 s run-up: fully leant back.
    let v = at(&pb, 2.52);
    assert_eq!(v.striker, Some(0));
    assert!(close(v.fighters[0].pose.lunge, -motion::WIND_UP));
    assert!(!v.veil);
    let v = at(&pb, 2.73);
    assert!(v.veil);
    assert_eq!(v.result, Some((1, "CRITICAL! -27".to_owned())));
    let hit = v.fighters[1].pose;
    assert!(hit.flash);
    assert!(close(hit.shake, 2.0 * (1.0 - 0.03 / motion::SHAKE_S)));
    // The stage's flash is over long before the target's.
    let v = at(&pb, 2.85);
    assert!(!v.veil);
    assert!(v.fighters[1].pose.flash);
}

#[test]
fn a_fallen_fighter_fades_and_stays_gone() {
    let pb = kill();
    let v = at(&pb, 3.1);
    assert!(close(v.fighters[1].pose.fade, 0.0));
    assert_eq!(v.fighters[1].hp, 0);
    let v = at(&pb, 3.45);
    assert!(close(v.fighters[1].pose.fade, 0.5));
    assert!(close(v.fighters[0].pose.fade, 0.0));
    assert_eq!((v.striker, v.result, v.veil), (None, None, false));
    let v = at(&pb, 4.0);
    assert!(close(v.fighters[1].pose.fade, 1.0));
    assert_eq!(v.fighters[0].pose, Pose::default());
}

#[test]
fn the_backdrop_follows_the_ground_and_the_banner_is_shown() {
    let c = ctx();
    let pb = kill().with_banner(Some("Pinning Shot".to_owned()));
    let on = |ground| {
        let v = view(&c, &pb, ground).unwrap();
        (v.backdrop.map(trpg_content::ImageId::path), v.banner)
    };
    let banner = Some("Pinning Shot".to_owned());
    assert_eq!(
        on(Some("mountain")),
        (Some("battlebacks/mountains.png"), banner.clone())
    );
    assert_eq!(
        on(Some("plain")),
        (Some("battlebacks/fields.png"), banner.clone())
    );
    assert_eq!(on(None), (Some("battlebacks/fields.png"), banner));
}

/// A playback of one hit by `attacker` (at `to`) on `defender`.
fn one_hit(before: &[Unit], attacker: (UnitId, Pos), defender: UnitId, counter: bool) -> Playback {
    let strikes = vec![strike(Side::Attacker, true, false, 3, 10)];
    let events = fight(attacker, defender, strikes, counter);
    Playback::new(Words::ENGLISH, &events, before, &[], T).unwrap()
}

#[test]
fn the_players_fighter_stands_on_the_left_else_the_attacker() {
    let mut before = units();
    // The brigand attacks the lord: the lord is the defender, on the left.
    let v = at(
        &one_hit(&before, (BRIGAND, Pos::new(4, 5)), LORD, true),
        0.0,
    );
    assert_eq!(v.fighters[0].name, "Brigand");
    assert_eq!(v.left, 1);
    // Two of the player's own, or none: the attacker.
    before[3].faction = Faction::Player;
    let v = at(
        &one_hit(&before, (BRIGAND, Pos::new(4, 5)), LORD, true),
        0.0,
    );
    assert_eq!(v.left, 0);
    before[0].faction = Faction::Ally;
    before[3].faction = Faction::Enemy;
    let v = at(
        &one_hit(&before, (BRIGAND, Pos::new(4, 5)), LORD, true),
        0.0,
    );
    assert_eq!(v.left, 0);
    let v = at(
        &one_hit(&before, (LORD, Pos::new(7, 2)), BRIGAND, true),
        0.0,
    );
    assert_eq!(v.left, 0);
}

#[test]
fn fighters_two_tiles_apart_dont_fight_hand_to_hand() {
    let before = units();
    let near = one_hit(&before, (LORD, Pos::new(8, 3)), BRIGAND, true);
    assert!(at(&near, 0.0).melee);
    let far = one_hit(&before, (LORD, Pos::new(6, 2)), BRIGAND, false);
    let v = at(&far, 0.0);
    assert!(!v.melee);
    // And the brigand can't strike back from there.
    assert_eq!(v.fighters[1].numbers, None);
    assert!(v.fighters[0].numbers.is_some());
    let diagonal = one_hit(&before, (LORD, Pos::new(7, 3)), BRIGAND, true);
    assert!(!at(&diagonal, 0.0).melee);
}

#[test]
fn an_absorbed_strike_heals_and_moves_nobody() {
    let before = units();
    let healed = Strike {
        healed: true,
        ..strike(Side::Attacker, true, false, 4, 14)
    };
    let mut events = fight((LORD, Pos::new(7, 2)), BRIGAND, vec![healed], false);
    if let Event::CombatResolved { forecast, .. } = &mut events[1] {
        forecast.attacker.affinity = Some(Affinity::Absorb);
    }
    let pb = Playback::new(Words::ENGLISH, &events, &before, &[], T).unwrap();
    let v = at(&pb, 0.65);
    assert_eq!(v.fighters[0].numbers.map(|n| n.heals), Some(true));
    assert_eq!(v.fighters[1].pose, Pose::default());
    assert!(!v.veil);
    assert_eq!(v.result, Some((1, "HEAL +4".to_owned())));
}

#[test]
fn every_kind_of_strike_is_its_own_blow() {
    let s = |hit, crit, healed| Strike {
        healed,
        ..strike(Side::Attacker, hit, crit, 1, 1)
    };
    assert_eq!(blow(&s(false, false, false)), Blow::Miss);
    assert_eq!(blow(&s(false, true, true)), Blow::Miss);
    assert_eq!(blow(&s(true, false, false)), Blow::Hit);
    assert_eq!(blow(&s(true, true, false)), Blow::Critical);
    assert_eq!(blow(&s(true, false, true)), Blow::Heal);
    assert_eq!(blow(&s(true, true, true)), Blow::Heal);
}

#[test]
fn a_picture_is_the_characters_then_that_of_the_class_then_the_default() {
    let mut c = ctx();
    let table = &mut c.content.battlers.battlers;
    let exile = table.get("exile").cloned().unwrap();
    let named = |id: &str| Battler {
        id: id.to_owned(),
        ..exile.clone()
    };
    table.insert("heretic".to_owned(), named("heretic"));
    table.insert("lead_m".to_owned(), named("lead_m"));
    let fighter = |character: Option<&str>, class: &str| Fighter {
        unit: LORD,
        name: String::new(),
        faction: Faction::Player,
        start: 1,
        max: 1,
        character: character.map(|c| CharacterId(c.to_owned())),
        class: ClassId(class.to_owned()),
        pos: Pos::new(0, 0),
    };
    let id = |f: &Fighter| picture(&c, f).map(|p| p.id);
    // Rue has her own, whatever her class.
    assert_eq!(
        id(&fighter(Some("heretic"), "mage")).as_deref(),
        Some("heretic")
    );
    // The lead's is by gender (male by default), before the class's.
    assert_eq!(
        id(&fighter(Some("lead"), "exile")).as_deref(),
        Some("lead_m")
    );
    // Hollis has none: his class's.
    assert_eq!(
        id(&fighter(Some("retainer"), "guard")).as_deref(),
        Some("guard")
    );
    assert_eq!(id(&fighter(None, "raider")).as_deref(), Some("raider"));
    // A class with no picture: the default one.
    assert_eq!(
        id(&fighter(None, "no_such_class")).as_deref(),
        Some("default")
    );
    assert_eq!(
        id(&fighter(Some("nobody"), "no_such_class")).as_deref(),
        Some("default")
    );
}

#[test]
fn no_combat_no_stage() {
    let c = ctx();
    let mut pb = kill();
    assert!(view(&c, &pb, None).is_some());
    // At the very end the last combat still shows.
    pb.skip();
    assert!(view(&c, &pb, None).is_some());
}
