//! The baseline bot plays every shipped battle file (ticket 0505), on the
//! engine's playtest runner (ticket 0506).

use shuyi_core::bots::playtest::play_tries;
use trpg_bots::{BaselineBot, BattleEntrant, BattleStage, Player};
use trpg_content::{Content, battle_campaign};
use trpg_core::{BattleDef, BattleState, GameMode, LeadGender, LeadProfile};

/// `def` as the playtest fields it: the characters of its player slots.
fn stage(content: &Content, def: &BattleDef) -> BattleStage {
    let setup = |mode| {
        let lead = LeadProfile::new("Bot", LeadGender::Male);
        battle_campaign(content, def, mode, lead).battle_setup(def, &content.tables())
    };
    BattleStage {
        classic: setup(GameMode::Classic),
        casual: setup(GameMode::Casual),
    }
}

fn baseline(content: &Content, stage: &BattleStage) -> BattleEntrant {
    BattleEntrant::new(
        "baseline",
        GameMode::Classic,
        Player::Baseline,
        content.ai,
        stage,
    )
}

#[test]
fn baseline_bot_plays_a_battle_to_the_end() {
    let content = trpg_content::load_embedded().unwrap();
    assert!(!content.battles.is_empty());
    for (id, def) in &content.battles {
        let stage = stage(&content, def);
        let tries = play_tries(&stage, &baseline(&content, &stage), 1..6, 60)
            .unwrap_or_else(|e| panic!("{id}: {e}"));
        let finished = tries.iter().filter(|t| t.outcome.is_some()).count();
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
/// `cargo xtask playtest ch01 --bot baseline --runs 200` plays the same
/// tries and prints every try; `--bot all` has the personas play it too.
#[test]
#[ignore = "a manual measurement: 200 battles"]
fn chapter_1_difficulty() {
    const TRIES: u64 = 200;
    let content = trpg_content::load_embedded().unwrap();
    let stage = stage(&content, &content.battles["ch01"]);
    let tries = play_tries(&stage, &baseline(&content, &stage), 1..TRIES + 1, 60)
        .unwrap_or_else(|e| panic!("{e}"));
    let won: i64 = tries.iter().map(|t| t.measures["won"]).sum();
    let fallen: i64 = tries.iter().map(|t| t.measures["deaths"]).sum();
    let rate = won * 100 / 200;
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
/// can't be killed) and 0513's (a green unit with Potions of its own): the
/// bots finish them, each rule happens in some try,
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
                "test_green" => {
                    matches!(e, Event::ItemUsed { unit, .. } if *unit == trpg_core::UnitId(4))
                }
                _ => matches!(e, Event::UnitFell { unit } if *unit == trpg_core::UnitId(5)),
            })
        })
    };
    for id in ["test_escape", "test_retreat", "test_hold", "test_green"] {
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
