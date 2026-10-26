//! The persona bots (ticket 0506, `docs/design/playtest-bots.md`): Casual,
//! Normal and Hardcore, each the engine's turn planner ([`TurnPlan`],
//! engine#2128) with settings and a scoring of its own, read from
//! `crates/bots/personas.ron`.
//!
//! Skill is the thinking budget and the mistakes ([`PlanSettings`]: how
//! many commands it weighs at each step, how many partial turns it keeps,
//! how many of its own commands a plan holds, on how many copies, how far
//! its pick may stray from the best); the player type is what it scores
//! ([`Weights`]). The search is the engine's: it plans only the persona's
//! own turn and never plays the enemy's out (Nick, 2026-10-21: a bot
//! knows the danger zone and the forecasts, not the enemy's mind). The
//! game's parts are here: which commands are tried first ([`order`]), the
//! scoring ([`Tally`], with the danger the units stand in) and the talk
//! rule ([`PersonaBot`]).

use std::cmp::Reverse;
use std::collections::BTreeMap;

use serde::Deserialize;
use shuyi_core::grid::Grid;
use shuyi_core::path::flow_field;
use shuyi_core::{Bot, Evaluate, Order, PlanSettings, TurnPlan};
use trpg_core::{
    AiBehavior, AiWeights, BattleState, CastTarget, Command, Faction, GameMode, Objective, Outcome,
    Phase, Pos, SideForecast, Unit, UnitAction, UnitId, legal_commands, next_command, threat_area,
};

/// Points for each thing a playout changed, added up into its score
/// (higher is better): a cost is a negative weight. What is counted is
/// [`Tally`]'s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Weights {
    /// The battle won.
    pub won: i32,
    /// The battle lost.
    pub lost: i32,
    /// Each lord that fell.
    pub lord_fell: i32,
    /// Each other player unit that fell (died, or retreated in Casual).
    pub unit_fell: i32,
    /// Each HP the standing player units other than lords lost (a heal
    /// counts back).
    pub hp_lost: i32,
    /// Each HP the standing lords lost.
    pub lord_hp_lost: i32,
    /// Each enemy that fell.
    pub enemy_fell: i32,
    /// Each step towards the objective ([`Tally::progress`]).
    pub progress: i32,
    /// Each tile the player's units came closer to their goal
    /// ([`Tally::approach`]).
    pub approach: i32,
    /// Each HP of a non-lord player unit the enemies threatening it are
    /// expected to take, by their forecasts ([`Tally::exposed`]).
    pub exposed: i32,
    /// The same for the lords.
    pub lord_exposed: i32,
    /// Each non-lord player unit that falls if every enemy threatening it
    /// strikes and hits ([`Tally::at_risk`]).
    pub at_risk: i32,
    /// The same for the lords.
    pub lord_at_risk: i32,
    /// Each enemy HP the player units are expected to take back when the
    /// enemies that can reach them strike ([`Tally::counter`]).
    pub counter: i32,
    /// Each turn that began.
    pub turns: i32,
    /// Each battle-pack item used.
    pub items: i32,
    /// Each house saved, less each house burned ([`Tally::houses`]).
    pub houses: i32,
}

/// One player type: the mode it plays and how it thinks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Persona {
    /// The game mode it plays in.
    pub mode: GameMode,
    /// Its thinking budget and mistakes.
    pub settings: PlanSettings,
    /// What it cares about.
    pub weights: Weights,
}

/// The personas by name, as `personas.ron` lists them.
pub type Personas = BTreeMap<String, Persona>;

/// Reads `personas.ron`.
pub fn personas_from_ron(text: &str) -> Result<Personas, String> {
    ron::from_str(text).map_err(|e| format!("personas: {e}"))
}

