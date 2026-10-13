//! Cinematic file tests, against the embedded content.

use super::*;
use crate::Content;

mod actions;

fn content() -> Content {
    crate::load_embedded().unwrap_or_else(|e| panic!("{e}"))
}

/// A cinematic file with `length` and `shots` as written.
fn source(length: &str, shots: &str) -> String {
    format!("(id: \"t\", length: {length}, shots: [{shots}])")
}

const PAN: &str = "(at: 0.0, shot: MapPan(battle: \"test\", from: (0, 0), to: (13, 7), zoom: 2))";

type UnitNames = BTreeMap<String, BTreeMap<String, UnitId>>;

fn refs<'a>(c: &'a Content, units: &'a UnitNames) -> CinematicRefs<'a> {
    CinematicRefs {
        battles: &c.battles,
        audio: &c.audio,
        units,
        classes: &c.classes,
        terrain: &c.terrain.rules,
        characters: &c.characters,
    }
}

fn load(c: &Content, source: &str) -> Result<CinematicDef, Vec<ContentError>> {
    let units = crate::battle::load_unit_names();
    from_source("c.ron", "t", source, &refs(c, &units))
}

/// The messages of the errors of `source` as file `t`.
fn problems(c: &Content, source: &str) -> Vec<String> {
    match load(c, source) {
        Ok(def) => panic!("loaded: {def:?}"),
        Err(errors) => {
            assert!(errors.iter().all(|e| e.file == "c.ron"), "{errors:?}");
            errors.into_iter().map(|e| e.message).collect()
        }
    }
}

#[test]
fn the_test_cinematic_loads() {
    let c = content();
    let test = &c.cinematics["test"];
    assert_eq!(test.id, "test");
    assert_eq!((test.music.as_deref(), test.length_ms), (None, 10_000));
    assert_eq!(test.shots.len(), 2);
    assert_eq!(test.shots[0].at_ms, 0);
    assert_eq!(
        test.shots[0].shot,
        Shot::MapPan {
            battle: "test".to_owned(),
            from: TilePoint::new(5.75, 3.5),
            to: TilePoint::new(7.25, 3.5),
            zoom: 4,
            action: Some(ShotAction {
                from_ms: 0,
                actions: actions::duel(),
            }),
        }
    );
    let logo = ShotDef {
        at_ms: 6000,
        shot: Shot::Logo,
    };
    assert_eq!(test.shots[1], logo);
    // The test files hold the only cinematic so far: a build that ships
    // has none.
    let shipped = bundle::story_only(|| load_all(&refs(&c, &BTreeMap::new())));
    assert_eq!(shipped, Ok(BTreeMap::new()));
}

#[test]
fn a_cinematic_on_a_music_cue_is_as_long_as_the_cue() {
    let c = content();
    let on_title = |last: &str| {
        let shots = format!("{PAN}, (at: {last}, shot: Logo)");
        source("Music(\"title\")", &shots)
    };
    let def = load(&c, &on_title("133.742")).unwrap();
    assert_eq!(def.music.as_deref(), Some("title"));
    assert_eq!(def.length_ms, c.audio.music["title"].length_ms);
    assert_eq!(def.length_ms, 133_743);
    assert_eq!(def.shots[1].at_ms, 133_742);
    // A shot at the cue's very end would never show.
    assert_eq!(
        problems(&c, &on_title("133.743")),
        ["shot 2: at 133.743 is not before the end (133.743 seconds)"]
    );
    // Without the cue there is no end to check the shots against.
    let src = on_title("500").replace("title", "nope");
    assert_eq!(problems(&c, &src), ["length: no music cue \"nope\""]);
}

#[test]
fn the_id_is_the_file_name() {
    let c = content();
    let src = source("Seconds(1)", PAN).replace("\"t\"", "\"x\"");
    assert_eq!(
        problems(&c, &src),
        ["id \"x\" must match the file name \"t\""]
    );
}

#[test]
fn a_stated_length_is_above_zero_and_not_too_long() {
    let c = content();
    let stated = |length: &str| load(&c, &source(length, PAN));
    assert_eq!(stated("Seconds(0.001)").unwrap().length_ms, 1);
    assert_eq!(stated("Seconds(3600)").unwrap().length_ms, 3_600_000);
    assert_eq!(stated("Seconds(2.5)").unwrap().music, None);
    for length in ["0", "0.0004", "-1", "3600.5", "inf", "NaN"] {
        assert_eq!(
            problems(&c, &source(&format!("Seconds({length})"), PAN)),
            [format!(
                "length: {length} seconds; it must be above 0 and at most 3600"
            )],
        );
    }
}

#[test]
fn shots_are_in_order_from_zero_to_before_the_end() {
    let c = content();
    let check = |shots: &str| problems(&c, &source("Seconds(10)", shots));
    assert_eq!(check(""), ["shots: a cinematic needs at least one shot"]);
    assert_eq!(
        check("(at: 0.5, shot: Logo)"),
        ["shot 1: the first shot must be at 0.0, not 0.5"]
    );
    assert_eq!(
        check("(at: 0, shot: Logo), (at: 4, shot: Logo), (at: 4, shot: Logo)"),
        ["shot 3: at 4 must be after the shot before it; shots are in order of time"]
    );
    assert_eq!(
        check("(at: 0, shot: Logo), (at: 4, shot: Logo), (at: 3.999, shot: Logo)"),
        ["shot 3: at 3.999 must be after the shot before it; shots are in order of time"]
    );
    assert_eq!(
        check("(at: 0, shot: Logo), (at: 10, shot: Logo)"),
        ["shot 2: at 10 is not before the end (10 seconds)"]
    );
    assert_eq!(
        check("(at: 0, shot: Logo), (at: 9.999, shot: Logo), (at: 11, shot: Logo)"),
        ["shot 3: at 11 is not before the end (10 seconds)"]
    );
    for at in ["-1", "NaN", "inf", "5000000"] {
        assert_eq!(
            check(&format!("(at: 0, shot: Logo), (at: {at}, shot: Logo)")),
            [format!("shot 2: at {at} is not a time")]
        );
    }
    // A first shot that is no time is only that.
    assert_eq!(
        check("(at: -1, shot: Logo)"),
        ["shot 1: at -1 is not a time"]
    );
    // In order, to the millisecond.
    let ok = source(
        "Seconds(10)",
        "(at: 0, shot: Logo), (at: 0.001, shot: Logo)",
    );
    let starts: Vec<u32> = load(&c, &ok)
        .unwrap()
        .shots
        .iter()
        .map(|s| s.at_ms)
        .collect();
    assert_eq!(starts, [0, 1]);
}

