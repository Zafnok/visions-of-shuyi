//! Tests of `@dies`, a scene's death (0319).

use super::*;
use crate::dialogue::deaths;

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
