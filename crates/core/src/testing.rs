//! Fixtures for tests (ticket 1237): the smallest valid value of each
//! content definition, for a test to write as
//! `ClassDef { tier: 2, ..testing::class("fighter") }` and name only the
//! fields it is about. Adding a field to a definition then edits one
//! fixture, not every test that builds one.
//!
//! Built in this crate's tests and, through the `testing` feature, in the
//! tests of the crates that use it. Never in a build that ships.

use crate::prelude::*;

use crate::Objective;
use crate::art::{ArtDef, ArtEffect, ArtId};
use crate::campaign::{BattleDef, BattleKind, BattleMusic, Difficulty};
use crate::class::{ClassDef, ClassId, UnitTags};
use crate::combat::DamageType;
use crate::geom::Grid;
use crate::item::{ItemDef, WeaponDef};
use crate::magic::Element;
use crate::map::BattleMap;
use crate::skill::{SkillDef, SkillId, SkillKind};
use crate::spell::{SpellDef, SpellId, SpellKind};
use crate::stats::{Growths, Stats};
use crate::terrain::{MovementTypeId, TerrainId};
use crate::weapon::{WeaponKind, WeaponRank};

/// A tier-1 class `id` (its name too): Mov 4, zero stats and growths, no
/// weapons, 3 weapon slots, no armour, tags, promotions, skills, spells or
/// affinities, open to anyone.
pub fn class(id: &str) -> ClassDef {
    ClassDef {
        id: ClassId(id.into()),
        name: id.into(),
        tier: 1,
        movement_type: MovementTypeId(0),
        move_points: 4,
        base: Stats::default(),
        growths: Growths::default(),
        weapons: vec![],
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

/// A passive skill `id` (its name and family too), rank 1, with no
/// effects.
pub fn skill(id: &str) -> SkillDef {
    SkillDef {
        id: SkillId::new(id),
        name: id.into(),
        family: id.into(),
        rank: 1,
        kind: SkillKind::Passive(vec![]),
    }
}

/// A healing spell `id` (its name too) of no power: no element, range 1,
/// 1 use, no terrain effect.
pub fn spell(id: &str) -> SpellDef {
    SpellDef {
        id: SpellId::new(id),
        name: id.into(),
        kind: SpellKind::Heal { heal_power: 0 },
        element: Element::None,
        min_range: 1,
        max_range: 1,
        uses: 1,
        terrain_effect: None,
        no_counter: false,
    }
}

/// A sword art `id` (its name too) of any rank, costing 1 durability,
/// that does nothing.
pub fn art(id: &str) -> ArtDef {
    ArtDef {
        id: ArtId::new(id),
        name: id.into(),
        kind: WeaponKind::Sword,
        rank: None,
        cost: 1,
        effect: ArtEffect::default(),
    }
}

/// A battle `id` on one plain tile: no units, slots, pack, rewards,
/// triggers or notes; Rout with no turn limit; a Normal story battle with
/// the `test` music cue and seed 0.
pub fn battle(id: &str) -> BattleDef {
    let tile = Grid::from_cells(1, 1, vec![TerrainId(0)]);
    BattleDef {
        id: id.into(),
        map: BattleMap::new(id, tile.unwrap_or_else(|| panic!("a 1 × 1 grid"))),
        player_slots: vec![],
        enemies: vec![],
        reinforcements: vec![],
        preparations: false,
        pack_cap: 0,
        default_pack: vec![],
        solo_stock: vec![],
        solo_gold: 0,
        solo_bench: vec![],
        bots_army: None,
        clear_gold: 0,
        clear_items: vec![],
        objective: Objective::Rout { turn_limit: None },
        triggers: vec![],
        battle_notes: vec![],
        place_name: None,
        village: None,
        difficulty: Difficulty::Normal,
        kind: BattleKind::Story,
        music: BattleMusic::Cue("test".into()),
        seed: 0,
    }
}

/// An E-rank sword's [`WeaponDef`] named `name`: zero might, hit, crit
/// and weight, range 1, physical, 1 durability, free, with nothing
/// effective, no arts and nothing special.
pub fn sword_def(name: &str) -> WeaponDef {
    WeaponDef {
        name: name.into(),
        short_name: None,
        kind: WeaponKind::Sword,
        rank: WeaponRank::E,
        might: 0,
        hit: 0,
        crit: 0,
        weight: 0,
        min_range: 1,
        max_range: 1,
        damage_type: DamageType::Physical,
        durability: 1,
        effective: vec![],
        arts: vec![],
        price: 0,
        special: None,
    }
}

/// [`sword_def`] as an item.
pub fn sword(name: &str) -> ItemDef {
    ItemDef::Weapon(sword_def(name))
}
