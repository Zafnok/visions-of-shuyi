//! Tests of the art rules: effects on a weapon and mods, notes, and which
//! arts a unit knows and can use (rank, class, weapon arts, cost).

use std::collections::BTreeMap;

use super::*;
use crate::class::{ClassDef, ClassId, UnitTags, WeaponProficiency};
use crate::combat::DamageType;
use crate::geom::Pos;
use crate::item::{ItemDef, ItemId, Loadout, WeaponDef, WeaponInstance};
use crate::magic::Element;
use crate::stats::{Growths, Stats};
use crate::terrain::MovementTypeId;
use crate::unit::{Faction, UnitId};

fn aid(id: &str) -> ArtId {
    ArtId::new(id)
}

fn art(id: &str, kind: WeaponKind, rank: Option<WeaponRank>, cost: u32) -> ArtDef {
    ArtDef {
        id: aid(id),
        name: id.into(),
        kind,
        rank,
        cost,
        effect: ArtEffect::default(),
    }
}

/// Sword arts `cut` (E, 2), `break` (D, 4), `finale` (A, 1); spear art
/// `thrust` (E, 2); weapon art `moonlight` (sword, 3).
fn arts() -> ArtTable {
    use WeaponKind::{Spear, Sword};
    use WeaponRank::{A, D, E};
    let all = [
        art("cut", Sword, Some(E), 2),
        art("break", Sword, Some(D), 4),
        art("finale", Sword, Some(A), 1),
        art("thrust", Spear, Some(E), 2),
        art("moonlight", Sword, None, 3),
    ];
    ArtTable {
        arts: all.into_iter().map(|a| (a.id.clone(), a)).collect(),
    }
}

fn weapon(kind: WeaponKind, rank: WeaponRank, arts: &[&str]) -> ItemDef {
    ItemDef::Weapon(WeaponDef {
        name: "W".into(),
        kind,
        rank,
        might: 5,
        hit: 90,
        crit: 0,
        weight: 2,
        min_range: 1,
        max_range: 1,
        damage_type: DamageType::Physical,
        durability: 20,
        effective: vec![],
        arts: arts.iter().map(|a| aid(a)).collect(),
        price: 0,
    })
}

/// `sword` (E), `moon_blade` (a sword with `moonlight`, rank C), `spear`.
fn items() -> ItemTable {
    ItemTable {
        items: BTreeMap::from([
            (
                ItemId::new("sword"),
                weapon(WeaponKind::Sword, WeaponRank::E, &[]),
            ),
            (
                ItemId::new("moon_blade"),
                weapon(WeaponKind::Sword, WeaponRank::C, &["moonlight"]),
            ),
            (
                ItemId::new("spear"),
                weapon(WeaponKind::Spear, WeaponRank::E, &[]),
            ),
        ]),
        ..ItemTable::default()
    }
}

fn class(id: &str, kinds: &[WeaponKind]) -> ClassDef {
    ClassDef {
        id: ClassId(id.into()),
        name: id.into(),
        tier: 1,
        movement_type: MovementTypeId(0),
        move_points: 5,
        base: Stats::default(),
        growths: Growths::default(),
        weapons: kinds
            .iter()
            .map(|&kind| WeaponProficiency {
                kind,
                start: WeaponRank::E,
                max: WeaponRank::S,
            })
            .collect(),
        max_armour: None,
        tags: UnitTags::default(),
        promotes_to: vec![],
        active: None,
        passives: vec![],
        enemy_only: false,
        lord_only: false,
        weapon_slots: 3,
        spells: vec![],
        affinities: vec![],
    }
}

/// `swordsman` (swords), `soldier` (spears), `sage` (nothing).
fn classes() -> ClassTable {
    let all = [
        class("swordsman", &[WeaponKind::Sword]),
        class("soldier", &[WeaponKind::Spear]),
        class("sage", &[]),
    ];
    ClassTable {
        classes: all.into_iter().map(|c| (c.id.clone(), c)).collect(),
        ..ClassTable::default()
    }
}

