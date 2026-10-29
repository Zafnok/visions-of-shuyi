//! Every Combat Art in `assets/data/arts.ron`, used in a battle built from
//! the real content tables (ticket 0312): `combat-arts.md`'s worked example
//! (W1 with each sword art: forecast and weapon EXP), and each art used by a
//! rank-D unit of a class that wields its kind, with an iron weapon.

use std::sync::{Arc, OnceLock};

use trpg_content::Content;
use trpg_core::{
    ArtId, BattleMap, BattlePack, BattleSetup, BattleState, ClassId, Command, Event, Faction, Grid,
    ItemDef, ItemId, LoadoutDef, Objective, Pos, Stats, Stock, Unit, UnitAction, UnitId,
    WeaponKind, WeaponRank,
};

fn content() -> &'static Content {
    static CONTENT: OnceLock<Content> = OnceLock::new();
    CONTENT.get_or_init(|| trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}")))
}

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

/// A generic unit of `class` with `stats` (HP, Str, Mag, Dex, Spd, Def, Res),
/// carrying `weapon`, at rank D in `kind`.
fn unit(
    id: u32,
    class: &str,
    faction: Faction,
    pos: Pos,
    stats: [i32; 7],
    (weapon, kind): (&str, WeaponKind),
) -> Unit {
    let c = content();
    let mut u = Unit::generic(
        UnitId(id),
        &ClassId(class.into()),
        &c.classes,
        1,
        faction,
        pos,
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let mov = u.stats.mov;
    u.stats = Stats::from_growable(stats, mov);
    u.hp = stats[0];
    u.weapon_ranks.insert(kind, WeaponRank::D);
    let loadout = LoadoutDef {
        weapons: vec![ItemId::new(weapon)],
        ..LoadoutDef::default()
    };
    u.with_loadout(&loadout, &c.classes, &c.items)
        .unwrap_or_else(|e| panic!("{e}"))
}

/// A battle on an 8×5 plain map.
fn battle(units: Vec<Unit>) -> BattleState {
    let c = content();
    let plain = c
        .terrain
        .display
        .id_of("plain")
        .unwrap_or_else(|| panic!("no plain"));
    BattleState::new(BattleSetup {
        map: BattleMap::new("Arts", Grid::filled(8, 5, plain)),
        terrain: Arc::new(c.terrain.rules.clone()),
        classes: Arc::new(c.classes.clone()),
        items: Arc::new(c.items.clone()),
        spells: Arc::new(c.spells.clone()),
        skills: Arc::new(c.skills.clone()),
        arts: Arc::new(c.arts.clone()),
        supports: Arc::new(c.supports.clone()),
        bonds: trpg_core::SupportBook::default(),
        pack: BattlePack::default(),
        gold: 0,
        stock: Stock::default(),
        units,
        reinforcements: vec![],
        objective: Objective::Rout { turn_limit: None },
        rewind_charges: 0,
        seed: 1,
        triggers: vec![],
        mode: trpg_core::GameMode::Classic,
        battle_notes: vec![],
        village: None,
    })
    .0
}

fn attack(target: u32, art: Option<&str>) -> UnitAction {
    UnitAction::Attack {
        target: UnitId(target),
        slot: 0,
        active: None,
        art: art.map(ArtId::new),
    }
}

fn act(s: &mut BattleState, id: u32, dest: Pos, action: UnitAction) -> Vec<Event> {
    s.apply(&Command::Act {
        unit: UnitId(id),
        dest,
        action,
    })
    .unwrap_or_else(|e| panic!("{e}"))
}

/// W1 of `weapons-and-items.md` (a Swordsman with an Iron Sword, rank D,
/// against a Brigand with an Iron Axe), side by side.
fn w1() -> BattleState {
    battle(vec![
        unit(
            1,
            "swordsman",
            Faction::Player,
            p(0, 0),
            [22, 8, 0, 7, 9, 5, 2],
            ("iron_sword", WeaponKind::Sword),
        ),
        unit(
            3,
            "brigand",
            Faction::Enemy,
            p(1, 0),
            [20, 9, 0, 3, 5, 4, 1],
            ("iron_axe", WeaponKind::Axe),
        ),
    ])
}

/// `combat-arts.md`, *Worked example*: the forecast with each sword art,
/// and the weapon EXP when both strikes hit (5 / 8 / 7).
#[test]
fn the_worked_example_w1_with_each_sword_art() {
    // (art, attacker (damage, follow-up, hit, crit, strikes), counter,
    // durability after, weapon EXP)
    let cases = [
        (None, (9, 10, 94, 3, 2), Some((12, 63)), 20, 5),
        (
            Some("flowing_cut"),
            (9, 13, 94, 3, 2),
            Some((12, 63)),
            18,
            8,
        ),
        (Some("guard_break"), (9, 10, 100, 3, 2), None, 16, 7),
    ];
    for (art, attacker, counter, left, exp) in cases {
        let mut s = w1();
        let action = attack(3, art);
        let shown = s
            .preview_attack(UnitId(1), p(0, 0), &action)
            .unwrap_or_else(|e| panic!("{e}"));
        let a = shown.forecast.attacker;
        assert_eq!(
            (a.damage, a.followup_damage, a.hit, a.crit, a.strikes),
            attacker,
            "{art:?}"
        );
        assert_eq!(
            shown.forecast.defender.map(|d| (d.damage, d.hit)),
            counter,
            "{art:?}"
        );
        assert_eq!(shown.durability, art.map(|_| (20, left)), "{art:?}");
        let events = act(&mut s, 1, p(0, 0), action);
        let outcome = events
            .iter()
            .find_map(|e| match e {
                Event::CombatResolved { outcome, .. } => Some(outcome.clone()),
                _ => None,
            })
            .unwrap_or_else(|| panic!("no combat"));
        let hits: Vec<bool> = outcome
            .strikes
            .iter()
            .filter(|x| x.by == trpg_core::Side::Attacker)
            .map(|x| x.hit)
            .collect();
        assert_eq!(hits, [true, true], "{art:?}: the example needs both hits");
        let gained = events.iter().find_map(|e| match e {
            Event::WeaponExpGained { unit, amount, .. } if *unit == UnitId(1) => Some(*amount),
            _ => None,
        });
        assert_eq!(gained, Some(exp), "{art:?}");
        let durability = s
            .unit(UnitId(1))
            .and_then(|u| u.loadout.weapon(0))
            .map(|w| w.durability_left);
        assert_eq!(durability, Some(left), "{art:?}");
    }
}

/// Each art, used by a rank-D unit of a class that wields its kind with
/// that kind's iron weapon (a weapon art: with the special weapon that
/// carries it, at that weapon's rank), on an enemy in range: the attack is
/// accepted, announced and paid.
#[test]
fn every_art_can_be_used_with_its_iron_weapon() {
    let weapon = |kind| match kind {
        WeaponKind::Sword => ("swordsman", "iron_sword"),
        WeaponKind::Spear => ("guard", "iron_spear"),
        WeaponKind::Axe => ("raider", "iron_axe"),
        WeaponKind::Bow => ("archer", "iron_bow"),
        WeaponKind::Gauntlet => ("brawler", "iron_gauntlets"),
    };
    let (arts, items) = (&content().arts, &content().items);
    assert_eq!(arts.arts.len(), 22);
    for (id, art) in &arts.arts {
        let (class, iron) = weapon(art.kind);
        let own = items.items.iter().find_map(|(item, def)| match def {
            ItemDef::Weapon(w) if w.arts.contains(id) => Some((item.0.as_str(), w)),
            _ => None,
        });
        assert_eq!(own.is_some(), art.rank.is_none(), "{}", id.0);
        let (iron, rank, durability) = own.map_or((iron, art.rank, 20), |(item, w)| {
            (item, Some(w.rank.max(WeaponRank::D)), w.durability)
        });
        // Bows shoot from 2 tiles (Close Shot could from 1).
        let at = if art.kind == WeaponKind::Bow {
            p(2, 0)
        } else {
            p(1, 0)
        };
        // The unit has the rank that teaches the art.
        let mut user = unit(
            1,
            class,
            Faction::Player,
            p(0, 0),
            [30, 5, 0, 5, 5, 2, 2],
            (iron, art.kind),
        );
        user.weapon_ranks
            .insert(art.kind, rank.unwrap_or(WeaponRank::D));
        let mut s = battle(vec![
            user,
            unit(
                3,
                "swordsman",
                Faction::Enemy,
                at,
                [30, 5, 0, 5, 5, 2, 2],
                ("iron_sword", WeaponKind::Sword),
            ),
        ]);
        let events = act(&mut s, 1, p(0, 0), attack(3, Some(&id.0)));
        assert_eq!(
            events.first(),
            Some(&Event::ArtUsed {
                unit: UnitId(1),
                art: id.clone(),
                weapon: ItemId::new(iron),
                durability_before: durability,
                durability_after: durability - art.cost,
            }),
            "{}",
            id.0
        );
    }
}

/// Ticket 0325: a unit at rank C, B, A or S in a kind knows that rank's
/// arts from the list (and the lower ranks' too).
#[test]
fn rank_arts_c_to_s_are_taught_by_their_rank() {
    use WeaponKind::{Axe, Bow, Gauntlet, Spear, Sword};
    use WeaponRank::{A, B, C, S};
    let by_rank = |kind, rank| {
        let mut names: Vec<_> = content()
            .arts
            .arts
            .values()
            .filter(|a| a.kind == kind && a.rank == Some(rank))
            .map(|a| a.id.0.as_str())
            .collect();
        names.sort_unstable();
        names
    };
    assert_eq!(by_rank(Sword, A), ["cascade"]);
    assert_eq!(by_rank(Spear, C), ["long_thrust"]);
    assert_eq!(by_rank(Spear, A), ["skypiercer"]);
    assert_eq!(by_rank(Axe, B), ["hurl"]);
    assert_eq!(by_rank(Axe, A), ["skull_splitter"]);
    assert_eq!(by_rank(Axe, S), ["felling_blow"]);
    assert_eq!(by_rank(Bow, C), ["rending_arrow"]);
    assert_eq!(by_rank(Gauntlet, C), ["off_balance"]);
    assert_eq!(by_rank(Gauntlet, S), ["vital_point"]);
    // The data loads the old one-debuff form for the older arts.
    let pin = &content().arts.arts[&ArtId("pinning_shot".into())];
    assert_eq!(pin.effect.on_first_hit.len(), 1);
}
