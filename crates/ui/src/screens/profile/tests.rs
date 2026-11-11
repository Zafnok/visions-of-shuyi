use crate::input::RIGHT_HANDED;
use insta::assert_snapshot;

use crate::harness::Harness;

fn ok<T, E: core::fmt::Display>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|e| panic!("{e}"))
}

/// The screen with its profiles.
#[test]
fn profile_screen_snapshot() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    ok(h.ctx_mut().add_profile("Mei"));
    ok(h.ctx_mut().add_profile("Odo"));
    h.keys("Down Down Down f Down");
    assert_eq!(h.top_screen(), "profiles");
    assert_snapshot!(h.snapshot());
}

/// The screen with only the first profile.
#[test]
fn one_profile_snapshot() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.keys("Down Down Down f");
    assert_eq!(h.top_screen(), "profiles");
    assert_snapshot!(h.snapshot());
}

/// Deleting Mei: the question over the list, focused on No.
#[test]
fn delete_question_snapshot() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    ok(h.ctx_mut().add_profile("Mei"));
    h.keys("Down Down Down f Down f Down Down f");
    assert_snapshot!(h.snapshot());
}

/// A later launch on a desktop whose saves are in `/saves`, with Mei
/// added, on the Profile screen (0872).
fn on_a_desktop() -> Harness {
    let mut h = Harness::with_layout_and(RIGHT_HANDED, |ctx| {
        ctx.platform.storage_place = shuyi_ui::StoragePlace::Folder("/saves".into());
    });
    ok(h.ctx_mut().add_profile("Mei"));
    h.keys("Down Down Down f");
    assert_eq!(h.top_screen(), "profiles");
    h
}

fn profile_screen(h: &Harness) -> &super::ProfileScreen {
    let screen = h.game().screen::<super::ProfileScreen>();
    screen.unwrap_or_else(|| panic!("no Profile screen: {:?}", h.screens()))
}

/// The export key exports the profile under the cursor; the notice says
/// where it went, and moving puts it away.
#[test]
fn the_export_key_exports_the_profile_under_the_cursor() {
    let mut h = on_a_desktop();
    let view = profile_screen(&h).view(h.game().ctx());
    assert_eq!(view.place.map(|p| p.path), Some("/saves".to_owned()));
    let help = h
        .game()
        .ctx()
        .text_with("profile.help.list_export_folder", &[]);
    assert_eq!(view.list.help, help);
    h.keys("Down e");
    let names: Vec<&str> = h.offered().iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["visions-of-shuyi-mei-2.ron"]);
    h.wait(0.1);
    let message = profile_screen(&h).view(h.game().ctx()).list.message;
    assert!(message.is_some_and(|m| m.contains("visions-of-shuyi-mei-2.ron")));
    h.keys("Up");
    assert_eq!(profile_screen(&h).view(h.game().ctx()).list.message, None);
    // The row that adds a profile has nothing to export.
    h.keys("Down Down e");
    assert_eq!(h.offered().len(), 1);
}

/// With a menu or a question open, the export key does nothing.
#[test]
fn nothing_is_exported_over_a_menu_or_a_question() {
    let mut h = on_a_desktop();
    // The profile's menu.
    h.keys("f e");
    assert_eq!(h.offered().len(), 0);
    // The question before deleting Mei.
    h.keys("d Down f Down Down f");
    assert!(
        profile_screen(&h)
            .view(h.game().ctx())
            .list
            .deleting
            .is_some()
    );
    h.keys("e x");
    assert_eq!(h.offered().len(), 0);
}

/// The screen declares the export and open-folder keys besides the list's.
#[test]
fn the_screen_declares_its_file_keys() {
    let h = on_a_desktop();
    let actions = crate::screen::Screen::actions(profile_screen(&h));
    let actions = actions.unwrap_or_default();
    for action in [crate::input::Action::Info, crate::input::Action::WorldFind] {
        assert!(actions.contains(&action), "{actions:?}");
    }
    assert!(actions.contains(&crate::input::Action::Confirm));
}
