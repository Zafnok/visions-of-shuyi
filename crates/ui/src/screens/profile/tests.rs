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
