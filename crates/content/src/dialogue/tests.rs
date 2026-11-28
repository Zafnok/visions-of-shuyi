//! Tests of this game's dialogue: its directives ([`Stage`]), its scene
//! rules (the cast, portraits, plain ASCII, music cues), the embedded
//! scripts and the README's examples. The format's own cases (parsing,
//! errors, printing, ids, `Scene::resolved`) are the engine's tests
//! (`shuyi_assets::dialogue`, ticket 1231).

use alloc::collections::BTreeMap;

use proptest::prelude::*;
use trpg_core::CharacterId;

use super::*;
use crate::bundle;

/// Loads files given as `(file name, source)`, a `None` source being a
/// file that isn't valid UTF-8, as [`load_scripts`] does the bundle's.
fn load_files(
    files: &[(String, Option<&str>)],
    characters: Option<&CharacterTable>,
    portraits: Option<&PortraitTable>,
    names: Option<&Names>,
    audio: Option<&AudioManifest>,
) -> Result<Scripts, Vec<ContentError>> {
    let rules = rules::Rules {
        characters,
        portraits,
        names,
        audio,
    };
    engine::scripts_from_files(files, &rules).map(|s| Scripts::with(s, StandIns::new()))
}

fn id(s: &str) -> CharacterId {
    CharacterId(s.into())
}

fn say(speaker: &str, expression: Option<&str>, text: &str) -> Step {
    Step::Say {
        direction: None,
        speaker: id(speaker),
        expression: expression.map(Into::into),
        text: text.into(),
        line: LineId::default(),
    }
}

fn place(side: Side, character: &str, expression: &str) -> Step {
    Step::Place {
        side,
        character: id(character),
        expression: expression.into(),
    }
}

fn narrate(text: &str) -> Step {
    Step::Narrate {
        direction: None,
        text: text.into(),
        line: LineId::default(),
    }
}

/// `steps` without their line ids, to compare with steps built by hand.
fn bare(steps: &[Step]) -> Vec<Step> {
    let mut steps = steps.to_vec();
    for step in &mut steps {
        match step {
            Step::Say { line, .. } | Step::Narrate { line, .. } => *line = LineId::default(),
            Step::Choice { options } => {
                for o in options {
                    o.line = LineId::default();
                    o.steps = bare(&o.steps);
                }
            }
            Step::If {
                then, otherwise, ..
            } => {
                *then = bare(then);
                *otherwise = bare(otherwise);
            }
            _ => {}
        }
    }
    steps
}

fn music(cue: &str) -> Step {
    Step::Music(MusicLine::Cue(cue.into()))
}

fn characters() -> CharacterTable {
    crate::character::load(None, None, None).unwrap_or_default()
}

fn names() -> Names {
    crate::names::load().unwrap_or_default()
}

/// Every error for one file `t.dlg`, checked against the embedded
/// (placeholder) characters and the names table.
fn errors(src: &str) -> Vec<String> {
    from_sources(
        &[("t.dlg", src)],
        Some(&characters()),
        None,
        Some(&names()),
        None,
    )
    .err()
    .unwrap_or_default()
    .iter()
    .map(ToString::to_string)
    .collect()
}

/// The scenes of `src`, parsed without checks, and without their line
/// ids (to compare with scenes built by hand).
fn scenes(src: &str) -> Vec<Scene> {
    let (parsed, errors) = parse_dlg("t.dlg", src);
    assert_eq!(errors, []);
    let bare = |s: Scene| Scene {
        steps: bare(&s.steps),
        ..s
    };
    parsed.into_iter().map(|p| bare(p.scene)).collect()
}

/// A valid scene around `body`: the lord on the left, the knight on the
/// right.
fn scene(body: &str) -> String {
    format!("@scene s\n@left test_lord neutral\n@right test_knight neutral\n{body}\n@end\n")
}

#[test]
fn a_valid_scene_loads() {
    let src = scene("test_lord[sad]: Hi.\n@left clear\n@left test_archer happy\ntest_archer: Yo.");
    let table = from_sources(
        &[("t.dlg", src.as_str())],
        Some(&characters()),
        None,
        None,
        None,
    );
    assert_eq!(
        table.map(|t| t.get("s").map(|s| s.steps.len())),
        Ok(Some(6))
    );
}