/// A Swordsman at sword rank `rank` carrying `weapons` (durability 20).
fn swordsman(rank: WeaponRank, weapons: &[&str]) -> Unit {
    let mut u = Unit::generic(
        UnitId(1),
        &ClassId("swordsman".into()),
        &classes(),
        1,
        Faction::Player,
        Pos::new(0, 0),
    )
    .unwrap_or_else(|e| panic!("{e}"));
    u.weapon_ranks.insert(WeaponKind::Sword, rank);
    let mut loadout = Loadout::default();
    for (slot, w) in weapons.iter().enumerate() {
        loadout.weapons[slot] = Some(WeaponInstance {
            def: ItemId::new(w),
            durability_left: 20,
        });
    }
    u.loadout = loadout;
    u
}

fn ids(arts: &[&ArtDef]) -> Vec<String> {
    arts.iter().map(|a| a.id.0.clone()).collect()
}

fn arts_for(u: &Unit, slot: usize) -> Vec<String> {
    ids(&u.arts_for(slot, &classes(), &items(), &arts()))
}

fn usable(u: &Unit, slot: usize) -> Vec<String> {
    ids(&u.usable_arts(slot, &classes(), &items(), &arts()))
}

fn stats() -> WeaponStats {
    WeaponStats {
        kind: Some(WeaponKind::Axe),
        trait_: WeaponTrait::AxeMinDamage(5),
        might: 8,
        hit: 75,
        crit: 0,
        weight: 6,
        min_range: 2,
        max_range: 2,
        damage_type: DamageType::Physical,
        effective: vec![(UnitTag::Flying, 3)],
        broken: false,
        element: Element::None,
    }
}

// ---- Effects ---------------------------------------------------------------

#[test]
fn the_default_effect_changes_nothing() {
    let (mut w, mut m) = (stats(), CombatMods::default());
    ArtEffect::default().apply(&mut w, &mut m, 30);
    assert_eq!((w, m), (stats(), CombatMods::default()));
    assert_eq!(ArtEffect::default().notes(), []);
}

#[test]
fn an_effect_changes_the_weapon_and_the_mods() {
    let effect = ArtEffect {
        hit: -10,
        sword_followup: Some((3, 2)),
        effective: vec![(UnitTag::Armored, 2)],
        axe_min_damage: Some(8),
        min_range: Some(1),
        ..ArtEffect::default()
    };
    let mut w = stats();
    let mut m = CombatMods {
        hit: 4,
        ..CombatMods::default()
    };
    effect.apply(&mut w, &mut m, 30);
    assert_eq!(
        m,
        CombatMods {
            hit: -6,
            sword_followup: Some((3, 2)),
            ..CombatMods::default()
        }
    );
    assert_eq!(
        w,
        WeaponStats {
            trait_: WeaponTrait::AxeMinDamage(8),
            min_range: 1,
            effective: vec![(UnitTag::Flying, 3), (UnitTag::Armored, 2)],
            ..stats()
        }
    );
}

#[test]
fn an_effect_without_a_follow_up_keeps_the_mods_one() {
    let mut m = CombatMods {
        sword_followup: Some((1, 1)),
        ..CombatMods::default()
    };
    ArtEffect::default().apply(&mut stats(), &mut m, 30);
    assert_eq!(m.sword_followup, Some((1, 1)));
}

#[test]
fn notes_spell_out_the_effects_that_are_not_numbers() {
    let slow = Debuff {
        stat: StatKind::Spd,
        amount: 3,
    };
    let stance = TimedMods {
        stats: vec![],
        combat: CombatMods {
            avoid: 20,
            ..CombatMods::default()
        },
    };
    let effect = ArtEffect {
        hit: 10,
        no_counter: true,
        line_pierce: true,
        on_first_hit: vec![slow],
        on_first_hit_avoid: 20,
        stance: Some(stance.clone()),
        ..ArtEffect::default()
    };
    assert_eq!(
        effect.notes(),
        [
            ArtNote::NoCounter,
            ArtNote::Pierces,
            ArtNote::Debuff(slow),
            ArtNote::AvoidDebuff(20),
            ArtNote::Stance(stance),
        ]
    );
}

