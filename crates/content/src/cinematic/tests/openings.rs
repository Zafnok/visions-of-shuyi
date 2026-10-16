//! Tests of the opening files (ticket 0734), on Chapter 1's places: the
//! yard at Harrowby (25 × 16 tiles, the house wall along rows 2 to 4) and
//! the study at Veyne Hall (the desk on rows 5 and 6, columns 9 to 15).

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
    "(at: 0.0, do: Walk(unit: \"lead\", path: [(11, 8), (12, 8)], secs: 0.2))";

#[test]
fn chapter_1_openings_load() {
    let c = content();
    let sparring = &c.openings["ch01_sparring"];
    assert_eq!(sparring.map, "harrowby_yard");
    let member = |unit, who: &str, at, facing| CastMember {
        unit: UnitId(unit),
        character: CharacterId(who.to_owned()),
        at,
        facing,
    };
    assert_eq!(
        sparring.cast,
        [
            member(1, "lead", p(11, 8), Heading::Right),
            member(2, "retainer", p(13, 8), Heading::Left),
        ]
    );
    assert_eq!(sparring.actions.len(), 6);
    let back = ActionDef {
        at_ms: 300,
        act: Act::Walk {
            unit: UnitId(1),
            path: vec![p(12, 8), p(11, 8)],
            ms: 200,
            facing: Some(Heading::Right),
        },
    };
    assert_eq!(sparring.actions[1], back);
    // About 1.5 s of movement (Nick: half the mockup's 3 s).
    assert_eq!(sparring.length_ms, 1550);
    let pacing = &c.openings["ch01_pacing"];
    assert_eq!(
        (pacing.map.as_str(), pacing.length_ms),
        ("veyne_study", 1450)
    );
    assert_eq!(pacing.cast, [member(1, "rival", p(9, 8), Heading::Right)]);
    let Act::Walk { path, facing, .. } = &pacing.actions[0].act else {
        panic!("not a walk: {:?}", pacing.actions[0]);
    };
    assert_eq!((path.len(), *facing), (7, None));
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
        "(who: \"lead\", at: (11, 8))",
        "(who: \"nobody\", at: (2, 9))",
        "(who: \"lead\", at: (12, 8))",
        "(who: \"rival\", at: (30, 3))",
        "(who: \"messenger\", at: (5, 2))",
        "(who: \"vowmaster\", at: (11, 8))",
    ];
    let source = opening("harrowby_yard", &cast.join(", "), LEAD_STEPS_IN);
    assert_eq!(
        problems(&c, &source),
        [
            "cast 2: no character \"nobody\"",
            "cast 3: \"lead\" is in the cast twice",
            "cast 4: (30, 3) is not on map \"harrowby_yard\" (25×16 tiles)",
            "cast 5: a person on foot can't stand on (5, 2)",
            "cast 6: (11, 8) is where \"lead\" stands",
        ]
    );
}

#[test]
fn the_walks_are_checked_as_a_cinematic_action_on_the_place() {
    let c = content();
    let cast = "(who: \"rival\", at: (9, 8))";
    let action = [
        // Into the desk: someone with no class walks on foot.
        "(at: 0.0, do: Walk(unit: \"rival\", path: [(9, 8), (9, 5)], secs: 0.5))",
        "(at: 1.0, do: Strike(unit: \"rival\", target: \"rival\", hp_left: 0))",
        "(at: 1.0, do: Walk(unit: \"hollis\", path: [(1, 1), (2, 1)], secs: 0.5))",
        "(at: 1.0, do: Walk(unit: \"rival\", path: [(9, 5), (9, 20)], secs: 0.5))",
    ];
    let source = opening("veyne_study", cast, &action.join(", "));
    assert_eq!(
        problems(&c, &source),
        [
            "action 1: path: a person on foot can't walk on (9, 6)",
            "action 1: path: a person on foot can't walk on (9, 5)",
            "action 2: an opening has walks only, no strikes",
            "action 3: no unit \"hollis\" in the opening's cast",
            "action 4: path: (9, 20) is not on the map (25×16 tiles)",
        ]
    );
    let none = opening("veyne_study", cast, "");
    assert_eq!(problems(&c, &none), ["an action needs at least one entry"]);
}

#[test]
fn a_walker_may_keep_a_facing() {
    let c = content();
    let cast = "(who: \"lead\", at: (11, 8), facing: Up)";
    let action =
        "(at: 0.5, do: Walk(unit: \"lead\", path: [(11, 8), (11, 9)], secs: 0.25, facing: Up))";
    let def = from_source(
        "o.ron",
        "t",
        &opening("harrowby_yard", cast, action),
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
        "harrowby_yard",
        "(who: \"lead\", at: (11, 8))",
        LEAD_STEPS_IN,
    );
    let plain = from_source("o.ron", "t", &plain, refs(&c)).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(plain.cast[0].facing, Heading::Down);
}
