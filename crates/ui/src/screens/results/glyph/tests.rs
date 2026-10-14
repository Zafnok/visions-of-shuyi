//! Tests of where the results screen puts what its view says, on rewards
//! made by hand (`results::tests`).

use super::*;
use crate::input::Action;
use crate::screen::{Screen, tests::ctx};
use crate::screens::battle::progress::{LEVEL_BANNER_ROW, LEVEL_TEXT_X};
use crate::screens::results::tests::{deployed, press, rewards, screen, state, wait};
use crate::screens::results::{FILL_S, INTRO_S, ResultsScreen};
use crate::words::Words;
use trpg_core::BattleRewards;

fn drawn(s: &ResultsScreen) -> GlyphBuffer {
    let c = ctx();
    let blank = Cell::new(
        '?',
        c.palette.get(UiColor::Text),
        c.palette.get(UiColor::Text),
    );
    let mut buf = GlyphBuffer::new(100, 32, blank);
    s.draw(&c, &mut buf);
    buf
}

fn row_text(buf: &GlyphBuffer, y: i32) -> String {
    let row: String = (0..100)
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect();
    row.trim().to_owned()
}

fn shows(buf: &GlyphBuffer, text: &str) -> bool {
    (0..32).any(|y| row_text(buf, y).contains(text))
}

#[test]
fn the_leads_level_up_page_shows_the_face_the_player_picked() {
    use crate::screens::battle::progress::shows_face_of;
    use trpg_core::CharacterId;
    use trpg_core::lead::{LEAD_ID, LeadGender, LeadProfile};
    for (gender, face) in [(LeadGender::Male, "lead_m"), (LeadGender::Female, "lead_f")] {
        let st = state();
        let mut rewards = rewards(&st);
        rewards.deployed[0].character = Some(CharacterId(LEAD_ID.into()));
        let mut s = ResultsScreen::new(&rewards, (&st, Words::ENGLISH), 3, 1500);
        press(&mut s, Action::Confirm);
        press(&mut s, Action::Confirm);
        let mut c = ctx();
        c.lead = LeadProfile::new("Ren", gender);
        let blank = Cell::new(
            '?',
            c.palette.get(UiColor::Text),
            c.palette.get(UiColor::Text),
        );
        let mut buf = GlyphBuffer::new(100, 32, blank);
        s.draw(&c, &mut buf);
        assert!(shows(&buf, "LEVEL UP!"));
        assert!(shows_face_of(&buf, &c, face), "{face}");
        assert!(!shows_face_of(&buf, &c, "test_lord"));
    }
}

#[test]
fn the_level_up_page_is_moved_from_the_map_view_to_the_middle_of_the_screen() {
    let mut s = screen();
    press(&mut s, Action::Confirm);
    let buf = drawn(&s);
    assert!(shows(&buf, "Gold for clearing the map"));
    press(&mut s, Action::Confirm);
    let buf = drawn(&s);
    assert!(!shows(&buf, "Gold for clearing the map"));
    let x = LEVEL_TEXT_X + 15;
    let banner: String = (x..x + 9)
        .map(|x| buf.get(x, LEVEL_BANNER_ROW).unwrap().glyph)
        .collect();
    assert_eq!(banner, "LEVEL UP!");
    assert!(shows(&buf, "Test Lord"));
    assert!(shows(&buf, "Lv 1 → 2"));
    assert!(!shows(&buf, "Str   6 → 7   +1"));
    // The help is the page's: it is still revealing its stats.
    assert!(shows(&buf, "f skip · hold f fast"));
    assert!(!shows(&buf, "f continue"));
    // It reveals its stats and then waits, however long.
    wait(&mut s, 30.0);
    assert!(shows(&drawn(&s), "Str   6 → 7   +1"));
    assert!(shows(&drawn(&s), "f continue"));
}

#[test]
fn a_press_mid_reveal_shows_every_stat() {
    let mut s = screen();
    press(&mut s, Action::Confirm);
    press(&mut s, Action::Confirm);
    assert!(!shows(&drawn(&s), "Spd"));
    press(&mut s, Action::Cancel);
    assert!(shows(&drawn(&s), "Spd   7 → 9   +2"));
}

#[test]
fn the_gold_line_sits_in_the_panel_and_no_units_means_no_rule() {
    let s0 = state();
    let r = BattleRewards {
        clear_gold: 500,
        deployed: deployed(&s0),
        ..BattleRewards::default()
    };
    let s = ResultsScreen::new(&r, (&s0, Words::ENGLISH), 3, 500);
    let buf = drawn(&s);
    assert_eq!(
        row_text(&buf, GOLD_ROW),
        "║   Gold for clearing the map                   +500 (now 500)         ║"
    );
    assert!(!shows(&buf, "Bonus EXP for each unit"));
    assert!(!shows(&buf, "EXP"));
    assert!(!shows(&buf, "─"));
    assert!(shows(&buf, "f continue"));
}

