//! The search that records a winning playthrough (tickets 0803, 0848,
//! 0849; moved here from `trpg-ui`'s tests by 0531): the player's commands,
//! phase by phase, each Player phase the best of many tries, judged after
//! the enemy has answered it. It knows the battle's luck, which a player
//! doesn't: a recording shows that a battle *can* be won, and the army it
//! leaves is one player's (`docs/playtesting.md`, *The army*).
//!
//! The other phases are played as the game plays them, here and in every
//! replay ([`play`]): the enemy AI's command, else the phase's end.

use shuyi_core::prelude::{Grid, flow_field};
use trpg_content::Content;
use trpg_core::{
    BattleState, Command, Faction, Objective, Outcome, Phase, Pos, SimRng, UnitAction,
    legal_commands, next_command,
};

/// A win that counts for [`record`]'s search.
pub type Wanted<'a> = &'a dyn Fn(&BattleState) -> bool;

/// A recorded win: the player's commands and how it ended.
#[derive(Debug, Clone, PartialEq)]
pub struct Recorded {
    /// Each Player phase's commands, its `EndPhase` included, turn 1 first.
    pub phases: Vec<Vec<Command>>,
    /// How it ended: `Some(Victory) on turn 8; lost: []`, and the houses
    /// saved in a battle with a village.
    pub summary: String,
}

impl Recorded {
    /// Every command, in order.
    pub fn commands(&self) -> Vec<Command> {
        self.phases.iter().flatten().cloned().collect()
    }
}

/// Plays `commands`, the player's, from `state` to the battle's end, the
/// other phases as the game plays them.
pub fn play(
    state: &mut BattleState,
    content: &Content,
    commands: &[Command],
) -> Result<(), String> {
    for (i, command) in commands.iter().enumerate() {
        play_others(state, content)?;
        if state.outcome().is_some() {
            return Err(format!("over before command {i}"));
        }
        state
            .apply(command)
            .map_err(|e| format!("command {i}, {command:?}, turn {}: {e}", state.turn()))?;
    }
    play_others(state, content)
}

/// Plays the phases that aren't the player's, to the player's next phase
/// or the battle's end.
pub fn play_others(state: &mut BattleState, content: &Content) -> Result<(), String> {
    while state.outcome().is_none() && state.phase() != Phase::Player {
        let command = next_command(state, &content.ai).unwrap_or(Command::EndPhase);
        state
            .apply(&command)
            .map_err(|e| format!("the AI's {command:?} was refused: {e}"))?;
    }
    Ok(())
}

/// The player units that have fallen in `state`.
pub fn lost(state: &BattleState) -> Vec<&str> {
    let fallen = state.fallen().iter();
    fallen
        .filter(|u| u.faction == Faction::Player)
        .map(|u| u.name.as_str())
        .collect()
}

/// Records a winning playthrough of `start` that is `wanted` (every win,
/// or for example one with a recruit beaten first, 1071), with nobody
/// lost: phase by phase, the best of many tries at the Player phase
/// (`seed` picks them), each judged after the enemy has answered it. When
/// every try at a phase loses someone, it goes back a turn and takes that
/// turn's next best. An error when every way to play turn 1 loses someone,
/// or the battle doesn't end.
pub fn record(
    start: BattleState,
    content: &Content,
    seed: u64,
    wanted: Wanted<'_>,
) -> Result<Recorded, String> {
    record_within(start, content, seed, wanted, LIMITS)
}

/// How long [`record`]'s search goes on before it gives up: Player phases
/// tried (backing up counts), and the last turn.
#[derive(Debug, Clone, Copy)]
struct Limits {
    steps: u32,
    turns: u32,
}

/// [`record`]'s limits.
const LIMITS: Limits = Limits {
    steps: 200,
    turns: 60,
};

