//! Tests of line ids (ADR-0045 §3, ticket 0717) that the engine's tests
//! don't hold yet: how stable an id stays as a script changes. The format's
//! own cases (parsing, errors, printing, ids, `Scene::resolved`) are the
//! engine's tests (`shuyi_assets::dialogue`, ticket 1231).

use shuyi_assets::dialogue::line_hash;

use super::*;

/// `(id, speaker, text)` of every line of the one scene in `src`.
fn lines(src: &str) -> Vec<(String, String, String)> {
    let (parsed, errors) = parse_dlg("t.dlg", src);
    assert_eq!(errors, []);
    let lines = parsed[0].scene.lines();
    lines
        .iter()
        .map(|l| (l.id.to_string(), l.speaker.to_owned(), l.text.to_owned()))
        .collect()
}

/// The line ids of the one scene in `src`.
fn ids(src: &str) -> Vec<String> {
    lines(src).into_iter().map(|(id, ..)| id).collect()
}

/// The id is of the scene, the speaker and the text as written: tokens
/// unexpanded, continuation lines joined with one space; the expression
/// and a reply's tone are not part of it.
#[test]
fn the_id_comes_from_the_scene_the_speaker_and_the_text_as_written() {
    let id = |src: &str| ids(src).remove(0);
    let base = id("@scene s\ntest_lord: One {lead} two.\n@end\n");
    assert_eq!(
        base,
        format!("s_{}", line_hash("test_lord", "One {lead} two."))
    );
    assert_eq!(
        id("@scene s\ntest_lord[happy]: One {lead}\n    two.\n@end\n"),
        base
    );
    assert_ne!(id("@scene t\ntest_lord: One {lead} two.\n@end\n"), base);
    assert_ne!(id("@scene s\ntest_knight: One {lead} two.\n@end\n"), base);
    assert_ne!(id("@scene s\n> One {lead} two.\n@end\n"), base);
    assert_eq!(
        ids("@scene s\n@choice\n* wry: Hm.\n* blunt: No.\n@endchoice\n@end\n"),
        ids("@scene s\n@choice\n* earnest: Hm.\n* wry: No.\n@endchoice\n@end\n")
    );
}

/// A scene with no `@end` (an error) still gets its ids, continuation
/// lines included.
#[test]
fn scenes_without_an_end_get_ids_too() {
    let (parsed, _) = parse_dlg("t.dlg", "@scene a\n> One\n  two\n@scene b\n> Three\n  four");
    let ids: Vec<Vec<String>> = parsed
        .iter()
        .map(|p| p.scene.lines().iter().map(|l| l.id.to_string()).collect())
        .collect();
    assert_eq!(
        ids,
        [
            [format!("a_{}", line_hash(">", "One two"))],
            [format!("b_{}", line_hash(">", "Three four"))]
        ]
    );
}

#[test]
fn inserting_or_moving_a_line_leaves_the_other_ids_unchanged() {
    let before = ids(&scene("test_lord: One.\ntest_knight: Two.\n> Three."));
    let inserted = ids(&scene(
        "> New.\ntest_lord: One.\ntest_knight: Two.\n> Three.",
    ));
    assert_eq!(inserted[1..], before[..]);
    let moved = ids(&scene("> Three.\ntest_lord: One.\ntest_knight: Two."));
    assert_eq!(
        moved.iter().collect::<Vec<_>>(),
        [&before[2], &before[0], &before[1]]
    );
}

#[test]
fn changing_one_word_changes_that_lines_id_only() {
    let before = ids(&scene("test_lord: One.\ntest_knight: Two words.\n> Three."));
    let after = ids(&scene("test_lord: One.\ntest_knight: Two birds.\n> Three."));
    assert_eq!(after[0], before[0]);
    assert_ne!(after[1], before[1]);
    assert_eq!(after[2], before[2]);
}

fn arb_id() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,8}"
}

/// Printable ASCII, no spaces at either end, no `(…):` that would read
/// as a direction.
fn arb_text() -> impl Strategy<Value = String> {
    "[!-~]([ -~]{0,40}[!-~])?"
        .prop_filter("a direction", |t| !(t.starts_with('(') && t.contains("):")))
}

/// A speech or narration line.
fn arb_line() -> impl Strategy<Value = Step> {
    prop_oneof![
        (arb_id(), arb_text()).prop_map(|(s, text)| say(&s, None, &text)),
        arb_text().prop_map(|text| narrate(&text)),
    ]
}

/// Lines, choices whose reactions hold lines, and `@if` blocks of lines.
fn arb_step() -> impl Strategy<Value = Step> {
    let lines = || proptest::collection::vec(arb_line(), 0..3);
    let option = (arb_id(), arb_text(), lines()).prop_map(|(tone, text, steps)| ChoiceOption {
        tone,
        text,
        line: LineId::default(),
        steps,
    });
    prop_oneof![
        3 => arb_line(),
        1 => proptest::collection::vec(option, 0..4).prop_map(|options| Step::Choice { options }),
        1 => (arb_id(), lines(), lines()).prop_map(|(c, then, otherwise)| Step::If {
            character: CharacterId(c),
            then,
            otherwise,
        }),
    ]
}

fn arb_scene() -> impl Strategy<Value = Scene> {
    (arb_id(), proptest::collection::vec(arb_step(), 0..12)).prop_map(|(id, steps)| {
        let mut scene = Scene { id, steps };
        scene.assign_line_ids();
        scene
    })
}

proptest! {
    /// Inserting a new line anywhere at the top of a scene never changes
    /// another line's id.
    #[test]
    fn inserting_a_line_never_changes_another_id(
        scene in arb_scene(),
        text in arb_text(),
        at in any::<proptest::sample::Index>(),
    ) {
        let before: Vec<String> = scene.lines().iter().map(|l| l.id.to_string()).collect();
        let repeat = |l: &Line| l.speaker == NARRATION_SPEAKER && l.text == text;
        prop_assume!(!scene.lines().iter().any(repeat));
        let mut after = scene.clone();
        after.steps.insert(at.index(scene.steps.len() + 1), narrate(&text));
        after.assign_line_ids();
        let new = format!("{}_{}", scene.id, line_hash(NARRATION_SPEAKER, &text));
        let rest: Vec<String> = after
            .lines()
            .iter()
            .map(|l| l.id.to_string())
            .filter(|id| *id != new)
            .collect();
        prop_assert_eq!(rest, before);
    }
}
