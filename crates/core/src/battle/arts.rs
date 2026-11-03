//! The battle's Combat Art rules (ticket 0312): validating an art for an
//! attack, paying for it, its stance and debuff, Line Pierce's strike, the
//! boss check and the attack preview. The rules are in the parent module's
//! docs.

use crate::prelude::*;

use super::{BattleState, CommandError, Event, Fight, Phase, Pos, Step, UnitAction};
use crate::art::{ArtEffect, ArtId, ArtNote};
use crate::combat::{
    CombatHp, CombatOutcome, Forecast, Reactions, Side, SideForecast, StrikePlan, StrikeReaction,
    forecast as combat_forecast, if_all_hit,
};
use crate::item::{Equipped, ItemId};
use crate::skill::{
    CostSource, EffectSource, SkillCost, SkillId, StanceReaction, TimedEffect, TimedMods,
    check_cost, pay_cost,
};
use crate::unit::{Unit, UnitId};

/// A validated use of a Combat Art in an attack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ArtUse {
    /// The art.
    pub art: ArtId,
    /// The attacking weapon's loadout slot, which pays.
    pub slot: usize,
    /// The attacking weapon.
    pub weapon: ItemId,
    /// The durability it costs.
    pub cost: u32,
    /// The weapon's durability before and after paying.
    pub durability: (u32, u32),
    /// What it does.
    pub effect: ArtEffect,
}

/// What an attack would do, for the UI's forecast panel
/// ([`BattleState::preview_attack`]): the numbers, the art or active
/// chosen, what it costs and the effects the numbers don't show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttackPreview {
    /// The combat's numbers, with every skill and art bonus.
    pub forecast: Forecast,
    /// The Combat Art chosen, if any.
    pub art: Option<ArtId>,
    /// The combat active chosen, if any.
    pub active: Option<SkillId>,
    /// The attacking weapon's durability before and after, when the art or
    /// active costs durability (`Guard Break (20 → 16)`).
    pub durability: Option<(u32, u32)>,
    /// The art's effects that aren't numbers (`no counter`, `pierces`…).
    pub notes: Vec<ArtNote>,
    /// Line Pierce's strike: the unit behind the target and the strike's
    /// numbers, if there is one to hit.
    pub pierce: Option<(UnitId, SideForecast)>,
    /// The combat if every strike hit and none crit, from both units'
    /// current HP (the forecast's strike list and kill mark).
    pub plan: StrikePlan,
}

/// Refuses a unit that may not use arts and active skills
/// ([`Unit::may_use_arts`]).
pub(super) fn check_arts_allowed(unit: &Unit) -> Result<(), CommandError> {
    if unit.may_use_arts() {
        Ok(())
    } else {
        Err(CommandError::ArtsNotAllowed(unit.id))
    }
}

/// The debuff `art`'s first hit puts on `target`, until the end of its side's
/// next phase; `None` if the art has none.
fn first_hit_effect(art: &ArtUse, target: &Unit) -> Option<TimedEffect> {
    Some(TimedEffect {
        source: EffectSource::Art(art.art.clone()),
        mods: art.effect.first_hit_mods()?,
        until: TimedEffect::debuff_until(Phase::of(target.faction)),
    })
}

/// The stance reactions of a combat of `attacker` against `defender` with
/// `forecast` ([`Reactions`]): Riposte's order if the defender counters, and
/// each side's Deflect. `stance` is a stance the attacker takes in this
/// combat but doesn't have yet (its art's, for a preview).
pub(super) fn reactions(
    attacker: &Unit,
    defender: &Unit,
    stance: Option<&TimedMods>,
    forecast: &Forecast,
) -> Reactions {
    let has = |u: &Unit, r| u.effects.iter().any(|e| e.mods.reacts(r));
    Reactions {
        defender_first: forecast.defender.is_some() && has(defender, StanceReaction::StrikesFirst),
        deflects: [
            has(attacker, StanceReaction::Deflect)
                || stance.is_some_and(|m| m.reacts(StanceReaction::Deflect)),
            has(defender, StanceReaction::Deflect),
        ],
    }
}

/// The stance `fight`'s art puts on `unit` in this combat, if `unit`
/// doesn't have it yet.
pub(super) fn new_stance<'a>(unit: &Unit, fight: &Fight<'a>) -> Option<&'a TimedMods> {
    let art = fight.art?;
    let stance = art.effect.stance.as_ref()?;
    (!unit.has_effect(&EffectSource::Art(art.art.clone()))).then_some(stance)
}