#[test]
fn a_debuff_is_a_negative_stat_bonus() {
    let pin = Debuff {
        stat: StatKind::Mov,
        amount: 3,
    };
    assert_eq!(
        pin.mods(),
        TimedMods {
            stats: vec![(StatKind::Mov, -3)],
            combat: CombatMods::default(),
        }
    );
}

fn applied(effect: &ArtEffect, target_max_hp: StatValue) -> (WeaponStats, CombatMods) {
    let (mut w, mut m) = (stats(), CombatMods::default());
    effect.apply(&mut w, &mut m, target_max_hp);
    (w, m)
}

#[test]
fn long_thrust_and_hurl_reach_two_at_minus_ten_hit() {
    let effect = ArtEffect {
        hit: -10,
        min_range: Some(1),
        max_range: Some(2),
        ..ArtEffect::default()
    };
    let (w, m) = applied(&effect, 30);
    assert_eq!((w.min_range, w.max_range, m.hit), (1, 2, -10));
    assert!(w.in_range(1) && w.in_range(2) && !w.in_range(3));
}

#[test]
fn skull_splitter_adds_crit_and_cascade_is_a_mod() {
    let effect = ArtEffect {
        crit: 30,
        cascade: 3,
        ..ArtEffect::default()
    };
    let (_, m) = applied(&effect, 30);
    assert_eq!((m.crit, m.cascade), (30, 3));
}

#[test]
fn felling_blow_takes_a_third_of_the_targets_max_hp_over_the_axe_minimum() {
    let effect = ArtEffect {
        axe_min_damage_max_hp_div: Some(3),
        ..ArtEffect::default()
    };
    let min = |hp| applied(&effect, hp).0.trait_;
    assert_eq!(min(60), WeaponTrait::AxeMinDamage(20));
    assert_eq!(min(41), WeaponTrait::AxeMinDamage(13));
    // A small target keeps the axe's own minimum.
    assert_eq!(min(12), WeaponTrait::AxeMinDamage(5));
}

#[test]
fn a_first_hit_debuff_lists_stats_and_avoid() {
    let effect = ArtEffect {
        on_first_hit: vec![
            Debuff {
                stat: StatKind::Def,
                amount: 4,
            },
            Debuff {
                stat: StatKind::Spd,
                amount: 4,
            },
        ],
        on_first_hit_avoid: 20,
        ..ArtEffect::default()
    };
    assert_eq!(
        effect.first_hit_mods(),
        Some(TimedMods {
            stats: vec![(StatKind::Def, -4), (StatKind::Spd, -4)],
            combat: CombatMods {
                avoid_debuff: 20,
                ..CombatMods::default()
            },
        })
    );
    assert_eq!(ArtEffect::default().first_hit_mods(), None);
}

#[test]
fn old_and_new_debuff_data_both_load() {
    let load = |s: &str| ron::from_str::<ArtEffect>(s).map(|e| e.on_first_hit);
    let mov = Debuff {
        stat: StatKind::Mov,
        amount: 3,
    };
    assert_eq!(
        load("(on_first_hit: Some((stat: Mov, amount: 3)))"),
        Ok(vec![mov])
    );
    assert_eq!(
        load("(on_first_hit: [(stat: Mov, amount: 3)])"),
        Ok(vec![mov])
    );
    assert_eq!(load("(on_first_hit: None)"), Ok(vec![]));
    assert_eq!(load("()"), Ok(vec![]));
    let error = load("(on_first_hit: 3)").unwrap_err().to_string();
    assert!(error.contains("a debuff or a list of debuffs"), "{error}");
}

#[test]
fn an_art_costs_durability() {
    assert_eq!(
        art("x", WeaponKind::Bow, None, 3).skill_cost(),
        SkillCost::Durability(3)
    );
}

