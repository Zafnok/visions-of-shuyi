//! Tests of the art file: the embedded data matches `combat-arts.md`, and
//! every validation rule reports its error.

use trpg_core::{CombatMods, Debuff, StatKind, TimedMods, UnitTag};

use super::*;

fn embedded() -> ArtTable {
    load().unwrap_or_else(|e| panic!("{e:?}"))
}

/// The default effect changed by `f`.
fn with(f: impl FnOnce(&mut ArtEffect)) -> ArtEffect {
    let mut e = ArtEffect::default();
    f(&mut e);
    e
}

fn get(t: &ArtTable, id: &str) -> ArtDef {
    t.get(&ArtId(id.into()))
        .cloned()
        .unwrap_or_else(|| panic!("no art {id}"))
}

/// `combat-arts.md`, *Chapter 1 arts*: every art's name, kind, rank and
/// cost.
#[test]
fn the_embedded_arts_match_the_design_table() {
    use WeaponKind::{Axe, Bow, Gauntlet, Spear, Sword};
    use WeaponRank::{A, B, C, D, E, S};
    let rows = [
        ("flowing_cut", "Flowing Cut", Sword, E, 2),
        ("guard_break", "Guard Break", Sword, D, 4),
        ("unhorse", "Unhorse", Spear, E, 2),
        ("line_pierce", "Line Pierce", Spear, D, 4),
        ("crushing_swing", "Crushing Swing", Axe, E, 2),
        ("armor_cleave", "Armor Cleave", Axe, D, 4),
        ("close_shot", "Close Shot", Bow, E, 2),
        ("pinning_shot", "Pinning Shot", Bow, D, 3),
        ("pressure_point", "Pressure Point", Gauntlet, E, 2),
        ("sidestep", "Sidestep", Gauntlet, D, 3),
    ];
    let t = embedded();
    // The ranks C-S arts that change the numbers (`combat-arts.md`, ticket 0325).
    let later = [
        ("cascade", "Cascade", Sword, A, 6),
        ("long_thrust", "Long Thrust", Spear, C, 3),
        ("skypiercer", "Skypiercer", Spear, A, 5),
        ("hurl", "Hurl", Axe, B, 4),
        ("skull_splitter", "Skull Splitter", Axe, A, 5),
        ("felling_blow", "Felling Blow", Axe, S, 8),
        ("rending_arrow", "Rending Arrow", Bow, C, 4),
        ("off_balance", "Off Balance", Gauntlet, C, 3),
        ("vital_point", "Vital Point", Gauntlet, S, 8),
    ];
    let rows: Vec<_> = rows
        .into_iter()
        .chain(later)
        .map(|(id, name, kind, rank, cost)| (id, name, kind, Some(rank), cost))
        .collect();
    // The special weapons' own arts (`combat-arts.md`, *Special weapons*).
    let weapon_arts = [
        ("pack_hunt", "Pack Hunt", Axe, None, 3),
        ("hold_the_ford", "Hold the Ford", Spear, None, 3),
        ("moonwave", "Moonwave", Sword, None, 4),
    ];
    assert_eq!(t.arts.len(), rows.len() + weapon_arts.len());
    for (id, name, kind, rank, cost) in rows.into_iter().chain(weapon_arts) {
        let def = get(&t, id);
        assert_eq!(
            (def.name.as_str(), def.kind, def.rank, def.cost),
            (name, kind, rank, cost),
            "{id}"
        );
    }
}

/// `combat-arts.md`, *Special weapons*: the weapon arts' effects.
#[test]
fn the_embedded_weapon_art_effects_match_the_design() {
    use trpg_core::{MitigationStat, PowerStat};
    let t = embedded();
    assert_eq!(
        get(&t, "pack_hunt").effect,
        with(|e| e.pack_might = Some((2, 6)))
    );
    assert_eq!(
        get(&t, "hold_the_ford").effect,
        with(|e| {
            e.stance = Some(TimedMods {
                stops_enemies: true,
                ..TimedMods::default()
            });
        })
    );
    assert_eq!(
        get(&t, "moonwave").effect,
        with(|e| {
            e.max_range = Some(2);
            e.power = Some(PowerStat::Higher);
            e.mitigation = Some(MitigationStat::Res);
        })
    );
}

