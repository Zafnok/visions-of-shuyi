//! Tests of the opening files (ticket 0734), on the test buildings
//! (`assets-test/maps/test_buildings.map`, 10 × 6 tiles: grass on row 0,
//! a wall at (7, 2) and the hall's walls from (5, 3) down).

use trpg_core::{CharacterId, Pos, UnitId};

use super::*;
use crate::cinematic::opening::from_source;

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

fn refs(c: &Content) -> OpeningRefs<'_> {
    OpeningRefs {
        maps: &c.maps,
        characters: &c.characters,
        classes: &c.classes,
        terrain: &c.terrain.rules,
    }
}

/// An opening file `t` on `map` with `cast` and `action` as written.
fn opening(map: &str, cast: &str, action: &str) -> String {
    format!("(id: \"t\", map: \"{map}\", cast: [{cast}], action: [{action}])")
}

/// The messages of the errors of `source` as file `t`.
fn problems(c: &Content, source: &str) -> Vec<String> {
    match from_source("o.ron", "t", source, refs(c)) {
        Ok(def) => panic!("loaded: {def:?}"),
        Err(errors) => {
            assert!(errors.iter().all(|e| e.file == "o.ron"), "{errors:?}");
            errors.into_iter().map(|e| e.message).collect()
        }
    }
}

const LEAD_STEPS_IN: &str =
    "(at: 0.0, do: Walk(unit: \"lead\", path: [(1, 0), (2, 0)], secs: 0.2))";

#[test]
fn the_test_opening_loads() {
    let c = content();
    let test = &c.openings["test_opening"];
    assert_eq!(test.map, "test_buildings");
    let member = |unit, who: &str, at, facing| CastMember {
        unit: UnitId(unit),
        character: CharacterId(who.to_owned()),
        at,
        facing,
    };
    assert_eq!(
        test.cast,
        [
            member(1, "lead", p(1, 0), Heading::Right),
            member(2, "test_knight", p(3, 0), Heading::Left),
        ]
    );
    assert_eq!(test.actions.len(), 3);
    let back = ActionDef {
        at_ms: 300,
        act: Act::Walk {
            unit: UnitId(1),
            path: vec![p(2, 0), p(1, 0)],
            ms: 200,
            facing: Some(Heading::Right),
        },
    };
    assert_eq!(test.actions[1], back);
    // Until its last walk ends.
    assert_eq!(test.length_ms, 1000);
    let Act::Walk { path, facing, .. } = &test.actions[2].act else {
        panic!("not a walk: {:?}", test.actions[2]);
    };
    // Round the corner: a tile a step.
    assert_eq!((path.len(), *facing), (4, None));
}

/// Chapter 1's openings load on their places (a test of the content,
/// ADR-0118).
#[test]
fn chapter_1s_openings_are_on_its_places() {
    if bundle::skip_without_the_game() {
        return;
    }
    let c = content();
    let place = |id: &str| c.openings.get(id).map(|o| o.map.as_str());
    assert_eq!(place("ch01_sparring"), Some("harrowby_yard"));
    assert_eq!(place("ch01_pacing"), Some("veyne_study"));
}

#[test]
fn an_opening_names_itself_a_map_and_a_cast() {
    let c = content();
    let source = "(id: \"x\", map: \"nowhere\", cast: [], action: [])";
    assert_eq!(
        problems(&c, source),
        [
            "id \"x\" must match the file name \"t\"",
            "map: no map \"nowhere\"; an opening plays on a map of assets/maps/ (its file name \
             without .map)",
            "cast: an opening needs someone in it",
        ]
    );
}

#[test]
fn the_cast_are_characters_on_tiles_they_can_stand_on() {
    let c = content();
    let cast = [
        "(who: \"lead\", at: (1, 0))",
        "(who: \"nobody\", at: (2, 4))",
        "(who: \"lead\", at: (2, 0))",
        "(who: \"test_knight\", at: (30, 3))",
        "(who: \"test_voice\", at: (7, 2))",
        "(who: \"test_lord\", at: (1, 0))",
    ];
    let source = opening("test_buildings", &cast.join(", "), LEAD_STEPS_IN);
    assert_eq!(
        problems(&c, &source),
        [
            "cast 2: no character \"nobody\"",
            "cast 3: \"lead\" is in the cast twice",
            "cast 4: (30, 3) is not on map \"test_buildings\" (10×6 tiles)",
            "cast 5: a person on foot can't stand on (7, 2)",
            "cast 6: (1, 0) is where \"lead\" stands",
        ]
    );
}

#[test]
fn the_walks_are_checked_as_a_cinematic_action_on_the_place() {
    let c = content();
    let cast = "(who: \"test_voice\", at: (6, 1))";
    let action = [
        // Into the hall's walls: someone with no class walks on foot.
        "(at: 0.0, do: Walk(unit: \"test_voice\", path: [(6, 1), (6, 5)], secs: 0.5))",
        "(at: 1.0, do: Strike(unit: \"test_voice\", target: \"test_voice\", hp_left: 0))",
        "(at: 1.0, do: Walk(unit: \"nobody\", path: [(1, 1), (2, 1)], secs: 0.5))",
        "(at: 1.0, do: Walk(unit: \"test_voice\", path: [(6, 5), (6, 20)], secs: 0.5))",
    ];
    let source = opening("test_buildings", cast, &action.join(", "));
    assert_eq!(
        problems(&c, &source),
        [
            "action 1: path: a person on foot can't walk on (6, 5)",
            "action 2: an opening has walks only, no strikes",
            "action 3: no unit \"nobody\" in the opening's cast",
            "action 4: path: (6, 20) is not on the map (10×6 tiles)",
        ]
    );
    let none = opening("test_buildings", cast, "");
    assert_eq!(problems(&c, &none), ["an action needs at least one entry"]);
}

#[test]
fn a_walker_may_keep_a_facing() {
    let c = content();
    let cast = "(who: \"lead\", at: (1, 0), facing: Up)";
    let action =
        "(at: 0.5, do: Walk(unit: \"lead\", path: [(1, 0), (1, 1)], secs: 0.25, facing: Up))";
    let def = from_source(
        "o.ron",
        "t",
        &opening("test_buildings", cast, action),
        refs(&c),
    );
    let def = def.unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(def.cast[0].facing, Heading::Up);
    assert_eq!(def.length_ms, 750);
    let Act::Walk { facing, .. } = &def.actions[0].act else {
        panic!("not a walk: {:?}", def.actions[0]);
    };
    assert_eq!(*facing, Some(Heading::Up));
    // Someone left without a facing faces the camera.
    let plain = opening(
        "test_buildings",
        "(who: \"lead\", at: (1, 0))",
        LEAD_STEPS_IN,
    );
    let plain = from_source("o.ron", "t", &plain, refs(&c)).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(plain.cast[0].facing, Heading::Down);
}