/// Characters aren't checked without the character table.
#[test]
fn without_characters_ids_are_not_checked() {
    let src = "@scene a\n@left nobody neutral\nnobody: Hi.\n@end\n";
    assert!(from_sources(&[("t.dlg", src)], None, None, None, None).is_ok());
    assert_eq!(
        errors(src),
        [
            "t.dlg:2: unknown character \"nobody\"",
            "t.dlg:3: unknown character \"nobody\"",
        ]
    );
}

// --- Syntax errors ---------------------------------------------------------

#[test]
fn non_ascii_text() {
    assert_eq!(
        errors(&scene(
            "test_lord: \u{2018}Hi\u{2019} \u{201C}you\u{201D}\u{2026}\n> A\u{2014}B\u{2013}C\n  x\u{e9}\n@caption Heth\u{a0}x\ntest_knight[sad]: a\tb"
        )),
        [
            "t.dlg:4:12: non-ASCII punctuation '\u{2018}'; use ' instead",
            "t.dlg:4:15: non-ASCII punctuation '\u{2019}'; use ' instead",
            "t.dlg:4:17: non-ASCII punctuation '\u{201C}'; use \" instead",
            "t.dlg:4:21: non-ASCII punctuation '\u{201D}'; use \" instead",
            "t.dlg:4:22: non-ASCII punctuation '\u{2026}'; use ... instead",
            "t.dlg:5:4: non-ASCII punctuation '\u{2014}'; use -- instead",
            "t.dlg:5:6: non-ASCII punctuation '\u{2013}'; use - instead",
            "t.dlg:6:4: unsupported character '\u{e9}'; dialogue text is plain ASCII",
            "t.dlg:7:14: unsupported character '\\u{a0}'; dialogue text is plain ASCII",
            "t.dlg:8:20: unsupported character '\\t'; dialogue text is plain ASCII",
        ]
    );
    // A reply's text too.
    assert_eq!(
        errors(&scene("> x\n@choice\n* a: Caf\u{e9}.\n* b: B.\n@endchoice")),
        ["t.dlg:6:9: unsupported character '\u{e9}'; dialogue text is plain ASCII"]
    );
}

// --- Scene checks ----------------------------------------------------------

#[test]
fn unknown_characters() {
    assert_eq!(
        errors(&scene("@left bors neutral\nbors: Hi.\nmira: Hey.")),
        [
            "t.dlg:4: unknown character \"bors\"",
            "t.dlg:5: unknown character \"bors\"",
            "t.dlg:6: unknown character \"mira\"",
        ]
    );
    // An id the format refuses is still looked up, with its expression.
    let errs = errors("@scene a\n@left Bors Angry\n> x\n@end\n");
    assert!(
        errs.contains(&"t.dlg:2: unknown character \"Bors\"".to_owned()),
        "{errs:?}"
    );
    assert!(
        errs.contains(
            &"t.dlg:2: unknown expression \"Angry\"; use one of neutral, happy, angry, sad, surprised"
                .to_owned()
        ),
        "{errs:?}"
    );
}

#[test]
fn speakers_who_are_not_units_may_be_placed_and_speak() {
    let src = "@scene s\n@left test_voice neutral\n@right test_knight angry\n\
               test_voice: Hold.\ntest_knight[happy]: No.\n@end\n";
    assert_eq!(errors(src), [] as [&str; 0]);
    let characters = characters();
    assert!(!characters.speakers.is_empty());
    let (parsed, parse_errors) = parse_dlg("t.dlg", src);
    assert_eq!(parse_errors, []);
    assert_eq!(parsed.len(), 1);
    for p in &parsed {
        assert_eq!(
            check_scene(p, Some(&characters), None, Some(&names()), None),
            []
        );
    }
    // Someone in neither list still fails, next to a speaker.
    assert_eq!(
        errors("@scene s\n@left test_voice neutral\n@right mira neutral\nmira: Hey.\n@end\n"),
        [
            "t.dlg:3: unknown character \"mira\"",
            "t.dlg:4: unknown character \"mira\"",
        ]
    );
}

