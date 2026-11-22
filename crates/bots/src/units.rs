//! Each player unit at a battle's start, and the flags of a unit that
//! falls behind (ticket 0527, `docs/playtesting.md`, *Units that fall
//! behind*). The report proposes, it never tunes: a flag is a line to
//! read, never a failure, and nothing here changes a unit.

use trpg_core::{BattleState, Command, Equipped, UnitId, WEAPON_SLOTS};

/// A unit that can hurt under this share of the battle's enemies is
/// flagged: `(2, 3)` is under 2/3 (*the agent's starting value*, tunable).
pub const CAN_HURT_AT_LEAST: (u32, u32) = (2, 3);

/// A unit whose level arriving at a battle is this many or more below the
/// army's median is flagged (*the agent's starting value*, tunable).
pub const LEVELS_BEHIND: i64 = 3;

/// A unit with under this share of an even split of the EXP the army
/// earned is flagged: `(1, 2)` is under half an even share (*the agent's
/// starting value*, tunable).
pub const EXP_SHARE_AT_LEAST: (u64, u64) = (1, 2);

/// How many of the enemies on the map at `state`'s start player unit
/// `unit` can hurt, and of how many ([`enemies_hurt`]). `(0, 0)` for a
/// unit not on the map.
pub fn can_hurt(state: &BattleState, unit: UnitId) -> (u32, u32) {
    let enemies = enemies_hurt(state, unit);
    let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    let hurt = enemies.iter().filter(|(_, hurt)| *hurt).count();
    (count(hurt), count(enemies.len()))
}

/// Each enemy on the map at `state`'s start (a unit hostile to `unit`'s
/// faction), in unit order, and whether player unit `unit` can hurt it: whether any weapon or attack spell
/// `unit` can equip gives a forecast of more than 0 damage per strike
/// against it where it stands (its terrain included), with any Combat Art
/// or combat active `unit` may use with it
/// ([`BattleState::threat_forecast`]). Empty for a unit not on the map.
pub fn enemies_hurt(state: &BattleState, unit: UnitId) -> Vec<(UnitId, bool)> {
    let Some(me) = state.unit(unit) else {
        return Vec::new();
    };
    // The battle as it is (what `unit` has equipped), then with each
    // weapon or attack spell equipped that the battle lets it equip.
    let weapons = (0..WEAPON_SLOTS)
        .filter(|&slot| me.loadout.weapon(slot).is_some())
        .map(Equipped::Weapon);
    let known = me.known_spells(state.classes());
    let spells = known.into_iter().map(Equipped::Spell);
    let mut armed = vec![state.clone()];
    for equipped in weapons.chain(spells) {
        let mut with = state.clone();
        if with.apply(&Command::Equip { unit, equipped }).is_ok() {
            armed.push(with);
        }
    }
    let hurts = |enemy: UnitId| {
        armed.iter().any(|state| {
            state
                .threat_forecast(unit, enemy)
                .is_some_and(|f| f.attacker.damage > 0)
        })
    };
    state
        .units()
        .iter()
        .filter(|u| me.faction.is_hostile_to(u.faction))
        .map(|u| (u.id, hurts(u.id)))
        .collect()
}

/// Why a unit's line is flagged.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    /// It can hurt under [`CAN_HURT_AT_LEAST`] of the battle's enemies.
    CantHurt,
    /// It arrives [`LEVELS_BEHIND`] or more levels below the army's median.
    Behind,
    /// It earned under [`EXP_SHARE_AT_LEAST`] of an even share of the EXP.
    LowExp,
}

impl Flag {
    /// The flag as the report prints it.
    pub fn text(self) -> &'static str {
        match self {
            Flag::CantHurt => "can hurt < 2/3",
            Flag::Behind => "lv 3+ below the army",
            Flag::LowExp => "exp < half a share",
        }
    }
}

/// One unit's numbers that the flags read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Standing {
    /// Enemies it can hurt, of how many ([`can_hurt`]).
    pub can_hurt: (u32, u32),
    /// Its level arriving at the battle.
    pub level: i64,
    /// The army's median level arriving at the battle.
    pub army_level: i64,
    /// The EXP it earned over every try.
    pub exp: u64,
    /// The EXP the whole army earned over every try.
    pub army_exp: u64,
    /// How many player units the army fields.
    pub army_size: u64,
}

/// The flags of a unit with `standing`, in [`Flag`]'s order. No enemies,
/// or no EXP earned, flags nothing.
pub fn flags(standing: &Standing) -> Vec<Flag> {
    let (hurt, of) = standing.can_hurt;
    let (num, den) = CAN_HURT_AT_LEAST;
    let cant_hurt = of > 0 && u64::from(hurt) * u64::from(den) < u64::from(of) * u64::from(num);
    let behind = standing.army_level - standing.level >= LEVELS_BEHIND;
    // exp / army_exp < share / army_size, without dividing.
    let (num, den) = EXP_SHARE_AT_LEAST;
    let low_exp = standing.army_exp > 0
        && standing.army_size > 0
        && u128::from(standing.exp) * u128::from(standing.army_size) * u128::from(den)
            < u128::from(standing.army_exp) * u128::from(num);
    [
        (cant_hurt, Flag::CantHurt),
        (behind, Flag::Behind),
        (low_exp, Flag::LowExp),
    ]
    .into_iter()
    .filter_map(|(on, flag)| on.then_some(flag))
    .collect()
}

#[cfg(test)]
mod tests;
