//! The AI that plays the Enemy and Other phases (ticket 0501): which unit
//! acts next, where it moves and what it does. Pure and deterministic
//! (ADR-0004): it reads a [`BattleState`] and returns the [`Command`] to
//! apply, the same commands the player issues. The same state always gives
//! the same command.
//!
//! # Rules
//!
//! [`next_command`] plans against the current state on every call:
//!
//! - **Battle over:** `None`.
//! - **A unit waiting to move after its attack**
//!   ([`BattleState::pending_move`]) answers first, with
//!   [`Command::MoveAfter`]: to the tile, among its own and
//!   [`BattleState::move_after_tiles`], farthest (Manhattan) from the
//!   nearest unit hostile to it; its own tile on a tie, then the first
//!   tile in the order given.
//! - Otherwise every **ready** unit of the current phase (not yet acted;
//!   reinforcements that arrived this phase are already done) decides what
//!   it would do (below), and the first in this **order** acts: units that
//!   attack, highest score first; then the others, lowest flow distance
//!   (below) from their tile first; then lowest id. When no unit is left to
//!   act, `None`: the AI is done and the caller ends the phase with
//!   [`Command::EndPhase`].
//! - **Targets** are the units hostile to the acting unit
//!   ([`Faction::is_hostile_to`]): a Neutral unit has none.
//!
//! It works in any phase. The game asks it only in the Enemy and Other
//! phases; the soak tests also let it play the player's side.
//!
//! ## A unit's decision
//!
//! From the tiles it can stop on (only its own tile when
//! [`Stationary`](AiBehavior::Stationary)):
//!
//! 1. [`Healer`](AiBehavior::Healer): casts a heal spell with uses left on
//!    the most injured ally it can reach (most HP missing, then lowest id),
//!    from the tile outside the danger zone if it can, then with the best
//!    terrain, then the lowest `(y, x)`, then the lowest spell id. Else it
//!    attacks, if it can. Else it moves to the tile outside the danger zone
//!    if it can, then nearest (Manhattan) to an ally, then cheapest to
//!    reach, then the lowest `(y, x)`, and waits. The danger zone is every
//!    tile a hostile unit could attack this turn
//!    ([`danger_zone`]).
//! 2. [`Aggressive`](AiBehavior::Aggressive): attacks if it can. Else it
//!    moves to the tile with the lowest flow distance (then cheapest to
//!    reach, so it stays put rather than move for nothing, then lowest
//!    `(y, x)`) and waits; with no target it can walk to, it waits where
//!    it is.
//! 3. [`Guard`](AiBehavior::Guard): attacks if it can (a target in its
//!    threat area this turn), else holds its tile (below).
//!    [`Stationary`](AiBehavior::Stationary): the same, from its own tile.
//!
//! 4. **Escaping** (an [`Objective::Escape`] battle): a **player** unit
//!    (the AI plays the player's side for the bots) runs for the exit,
//!    whatever its behaviour: if it can stop in the exit area it escapes,
//!    from the lowest `(y, x)` such tile; else it moves as an `Aggressive`
//!    unit with no attack does, by the flow distance to the exit area, and
//!    waits. It never attacks. Enemies never escape (the battle refuses
//!    them) and green units don't need to, so they play as usual. Units
//!    that went down are no one's targets: they aren't among the battle's
//!    units.
//!
//! 5. **Saving the village** (a battle with a village goal, ticket 0313):
//!    a **player** unit that can stop on a standing house evacuates it,
//!    from the lowest `(y, x)` such tile, before anything else but
//!    escaping. A **raider** (*Claude's starting rule*, tuned by the bots),
//!    whatever its behaviour, while a house stands: if it can stop where it
//!    burns one (on a standing house or beside it), it goes there and makes
//!    its best attack from such a tile, or else waits on the one that burns
//!    the most houses (then the lowest `(y, x)`): burning comes before any
//!    other attack. Otherwise it attacks if it can, else walks towards the
//!    nearest standing house as an `Aggressive` unit with no attack walks
//!    towards its targets (the flow distance to the tiles that burn one)
//!    and waits. It is ordered by that flow distance.
//!
//! A unit **holding its tile** waits there. One that may use arts and
//! actives (below) first uses a non-combat active (Brace, War Cry…) if a
//! hostile unit could attack its tile this turn (the danger zone): the
//! first the battle accepts, lowest skill id first, with no target and then
//! on each unit (lowest id first). The battle refuses one with no use left
//! this battle (`combat-arts.md`, *Non-attack actives: uses per battle*), so
//! the unit then waits.
//!
//! **Consumables** (ticket 0513; *Claude's starting rule*, after Fire
//! Emblem's allied units): before all of the above, a **green** unit
//! carrying an item ([`Unit::carried`]) uses one on itself, where it stands,
//! when it is below half its max HP (`hp × 2 < max`) and no attack it could
//! make this turn might kill a foe (a kill chance above 0, below); its
//! first carried item the battle accepts. Otherwise it plays as above. It
//! never heals others with items. Enemies carry none (Nick, 0501), and
//! player units use the pack, which the AI never does.
//!
//! ## Attack choice
//!
//! Every stoppable tile × every target in range × every weapon the unit can
//! wield and attack spell with uses left (tile casts are never used) is
//! scored from its forecast:
//!
//! ```text
//! score = damage × E[HP dealt] + kill × P(kill) + lord × (target is a lord)
//!       − risk × E[HP taken] + terrain × (tile's Def + Avoid / 10)
//! ```
//!
//! with the weights of [`AiWeights`] (`assets/data/ai.ron`). The
//! expectations play the forecast's strikes out in order, each hitting with
//! its displayed hit chance (follow-ups included), ignoring crits and
//! stopping when a unit falls, as combat does. The highest score wins; ties
//! go to the lowest `(target id, y, x, weapon or spell id, slot)`, then to
//! the plain attack, then to the lowest art or active id (an active before
//! an art of the same id). A unit that can attack always does, whatever the
//! score.
//!
//! Scores and chances are whole numbers, so every machine plays the same
//! (ADR-0004): they count in units of 10^-16, a scale at which every chance
//! of a combat of up to 8 strikes, and so every score, is exact.
//!
//! ## Arts and actives (ticket 0503)
//!
//! Only the units the battle lets use them do ([`Unit::may_use_arts`],
//! `combat-arts.md`): bosses among enemies and combat green units (and
//! player units, when the AI plays them). For such a unit every attack
//! above is also scored with each Combat Art it can use with that weapon
//! and each of its combat actives, one per attack, from the forecast the
//! battle gives with it (its range changes included):
//!
//! ```text
//! score = the attack's score with the art or active
//!       + damage × E[HP dealt] + kill × P(kill) + lord   (Line Pierce's strike)
//!       − durability × its durability cost   (or − spell_use, a spell active)
//! ```
//!
//! Line Pierce's strike at the unit behind the target is counted as if the
//! attacker is still standing after the combat. What an art or active does
//! beyond the forecast's numbers (a debuff, a stance on later turns, a move
//! or healing after the attack) scores nothing.
//!
//! ## Flow distance
//!
//! For each tile, the cheapest cost for the mover's movement type to walk
//! from it to a tile next to one of its targets, ignoring units (the
//! engine's [`flow_field`](shuyi_core::path::flow_field), from every target
//! at once). A tile it can't walk from has none. For a player unit on an
//! escape map, the cost to walk into the exit area instead (0 inside it);
//! for a raider, the cost to walk to a tile that burns a standing house
//! (0 there).

