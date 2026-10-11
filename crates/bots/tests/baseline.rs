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
