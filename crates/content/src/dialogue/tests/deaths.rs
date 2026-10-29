//! Tests of `@dies`, a scene's death (0319).

use super::*;
use crate::dialogue::{deaths, joined, joins};

fn dies(id: &str) -> Step {
    Step::Directive(Stage::Dies(CharacterId(id.into())))
}

#[test]
fn a_death_line_parses_and_prints_back() {
    let src = "@scene s\n> x\n@dies test_knight\n@end\n";
    let scene = Scene {
        id: "s".into(),
        steps: vec![narrate("x"), dies("test_knight")],
    };
    assert_eq!(scenes(src), std::slice::from_ref(&scene));
    assert_eq!(print_scene(&scene), src);
    assert!(!dies("test_knight").has_text());
    assert_eq!(errors(src), [] as [&str; 0]);
}

#[test]
fn a_death_names_one_character_who_isnt_the_lead() {
    let needs = "@dies needs one character id, e.g. \"@dies hollis\"";
    assert_eq!(
        errors(&scene(
            "> x\n@dies\n@dies a b\n@dies Knight\n@dies lead\n@dies nobody"
        )),
        [
            format!("t.dlg:5: {needs}"),
            format!("t.dlg:6: {needs}"),
            "t.dlg:7: \"Knight\" is not a valid id; use lowercase letters, digits and _".to_owned(),
            "t.dlg:8: the lead never dies in a scene".to_owned(),
            "t.dlg:9: @dies: no character \"nobody\"".to_owned(),
        ]
    );
}

#[test]
fn a_death_never_hangs_on_a_reply() {
    let src = scene("> x\n@choice\n* wry: Hm.\n  @dies test_knight\n* blunt: No.\n@endchoice");
    assert_eq!(
        errors(&src),
        [
            "t.dlg:7: a @dies goes at a scene's top level or in an @if block, not in a reply's \
          reaction"
        ]
    );
    // In an @if block it is fine, and checked there too.
    let src = scene("> x\n@if test_knight\n@dies test_knight\n@endif");
    assert_eq!(errors(&src), [] as [&str; 0]);
    let src = scene("> x\n@if test_knight\n@dies nobody\n@else\n@dies lead\n@endif");
    assert_eq!(
        errors(&src),
        [
            "t.dlg:6: @dies: no character \"nobody\"",
            "t.dlg:8: the lead never dies in a scene",
        ]
    );
}

/// The deaths of a scene as it plays for those present: only the `@if`
/// part that holds.
#[test]
fn the_deaths_are_those_of_the_scene_as_it_plays() {
    let src = "@scene s\n> x\n@if test_knight\n@dies test_knight\n@else\n> y\n@endif\n@end\n";
    let scene = &scenes(src)[0];
    let knight = CharacterId("test_knight".into());
    let with = Present::only([knight.clone()]);
    assert_eq!(deaths(&scene.resolved(&with)), [knight]);
    let without = Present::only([]);
    assert_eq!(deaths(&scene.resolved(&without)), []);
}

// `@joins`, a scene's recruit (0848): the same rules.

fn joins_step(id: &str) -> Step {
    Step::Directive(Stage::Joins(CharacterId(id.into())))
}

#[test]
fn a_join_line_parses_and_prints_back() {
    let src = "@scene s\n> x\n@joins test_knight\n@end\n";
    let scene = Scene {
        id: "s".into(),
        steps: vec![narrate("x"), joins_step("test_knight")],
    };
    assert_eq!(scenes(src), std::slice::from_ref(&scene));
    assert_eq!(print_scene(&scene), src);
    assert_eq!(errors(src), [] as [&str; 0]);
}

#[test]
fn a_join_names_one_character_who_isnt_the_lead_nor_in_a_reply() {
    let needs = "@joins needs one character id, e.g. \"@joins hollis\"";
    assert_eq!(
        errors(&scene("> x\n@joins\n@joins lead\n@joins nobody")),
        [
            format!("t.dlg:5: {needs}"),
            "t.dlg:6: the lead never joins in a scene".to_owned(),
            "t.dlg:7: @joins: no character \"nobody\"".to_owned(),
        ]
    );
    let src = scene("> x\n@choice\n* wry: Hm.\n  @joins test_knight\n* blunt: No.\n@endchoice");
    assert_eq!(
        errors(&src),
        [
            "t.dlg:7: a @joins goes at a scene's top level or in an @if block, not in a reply's \
          reaction"
        ]
    );
}

/// The recruits of a scene as it plays for those present, and every one
/// it may make (`joined`, for the presence check).
#[test]
fn the_joins_are_those_of_the_scene_as_it_plays() {
    let src = "@scene s\n> x\n@if test_mage\n@joins test_knight\n@else\n> y\n@endif\n@end\n";
    let parsed = &crate::dialogue::scripts_from_sources(&[("t.dlg", src)], None, None, None, None)
        .unwrap()
        .parsed[0];
    let knight = CharacterId("test_knight".into());
    let with = Present::only([CharacterId("test_mage".into())]);
    assert_eq!(
        joins(&parsed.scene.resolved(&with)),
        std::slice::from_ref(&knight)
    );
    assert_eq!(joins(&parsed.scene.resolved(&Present::only([]))), []);
    assert_eq!(joined(parsed), [knight]);
}