/// `combat-arts.md`, *Chapter 1 arts*: every art's effect.
#[test]
fn the_embedded_art_effects_match_the_design_table() {
    let debuff = |stat, amount| vec![Debuff { stat, amount }];
    let sidestep = TimedMods {
        stats: vec![],
        combat: CombatMods {
            avoid: 20,
            ..CombatMods::default()
        },
        stops_enemies: false,
    };
    let rows = [
        ("flowing_cut", with(|e| e.sword_followup = Some((3, 2)))),
        (
            "guard_break",
            with(|e| {
                e.hit = 10;
                e.no_counter = true;
            }),
        ),
        (
            "unhorse",
            with(|e| {
                e.hit = 10;
                e.effective = vec![(UnitTag::Mounted, 3)];
            }),
        ),
        ("line_pierce", with(|e| e.line_pierce = true)),
        (
            "crushing_swing",
            with(|e| {
                e.hit = 20;
                e.axe_min_damage = Some(8);
            }),
        ),
        (
            "armor_cleave",
            with(|e| e.effective = vec![(UnitTag::Armored, 2)]),
        ),
        (
            "close_shot",
            with(|e| {
                e.hit = -10;
                e.min_range = Some(1);
            }),
        ),
        (
            "pinning_shot",
            with(|e| e.on_first_hit = debuff(StatKind::Mov, 3)),
        ),
        (
            "pressure_point",
            with(|e| e.on_first_hit = debuff(StatKind::Spd, 3)),
        ),
        ("sidestep", with(|e| e.stance = Some(sidestep))),
    ];
    let t = embedded();
    for (id, effect) in rows {
        assert_eq!(get(&t, id).effect, effect, "{id}");
    }
}

/// `combat-arts.md`, *Ranks C-S* (ticket 0325): the effects that change the
/// numbers or debuff the target.
#[test]
fn the_embedded_ranks_c_to_s_effects_match_the_design_table() {
    let debuff = |stat, amount| vec![Debuff { stat, amount }];
    let rows = [
        ("cascade", with(|e| e.cascade = 3)),
        (
            "long_thrust",
            with(|e| {
                e.hit = -10;
                e.min_range = Some(1);
                e.max_range = Some(2);
            }),
        ),
        (
            "skypiercer",
            with(|e| e.effective = vec![(UnitTag::Flying, 2)]),
        ),
        (
            "hurl",
            with(|e| {
                e.hit = -10;
                e.min_range = Some(1);
                e.max_range = Some(2);
            }),
        ),
        ("skull_splitter", with(|e| e.crit = 30)),
        (
            "felling_blow",
            with(|e| e.axe_min_damage_max_hp_div = Some(3)),
        ),
        (
            "rending_arrow",
            with(|e| e.on_first_hit = debuff(StatKind::Def, 5)),
        ),
        ("off_balance", with(|e| e.on_first_hit_avoid = 20)),
        (
            "vital_point",
            with(|e| {
                e.on_first_hit = [
                    StatKind::Str,
                    StatKind::Mag,
                    StatKind::Def,
                    StatKind::Res,
                    StatKind::Spd,
                ]
                .map(|stat| Debuff { stat, amount: 4 })
                .to_vec();
            }),
        ),
    ];
    let t = embedded();
    for (id, effect) in rows {
        assert_eq!(get(&t, id).effect, effect, "{id}");
    }
}

/// Only special weapons have arts of their own (`combat-arts.md`), and the
/// embedded items pass the checks.
#[test]
fn only_special_weapons_have_arts() {
    let items = crate::item::load().unwrap_or_else(|e| panic!("{e:?}"));
    for (id, item) in &items.items {
        if let ItemDef::Weapon(w) = item {
            assert!(w.arts.is_empty() || w.is_special(), "{}", id.0);
        }
    }
    assert_eq!(check_references(&embedded(), &items), []);
}

fn errors(source: &str) -> Vec<String> {
    match from_source("arts.ron", source) {
        Ok(_) => Vec::new(),
        Err(e) => e.into_iter().map(|e| e.to_string()).collect(),
    }
}

fn one(fields: &str) -> String {
    format!("(arts: [({fields})])")
}

