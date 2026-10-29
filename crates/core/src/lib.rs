//! Pure, deterministic game rules. See ADR-0004.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// What `std`'s prelude gives for free, for the `no_std` build.
mod prelude {
    pub use alloc::borrow::ToOwned;
    pub use alloc::boxed::Box;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec;
    pub use alloc::vec::Vec;
}

pub mod ai;
pub mod art;
pub mod battle;
pub mod campaign;
pub mod class;
pub mod combat;
pub mod history;
pub mod item;
pub mod lead;
pub mod legal;
pub mod magic;
pub mod map;
pub mod movement;
pub mod prep;
pub mod progression;
pub mod save;
pub mod shop;
pub mod skill;
pub mod spell;
pub mod stats;
pub mod support;
pub mod terrain;
pub mod unit;
pub mod weapon;

// The engine's grid and RNG, under the names the game's code uses
// (ADR-0063 §6).
pub use shuyi_core::grid as geom;
pub use shuyi_core::rng;

pub use ai::{AiBehavior, AiWeights, next_command};
pub use art::{ArtDef, ArtEffect, ArtId, ArtNote, ArtTable, Debuff};
pub use battle::{
    AttackPreview, BattleNote, BattleSetup, BattleState, Burning, CastTarget, Command,
    CommandError, Destination, Event, GameMode, HouseState, Leftovers, Objective, Outcome,
    PendingMove, Phase, Reinforcement, SellFrom, ShopTxn, TileRect, Trigger, TriggerWhen, Turn,
    UnitAction, Village, VillageGoal, Who,
};
pub use campaign::{
    ApplyError, BattleDef, BattleKind, BattleMusic, BattleRewards, Campaign, Difficulty,
    GameTables, PlayerSlot, TutorialLevel, UNUSED_CHARGE_PERCENT,
};
pub use class::{
    ArmourWeight, ClassDef, ClassId, ClassLevel, ClassPoints, ClassTable, Tier, UnitTag, UnitTags,
    WeaponProficiency,
};
pub use combat::{
    CombatHp, CombatMods, CombatOutcome, CombatRules, CombatantInput, DamageType, Forecast,
    MitigationStat, PlannedStrike, PowerStat, Side, SideForecast, Strike, StrikePlan, WeaponStats,
    WeaponTrait, forecast, if_all_hit, resolve, roll_hit, strike_order,
};
pub use geom::{Dir, Grid, GridSizeError, Pos};
pub use history::{BattleHistory, Replayed, RewindError};
pub use item::{
    AccessoryDef, ArmourDef, BattlePack, ConsumableDef, ConsumableEffect, Equipped, ItemDef,
    ItemId, ItemTable, Loadout, LoadoutDef, LoadoutError, SealDef, SealKind, SpecialWeapon, Stock,
    WEAPON_SLOTS, WeaponDef, WeaponInstance, WeaponPassive, WeaponRules,
};
pub use lead::{LEAD_ID, LeadGender, LeadProfile, Pronouns};
pub use legal::legal_commands;
pub use magic::{Affinity, Element};
pub use map::{BattleMap, TileFeature};
pub use movement::{
    AttackRange, MoveError, PathError, Reach, TileSet, attack_tiles, danger_zone, path_cost,
    reachable, threat_area, threats_at,
};
pub use prep::{GearSlot, PrepError, PrepUnit, Preparations, StockItem, Unusable, WeaponAt};
pub use progression::{
    ChangeTables, ClassChangeError, CombatResult, StatGains, apply_gains, exp_for_combat,
    grant_class_points, grant_exp, growth, has_mastered, level_up, paced_exp, promote,
    promotion_gains, promotion_targets, reclass, reclass_targets,
};
pub use rng::{RandomSource, ScriptedRng, SimRng};
pub use save::{SAVE_VERSION, SaveFile, SaveHeader, SavePoint};
pub use shop::{
    BasicStock, Gold, Loot, Shop, ShopError, ShopKind, ShopSession, Unlock, repair_cost, sell_price,
};
pub use skill::{
    ActiveEffect, Area, Bonuses, Condition, CostError, CostSource, EffectSource, Paid,
    PassiveEffect, SkillContext, SkillCost, SkillDef, SkillId, SkillKind, SkillTable, SkillUses,
    Stance, TimedEffect, TimedMods, WeaponReq, check_cost, pay_cost,
};
pub use spell::{
    EffectDuration, SpellChanges, SpellDef, SpellId, SpellKind, SpellState, SpellTable,
    TerrainEffect,
};
pub use stats::{GrowthValue, Growths, StatKind, StatValue, Stats};
pub use support::{
    ByRank, PairDef, PointValues, SupportBonus, SupportBook, SupportError, SupportPair,
    SupportRank, SupportRules, SupportState, SupportTable, SupportViewed, Thresholds,
};
pub use terrain::{MovementTypeId, TerrainId, TerrainRules, TerrainTable};
pub use unit::{
    CharacterDef, CharacterId, ClassRecord, Defeat, Faction, Level, MAP_LABEL_LEN, Role, Unit,
    UnitError, UnitId, default_map_label, is_valid_map_label,
};
pub use weapon::{WeaponKind, WeaponRank};
