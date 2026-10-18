//! Cinematic file tests, against the embedded content.

use super::*;
use crate::Content;

mod actions;
mod combat;
mod openings;

fn content() -> Content {
    crate::load_embedded().unwrap_or_else(|e| panic!("{e}"))
}

/// A cinematic file with `length` and `shots` as written.
fn source(length: &str, shots: &str) -> String {
    format!("(id: \"t\", length: {length}, shots: [{shots}])")
}

const PAN: &str = "(at: 0.0, shot: MapPan(battle: \"test\", from: (0, 0), to: (13, 7), zoom: 2))";

fn load(c: &Content, source: &str) -> Result<CinematicDef, Vec<ContentError>> {
    from_source("c.ron", "t", source, Refs::of(c))
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
    assert_eq!((test.music.as_deref(), test.length_ms), (None, 20_000));
    assert_eq!(test.shots.len(), 4);
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
    let talk = ShotDef {
        at_ms: 6000,
        shot: Shot::Talk {
            scene: "test".to_owned(),
            first: 6,
            count: 2,
        },
    };
    assert_eq!(test.shots[1], talk);
    let who = ["lead", "test_lord", "test_knight"];
    let faces = ShotDef {
        at_ms: 10_000,
        shot: Shot::Characters {
            who: who.iter().map(|id| CharacterId((*id).to_owned())).collect(),
            expression: "neutral".to_owned(),
            from: Side::Right,
        },
    };
    assert_eq!(test.shots[2], faces);
    let logo = ShotDef {
        at_ms: 16_000,
        shot: Shot::Logo,
    };
    assert_eq!(test.shots[3], logo);
    // A build that ships has the title's alone.
    let shipped = bundle::story_only(|| load_all(Refs::of(&c))).unwrap();
    assert_eq!(shipped.keys().collect::<Vec<_>>(), ["title"]);
    assert_eq!(shipped["title"], c.cinematics["title"]);
}

