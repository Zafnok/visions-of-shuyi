use insta::assert_snapshot;

use super::*;
use crate::harness::Harness;
use crate::input::Layout;
use crate::screen::tests::ctx;
use crate::settings::TextSpeed;

use Action::{Cancel, Confirm, CursorDown, CursorLeft, CursorUp};

fn press(s: &mut ProfileScreen, c: &mut Ctx, actions: &[Action]) -> String {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    format!("{:?}", s.update(c, &input))
}

/// `actions` from a controller.
fn pad(s: &mut ProfileScreen, c: &mut Ctx, actions: &[Action]) -> String {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]).with_pad(true);
    format!("{:?}", s.update(c, &input))
}

fn ok<T, E: core::fmt::Display>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|e| panic!("{e}"))
}

/// The test context with `Player 1` (in use), `Mei` and `Odo`.
fn three() -> Ctx {
    let mut c = ctx();
    ok(c.add_profile("Mei"));
    ok(c.add_profile("Odo"));
    c
}

fn names(c: &Ctx) -> Vec<String> {
    c.profiles().all().iter().map(|p| p.name.clone()).collect()
}

fn id_of(c: &Ctx, name: &str) -> ProfileId {
    let profile = c.profiles().all().iter().find(|p| p.name == name);
    profile.map_or_else(|| panic!("no {name}"), |p| p.id)
}

/// The labels of the open menu of what can be done to a profile.
fn actions(s: &ProfileScreen, c: &Ctx) -> Vec<String> {
    let view = s.view(c).actions.unwrap_or_else(|| panic!("no menu"));
    view.items.into_iter().map(|i| i.label).collect()
}

#[test]
fn the_view_lists_the_profiles_the_one_in_use_marked_and_focused_then_add() {
    let mut c = three();
    ok(c.switch_profile(id_of(&c, "Mei")));
    let s = ProfileScreen::new(&c);
    let view = s.view(&c);
    let labels: Vec<&str> = view.rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, ["Player 1", "Mei", "Odo", "Add a profile"]);
    assert_eq!(view.current().map(|r| r.label.as_str()), Some("Mei"));
    assert_eq!(view.focused().map(|r| r.label.as_str()), Some("Mei"));
    assert_eq!(view.current_mark, "in use");
    assert_eq!(view.rows.last().map(|r| r.kind), Some(RowKind::Add));
    assert!(view.rows.iter().all(|r| r.enabled));
}

#[test]
fn the_rows_wrap_and_cancel_goes_back() {
    let mut c = three();
    let mut s = ProfileScreen::new(&c);
    assert_eq!(s.help(&c), "arrows move · f choose · d back");
    press(&mut s, &mut c, &[CursorUp]);
    assert_eq!(s.focused(&c), RowKind::Add);
    assert_eq!(s.help(&c), "arrows move · f add · d back");
    press(&mut s, &mut c, &[CursorDown]);
    assert_eq!(s.focused(&c), RowKind::Profile(ProfileId::FIRST));
    assert_eq!(press(&mut s, &mut c, &[Cancel]), "Pop");
}

#[test]
fn a_profile_offers_switch_unless_in_use_and_delete_unless_the_only_one() {
    let mut c = ctx();
    let mut s = ProfileScreen::new(&c);
    press(&mut s, &mut c, &[Confirm]);
    assert_eq!(actions(&s, &c), ["Rename"]);
    let mut c = three();
    let mut s = ProfileScreen::new(&c);
    press(&mut s, &mut c, &[Confirm]);
    assert_eq!(actions(&s, &c), ["Rename", "Delete"]);
    assert_eq!(s.view(&c).actions.map(|a| a.title), Some("Player 1".into()));
    // Cancel closes the menu, not the screen.
    assert_eq!(press(&mut s, &mut c, &[Cancel]), "None");
    assert!(s.view(&c).actions.is_none());
    press(&mut s, &mut c, &[CursorDown, Confirm]);
    assert_eq!(actions(&s, &c), ["Use this profile", "Rename", "Delete"]);
    assert_eq!(s.help(&c), "arrows move · f select · d back");
}

#[test]
fn switching_uses_that_profiles_options_and_goes_back_to_the_title() {
    let mut c = three();
    let mei = id_of(&c, "Mei");
    ok(c.switch_profile(mei));
    ok(c.change_settings(|s| s.text_speed = TextSpeed::Slow));
    ok(c.switch_profile(ProfileId::FIRST));
    assert_ne!(c.settings().text_speed, TextSpeed::Slow);
    let mut s = ProfileScreen::new(&c);
    // Mei, the menu, `Use this profile`.
    assert_eq!(press(&mut s, &mut c, &[CursorDown, Confirm]), "None");
    assert_eq!(press(&mut s, &mut c, &[Confirm]), "Pop");
    assert_eq!(c.profiles().current(), mei);
    assert_eq!(c.settings().text_speed, TextSpeed::Slow);
}

