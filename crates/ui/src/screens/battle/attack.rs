//! Choosing an attack (ticket 0404): which weapons can attack someone from
//! where the unit stands, the targets of each, in `(y, x)` order, and the
//! targeting state with its forecast. What the attack is made with (plainly,
//! a Combat Art, a combat active) is chosen before the forecast opens
//! (0442): `Attack` and pointing at an enemy attack plainly, the arts menu
//! and the spell list fix an art or an active. Every check is
//! [`BattleState::preview_attack`], the same validation the attack command
//! gets, so the UI never decides what is legal itself (ADR-0004).

use crate::prelude::*;
use trpg_core::{
    AttackPreview, BattleState, Equipped, Pos, SpellDef, SpellId, UnitAction, UnitId, WEAPON_SLOTS,
};

use super::art_list::{ArtLine, Technique};
use super::mode::{Line, Selection};
use crate::widgets::menu::Menu;
use crate::widgets::wrap_step;
use crate::words::Words;

/// The weapon attack with the weapon in `slot` on `target`: no art, no
/// active.
pub fn attack(target: UnitId, slot: usize) -> UnitAction {
    Technique::Attack.action(target, &Equipped::Weapon(slot))
}

/// The units `unit` could attack from `dest` with `with` (a weapon's slot or
/// an attack spell) using `technique`, ordered by `(y, x)` of their tiles.
pub fn reach(
    state: &BattleState,
    unit: UnitId,
    dest: Pos,
    with: &Equipped,
    technique: &Technique,
) -> Vec<UnitId> {
    let hit = |target| {
        let action = technique.action(target, with);
        state.preview_attack(unit, dest, &action).is_ok()
    };
    let mut found: Vec<(Pos, UnitId)> = state
        .units()
        .iter()
        .filter(|u| hit(u.id))
        .map(|u| (u.pos, u.id))
        .collect();
    found.sort_by_key(|(p, _)| (p.y, p.x));
    found.into_iter().map(|(_, id)| id).collect()
}

/// The units `unit` could attack plainly from `dest` with the weapon in
/// `slot`, ordered by `(y, x)` of their tiles.
pub fn targets(state: &BattleState, unit: UnitId, dest: Pos, slot: usize) -> Vec<UnitId> {
    reach(
        state,
        unit,
        dest,
        &Equipped::Weapon(slot),
        &Technique::Attack,
    )
}

/// What `unit` could attack with: its weapon slots in order, then its
/// attack spells in id order (the order of the `Equip` list).
fn arsenal(state: &BattleState, unit: UnitId) -> Vec<Equipped> {
    let spells = state.unit(unit).into_iter().flat_map(|u| {
        u.learned
            .iter()
            .filter(|id| state.spells().get(id).is_some_and(SpellDef::is_attack))
            .map(|id| Equipped::Spell(id.clone()))
    });
    (0..WEAPON_SLOTS)
        .map(Equipped::Weapon)
        .chain(spells)
        .collect()
}

/// The weapons and attack spells of `unit` that reach `target` from `from`
/// with a plain attack, weapons first (0430).
pub fn aimed_options(
    state: &BattleState,
    unit: UnitId,
    from: Pos,
    target: UnitId,
) -> Vec<Equipped> {
    let mut all = arsenal(state, unit);
    all.retain(|with| {
        let action = Technique::Attack.action(target, with);
        state.preview_attack(unit, from, &action).is_ok()
    });
    all
}

/// What the forecast on an enemy pointed at opens with (Nick, 0430): the
/// unit's equipped weapon or spell if it is one of `options`, else the
/// first of them.
pub fn aimed_first(state: &BattleState, unit: UnitId, options: &[Equipped]) -> Option<Equipped> {
    let equipped = state.unit(unit).and_then(|u| u.loadout.equipped.as_ref());
    options
        .iter()
        .find(|o| Some(*o) == equipped)
        .or(options.first())
        .cloned()
}

/// Whether some weapon or attack spell of `unit` can attack `target`
/// plainly from `from` (0428). An enemy only an art or active reaches
/// (Close Shot on an adjacent one) is attacked through the arts menu.
pub fn can_hit(state: &BattleState, unit: UnitId, from: Pos, target: UnitId) -> bool {
    !aimed_options(state, unit, from, target).is_empty()
}

