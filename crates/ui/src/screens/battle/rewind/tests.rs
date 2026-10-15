use trpg_core::Pos;

use super::*;
use crate::screen::tests::ctx;
use crate::screens::battle::testing::skirmish;

fn screen(lines: usize, focus: usize) -> RewindScreen {
    let before = skirmish(&ctx(), 20);
    let entries = (0..lines)
        .map(|i| Entry {
            line: format!("e{i}"),
            point: lines - 1 - i,
            before: before.clone(),
        })
        .collect();
    RewindScreen {
        entries,
        focus,
        confirming: false,
        charges_left: 3,
        charges: 3,
    }
}

#[test]
fn the_view_lists_the_entries_and_asks_before_a_rewind() {
    let mut r = screen(3, 0);
    let v = r.view(Words::ENGLISH);
    assert_eq!(v.title, "Rewind");
    assert_eq!(
        (v.charges.as_str(), v.spent),
        ("3 of 3 charges left", false)
    );
    assert_eq!(
        (v.entries.as_slice(), v.focus),
        (["e0", "e1", "e2"].map(String::from).as_slice(), 0)
    );
    assert_eq!((v.nothing, v.prompt), (None, None));
    r.step(Action::CursorDown);
    r.step(Action::Confirm);
    let v = r.view(Words::ENGLISH);
    assert_eq!(v.focus, 1);
    assert_eq!(v.prompt.as_deref(), Some("Use 1 of 3 charges?"));
    // One charge left; then none.
    r.charges_left = 1;
    let v = r.view(Words::ENGLISH);
    assert_eq!(v.prompt.as_deref(), Some("Use 1 of 1 charge?"));
    r.charges_left = 0;
    let v = r.view(Words::ENGLISH);
    assert_eq!((v.charges.as_str(), v.spent), ("No charges left", true));
    // Nothing to go back to.
    let empty = screen(0, 0).view(Words::ENGLISH);
    assert_eq!(empty.nothing.as_deref(), Some("Nothing to rewind yet"));
    assert!(empty.entries.is_empty());
}

#[test]
fn the_prompt_freezes_the_choice() {
    let mut r = screen(3, 0);
    assert_eq!(r.step(Action::CursorDown), RewindEffect::None);
    assert_eq!(r.step(Action::Confirm), RewindEffect::None);
    assert!(r.is_confirming());
    // Up and down do nothing while it's up.
    r.step(Action::CursorDown);
    assert_eq!(r.focused().map(|e| e.point), Some(1));
    r.step(Action::CursorUp);
    assert_eq!(r.focused().map(|e| e.point), Some(1));
    assert_eq!(r.step(Action::Confirm), RewindEffect::Rewind(1));
}

#[test]
fn using_an_item_names_the_target_unless_it_is_the_user() {
    let before = skirmish(&ctx(), 20);
    let used = |target| Replayed {
        before: before.clone(),
        command: Command::Act {
            unit: UnitId(1),
            dest: Pos::new(6, 2),
            action: UnitAction::UseItem {
                pack_index: 0,
                target: UnitId(target),
            },
        },
        events: vec![],
    };
    assert_eq!(
        describe(&used(1), Words::ENGLISH),
        "Turn 1 · Test Lord used an item"
    );
    assert_eq!(
        describe(&used(2), Words::ENGLISH),
        "Turn 1 · Test Lord used an item on Test Knight"
    );
}

#[test]
fn phase_names() {
    let ended = |phase| Words::ENGLISH.text(phase_ended(phase));
    assert_eq!(ended(Phase::Player), "Player phase ended");
    assert_eq!(ended(Phase::Enemy), "Enemy phase ended");
    assert_eq!(ended(Phase::Other), "Ally phase ended");
}