use crate::prelude::*;

use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use core::cell::RefCell;
use core::cmp::{Ordering, Reverse};

use serde::{Deserialize, Serialize};
use shuyi_core::path;

use crate::art::ArtId;
use crate::battle::{
    BattleState, CastTarget, Command, HouseState, Objective, PendingMove, Phase, TileRect,
    UnitAction, burns,
};

use crate::combat::{CombatHp, Forecast, Side, SideForecast, strike_order};
use crate::geom::{Grid, Pos};
use crate::item::{Equipped, WEAPON_SLOTS};
use crate::magic::Affinity;
use crate::movement::{Reach, TileSet, danger_zone, reachable};
use crate::skill::{ActiveEffect, SkillCost, SkillId, SkillKind};
use crate::spell::{SpellId, SpellKind};
use crate::stats::StatValue;
use crate::terrain::MovementTypeId;
use crate::unit::{Faction, Unit, UnitId};

/// A score, or an expected number of HP, in units of 1 / [`CERTAIN`]. Wide
/// enough for any weights and HP: `u32` × `u32` × [`CERTAIN`], a few times.
type Score = i128;

/// A chance of 1: 100^8. A strike multiplies a chance by a hit percent, and
/// the game's rules give a side at most 4 strikes, so a combat's chances stay
/// whole. A longer combat rounds a branch's hit chance down and gives the
/// rest to its miss, so they still add up to this.
const CERTAIN: u64 = 10_000_000_000_000_000;

/// `n` score points.
fn points(n: u32) -> Score {
    Score::from(n) * Score::from(CERTAIN)
}

/// How an AI unit behaves. Set per unit by chapter data.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum AiBehavior {
    /// Attacks the best target in reach, else charges the nearest.
    #[default]
    Aggressive,
    /// Attacks only a target in its threat area this turn, else holds.
    Guard,
    /// Never moves: attacks only from its own tile (bosses on forts).
    Stationary,
    /// Heals the most injured ally in reach, else keeps out of danger near
    /// its allies.
    Healer,
}

