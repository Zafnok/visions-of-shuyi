//! Chapter 1 can be won (ticket 0803): the player's commands in
//! `fixtures/ch01_winning.ron`, sent to the real game from New Game, rout
//! the Red Company with nobody lost, and the chapter runs on to "To be
//! continued".
//!
//! The fixture holds only what the player does (every Player phase's
//! commands, its `EndPhase` included). The enemy plays as it does in the
//! game: the AI's command, else the end of its phase.
//!
//! **If a balance change breaks this test**, the change must update the
//! playthrough on purpose: look at what it did to the chapter, then record
//! a new one with
//!
//! ```text
//! cargo test -p trpg-ui --test it ch01_winnable::record -- --ignored
//! ```
//!
//! and read the new fixture's last lines (who fell, how many turns).

use trpg_content::{Content, new_campaign};
use trpg_core::{
    BattleState, Command, Faction, GameMode, LeadGender, LeadProfile, Outcome, Phase, Pos, SimRng,
    UnitAction, legal_commands, next_command,
};
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;

const FIXTURE: &str = include_str!("fixtures/ch01_winning.ron");

/// The player's commands, in order.
fn winning_commands() -> Vec<Command> {
    ron::from_str(FIXTURE).unwrap_or_else(|e| panic!("ch01_winning.ron: {e}"))
}

/// Chapter 1's battle as a new game starts it.
fn chapter_1(content: &Content) -> BattleState {
    let lead = LeadProfile::new("Ellery", LeadGender::Male);
    let campaign = new_campaign(content, GameMode::Classic, lead);
    let def = &content.battles["ch01"];
    BattleState::new(campaign.battle_setup(def, &content.tables())).0
}

/// Plays the phases that aren't the player's, as the game does.
fn play_others(state: &mut BattleState, content: &Content) {
    while state.outcome().is_none() && state.phase() != Phase::Player {
        let command = next_command(state, &content.ai).unwrap_or(Command::EndPhase);
        if let Err(e) = state.apply(&command) {
            panic!("the AI's {command:?} was refused: {e}");
        }
    }
}

/// The rules alone: the fixture's commands win, in Classic, losing nobody.
#[test]
fn the_recorded_playthrough_routs_the_red_company() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let mut state = chapter_1(&content);
    let commands = winning_commands();
    for (i, command) in commands.iter().enumerate() {
        play_others(&mut state, &content);
        assert_eq!(state.outcome(), None, "over before command {i}");
        if let Err(e) = state.apply(command) {
            panic!("command {i}, {command:?}, turn {}: {e}", state.turn());
        }
    }
    play_others(&mut state, &content);
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&state), Vec::<&str>::new());
    assert_eq!(state.fallen().len(), 14, "13 soldiers and Harl");
}

/// Skips every scene on screen: Cancel, then Confirm on "Skip scene?".
fn skip_scenes(h: &mut Harness) {
    for _ in 0..8 {
        if h.top_screen() != "dialogue" {
            return;
        }
        h.keys("d f");
    }
    panic!("the scenes never end: {:?}", h.screens());
}

fn battle(h: &Harness) -> BattleState {
    let battle = h.flow().and_then(|f| f.battle());
    battle
        .unwrap_or_else(|| panic!("no battle"))
        .state()
        .clone()
}

fn send(h: &mut Harness, command: &Command) {
    let battle = h.flow_mut().and_then(|f| f.battle_mut());
    battle.unwrap_or_else(|| panic!("no battle")).send(command);
}

