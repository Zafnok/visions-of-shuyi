//! The battle screen's view (ADR-0054): each thing it shows appears and
//! goes with the battle and the mode. Where and how they are painted is
//! the skin's (`glyph/tests.rs`).

use trpg_content::battle_campaign;
use trpg_core::lead::DEFAULT_NAME;
use trpg_core::{GameMode, LeadGender, LeadProfile, Phase, Pos, UnitId};

use super::banner::{BannerKind, PHASE_BANNER_S};
use super::info;
use super::testing::{Stacked, wait};
use super::tests::{frame, lord_menu, quick, step};
use super::view::{
    BannerView, BattleMenuView, DialogView, MenuPlace, MessageKind, MessageView, NotesView,
    QuestionView,
};
use super::*;
use crate::harness::Harness;
use crate::screen::tests::ctx;
use crate::widgets::menu::{Menu, MenuItem, english};

/// The labels of the menu open, if any.
fn menu_labels(view: &BattleView) -> Option<Vec<String>> {
    let menu = view.menu.as_ref()?;
    Some(menu.menu.items.iter().map(|i| english(&i.label)).collect())
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
    assert_eq!(view.popups, [] as [crate::screens::battle::HealPopup; 0]);
    assert_eq!(view.stage, None);
    assert_eq!(view.dialog, None);
    assert_eq!(view.notes, None);
    assert_eq!(view.noted, [] as [trpg_core::Pos; 0]);
    assert_eq!(view.banner, None);
    assert_eq!(view.tip, None);
    assert_eq!(view.message, None);
    assert!(view.forecast.is_none() && view.info.is_none());
    assert!(view.fight_box.is_none() && view.progress.is_none());
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
            "Guides",
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
    let group = items::PackGroup {
        item: trpg_core::ItemId::new("potion"),
        index: 0,
        count: 1,
        targets: vec![],
    };
    let line = MenuItem::new("Potion").with_value(super::mode::Line::Item(group));
    s.mode = Mode::List {
        sel,
        menu: Menu::new(vec![line]),
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
    assert_eq!(s.view(&c).noted, [] as [trpg_core::Pos; 0]);
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
    let def = &c.game.content.battles[QUICK_BATTLE];
    let lead = LeadProfile::new(DEFAULT_NAME, LeadGender::Male);
    let campaign = battle_campaign(&c.game.content, def, GameMode::Classic, lead);
    let (state, events) = BattleState::new(campaign.battle_setup(def, &c.game.content.tables()));
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
    let info = s.view(&c).info.expect("the info screen");
    assert_eq!(info.name, "Test Lord");
    assert_eq!(
        info,
        info::view((s.state(), c.words()), s.state().unit(UnitId(1)).unwrap())
    );
    step(&mut s, &mut c, &[Action::Cancel]);
    assert!(s.view(&c).info.is_none());
}

/// The Quick Battle with the lord at C with the knight and at B with the
/// mage (viewed at camp), and the mage benched, as a battle screen.
fn bonded_quick(c: &Ctx) -> BattleScreen {
    use trpg_core::{CharacterId, SupportPair};
    let content = &c.game.content;
    let def = &content.battles["quick"];
    let lead = LeadProfile::new(DEFAULT_NAME, LeadGender::Male);
    let mut campaign = battle_campaign(content, def, GameMode::Classic, lead);
    let table = &content.supports;
    let pair = |b: &str| SupportPair::new(CharacterId("test_lord".into()), CharacterId(b.into()));
    for (partner, points) in [("test_knight", 20), ("test_mage", 80)] {
        campaign
            .supports
            .gain(&pair(partner), points.min(20), table);
        campaign.supports.view(&pair(partner), table).unwrap();
        if points > 20 {
            campaign.supports.gain(&pair(partner), points, table);
            campaign.supports.view(&pair(partner), table).unwrap();
        }
    }
    let mut setup = campaign.battle_setup(def, &content.tables());
    let mage = Some(CharacterId("test_mage".into()));
    setup.units.retain(|u| u.character != mage);
    BattleScreen::new(trpg_core::BattleState::new(setup).0)
}

/// 1003: the info screen lists the unit's support partners with a rank,
/// a benched one by name; a unit with none lists nothing.
#[test]
fn the_info_screen_lists_the_units_supports_and_ranks() {
    let mut c = ctx();
    let mut s = bonded_quick(&c);
    step(&mut s, &mut c, &[Action::Info]);
    let info = s.view(&c).info.expect("the info screen");
    assert_eq!(info.supports_title, "Supports");
    let supports = [("Test Knight", "C"), ("Test Mage", "B")].map(|(n, r)| (n.into(), r.into()));
    assert_eq!(info.supports, supports);
    let knight = s
        .state()
        .units()
        .iter()
        .find(|u| u.name == "Test Knight")
        .unwrap();
    let words = c.words();
    let of_knight = info::supports((s.state(), words), &c.game.content.names, knight);
    assert_eq!(of_knight, [("Test Lord".to_owned(), "C".to_owned())]);
    let enemy = s
        .state()
        .units()
        .iter()
        .find(|u| u.character.is_none())
        .unwrap();
    let none = info::supports((s.state(), words), &c.game.content.names, enemy);
    assert_eq!(none, []);
}

/// The info screen with its supports section, under the weapon ranks.
#[test]
fn info_supports_snapshot() {
    let c = ctx();
    let mut h = Harness::with_screen(Box::new(bonded_quick(&c)));
    h.keys("e");
    insta::assert_snapshot!(h.snapshot());
}

#[test]
fn the_forecast_takes_the_panels_place_while_an_attack_is_aimed() {
    let state = super::testing::skirmish(&ctx(), 20);
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state)));
    let view = |h: &Harness| {
        let s = h.battle().unwrap();
        let v = s.view(h.game().ctx());
        (v.forecast.is_some(), v.panel.is_some())
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
    let mut s = Stacked::new(quick());
    s.press(&mut c, &[Action::Rewind]);
    let view = s.rewind().expect("the rewind screen").view(&c);
    assert_eq!(view.list.title, "Rewind");
    assert_eq!(view.help, "d close");
    assert_eq!(view.lesson, None);
    s.press(&mut c, &[Action::Cancel]);
    assert!(s.rewind().is_none());
    assert!(s.view(&c).panel.is_some());
}