#[test]
fn a_battle_without_clear_gold_has_the_bonus_and_a_rule_under_the_lines() {
    let s0 = state();
    let mut r = rewards(&s0);
    r.clear_gold = 0;
    let buf = drawn(&ResultsScreen::new(&r, (&s0, Words::ENGLISH), 3, 40));
    assert!(!shows(&buf, "Gold for clearing the map"));
    assert!(!shows(&buf, "now"));
    assert!(shows(&buf, "3 of 3"));
    assert!(shows(
        &buf,
        "Bonus EXP for each unit                     +21"
    ));
    // The rule under the lines, with the panel's margin on both sides.
    let rule = format!("║   {}   ║", "─".repeat(64));
    assert_eq!(row_text(&buf, PANEL.y + 7), rule);
}

#[test]
fn rows_are_columns_with_a_bar_and_a_mark() {
    let mut s = screen();
    let lord = |s: &ResultsScreen| row_text(&drawn(s), UNIT_ROW);
    assert!(lord(&s).contains("Test Lord     Exile    Lv 1  EXP ██████████████████░░ 90"));
    assert!(!lord(&s).contains("LEVEL UP"));
    assert!(shows(&drawn(&s), "f skip · hold f fast"));
    wait(&mut s, INTRO_S + FILL_S * 0.5);
    assert!(lord(&s).contains("Lv 2  EXP ░░░░░░░░░░░░░░░░░░░░  0  LEVEL UP"));
    press(&mut s, Action::Confirm);
    let buf = drawn(&s);
    assert!(lord(&s).contains("Lv 2  EXP ██░░░░░░░░░░░░░░░░░░ 11  LEVEL UP"));
    // A blank row between units; the unit at the cap keeps its bar.
    assert_eq!(
        row_text(&buf, UNIT_ROW + 1),
        "║".to_owned() + &" ".repeat(70) + "║"
    );
    assert!(
        row_text(&buf, UNIT_ROW + 2)
            .contains("Test Knight   Guard    Lv 1  EXP ██████░░░░░░░░░░░░░░ 31")
    );
    assert!(!row_text(&buf, UNIT_ROW + 2).contains("LEVEL UP"));
    assert!(
        row_text(&buf, UNIT_ROW + 4)
            .contains("Test Archer   Archer   Lv 1  EXP ░░░░░░░░░░░░░░░░░░░░  0")
    );
    assert!(shows(&buf, "f continue"));
}

#[test]
fn a_big_army_drops_the_blank_rows() {
    let s0 = state();
    let unit = deployed(&s0).remove(1);
    let army = |n: usize| {
        let mut r = rewards(&s0);
        r.events.clear();
        r.deployed = vec![unit.clone(); n];
        drawn(&ResultsScreen::new(&r, (&s0, Words::ENGLISH), 3, 0))
    };
    // Seven fit with blank rows between them; eight don't.
    let buf = army(7);
    assert!(!row_text(&buf, UNIT_ROW + 1).contains("Test Knight"));
    assert!(row_text(&buf, UNIT_ROW + 12).contains("Test Knight"));
    let buf = army(8);
    assert!(row_text(&buf, UNIT_ROW + 1).contains("Test Knight"));
    assert!(row_text(&buf, UNIT_ROW + 7).contains("Test Knight"));
    assert!(!row_text(&buf, UNIT_ROW + 8).contains("Test Knight"));
    // Never over the panel's border.
    let buf = army(40);
    assert!(row_text(&buf, PANEL.y + PANEL.h - 2).contains("Test Knight"));
    assert!(row_text(&buf, PANEL.y + PANEL.h - 1).starts_with('╚'));
}

#[test]
fn long_names_are_cut_to_their_columns() {
    let s0 = state();
    let mut r = rewards(&s0);
    r.deployed[0].name = "Bartholomew the Bold".into();
    let buf = drawn(&ResultsScreen::new(&r, (&s0, Words::ENGLISH), 3, 0));
    assert!(row_text(&buf, UNIT_ROW).contains("Bartholomew t Exile    Lv 1"));
}

#[test]
fn a_bar_fills_a_fifth_of_its_cells_per_twenty_exp() {
    for (exp, cells) in [
        (0, 0),
        (4, 0),
        (5, 1),
        (90, 18),
        (99, 19),
        (100, 20),
        (150, 20),
    ] {
        assert_eq!(bar_cells(exp, 100), cells, "{exp} of 100");
    }
    assert_eq!(bar_cells(5, 0), 0, "a bar of nothing is empty");
}
