//! The objective's lines (0481) for each objective kind, with and without
//! a village and a turn limit.

use trpg_content::battle_campaign;
use trpg_core::lead::DEFAULT_NAME;
use trpg_core::{
    BattleSetup, GameMode, LeadGender, LeadProfile, Leftovers, Loot, Objective, Pos, TileRect,
    UnitId,
};

use super::*;
use crate::screen::Ctx;
use crate::screen::tests::ctx;
use crate::screens::battle::quick_battle;
use crate::screens::battle::testing::battle_with;

/// Each line as `(label, text)`.
fn shown(lines: &[ObjectiveLine]) -> Vec<(&str, &str)> {
    lines
        .iter()
        .map(|l| (l.label.as_str(), l.text.as_str()))
        .collect()
}

/// The Quick Battle's map and units with `objective`.
fn quick_with(c: &Ctx, objective: Objective) -> BattleState {
    let q = quick_battle(&c.game.content).unwrap();
    battle_with(c, q.map().clone(), q.units().to_vec(), objective)
}

/// The test village (five houses, save three, two rewards), its setup
/// changed by `change`.
fn village(c: &Ctx, change: impl FnOnce(&mut BattleSetup)) -> BattleState {
    let content = &c.game.content;
    let def = &content.battles["test_village"];
    let lead = LeadProfile::new(DEFAULT_NAME, LeadGender::Male);
    let campaign = battle_campaign(content, def, GameMode::Classic, lead);
    let mut setup = campaign.battle_setup(def, &content.tables());
    change(&mut setup);
    BattleState::new(setup).0
}

/// Acceptance: every objective wins with `Win:`; the lead's fall loses.
#[test]
fn each_objective_kind_wins_and_the_lead_falling_loses() {
    let c = ctx();
    let lead = ("Lose:", "Test Lord falls");
    let kinds = [
        (Objective::Rout { turn_limit: None }, "Rout the enemy"),
        (
            Objective::DefeatUnit {
                unit: UnitId(6),
                turn_limit: None,
                leftovers: Leftovers::Flee,
            },
            "Defeat Raider",
        ),
        (
            Objective::Seize {
                pos: Pos::new(5, 5),
                by_lord: true,
                turn_limit: None,
            },
            "Seize the Fort",
        ),
        (Objective::Survive { turns: 5 }, "Survive 5 turns"),
        (
            Objective::Escape {
                area: TileRect {
                    x: 3,
                    y: 5,
                    w: 2,
                    h: 1,
                },
            },
            "Get everyone to the exit (0/4 out)",
        ),
    ];
    for (objective, win) in kinds {
        let s = quick_with(&c, objective);
        let l = lines(&s, None, Words::ENGLISH);
        assert_eq!(shown(&l), [("Win:", win), lead], "{objective:?}");
    }
}

#[test]
fn a_turn_limit_loses_when_its_turn_ends() {
    let c = ctx();
    let s = quick_with(
        &c,
        Objective::Rout {
            turn_limit: Some(8),
        },
    );
    assert_eq!(
        shown(&lines(&s, None, Words::ENGLISH)),
        [
            ("Win:", "Rout the enemy"),
            ("Lose:", "Turn 8 ends"),
            ("or", "Test Lord falls"),
        ]
    );
}

/// Acceptance: the village reads as a condition to fail (Chapter 2's is
/// AND: the objective wins, too many houses burning loses), and its
/// rewards as a bonus.
#[test]
fn a_village_loses_when_too_many_houses_burn_and_offers_its_rewards() {
    let c = ctx();
    let s = village(&c, |_| {});
    assert_eq!(
        shown(&lines(&s, None, Words::ENGLISH)),
        [
            ("Win:", "Rout the enemy"),
            ("Lose:", "3 of 5 houses burn"),
            ("or", "Test Lord falls"),
            ("Bonus:", "Save 4 or 5 houses for rewards"),
        ]
    );
    // With a turn limit too, the village comes first.
    let s = village(&c, |s| {
        s.objective = Objective::Rout {
            turn_limit: Some(6),
        };
    });
    let l = lines(&s, None, Words::ENGLISH);
    assert_eq!(
        shown(&l)[1..3],
        [("Lose:", "3 of 5 houses burn"), ("or", "Turn 6 ends")]
    );
}