#[test]
fn the_battle_notes_show_at_the_start_with_their_units_and_close() {
    let mut c = ctx();
    let lead = LeadProfile::new(DEFAULT_NAME, LeadGender::Male);
    let campaign = trpg_content::new_campaign(&c.game.content, GameMode::Classic, lead);
    let def = &c.game.content.battles["test"];
    let (state, events) = BattleState::new(campaign.battle_setup(def, &c.game.content.tables()));
    let mut s = BattleScreen::start(state, &events);
    let view = s.view(&c);
    let Some(NotesView { title, lines }) = view.notes else {
        panic!("no notes");
    };
    assert_eq!(title, "BATTLE NOTES");
    assert_ne!(lines, [] as [std::string::String; 0]);
    // The brigand the first note is about, where it stands.
    let brigand = s.state().unit(UnitId(3)).unwrap().pos;
    assert!(view.noted.contains(&brigand), "{:?}", view.noted);
    step(&mut s, &mut c, &[Action::Confirm]);
    let view = s.view(&c);
    assert_eq!(view.notes, None);
    assert_eq!(view.noted, [] as [trpg_core::Pos; 0]);
}

#[test]
fn without_the_stage_the_fight_box_plays_then_the_exp_bar_and_both_go() {
    let mut c = ctx();
    let mut s = BattleScreen::new(super::testing::skirmish(&c, 20));
    let parts = |s: &BattleScreen, c: &Ctx| {
        let v = s.view(c);
        (v.fight_box, v.progress, v.stage.is_some())
    };
    // The lord a step right, `Attack`, the forecast on the raider: attack.
    step(&mut s, &mut c, &[Action::Confirm, Action::CursorRight]);
    step(&mut s, &mut c, &[Action::Confirm]);
    frame(&mut s, &mut c, &[], 0.5, false);
    step(&mut s, &mut c, &[Action::Confirm]);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Combat(_)), "{:?}", s.mode());
    // On the stage, no box; with combat animations off (turned off
    // mid-fight here, as the playback isn't skipped then), the box.
    let (fight_box, _, stage) = parts(&s, &c);
    assert_eq!((fight_box, stage), (None, true));
    c.change_settings(|set| set.game.combat_animations = false)
        .unwrap();
    s.show_fights = false;
    let (fight_box, progress, stage) = parts(&s, &c);
    let names = fight_box.map(|b| b.sides.map(|f| f.name));
    assert_eq!(names, Some(["Test Lord".to_owned(), "Raider".to_owned()]));
    assert_eq!((progress, stage), (None, false));
    // The playback over: the lord's EXP bar instead.
    while parts(&s, &c).0.is_some() {
        frame(&mut s, &mut c, &[], 0.1, false);
    }
    let (_, progress, _) = parts(&s, &c);
    let Some(super::view::ProgressView::Exp(bar)) = progress else {
        panic!("the EXP bar: {progress:?}")
    };
    assert_eq!(bar.name, "Test Lord");
    // Then it closes by itself.
    frame(&mut s, &mut c, &[], 2.0, false);
    assert_eq!(parts(&s, &c), (None, None, false));
}
