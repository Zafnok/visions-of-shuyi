use trpg_core::{GameMode, LeadGender, LeadProfile};

use super::*;
use crate::testkit::test_story;
use crate::{Wins, start_of};

/// The test story's first battle as its first line starts it: a Seize
/// within 3 turns.
fn first_battle(content: &Content) -> BattleState {
    let def = &content.battles["test_world_story"];
    let lead = LeadProfile::new("Lead", LeadGender::Male);
    let wins = Wins::default();
    let setup = start_of(content, def, GameMode::Classic, lead, &wins, 0);
    BattleState::new(setup.unwrap().unwrap()).0
}

/// A recording wins with nobody lost, and its commands win again when
/// played back as the game plays the other phases.
#[test]
fn a_recording_wins_and_its_commands_win_again() {
    let content = test_story();
    let recorded = record(first_battle(&content), &content, 1, &|_| true).unwrap();
    assert!(
        recorded.summary.starts_with("Some(Victory) on turn "),
        "{}",
        recorded.summary
    );
    assert!(recorded.summary.contains("lost: []"));
    assert!(
        recorded.phases.iter().all(|p| !p.is_empty()),
        "a phase a turn"
    );
    let mut again = first_battle(&content);
    play(&mut again, &content, &recorded.commands()).unwrap();
    assert_eq!(again.outcome(), Some(Outcome::Victory));
    assert_eq!(lost(&again), Vec::<&str>::new());
}

/// The same battle and seed give the same recording.
#[test]
fn a_recording_is_the_same_every_time() {
    let content = test_story();
    let once = record(first_battle(&content), &content, 7, &|_| true).unwrap();
    let twice = record(first_battle(&content), &content, 7, &|_| true).unwrap();
    assert_eq!(once, twice);
}

/// A win that isn't wanted counts for nothing: with none wanted, there is
/// no recording, and the error says why.
#[test]
fn a_win_nobody_wants_is_no_recording() {
    let content = test_story();
    let error = record(first_battle(&content), &content, 1, &|_| false).unwrap_err();
    assert_ne!(error, "");
}

/// Replaying commands that don't fit the battle says which command.
#[test]
fn a_command_the_rules_refuse_is_named() {
    let content = test_story();
    let mut state = first_battle(&content);
    let bogus = Command::Act {
        unit: trpg_core::UnitId(999),
        dest: Pos::new(0, 0),
        action: UnitAction::Wait,
    };
    let error = play(&mut state, &content, &[bogus]).unwrap_err();
    assert!(error.starts_with("command 0, "), "{error}");
}

// ---- What the search weighs (moved from trpg-ui's tests, ticket 0531) ------

use trpg_core::{BattleSetup, UnitId};

use crate::testkit::{
    Scripted, attack, content as game, enemy, lord, player, rigged, setup, start, stationary, tile,
    weak,
};

fn every(_: &BattleState) -> bool {
    true
}

fn nobody(_: &BattleState) -> bool {
    false
}

/// The HP of `state`'s only enemy.
fn foe(state: &BattleState) -> i64 {
    let enemy = state.units().iter().find(|u| u.faction == Faction::Enemy);
    i64::from(enemy.map_or(0, |u| u.hp))
}

#[test]
fn a_wanted_win_is_worth_most_and_any_other_end_least() {
    let mut won = Scripted::new(start(vec![rigged(lord(player(1, 4))), weak(enemy(2, 5))]));
    won.act(1, 4, attack(2));
    assert_eq!(won.state.outcome(), Some(Outcome::Victory));
    assert_eq!(worth(&won.state, &every), 1_000_000);
    assert_eq!(worth(&won.state, &nobody), -1_000_000);
    assert!(may_go_on(&won.state, &every));
    assert!(!may_go_on(&won.state, &nobody));
    let foes = vec![weak(lord(player(1, 4))), stationary(rigged(enemy(2, 5)))];
    let mut beaten = Scripted::new(start(foes));
    beaten.end_phase();
    beaten.act(2, 5, attack(1));
    assert_eq!(beaten.state.outcome(), Some(Outcome::Defeat));
    assert_eq!(worth(&beaten.state, &every), -1_000_000);
    assert!(!may_go_on(&beaten.state, &every));
}