#[test]
fn a_valid_file_loads() {
    let t = from_source(
        "arts.ron",
        &one(r#"id: "a", name: "A", kind: Sword, rank: None, cost: 1"#),
    )
    .unwrap_or_else(|e| panic!("{e:?}"));
    let a = t.get(&ArtId("a".into()));
    assert_eq!(a.map(|a| (a.rank, a.cost)), Some((None, 1)));
    assert_eq!(a.map(|a| &a.effect), Some(&ArtEffect::default()));
}

#[test]
fn every_rule_reports_its_error() {
    let cases = [
        (
            r#"id: "", name: "A", kind: Sword, rank: None, cost: 1"#,
            "an art id is empty",
        ),
        (
            r#"id: "a", name: "A", kind: Sword, rank: None, cost: 0"#,
            "art \"a\": cost must be at least 1",
        ),
        (
            r#"id: "a", name: "A", kind: Spear, rank: None, cost: 1, effect: (sword_followup: Some((3, 2)))"#,
            "sword_followup is for Sword arts only",
        ),
        (
            r#"id: "a", name: "A", kind: Sword, rank: None, cost: 1, effect: (sword_followup: Some((3, 0)))"#,
            "sword_followup's denominator must be at least 1",
        ),
        (
            r#"id: "a", name: "A", kind: Bow, rank: None, cost: 1, effect: (axe_min_damage: Some(8))"#,
            "axe_min_damage is for Axe arts only",
        ),
        (
            r#"id: "a", name: "A", kind: Bow, rank: None, cost: 1, effect: (min_range: Some(0))"#,
            "min_range must be at least 1",
        ),
        (
            r#"id: "a", name: "A", kind: Bow, rank: None, cost: 1, effect: (on_first_hit: Some((stat: Mov, amount: 0)))"#,
            "a debuff's amount must be at least 1",
        ),
        (
            r#"id: "a", name: "A", kind: Bow, rank: None, cost: 1, effect: (max_range: Some(0))"#,
            "max_range must be at least 1",
        ),
        (
            r#"id: "a", name: "A", kind: Bow, rank: None, cost: 1, effect: (cascade: 3)"#,
            "cascade is for Sword arts only",
        ),
        (
            r#"id: "a", name: "A", kind: Sword, rank: None, cost: 1, effect: (axe_min_damage_max_hp_div: Some(3))"#,
            "axe_min_damage_max_hp_div is for Axe arts only",
        ),
        (
            r#"id: "a", name: "A", kind: Axe, rank: None, cost: 1, effect: (axe_min_damage_max_hp_div: Some(0))"#,
            "axe_min_damage_max_hp_div must be at least 1",
        ),
        (
            r#"id: "a", name: "A", kind: Bow, rank: None, cost: 1, effect: (on_first_hit: [(stat: Def, amount: 5), (stat: Mag, amount: 0)])"#,
            "a debuff's amount must be at least 1",
        ),
        (
            r#"id: "a", name: "A", kind: Bow, rank: None, cost: 1, effect: (effective: [(Flying, 0)])"#,
            "an effectiveness multiplier must be at least 1",
        ),
    ];
    for (fields, message) in cases {
        let found = errors(&one(fields));
        assert!(
            found.len() == 1 && found[0].contains(message),
            "{fields}: {found:?}"
        );
    }
    // Fine at the edges.
    for fields in [
        r#"id: "a", name: "A", kind: Sword, rank: None, cost: 1, effect: (sword_followup: Some((3, 1)))"#,
        r#"id: "a", name: "A", kind: Axe, rank: None, cost: 1, effect: (axe_min_damage: Some(8))"#,
        r#"id: "a", name: "A", kind: Bow, rank: None, cost: 1, effect: (min_range: Some(1), effective: [(Flying, 1)])"#,
        r#"id: "a", name: "A", kind: Bow, rank: None, cost: 1, effect: (on_first_hit: Some((stat: Mov, amount: 1)))"#,
    ] {
        assert_eq!(errors(&one(fields)), Vec::<String>::new(), "{fields}");
    }
}

#[test]
fn duplicates_unknown_fields_and_kinds_are_errors() {
    let art = r#"(id: "a", name: "A", kind: Sword, rank: Some(E), cost: 1)"#;
    let found = errors(&format!("(arts: [{art}, {art}])"));
    assert!(
        found.len() == 1 && found[0].contains("duplicate art id \"a\""),
        "{found:?}"
    );
    assert_eq!(
        errors(&one(
            r#"id: "a", name: "A", kind: Staff, rank: None, cost: 1"#
        ))
        .len(),
        1
    );
    assert_eq!(
        errors(&one(
            r#"id: "a", name: "A", kind: Sword, rank: None, cost: 1, effect: (might: 3)"#
        ))
        .len(),
        1
    );
    // Every problem of one art is reported.
    let found = errors(&one(
        r#"id: "", name: "A", kind: Bow, rank: None, cost: 0, effect: (min_range: Some(0))"#,
    ));
    assert_eq!(found.len(), 3, "{found:?}");
}

#[test]
fn weapon_arts_must_exist_be_weapon_arts_and_match_the_kind() {
    let arts = from_source(
        "arts.ron",
        r#"(arts: [
            (id: "moonlight", name: "Moonlight", kind: Sword, rank: None, cost: 3),
            (id: "cut", name: "Cut", kind: Sword, rank: Some(E), cost: 2),
            (id: "volley", name: "Volley", kind: Bow, rank: None, cost: 2),
        ])"#,
    )
    .unwrap_or_else(|e| panic!("{e:?}"));
    let mut items = crate::item::load().unwrap_or_else(|e| panic!("{e:?}"));
    // Only the arts given below are checked.
    for item in items.items.values_mut() {
        if let ItemDef::Weapon(w) = item {
            w.arts.clear();
        }
    }
    let give = |items: &mut ItemTable, art: &str| {
        if let Some(ItemDef::Weapon(w)) = items.items.get_mut(&trpg_core::ItemId::new("iron_sword"))
        {
            w.arts = vec![ArtId(art.into())];
        }
    };
    give(&mut items, "moonlight");
    assert_eq!(check_references(&arts, &items), []);
    for (art, message) in [
        ("nope", "unknown art \"nope\""),
        ("cut", "\"cut\" is a rank art"),
        ("volley", "\"volley\" is a Bow art, not a Sword one"),
    ] {
        give(&mut items, art);
        let found: Vec<String> = check_references(&arts, &items)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert!(
            found.len() == 1
                && found[0].contains("weapon \"iron_sword\"")
                && found[0].contains(message),
            "{art}: {found:?}"
        );
    }
}
