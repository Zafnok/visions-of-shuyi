//! Tests of `@place`, the place behind the talk (0732).

use trpg_core::{BattleMap, Grid, Pos, TerrainId};

use super::*;

fn map_at(map: &str, x: i32, y: i32) -> Step {
    Step::Directive(Stage::Setting(PlaceLine::Map {
        map: map.into(),
        at: Pos::new(x, y),
    }))
}

fn battle_at(x: i32, y: i32) -> Step {
    Step::Directive(Stage::Setting(PlaceLine::Battle { at: Pos::new(x, y) }))
}

#[test]
fn place_lines_parse_anywhere_in_a_scene() {
    let src = "\
@scene s
@place yard 3,12
> One.
@place  battle  0,0
@choice
* wry: Hm.
  @place none
  > Two.
* blunt: No.
  @place none
@endchoice
@if test_knight
@place hall 10,4
@else
@place hall 10,4
@endif
> Three.
@end
";
    let (parsed, errors) = parse_dlg("t.dlg", src);
    assert_eq!(errors, []);
    let steps = bare(&parsed[0].scene.steps);
    assert_eq!(
        steps[..3],
        [map_at("yard", 3, 12), narrate("One."), battle_at(0, 0)]
    );
    let Step::Choice { options } = &steps[3] else {
        panic!("no choice: {steps:?}");
    };
    assert_eq!(
        options[0].steps[0],
        Step::Directive(Stage::Setting(PlaceLine::None))
    );
    let Step::If { then, .. } = &steps[4] else {
        panic!("no @if: {steps:?}");
    };
    assert_eq!(then[..], [map_at("hall", 10, 4)]);
    assert_eq!(parsed[0].lines.steps, [2, 3, 4, 5, 12, 17]);
    // The places' maps aren't known here: nothing else is wrong.
    assert_eq!(errors_with_audio(src), [] as [&str; 0]);
}

#[test]
fn place_needs_a_map_and_a_tile_or_none() {
    let needs = "@place needs a map id and a tile (\"@place ch01 3,12\"), \"battle\" and a tile, \
                 or \"none\"";
    let tile = |t: &str| format!("\"{t}\" is not a tile; write it as <x>,<y>, e.g. 3,12");
    assert_eq!(
        errors(&scene(
            "@place\n@place yard\n@place none now\n@place none 1,2\n@place yard 3,12 4,5\n\
             @place yard 3\n@place yard 3,\n@place battle -1,2\n@place yard 1, 2\n\
             @place yard a,b\n@place Yard 1,2\n> x"
        )),
        [
            format!("t.dlg:4: {needs}"),
            format!("t.dlg:5: {needs}"),
            format!("t.dlg:6: {needs}"),
            format!("t.dlg:7: {needs}"),
            format!("t.dlg:8: {needs}"),
            format!("t.dlg:9: {}", tile("3")),
            format!("t.dlg:10: {}", tile("3,")),
            format!("t.dlg:11: {}", tile("-1,2")),
            format!("t.dlg:12: {needs}"),
            format!("t.dlg:13: {}", tile("a,b")),
            "t.dlg:14: \"Yard\" is not a valid id; use lowercase letters, digits and _".to_owned(),
        ]
    );
}

#[test]
fn place_line_arguments_print_as_written() {
    let scene = Scene {
        id: "s".into(),
        steps: vec![
            map_at("ch01", 3, 12),
            narrate("x"),
            battle_at(20, 3),
            Step::Directive(Stage::Setting(PlaceLine::None)),
        ],
    };
    let printed = print_scene(&scene);
    assert_eq!(
        printed,
        "@scene s\n@place ch01 3,12\n> x\n@place battle 20,3\n@place none\n@end\n"
    );
    assert_eq!(scenes(&printed), [scene]);
    assert_eq!((PlaceLine::BATTLE, PlaceLine::NONE), ("battle", "none"));
    assert_eq!(map_at("x", 0, 0).text(), None);
    assert!(!battle_at(0, 0).has_text());
}