/// Acceptance: Chapter 1 from New Game to "To be continued", through the
/// real game: the intro scenes, the battle won with the recorded commands
/// (its story scenes skipped as they come up), the results, the victory
/// scenes, the save prompt.
#[test]
fn chapter_1_plays_from_new_game_to_to_be_continued() {
    let mut h = Harness::with_layout(Layout::RightHanded).with_shipped_story();
    // New Game, Classic, the default lead.
    h.keys("f f Up f");
    let chapter = h.flow().and_then(|f| f.chapter()).map(|c| c.id.clone());
    assert_eq!(chapter.as_deref(), Some("ch01"));
    // The letter, then the Red Company.
    skip_scenes(&mut h);
    assert_eq!(h.screens(), ["title", "battle"]);
    let start = battle(&h);
    let players = start
        .units()
        .iter()
        .filter(|u| u.faction == Faction::Player);
    assert_eq!(players.count(), 6);
    assert_eq!(start.units().len(), 20);
    let ai = h.game().ctx().content.ai;
    for command in winning_commands() {
        loop {
            skip_scenes(&mut h);
            let state = battle(&h);
            assert_eq!(state.outcome(), None);
            if state.phase() == Phase::Player {
                break;
            }
            let theirs = next_command(&state, &ai).unwrap_or(Command::EndPhase);
            send(&mut h, &theirs);
        }
        send(&mut h, &command);
    }
    assert_eq!(battle(&h).outcome(), Some(Outcome::Victory));
    // Harl's last words, the VICTORY banner, the results, the victory
    // scenes, "Save your progress?" (No), "To be continued", the title.
    for _ in 0..40 {
        match h.top_screen() {
            "dialogue" => skip_scenes(&mut h),
            "save_prompt" => break,
            // The last fight plays out; banners and the results take a
            // press.
            _ => {
                h.wait(0.5).keys("f");
            }
        }
    }
    assert_eq!(h.screens(), ["title", "save_prompt"], "{}", h.snapshot());
    let flow = h.flow().unwrap_or_else(|| panic!("no flow"));
    let campaign = flow.campaign().unwrap_or_else(|| panic!("no campaign"));
    assert_eq!(campaign.gold, 1000, "the clear gold");
    assert_eq!(campaign.roster.len(), 6, "nobody was lost");
    h.keys("Down f");
    assert_eq!(h.screens(), ["title", "to_be_continued"]);
    h.keys("f");
    assert_eq!(h.screens(), ["title"]);
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

/// The player units that have fallen in `state`.
fn lost(state: &BattleState) -> Vec<&str> {
    let fallen = state.fallen().iter();
    fallen
        .filter(|u| u.faction == Faction::Player)
        .map(|u| u.name.as_str())
        .collect()
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

/// Records a new `fixtures/ch01_winning.ron`: phase by phase, the best of
/// many tries at the Player phase, each judged after the enemy has
/// answered it. When every try at a phase loses someone, it goes back a
/// turn and takes that turn's next best. It knows the battle's luck, which
/// a player doesn't: it shows that the chapter *can* be won, not how hard
/// it is (the difficulty sim is `crates/bots/tests/baseline.rs`).
#[test]
#[ignore = "writes the fixture: run it on purpose"]
fn record() {
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let mut rng = SimRng::new(803);
    // Each turn played: the plans for it that are left, best first. The
    // first of each is the one being played.
    let mut turns: Vec<Vec<Plan>> = Vec::new();
    let mut state = chapter_1(&content);
    let mut steps = 0;
    while state.outcome().is_none() {
        steps += 1;
        assert!(steps <= 200 && state.turn() <= 60, "no end in sight");
        let found = plans(&state, &content, &mut rng);
        let safe = found
            .first()
            .is_some_and(|(_, after)| lost(after).is_empty());
        turns.push(if safe { found } else { Vec::new() });
        // Back to the last turn with another plan to try.
        while turns.last().is_some_and(|plans| {
            plans
                .first()
                .is_none_or(|(_, after)| !lost(after).is_empty())
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
    let mut out: Vec<String> = [
        "// A winning playthrough of Chapter 1 (ticket 0803): the player's commands,",
        "// phase by phase. Played back by ch01_winnable.rs, which says how to record",
        "// a new one. Units 1-6: the lead, Hollis, Tamsin, Aske, Maud, Rue.",
        "[",
    ]
    .map(String::from)
    .into();
    for (turn, plans) in turns.iter().enumerate() {
        out.push(format!("    // Turn {}", turn + 1));
        for command in plans.first().map(|(c, _)| c).into_iter().flatten() {
            let text = ron::to_string(command).unwrap_or_else(|e| panic!("{e}"));
            out.push(format!("    {text},"));
        }
    }
    let (outcome, turn) = (state.outcome(), state.turn());
    out.push(format!(
        "    // {outcome:?} on turn {turn}; lost: {:?}",
        lost(&state)
    ));
    out.extend(["]".to_owned(), String::new()]);
    let out = out.join("\n");
    assert_eq!(outcome, Some(Outcome::Victory), "{out}");
    assert!(lost(&state).is_empty(), "{out}");
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/it/fixtures/ch01_winning.ron"
    );
    std::fs::write(path, out).unwrap_or_else(|e| panic!("{path}: {e}"));
}