/// Each HP a unit is short costs 4 (8 for the lord), each tile to the
/// nearest enemy 1, each enemy 200 and 3 an HP; each item in the pack is
/// worth 20.
#[test]
fn the_search_counts_wounds_distance_enemies_and_the_pack() {
    let mut sore_lord = lord(player(1, 1));
    sore_lord.hp -= 2;
    let mut sore = player(3, 2);
    sore.hp -= 3;
    let state = start(vec![sore_lord, sore, enemy(2, 6)]);
    assert_eq!(state.pack().items.len(), 3);
    let expected = -2 * 8 - 5 - 3 * 4 - 4 - (200 + 3 * foe(&state)) + 3 * 20;
    assert_eq!(worth(&state, &every), expected);
}

/// A unit lost costs 5,000, and the search goes no further from there.
#[test]
fn a_unit_lost_costs_five_thousand() {
    let units = vec![
        lord(player(1, 0)),
        weak(player(3, 4)),
        stationary(rigged(enemy(2, 5))),
    ];
    let mut battle = Scripted::new(start(units));
    battle.end_phase();
    battle.act(2, 5, attack(3));
    let state = &battle.state;
    assert_eq!(state.outcome(), None);
    assert_eq!(lost(state).len(), 1);
    let expected = -5 - (200 + 3 * foe(state)) + 3 * 20 - 5_000;
    assert_eq!(worth(state, &every), expected);
    assert!(!may_go_on(state, &every));
}

/// In a Seize battle each tile the lord walks to the tile to seize costs
/// 20.
#[test]
fn the_lord_walks_to_the_tile_to_seize() {
    let seize = |x| {
        let objective = Objective::Seize {
            pos: tile(6),
            by_lord: true,
            turn_limit: None,
        };
        let units = vec![lord(player(1, x)), enemy(2, 8)];
        BattleState::new(BattleSetup {
            objective,
            ..setup(units)
        })
        .0
    };
    let (far, near) = (seize(1), seize(2));
    // Four tiles to walk: the tile to seize itself costs nothing.
    let expected = -20 * 4 - 7 - (200 + 3 * foe(&far)) + 3 * 20;
    assert_eq!(worth(&far, &every), expected);
    assert_eq!(worth(&near, &every) - worth(&far, &every), 20 + 1);
}

/// A house saved is worth 300, one burned costs 600.
#[test]
fn a_house_saved_is_worth_300_and_one_burned_costs_600() {
    let mut raider = enemy(2, 8);
    raider.raider = true;
    let village = trpg_core::VillageGoal {
        houses: vec![tile(2), tile(4), tile(6)],
        minimum: 1,
        rewards: vec![],
        burnt: game().terrain.display.id_of("burnt").unwrap(),
    };
    let (state, _) = BattleState::new(BattleSetup {
        village: Some(village),
        ..setup(vec![lord(player(1, 0)), raider])
    });
    let mut battle = Scripted::new(state);
    let begin = worth(&battle.state, &every);
    battle.act(1, 2, UnitAction::Evacuate);
    let saved = worth(&battle.state, &every);
    // Saved, and the lord 2 tiles nearer the raider.
    assert_eq!(saved - begin, 300 + 2);
    battle.end_phase();
    battle.act(2, 7, UnitAction::Wait);
    let houses = battle.state.village().map(|v| (v.saved(), v.lost()));
    assert_eq!(houses, Some((1, 1)));
    // Burned, and the raider a tile nearer.
    assert_eq!(worth(&battle.state, &every) - saved, -600 + 1);
}