/// A reply's reaction or an `@if` block may change the place, as long as
/// every way through leaves the same one (as with the caption).
#[test]
fn every_way_through_a_block_leaves_the_same_place() {
    let src = scene(
        "\
> x
@choice
* wry: Hm.
  @place yard 1,1
* blunt: No.
  @place yard 1,2
@endchoice
@if test_knight
@place yard 1,1
@else
@place none
@endif
@choice
* wry: Hm.
  @place none
* blunt: No.
  @place none
@endchoice",
    );
    assert_eq!(
        errors(&src),
        [
            "t.dlg:8: this reaction leaves a different @place from the first option's; every \
             option must leave the same place",
            "t.dlg:13: this @if block leaves a different @place with \"test_knight\" here and \
             without; it must leave the same place either way",
        ]
    );
}

// --- Maps and battles -------------------------------------------------------

/// A `w × h` map of plain.
fn field(w: u16, h: u16) -> BattleMap {
    BattleMap::new("Field", Grid::filled(w, h, TerrainId(0)))
}

/// The `@place` errors of `src` (file `t.dlg`): the map `yard` is 6 × 4;
/// scene `fight` is played for battle `b1` (5 × 5) and `b2` (8 × 3),
/// `talk` for `b1` and as a support conversation.
fn place_errors(src: &str, battles_known: bool) -> Vec<String> {
    let (yard, b1, b2) = (field(6, 4), field(5, 5), field(8, 3));
    let battles = BTreeMap::from([
        ("fight".to_owned(), vec![("b1", &b1), ("b2", &b2)]),
        ("talk".to_owned(), vec![("b1", &b1)]),
    ]);
    let refs = PlaceRefs {
        maps: BTreeMap::from([("yard", &yard)]),
        battles: battles_known.then_some(battles),
        elsewhere: ["talk".to_owned()].into(),
        openings: None,
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
fn a_place_is_a_tile_of_a_map() {
    let src = "\
@scene fight
@place yard 5,3
@place yard 6,3
@place yard 0,4
@place hall 0,0
@place none
@choice
* wry: Hm.
  @place cellar 1,1
* blunt: No.
@endchoice
> x
@end
";
    let size = "6 by 4 tiles, from 0,0 to 5,3";
    let none = "a place is a map of assets/maps/ (its file name without .map)";
    let expected = [
        format!("t.dlg:3: tile 6,3 is off map \"yard\" ({size})"),
        format!("t.dlg:4: tile 0,4 is off map \"yard\" ({size})"),
        format!("t.dlg:5: no map \"hall\"; {none}"),
        format!("t.dlg:9: no map \"cellar\"; {none}"),
    ];
    assert_eq!(place_errors(src, true), expected);
    assert_eq!(place_errors(src, false), expected);
}

#[test]
fn the_battle_is_only_for_a_battles_scenes() {
    let src = "\
@scene fight
@place battle 4,2
@place battle 4,4
@place battle 7,2
> x
@end
@scene talk
@place battle 0,0
> x
@end
@scene alone
> x
@if test_knight
@place battle 0,0
@endif
@end
";
    let only = "\"@place battle\" is only for a battle's scenes (a chapter's intro and victory \
                scenes, a battle trigger's scene)";
    assert_eq!(
        place_errors(src, true),
        [
            "t.dlg:3: tile 4,4 is off the map of battle \"b2\" (8 by 3 tiles, from 0,0 to 7,2)"
                .to_owned(),
            "t.dlg:4: tile 7,2 is off the map of battle \"b1\" (5 by 5 tiles, from 0,0 to 4,4)"
                .to_owned(),
            format!(
                "t.dlg:8: {only}; \"talk\" is also played at camp, away from a battle"
            ),
            format!("t.dlg:14: {only}; no battle plays \"alone\""),
        ]
    );
    // Without the battles (they failed to load), the battle isn't checked.
    assert_eq!(place_errors(src, false), [] as [&str; 0]);
}

/// The places example in `assets/dialogue/README.md` (its last) is valid.
#[test]
fn readme_places_example_is_valid() {
    let readme = bundle::file("dialogue/README.md").unwrap_or_default();
    let example = readme.rsplit("```").nth(1).unwrap_or_default();
    assert_eq!(errors_with_audio(example), [] as [&str; 0]);
    let (parsed, _) = parse_dlg("README.md", example);
    assert_eq!(parsed[0].scene.id, "ch01_after");
    assert_eq!(parsed[0].scene.steps[1], battle_at(20, 3));
}