/// What changed for the player between the state a decision began in and
/// the end of a playout. A [`Persona`] weighs each count ([`Tally::score`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Tally {
    /// 1 if the battle was won.
    pub won: i64,
    /// 1 if the battle was lost.
    pub lost: i64,
    /// Lords that fell.
    pub lord_fell: i64,
    /// Other player units that fell.
    pub unit_fell: i64,
    /// HP the standing non-lord player units lost (less what they healed).
    pub hp_lost: i64,
    /// HP the standing lords lost (less what they healed).
    pub lord_hp_lost: i64,
    /// Enemies that fell, or went down (a boss who can't be killed).
    pub enemy_fell: i64,
    /// Progress towards the objective: in a rout, the enemies' HP taken; in
    /// a boss fight, the boss's; in a seize, how many tiles closer the
    /// nearest unit that may seize got; in an escape (0319), how many units
    /// got out; nothing in a survive.
    pub progress: i64,
    /// Tiles the standing player units came closer to their goal, all
    /// added up: the nearest enemy in a rout, the boss, the tile to seize,
    /// the exit area (a unit that escaped reached it); nothing in a
    /// survive. Without it a search that looks one reply
    /// ahead has no reason to walk towards enemies that hold their ground.
    pub approach: i64,
    /// Where the playout ends, the HP the non-lord player units stand to
    /// lose in the enemies' next phase: for each, the expected damage of
    /// every enemy that can reach it (the danger zone; one that never moves
    /// only from its own tile), by the forecast a player reads off, at most
    /// its HP. Whom the enemies will choose isn't known, so every threat
    /// counts. It is the same at the start for every candidate, so it is
    /// counted at the end only.
    pub exposed: i64,
    /// The same for the lords, unless a lord is the last player unit
    /// standing: then waiting can't win, and its danger isn't counted.
    pub lord_exposed: i64,
    /// Where the playout ends, the non-lord player units that would fall if
    /// every enemy that can reach them struck and hit.
    pub at_risk: i64,
    /// The same for the lords.
    pub lord_at_risk: i64,
    /// Where the playout ends, the HP the player units are expected to take
    /// back from the enemies that can reach them, by the same forecasts
    /// (the counter side), at most each enemy's HP: a sturdy unit in a
    /// guard's reach is bait as well as a target.
    pub counter: i64,
    /// Turns that began.
    pub turns: i64,
    /// Battle-pack items used.
    pub items: i64,
    /// Houses saved less houses burned, in a battle with a village goal
    /// (0313).
    pub houses: i64,
}

impl Tally {
    /// What changed from `start` to `end`, a later state of the same battle.
    pub fn between(start: &BattleState, end: &BattleState) -> Tally {
        let danger = danger(end);
        let fell = end.fallen().get(start.fallen().len()..).unwrap_or_default();
        let fell_count = |keep: &dyn Fn(&Unit) -> bool| count(fell.iter().filter(|u| keep(u)));
        let went_down = end.down().get(start.down().len()..).unwrap_or_default();
        let enemy = |u: &&Unit| u.faction == Faction::Enemy;
        let hp_lost = |lord: bool| -> i64 {
            start
                .units()
                .iter()
                .filter(|u| u.faction == Faction::Player && u.is_lord == lord)
                .filter_map(|u| Some(i64::from(u.hp - end.unit(u.id)?.hp)))
                .sum()
        };
        Tally {
            won: i64::from(end.outcome() == Some(Outcome::Victory)),
            lost: i64::from(end.outcome() == Some(Outcome::Defeat)),
            lord_fell: fell_count(&|u| u.faction == Faction::Player && u.is_lord),
            unit_fell: fell_count(&|u| u.faction == Faction::Player && !u.is_lord),
            hp_lost: hp_lost(false),
            lord_hp_lost: hp_lost(true),
            enemy_fell: count(fell.iter().chain(went_down).filter(enemy)),
            progress: progress(start, end),
            approach: approach(start, end),
            exposed: danger.exposed,
            lord_exposed: danger.lord_exposed,
            at_risk: danger.at_risk,
            lord_at_risk: danger.lord_at_risk,
            counter: danger.counter,
            turns: i64::from(end.turn()) - i64::from(start.turn()),
            items: count(start.pack().items.iter()) - count(end.pack().items.iter()),
            houses: houses(end) - houses(start),
        }
    }

