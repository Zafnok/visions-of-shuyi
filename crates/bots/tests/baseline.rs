//! The baseline bot plays every shipped battle file (ticket 0505), on the
//! engine's playtest runner (ticket 0506).

use shuyi_core::bots::playtest::play_tries;
use trpg_bots::{BattleEntrant, BattleStage, Player};
use trpg_content::{Content, battle_campaign};
use trpg_core::{BattleDef, GameMode, LeadGender, LeadProfile};

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