/// The AI's tunable numbers (`assets/data/ai.ron`). See the module docs for
/// how each is used. The default is all zeros (every attack scores 0), a
/// placeholder for when the file fails to load; the game's values are
/// [`AiWeights::STARTING`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiWeights {
    /// Score per expected HP of damage dealt.
    pub damage: u32,
    /// Score for a certain kill (times the kill chance).
    pub kill: u32,
    /// Score for attacking a lord.
    pub lord: u32,
    /// Score lost per expected HP of damage taken.
    pub risk: u32,
    /// Score per point of the attacking tile's Def + Avoid / 10.
    pub terrain: u32,
    /// Score lost per point of durability an art or active costs.
    pub durability: u32,
    /// Score lost for the extra spell use a spell active costs.
    pub spell_use: u32,
}

impl AiWeights {
    /// The starting values (*tunable*), as in `assets/data/ai.ron`.
    pub const STARTING: AiWeights = AiWeights {
        damage: 10,
        kill: 300,
        lord: 50,
        risk: 5,
        terrain: 5,
        durability: 5,
        spell_use: 15,
    };
}

/// The command the unit acting next issues, planned against `state` (see
/// the module docs): a [`Command::MoveAfter`] or a unit's [`Command::Act`],
/// always one `state` accepts. `None` when the AI has nothing left to do:
/// the battle is over, or no unit of the phase is left to act (the caller
/// then applies [`Command::EndPhase`]).
pub fn next_command(state: &BattleState, weights: &AiWeights) -> Option<Command> {
    if state.outcome().is_some() {
        return None;
    }
    let command = if let Some(pending) = state.pending_move() {
        move_after(state, pending)
    } else {
        let planner = Planner::new(state, weights);
        state
            .units()
            .iter()
            .filter(|u| Phase::of(u.faction) == state.phase() && !u.acted)
            .filter_map(|u| planner.decide(u))
            .min_by(Decision::order)?
            .command
    };
    debug_assert!(
        state.clone().apply(&command).is_ok(),
        "the AI chose a command the battle refuses: {command:?}"
    );
    Some(command)
}

/// The waiting unit's move after its attack (see the module docs).
fn move_after(state: &BattleState, pending: PendingMove) -> Command {
    let unit = pending.unit;
    let to = state.unit(unit).and_then(|u| {
        let hostiles: Vec<Pos> = hostiles_of(state, u).map(|h| h.pos).collect();
        let room = |p: Pos| {
            hostiles
                .iter()
                .map(|&h| Pos::manhattan(h, p))
                .min()
                .unwrap_or(u32::MAX)
        };
        let mut best = (room(u.pos), None);
        for tile in state.move_after_tiles() {
            let r = room(tile);
            if r > best.0 {
                best = (r, Some(tile));
            }
        }
        best.1
    });
    Command::MoveAfter { unit, to }
}

/// The units hostile to `unit`.
fn hostiles_of<'a>(state: &'a BattleState, unit: &'a Unit) -> impl Iterator<Item = &'a Unit> {
    state
        .units()
        .iter()
        .filter(|t| unit.faction.is_hostile_to(t.faction))
}

/// What one ready unit would do, and where that puts it in the order.
#[derive(Debug, Clone)]
struct Decision {
    unit: UnitId,
    rank: Rank,
    command: Command,
}

/// A decision's place in the order, before the unit id.
#[derive(Debug, Clone, Copy)]
enum Rank {
    /// An attack with this score: first, highest first.
    Attack(Score),
    /// Anything else, by flow distance from the unit's tile: lowest first.
    Other(u32),
}

impl Decision {
    /// Which of `a` and `b` acts first (`Less`: `a`).
    fn order(a: &Decision, b: &Decision) -> Ordering {
        let rank = match (a.rank, b.rank) {
            (Rank::Attack(x), Rank::Attack(y)) => y.cmp(&x),
            (Rank::Attack(_), Rank::Other(_)) => Ordering::Less,
            (Rank::Other(_), Rank::Attack(_)) => Ordering::Greater,
            (Rank::Other(x), Rank::Other(y)) => x.cmp(&y),
        };
        rank.then(a.unit.cmp(&b.unit))
    }
}

/// A Combat Art or a combat active used with an attack.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Technique {
    Active(SkillId),
    Art(ArtId),
}

impl Technique {
    /// Its id.
    fn id(&self) -> &str {
        match self {
            Technique::Active(skill) => &skill.0,
            Technique::Art(art) => &art.0,
        }
    }
}

/// A technique an [`Arm`] can attack with.
struct Extra {
    technique: Technique,
    /// Added to the arm's max range.
    range: u32,
    /// Score its cost loses.
    penalty: Score,
}

/// Something a unit can attack with, and its range.
struct Arm {
    with: Equipped,
    min: u32,
    max: u32,
    /// The arts and combat actives to try with it, in tie-break order (none
    /// for a unit that may not use them).
    extras: Vec<Extra>,
}

impl Arm {
    /// The action attacking `target` with it, using `technique`.
    fn action(&self, target: UnitId, technique: Option<&Technique>) -> UnitAction {
        let (mut active, mut art) = (None, None);
        match technique {
            Some(Technique::Active(skill)) => active = Some(skill.clone()),
            Some(Technique::Art(id)) => art = Some(id.clone()),
            None => {}
        }
        match &self.with {
            Equipped::Weapon(slot) => UnitAction::Attack {
                target,
                slot: *slot,
                active,
                art,
            },
            Equipped::Spell(spell) => UnitAction::Cast {
                spell: spell.clone(),
                target: CastTarget::Unit(target),
                active,
            },
        }
    }