    /// The counts weighed by `weights`, in whole points.
    pub fn score(&self, weights: &Weights) -> i32 {
        let terms = [
            (self.won, weights.won),
            (self.lost, weights.lost),
            (self.lord_fell, weights.lord_fell),
            (self.unit_fell, weights.unit_fell),
            (self.hp_lost, weights.hp_lost),
            (self.lord_hp_lost, weights.lord_hp_lost),
            (self.enemy_fell, weights.enemy_fell),
            (self.progress, weights.progress),
            (self.approach, weights.approach),
            (self.exposed, weights.exposed),
            (self.lord_exposed, weights.lord_exposed),
            (self.at_risk, weights.at_risk),
            (self.lord_at_risk, weights.lord_at_risk),
            (self.counter, weights.counter),
            (self.turns, weights.turns),
            (self.items, weights.items),
            (self.houses, weights.houses),
        ];
        let total: i64 = terms.iter().map(|&(n, w)| n * i64::from(w)).sum();
        i32::try_from(total.clamp(i32::MIN.into(), i32::MAX.into())).unwrap_or(0)
    }
}

/// Houses saved less houses burned so far.
fn houses(state: &BattleState) -> i64 {
    state
        .village()
        .map_or(0, |v| i64::from(v.saved()) - i64::from(v.lost()))
}

fn count<T>(items: impl Iterator<Item = T>) -> i64 {
    i64::try_from(items.count()).unwrap_or(i64::MAX)
}

/// [`Tally::progress`].
fn progress(start: &BattleState, end: &BattleState) -> i64 {
    match start.objective() {
        Objective::Rout { .. } => enemy_hp(start) - enemy_hp(end),
        Objective::DefeatUnit { unit, .. } => hp_of(start, unit) - hp_of(end, unit),
        Objective::Seize { pos, by_lord, .. } => {
            seize_distance(start, pos, by_lord) - seize_distance(end, pos, by_lord)
        }
        Objective::Escape { .. } => count(end.escaped().iter()) - count(start.escaped().iter()),
        Objective::Survive { .. } => 0,
    }
}

/// [`Tally::approach`].
fn approach(start: &BattleState, end: &BattleState) -> i64 {
    let (Some(mut before), Some(mut after)) = (Goals::of(start), Goals::of(end)) else {
        return 0;
    };
    let player = start
        .units()
        .iter()
        .filter(|u| u.faction == Faction::Player);
    player
        .filter_map(|u| {
            let from = before.distance(u, u.pos)?;
            if end.escaped().iter().any(|e| e.id == u.id) {
                return Some(from);
            }
            let now = end.unit(u.id)?;
            Some(from - after.distance(now, now.pos)?)
        })
        .sum()
}

/// How far, walking, each tile is from the objective's goals (an enemy in
/// a rout, the boss, the tile to seize, the exit area's tiles; and a village's
/// standing houses), worked out once per movement type:
/// the engine's flow field, as the enemy AI walks ([`next_command`]).
struct Goals<'a> {
    state: &'a BattleState,
    targets: Vec<Pos>,
    fields: BTreeMap<u8, Grid<Option<u32>>>,
}