#[test]
fn speaker_not_on_screen() {
    assert_eq!(
        errors(&scene(
            "test_archer: Hi.\n@right clear\ntest_knight[sad]: Bye.\ntest_lord: Ok."
        )),
        [
            "t.dlg:4: \"test_archer\" speaks but is not on screen; place them with @left or @right first",
            "t.dlg:6: \"test_knight\" speaks but is not on screen; place them with @left or @right first",
        ]
    );
}

#[test]
fn same_character_on_both_sides() {
    assert_eq!(
        errors(&scene(
            "@right test_lord happy\n@left test_knight neutral\n> x"
        )),
        ["t.dlg:4: \"test_lord\" is already on the left; a character can't be on both sides",]
    );
    // Replacing yourself on the same side, or crossing over after leaving,
    // is fine.
    assert_eq!(
        errors(&scene(
            "@left test_lord sad\n@left clear\n@right test_lord happy\n> x"
        )),
        Vec::<String>::new()
    );
}

/// Every error for `t.dlg`, with expressions checked against `portraits`.
fn errors_with(src: &str, portraits: &PortraitTable) -> Vec<String> {
    from_sources(
        &[("t.dlg", src)],
        Some(&characters()),
        Some(portraits),
        None,
        None,
    )
    .err()
    .unwrap_or_default()
    .iter()
    .map(ToString::to_string)
    .collect()
}

#[test]
fn expressions_come_from_the_portrait() {
    let images = crate::image::load().unwrap_or_default();
    let mut portraits = crate::portrait::load_all(&images).unwrap_or_default();
    // The lord's portrait gains a "smug" expression and loses "sad".
    let lord = portraits.get_mut("test_lord").map(|p| &mut p.expressions);
    if let Some(expressions) = lord {
        expressions.retain(|e| e.name != "sad");
        let smug = expressions
            .first()
            .cloned()
            .map(|e| crate::portrait::Expression {
                name: "smug".into(),
                ..e
            });
        expressions.extend(smug);
    }
    let src = scene(
        "@left test_lord smug
test_lord[sad]: Hi.
test_knight[smug]: Hi.",
    );
    assert_eq!(
        errors_with(&src, &portraits),
        [
            "t.dlg:5: \"test_lord\"'s portrait has no expression \"sad\"; use one of neutral, happy, angry, surprised, smug",
            "t.dlg:6: \"test_knight\"'s portrait has no expression \"smug\"; use one of neutral, happy, angry, sad, surprised",
        ]
    );
    // Characters without a portrait keep the standard five.
    portraits.clear();
    assert_eq!(
        errors_with(&src, &portraits),
        [
            "t.dlg:4: unknown expression \"smug\"; use one of neutral, happy, angry, sad, surprised",
            "t.dlg:6: unknown expression \"smug\"; use one of neutral, happy, angry, sad, surprised",
        ]
    );
    assert_eq!(STANDARD_EXPRESSIONS, crate::portrait::REQUIRED_EXPRESSIONS);
}

#[test]
fn unknown_expressions() {
    assert_eq!(
        errors(&scene("@left test_lord smug\ntest_knight[furious]: Hi.")),
        [
            "t.dlg:4: unknown expression \"smug\"; use one of neutral, happy, angry, sad, surprised",
            "t.dlg:5: unknown expression \"furious\"; use one of neutral, happy, angry, sad, surprised",
        ]
    );
    for e in STANDARD_EXPRESSIONS {
        assert_eq!(
            errors(&scene(&format!("test_lord[{e}]: Hi."))),
            Vec::<String>::new()
        );
    }
}

#[test]
fn text_length_limit() {
    let at_limit = "a".repeat(MAX_TEXT_LEN);
    assert_eq!(
        errors(&scene(&format!("test_lord: {at_limit}\n> {at_limit}"))),
        Vec::<String>::new()
    );
    // Continuation lines count, joined with a space.
    let half = "b".repeat(MAX_TEXT_LEN / 2);
    assert_eq!(
        errors(&scene(&format!(
            "test_lord: {at_limit}x\n> {half}\n  {half}"
        ))),
        [
            "t.dlg:4: text is 201 cells; the limit is 200",
            "t.dlg:5: text is 201 cells; the limit is 200",
        ]
    );
}