// ---- Learning and using ----------------------------------------------------

#[test]
fn rank_arts_are_learned_by_rank() {
    assert_eq!(arts_for(&swordsman(WeaponRank::E, &["sword"]), 0), ["cut"]);
    assert_eq!(
        arts_for(&swordsman(WeaponRank::D, &["sword"]), 0),
        ["cut", "break"]
    );
    // Lowest rank first, whatever the ids.
    assert_eq!(
        arts_for(&swordsman(WeaponRank::S, &["sword"]), 0),
        ["cut", "break", "finale"]
    );
}

#[test]
fn a_weapon_art_comes_with_its_weapon_whatever_the_rank() {
    let u = swordsman(WeaponRank::C, &["sword", "moon_blade"]);
    assert_eq!(arts_for(&u, 0), ["cut", "break"]);
    assert_eq!(arts_for(&u, 1), ["cut", "break", "moonlight"]);
}

#[test]
fn no_arts_without_a_weapon_the_unit_can_wield() {
    let u = swordsman(WeaponRank::D, &["sword", "spear", "moon_blade"]);
    // Empty slot.
    assert_eq!(arts_for(&u, 2), Vec::<String>::new());
    // A spear, which Swordsmen can't wield.
    assert_eq!(arts_for(&u, 1), Vec::<String>::new());
    // Rank D can't wield a rank-C blade, so no weapon art either.
    let low = swordsman(WeaponRank::D, &["moon_blade"]);
    assert_eq!(arts_for(&low, 0), Vec::<String>::new());
    // Unknown class.
    let lost = Unit {
        class: ClassId("nope".into()),
        ..swordsman(WeaponRank::D, &["sword"])
    };
    assert_eq!(arts_for(&lost, 0), Vec::<String>::new());
}

#[test]
fn after_a_reclass_arts_are_known_but_unusable() {
    let mut u = swordsman(WeaponRank::D, &["sword"]);
    u.class = ClassId("sage".into());
    assert_eq!(ids(&u.known_arts(&arts())), ["cut", "break"]);
    assert_eq!(arts_for(&u, 0), Vec::<String>::new());
    // Back in a sword class, they are usable again.
    u.class = ClassId("swordsman".into());
    assert_eq!(arts_for(&u, 0), ["cut", "break"]);
}

#[test]
fn known_arts_follow_recorded_ranks_only() {
    let mut u = swordsman(WeaponRank::E, &[]);
    u.weapon_ranks = BTreeMap::from([(WeaponKind::Sword, WeaponRank::D)]);
    assert_eq!(ids(&u.known_arts(&arts())), ["cut", "break"]);
    u.weapon_ranks.insert(WeaponKind::Spear, WeaponRank::E);
    assert_eq!(ids(&u.known_arts(&arts())), ["cut", "break", "thrust"]);
    u.weapon_ranks.clear();
    assert_eq!(ids(&u.known_arts(&arts())), Vec::<String>::new());
}

#[test]
fn every_art_is_usable_until_the_weapon_breaks() {
    let mut u = swordsman(WeaponRank::D, &["sword"]);
    assert_eq!(usable(&u, 0), ["cut", "break"]);
    let set = |u: &mut Unit, left| {
        if let Some(Some(w)) = u.loadout.weapons.first_mut() {
            w.durability_left = left;
        }
    };
    // Less left than the cost is fine: the rest is spent (Nick, 0414).
    for left in [4, 3, 1] {
        set(&mut u, left);
        assert_eq!(usable(&u, 0), ["cut", "break"], "{left}");
    }
    // Broken: none, though every art is still for this weapon.
    set(&mut u, 0);
    assert_eq!(usable(&u, 0), Vec::<String>::new());
    assert_eq!(arts_for(&u, 0), ["cut", "break"]);
}

#[test]
fn the_table_finds_arts_by_id() {
    let t = arts();
    assert_eq!(t.get(&aid("cut")).map(|a| a.cost), Some(2));
    assert_eq!(t.get(&aid("nope")), None);
}