impl<'a> Goals<'a> {
    /// The goals of `state`'s objective; `None` in a survive, or with none
    /// left.
    fn of(state: &'a BattleState) -> Option<Goals<'a>> {
        let goals: Vec<Pos> = match state.objective() {
            Objective::Rout { .. } => {
                let enemies = state.units().iter().filter(|u| u.faction == Faction::Enemy);
                enemies.map(|u| u.pos).collect()
            }
            Objective::DefeatUnit { unit, .. } => {
                state.unit(unit).map(|u| u.pos).into_iter().collect()
            }
            Objective::Seize { pos, .. } => vec![pos],
            Objective::Escape { area } => {
                let tiles = state.map().tiles.positions();
                tiles.filter(|&p| area.contains(p)).collect()
            }
            Objective::Survive { .. } => Vec::new(),
        };
        // The houses still standing are goals too: a race to them (0313).
        let houses = state
            .village()
            .into_iter()
            .flat_map(trpg_core::Village::standing);
        let goals: Vec<Pos> = goals.into_iter().chain(houses).collect();
        (!goals.is_empty()).then(|| Goals {
            state,
            targets: goals,
            fields: BTreeMap::new(),
        })
    }

    /// How far `unit` would walk from `pos` to the nearest goal (stepping
    /// onto a goal's tile is free); `None` if it can't get there.
    fn distance(&mut self, unit: &Unit, pos: Pos) -> Option<i64> {
        let movement = self.state.classes().get(&unit.class)?.movement_type;
        let (state, goals) = (self.state, &self.targets);
        let field = self.fields.entry(movement.0).or_insert_with(|| {
            let tiles = &state.map().tiles;
            let cost = |tile: Pos| {
                if goals.contains(&tile) {
                    return Some(0);
                }
                let terrain = tiles.get(tile)?;
                state.terrain().move_cost(*terrain, movement).map(u32::from)
            };
            flow_field(tiles.width(), tiles.height(), goals.iter().copied(), cost)
        });
        field.get(pos).copied().flatten().map(i64::from)
    }
}

/// [`Tally::exposed`] and the rest, apart and for the lords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Danger {
    exposed: i64,
    lord_exposed: i64,
    at_risk: i64,
    lord_at_risk: i64,
    counter: i64,
}

/// The danger the player's units stand in, in `state` ([`Tally::exposed`]).
fn danger(state: &BattleState) -> Danger {
    let threats = threats(state);
    let mut out = Danger::default();
    // A lord left alone has nobody to wait for: waiting can't win, so its
    // danger doesn't hold it back.
    let alone = !state
        .units()
        .iter()
        .any(|u| u.faction == Faction::Player && !u.is_lord);
    for unit in state
        .units()
        .iter()
        .filter(|u| u.faction == Faction::Player)
    {
        let (mut expected, mut worst) = (0, 0);
        for (enemy, strikes) in &threats {
            if !strikes(unit.pos) {
                continue;
            }
            if let Some(forecast) = state.threat_forecast(*enemy, unit.id) {
                expected += expected_damage(&forecast.attacker);
                worst += all_hit(&forecast.attacker);
                let back = forecast.defender.as_ref().map_or(0, expected_damage) / 100;
                let enemy_hp = state.unit(*enemy).map_or(0, |e| i64::from(e.hp));
                out.counter += back.min(enemy_hp);
            }
        }
        let hp = i64::from(unit.hp);
        let exposed = (expected / 100).min(hp);
        // A standing unit has at least 1 HP.
        let at_risk = i64::from(worst >= hp);
        if unit.is_lord && alone {
        } else if unit.is_lord {
            out.lord_exposed += exposed;
            out.lord_at_risk += at_risk;
        } else {
            out.exposed += exposed;
            out.at_risk += at_risk;
        }
    }
    out
}

/// An enemy, and whether it can strike a tile in its next phase.
type Threat<'a> = (UnitId, Box<dyn Fn(Pos) -> bool + 'a>);

/// Each enemy of the player, and whether it can strike a tile in its next
/// phase: the danger zone, one enemy at a time, as the player sees it (one
/// that never moves only from its own tile).
fn threats(state: &BattleState) -> Vec<Threat<'_>> {
    let ranges = |u: &Unit| u.attack_ranges(state.classes(), state.items(), state.spells());
    let enemies = state
        .units()
        .iter()
        .filter(|u| u.faction.is_hostile_to(Faction::Player));
    enemies
        .filter_map(|e| {
            let reach = ranges(e);
            let strikes: Box<dyn Fn(Pos) -> bool> = if e.ai == AiBehavior::Stationary {
                let from = e.pos;
                Box::new(move |at| {
                    let d = Pos::manhattan(from, at);
                    reach.iter().any(|&(min, max)| (min..=max).contains(&d))
                })
            } else {
                let units = state.units();
                let area = threat_area(
                    state.map(),
                    state.terrain(),
                    state.classes(),
                    units,
                    e.id,
                    &reach,
                )
                .ok()?;
                Box::new(move |at| area.contains(at))
            };
            Some((e.id, strikes))
        })
        .collect()
}