    /// The farthest it reaches with any of its techniques.
    fn reach(&self) -> u32 {
        let bonus = self.extras.iter().map(|e| e.range).max().unwrap_or(0);
        self.max.saturating_add(bonus)
    }
}

/// Flow distances by tile (see the module docs).
type Flow = Grid<Option<u32>>;

/// One `next_command` call's planning: the state and what is worth
/// working out once for every unit (flow fields, danger zones).
struct Planner<'a> {
    state: &'a BattleState,
    weights: &'a AiWeights,
    flows: RefCell<BTreeMap<(Faction, MovementTypeId), Rc<Flow>>>,
    /// The raiders' flow fields, by movement type.
    raids: RefCell<BTreeMap<MovementTypeId, Rc<Flow>>>,
    dangers: RefCell<BTreeMap<Faction, Rc<TileSet>>>,
}

impl<'a> Planner<'a> {
    fn new(state: &'a BattleState, weights: &'a AiWeights) -> Self {
        Planner {
            state,
            weights,
            flows: RefCell::default(),
            raids: RefCell::default(),
            dangers: RefCell::default(),
        }
    }

    /// What `unit` would do, or `None` if it can't act (its class or tile
    /// is unknown to the battle).
    fn decide(&self, unit: &Unit) -> Option<Decision> {
        let state = self.state;
        let class = state.classes().get(&unit.class)?;
        let reach = reachable(
            state.map(),
            state.terrain(),
            state.classes(),
            state.units(),
            unit.id,
        )
        .ok()?;
        let dests: Vec<Pos> = if unit.ai == AiBehavior::Stationary {
            vec![unit.pos]
        } else {
            reach.stoppable().iter().collect()
        };
        let flow = self.flow(unit.faction, class.movement_type);
        let distance = flow.get(unit.pos).copied().flatten().unwrap_or(u32::MAX);
        let other = |command| Decision {
            unit: unit.id,
            rank: Rank::Other(distance),
            command,
        };
        let wait_at = |dest| Command::Act {
            unit: unit.id,
            dest,
            action: UnitAction::Wait,
        };
        if let Some(area) = escape_area(state, unit) {
            let exit = dests
                .iter()
                .copied()
                .filter(|&d| area.contains(d))
                .min_by_key(|d| (d.y, d.x));
            let command = match exit {
                Some(dest) => Command::Act {
                    unit: unit.id,
                    dest,
                    action: UnitAction::Escape,
                },
                None => wait_at(approach(&reach, &dests, &flow).unwrap_or(unit.pos)),
            };
            return Some(other(command));
        }
        if let Some(dest) = evacuation(state, unit, &dests) {
            return Some(other(Command::Act {
                unit: unit.id,
                dest,
                action: UnitAction::Evacuate,
            }));
        }
        if let Some(decision) = self.raid(unit, &reach, &dests) {
            return Some(decision);
        }
        let (attack, can_kill) = self.best_attack(unit, &reach, &dests);
        if !can_kill && let Some(command) = self.own_item(unit) {
            return Some(other(command));
        }
        let attack = attack.map(|(score, command)| Decision {
            unit: unit.id,
            rank: Rank::Attack(score),
            command,
        });
        Some(match unit.ai {
            AiBehavior::Healer => match self.heal(unit, &dests) {
                Some(command) => other(command),
                None => {
                    attack.unwrap_or_else(|| other(wait_at(self.retreat(unit, &reach, &dests))))
                }
            },
            AiBehavior::Aggressive => attack.unwrap_or_else(|| {
                other(wait_at(approach(&reach, &dests, &flow).unwrap_or(unit.pos)))
            }),
            AiBehavior::Guard | AiBehavior::Stationary => attack
                .unwrap_or_else(|| other(self.stand(unit).unwrap_or_else(|| wait_at(unit.pos)))),
        })
    }

