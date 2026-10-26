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
/// can't be killed), 0513's (a green unit with Potions of its own) and
/// 0313's (a village to save, with raiders): the
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
                "test_village" => matches!(e, Event::HouseBurned { .. }),
                "test_green" => {
                    matches!(e, Event::ItemUsed { unit, .. } if *unit == trpg_core::UnitId(4))
                }
                _ => matches!(e, Event::UnitFell { unit } if *unit == trpg_core::UnitId(5)),
            })
        })
    };
    for id in [
        "test_escape",
        "test_retreat",
        "test_hold",
        "test_green",
        "test_village",
    ] {
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

/// Chapter 1 is won the moment Red Harl falls, with his company still on
/// the map, and the rest of it runs (ticket 0845): the lead, unable to
/// miss, strikes Harl at 1 HP from a tile next to him on turn 1.
#[test]
fn chapter_1_is_won_when_harl_falls_first_and_the_rest_flee() {
    use trpg_core::{CharacterId, Command, Event, Faction, Objective, Outcome, Pos, UnitAction};
    let content = trpg_content::load_embedded().unwrap();
    let def = &content.battles["ch01"];
    let lead = LeadProfile::new("Bot", LeadGender::Male);
    let campaign = battle_campaign(&content, def, GameMode::Classic, lead);
    let base = campaign.battle_setup(def, &content.tables());
    let harl_id = Some(CharacterId("red_captain".into()));
    let harl = base.units.iter().find(|u| u.character == harl_id).unwrap();
    let (harl, at) = (harl.id, harl.pos);
    assert!(matches!(base.objective, Objective::DefeatUnit { unit, .. } if unit == harl));
    let enemies = base
        .units
        .iter()
        .filter(|u| u.faction == Faction::Enemy)
        .count();
    // The first tile next to Harl the lead can stand on and strike from.
    let next_to = [(0, 1), (-1, 0), (1, 0), (0, -1)].map(|(dx, dy)| Pos::new(at.x + dx, at.y + dy));
    let won = next_to.into_iter().find_map(|pos| {
        if base.units.iter().any(|u| u.pos == pos) {
            return None;
        }
        let mut setup = base.clone();
        for u in &mut setup.units {
            if u.id == harl {
                u.hp = 1;
            }
            if u.id == trpg_core::UnitId(1) {
                (u.pos, u.stats.dex, u.stats.str) = (pos, 99, 99);
            }
        }
        let (mut state, _) = BattleState::new(setup);
        let action = UnitAction::Attack {
            target: harl,
            slot: 0,
            active: None,
            art: None,
        };
        let unit = trpg_core::UnitId(1);
        let events = state
            .apply(&Command::Act {
                unit,
                dest: pos,
                action,
            })
            .ok()?;
        Some((state, events))
    });
    let (state, events) = won.expect("no tile next to Harl to strike from");
    assert_eq!(state.outcome(), Some(Outcome::Victory));
    assert_eq!(state.turn(), 1);
    // Everyone else ran: none defeated, none left on the map.
    assert_eq!(state.fled().len(), enemies - 1);
    assert!(state.units().iter().all(|u| u.faction != Faction::Enemy));
    assert!(events.iter().any(|e| matches!(e, Event::Fled { .. })));
}