impl BattleState {
    /// Validates `unit` using art `id` in an attack with `with`: never an
    /// art used as an action of its own (Overwatch).
    pub(super) fn plan_art(
        &self,
        unit: &Unit,
        id: &ArtId,
        with: &Equipped,
    ) -> Result<ArtUse, CommandError> {
        self.art_use(unit, id, with, false)
    }

    /// Validates `unit` using art `id`, an art used as an action of its own
    /// ([`ArtEffect::is_action`]), with the weapon in `slot`.
    pub(super) fn plan_action_art(
        &self,
        unit: &Unit,
        id: &ArtId,
        slot: usize,
    ) -> Result<Step, CommandError> {
        self.check_wield(unit.id, slot)?;
        check_arts_allowed(unit)?;
        let art = self.art_use(unit, id, &Equipped::Weapon(slot), true)?;
        Ok(Step::Art(Box::new(art)))
    }

    /// Validates `unit` using art `id` with `with`, in an attack or, if
    /// `action`, as an action of its own: the art must be of that sort.
    fn art_use(
        &self,
        unit: &Unit,
        id: &ArtId,
        with: &Equipped,
        action: bool,
    ) -> Result<ArtUse, CommandError> {
        let tables = &self.tables;
        let def = tables
            .arts
            .get(id)
            .ok_or_else(|| CommandError::UnknownArt(id.clone()))?;
        let not_usable = || CommandError::ArtNotUsable {
            unit: unit.id,
            art: id.clone(),
        };
        // Arts need a weapon: never with a spell.
        let Equipped::Weapon(slot) = *with else {
            return Err(not_usable());
        };
        let fits = def.effect.is_action() == action
            && unit
                .arts_for(slot, &tables.classes, &tables.items, &tables.arts)
                .contains(&def);
        let copy = unit
            .loadout
            .weapon(slot)
            .filter(|_| fits)
            .ok_or_else(not_usable)?;
        check_cost(unit, def.skill_cost(), &CostSource::Weapon(slot)).map_err(|error| {
            CommandError::CannotPayArt {
                art: id.clone(),
                error,
            }
        })?;
        let before = copy.durability_left;
        Ok(ArtUse {
            art: id.clone(),
            slot,
            weapon: copy.def.clone(),
            cost: def.cost,
            durability: (before, before.saturating_sub(def.cost)),
            effect: def.effect.clone(),
        })
    }

    /// Line Pierce's strike for `unit` attacking `defender` as `fight` says,
    /// from `distance` tiles: the hostile unit on the tile directly behind
    /// the target (one step past it, on a straight or diagonal line from
    /// `dest`; any other angle has none) and the numbers of one strike at
    /// it, with no counter. The strike ignores range, so its forecast is
    /// worked out at the target's distance.
    pub(super) fn plan_pierce(
        &self,
        unit: &Unit,
        defender: &Unit,
        fight: &Fight,
        distance: u32,
    ) -> Result<Option<(UnitId, Forecast)>, CommandError> {
        let (dx, dy) = (defender.pos.x - fight.dest.x, defender.pos.y - fight.dest.y);
        if dx != 0 && dy != 0 && dx.abs() != dy.abs() {
            return Ok(None);
        }
        let behind = Pos::new(defender.pos.x + dx.signum(), defender.pos.y + dy.signum());
        // The attacker has left its own tile, and is never hostile to itself.
        let Some(victim) = self
            .units
            .iter()
            .find(|u| u.pos == behind && unit.faction.is_hostile_to(u.faction))
        else {
            return Ok(None);
        };
        let (a, v, _) = self.fighters(unit, victim, fight)?;
        let rules = self.tables.items.combat_rules();
        Ok(combat_forecast(&rules, &a, &v, distance).map(|mut f| {
            f.defender = None;
            f.attacker.strikes = 1;
            (victim.id, f)
        }))
    }