#[test]
fn scene_without_text() {
    assert_eq!(
        errors(
            "@scene a\n@caption Only a caption\n@left test_lord neutral\n@end\n@scene b\n@end\n"
        ),
        [
            "t.dlg:1: scene \"a\" has no speech or narration",
            "t.dlg:5: scene \"b\" has no speech or narration",
        ]
    );
}

mod ambience;
mod choice;
mod deaths;
mod knowledge;
mod line_ids;
mod names;
mod openings;
mod places;
mod presence;

// --- Embedded files ---------------------------------------------------------

#[test]
fn embedded_test_scene_loads() {
    let table = load(Some(&characters()), None, Some(&names()), None);
    assert!(table.is_ok(), "{table:?}");
    let steps = table
        .ok()
        .and_then(|t| t.get("test").map(|s| bare(&s.steps)));
    assert_eq!(
        steps.as_ref().map(|s| s[..3].to_vec()),
        Some(vec![
            Step::Caption {
                text: "Village of Heth, dusk".into()
            },
            Step::Narrate {
                direction: Some("slowly, tired".into()),
                text: "The rain had not stopped for three days.".into(),
                line: LineId::default(),
            },
            place(Side::Left, "test_lord", "neutral"),
        ])
    );
    // The knight's lines are in an `@if` block: the army can lose them.
    let Some(Step::If {
        character,
        then,
        otherwise,
    }) = steps.and_then(|s| s.get(3).cloned())
    else {
        panic!("no @if block");
    };
    assert_eq!(character, id("test_knight"));
    assert_eq!(then[0], place(Side::Right, "test_knight", "angry"));
    assert_eq!(
        then[3],
        say(
            "test_knight",
            None,
            "Than never. Say it. I've heard it from you often enough."
        )
    );
    assert_eq!(otherwise, []);
}

/// The format's description, whose examples these tests run. No build
/// embeds it (ADR-0105).
const README: &str = include_str!("../../../../assets/dialogue/README.md");

/// The full example in `assets/dialogue/README.md` parses and passes every
/// check except the character ids (its cast isn't in `characters.ron`). Its
/// `@music` lines name real music cues.
#[test]
fn readme_example_is_valid() {
    let example = README.split("```").nth(1).unwrap_or_default();
    let audio = crate::audio::load().ok();
    assert!(audio.is_some());
    let table = from_sources(&[("README.md", example)], None, None, None, audio.as_ref());
    assert!(table.is_ok(), "{table:?}");
    let table = table.unwrap_or_default();
    let steps = &table.scenes["ch01_opening"].steps;
    assert!(steps.contains(&music("talk_calm")));
    assert!(steps.contains(&Step::Music(MusicLine::Stop)));
    assert_eq!(
        table.scenes.keys().collect::<Vec<_>>(),
        ["ch01_after_battle", "ch01_opening"]
    );
}

// --- Music ------------------------------------------------------------------

/// Every error for `t.dlg`, with `@music` cues checked against the
/// embedded audio manifest.
fn errors_with_audio(src: &str) -> Vec<String> {
    let audio = crate::audio::load().unwrap_or_default();
    from_sources(&[("t.dlg", src)], None, None, None, Some(&audio))
        .err()
        .unwrap_or_default()
        .iter()
        .map(ToString::to_string)
        .collect()
}

/// The music example in `assets/dialogue/README.md` is valid and names
/// real music cues.
#[test]
fn readme_music_example_is_valid() {
    let example = README.split("```").nth(3).unwrap_or_default();
    assert_eq!(errors_with_audio(example), [] as [&str; 0]);
    let (parsed, _) = parse_dlg("README.md", example);
    assert_eq!(parsed[0].scene.id, "ch01_bad_news");
    assert_eq!(parsed[0].scene.steps[0], music("talk_calm"));
    assert_eq!(parsed[0].scene.steps[4], music("scene_sad"));
}