/// The tile `sel`'s unit attacks `target` from when the player points at it
/// (0428): [`aim_tile`] with what can hit it.
pub fn attack_tile(state: &BattleState, sel: &Selection, target: UnitId) -> Option<Pos> {
    aim_tile(sel, |from| can_hit(state, sel.unit, from, target))
}

/// The tile `sel`'s unit acts from when the player points at a unit, an
/// enemy to attack (0428) or an ally to heal (0451), where `reaches` says
/// from which tiles it can: the path's end if it reaches from there, else
/// the stoppable tile that does, cheapest first, then the shortest path,
/// then lowest `(y, x)`. `None` if no tile does.
pub fn aim_tile(sel: &Selection, reaches: impl Fn(Pos) -> bool) -> Option<Pos> {
    let end = sel.dest();
    if sel.reach.is_stoppable(end) && reaches(end) {
        return Some(end);
    }
    sel.reach
        .stoppable()
        .iter()
        .filter(|&t| reaches(t))
        .filter_map(|t| {
            let cost = sel.reach.cost(t)?;
            let len = sel.reach.path_to(t)?.len();
            Some((cost, len, t.y, t.x, t))
        })
        .min_by_key(|&(cost, len, y, x, _)| (cost, len, y, x))
        .map(|(.., t)| t)
}

/// A weapon that can attack someone from the destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeaponChoice {
    /// Its loadout slot.
    pub slot: usize,
    /// Who it can attack plainly, in `(y, x)` order (never empty).
    pub targets: Vec<UnitId>,
}

/// The weapons of `sel`'s unit that can attack someone from its path's
/// end, in slot order.
pub fn weapon_choices(state: &BattleState, sel: &Selection) -> Vec<WeaponChoice> {
    (0..WEAPON_SLOTS)
        .filter_map(|slot| {
            let targets = targets(state, sel.unit, sel.dest(), slot);
            (!targets.is_empty()).then_some(WeaponChoice { slot, targets })
        })
        .collect()
}

/// The weapon in `unit`'s `slot`: its name, durability left and max.
pub fn weapon_durability(
    state: &BattleState,
    words: Words<'_>,
    unit: UnitId,
    slot: usize,
) -> Option<(String, u32, u32)> {
    let copy = state.unit(unit)?.loadout.weapon(slot)?;
    let def = state.items().weapon(&copy.def)?;
    let name = words.item(&copy.def, state.items()).to_owned();
    Some((name, copy.durability_left, def.durability))
}

/// Spell `spell` of `unit`: its name, uses left and uses per battle.
pub fn spell_uses(
    state: &BattleState,
    words: Words<'_>,
    unit: UnitId,
    spell: &SpellId,
) -> Option<(String, u32, u32)> {
    let def = state.spells().get(spell)?;
    let left = state.unit(unit)?.spells.uses_left(spell);
    Some((
        words.spell(def).to_owned(),
        u32::from(left),
        u32::from(def.uses),
    ))
}

/// The name of the weapon in `unit`'s `slot` (empty if none).
pub fn weapon_name(state: &BattleState, words: Words<'_>, unit: UnitId, slot: usize) -> String {
    state
        .unit(unit)
        .and_then(|u| u.loadout.weapon(slot))
        .map(|w| words.item(&w.def, state.items()).to_owned())
        .unwrap_or_default()
}

/// Picking a target for an attack with one weapon or attack spell, with the
/// forecast. What the attack is made with (plainly, a Combat Art, a combat
/// active) is fixed before it opens: by the arts menu or the spell list
/// (0442).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Targeting {
    /// The attacker and its path (it stands at the path's end).
    pub sel: Selection,
    /// What it attacks with: a weapon's loadout slot, or an attack spell.
    pub with: Equipped,
    /// Who it can attack, in `(y, x)` order (never empty).
    pub targets: Vec<UnitId>,
    /// The target under the cursor.
    pub index: usize,
    /// What the attack is made with.
    pub technique: Technique,
    /// The forecast against the target.
    pub preview: AttackPreview,
    /// The arts menu it was opened from (Cancel goes back to it).
    pub arts: Option<Menu<Line>>,
    /// The weapons and attack spells left and right swap between: those
    /// that reach the enemy the unit was pointed at (0430), the weapons
    /// that reach someone (`Attack`), or those the chosen art can be used
    /// with (0442). Fewer than two: left and right change the target.
    pub options: Vec<Equipped>,
}

