//! Tests of the combat shot (ticket 0832), on the test battle (the combat
//! test cinematic, `assets-test/cinematics/combat.ron`): the lead at
//! (3, 5) and the knight at (3, 6), the brigand the battle file calls
//! `road_brigand` at (8, 3) on the plain; a wall at (9, 4).

use trpg_core::{
    CastTarget, CombatOutcome, Equipped, Event, LeadGender, LeadProfile, Pos, Side, UnitAction,
    UnitId, lead::DEFAULT_NAME,
};

use super::*;

const LEAD: UnitId = UnitId(1);
const KNIGHT: UnitId = UnitId(2);
const BRIGAND: UnitId = UnitId(3);

fn lead() -> LeadProfile {
    LeadProfile::new(DEFAULT_NAME, LeadGender::Male)
}

/// The lead attacking the brigand from beside it, on the test battle's
/// own luck: the test cinematic's combat shot.
fn shot() -> CombatShot {
    CombatShot {
        battle: "test".to_owned(),
        attacker: LEAD,
        target: BRIGAND,
        from: Some(Pos::new(7, 3)),
        seed: None,
    }
}

/// The combat `fight` fought: who attacked, who was attacked and how it
/// went.
fn combat(fight: &Fight) -> (UnitId, UnitId, &CombatOutcome) {
    let resolved = fight.events.iter().find_map(|e| match e {
        Event::CombatResolved {
            attacker,
            defender,
            outcome,
            ..
        } => Some((*attacker, *defender, outcome)),
        _ => None,
    });
    resolved.expect("a combat")
}

#[test]
fn the_combat_test_cinematic_is_the_lead_attacking_the_brigand() {
    let c = content();
    let test = &c.cinematics["combat"];
    assert_eq!((test.music.as_deref(), test.length_ms), (None, 6000));
    let fight = ShotDef {
        at_ms: 0,
        shot: Shot::Combat(shot()),
    };
    assert_eq!(test.shots, [fight]);
}

#[test]
fn a_combat_shot_names_two_units_of_its_battle() {
    let c = content();
    let shot_of = |written: &str| load(&c, &one(written)).unwrap().shots[0].shot.clone();
    let written = "Combat(battle: \"test\", attacker: \"lead\", target: \"road_brigand\")";
    let plain = CombatShot {
        from: None,
        ..shot()
    };
    assert_eq!(shot_of(written), Shot::Combat(plain.clone()));
    // The tile it strikes from and the luck are written bare.
    let written = "Combat(battle: \"test\", attacker: \"road_brigand\", target: \"test_knight\", \
                   from: (3, 7), seed: 12)";
    let placed = CombatShot {
        attacker: BRIGAND,
        target: KNIGHT,
        from: Some(Pos::new(3, 7)),
        seed: Some(12),
        ..plain
    };
    assert_eq!(shot_of(written), Shot::Combat(placed));
}

#[test]
fn a_combat_shot_with_a_wrong_name_or_tile_is_refused() {
    let c = content();
    let wrong = |written: &str| problems(&c, &one(written));
    assert_eq!(
        wrong("Combat(battle: \"nope\", attacker: \"lead\", target: \"road_brigand\")"),
        ["shot 1: no battle \"nope\""]
    );
    // Both names are checked; a unit that arrives later isn't there.
    assert_eq!(
        wrong("Combat(battle: \"test\", attacker: \"test_lord\", target: \"brigand\")"),
        [
            "shot 1: no unit \"test_lord\" on the map of battle \"test\" at its start",
            "shot 1: no unit \"brigand\" on the map of battle \"test\" at its start",
        ]
    );
    assert_eq!(
        wrong("Combat(battle: \"test\", attacker: \"lead\", target: \"lead\")"),
        ["shot 1: \"lead\" can't attack itself"]
    );
    assert_eq!(
        wrong(
            "Combat(battle: \"test\", attacker: \"lead\", target: \"road_brigand\", from: (14, 3))"
        ),
        ["shot 1: from (14, 3) is not on the map of battle \"test\" (14×8 tiles)"]
    );
    // A tile the attacker could never stand on: water, or another unit's.
    let from = |tile: &str| {
        let written = format!(
            "Combat(battle: \"test\", attacker: \"lead\", target: \"road_brigand\", from: {tile})"
        );
        wrong(&written)
    };
    assert_eq!(
        from("(9, 4)"),
        ["shot 1: from (9, 4): \"lead\" (Exile) can't stand there"]
    );
    assert_eq!(
        from("(3, 6)"),
        ["shot 1: from (3, 6) is where \"test_knight\" starts"]
    );
    // Its own tile is fine (then the rules say it is out of range).
    let own =
        "Combat(battle: \"test\", attacker: \"lead\", target: \"road_brigand\", from: (3, 5))";
    assert!(load(&c, &one(own)).is_ok());
    // The file says nothing about how it goes: there is no field for it.
    let told =
        wrong("Combat(battle: \"test\", attacker: \"lead\", target: \"road_brigand\", strikes: 2)");
    assert!(told[0].contains("strikes"), "{told:?}");
}

