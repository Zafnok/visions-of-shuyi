//! Tests of `@opening`, a conversation's moving opening (0734).

use alloc::collections::BTreeSet;

use trpg_core::{BattleMap, Grid, Pos, TerrainId};

use super::*;

fn opening(id: &str) -> Step {
    Step::Directive(Stage::Opening(id.into()))
}

#[test]
fn an_opening_line_parses_and_prints_back() {
    let src = "@scene s\n@place yard 1,1\n@opening spar\n> x\n@end\n";
    let place = Step::Directive(Stage::Setting(PlaceLine::Map {
        map: "yard".into(),
        at: Pos::new(1, 1),
    }));
    let scene = Scene {
        id: "s".into(),
        steps: vec![place, opening("spar"), narrate("x")],
    };
    assert_eq!(scenes(src), std::slice::from_ref(&scene));
    assert_eq!(print_scene(&scene), src);
    assert_eq!(opening("spar").text(), None);
    assert!(!opening("spar").has_text());
    assert_eq!(errors_with_audio(src), [] as [&str; 0]);
}

#[test]
fn an_opening_needs_one_id() {
    let needs = "@opening needs one opening id (a file of assets/openings/, without .ron)";
    assert_eq!(
        errors(&scene(
            "@place yard 1,1\n@opening\n> x\n@opening a b\n> x\n@opening Spar\n> x"
        )),
        [
            format!("t.dlg:5: {needs}"),
            format!("t.dlg:7: {needs}"),
            "t.dlg:9: \"Spar\" is not a valid id; use lowercase letters, digits and _".to_owned(),
        ]
    );
}

/// The place a top-level block leaves (its first part's) is the one an
/// opening after it plays on.
#[test]
fn an_opening_plays_on_the_place_a_block_leaves() {
    let src =
        scene("@if test_knight\n@place yard 1,1\n@else\n@place yard 1,1\n@endif\n@opening a\n> x");
    assert_eq!(errors(&src), [] as [&str; 0]);
    let src = scene(
        "@choice\n* wry: Hm.\n  @place yard 1,1\n* blunt: No.\n  @place yard 1,1\n@endchoice\n@opening a\n> x",
    );
    assert_eq!(errors(&src), [] as [&str; 0]);
}

/// [`check_scene`] runs the engine's checks and the game's: an unknown
/// character, and an opening in a block.
#[test]
fn check_scene_runs_the_engines_checks_and_the_games() {
    let src = scene("nobody: Hi.\n@if test_knight\n@opening a\n@endif");
    let (parsed, errors) = parse_dlg("t.dlg", &src);
    assert_eq!(errors, []);
    let characters = crate::character::load(None, None, None).unwrap_or_default();
    let errors: Vec<String> = check_scene(&parsed[0], Some(&characters), None, None, None)
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        errors,
        [
            "t.dlg:4: unknown character \"nobody\"".to_owned(),
            "t.dlg:6: an @opening goes at a scene's top level, not in a reply's reaction or an \
             @if block"
                .to_owned(),
        ]
    );
}

/// An opening goes at the top level, on a place from `assets/maps/`, and
/// something is said after it before the next one.
#[test]
fn an_opening_plays_on_a_map_before_a_line() {
    let src = scene(
        "\
@opening a
> x
@place battle 1,1
@opening b
> x
@place yard 1,1
@opening c
@opening d
> x
@choice
* wry: Hm.
  @opening e
* blunt: No.
@endchoice
@if test_knight
@opening f
@endif
@place none
> x
@place yard 2,2
@opening g",
    );
    let on_a_map = "an @opening plays on a place from assets/maps/: put a \"@place <map> \
                    <x>,<y>\" line before it (not \"@place battle\" or \"@place none\")";
    let top = "an @opening goes at a scene's top level, not in a reply's reaction or an @if block";
    assert_eq!(
        errors(&src),
        [
            format!("t.dlg:4: {on_a_map}"),
            format!("t.dlg:7: {on_a_map}"),
            "t.dlg:11: nothing is said between the @opening on line 10 and this one; an \
             opening plays before the next speech or narration line"
                .to_owned(),
            format!("t.dlg:15: {top}"),
            format!("t.dlg:19: {top}"),
            "t.dlg:24: nothing is said after this @opening; it plays before the next speech \
             or narration line"
                .to_owned(),
        ]
    );
}

fn field(w: u16, h: u16) -> BattleMap {
    BattleMap::new("Field", Grid::filled(w, h, TerrainId(0)))
}

/// The `@place` and `@opening` errors of `src` (file `t.dlg`), with the
/// maps `yard` and `hall` and, when `known`, the openings `spar` (on the
/// yard) and `pace` (in the hall).
fn opening_errors(src: &str, known: bool) -> Vec<String> {
    let (yard, hall) = (field(6, 4), field(5, 5));
    let openings = BTreeMap::from([("spar", "yard"), ("pace", "hall")]);
    let refs = PlaceRefs {
        maps: BTreeMap::from([("yard", &yard), ("hall", &hall)]),
        battles: None,
        elsewhere: BTreeSet::new(),
        openings: known.then_some(openings),
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
fn an_opening_is_a_file_that_plays_on_the_place_in_force() {
    let src = "\
@scene s
@place yard 1,1
@opening spar
> x
@place hall 1,1
@opening spar
> x
@choice
* wry: Hm.
  @place yard 2,2
* blunt: No.
  @place yard 2,2
@endchoice
@opening pace
> x
@if test_knight
@place hall 0,0
@endif
@opening pace
@opening dance
> x
@end
";
    assert_eq!(
        opening_errors(src, true),
        [
            "t.dlg:6: opening \"spar\" plays on map \"yard\", but the place here is \"hall\"",
            "t.dlg:14: opening \"pace\" plays on map \"hall\", but the place here is \"yard\"",
            "t.dlg:20: no opening \"dance\"; an opening is a file of assets/openings/ (its file \
             name without .ron)",
        ]
    );
    // Without the openings (they failed to load), they aren't checked.
    assert_eq!(opening_errors(src, false), [] as [&str; 0]);
}

/// Chapter 1's two openings are where the script asks for them, on their
/// places.
#[test]
fn chapter_1_opens_its_two_scenes_away_from_the_battle() {
    let content = crate::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let openings = |scene: &str| -> Vec<Step> {
        let steps = &content.dialogue.get(scene).map(|s| s.steps.clone());
        let steps = steps.iter().flatten();
        steps
            .filter(|s| matches!(s, Step::Directive(Stage::Opening(_))))
            .cloned()
            .collect()
    };
    assert_eq!(openings("ch01_intro"), [opening("ch01_sparring")]);
    assert_eq!(openings("ch01_tbc"), [opening("ch01_pacing")]);
    for scene in ["ch01_prebattle", "ch01_victory"] {
        assert_eq!(openings(scene), [] as [Step; 0]);
    }
}
