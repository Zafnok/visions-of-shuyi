//! Tests of `@ambience`, the bed a scene names for itself (ticket 1383).

use super::*;

fn ambience(id: Option<&str>) -> Step {
    Step::Directive(Stage::Ambience(
        id.map_or(AmbienceLine::None, |id| AmbienceLine::Bed(id.into())),
    ))
}

#[test]
fn ambience_lines_name_an_ambience_or_none() {
    let src = "@scene s\n@ambience  camp_quiet \n> One.\n@ambience none\n> Two.\n@end\n";
    let steps = &scenes(src)[0].steps;
    assert_eq!(
        steps[..],
        [
            ambience(Some("camp_quiet")),
            narrate("One."),
            ambience(None),
            narrate("Two."),
        ]
    );
}

#[test]
fn ambience_needs_one_id() {
    let needs = "@ambience needs one ambience id (one of data/ambience.ron's ambiences), or \
                 \"none\" for none";
    assert_eq!(
        errors(&scene("@ambience\n@ambience a b\n@ambience Quiet\n> x")),
        [
            format!("t.dlg:4: {needs}"),
            format!("t.dlg:5: {needs}"),
            "t.dlg:6: \"Quiet\" is not a valid id; use lowercase letters, digits and _".to_owned(),
        ]
    );
}

#[test]
fn ambience_lines_print_as_written() {
    let scene = Scene {
        id: "s".into(),
        steps: vec![ambience(Some("meadow")), narrate("x"), ambience(None)],
    };
    let printed = print_scene(&scene);
    assert_eq!(
        printed,
        "@scene s\n@ambience meadow\n> x\n@ambience none\n@end\n"
    );
    assert_eq!(scenes(&printed), [scene]);
}

/// The `@ambience` errors of `src` (file `t.dlg`), the ambiences known
/// being `meadow` and `hall` when `known`.
fn ambience_errors(src: &str, known: bool) -> Vec<String> {
    let ambiences = ["meadow".to_owned(), "hall".to_owned()].into();
    let refs = PlaceRefs {
        ambiences: known.then_some(ambiences),
        ..PlaceRefs::default()
    };
    let (parsed, errors) = parse_dlg("t.dlg", src);
    assert_eq!(errors, []);
    parsed
        .iter()
        .flat_map(|scene| check_places(scene, &refs))
        .map(|e| e.to_string())
        .collect()
}

#[test]
fn an_ambience_is_one_of_the_ambience_file() {
    let src = "\
@scene s
@ambience meadow
> x
@ambience marsh
> y
@choice
* wry: Hm.
  @ambience fen
* blunt: No.
  @ambience fen
@endchoice
@ambience none
> z
@end
";
    let no = |line: u32, id: &str| {
        format!(
            "t.dlg:{line}: no ambience \"{id}\"; an ambience is one of data/ambience.ron's \
             ambiences"
        )
    };
    assert_eq!(
        ambience_errors(src, true),
        [no(4, "marsh"), no(8, "fen"), no(10, "fen")]
    );
    // Without the file, nothing is checked.
    assert_eq!(ambience_errors(src, false), [] as [&str; 0]);
}

/// The embedded file's ambiences are what a scene may name, and the
/// whole load checks a scene's line against them.
#[test]
fn the_embedded_ambiences_are_what_a_scene_may_name() {
    let ids = crate::ambience::ids().expect("the ambience file reads");
    assert!(ids.contains("meadow"), "{ids:?}");
    let refs = PlaceRefs::default().with_ambiences(Some(ids));
    let (parsed, _) = parse_dlg("t.dlg", "@scene s\n@ambience meadow\n> x\n@end\n");
    assert_eq!(check_places(&parsed[0], &refs), []);
    let (parsed, _) = parse_dlg("t.dlg", "@scene s\n@ambience marsh\n> x\n@end\n");
    assert_eq!(check_places(&parsed[0], &refs).len(), 1, "marsh is none");
}
