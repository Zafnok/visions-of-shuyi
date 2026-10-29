//! Recorded playthroughs of the story's battles (tickets 0803, 0848): the
//! player's commands, phase by phase, played back against the rules alone
//! or the real game, and the search that records them.
//!
//! A fixture holds only what the player does (every Player phase's
//! commands, its `EndPhase` included). The enemy plays as it does in the
//! game: the AI's command, else the end of its phase.

use trpg_content::Content;
use trpg_core::{
    BattleState, Command, Faction, Outcome, Phase, Pos, SimRng, UnitAction, legal_commands,
    next_command,
};
use trpg_ui::harness::Harness;

/// The player's commands in fixture `source` (`name` for errors).
pub fn commands(name: &str, source: &str) -> Vec<Command> {
    ron::from_str(source).unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// Plays the phases that aren't the player's, as the game does.
pub fn play_others(state: &mut BattleState, content: &Content) {
    while state.outcome().is_none() && state.phase() != Phase::Player {
        let command = next_command(state, &content.ai).unwrap_or(Command::EndPhase);
        if let Err(e) = state.apply(&command) {
            panic!("the AI's {command:?} was refused: {e}");
        }
    }
}

/// Plays `commands` from `state` against the rules alone, the others'
/// phases as the game plays them, to the battle's end.
pub fn play(state: &mut BattleState, content: &Content, commands: &[Command]) {
    for (i, command) in commands.iter().enumerate() {
        play_others(state, content);
        assert_eq!(state.outcome(), None, "over before command {i}");
        if let Err(e) = state.apply(command) {
            panic!("command {i}, {command:?}, turn {}: {e}", state.turn());
        }
    }
    play_others(state, content);
}

/// Skips every scene on screen: Cancel, then Confirm on "Skip scene?".
pub fn skip_scenes(h: &mut Harness) {
    for _ in 0..8 {
        if h.top_screen() != "dialogue" {
            return;
        }
        h.keys("d f");
    }
    panic!("the scenes never end: {:?}", h.screens());
}

/// The battle on screen.
pub fn battle(h: &Harness) -> BattleState {
    let battle = h.battle();
    battle
        .unwrap_or_else(|| panic!("no battle"))
        .state()
        .clone()
}

fn send(h: &mut Harness, command: &Command) {
    let battle = h.battle_mut();
    battle.unwrap_or_else(|| panic!("no battle")).send(command);
}

/// Plays `commands` in the battle on screen, through the real game: the
/// story scenes skipped as they come up, the others' phases as the game
/// plays them. Ends with the battle won.
pub fn play_in_game(h: &mut Harness, commands: &[Command]) {
    let ai = h.game().ctx().game.content.ai;
    for command in commands {
        loop {
            skip_scenes(h);
            let state = battle(h);
            assert_eq!(state.outcome(), None);
            if state.phase() == Phase::Player {
                break;
            }
            let theirs = next_command(&state, &ai).unwrap_or(Command::EndPhase);
            send(h, &theirs);
        }
        send(h, command);
    }
    assert_eq!(battle(h).outcome(), Some(Outcome::Victory));
}

/// After a won battle: its last scenes, the VICTORY banner, the results
/// and the victory scenes, up to "Save your progress?".
pub fn on_to_the_save_prompt(h: &mut Harness) {
    for _ in 0..40 {
        match h.top_screen() {
            "dialogue" => skip_scenes(h),
            "save_prompt" => break,
            // The last fight plays out; banners and the results take a
            // press.
            _ => {
                h.wait(0.5).keys("f");
            }
        }
    }
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "save_prompt"],
        "{}",
        h.snapshot()
    );
}

/// The player units that have fallen in `state`.
pub fn lost(state: &BattleState) -> Vec<&str> {
    let fallen = state.fallen().iter();
    fallen
        .filter(|u| u.faction == Faction::Player)
        .map(|u| u.name.as_str())
        .collect()
}

// --- Recording a playthrough ---------------------------------------------------