    /// Emits [`Event::ArtUsed`], pays for `art` (validated) by unit `id` and
    /// puts on its stance. Returns the [`Event::ItemBroke`] to emit after
    /// the combat.
    pub(super) fn commit_art(
        &mut self,
        id: UnitId,
        art: &ArtUse,
        events: &mut Vec<Event>,
    ) -> Option<Event> {
        events.push(Event::ArtUsed {
            unit: id,
            art: art.art.clone(),
            weapon: art.weapon.clone(),
            durability_before: art.durability.0,
            durability_after: art.durability.1,
        });
        let until = self.phase;
        let unit = self.unit_mut(id)?;
        let paid = pay_cost(
            unit,
            SkillCost::Durability(art.cost),
            &CostSource::Weapon(art.slot),
        )
        .ok()?;
        events.extend(paid.events);
        if let Some(stance) = &art.effect.stance {
            let source = EffectSource::Art(art.art.clone());
            unit.add_effect(TimedEffect {
                source: source.clone(),
                mods: stance.clone(),
                until,
            });
            events.push(Event::EffectApplied {
                unit: id,
                source,
                until,
            });
        }
        paid.broke
    }

    /// Unit `id` uses the action art `art` (validated, [`Step::Art`]): it
    /// equips the art's weapon, pays and takes the stance. Nothing else
    /// happens until the stance reacts ([`Self::overwatch`]).
    pub(super) fn use_action_art(&mut self, id: UnitId, art: &ArtUse, events: &mut Vec<Event>) {
        let with = Equipped::Weapon(art.slot);
        if self
            .unit(id)
            .is_some_and(|u| u.loadout.equipped.as_ref() != Some(&with))
        {
            self.equip(id, with, events);
        }
        let broke = self.commit_art(id, art, events);
        events.extend(broke);
    }

    /// Overwatch: unit `id` has just ended a move. Each unit hostile to it
    /// on an Overwatch stance whose equipped weapon reaches it shoots it
    /// once, in the battle's unit order, each stance then ending: one
    /// strike with no counter ([`Event::OverwatchShot`], then the combat),
    /// weapon EXP as an art's and unit EXP as a combat's. Returns whether
    /// `id` fell (or went down): it then doesn't act.
    pub(super) fn overwatch(&mut self, id: UnitId, events: &mut Vec<Event>) -> bool {
        let Some(mover) = self.unit(id) else {
            return false;
        };
        let watchers: Vec<(UnitId, ArtId)> = self
            .units
            .iter()
            .filter(|u| u.faction.is_hostile_to(mover.faction))
            .filter_map(|u| {
                u.effects.iter().find_map(|e| match &e.source {
                    EffectSource::Art(art) if e.mods.reacts(StanceReaction::Overwatch) => {
                        Some((u.id, art.clone()))
                    }
                    _ => None,
                })
            })
            .collect();
        for (watcher, art) in watchers {
            let Some((shot, kind)) = self.overwatch_shot(watcher, id) else {
                continue;
            };
            events.push(Event::OverwatchShot {
                unit: watcher,
                art,
                target: id,
            });
            let tally = self.clash(watcher, id, shot, events);
            self.end_reaction(watcher, StanceReaction::Overwatch, events);
            let exp = self.tables.items.rules.weapon_exp(
                tally.struck[0],
                tally.hits[0],
                tally.dealt[0],
                true,
            );
            if let Some(kind) = kind {
                self.gain_weapon_exp(watcher, kind, exp, events);
            }
            self.remove_fallen(&[id], events);
            self.award_combat(&tally, events);
            if self.unit(id).is_none() {
                return true;
            }
        }
        false
    }

    /// The numbers of unit `watcher`'s Overwatch shot at `target` where it
    /// stands, with its equipped weapon (and that weapon's kind): one strike,
    /// no counter. `None` if it has no weapon equipped or `target` is out of
    /// its range.
    fn overwatch_shot(
        &self,
        watcher: UnitId,
        target: UnitId,
    ) -> Option<(Forecast, Option<crate::weapon::WeaponKind>)> {
        let (unit, target) = (self.unit(watcher)?, self.unit(target)?);
        let with = unit.loadout.equipped.clone()?;
        if !matches!(with, Equipped::Weapon(_)) {
            return None;
        }
        let fight = Fight {
            dest: unit.pos,
            moved: 0,
            with: &with,
            active: None,
            art: None,
        };
        let (a, d, _) = self.fighters(unit, target, &fight).ok()?;
        let rules = self.tables.items.combat_rules();
        let distance = Pos::manhattan(unit.pos, target.pos);
        let mut shot = combat_forecast(&rules, &a, &d, distance)?;
        shot.defender = None;
        shot.attacker.strikes = 1;
        Some((shot, a.weapon.and_then(|w| w.kind)))
    }