    /// The best attack `unit` can make from `dests`, with its score, and
    /// whether any attack it could make might kill.
    fn best_attack(
        &self,
        unit: &Unit,
        reach: &Reach,
        dests: &[Pos],
    ) -> (Option<(Score, Command)>, bool) {
        let state = self.state;
        let arms = arms(state, unit, self.weights);
        let Some(longest) = arms.iter().map(Arm::reach).max() else {
            return (None, false);
        };
        let mut best: Option<(Score, Command)> = None;
        let mut can_kill = false;
        let mut targets: Vec<&Unit> = hostiles_of(state, unit).collect();
        targets.sort_by_key(|t| t.id);
        let own = CombatHp {
            current: unit.hp,
            max: unit.stats.hp,
        };
        for target in targets {
            let theirs = CombatHp {
                current: target.hp,
                max: target.stats.hp,
            };
            for &dest in dests {
                let distance = Pos::manhattan(dest, target.pos);
                if distance > longest {
                    continue;
                }
                let path = reach.path_to(dest).unwrap_or_default();
                let moved = u32::try_from(path.len().saturating_sub(1)).unwrap_or(u32::MAX);
                let terrain = terrain_tenths(state, dest);
                for arm in &arms {
                    let mut offer = |score: Score, action: UnitAction| {
                        if best.as_ref().is_none_or(|(top, _)| score > *top) {
                            let command = Command::Act {
                                unit: unit.id,
                                dest,
                                action,
                            };
                            best = Some((score, command));
                        }
                    };
                    if (arm.min..=arm.max).contains(&distance)
                        && let Some(forecast) =
                            state.plain_forecast(unit, dest, moved, target, &arm.with)
                    {
                        let odds = Odds::of(&forecast, own, theirs);
                        can_kill |= odds.kill > 0;
                        let score = self.score(&odds, target.is_lord, terrain);
                        offer(score, arm.action(target.id, None));
                    }
                    for extra in &arm.extras {
                        let action = arm.action(target.id, Some(&extra.technique));
                        let Some((forecast, pierce)) =
                            state.attack_forecast(unit, dest, &path, &action)
                        else {
                            continue;
                        };
                        let odds = Odds::of(&forecast, own, theirs);
                        can_kill |= odds.kill > 0;
                        let score = self.score(&odds, target.is_lord, terrain)
                            + self.pierce_score(own, pierce)
                            - extra.penalty;
                        offer(score, action);
                    }
                }
            }
        }
        (best, can_kill)
    }

    /// The carried item a wounded green `unit` uses on itself, if any (see
    /// the module docs).
    fn own_item(&self, unit: &Unit) -> Option<Command> {
        if unit.faction != Faction::Ally || unit.hp.saturating_mul(2) >= unit.stats.hp {
            return None;
        }
        (0..unit.carried.len())
            .map(|index| UnitAction::UseOwnItem {
                index,
                target: unit.id,
            })
            .find(|action| {
                self.state
                    .check_action(unit, unit.pos, &[unit.pos], action)
                    .is_ok()
            })
            .map(|action| Command::Act {
                unit: unit.id,
                dest: unit.pos,
                action,
            })
    }

    /// What Line Pierce's strike `pierce` (the unit behind the target and
    /// the strike's forecast) adds to an attack's score (see the module
    /// docs).
    fn pierce_score(&self, own: CombatHp, pierce: Option<(UnitId, Forecast)>) -> Score {
        let Some((forecast, victim)) = pierce.and_then(|(id, f)| Some((f, self.state.unit(id)?)))
        else {
            return 0;
        };
        let theirs = CombatHp {
            current: victim.hp,
            max: victim.stats.hp,
        };
        self.score(&Odds::of(&forecast, own, theirs), victim.is_lord, 0)
    }

    /// The non-combat active `unit`, holding its tile, uses instead of
    /// waiting, if any (see the module docs).
    fn stand(&self, unit: &Unit) -> Option<Command> {
        let state = self.state;
        if !unit.may_use_arts() || !self.danger(unit.faction).contains(unit.pos) {
            return None;
        }
        let mut ids: Vec<UnitId> = state.units().iter().map(|u| u.id).collect();
        ids.sort();
        let targets: Vec<Option<UnitId>> = core::iter::once(None)
            .chain(ids.into_iter().map(Some))
            .collect();
        let action = unit
            .usable_skills(state.classes(), state.skills())
            .into_iter()
            // The battle refuses a passive, and an active with no use left.
            .filter(|def| !def.is_combat())
            .flat_map(|def| {
                targets.iter().map(|&target| UnitAction::UseSkill {
                    skill: def.id.clone(),
                    target,
                })
            })
            .find(|action| {
                state
                    .check_action(unit, unit.pos, &[unit.pos], action)
                    .is_ok()
            })?;
        Some(Command::Act {
            unit: unit.id,
            dest: unit.pos,
            action,
        })
    }

    /// An attack's score (see the module docs), from a tile with this terrain
    /// bonus in tenths.
    fn score(&self, odds: &Odds, lord: bool, terrain: i32) -> Score {
        let w = self.weights;
        let lord = if lord { points(w.lord) } else { 0 };
        Score::from(w.damage) * odds.dealt + Score::from(w.kill) * odds.kill + lord
            - Score::from(w.risk) * odds.taken
            + points(w.terrain) * Score::from(terrain) / 10
    }