#[test]
fn deleting_asks_first_focused_on_no() {
    let mut c = three();
    let mut s = ProfileScreen::new(&c);
    // Mei, the menu, down past Rename to Delete.
    press(&mut s, &mut c, &[CursorDown, Confirm]);
    press(&mut s, &mut c, &[CursorDown, CursorDown, Confirm]);
    let question = s.view(&c).question.unwrap_or_else(|| panic!("no question"));
    assert_eq!(
        question.text,
        "Delete Mei? Its options and saves go with it."
    );
    assert_eq!(question.focused(), Some("No"));
    assert_eq!(s.help(&c), "arrows choose · f pick · d back");
    // Confirm at once is No: nothing is deleted.
    press(&mut s, &mut c, &[Confirm]);
    assert!(s.view(&c).question.is_none());
    assert_eq!(names(&c), ["Player 1", "Mei", "Odo"]);
    // Cancel is No too.
    press(&mut s, &mut c, &[Confirm]);
    press(&mut s, &mut c, &[CursorDown, CursorDown, Confirm]);
    assert_eq!(
        s.view(&c)
            .question
            .and_then(|q| q.focused().map(str::to_owned)),
        Some("No".into())
    );
    press(&mut s, &mut c, &[Cancel]);
    assert_eq!(names(&c), ["Player 1", "Mei", "Odo"]);
    // Left to Yes, Confirm: deleted.
    press(&mut s, &mut c, &[Confirm]);
    press(&mut s, &mut c, &[CursorDown, CursorDown, Confirm]);
    press(&mut s, &mut c, &[CursorLeft]);
    assert_eq!(
        s.view(&c)
            .question
            .and_then(|q| q.focused().map(str::to_owned)),
        Some("Yes".into())
    );
    assert_eq!(press(&mut s, &mut c, &[Confirm]), "None");
    assert_eq!(names(&c), ["Player 1", "Odo"]);
}

#[test]
fn deleting_the_profile_in_use_uses_the_first_one_left_and_keeps_the_focus_in_the_list() {
    let mut c = three();
    let odo = id_of(&c, "Odo");
    ok(c.switch_profile(odo));
    let mut s = ProfileScreen::new(&c);
    assert_eq!(s.focused(&c), RowKind::Profile(odo));
    // In use: Rename, Delete.
    press(&mut s, &mut c, &[Confirm]);
    press(&mut s, &mut c, &[CursorDown, Confirm]);
    press(&mut s, &mut c, &[CursorLeft]);
    press(&mut s, &mut c, &[Confirm]);
    assert_eq!(names(&c), ["Player 1", "Mei"]);
    assert_eq!(c.profiles().current(), ProfileId::FIRST);
    assert_eq!(s.focused(&c), RowKind::Profile(id_of(&c, "Mei")));
}

#[test]
fn adding_past_the_most_profiles_is_refused_with_a_message() {
    let mut c = ctx();
    for i in 2..=MAX_PROFILES {
        ok(c.add_profile(&format!("P{i}")));
    }
    let mut s = ProfileScreen::new(&c);
    press(&mut s, &mut c, &[CursorUp]);
    let view = s.view(&c);
    assert_eq!(view.focused().map(|r| r.enabled), Some(false));
    assert_eq!(s.help(&c), "arrows move · d back");
    press(&mut s, &mut c, &[Confirm]);
    assert!(s.typed().is_none());
    assert_eq!(
        s.view(&c).message.as_deref(),
        Some("There can be at most 8 profiles.")
    );
    // The message goes with the next move.
    press(&mut s, &mut c, &[CursorUp]);
    assert_eq!(s.message(), None);
}

#[test]
fn on_a_controller_add_spells_the_name_on_the_grid_and_uses_the_new_profile() {
    let mut c = three();
    let mut s = ProfileScreen::new(&c);
    pad(&mut s, &mut c, &[CursorUp]);
    pad(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.naming(), Some(NamePurpose::Add));
    let grid = s.view(&c).grid.unwrap_or_else(|| panic!("no grid"));
    assert_eq!(grid.rows.len(), 6, "a row of digits");
    assert_eq!(s.help(&c), "arrows choose · f type · d delete");
    // "A", then down to the last row and round left to Done.
    pad(&mut s, &mut c, &[Confirm]);
    assert_eq!(s.typed(), Some("A"));
    let to_done = [CursorUp, CursorLeft];
    pad(&mut s, &mut c, &to_done);
    let transition = pad(&mut s, &mut c, &[Confirm]);
    assert_eq!(transition, "Replace(layout_picker)");
    assert_eq!(names(&c), ["Player 1", "Mei", "Odo", "A"]);
    assert_eq!(c.profiles().current(), id_of(&c, "A"));
    // The new profile starts from the default options, with no layout.
    assert_eq!(*c.settings(), crate::settings::Settings::default());
    assert_eq!(c.layout(), None);
}

#[test]
fn cancel_on_an_empty_grid_closes_it_without_asking() {
    let mut c = three();
    let mut s = ProfileScreen::new(&c);
    pad(&mut s, &mut c, &[CursorUp]);
    pad(&mut s, &mut c, &[Confirm]);
    pad(&mut s, &mut c, &[Cancel]);
    assert!(s.typed().is_none());
    assert!(s.view(&c).question.is_none());
    assert_eq!(names(&c), ["Player 1", "Mei", "Odo"]);
}

/// The screen with its profiles.
#[test]
fn profile_screen_snapshot() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    ok(h.ctx_mut().add_profile("Mei"));
    ok(h.ctx_mut().add_profile("Odo"));
    h.keys("Down Down Down f Down");
    assert_eq!(h.top_screen(), "profile");
    assert_snapshot!(h.snapshot());
}

/// The screen with only the first profile.
#[test]
fn one_profile_snapshot() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    h.keys("Down Down Down f");
    assert_eq!(h.top_screen(), "profile");
    assert_snapshot!(h.snapshot());
}

/// Deleting Mei: the question over the list, focused on No.
#[test]
fn delete_question_snapshot() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    ok(h.ctx_mut().add_profile("Mei"));
    h.keys("Down Down Down f Down f Down Down f");
    assert_snapshot!(h.snapshot());
}