/// The damage of `side`'s strikes if every one hits.
fn all_hit(side: &SideForecast) -> i64 {
    let followups = i64::from(side.strikes.saturating_sub(1));
    i64::from(side.damage) + followups * i64::from(side.followup_damage)
}

/// The HP of the enemies on the map.
fn enemy_hp(state: &BattleState) -> i64 {
    let enemies = state.units().iter().filter(|u| u.faction == Faction::Enemy);
    enemies.map(|u| i64::from(u.hp)).sum()
}

/// Unit `id`'s HP, 0 once it has fallen.
fn hp_of(state: &BattleState, id: UnitId) -> i64 {
    state.unit(id).map_or(0, |u| i64::from(u.hp))
}

/// How far the nearest player unit that may seize stands from `pos`; 0 if
/// there is none.
fn seize_distance(state: &BattleState, pos: Pos, by_lord: bool) -> i64 {
    let may_seize = |u: &&Unit| u.faction == Faction::Player && (u.is_lord || !by_lord);
    let units = state.units().iter().filter(may_seize);
    units
        .map(|u| i64::from(Pos::manhattan(u.pos, pos)))
        .min()
        .unwrap_or(0)
}

/// A persona's scoring of a playout: its [`Weights`] over the
/// [`Tally`] from the decision's start. Personas play the Player phase,
/// so the side is always the player's.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scoring(pub Weights);

impl Evaluate<BattleState> for Scoring {
    fn score(&self, start: &BattleState, end: &BattleState, _: Phase) -> i32 {
        Tally::between(start, end).score(&self.0)
    }
}

/// The candidates a persona weighs, best-looking first ([`order`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PersonaOrder {
    /// The enemy AI's weights, for the baseline's own pick.
    pub ai: AiWeights,
    /// How many candidates the persona tries.
    pub cap: usize,
}

impl Order<BattleState> for PersonaOrder {
    fn order(&mut self, state: &BattleState, commands: Vec<Command>) -> Vec<Command> {
        order(state, commands, &self.ai, self.cap)
    }
}