impl Targeting {
    /// The forecast on `sel`'s aimed enemy from its path's end, with the
    /// unit's equipped weapon or spell if that reaches it, else the first
    /// that does. `None` if it has no aimed enemy or nothing reaches it.
    pub fn aimed(state: &BattleState, sel: Selection) -> Option<Self> {
        let options = aimed_options(state, sel.unit, sel.dest(), sel.target?);
        let with = aimed_first(state, sel.unit, &options)?;
        Self::open(state, sel, with, Technique::Attack, options)
    }

    /// The forecast of `Attack` in the action menu (0442): with the unit's
    /// equipped weapon if it is one of `weapons` (those that reach
    /// someone), else the first of them. `None` if there are none.
    pub fn attack(state: &BattleState, sel: Selection, weapons: &[WeaponChoice]) -> Option<Self> {
        let options: Vec<Equipped> = weapons.iter().map(|w| Equipped::Weapon(w.slot)).collect();
        let with = aimed_first(state, sel.unit, &options)?;
        Self::open(state, sel, with, Technique::Attack, options)
    }

    /// The forecast of `line`, a line of the arts menu `menu` (0442), with
    /// the weapon the line opens with. `None` for a dimmed line.
    pub fn art(
        state: &BattleState,
        sel: Selection,
        menu: Menu<Line>,
        line: &ArtLine,
    ) -> Option<Self> {
        if !line.usable() {
            return None;
        }
        let options = line.slots.iter().map(|&s| Equipped::Weapon(s)).collect();
        let (with, technique) = (Equipped::Weapon(line.slot), line.technique.clone());
        let mut t = Self::open(state, sel, with, technique, options)?;
        t.arts = Some(menu);
        Some(t)
    }

    /// Targeting with `with` and `technique`, on `sel`'s aimed enemy if
    /// they reach it; the others follow in `(y, x)` order. Left and right
    /// swap between `options`.
    fn open(
        state: &BattleState,
        sel: Selection,
        with: Equipped,
        technique: Technique,
        options: Vec<Equipped>,
    ) -> Option<Self> {
        let targets = Self::targets_from(state, &sel, &with, &technique, &[]);
        let mut t = Self::with(state, sel, with, technique, targets)?;
        t.options = options;
        Some(t)
    }

    /// Who `sel`'s unit reaches with `with` and `technique`, starting from
    /// the first of `keep` it reaches, else from `sel`'s aimed enemy.
    fn targets_from(
        state: &BattleState,
        sel: &Selection,
        with: &Equipped,
        technique: &Technique,
        keep: &[UnitId],
    ) -> Vec<UnitId> {
        let mut targets = reach(state, sel.unit, sel.dest(), with, technique);
        let at = keep
            .iter()
            .chain(&sel.target)
            .find_map(|t| targets.iter().position(|x| x == t));
        targets.rotate_left(at.unwrap_or(0));
        targets
    }

    /// Whether left and right swap the weapon or spell (see
    /// [`Self::options`]).
    pub fn can_swap(&self) -> bool {
        self.options.len() > 1
    }

    /// Swaps to the next weapon or spell of [`Self::options`] (or the
    /// previous one), wrapping. The art or active stays. The target stays
    /// if the new one reaches it, else it is the enemy pointed at (the
    /// first target, if none was).
    pub fn swap(&mut self, forward: bool, state: &BattleState) {
        let n = self.options.len();
        let Some(i) = self.options.iter().position(|o| *o == self.with) else {
            return;
        };
        let with = self.options[wrap_step(i, n, forward)].clone();
        let keep = [self.target()];
        let targets = Self::targets_from(state, &self.sel, &with, &self.technique, &keep);
        let technique = self.technique.clone();
        if let Some(mut next) = Self::with(state, self.sel.clone(), with, technique, targets) {
            next.options = core::mem::take(&mut self.options);
            next.arts = self.arts.take();
            *self = next;
        }
    }