/// `@music` must name a music cue of `assets/audio/audio.ron`: not an
/// unknown id, a sound or a music pool. In a reply's reaction too.
#[test]
fn music_must_be_a_music_cue() {
    let listed = "music cues are listed in assets/audio/audio.ron";
    let src = "\
@scene s
@music no_such_cue
@music menu_move
@music skirmish
@music talk_calm
@music stop
> x
@choice
* wry: Hm.
  @music heal
* blunt: No.
@endchoice
@end
";
    assert_eq!(
        errors_with_audio(src),
        [
            format!("t.dlg:2: \"no_such_cue\" is not a music cue; {listed}"),
            format!("t.dlg:3: \"menu_move\" is a sound, not a music cue; {listed}"),
            format!("t.dlg:4: \"skirmish\" is a music pool, not a music cue; {listed}"),
            format!("t.dlg:10: \"heal\" is a sound, not a music cue; {listed}"),
        ]
    );
    // Without an audio manifest (it failed to load) cues aren't checked.
    assert_eq!(errors(src), [] as [&str; 0]);
}

/// Every embedded script's `@music` cues are checked when the game's
/// content loads.
#[test]
fn the_embedded_scripts_are_checked_against_the_audio_manifest() {
    let mut audio = crate::audio::load().unwrap_or_default();
    assert!(load(None, None, None, Some(&audio)).is_ok());
    let files = [(
        "t.dlg".to_owned(),
        Some("@scene s\n@music talk_calm\n> x\n@end\n"),
    )];
    assert!(load_files(&files, None, None, None, Some(&audio)).is_ok());
    audio.music.remove("talk_calm");
    let errs = load_files(&files, None, None, None, Some(&audio)).err();
    assert_eq!(errs.map(|e| e.len()), Some(1));
}

// --- Round trip of the game's directives -------------------------------------
//
// The engine's own round trip covers every step of the format; these are
// the lines the game adds.

fn arb_id() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,8}"
}

/// No brightness, or one from 1% to 100%.
fn arb_bright() -> impl Strategy<Value = Option<Bright>> {
    proptest::option::of(1u8..=100).prop_map(|n| n.and_then(Bright::new))
}

/// Any of the game's directives.
fn arb_stage() -> impl Strategy<Value = Stage> {
    let at = || (0..100, 0..100).prop_map(|(x, y)| Pos::new(x, y));
    prop_oneof![
        // A map named "battle" or "none" would print as those.
        (arb_id(), at(), arb_bright())
            .prop_filter("reserved", |(m, ..)| m != PlaceLine::BATTLE
                && m != PlaceLine::NONE)
            .prop_map(|(map, at, bright)| Stage::Setting(PlaceLine::Map { map, at, bright })),
        (at(), arb_bright())
            .prop_map(|(at, bright)| Stage::Setting(PlaceLine::Battle { at, bright })),
        Just(Stage::Setting(PlaceLine::None)),
        arb_id()
            .prop_filter("reserved", |a| a != AmbienceLine::NONE)
            .prop_map(|a| Stage::Ambience(AmbienceLine::Bed(a))),
        Just(Stage::Ambience(AmbienceLine::None)),
        arb_id().prop_map(Stage::Opening),
        arb_id().prop_map(|c| Stage::Dies(CharacterId(c))),
        arb_id().prop_map(|c| Stage::Joins(CharacterId(c))),
    ]
}

proptest! {
    /// Every directive of the game prints as a line that parses back to it.
    #[test]
    fn the_games_directives_print_and_parse_back(
        stages in proptest::collection::vec(arb_stage(), 1..8),
    ) {
        let mut steps: Vec<Step> = stages.into_iter().map(Step::Directive).collect();
        steps.push(narrate("x"));
        let scene = Scene { id: "s".into(), steps };
        let (parsed, errors) = parse_dlg("t.dlg", &print_scene(&scene));
        prop_assert_eq!(errors, []);
        prop_assert_eq!(parsed.len(), 1);
        prop_assert_eq!(bare(&parsed[0].scene.steps), scene.steps);
    }
}