    /// A healer's heal of the most injured ally it can reach, if any.
    fn heal(&self, unit: &Unit, dests: &[Pos]) -> Option<Command> {
        let state = self.state;
        let spells: Vec<_> = unit
            .learned
            .iter()
            .filter(|s| unit.spells.uses_left(s) > 0)
            .filter_map(|s| state.spells().get(s))
            .filter(|def| matches!(def.kind, SpellKind::Heal { .. }))
            .collect();
        if spells.is_empty() {
            return None;
        }
        let mut wounded: Vec<&Unit> = state
            .units()
            .iter()
            .filter(|a| {
                a.id != unit.id && unit.faction.is_allied_to(a.faction) && a.hp < a.stats.hp
            })
            .collect();
        wounded.sort_by_key(|a| (Reverse(a.stats.hp - a.hp), a.id));
        let danger = self.danger(unit.faction);
        let key = |&(dest, spell): &(Pos, &SpellId)| {
            (
                danger.contains(dest),
                Reverse(terrain_tenths(state, dest)),
                dest.y,
                dest.x,
                spell.clone(),
            )
        };
        wounded.iter().find_map(|ally| {
            let (dest, spell) = dests
                .iter()
                .flat_map(|&dest| {
                    let distance = Pos::manhattan(dest, ally.pos);
                    spells
                        .iter()
                        .filter(move |def| def.in_range(distance))
                        .map(move |def| (dest, &def.id))
                })
                .min_by_key(key)?;
            Some(Command::Act {
                unit: unit.id,
                dest,
                action: UnitAction::Cast {
                    spell: spell.clone(),
                    target: CastTarget::Unit(ally.id),
                    active: None,
                },
            })
        })
    }

    /// Where a healer with nobody to heal and nothing to attack goes.
    fn retreat(&self, unit: &Unit, reach: &Reach, dests: &[Pos]) -> Pos {
        let danger = self.danger(unit.faction);
        let allies: Vec<Pos> = self
            .state
            .units()
            .iter()
            .filter(|a| a.id != unit.id && unit.faction.is_allied_to(a.faction))
            .map(|a| a.pos)
            .collect();
        dests
            .iter()
            .copied()
            .min_by_key(|&d| {
                let near = allies.iter().map(|&a| Pos::manhattan(a, d)).min();
                (
                    danger.contains(d),
                    near.unwrap_or(0),
                    reach.cost(d).unwrap_or(u32::MAX),
                    d.y,
                    d.x,
                )
            })
            .unwrap_or(unit.pos)
    }

    /// What raider `unit` does while a house stands (see the module docs);
    /// `None` if it isn't a raider or no house stands.
    fn raid(&self, unit: &Unit, reach: &Reach, dests: &[Pos]) -> Option<Decision> {
        let state = self.state;
        let standing: Vec<Pos> = raidable(state, unit)?;
        let burned = |d: Pos| standing.iter().filter(|&&h| burns(d, h)).count();
        let burning: Vec<Pos> = dests.iter().copied().filter(|&d| burned(d) > 0).collect();
        let class = state.classes().get(&unit.class)?;
        let flow = self.raid_flow(class.movement_type, &standing);
        let distance = flow.get(unit.pos).copied().flatten().unwrap_or(u32::MAX);
        let wait_at = |dest| Decision {
            unit: unit.id,
            rank: Rank::Other(distance),
            command: Command::Act {
                unit: unit.id,
                dest,
                action: UnitAction::Wait,
            },
        };
        let attack = |dests: &[Pos]| {
            let (attack, _) = self.best_attack(unit, reach, dests);
            attack.map(|(score, command)| Decision {
                unit: unit.id,
                rank: Rank::Attack(score),
                command,
            })
        };
        if !burning.is_empty() {
            let most = burning
                .iter()
                .copied()
                .min_by_key(|&d| (Reverse(burned(d)), d.y, d.x))?;
            return Some(attack(&burning).unwrap_or_else(|| wait_at(most)));
        }
        Some(
            attack(dests)
                .unwrap_or_else(|| wait_at(approach(reach, dests, &flow).unwrap_or(unit.pos))),
        )
    }

    /// The flow distances to the tiles that burn a house of `standing`,
    /// for `movement_type`.
    fn raid_flow(&self, movement_type: MovementTypeId, standing: &[Pos]) -> Rc<Flow> {
        let mut raids = self.raids.borrow_mut();
        Rc::clone(raids.entry(movement_type).or_insert_with(|| {
            let state = self.state;
            let tiles = &state.map().tiles;
            let goals = tiles
                .positions()
                .filter(|&p| standing.iter().any(|&h| burns(p, h)));
            Rc::new(path::flow_field(
                tiles.width(),
                tiles.height(),
                goals,
                |pos| {
                    let terrain = tiles.get(pos)?;
                    state
                        .terrain()
                        .move_cost(*terrain, movement_type)
                        .map(u32::from)
                },
            ))
        }))
    }

    /// The flow distances to `faction`'s targets for `movement_type`.
    fn flow(&self, faction: Faction, movement_type: MovementTypeId) -> Rc<Flow> {
        let mut flows = self.flows.borrow_mut();
        Rc::clone(
            flows
                .entry((faction, movement_type))
                .or_insert_with(|| Rc::new(flow_field(self.state, faction, movement_type))),
        )
    }