/// The title's cinematic is as long as the title song and ends on the
/// logo, from the song's last note (1:57) to its loop
/// (`docs/design/title-screen.md`, *Intro cinematic*). Its shots before
/// that are a stand-in until ticket 0820: a pan over a story battle.
#[test]
fn the_title_cinematic_is_as_long_as_its_song_and_ends_on_the_logo() {
    let c = content();
    let title = &c.cinematics["title"];
    assert_eq!(title.music.as_deref(), Some("title"));
    assert_eq!(title.length_ms, c.audio.music["title"].length_ms);
    assert_eq!(title.shots[0].at_ms, 0);
    let logo = ShotDef {
        at_ms: 117_000,
        shot: Shot::Logo,
    };
    assert_eq!(title.shots.last(), Some(&logo));
    for shot in &title.shots {
        if let Shot::MapPan { battle, .. } = &shot.shot {
            assert_eq!(battle, "ch01");
        }
    }
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

// The file's frame (its id, its length and the order of its shots) is the
// engine's, tested there: `shuyi_assets::cinematic`'s tests.

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

/// A cinematic that is one shot, as written.
fn one(shot: &str) -> String {
    source("Seconds(5)", &format!("(at: 0, shot: {shot})"))
}

/// The messages of the errors of `source` checked against `refs`.
fn problems_with(refs: Refs<'_>, source: &str) -> Vec<String> {
    let errors = from_source("c.ron", "t", source, refs).unwrap_err();
    errors.into_iter().map(|e| e.message).collect()
}

#[test]
fn the_opening_boxes_are_the_text_before_the_first_reply_choice() {
    let c = content();
    // Everyone is there: the `@if test_knight` block plays.
    let (boxes, choice) = opening_boxes(&c.dialogue.scenes["test"]);
    assert!(choice);
    assert_eq!(boxes.len(), 8);
    assert_eq!(boxes[0], "The rain had not stopped for three days.");
    assert_eq!(boxes[1], "You're late.");
    assert_eq!(boxes[7], "{lead}! You came back for us.");
    let intro = &c.dialogue.scenes["test_intro"];
    let (boxes, choice) = opening_boxes(intro);
    assert!(!choice);
    let texts = intro.steps.iter().filter(|s| s.text().is_some()).count();
    assert_eq!(boxes.len(), texts);
    assert!(!boxes.is_empty());
}

#[test]
fn a_talk_shot_names_boxes_of_a_scene_before_its_first_choice() {
    let c = content();
    let talk = |scene: &str, first: usize, count: usize| {
        one(&format!(
            "Talk(scene: \"{scene}\", first: {first}, count: {count})"
        ))
    };
    for (first, count) in [(0, 1), (0, 8), (7, 1), (6, 2)] {
        let def = load(&c, &talk("test", first, count)).unwrap();
        let scene = "test".to_owned();
        let shot = Shot::Talk {
            scene,
            first,
            count,
        };
        assert_eq!(def.shots[0].shot, shot);
    }
    assert_eq!(
        problems(&c, &talk("nope", 0, 1)),
        ["shot 1: no dialogue scene \"nope\""]
    );
    assert_eq!(
        problems(&c, &talk("test", 0, 0)),
        ["shot 1: count 0; a talk shot shows at least one text box"]
    );
    // The eighth box is the last before the replies.
    let past = [
        (7, 2, 8),
        (0, 9, 8),
        (8, 1, 8),
        (usize::MAX, 2, usize::MAX - 1),
    ];
    for (first, count, last) in past {
        assert_eq!(
            problems(&c, &talk("test", first, count)),
            [format!(
                "shot 1: text boxes {first} to {last} of scene \"test\": it has 8 before its \
                 first reply choice, and a cinematic never shows a choice (the first is 0)"
            )]
        );
    }
    // A scene without a choice: up to its last box.
    let all = opening_boxes(&c.dialogue.scenes["test_intro"]).0.len();
    assert!(load(&c, &talk("test_intro", 0, all)).is_ok());
    assert_eq!(
        problems(&c, &talk("test_intro", 1, all)),
        [format!(
            "shot 1: text boxes 1 to {all} of scene \"test_intro\": it has {all} in all \
             (the first is 0)"
        )]
    );
}

#[test]
fn the_boxes_of_a_talk_shot_each_fit_one_page() {
    let c = content();
    let narrate = |text: String| Step::Narrate {
        text,
        line: crate::LineId::default(),
    };
    // Three full lines fit; a word more is a fourth line.
    let line = "x".repeat(BOX_WIDTH);
    let fits = [line.as_str(); 3].join(" ");
    let mut scenes = c.dialogue.clone();
    let steps = vec![
        narrate(fits.clone()),
        narrate(format!("{fits} y")),
        narrate(fits),
    ];
    let id = "long".to_owned();
    scenes.scenes.insert(id.clone(), Scene { id, steps });
    let refs = Refs {
        scenes: &scenes,
        ..Refs::of(&c)
    };
    assert_eq!(
        problems_with(refs, &one("Talk(scene: \"long\", first: 0, count: 3)")),
        [
            "shot 1: text box 1 of scene \"long\" may not fit one page (3 lines of 92); \
          a cinematic turns no pages"
        ]
    );
    let src = one("Talk(scene: \"long\", first: 2, count: 1)");
    assert!(from_source("c.ron", "t", &src, refs).is_ok());
}

#[test]
fn page_lines_wraps_as_the_text_box_does_with_tokens_at_their_longest() {
    assert_eq!(page_lines("", 5), 0);
    assert_eq!(page_lines("one two", 5), 1);
    let w = BOX_WIDTH;
    let a = |n: usize| "a".repeat(n);
    // A word that just fits the line, and one that doesn't.
    assert_eq!(page_lines(&format!("{} b", a(w - 2)), 5), 1);
    assert_eq!(page_lines(&format!("{} b", a(w - 1)), 5), 2);
    assert_eq!(page_lines(&format!("{} b", a(w)), 5), 2);
    // A word longer than a line is cut; the next joins its last piece.
    assert_eq!(page_lines(&a(w + 1), 5), 2);
    assert_eq!(page_lines(&a(2 * w), 5), 2);
    assert_eq!(page_lines(&a(2 * w + 1), 5), 3);
    assert_eq!(page_lines(&format!("{} b", a(w + 1)), 5), 2);
    assert_eq!(page_lines(&format!("{} {}", a(w + 1), a(w - 2)), 5), 2);
    assert_eq!(page_lines(&format!("{} {}", a(w + 1), a(w - 1)), 5), 3);
    assert_eq!(page_lines(&format!("{} b", a(2 * w)), 5), 3);
    // The lead's name counts as its longest (12), a name token as the
    // longest name given, a pronoun as its longest ("himself").
    let pad = a(w - 13);
    assert_eq!(page_lines(&format!("{pad} {{lead}}"), 5), 1);
    assert_eq!(page_lines(&format!("{pad}a {{lead}}"), 5), 2);
    assert_eq!(page_lines(&format!("{pad} {{n:king}}"), 12), 1);
    assert_eq!(page_lines(&format!("{pad} {{n:king}}"), 13), 2);
    assert_eq!(page_lines(&format!("{pad}aaaaa {{themself}}"), 5), 1);
    assert_eq!(page_lines(&format!("{pad}aaaaaa {{themself}}"), 5), 2);
    // Something in braces that is no token counts as written.
    assert_eq!(page_lines(&format!("{pad}aaaaaaa {{abc}}"), 5), 1);
    assert_eq!(page_lines(&format!("{pad}aaaaaaaa {{abc}}"), 5), 2);
    // The space between words counts, for every word of a line.
    assert_eq!(page_lines(&format!("{} b c", a(w - 4)), 5), 1);
    assert_eq!(page_lines(&format!("{} b c", a(w - 3)), 5), 2);
    assert_eq!(page_lines(&format!("{pad}aaa {{unclosed"), 5), 1);
    assert_eq!(page_lines(&format!("{pad}aaaa {{unclosed"), 5), 2);
}

#[test]
fn a_characters_shot_names_characters_and_an_expression_they_have() {
    let c = content();
    let faces = |who: &str, rest: &str| one(&format!("Characters(who: [{who}]{rest})"));
    let neutral = ", expression: \"neutral\"";
    let def = load(&c, &faces("\"test_lord\", \"lead\"", neutral)).unwrap();
    let Shot::Characters {
        who,
        expression,
        from,
    } = &def.shots[0].shot
    else {
        panic!("{def:?}");
    };
    let ids: Vec<_> = who.iter().map(|c| c.0.as_str()).collect();
    assert_eq!(ids, ["test_lord", "lead"]);
    assert_eq!((expression.as_str(), *from), ("neutral", Side::Right));
    // The side is the right unless the file says the left.
    let side = |from: &str| {
        let rest = format!("{neutral}, from: {from}");
        match load(&c, &faces("\"lead\"", &rest)).unwrap().shots[0].shot {
            Shot::Characters { from, .. } => from,
            ref other => panic!("{other:?}"),
        }
    };
    assert_eq!((side("Left"), side("Right")), (Side::Left, Side::Right));
    assert_eq!(
        problems(&c, &faces("", neutral)),
        ["shot 1: who: a characters shot needs at least one character"]
    );
    // Someone unknown; a portrait without the expression (the lead has
    // two, and both are checked); a character without a portrait wears
    // only the standard expressions.
    assert!(c.portraits.contains_key("test_lord") && !c.portraits.contains_key("test_archer"));
    let who = "\"nobody\", \"test_lord\", \"lead\", \"test_archer\"";
    assert_eq!(
        problems(&c, &faces(who, ", expression: \"smug\"")),
        [
            "shot 1: no character \"nobody\"",
            "shot 1: \"test_lord\"'s portrait has no expression \"smug\"",
            "shot 1: \"lead_m\"'s portrait has no expression \"smug\"",
            "shot 1: \"lead_f\"'s portrait has no expression \"smug\"",
            "shot 1: \"test_archer\" has no portrait, and \"smug\" is not a standard expression",
        ]
    );
    let sad = ", expression: \"sad\"";
    assert!(load(&c, &faces("\"test_archer\"", sad)).is_ok());
    // Only one of the lead's looks lacks it.
    let mut portraits = c.portraits.clone();
    let lead_f = portraits.get_mut("lead_f").unwrap();
    lead_f.expressions.retain(|e| e.name != "sad");
    let refs = Refs {
        portraits: &portraits,
        ..Refs::of(&c)
    };
    assert_eq!(
        problems_with(refs, &faces("\"lead\"", sad)),
        ["shot 1: \"lead_f\"'s portrait has no expression \"sad\""]
    );
    // A lead without portraits is a character without one.
    portraits.remove("lead_m");
    portraits.remove("lead_f");
    let refs = Refs {
        portraits: &portraits,
        ..Refs::of(&c)
    };
    assert!(from_source("c.ron", "t", &faces("\"lead\"", sad), refs).is_ok());
    assert_eq!(
        problems_with(refs, &faces("\"lead\"", ", expression: \"smug\"")),
        ["shot 1: \"lead\" has no portrait, and \"smug\" is not a standard expression"]
    );
}