/// What [`load_scripts`] reports on the files of `files` (bundle path and
/// text), shown.
fn load_errors(files: &'static [(&'static str, &'static [u8])]) -> Vec<String> {
    // The registry is built from the game's own files first.
    crate::formats::install();
    let layer = bundle::Layer::table("fixture", files);
    let fixture: &'static bundle::Bundle = Box::leak(Box::new(bundle::Bundle::new(vec![layer])));
    let result = bundle::reading(fixture, || load_scripts(None, None, None, None));
    let errors = result.err().unwrap_or_default();
    errors.iter().map(ToString::to_string).collect()
}

/// A file that fails to read is reported, and the files that read are
/// still checked: every problem, by file and line.
#[test]
fn load_scripts_reports_a_broken_file_and_the_checks_of_the_others() {
    static BROKEN: &[(&str, &[u8])] = &[
        ("dialogue/a.dlg", b"@scene a\n> One.\n@end\n"),
        ("dialogue/b.dlg", b"@scene b\nnot a line\n@end\n"),
    ];
    static BOTH: &[(&str, &[u8])] = &[
        (
            "dialogue/a.dlg",
            b"@scene a\n> One.\n@end\n@scene a\n> Two.\n@end\n",
        ),
        ("dialogue/b.dlg", b"@scene b\nnot a line\n@end\n"),
    ];
    let unrecognised = "fixture/dialogue/b.dlg:2: unrecognised line; expected \"@directive\", \
                        \"> narration\", \"speaker: text\" or \"# comment\"";
    assert_eq!(load_errors(BROKEN), [unrecognised]);
    // The duplicate in the file that read comes first: it is the first
    // file's.
    let duplicate = "fixture/dialogue/a.dlg:4: duplicate scene id \"a\"; first used at \
                     fixture/dialogue/a.dlg:1";
    assert_eq!(load_errors(BOTH), [duplicate, unrecognised]);
}

/// The format's rules on a scene as written (ticket 0764): a stand-in,
/// `{lead|the exile}`, is the name and as long; every other answer is the
/// lead's tokens'.
#[test]
fn the_format_rules_read_a_stand_in_as_the_name() {
    use shuyi_assets::dialogue::{SceneRules, Tokens};
    let written = FormatRules.tokens();
    let lead = LeadTokens;
    assert_eq!(written.longest("lead|the exile"), lead.longest("lead"));
    assert!(written.longest("lead").is_some());
    assert_eq!(written.longest("lead|"), None, "no words: no stand-in");
    assert_eq!(written.longest("nope"), None);
    assert_eq!(written.unknown("nope"), lead.unknown("nope"));
    assert!(written.unknown("nope").contains("{lead}"));
    for (id, token) in [("lead", "n:lead"), ("king", "n:king")] {
        assert_eq!(
            written.reserved_name(id, token),
            lead.reserved_name(id, token)
        );
        assert_eq!(
            written.unknown_name(id, token),
            lead.unknown_name(id, token)
        );
    }
    assert!(written.reserved_name("lead", "n:lead").is_some());
    assert!(written.reserved_name("king", "n:king").is_none());
    for (id, form) in [("lead", "Exile"), ("king", "Aldric")] {
        assert_eq!(written.written_name(id, form), lead.written_name(id, form));
    }
    assert!(written.written_name("lead", "Exile").is_some());
    assert!(written.written_name("king", "Aldric").is_none());
    assert_eq!(FormatRules.limits(), &LIMITS);
}

/// A scene as the text box shows it (ticket 0764): its stand-ins written
/// `{lead}`, every line keeping the id it was read with.
#[test]
fn a_scene_as_shown_has_the_name_where_it_had_a_stand_in() {
    let (parsed, errors) = parse_dlg(
        "t.dlg",
        "@scene a\n> {lead|The exile} waits.\n> Rain.\n@end\n",
    );
    assert_eq!(errors, []);
    let scene = &parsed[0].scene;
    let shown = as_shown(scene);
    assert_eq!(shown.id, "a");
    let texts: Vec<&str> = shown.steps.iter().filter_map(|s| s.text()).collect();
    assert_eq!(texts, ["{lead} waits.", "Rain."]);
    let ids = |s: &Scene| {
        s.lines()
            .into_iter()
            .map(|l| l.id.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(ids(&shown), ids(scene));
    assert_eq!(as_shown(&shown), shown, "nothing left to take out");
}