/// How good `state` is for the player, for [`record`]'s search.
fn worth(state: &BattleState) -> i64 {
    match state.outcome() {
        Some(Outcome::Victory) => return 1_000_000,
        Some(_) => return -1_000_000,
        None => {}
    }
    let mut worth = 0;
    let enemies = || state.units().iter().filter(|u| u.faction == Faction::Enemy);
    for unit in state.units() {
        let hp = i64::from(unit.hp);
        if unit.faction == Faction::Player {
            let missing = i64::from(unit.stats.hp) - hp;
            worth -= missing * if unit.is_lord { 8 } else { 4 };
            // Keep moving toward the fight.
            let nearest = enemies().map(|e| Pos::manhattan(e.pos, unit.pos)).min();
            worth -= i64::from(nearest.unwrap_or(0));
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

/// One way to play the Player phase `state` is in, picked by `rng`: the
/// AI's command for the player's side, or any legal command.
fn try_phase(
    state: &mut BattleState,
    content: &Content,
    rng: &mut SimRng,
    plain: bool,
) -> Vec<Command> {
    let mut commands = Vec::new();
    while state.outcome().is_none() && state.phase() == Phase::Player {
        let ai = next_command(state, &content.ai);
        // A potion is for someone who is hurt.
        let wasted = |action: &UnitAction| match action {
            UnitAction::UseItem { target, .. } => {
                let target = state.unit(*target);
                target.is_none_or(|u| u.hp == u.stats.hp)
            }
            _ => false,
        };
        let legal: Vec<Command> = legal_commands(state)
            .into_iter()
            .filter(|c| !matches!(c, Command::Act { action, .. } if wasted(action)))
            .collect();
        let acts =
            |c: &&Command| matches!(c, Command::Act { action, .. } if *action != UnitAction::Wait);
        let pick = |from: Vec<&Command>, rng: &mut SimRng| {
            let n = u32::try_from(from.len()).unwrap_or(0);
            (n > 0).then(|| from[rng.roll_below(n) as usize].clone())
        };
        let roll = if plain { 0 } else { rng.roll_below(10) };
        let command = match roll {
            0..=4 => ai,
            5..=7 => pick(legal.iter().filter(acts).collect(), rng).or(ai),
            8 => pick(
                legal.iter().filter(|c| **c != Command::EndPhase).collect(),
                rng,
            ),
            _ => Some(Command::EndPhase),
        };
        let command = command.unwrap_or(Command::EndPhase);
        if state.apply(&command).is_ok() {
            commands.push(command);
        }
    }
    commands
}

/// A Player phase's commands and the battle after the enemy answered them.
type Plan = (Vec<Command>, BattleState);

/// The best few of many tries at `state`'s Player phase, best first.
fn plans(state: &BattleState, content: &Content, rng: &mut SimRng) -> Vec<Plan> {
    const TRIES: u32 = 300;
    const KEPT: usize = 4;
    let mut plans: Vec<(i64, Plan)> = Vec::new();
    for attempt in 0..TRIES {
        let mut after = state.clone();
        let commands = try_phase(&mut after, content, rng, attempt == 0);
        play_others(&mut after, content);
        let worth = worth(&after);
        if plans.iter().all(|(_, (c, _))| *c != commands) {
            plans.push((worth, (commands, after)));
            plans.sort_by_key(|(w, _)| std::cmp::Reverse(*w));
            plans.truncate(KEPT);
        }
    }
    plans.into_iter().map(|(_, plan)| plan).collect()
}

/// Records a winning playthrough of `start` into the fixture at `path`
/// (under the crate), its first lines `header`: phase by phase, the best
/// of many tries at the Player phase (`seed` picks them), each judged
/// after the enemy has answered it. When every try at a phase loses
/// someone, it goes back a turn and takes that turn's next best. It knows
/// the battle's luck, which a player doesn't: it shows that the battle
/// *can* be won, not how hard it is (that is the bots', `cargo xtask
/// playtest`).
pub fn record(start: BattleState, content: &Content, seed: u64, header: &[&str], path: &str) {
    let mut rng = SimRng::new(seed);
    // Each turn played: the plans for it that are left, best first. The
    // first of each is the one being played.
    let mut turns: Vec<Vec<Plan>> = Vec::new();
    let mut state = start;
    let mut steps = 0;
    while state.outcome().is_none() {
        steps += 1;
        assert!(steps <= 200 && state.turn() <= 60, "no end in sight");
        let found = plans(&state, content, &mut rng);
        let safe = found.first().is_some_and(|(_, after)| {
            lost(after).is_empty() && after.outcome().is_none_or(|o| o == Outcome::Victory)
        });
        turns.push(if safe { found } else { Vec::new() });
        // Back to the last turn with another plan to try.
        while turns.last().is_some_and(|plans| {
            plans.first().is_none_or(|(_, after)| {
                !lost(after).is_empty() || after.outcome().is_some_and(|o| o != Outcome::Victory)
            })
        }) {
            turns.pop();
            let Some(earlier) = turns.last_mut() else {
                panic!("every way to play turn 1 loses someone");
            };
            earlier.remove(0);
        }
        let Some((_, after)) = turns.last().and_then(|plans| plans.first()) else {
            panic!("no plan");
        };
        state = after.clone();
    }
    let mut out: Vec<String> = header.iter().map(|l| (*l).to_owned()).collect();
    out.push("[".to_owned());
    for (turn, plans) in turns.iter().enumerate() {
        out.push(format!("    // Turn {}", turn + 1));
        for command in plans.first().map(|(c, _)| c).into_iter().flatten() {
            let text = ron::to_string(command).unwrap_or_else(|e| panic!("{e}"));
            out.push(format!("    {text},"));
        }
    }
    let (outcome, turn) = (state.outcome(), state.turn());
    let houses = state
        .village()
        .map(|v| format!("; houses saved: {} of {}", v.saved(), v.total()))
        .unwrap_or_default();
    out.push(format!(
        "    // {outcome:?} on turn {turn}; lost: {:?}{houses}",
        lost(&state)
    ));
    out.extend(["]".to_owned(), String::new()]);
    let out = out.join("\n");
    assert_eq!(outcome, Some(Outcome::Victory), "{out}");
    assert!(lost(&state).is_empty(), "{out}");
    let path = format!("{}/{path}", env!("CARGO_MANIFEST_DIR"));
    std::fs::write(&path, out).unwrap_or_else(|e| panic!("{path}: {e}"));
}