    /// Every tile a unit hostile to `faction` could attack this turn.
    fn danger(&self, faction: Faction) -> Rc<TileSet> {
        let state = self.state;
        let mut dangers = self.dangers.borrow_mut();
        Rc::clone(dangers.entry(faction).or_insert_with(|| {
            let (classes, items, spells) = (state.classes(), state.items(), state.spells());
            let zone = danger_zone(
                state.map(),
                state.terrain(),
                classes,
                state.units(),
                faction,
                |u| u.attack_ranges(classes, items, spells),
            );
            let tiles = &state.map().tiles;
            Rc::new(zone.unwrap_or_else(|_| TileSet::new(tiles.width(), tiles.height())))
        }))
    }
}

/// What `unit` can attack with: every weapon it can wield, then every
/// attack spell with uses left, ordered by `(id, slot)`, each with the
/// techniques to try with it.
fn arms(state: &BattleState, unit: &Unit, weights: &AiWeights) -> Vec<Arm> {
    let mut arms: Vec<(String, usize, Arm)> = Vec::new();
    if let Some(class) = state.classes().get(&unit.class) {
        for slot in 0..WEAPON_SLOTS {
            if let Some((copy, def)) = unit.usable_weapon(slot, class, state.items()) {
                let with = Equipped::Weapon(slot);
                let arm = Arm {
                    extras: extras(state, unit, &with, weights),
                    with,
                    min: def.min_range,
                    max: def.max_range,
                };
                arms.push((copy.def.0.clone(), slot, arm));
            }
        }
    }
    for spell in &unit.learned {
        if let Some(def) = unit.castable_attack(spell, state.spells()) {
            let with = Equipped::Spell(spell.clone());
            let arm = Arm {
                extras: extras(state, unit, &with, weights),
                with,
                min: def.min_range,
                max: def.max_range,
            };
            arms.push((spell.0.clone(), WEAPON_SLOTS, arm));
        }
    }
    arms.sort_by(|a, b| (&a.0, a.1).cmp(&(&b.0, b.1)));
    arms.into_iter().map(|(_, _, arm)| arm).collect()
}

/// The techniques `unit` tries when attacking with `with`: its combat
/// actives, and with a weapon the Combat Arts it can pay for, ordered by id
/// (an active first). None for a unit that may not use them. The battle
/// refuses those that don't fit the attack ([`BattleState::attack_forecast`]).
fn extras(state: &BattleState, unit: &Unit, with: &Equipped, weights: &AiWeights) -> Vec<Extra> {
    if !unit.may_use_arts() {
        return Vec::new();
    }
    let mut out: Vec<Extra> = unit
        .usable_skills(state.classes(), state.skills())
        .into_iter()
        .filter_map(|def| match &def.kind {
            SkillKind::Active {
                cost,
                effect: ActiveEffect::Strike { range, .. },
            } => Some(Extra {
                technique: Technique::Active(def.id.clone()),
                range: *range,
                penalty: match cost {
                    SkillCost::Durability(cost) => points(weights.durability) * Score::from(*cost),
                    SkillCost::ExtraSpellUse => points(weights.spell_use),
                    // Never a combat active's cost (the content check
                    // refuses it, and so does the battle).
                    SkillCost::Uses(_) => 0,
                },
            }),
            _ => None,
        })
        .collect();
    if let Equipped::Weapon(slot) = *with {
        let arts = unit.usable_arts(slot, state.classes(), state.items(), state.arts());
        out.extend(arts.into_iter().map(|art| Extra {
            technique: Technique::Art(art.id.clone()),
            range: 0,
            penalty: points(weights.durability) * Score::from(art.cost),
        }));
    }
    out.sort_by(|a, b| (a.technique.id(), &a.technique).cmp(&(b.technique.id(), &b.technique)));
    out
}

/// The terrain bonus of standing on `pos`, in tenths: Def × 10 + Avoid.
fn terrain_tenths(state: &BattleState, pos: Pos) -> i32 {
    state
        .map()
        .tiles
        .get(pos)
        .and_then(|&t| state.terrain().get(t))
        .map_or(0, |r| i32::from(r.defense) * 10 + i32::from(r.avoid))
}

/// The tile of `dests` an attack-less `Aggressive` unit moves to, if any
/// has a flow distance.
fn approach(reach: &Reach, dests: &[Pos], flow: &Flow) -> Option<Pos> {
    dests
        .iter()
        .filter_map(|&d| {
            let distance = flow.get(d).copied().flatten()?;
            Some((distance, reach.cost(d).unwrap_or(u32::MAX), d.y, d.x))
        })
        .min()
        .map(|(_, _, y, x)| Pos::new(x, y))
}

/// The tile of `dests` player unit `unit` evacuates a house from, if any:
/// the lowest `(y, x)` standing house.
fn evacuation(state: &BattleState, unit: &Unit, dests: &[Pos]) -> Option<Pos> {
    if unit.faction != Faction::Player {
        return None;
    }
    let village = state.village()?;
    dests
        .iter()
        .copied()
        .filter(|&d| village.house(d) == Some(HouseState::Standing))
        .min_by_key(|d| (d.y, d.x))
}