#[test]
fn the_bonus_names_only_the_rewards_the_houses_can_give() {
    let c = ctx();
    let bonus = |minimum, rewards: Vec<Loot>| {
        let s = village(&c, |s| {
            let v = s.village.as_mut().unwrap();
            v.minimum = minimum;
            v.rewards = rewards;
        });
        let l = lines(&s, None, Words::ENGLISH);
        l.iter()
            .find(|l| l.label == "Bonus:")
            .map(|l| l.text.clone())
    };
    let gold = || Loot::Gold(100);
    assert_eq!(bonus(3, vec![]), None);
    assert_eq!(
        bonus(3, vec![gold()]).as_deref(),
        Some("Save 4 houses for a reward")
    );
    assert_eq!(
        bonus(2, vec![gold(), gold(), gold()]).as_deref(),
        Some("Save 3 to 5 houses for rewards")
    );
    // Four rewards, but only two houses past the minimum.
    assert_eq!(
        bonus(3, vec![gold(), gold(), gold(), gold()]).as_deref(),
        Some("Save 4 or 5 houses for rewards")
    );
    // Every house needed: nothing past the minimum.
    assert_eq!(bonus(5, vec![gold()]), None);
}

#[test]
fn padded_lines_line_their_texts_up() {
    let line = |label: &str, text: &str| ObjectiveLine {
        label: label.to_owned(),
        text: text.to_owned(),
    };
    let l = [
        line("Win:", "Defeat Hendry"),
        line("Lose:", "3 of 5 houses burn"),
        line("or", "Ellery falls"),
    ];
    let narrow = CellWidths::NARROW;
    assert_eq!(label_width(&l, &narrow), 6);
    assert_eq!(
        padded(&l, &narrow),
        [
            "Win:  Defeat Hendry",
            "Lose: 3 of 5 houses burn",
            "or    Ellery falls"
        ]
    );
    assert_eq!(label_width(&[], &narrow), 0);
}

#[test]
fn the_village_count_is_only_in_a_village() {
    let c = ctx();
    let s = village(&c, |_| {});
    assert_eq!(
        village_count(&s, Words::ENGLISH).as_deref(),
        Some("Houses: 0 saved, 0 lost")
    );
    let q = quick_with(&c, Objective::Rout { turn_limit: None });
    assert_eq!(village_count(&q, Words::ENGLISH), None);
}

/// The lead is the player's lord, not the first player unit listed.
#[test]
fn the_lead_is_the_players_lord() {
    let c = ctx();
    let q = quick_battle(&c.game.content).unwrap();
    let mut units = q.units().to_vec();
    units.reverse();
    let first = units
        .iter()
        .find(|u| u.faction == trpg_core::Faction::Player);
    assert!(first.is_some_and(|u| !u.is_lord), "{first:?}");
    let s = battle_with(
        &c,
        q.map().clone(),
        units,
        Objective::Rout { turn_limit: None },
    );
    let l = lines(&s, None, Words::ENGLISH);
    assert_eq!(shown(&l)[1], ("Lose:", "Test Lord falls"));
}

/// Ticket 1282: the bonus is one plural message, its English as before
/// for 0, 1, 2 and 5 rewards on offer.
#[test]
fn the_bonus_message_reads_as_before_for_each_count() {
    let c = ctx();
    let bonus = |rewards: usize| {
        let s = village(&c, |s| {
            let v = s.village.as_mut().unwrap();
            v.minimum = 0;
            v.rewards = vec![Loot::Gold(100); rewards];
        });
        let l = lines(&s, None, Words::ENGLISH);
        l.iter()
            .find(|l| l.label == "Bonus:")
            .map(|l| l.text.clone())
    };
    assert_eq!(bonus(0), None);
    assert_eq!(bonus(1).as_deref(), Some("Save 1 houses for a reward"));
    assert_eq!(bonus(2).as_deref(), Some("Save 1 or 2 houses for rewards"));
    assert_eq!(bonus(5).as_deref(), Some("Save 1 to 5 houses for rewards"));
}

/// A wide label (two cells a character) pads the labels' column in cells
/// (ticket 0263): the texts start in one column.
#[test]
fn a_wide_label_pads_the_objective_column_in_cells() {
    let line = |label: &str, text: &str| ObjectiveLine {
        label: label.to_owned(),
        text: text.to_owned(),
    };
    let wide = CellWidths::wide("姝怡".chars());
    let l = [
        line("姝怡:", "Defeat Hendry"),
        line("Lose:", "Ellery falls"),
    ];
    assert_eq!(label_width(&l, &wide), 6);
    assert_eq!(
        padded(&l, &wide),
        ["姝怡: Defeat Hendry", "Lose: Ellery falls"]
    );
}