    /// Targeting the first of `targets` with `with` and `technique`. `None`
    /// if there is no target or its forecast can't be made (the battle
    /// changed; never while choosing).
    pub fn with(
        state: &BattleState,
        sel: Selection,
        with: Equipped,
        technique: Technique,
        targets: Vec<UnitId>,
    ) -> Option<Self> {
        let first = *targets.first()?;
        let preview = state
            .preview_attack(sel.unit, sel.dest(), &technique.action(first, &with))
            .ok()?;
        Some(Self {
            sel,
            with,
            targets,
            index: 0,
            technique,
            preview,
            arts: None,
            options: vec![],
        })
    }

    /// The target under the cursor.
    pub fn target(&self) -> UnitId {
        // `index` is always a valid index of `targets`, which is never empty.
        self.targets[self.index]
    }

    /// Moves to the next target (or the previous one), wrapping, and
    /// updates the forecast.
    pub fn cycle(&mut self, forward: bool, state: &BattleState) {
        if self.targets.is_empty() {
            return;
        }
        let index = wrap_step(self.index, self.targets.len(), forward);
        self.retarget(index, state);
    }

    /// Moves to `target`, if it is one of the targets, as [`Self::cycle`]
    /// does.
    pub fn aim(&mut self, target: UnitId, state: &BattleState) {
        if let Some(index) = self.targets.iter().position(|&t| t == target) {
            self.retarget(index, state);
        }
    }

    /// Moves to target `index` and updates the forecast (unchanged if the
    /// core refuses it, which it doesn't for a target).
    fn retarget(&mut self, index: usize, state: &BattleState) {
        self.index = index;
        let action = self.technique.action(self.target(), &self.with);
        if let Ok(p) = state.preview_attack(self.sel.unit, self.sel.dest(), &action) {
            self.preview = p;
        }
    }

    /// The command that makes the attack.
    pub fn command(&self) -> trpg_core::Command {
        trpg_core::Command::Act {
            unit: self.sel.unit,
            dest: self.sel.dest(),
            action: self.technique.action(self.target(), &self.with),
        }
    }
}

#[cfg(test)]
mod tests {
    use trpg_core::{Command, Objective};

    use super::*;
    use crate::screen::tests::ctx;
    use crate::screens::battle::testing::{battle_with, skirmish};

    fn p(x: i32, y: i32) -> Pos {
        Pos::new(x, y)
    }

    /// The lord of `state` selected with its path to `dest`.
    fn at(state: &BattleState, dest: Pos) -> Selection {
        let mut sel = Selection::new(state, UnitId(1)).unwrap();
        sel.path = vec![sel.origin(), dest];
        sel
    }

    #[test]
    fn targets_are_the_attackable_units_in_row_then_column_order() {
        let c = ctx();
        let s = skirmish(&c, 20);
        // From (7, 2): the raider above (row 1), then the brigand right.
        assert_eq!(targets(&s, UnitId(1), p(7, 2), 0), [UnitId(6), UnitId(4)]);
        // Two brigands either side of (7, 3), in one row: left first.
        let mut units = s.units().to_vec();
        units[3].pos = p(8, 3);
        units[4].pos = p(6, 3);
        let rout = Objective::Rout { turn_limit: None };
        let s = battle_with(&c, s.map().clone(), units, rout);
        assert_eq!(targets(&s, UnitId(1), p(7, 3), 0), [UnitId(5), UnitId(4)]);
        // An empty slot, or a tile the lord can't reach: none.
        assert_eq!(
            targets(&s, UnitId(1), p(7, 3), 2),
            [] as [trpg_core::UnitId; 0]
        );
        assert_eq!(
            targets(&s, UnitId(1), p(0, 0), 0),
            [] as [trpg_core::UnitId; 0]
        );
    }

