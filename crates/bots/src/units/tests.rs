use trpg_core::{BattleState, GameMode, LeadGender, LeadProfile, Pos, UnitId};

use super::*;
use crate::testkit::{content, enemy, player, start};

/// `unit` with Def and Res out of reach.
fn armoured(mut unit: trpg_core::Unit) -> trpg_core::Unit {
    unit.stats.def = 99;
    unit.stats.res = 99;
    unit
}

#[test]
fn can_hurt_counts_the_enemies_a_strike_hurts() {
    let state = start(vec![
        player(1, 4),
        enemy(2, 5),
        armoured(enemy(3, 6)),
        enemy(4, 0),
    ]);
    assert_eq!(can_hurt(&state, UnitId(1)), (2, 3));
}

#[test]
fn can_hurt_counts_nobody_for_a_unit_off_the_map_or_no_enemies() {
    let state = start(vec![player(1, 4), enemy(2, 5)]);
    assert_eq!(can_hurt(&state, UnitId(9)), (0, 0));
    // An enemy's enemies are the player's units; nobody counts itself.
    assert_eq!(can_hurt(&state, UnitId(2)), (1, 1));
    let alone = start(vec![player(1, 4)]);
    assert_eq!(can_hurt(&alone, UnitId(1)), (0, 0));
}

#[test]
fn can_hurt_tries_every_weapon_the_unit_carries() {
    // A swordsman with nothing equipped still counts the sword it carries.
    let mut unarmed = player(1, 4);
    unarmed.loadout.equipped = None;
    let state = start(vec![unarmed, enemy(2, 5)]);
    assert_eq!(can_hurt(&state, UnitId(1)), (1, 1));
}

#[test]
fn can_hurt_tries_every_attack_spell_the_unit_knows() {
    // A mage with no weapon and nothing equipped hurts a knight of Def 99
    // and Res 0 with its Fire, and a target with both out of reach with
    // nothing.
    let content = content();
    let class = trpg_core::ClassId("mage".into());
    let mut mage = trpg_core::Unit::generic(
        UnitId(1),
        &class,
        &content.classes,
        1,
        trpg_core::Faction::Player,
        crate::testkit::tile(4),
    )
    .unwrap();
    mage.loadout.equipped = None;
    let mut knight = enemy(2, 5);
    knight.stats.def = 99;
    knight.stats.res = 0;
    let state = start(vec![mage, knight, armoured(enemy(3, 6))]);
    assert_eq!(
        enemies_hurt(&state, UnitId(1)),
        [(UnitId(2), true), (UnitId(3), false)]
    );
}

/// The acceptance check (ADR-0118 (a), a test of the content): at
/// Chapter 2's start, Aske's bow can't hurt the two hound guards (Def 15)
/// and Rue can.
#[test]
fn at_chapter_2_aske_cant_hurt_the_hound_guards_and_rue_can() {
    if trpg_content::bundle::skip_without_the_game() {
        return;
    }
    let content = content();
    let def = &content.battles["ch02"];
    let lead = LeadProfile::new("Lead", LeadGender::Male);
    let setup = crate::arrival(content, def, GameMode::Classic, lead)
        .unwrap()
        .unwrap_or_else(|| panic!("no army"));
    let state = BattleState::new(setup).0;
    let id = |name: &str| {
        let unit = state.units().iter().find(|u| u.name == name);
        unit.unwrap_or_else(|| panic!("no {name}")).id
    };
    let at = |pos: Pos| {
        let unit = state.units().iter().find(|u| u.pos == pos);
        unit.unwrap_or_else(|| panic!("no guard at {pos:?}")).id
    };
    let guards = [at(Pos::new(8, 6)), at(Pos::new(10, 6))];
    let hurts = |unit, enemy| {
        let all = enemies_hurt(&state, unit);
        all.iter().any(|&(e, hurt)| e == enemy && hurt)
    };
    for guard in guards {
        assert!(!hurts(id("Aske"), guard), "Aske hurts {guard:?}");
        assert!(hurts(id("Rue"), guard), "Rue can't hurt {guard:?}");
    }
    let (aske, of) = can_hurt(&state, id("Aske"));
    let (rue, _) = can_hurt(&state, id("Rue"));
    assert!(
        aske + 2 <= of && rue > aske,
        "Aske {aske}/{of}, Rue {rue}/{of}"
    );
}

/// A unit at the army's level, earning an even share, that can hurt all.
fn even() -> Standing {
    Standing {
        can_hurt: (6, 6),
        level: 5,
        army_level: 5,
        exp: 25,
        army_exp: 100,
        army_size: 4,
    }
}

#[test]
fn an_even_unit_has_no_flags() {
    assert_eq!(flags(&even()), []);
}

#[test]
fn can_hurt_flags_under_two_thirds() {
    let with = |hurt, of| {
        flags(&Standing {
            can_hurt: (hurt, of),
            ..even()
        })
    };
    assert_eq!(with(4, 6), [], "2/3 exactly");
    assert_eq!(with(3, 6), [Flag::CantHurt]);
    assert_eq!(with(2, 3), []);
    assert_eq!(with(1, 2), [Flag::CantHurt]);
    assert_eq!(with(0, 1), [Flag::CantHurt]);
    assert_eq!(with(0, 0), [], "no enemies");
}

#[test]
fn level_flags_three_or_more_below_the_army() {
    let at = |level| flags(&Standing { level, ..even() });
    assert_eq!(at(3), [], "2 below");
    assert_eq!(at(2), [Flag::Behind], "3 below");
    assert_eq!(at(1), [Flag::Behind]);
    assert_eq!(at(9), [], "above");
}

#[test]
fn exp_flags_under_half_an_even_share() {
    // An even share of 100 among 4 is 25: half is 12.5.
    let earned = |exp| flags(&Standing { exp, ..even() });
    assert_eq!(earned(13), []);
    assert_eq!(earned(12), [Flag::LowExp]);
    assert_eq!(earned(0), [Flag::LowExp]);
    let none = Standing {
        exp: 0,
        army_exp: 0,
        ..even()
    };
    assert_eq!(flags(&none), [], "no EXP earned");
    // Exactly half (army of 2, 100 EXP, 25 earned) isn't under it.
    let half = Standing {
        army_size: 2,
        ..even()
    };
    assert_eq!(flags(&half), []);
    // An army of no units flags no EXP.
    let nobody = Standing {
        exp: 0,
        army_size: 0,
        ..even()
    };
    assert_eq!(flags(&nobody), []);
}

#[test]
fn flags_come_in_order_with_their_text() {
    let all = Standing {
        can_hurt: (0, 3),
        level: 1,
        exp: 0,
        ..even()
    };
    assert_eq!(flags(&all), [Flag::CantHurt, Flag::Behind, Flag::LowExp]);
    let text: Vec<&str> = flags(&all).into_iter().map(Flag::text).collect();
    assert_eq!(
        text,
        [
            "can hurt < 2/3",
            "lv 3+ below the army",
            "exp < half a share"
        ]
    );
}
