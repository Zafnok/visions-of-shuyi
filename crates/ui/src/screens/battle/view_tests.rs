//! The battle screen's view (ADR-0054): each thing it shows appears and
//! goes with the battle and the mode. Where and how they are painted is
//! the skin's (`glyph/tests.rs`).

use trpg_content::battle_campaign;
use trpg_core::lead::DEFAULT_NAME;
use trpg_core::{GameMode, LeadGender, LeadProfile, Phase, Pos, UnitId};

use super::banner::{BannerKind, PHASE_BANNER_S};
use super::testing::wait;
use super::tests::{frame, lord_menu, quick, step};
use super::view::{
    BannerView, BattleMenuView, DialogView, MenuPlace, MessageKind, MessageView, NotesView,
    QuestionView,
};
use super::*;
use crate::harness::Harness;
use crate::screen::tests::ctx;
use crate::widgets::menu::Menu;

/// The labels of the menu open, if any.
fn menu_labels(view: &BattleView<'_>) -> Option<Vec<String>> {
    let menu = view.menu.as_ref()?;
    Some(menu.menu.items.iter().map(|i| i.label.clone()).collect())
}

#[test]
fn browsing_shows_the_panel_the_toggles_and_the_help_and_nothing_over_the_map() {
    let c = ctx();
    let s = quick();
    let view = s.view(&c);
    assert!(view.panel.is_some());
    assert_eq!(view.status, s.status(&c));
    assert_eq!(view.help, s.help(&c));
    assert_eq!(view.debug_hint.as_deref(), Some("F2 debug"));
    assert_eq!(view.scene, s.scene(&c));
    assert_eq!(view.menu, None);
    assert!(view.popups.is_empty());
    assert_eq!(view.stage, None);
    assert_eq!(view.dialog, None);
    assert_eq!(view.notes, None);
    assert!(view.noted.is_empty());
    assert_eq!(view.banner, None);
    assert_eq!(view.tip, None);
    assert_eq!(view.message, None);
    let old = view.old;
    assert!(old.forecast.is_none() && old.info.is_none() && old.rewind.is_none());
    assert!(old.fight_box.is_none() && old.progress.is_none());
}

#[test]
fn the_map_menu_opens_beside_the_cursor_and_closes() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Cancel]);
    let view = s.view(&c);
    let menu = view.menu.as_ref().unwrap();
    assert_eq!(menu.place, MenuPlace::Beside(s.cursor().pos));
    assert_eq!(menu.header, None);
    assert_eq!(
        menu_labels(&view).unwrap(),
        [
            "Units",
            "Objective",
            "Options",
            "Suspend",
            "Restart Battle",
            "End Turn"
        ]
    );
    step(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.view(&c).menu, None);
}

#[test]
fn the_unit_list_opens_in_the_middle_of_the_map() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Cancel, Action::Confirm]);
    assert!(matches!(s.mode(), Mode::UnitList { .. }), "{:?}", s.mode());
    let view = s.view(&c);
    assert_eq!(
        view.menu.as_ref().map(|m| m.place),
        Some(MenuPlace::Centred)
    );
    assert!(menu_labels(&view).unwrap().len() > 1);
    step(&mut s, &mut c, &[Action::Cancel]);
    assert!(matches!(
        s.view(&c).menu,
        Some(BattleMenuView {
            place: MenuPlace::Beside(_),
            ..
        })
    ));
}

#[test]
fn the_action_menu_opens_beside_where_the_unit_will_stand() {
    let mut c = ctx();
    let mut s = lord_menu(&mut c);
    let menu = s.view(&c).menu.unwrap();
    assert_eq!(menu.place, MenuPlace::Beside(Pos::new(5, 5)));
    step(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.view(&c).menu, None);
}

#[test]
fn the_item_list_has_the_packs_size_in_its_header() {
    let mut c = ctx();
    let mut s = lord_menu(&mut c);
    let Mode::ActionMenu { sel, .. } = s.mode().clone() else {
        panic!("{:?}", s.mode());
    };
    // The list `Item` opens (disabled for the lord here, so set directly).
    s.mode = Mode::ItemMenu {
        sel,
        menu: Menu::new(vec![]),
        groups: vec![],
    };
    let header = s.view(&c).menu.unwrap().header.unwrap();
    assert_eq!(header.name, items::pack_header(s.state(), Words::ENGLISH));
    assert_eq!(header.uses, None);
    step(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.view(&c).menu.and_then(|m| m.header), None);
}

#[test]
fn the_objective_page_shows_the_goal_and_the_turn_and_closes() {
    let mut c = ctx();
    let mut s = quick();
    step(
        &mut s,
        &mut c,
        &[Action::Cancel, Action::CursorDown, Action::Confirm],
    );
    let Some(DialogView::Objective(page)) = s.view(&c).dialog else {
        panic!("{:?}", s.mode());
    };
    assert_eq!(page.objective, "Rout the enemy");
    assert_eq!(page.turn, "Turn 1");
    // The Quick Battle has no notes.
    assert_eq!(page.notes, None);
    assert!(s.view(&c).noted.is_empty());
    assert_eq!(s.view(&c).menu, None);
    step(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.view(&c).dialog, None);
}