#[test]
fn the_rules_work_the_fight_out_from_the_battles_start() {
    let c = content();
    let fight = fight(&c, &shot(), lead()).unwrap();
    // The lead was put beside the brigand, everyone else where they start.
    let at = |id| fight.before.iter().find(|u| u.id == id).unwrap().pos;
    assert_eq!(
        [at(LEAD), at(KNIGHT), at(BRIGAND)],
        [Pos::new(7, 3), Pos::new(3, 6), Pos::new(8, 3)]
    );
    let (attacker, defender, outcome) = combat(&fight);
    assert_eq!((attacker, defender), (LEAD, BRIGAND));
    assert_eq!(outcome.strikes[0].by, Side::Attacker);
    // What the rules left them with is the battle's after the attack.
    let hp = |id| {
        let standing = fight.after.unit(id).map(|u| u.hp);
        standing.unwrap_or_else(|| fight.after.fallen().iter().find(|u| u.id == id).unwrap().hp)
    };
    assert_eq!(hp(LEAD), outcome.attacker_hp);
    assert_eq!(hp(BRIGAND), outcome.defender_hp);
    // The same shot is the same fight.
    assert_eq!(
        fight.events,
        super::fight(&c, &shot(), lead()).unwrap().events
    );
}

#[test]
fn the_luck_is_the_battles_own_unless_the_shot_names_a_seed() {
    let c = content();
    let seeded = |seed| {
        let shot = CombatShot { seed, ..shot() };
        fight(&c, &shot, lead()).unwrap().events
    };
    let own = c.battles["test"].seed;
    assert_eq!(seeded(None), seeded(Some(own)));
    // Some other seed rolls otherwise.
    let rolls = |seed| {
        combat(&fight(&c, &CombatShot { seed, ..shot() }, lead()).unwrap())
            .2
            .clone()
    };
    let first = rolls(Some(0));
    assert!((1..40).any(|s| rolls(Some(s)) != first));
}

#[test]
fn an_enemy_attacks_in_its_own_phase() {
    let c = content();
    let shot = CombatShot {
        attacker: BRIGAND,
        target: LEAD,
        from: Some(Pos::new(4, 5)),
        ..shot()
    };
    let fight = fight(&c, &shot, lead()).unwrap();
    assert_eq!(combat(&fight).0, BRIGAND);
    assert_eq!(fight.after.phase(), trpg_core::Phase::Enemy);
}

#[test]
fn a_fight_the_rules_refuse_fails_the_content_check() {
    let c = content();
    let refused = |shot: CombatShot| fight(&c, &shot, lead()).unwrap_err();
    // Where it starts, the lead can't reach the brigand.
    assert_eq!(
        refused(CombatShot {
            from: None,
            ..shot()
        }),
        "the rules refuse the attack: unit 3 is out of range (7 tiles)"
    );
    assert_eq!(
        refused(CombatShot {
            target: KNIGHT,
            from: None,
            ..shot()
        }),
        "the rules refuse the attack: unit 2 is not an enemy"
    );
    // Loaded with the rest, such a shot is an error of its file.
    let mut broken = c.clone();
    let mut def = broken.cinematics["combat"].clone();
    def.shots[0].shot = Shot::Combat(CombatShot {
        from: None,
        ..shot()
    });
    broken.cinematics.insert("broken".to_owned(), def);
    let errors = check_fights(broken).unwrap_err().0;
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert!(
        errors[0].file.ends_with("cinematics/broken.ron"),
        "{}",
        errors[0].file
    );
    assert_eq!(
        errors[0].message,
        "shot 1: the rules refuse the attack: unit 3 is out of range (7 tiles)"
    );
    assert!(check_fights(c).is_ok());
}

#[test]
fn an_attacker_tries_what_it_has_equipped_first_then_its_other_weapons() {
    let c = content();
    let fight = fight(&c, &shot(), lead()).unwrap();
    let mut unit = fight.before.iter().find(|u| u.id == LEAD).unwrap().clone();
    let weapon = unit
        .loadout
        .weapons
        .iter()
        .flatten()
        .next()
        .unwrap()
        .clone();
    let attack = |slot| UnitAction::Attack {
        target: BRIGAND,
        slot,
        active: None,
        art: None,
    };
    // Two weapons, the second equipped: it first, then the other.
    unit.loadout.weapons = [Some(weapon.clone()), None, Some(weapon)];
    unit.loadout.equipped = Some(Equipped::Weapon(2));
    assert_eq!(
        crate::cinematic::combat::attacks(&unit, BRIGAND),
        [attack(2), attack(0)]
    );
    // Nothing equipped: each weapon, first slot first.
    unit.loadout.equipped = None;
    assert_eq!(
        crate::cinematic::combat::attacks(&unit, BRIGAND),
        [attack(0), attack(2)]
    );
    // A spell equipped: cast at the target first.
    let spell = trpg_core::SpellId("fire".to_owned());
    unit.loadout.equipped = Some(Equipped::Spell(spell.clone()));
    let cast = UnitAction::Cast {
        spell,
        target: CastTarget::Unit(BRIGAND),
        active: None,
    };
    assert_eq!(
        crate::cinematic::combat::attacks(&unit, BRIGAND),
        [cast, attack(0), attack(2)]
    );
    // Nothing to attack with: nothing to try.
    unit.loadout.weapons = Default::default();
    unit.loadout.equipped = None;
    assert_eq!(
        crate::cinematic::combat::attacks(&unit, BRIGAND),
        [] as [trpg_core::UnitAction; 0]
    );
}