/// [`record`], giving up past `limits`.
fn record_within(
    start: BattleState,
    content: &Content,
    seed: u64,
    wanted: Wanted<'_>,
    limits: Limits,
) -> Result<Recorded, String> {
    let mut rng = SimRng::new(seed);
    let good = |after: &BattleState| may_go_on(after, wanted);
    // Each turn played: the plans for it that are left, best first. The
    // first of each is the one being played.
    let mut turns: Vec<Vec<Plan>> = Vec::new();
    let mut state = start;
    let mut steps = 0;
    while state.outcome().is_none() {
        steps += 1;
        if steps > limits.steps || state.turn() > limits.turns {
            return Err(format!("no end in sight on turn {}", state.turn()));
        }
        let found = plans(&state, content, &mut rng, wanted)?;
        let safe = found.first().is_some_and(|(_, after)| good(after));
        turns.push(if safe { found } else { Vec::new() });
        // Back to the last turn with another plan to try.
        while turns
            .last()
            .is_some_and(|plans| plans.first().is_none_or(|(_, after)| !good(after)))
        {
            turns.pop();
            let Some(earlier) = turns.last_mut() else {
                return Err("every way to play turn 1 loses someone".to_owned());
            };
            earlier.remove(0);
        }
        let Some((_, after)) = turns.last().and_then(|plans| plans.first()) else {
            return Err("no plan".to_owned());
        };
        state = after.clone();
    }
    let (outcome, turn) = (state.outcome(), state.turn());
    let houses = state
        .village()
        .map_or_default(|v| format!("; houses saved: {} of {}", v.saved(), v.total()));
    let summary = format!(
        "{outcome:?} on turn {turn}; lost: {:?}{houses}",
        lost(&state)
    );
    if outcome != Some(Outcome::Victory) || !lost(&state).is_empty() || !wanted(&state) {
        return Err(format!("not a win: {summary}"));
    }
    let phases = turns
        .into_iter()
        .map(|plans| plans.into_iter().next().map_or_default(|(c, _)| c))
        .collect();
    Ok(Recorded { phases, summary })
}

/// Whether the search may go on from `after`: nobody lost, and the battle
/// not over or won as `wanted`.
fn may_go_on(after: &BattleState, wanted: Wanted<'_>) -> bool {
    lost(after).is_empty()
        && after
            .outcome()
            .is_none_or(|o| o == Outcome::Victory && wanted(after))
}

/// How good `state` is for the player, for [`record`]'s search: a win
/// that isn't `wanted` is as bad as a defeat.
fn worth(state: &BattleState, wanted: Wanted<'_>) -> i64 {
    match state.outcome() {
        Some(Outcome::Victory) if wanted(state) => return 1_000_000,
        Some(_) => return -1_000_000,
        None => {}
    }
    let mut worth = 0;
    let enemies = || state.units().iter().filter(|u| u.faction == Faction::Enemy);
    let seize = seize_field(state);
    for unit in state.units() {
        let hp = i64::from(unit.hp);
        if unit.faction == Faction::Player {
            let missing = i64::from(unit.stats.hp) - hp;
            worth -= missing * if unit.is_lord { 8 } else { 4 };
            // Keep moving toward the fight.
            let nearest = enemies().map(|e| Pos::manhattan(e.pos, unit.pos)).min();
            worth -= i64::from(nearest.unwrap_or(0));
            // And the lord toward a tile to seize, walking: only it wins it.
            if let Some(field) = seize.as_ref().filter(|_| unit.is_lord) {
                let walk = field.get(unit.pos).copied().flatten();
                worth -= 20 * i64::from(walk.unwrap_or(u32::MAX / 64));
            }
        } else {
            worth -= 200 + hp * 3;
        }
    }
    // A potion is for someone who needs it.
    worth += 20 * i64::try_from(state.pack().items.len()).unwrap_or(0);
    // A house saved is worth a fight; one burned costs one.
    if let Some(village) = state.village() {
        worth += 300 * i64::from(village.saved()) - 600 * i64::from(village.lost());
    }
    worth - 5_000 * i64::try_from(lost(state).len()).unwrap_or(0)
}

/// How far the lord walks from each tile to the tile to seize, in a Seize
/// battle: as the crow flies, a lead on a shore would walk away from the
/// only way across (0849).
fn seize_field(state: &BattleState) -> Option<Grid<Option<u32>>> {
    let Objective::Seize { pos, .. } = state.objective() else {
        return None;
    };
    let lord = state.units().iter().find(|u| u.is_lord)?;
    let movement = state.classes().get(&lord.class)?.movement_type;
    let tiles = &state.map().tiles;
    let cost = |tile: Pos| {
        if tile == pos {
            return Some(0);
        }
        let terrain = tiles.get(tile)?;
        state.terrain().move_cost(*terrain, movement).map(u32::from)
    };
    Some(flow_field(tiles.width(), tiles.height(), [pos], cost))
}