#[test]
fn the_end_turn_question_asks_and_goes() {
    let mut c = ctx();
    let mut s = quick();
    wait(&mut s.state, 2);
    step(&mut s, &mut c, &[Action::EndTurn]);
    assert_eq!(
        s.view(&c).dialog,
        Some(DialogView::Question(QuestionView {
            question: "End turn with 3 units ready?".to_owned(),
            answers: "f yes / d no".to_owned(),
        }))
    );
    step(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.view(&c).dialog, None);
}

#[test]
fn restart_and_suspend_ask_first() {
    let mut c = ctx();
    for (mode, question) in [
        (Mode::RestartPrompt, "Restart the battle from turn 1?"),
        (
            Mode::SuspendPrompt,
            "Suspend the battle and return to the title?",
        ),
    ] {
        let mut s = quick();
        s.mode = mode;
        let Some(DialogView::Question(q)) = s.view(&c).dialog else {
            panic!("no question");
        };
        assert_eq!(q.question, question);
        step(&mut s, &mut c, &[Action::Cancel]);
        assert_eq!(s.view(&c).dialog, None);
    }
}

/// The Quick Battle as the flow starts it, and its start events.
fn quick_start(c: &Ctx) -> BattleScreen {
    let def = &c.content.battles[QUICK_BATTLE];
    let lead = LeadProfile::new(DEFAULT_NAME, LeadGender::Male);
    let campaign = battle_campaign(&c.content, def, GameMode::Classic, lead);
    let (state, events) = BattleState::new(campaign.battle_setup(def, &c.content.tables()));
    BattleScreen::start(state, &events)
}

#[test]
fn a_phase_banner_shows_then_expires() {
    let mut c = ctx();
    let mut s = quick_start(&c);
    assert_eq!(
        s.view(&c).banner,
        Some(BannerView {
            kind: BannerKind::Phase {
                phase: Phase::Player,
                turn: 1
            },
            title: "PLAYER PHASE".to_owned(),
            subtitle: "Turn 1".to_owned(),
        })
    );
    frame(&mut s, &mut c, &[], PHASE_BANNER_S * 1.1, false);
    assert_eq!(s.view(&c).banner, None);
}

#[test]
fn a_toast_shows_for_a_moment() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::ToggleAutoEnd]);
    assert_eq!(
        s.view(&c).message,
        Some(MessageView {
            text: "Auto-end: ON".to_owned(),
            kind: MessageKind::Toast,
        })
    );
    frame(&mut s, &mut c, &[], 10.0, false);
    assert_eq!(s.view(&c).message, None);
}

#[test]
fn the_info_screen_is_the_units_and_closes() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Info]);
    assert_eq!(s.view(&c).old.info.map(|u| u.id), Some(UnitId(1)));
    step(&mut s, &mut c, &[Action::Cancel]);
    assert!(s.view(&c).old.info.is_none());
}

#[test]
fn the_forecast_takes_the_panels_place_while_an_attack_is_aimed() {
    let state = super::testing::skirmish(&ctx(), 20);
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state)));
    let view = |h: &Harness| {
        let s = h.battle().unwrap();
        let v = s.view(h.game().ctx());
        (v.old.forecast.is_some(), v.panel.is_some())
    };
    // The lord moved a step right, its action menu open on `Attack`.
    h.keys("f Right f").wait(0.5);
    assert_eq!(view(&h), (false, true));
    h.keys("f");
    assert_eq!(view(&h), (true, false));
    h.keys("d");
    assert_eq!(view(&h), (false, true));
}

#[test]
fn the_rewind_screen_shows_only_the_map_its_list_and_the_help() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Rewind]);
    let view = s.view(&c);
    assert!(view.old.rewind.is_some());
    assert_eq!(view.help, s.help(&c));
    assert_eq!((view.panel, view.status.as_str()), (None, ""));
    assert_eq!((view.menu, view.message), (None, None));
    step(&mut s, &mut c, &[Action::Cancel]);
    let view = s.view(&c);
    assert!(view.old.rewind.is_none());
    assert!(view.panel.is_some());
}

#[test]
fn the_battle_notes_show_at_the_start_with_their_units_and_close() {
    let mut c = ctx();
    let lead = LeadProfile::new(DEFAULT_NAME, LeadGender::Male);
    let campaign = trpg_content::new_campaign(&c.content, GameMode::Classic, lead);
    let def = &c.content.battles["test"];
    let (state, events) = BattleState::new(campaign.battle_setup(def, &c.content.tables()));
    let mut s = BattleScreen::start(state, &events);
    let view = s.view(&c);
    let Some(NotesView { title, lines }) = view.notes else {
        panic!("no notes");
    };
    assert_eq!(title, "BATTLE NOTES");
    assert!(!lines.is_empty());
    // The brigand the first note is about, where it stands.
    let brigand = s.state().unit(UnitId(3)).unwrap().pos;
    assert!(view.noted.contains(&brigand), "{:?}", view.noted);
    step(&mut s, &mut c, &[Action::Confirm]);
    let view = s.view(&c);
    assert_eq!(view.notes, None);
    assert!(view.noted.is_empty());
}
