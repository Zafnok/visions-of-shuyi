//! The baseline bot plays every shipped battle file (ticket 0505).

use trpg_bots::{BaselineBot, play_battle};
use trpg_content::battle_campaign;
use trpg_core::{BattleState, GameMode, LeadGender, LeadProfile};

#[test]
fn baseline_bot_plays_a_battle_to_the_end() {
    let content = trpg_content::load_embedded().unwrap();
    let tables = content.tables();
    assert!(!content.battles.is_empty());
    for (id, def) in &content.battles {
        let lead = LeadProfile::new("Bot", LeadGender::Male);
        let campaign = battle_campaign(&content, def, GameMode::Classic, lead);
        let mut finished = 0;
        for seed in 1..=5 {
            let mut setup = campaign.battle_setup(def, &tables);
            setup.seed = seed;
            let (state, _) = BattleState::new(setup);
            let mut bot = BaselineBot::new(content.ai);
            let measures = play_battle(state, &mut bot, &content.ai, 60)
                .unwrap_or_else(|e| panic!("{id}, seed {seed}: {e}"));
            assert!(measures.player_commands > 0, "{id}, seed {seed}");
            finished += u32::from(measures.outcome.is_some());
        }
        assert!(finished > 0, "{id}: no try ended in 60 turns");
    }
}

/// Chapter 1's difficulty sim (ticket 0803): the baseline bot, a mindless
/// player, plays it with 200 seeds. The target is 30-70% won: a thinking
/// player should win comfortably, a mindless one shouldn't always win. Run
/// it after changing Chapter 1's units or the combat numbers:
///
/// ```text
/// cargo test -p trpg-bots --test baseline -- --ignored
/// ```
///
/// `cargo xtask playtest ch01 --runs 200` plays the same tries and prints
/// the win rate, the units lost per try and every try.
#[test]
#[ignore = "a manual measurement: 200 battles"]
fn chapter_1_difficulty() {
    const TRIES: u32 = 200;
    let content = trpg_content::load_embedded().unwrap();
    let tables = content.tables();
    let def = &content.battles["ch01"];
    let lead = LeadProfile::new("Bot", LeadGender::Male);
    let campaign = trpg_content::new_campaign(&content, GameMode::Classic, lead);
    let (mut won, mut fallen) = (0, 0);
    for seed in 1..=u64::from(TRIES) {
        let mut setup = campaign.battle_setup(def, &tables);
        setup.seed = seed;
        let (state, _) = BattleState::new(setup);
        let mut bot = BaselineBot::new(content.ai);
        let measures = play_battle(state, &mut bot, &content.ai, 60)
            .unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        won += u32::from(measures.outcome == Some(trpg_core::Outcome::Victory));
        fallen += measures.player_fallen.len();
    }
    let rate = won * 100 / TRIES;
    assert!(
        (30..=70).contains(&rate),
        "won {won} of {TRIES} ({rate}%), outside 30-70%; {fallen} player units lost in all"
    );
}

/// A try: every command with its events, and the state it ended in.
type Try = (
    Vec<(trpg_core::Command, Vec<trpg_core::Event>)>,
    BattleState,
);

/// One try of battle `id` with `seed`, the baseline bot on the player's
/// side and the AI on the others, for at most 60 turns: every command
/// and its events, in order, and the state it ends in.
fn recorded(id: &str, seed: u64) -> Try {
    use shuyi_core::Bot;
    use trpg_core::{Command, Phase, next_command};
    let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let def = &content.battles[id];
    let lead = LeadProfile::new("Bot", LeadGender::Male);
    let campaign = battle_campaign(&content, def, GameMode::Classic, lead);
    let mut setup = campaign.battle_setup(def, &content.tables());
    setup.seed = seed;
    let (mut state, _) = BattleState::new(setup);
    let mut bot = BaselineBot::new(content.ai);
    let mut record = Vec::new();
    while state.outcome().is_none() && state.turn() <= 60 {
        let command = if state.phase() == Phase::Player {
            bot.choose(&state)
        } else {
            next_command(&state, &content.ai).unwrap_or(Command::EndPhase)
        };
        let events = state
            .apply(&command)
            .unwrap_or_else(|e| panic!("{id}, seed {seed}: {command:?}: {e}"));
        record.push((command, events));
    }
    (record, state)
}

/// Ticket 0319's test battles (an escape, a boss who retreats, a boss who
/// can't be killed): the bots finish them, each rule happens in some try,
/// and every try's commands replay to the same events and the same end.
#[test]
fn the_act_1_rule_battles_finish_and_replay_identically() {
    use trpg_core::Event;
    let content = trpg_content::load_embedded().unwrap();
    let happened = |id: &str, seeds: &[Try]| {
        seeds.iter().any(|(record, _)| {
            record.iter().flat_map(|(_, e)| e).any(|e| match id {
                "test_escape" => matches!(e, Event::UnitEscaped { .. }),
                "test_hold" => matches!(e, Event::UnitDowned { .. }),
                _ => matches!(e, Event::UnitFell { unit } if *unit == trpg_core::UnitId(5)),
            })
        })
    };
    for id in ["test_escape", "test_retreat", "test_hold"] {
        let tries: Vec<_> = (1..=5).map(|seed| recorded(id, seed)).collect();
        for (seed, (record, end)) in (1..).zip(&tries) {
            assert!(
                end.outcome().is_some(),
                "{id}, seed {seed}: no end in 60 turns"
            );
            // The replay: the same setup, the same commands.
            let def = &content.battles[id];
            let lead = LeadProfile::new("Bot", LeadGender::Male);
            let campaign = battle_campaign(&content, def, GameMode::Classic, lead);
            let mut setup = campaign.battle_setup(def, &content.tables());
            setup.seed = seed;
            let (mut replay, _) = BattleState::new(setup);
            for (command, events) in record {
                assert_eq!(
                    replay.apply(command).as_ref(),
                    Ok(events),
                    "{id}, seed {seed}"
                );
            }
            assert_eq!(&replay, end, "{id}, seed {seed}");
        }
        assert!(happened(id, &tries), "{id}: its rule never came up");
    }
}
