//! The battle's Combat Art rules (ticket 0312): validating an art for an
//! attack, paying for it, its stance and debuff, the strikes at other units
//! (Line Pierce's, an area art's, Chain Strike's allies'; 0327), the boss
//! check and the attack preview. The rules are in the parent module's
//! docs.

use crate::prelude::*;

use super::{BattleState, CommandError, Event, Fight, Phase, Pos, Step, UnitAction};
use crate::art::{ArtArea, ArtEffect, ArtId, ArtNote};
use crate::combat::{
    CombatHp, CombatOutcome, Forecast, Reactions, Side, SideForecast, StrikePlan, StrikeReaction,
    forecast as combat_forecast, if_all_hit,
};
use crate::item::{Equipped, ItemId};
use crate::skill::{
    CostSource, EffectSource, SkillCost, SkillId, StanceReaction, TimedEffect, TimedMods,
    check_cost, pay_cost,
};
use crate::stats::StatValue;
use crate::unit::{Role, Unit, UnitId};

/// The four tiles next to a tile, clockwise from the north: `(dx, dy)`, `y`
/// growing south.
const AROUND: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

/// One strike of an art at a unit, after the combat (Line Pierce's, Chain
/// Strike's allies') or instead of it (an [`ArtArea`]'s): by unit `by` at
/// unit `at`, with no counter and no extra strikes. Its numbers are worked
/// out when the attack is validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ArtStrike {
    /// Who strikes: the art's user, or an ally of it (Chain Strike).
    pub by: UnitId,
    /// The unit struck.
    pub at: UnitId,
    /// The strike's numbers (attacker side only, 1 strike).
    pub forecast: Forecast,
}

/// One row of the forecast's strikes at units ([`AttackPreview::strikes`]):
/// one strike by `by` at `at`, and `at`'s HP before it and after it if it
/// hits (no crit), after the rows before it (and the combat, if every
/// strike of it hit).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrikePreview {
    /// Who strikes.
    pub by: UnitId,
    /// The unit struck.
    pub at: UnitId,
    /// The strike's numbers.
    pub numbers: SideForecast,
    /// `at`'s HP before the strike and after it if it hits.
    pub hp: (StatValue, StatValue),
}

/// The tiles `area` strikes for a user at `from` aiming at `target`, in
/// strike order; `None` if `target` isn't where the area can aim (Wide
/// Swing: not next to the user; Comet Lance: not on its row or column).
pub(crate) fn area_tiles(area: ArtArea, from: Pos, target: Pos) -> Option<Vec<Pos>> {
    let (dx, dy) = (target.x - from.x, target.y - from.y);
    let at = |p: Pos, (x, y): (i32, i32)| Pos::new(p.x + x, p.y + y);
    match area {
        // Facing `(dx, dy)`: left is `(dy, -dx)`, right `(-dy, dx)`.
        ArtArea::Cleave if dx.abs() + dy.abs() == 1 => {
            Some(vec![target, at(from, (dy, -dx)), at(from, (-dy, dx))])
        }
        ArtArea::Line(length) if (dx == 0) != (dy == 0) => {
            let step = (dx.signum(), dy.signum());
            let steps = 1..=i32::try_from(length).unwrap_or(i32::MAX);
            Some(steps.map(|k| at(from, (step.0 * k, step.1 * k))).collect())
        }
        ArtArea::Burst => {
            let around = AROUND.iter().map(|&d| at(target, d));
            Some(core::iter::once(target).chain(around).collect())
        }
        ArtArea::Cleave | ArtArea::Line(_) => None,
    }
}

/// The tile directly behind `target` for Line Pierce from `from`: one step
/// past it on the straight or diagonal line; any other angle has none.
fn behind(from: Pos, target: Pos) -> Option<Pos> {
    let (dx, dy) = (target.x - from.x, target.y - from.y);
    if dx != 0 && dy != 0 && dx.abs() != dy.abs() {
        return None;
    }
    Some(Pos::new(target.x + dx.signum(), target.y + dy.signum()))
}

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