/// More commands than any Player phase plays: a try stops there, where a
/// command the rules keep refusing would loop for ever.
const MOST_COMMANDS: usize = 1_000;

/// One way to play the Player phase `state` is in, picked by `rng`: the
/// AI's command for the player's side, or any legal command. `plain`: the
/// AI's command every time.
fn try_phase(
    state: &mut BattleState,
    content: &Content,
    rng: &mut SimRng,
    plain: bool,
) -> Vec<Command> {
    let mut commands = Vec::new();
    // One Player phase: with no enemy left (a Seize map), ending it
    // starts the next turn's Player phase at once.
    let turn = state.turn();
    for _ in 0..MOST_COMMANDS {
        if state.outcome().is_some() || state.phase() != Phase::Player || state.turn() != turn {
            break;
        }
        let ai = next_command(state, &content.ai);
        let roll = if plain { 0 } else { rng.roll_below(10) };
        let command = choose(roll, ai, &useful(state), rng);
        if state.apply(&command).is_ok() {
            commands.push(command);
        }
    }
    commands
}

/// The legal commands of `state` but a potion for someone who isn't hurt.
fn useful(state: &BattleState) -> Vec<Command> {
    let wasted = |action: &UnitAction| match action {
        UnitAction::UseItem { target, .. } => {
            let target = state.unit(*target);
            target.is_none_or(|u| u.hp == u.stats.hp)
        }
        _ => false,
    };
    legal_commands(state)
        .into_iter()
        .filter(|c| !matches!(c, Command::Act { action, .. } if wasted(action)))
        .collect()
}

/// The command a try plays for a `roll` of 0 to 9: the AI's (`ai`) for 0
/// to 4; for 5 to 7, any of `legal` that acts (not a Wait), else the AI's;
/// for 8, any of `legal` but ending the phase; for 9, ending the phase.
/// `rng` picks among them; with nothing to play, the phase ends.
fn choose(roll: u32, ai: Option<Command>, legal: &[Command], rng: &mut SimRng) -> Command {
    let acts =
        |c: &&Command| matches!(c, Command::Act { action, .. } if *action != UnitAction::Wait);
    let pick = |from: Vec<&Command>, rng: &mut SimRng| {
        let n = u32::try_from(from.len()).unwrap_or(0);
        (n > 0).then(|| from[rng.roll_below(n) as usize].clone())
    };
    let command = match roll {
        0..=4 => ai,
        5..=7 => pick(legal.iter().filter(acts).collect(), rng).or(ai),
        8 => pick(
            legal.iter().filter(|c| **c != Command::EndPhase).collect(),
            rng,
        ),
        _ => Some(Command::EndPhase),
    };
    command.unwrap_or(Command::EndPhase)
}

/// A Player phase's commands and the battle after the enemy answered them.
type Plan = (Vec<Command>, BattleState);

/// The best few of many tries at `state`'s Player phase, best first.
fn plans(
    state: &BattleState,
    content: &Content,
    rng: &mut SimRng,
    wanted: Wanted<'_>,
) -> Result<Vec<Plan>, String> {
    const TRIES: u32 = 300;
    const KEPT: usize = 4;
    let mut plans: Vec<(i64, Plan)> = Vec::new();
    for attempt in 0..TRIES {
        let mut after = state.clone();
        let commands = try_phase(&mut after, content, rng, attempt == 0);
        play_others(&mut after, content)?;
        let worth = worth(&after, wanted);
        if plans.iter().all(|(_, (c, _))| *c != commands) {
            plans.push((worth, (commands, after)));
            plans.sort_by_key(|(w, _)| std::cmp::Reverse(*w));
            plans.truncate(KEPT);
        }
    }
    Ok(plans.into_iter().map(|(_, plan)| plan).collect())
}

#[cfg(test)]
mod tests;