    #[test]
    fn every_weapon_that_reaches_is_a_choice_and_attack_opens_with_the_equipped_one() {
        let c = ctx();
        let s = skirmish(&c, 20);
        let sel = at(&s, p(7, 2));
        let choices = weapon_choices(&s, &sel);
        let slots: Vec<usize> = choices.iter().map(|w| w.slot).collect();
        assert_eq!(slots, [0, 1]);
        assert!(choices.iter().all(|w| w.targets == [UnitId(6), UnitId(4)]));
        // `Attack` opens with the equipped weapon; left and right swap
        // between the two.
        let open = |s: &BattleState| Targeting::attack(s, sel.clone(), &choices).unwrap();
        let t = open(&s);
        assert_eq!(t.with, Equipped::Weapon(0));
        assert_eq!(t.options, [Equipped::Weapon(0), Equipped::Weapon(1)]);
        assert_eq!(
            (t.technique.clone(), t.arts.clone()),
            (Technique::Attack, None)
        );
        assert!(t.can_swap());
        // With the steel sword equipped, it opens with that.
        let mut units = s.units().to_vec();
        units[0].loadout.equipped = Some(trpg_core::Equipped::Weapon(1));
        let s2 = battle_with(
            &c,
            s.map().clone(),
            units,
            Objective::Rout { turn_limit: None },
        );
        assert_eq!(open(&s2).with, Equipped::Weapon(1));
        // No weapon reaches: no forecast.
        assert_eq!(Targeting::attack(&s, sel.clone(), &[]), None);
        // Nobody in reach: no choices.
        assert_eq!(
            weapon_choices(&s, &at(&s, p(6, 3))),
            [] as [crate::screens::battle::attack::WeaponChoice; 0]
        );
        // An empty slot has no name.
        assert_eq!(weapon_name(&s, Words::ENGLISH, UnitId(1), 2), "");
    }

    #[test]
    fn targeting_cycles_both_ways_and_keeps_the_forecast_current() {
        let c = ctx();
        let s = skirmish(&c, 20);
        let sel = at(&s, p(7, 2));
        let choice = &weapon_choices(&s, &sel)[1];
        let plain = |s: &BattleState, sel: Selection, w: &WeaponChoice| {
            let with = Equipped::Weapon(w.slot);
            Targeting::with(s, sel, with, Technique::Attack, w.targets.clone())
        };
        let mut t = plain(&s, sel, choice).unwrap();
        assert!(!t.can_swap());
        assert_eq!((t.target(), &t.with), (UnitId(6), &Equipped::Weapon(1)));
        let forecast_on = |id| {
            s.preview_attack(UnitId(1), p(7, 2), &attack(id, 1))
                .unwrap()
        };
        assert_eq!(t.preview, forecast_on(UnitId(6)));
        t.cycle(true, &s);
        assert_eq!(t.target(), UnitId(4));
        assert_eq!(t.preview, forecast_on(UnitId(4)));
        t.cycle(true, &s);
        assert_eq!(t.target(), UnitId(6));
        t.cycle(false, &s);
        assert_eq!(t.target(), UnitId(4));
        t.cycle(false, &s);
        assert_eq!(t.preview, forecast_on(UnitId(6)));
        assert_eq!(
            t.command(),
            Command::Act {
                unit: UnitId(1),
                dest: p(7, 2),
                action: attack(UnitId(6), 1),
            }
        );
        // Three targets around (7, 3): the raider left, brigand 4 right,
        // brigand 5 below. Back from the first is the last.
        let mut units = s.units().to_vec();
        units[3].pos = p(8, 3);
        units[5].pos = p(6, 3);
        let rout = Objective::Rout { turn_limit: None };
        let s3 = battle_with(&c, s.map().clone(), units, rout);
        let sel3 = at(&s3, p(7, 3));
        let three = &weapon_choices(&s3, &sel3)[0];
        assert_eq!(three.targets, [UnitId(6), UnitId(4), UnitId(5)]);
        let mut t3 = plain(&s3, sel3, three).unwrap();
        t3.cycle(false, &s3);
        assert_eq!(t3.target(), UnitId(5));
        t3.cycle(false, &s3);
        assert_eq!(t3.target(), UnitId(4));
        // Aiming at a target moves there; at anyone else, nowhere.
        t3.aim(UnitId(6), &s3);
        assert_eq!(t3.target(), UnitId(6));
        let on = |id| s3.preview_attack(UnitId(1), p(7, 3), &attack(id, 0));
        assert_eq!(Ok(&t3.preview), on(UnitId(6)).as_ref());
        t3.aim(UnitId(2), &s3);
        assert_eq!(t3.target(), UnitId(6));
        t3.aim(UnitId(5), &s3);
        assert_eq!(t3.target(), UnitId(5));
        assert_eq!(Ok(&t3.preview), on(UnitId(5)).as_ref());
        // No target: no targeting.
        let none = WeaponChoice {
            slot: 0,
            targets: vec![],
        };
        assert_eq!(plain(&s, at(&s, p(7, 2)), &none), None);
    }
}