    /// Ends unit `id`'s first stance with `reaction` (its Deflect, its
    /// Overwatch): it has reacted ([`Event::EffectExpired`]).
    pub(super) fn end_reaction(
        &mut self,
        id: UnitId,
        reaction: StanceReaction,
        events: &mut Vec<Event>,
    ) {
        let Some(unit) = self.unit_mut(id) else {
            return;
        };
        let Some(i) = unit.effects.iter().position(|e| e.mods.reacts(reaction)) else {
            return;
        };
        let source = unit.effects.remove(i).source;
        events.push(Event::EffectExpired { unit: id, source });
    }

    /// Ends the Deflect stance of each side of `outcome`'s combat between
    /// `units` (`[attacker, defender]`) that deflected a strike.
    pub(super) fn end_deflects(
        &mut self,
        units: [UnitId; 2],
        outcome: &CombatOutcome,
        events: &mut Vec<Event>,
    ) {
        for strike in &outcome.strikes {
            if strike.reaction == Some(StrikeReaction::Deflected) {
                let target = match strike.by {
                    Side::Attacker => units[1],
                    Side::Defender => units[0],
                };
                self.end_reaction(target, StanceReaction::Deflect, events);
            }
        }
    }

    /// Works out the numbers of `forecast` from its art's first hit on, if
    /// `fight`'s art has a debuff: the same fight, with the debuff on
    /// `defender` (replacing the one it may have, as [`Self::debuff`] will).
    pub(super) fn plan_first_hit(
        &self,
        unit: &Unit,
        defender: &Unit,
        fight: &Fight,
        forecast: &mut Forecast,
    ) -> Result<(), CommandError> {
        let Some(effect) = fight.art.and_then(|art| first_hit_effect(art, defender)) else {
            return Ok(());
        };
        let mut debuffed = defender.clone();
        debuffed.add_effect(effect);
        let (a, d, _) = self.fighters(unit, &debuffed, fight)?;
        let rules = self.tables.items.combat_rules();
        let distance = Pos::manhattan(fight.dest, defender.pos);
        if let Some(after) = combat_forecast(&rules, &a, &d, distance) {
            forecast.set_after_first_hit(&after);
        }
        Ok(())
    }

    /// Puts `art`'s debuff, if it has one, on `target` if it still stands,
    /// until the end of its side's next phase. The combat has already used
    /// it from the first hit on ([`Self::plan_first_hit`]).
    pub(super) fn debuff(&mut self, target: UnitId, art: &ArtUse, events: &mut Vec<Event>) {
        let Some(unit) = self.unit_mut(target).filter(|u| u.hp > 0) else {
            return;
        };
        let Some(effect) = first_hit_effect(art, unit) else {
            return;
        };
        let (source, until) = (effect.source.clone(), effect.until);
        unit.add_effect(effect);
        events.push(Event::EffectApplied {
            unit: target,
            source,
            until,
        });
    }

    /// What `unit` moving to `dest` and doing `action` (an attack, or an
    /// attack spell cast at a unit) would do, without doing it. It is
    /// validated exactly as [`BattleState::apply`] would validate the `Act`,
    /// with the same errors; any other action is [`CommandError::NotAnAttack`].
    pub fn preview_attack(
        &self,
        unit: UnitId,
        dest: Pos,
        action: &UnitAction,
    ) -> Result<AttackPreview, CommandError> {
        let (_, step) = self.plan(unit, dest, action)?;
        let Step::Attack(step) = step else {
            return Err(CommandError::NotAnAttack);
        };
        let active_durability = step.active.as_ref().and_then(|a| {
            let (SkillCost::Durability(cost), CostSource::Weapon(slot)) = (a.cost, &a.source)
            else {
                return None;
            };
            let left = self.unit(unit)?.loadout.weapon(*slot)?.durability_left;
            Some((left, left.saturating_sub(cost)))
        });
        let hp = |id| {
            self.unit(id)
                .map_or(CombatHp { current: 0, max: 0 }, |u| CombatHp {
                    current: u.hp,
                    max: u.stats.hp,
                })
        };
        Ok(AttackPreview {
            plan: if_all_hit(&step.forecast, hp(unit), hp(step.target)),
            forecast: step.forecast,
            art: step.art.as_ref().map(|a| a.art.clone()),
            active: step.active.as_ref().map(|a| a.skill.clone()),
            durability: step
                .art
                .as_ref()
                .map(|a| a.durability)
                .or(active_durability),
            notes: step
                .art
                .as_ref()
                .map(|a| a.effect.notes())
                .unwrap_or_default(),
            pierce: step.pierce.map(|(victim, f)| (victim, f.attacker)),
        })
    }
}