/// `commands` (the legal commands of `state`) as a persona tries them:
///
/// 1. each unit's best attack on each target (the most expected damage
///    dealt, then the least taken back, of every weapon, art and active
///    and every tile), the best first, so that every unit that can strike
///    has a say before any unit's second way of striking the same target;
/// 2. the baseline's own pick ([`next_command`] with `ai`), so a persona
///    can always do what the enemy AI would, moves towards the enemy
///    included;
/// 3. the heals (an item used on, or a heal cast at, a wounded unit), the
///    most wounded first, each once, from the healer's own tile if it can;
/// 4. each ready unit's advance: waiting on the tile it can reach that is
///    nearest the objective's goals, walking (the engine's flow field), if that is
///    nearer than where it stands;
/// 5. each ready unit's hold: waiting on the tile it can reach that no
///    enemy can strike next phase (the danger zone), nearest the
///    objective's goals, if it isn't its advance;
/// 6. the other attacks: each other way of striking (weapon, art or
///    active) once, from its best tile, the best first;
/// 7. the rest in the legal order;
/// 8. ending the phase last, in place of the last of the first `cap` if
///    there are that many, so the cap never cuts it.
///
/// `Equip` is dropped (an attack equips its weapon) and so is `Talk` (the
/// persona talks before it plans, [`PersonaBot`]).
pub fn order(
    state: &BattleState,
    commands: Vec<Command>,
    ai: &AiWeights,
    cap: usize,
) -> Vec<Command> {
    let pick = next_command(state, ai).filter(|pick| {
        !matches!(
            pick,
            Command::EndPhase | Command::Equip { .. } | Command::Talk { .. }
        ) && commands.contains(pick)
    });
    let mut end_phase = false;
    let mut attacks = Vec::new();
    let mut heals = Vec::new();
    let mut rest = Vec::new();
    let mut goals = Goals::of(state);
    // Each unit's best advance yet: the unit, its distance, the command.
    let mut advances: Vec<(UnitId, i64, Command)> = Vec::new();
    // The same, among the tiles out of every enemy's reach.
    let threats = threats(state);
    let safe = |at: Pos| !threats.iter().any(|(_, strikes)| strikes(at));
    let mut holds: Vec<(UnitId, i64, Command)> = Vec::new();
    for command in commands {
        match &command {
            Command::Equip { .. } | Command::Talk { .. } => {}
            Command::EndPhase => end_phase = true,
            Command::Act { unit, dest, action } => {
                if let Ok(preview) = state.preview_attack(*unit, *dest, action) {
                    let forecast = preview.forecast;
                    let taken = forecast.defender.as_ref().map_or(0, expected_damage);
                    attacks.push(((expected_damage(&forecast.attacker), -taken), command));
                } else if let Some(missing) = heal_need(state, *unit, *dest, action) {
                    let moves = state.unit(*unit).is_some_and(|u| u.pos != *dest);
                    heals.push(((missing, !moves), command));
                } else {
                    if let (UnitAction::Wait, Some(goals)) = (action, goals.as_mut()) {
                        advance(state, goals, *unit, *dest, &command, &mut advances);
                        if safe(*dest) {
                            hold(state, goals, *unit, *dest, &command, &mut holds);
                        }
                    }
                    rest.push(command);
                }
            }
            Command::MoveAfter { .. } => rest.push(command),
        }
    }
    attacks.sort_by_key(|a| Reverse(a.0));
    heals.sort_by_key(|h| Reverse(h.0));
    let (mut out, others): (Vec<Command>, Vec<Command>) = {
        let mut struck: Vec<(UnitId, Option<UnitId>)> = Vec::new();
        once_per_action(attacks.into_iter().map(|(_, c)| c))
            .into_iter()
            .partition(|c| {
                let key = attack_key(c);
                let first = !struck.contains(&key);
                struck.push(key);
                first
            })
    };
    // Only the baseline's pick can be in two of the lists.
    let pick = pick.filter(|pick| !out.contains(pick));
    let not_pick = |c: &Command| pick.as_ref() != Some(c);
    out.extend(pick.clone());
    out.extend(
        once_per_action(heals.into_iter().map(|(_, c)| c))
            .into_iter()
            .filter(not_pick),
    );
    let advances: Vec<Command> = advances.into_iter().map(|(_, _, c)| c).collect();
    let holds: Vec<Command> = holds
        .into_iter()
        .map(|(_, _, c)| c)
        .filter(|c| !advances.contains(c))
        .collect();
    let not_listed = |c: &Command| not_pick(c) && !advances.contains(c) && !holds.contains(c);
    out.extend(advances.iter().filter(|c| not_pick(c)).cloned());
    out.extend(holds.iter().filter(|c| not_pick(c)).cloned());
    out.extend(others.into_iter().filter(not_pick));
    out.extend(rest.into_iter().filter(not_listed));
    if end_phase {
        out.truncate(cap.max(1) - 1);
        out.push(Command::EndPhase);
    }
    out
}

/// Keeps `command` (unit `id` waiting on `dest`, a tile out of every
/// enemy's reach) in `holds` if it is the unit's nearest yet to the goals.
fn hold(
    state: &BattleState,
    goals: &mut Goals<'_>,
    id: UnitId,
    dest: Pos,
    command: &Command,
    holds: &mut Vec<(UnitId, i64, Command)>,
) {
    let Some(there) = state.unit(id).and_then(|unit| goals.distance(unit, dest)) else {
        return;
    };
    match holds.iter_mut().find(|h| h.0 == id) {
        Some(best) if there < best.1 => *best = (id, there, command.clone()),
        Some(_) => {}
        None => holds.push((id, there, command.clone())),
    }
}

/// The unit acting and the unit it strikes, of an attack.
fn attack_key(command: &Command) -> (UnitId, Option<UnitId>) {
    match command {
        Command::Act {
            unit,
            action: UnitAction::Attack { target, .. },
            ..
        }
        | Command::Act {
            unit,
            action:
                UnitAction::Cast {
                    target: CastTarget::Unit(target),
                    ..
                },
            ..
        } => (*unit, Some(*target)),
        Command::Act { unit, .. } => (*unit, None),
        _ => (UnitId(0), None),
    }
}