#[test]
fn a_map_pan_names_a_battle_points_on_its_map_and_a_whole_zoom() {
    let c = content();
    let pan = |battle: &str, from: &str, to: &str, zoom: &str| {
        let shot = format!(
            "(at: 0, shot: MapPan(battle: \"{battle}\", from: {from}, to: {to}, zoom: {zoom}))"
        );
        source("Seconds(10)", &shot)
    };
    let check = |b: &str, from: &str, to: &str, zoom: &str| problems(&c, &pan(b, from, to, zoom));
    // The test map is 14 × 8 tiles: its edges are half a tile past the
    // middles of its outer tiles.
    for (from, to) in [
        ("(-0.5, -0.5)", "(13.5, 7.5)"),
        ("(13.5, -0.5)", "(0, 7.5)"),
    ] {
        assert!(load(&c, &pan("test", from, to, "1")).is_ok(), "{from} {to}");
    }
    assert_eq!(
        check("nope", "(99, 99)", "(0, 0)", "1"),
        ["shot 1: no battle \"nope\""]
    );
    let off = |name: &str, point: &str| {
        format!("shot 1: {name} {point} is not on the map of battle \"test\" (14×8 tiles)")
    };
    for point in ["(-0.501, 0)", "(0, -0.501)", "(13.501, 7)", "(13, 7.501)"] {
        assert_eq!(check("test", point, "(0, 0)", "1"), [off("from", point)]);
        assert_eq!(check("test", "(0, 0)", point, "1"), [off("to", point)]);
    }
    assert_eq!(
        check("test", "(NaN, 0)", "(0, inf)", "1"),
        [off("from", "(NaN, 0)"), off("to", "(0, inf)")]
    );
    for zoom in ["0", "5"] {
        assert_eq!(
            check("test", "(0, 0)", "(0, 0)", zoom),
            [format!("shot 1: zoom {zoom}; a zoom is 1 to 4")]
        );
    }
    for zoom in ["1", "4"] {
        assert!(load(&c, &pan("test", "(0, 0)", "(0, 0)", zoom)).is_ok());
    }
}

#[test]
fn every_problem_is_reported_at_once() {
    let c = content();
    let src = "(id: \"x\", length: Music(\"nope\"), shots: [
        (at: 1, shot: MapPan(battle: \"nope\", from: (0, 0), to: (0, 0), zoom: 9)),
        (at: 1, shot: MapPan(battle: \"test\", from: (50, 0), to: (0, 0), zoom: 2)),
    ])";
    assert_eq!(
        problems(&c, src),
        [
            "id \"x\" must match the file name \"t\"",
            "length: no music cue \"nope\"",
            "shot 1: the first shot must be at 0.0, not 1",
            "shot 1: no battle \"nope\"",
            "shot 1: zoom 9; a zoom is 1 to 4",
            "shot 2: at 1 must be after the shot before it; shots are in order of time",
            "shot 2: from (50, 0) is not on the map of battle \"test\" (14×8 tiles)",
        ]
    );
    // A file that isn't RON is one error, with its place.
    let errors = load(&c, "(id: 3)").unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(
        (errors[0].file.as_str(), errors[0].line),
        ("c.ron", Some(1))
    );
    // A field the format doesn't have is refused: there are no fades.
    let src = source("Seconds(1)", PAN).replace("id:", "fade: 1, id:");
    assert_eq!(problems(&c, &src).len(), 1);
}

#[test]
fn a_tile_point_is_kept_to_the_thousandth_of_a_tile() {
    assert_eq!(TilePoint::new(4.0, 9.5).tiles(), (4.0, 9.5));
    assert_eq!(TilePoint::new(-0.25, 0.0).tiles(), (-0.25, 0.0));
    assert_eq!(TilePoint::new(1.0004, 2.0006), TilePoint::new(1.0, 2.001));
    assert_eq!(TilePoint::new(f32::NAN, 1.0).tiles(), (0.0, 1.0));
    assert!(TilePoint::new(0.0, 0.0).is_on(1, 1));
    assert!(TilePoint::new(0.5, 0.5).is_on(1, 1));
    assert!(TilePoint::new(-0.5, -0.5).is_on(1, 1));
    assert!(!TilePoint::new(0.501, 0.0).is_on(1, 1));
    assert!(!TilePoint::new(0.0, 0.501).is_on(1, 1));
    assert!(!TilePoint::new(-0.501, 0.0).is_on(1, 1));
    assert!(!TilePoint::new(0.0, -0.501).is_on(1, 1));
    assert!(TilePoint::new(2.5, 0.0).is_on(3, 1));
    assert!(!TilePoint::new(0.0, 2.5).is_on(3, 1));
    assert!(!TilePoint::new(0.0, 0.0).is_on(0, 0));
}