/// A ready unit's move to a tile it can stop on, its path searched once
/// ([`BattleState::plan_move`]), so the attacks it could make there are
/// previewed without searching again ([`BattleState::preview_attack_after`]).
/// Only good in the state that planned it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedMove {
    /// The unit.
    pub(super) unit: UnitId,
    /// Where it stops.
    pub(super) dest: Pos,
    /// Its path there, its tile first.
    pub(super) path: Vec<Pos>,
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
    /// The art's strikes at units, in strike order: Line Pierce's and Chain
    /// Strike's after the combat, or an area art's (the target's among
    /// them) instead of it; one row each. Empty for any other attack.
    pub strikes: Vec<StrikePreview>,
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

    /// The strikes at units of `fight`'s art for `unit` attacking
    /// `defender` from `distance` tiles, in strike order: Line Pierce's at
    /// the hostile unit directly behind the target, an area's at each
    /// hostile unit on its tiles ([`area_tiles`]; [`CommandError::OutOfRange`]
    /// if the target isn't where it aims), then Chain Strike's by each ally
    /// next to the target. None without an art.
    pub(super) fn plan_strikes(
        &self,
        unit: &Unit,
        defender: &Unit,
        fight: &Fight,
        distance: u32,
    ) -> Result<Vec<ArtStrike>, CommandError> {
        let Some(art) = fight.art else {
            return Ok(Vec::new());
        };
        let effect = &art.effect;
        let mut tiles = Vec::new();
        if effect.line_pierce {
            tiles.extend(behind(fight.dest, defender.pos));
        }
        if let Some(area) = effect.area {
            let aimed = area_tiles(area, fight.dest, defender.pos);
            tiles.extend(aimed.ok_or(CommandError::OutOfRange {
                target: defender.id,
                distance,
            })?);
        }
        let mut out = Vec::new();
        for tile in tiles {
            // The attacker has left its own tile, and is never hostile to itself.
            let hostile = |u: &&Unit| u.pos == tile && unit.faction.is_hostile_to(u.faction);
            if let Some(victim) = self.units.iter().find(hostile)
                && let Some(forecast) = self.one_strike(unit, victim, fight, distance)?
            {
                out.push(ArtStrike {
                    by: unit.id,
                    at: victim.id,
                    forecast,
                });
            }
        }
        if effect.chain {
            out.extend(self.chain_strikes(unit, defender));
        }
        Ok(out)
    }

    /// The numbers of one strike by `unit` at `victim` as `fight` says, with
    /// no counter. The strikes at units other than the target ignore range,
    /// so all are worked out at the target's `distance`.
    fn one_strike(
        &self,
        unit: &Unit,
        victim: &Unit,
        fight: &Fight,
        distance: u32,
    ) -> Result<Option<Forecast>, CommandError> {
        let (a, v, _) = self.fighters(unit, victim, fight)?;
        let rules = self.tables.items.combat_rules();
        Ok(combat_forecast(&rules, &a, &v, distance).map(|mut f| {
            f.defender = None;
            f.attacker.strikes = 1;
            f
        }))
    }

    /// Chain Strike's strikes at `target` after `user`'s combat: one by each
    /// ally of `user` on a tile next to `target`, clockwise from the north,
    /// whose equipped weapon reaches it (green combat units too, never a
    /// non-combat one), with its own weapon and stats where it stands.
    fn chain_strikes(&self, user: &Unit, target: &Unit) -> Vec<ArtStrike> {
        let rules = self.tables.items.combat_rules();
        let ally = |tile: Pos| {
            self.units.iter().find(|u| {
                u.pos == tile
                    && u.id != user.id
                    && !user.faction.is_hostile_to(u.faction)
                    && u.faction.is_hostile_to(target.faction)
                    && u.role != Role::Noncombatant
            })
        };
        let mut out = Vec::new();
        for (dx, dy) in AROUND {
            let Some(ally) = ally(Pos::new(target.pos.x + dx, target.pos.y + dy)) else {
                continue;
            };
            let Some(with @ Equipped::Weapon(_)) = ally.loadout.equipped.clone() else {
                continue;
            };
            let fight = Fight {
                dest: ally.pos,
                moved: 0,
                with: &with,
                active: None,
                art: None,
            };
            let Ok((a, d, _)) = self.fighters(ally, target, &fight) else {
                continue;
            };
            let Some(mut forecast) = combat_forecast(&rules, &a, &d, 1) else {
                continue;
            };
            if forecast.attacker.broken {
                continue;
            }
            forecast.defender = None;
            forecast.attacker.strikes = 1;
            out.push(ArtStrike {
                by: ally.id,
                at: target.id,
                forecast,
            });
        }
        out
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
        self.preview_attack_after(&self.plan_move(unit, dest)?, action)
    }

    /// [`Self::preview_attack`] after `planned`, a move planned in this
    /// state: the same answer, without searching the move's path again.
    pub fn preview_attack_after(
        &self,
        planned: &PlannedMove,
        action: &UnitAction,
    ) -> Result<AttackPreview, CommandError> {
        let unit = planned.unit;
        let Step::Attack(step) = self.plan_step_after(planned, action)? else {
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
        let plan = if_all_hit(&step.forecast, hp(unit), hp(step.target));
        // Each struck unit's HP as the rows go: the target's after the
        // combat, if there is one.
        let mut left: Vec<(UnitId, StatValue)> = Vec::new();
        if step.fights() {
            left.push((step.target, plan.defender_hp));
        }
        let strikes = step
            .strikes
            .iter()
            .map(|s| {
                let before = left
                    .iter()
                    .rev()
                    .find(|(id, _)| *id == s.at)
                    .map_or_else(|| hp(s.at).current, |&(_, hp)| hp);
                let after = (before - s.forecast.attacker.damage).max(0);
                left.push((s.at, after));
                StrikePreview {
                    by: s.by,
                    at: s.at,
                    numbers: s.forecast.attacker,
                    hp: (before, after),
                }
            })
            .collect();
        Ok(AttackPreview {
            plan,
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
            strikes,
        })
    }
}