#[test]
fn a_roll_plays_the_ai_an_act_any_move_or_the_end() {
    let act = |x, action| Command::Act {
        unit: UnitId(1),
        dest: tile(x),
        action,
    };
    let hit = act(4, attack(2));
    let step = act(3, UnitAction::Wait);
    let ai = act(2, UnitAction::Wait);
    let legal = [hit.clone(), step.clone(), Command::EndPhase];
    let mut rng = SimRng::new(1);
    for roll in 0..=4 {
        assert_eq!(choose(roll, Some(ai.clone()), &legal, &mut rng), ai);
    }
    for roll in 5..=7 {
        assert_eq!(choose(roll, Some(ai.clone()), &legal, &mut rng), hit);
    }
    let waits = [step.clone(), Command::EndPhase];
    assert_eq!(
        choose(6, Some(ai.clone()), &waits, &mut rng),
        ai,
        "nothing acts"
    );
    for _ in 0..20 {
        let any = choose(8, Some(ai.clone()), &legal, &mut rng);
        assert_ne!(any, Command::EndPhase);
    }
    let end_first = [Command::EndPhase, step.clone()];
    assert_eq!(choose(8, None, &end_first, &mut rng), step);
    assert_eq!(choose(9, Some(ai), &legal, &mut rng), Command::EndPhase);
    assert_eq!(
        choose(2, None, &legal, &mut rng),
        Command::EndPhase,
        "no AI command"
    );
}

#[test]
fn a_potion_is_only_for_someone_hurt() {
    let potions = |commands: &[Command]| {
        let potion = |c: &&Command| {
            matches!(
                c,
                Command::Act {
                    action: UnitAction::UseItem { .. },
                    ..
                }
            )
        };
        commands.iter().filter(potion).count()
    };
    let fresh = start(vec![lord(player(1, 1)), enemy(2, 8)]);
    assert_ne!(potions(&legal_commands(&fresh)), 0, "the rules allow it");
    assert_eq!(potions(&useful(&fresh)), 0);
    let mut sore = lord(player(1, 1));
    sore.hp -= 3;
    let hurt = start(vec![sore, enemy(2, 8)]);
    assert_ne!(potions(&useful(&hurt)), 0);
}

/// A try plays the Player phase it starts in and stops when it ends.
#[test]
fn a_try_plays_one_player_phase() {
    let mut state = start(vec![rigged(lord(player(1, 0))), weak(enemy(2, 8))]);
    let mut rng = SimRng::new(1);
    let commands = try_phase(&mut state, game(), &mut rng, true);
    assert_eq!(commands.last(), Some(&Command::EndPhase));
    assert_eq!(state.outcome(), None);
    assert_eq!((state.turn(), state.phase()), (1, Phase::Enemy));
}

/// The search gives up past its limits: Player phases tried, and the last
/// turn.
#[test]
fn the_search_gives_up_past_its_limits() {
    let content = test_story();
    let within = |steps, turns| {
        let limits = Limits { steps, turns };
        record_within(first_battle(&content), &content, 1, &|_| true, limits).unwrap_err()
    };
    assert_eq!(within(200, 1), "no end in sight on turn 2");
    assert_eq!(within(1, 60), "no end in sight on turn 2");
    assert_eq!(within(0, 60), "no end in sight on turn 1");
}

/// A battle already won before the search starts is no recording unless
/// its win is wanted.
#[test]
fn a_battle_won_before_the_search_counts_only_if_wanted() {
    let mut won = Scripted::new(start(vec![rigged(lord(player(1, 4))), weak(enemy(2, 5))]));
    won.act(1, 4, attack(2));
    let content = game();
    let error = record(won.state.clone(), content, 1, &nobody).unwrap_err();
    assert!(
        error.starts_with("not a win: Some(Victory) on turn 1"),
        "{error}"
    );
    let recorded = record(won.state, content, 1, &every).unwrap();
    assert_eq!(recorded.phases, Vec::<Vec<Command>>::new());
}