/// Keeps `command` (unit `id` waiting on `dest`) in `advances` if it is
/// the unit's nearest yet to the goals and nearer than its own tile.
fn advance(
    state: &BattleState,
    goals: &mut Goals<'_>,
    id: UnitId,
    dest: Pos,
    command: &Command,
    advances: &mut Vec<(UnitId, i64, Command)>,
) {
    let Some(unit) = state.unit(id) else {
        return;
    };
    let (Some(here), Some(there)) = (goals.distance(unit, unit.pos), goals.distance(unit, dest))
    else {
        return;
    };
    match advances.iter_mut().find(|a| a.0 == id) {
        Some(best) if there < best.1 => *best = (id, there, command.clone()),
        None if there < here => advances.push((id, there, command.clone())),
        _ => {}
    }
}

/// The expected damage of `side`'s strikes, in hundredths of an HP.
fn expected_damage(side: &SideForecast) -> i64 {
    all_hit(side) * i64::from(side.hit)
}

/// The HP its target is missing, if `action` is a heal: an item used on, or
/// a heal spell cast at, a wounded unit.
fn heal_need(state: &BattleState, unit: UnitId, dest: Pos, action: &UnitAction) -> Option<i64> {
    let target = match action {
        UnitAction::UseItem { target, .. } => *target,
        UnitAction::Cast {
            target: CastTarget::Unit(target),
            ..
        } if state.preview_heal(unit, dest, action).is_some() => *target,
        _ => return None,
    };
    let target = state.unit(target)?;
    let missing = i64::from(target.stats.hp - target.hp);
    (missing > 0).then_some(missing)
}

/// `commands` with only the first `Act` of each unit and action kept.
fn once_per_action(commands: impl Iterator<Item = Command>) -> Vec<Command> {
    let mut seen: Vec<(UnitId, UnitAction)> = Vec::new();
    commands
        .filter(|command| match command {
            Command::Act { unit, action, .. } => {
                let key = (*unit, action.clone());
                let new = !seen.contains(&key);
                if new {
                    seen.push(key);
                }
                new
            }
            _ => true,
        })
        .collect()
}

/// Mixed into a try's seed for the persona's own draws, so they never
/// follow the battle's own rolls (which start from the same seed).
const SEED_SALT: u64 = 0x5EED_0506_B075_0000;

/// A persona playing the Player phase: the engine's [`TurnPlan`] with the
/// persona's order and scoring.
///
/// Talking costs nothing (the unit can still act), so before planning it
/// issues each talk it can, once per battle (each pair of units once,
/// whichever starts it).
pub struct PersonaBot {
    plan: TurnPlan<PersonaOrder, Scoring>,
    talked: Vec<(UnitId, UnitId)>,
}

impl PersonaBot {
    /// `persona` for the try of `seed`, its order taking the enemy AI's
    /// own pick with `ai`'s weights.
    pub fn new(persona: &Persona, ai: AiWeights, seed: u64) -> Self {
        let cap = usize::try_from(persona.settings.candidates).unwrap_or(usize::MAX);
        let order = PersonaOrder { ai, cap };
        let scoring = Scoring(persona.weights);
        PersonaBot {
            plan: TurnPlan::new(order, scoring, persona.settings, seed ^ SEED_SALT),
            talked: Vec::new(),
        }
    }

    /// A talk between two units that haven't talked yet (either way
    /// round), if one is legal now.
    fn talk(&mut self, state: &BattleState) -> Option<Command> {
        let talked = |a: UnitId, b: UnitId| self.talked.iter().any(|&p| p == (a, b) || p == (b, a));
        let talk = legal_commands(state)
            .into_iter()
            .find(|c| matches!(c, Command::Talk { unit, target, .. } if !talked(*unit, *target)))?;
        if let Command::Talk { unit, target, .. } = talk {
            self.talked.push((unit, target));
        }
        Some(talk)
    }
}

impl Bot<BattleState> for PersonaBot {
    fn choose(&mut self, state: &BattleState) -> Command {
        match self.talk(state) {
            Some(talk) => talk,
            None => self.plan.choose(state),
        }
    }
}

#[cfg(test)]
mod tests;