/// The standing houses raider `unit` goes for; `None` if it isn't a raider
/// or none stands.
fn raidable(state: &BattleState, unit: &Unit) -> Option<Vec<Pos>> {
    if !unit.raider || unit.faction != Faction::Enemy {
        return None;
    }
    let standing: Vec<Pos> = state.village()?.standing().collect();
    (!standing.is_empty()).then_some(standing)
}

/// The exit area `unit` runs for: a player unit's on an escape map.
fn escape_area(state: &BattleState, unit: &Unit) -> Option<TileRect> {
    match state.objective() {
        Objective::Escape { area } if unit.faction == Faction::Player => Some(area),
        _ => None,
    }
}

/// The flow distance of every tile to `faction`'s targets for
/// `movement_type` (see the module docs).
fn flow_field(state: &BattleState, faction: Faction, movement_type: MovementTypeId) -> Flow {
    let tiles = &state.map().tiles;
    let (width, height) = (tiles.width(), tiles.height());
    let cost = |pos| {
        let step = tiles
            .get(pos)
            .and_then(|&t| state.terrain().move_cost(t, movement_type))?;
        Some(u32::from(step))
    };
    if let Objective::Escape { area } = state.objective()
        && faction == Faction::Player
    {
        let exit = tiles.positions().filter(|&p| area.contains(p));
        return path::flow_field(width, height, exit, cost);
    }
    let mut targets = TileSet::new(width, height);
    for t in state
        .units()
        .iter()
        .filter(|t| faction.is_hostile_to(t.faction))
    {
        targets.insert(t.pos);
    }
    // Walking onto a tile costs entering it, but the last step, onto a
    // target's tile, is free: only the tiles next to it count.
    path::flow_field(width, height, targets.iter(), |pos| {
        if targets.contains(pos) {
            return Some(0);
        }
        cost(pos)
    })
}

/// The expected result of a combat, played out from its forecast.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Odds {
    /// Expected HP the defender loses.
    dealt: Score,
    /// Expected HP the attacker loses.
    taken: Score,
    /// Chance the defender falls.
    kill: Score,
}

impl Odds {
    /// The odds of `forecast` between an attacker and a defender with these
    /// HP (see the module docs).
    fn of(forecast: &Forecast, attacker: CombatHp, defender: CombatHp) -> Odds {
        let walk = Walk {
            forecast,
            order: strike_order(forecast),
            start: [attacker.current, defender.current],
            max: [attacker.max, defender.max],
        };
        let mut odds = Odds::default();
        walk.strike(0, walk.start, [0, 0], CERTAIN, &mut odds);
        odds
    }
}

/// Every hit-or-miss branch of a combat's strikes.
struct Walk<'a> {
    forecast: &'a Forecast,
    order: Vec<Side>,
    /// HP going in: `[attacker, defender]`.
    start: [StatValue; 2],
    /// Max HP: `[attacker, defender]`.
    max: [StatValue; 2],
}

impl Walk<'_> {
    /// Plays strike `i` on, from `hp` with `struck` strikes made per side,
    /// on a branch of this `chance`, adding its outcomes to `odds`.
    fn strike(&self, i: usize, hp: [StatValue; 2], struck: [u8; 2], chance: u64, odds: &mut Odds) {
        let striker = self.order.get(i).and_then(|&by| match by {
            Side::Attacker => Some((0, self.forecast.attacker)),
            Side::Defender => self.forecast.defender.map(|d| (1, d)),
        });
        let over = hp[0] <= 0 || hp[1] <= 0;
        let Some((me, numbers)) = striker.filter(|_| !over) else {
            let share = Score::from(chance);
            odds.dealt += share * Score::from((self.start[1] - hp[1]).max(0));
            odds.taken += share * Score::from((self.start[0] - hp[0]).max(0));
            if hp[1] <= 0 {
                odds.kill += share;
            }
            return;
        };
        let mut struck = struck;
        struck[me] += 1;
        // Both branches, even one with no chance: it adds nothing.
        let hits = chance * u64::from(numbers.hit) / 100;
        let mut after = hp;
        let target = 1 - me;
        after[target] = struck_hp(&numbers, struck[me], hp[target], self.max[target]);
        self.strike(i + 1, after, struck, hits, odds);
        self.strike(i + 1, hp, struck, chance - hits, odds);
    }
}

/// A target's HP after the `nth` strike of a side with `numbers` hits it
/// without a crit (an Absorb strike heals it, up to `max`).
fn struck_hp(numbers: &SideForecast, nth: u8, hp: StatValue, max: StatValue) -> StatValue {
    let damage = if nth > 1 {
        numbers.followup_damage
    } else {
        numbers.damage
    };
    if numbers.affinity == Some(Affinity::Absorb) {
        (hp + damage).min(max)
    } else {
        (hp - damage).max(0)
    }
}

#[cfg(test)]
mod tests;
