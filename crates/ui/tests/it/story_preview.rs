//! The writer's script preview (ticket 0764) through the Harness: the
//! scenes of a folder listed, one played in the game's dialogue screen, a
//! character sent away and the `@else` line played, a jump to a line, a
//! saved change and a broken file read through the headless backend's
//! `change_file` with debug tools off, the page of names and faces, and two
//! scenes of Chapter 1 alone checked as a trial packet.

use std::collections::BTreeMap;

use shuyi_assets::bundle::Layer;
use shuyi_ui::dialogue::preview::{PreviewView, Tab};
use trpg_content::bundle::skip_without_the_game;
use trpg_core::LeadGender;
use trpg_core::lead::DEFAULT_NAME;
use trpg_ui::harness::{Core, FRAME_DT};
use trpg_ui::input::{Action, RIGHT_HANDED};
use trpg_ui::screen::Screen;
use trpg_ui::story_preview::{self, LONG_NAME, ReferenceRow, StoryPreview, StoryPreviewView};

/// A scene file of the test characters: a caption, a place, an `@if`
/// block for the knight, the lead's name, and a second scene.
const TRIAL: &str = "@scene trial
@caption Village of Heth, dusk
@place test_small 2,2
> (slowly): The rain had not stopped.
@left test_lord neutral
@if test_knight
@right test_knight angry
test_knight: You're late.
@right clear
@else
> Nobody waits for you.
@endif
test_lord[sad] (quietly): Let's go, {lead|exile}.
@end

@scene trial_two
> The road goes on.
@end
";

/// The preview over a folder holding `files`, watched in every build, in
/// a Harness with the right-handed keys: its first frame read.
fn preview_of(files: &[(&str, &str)]) -> (Core, Layer) {
    let files: BTreeMap<String, Vec<u8>> = files
        .iter()
        .map(|(name, text)| ((*name).to_owned(), text.as_bytes().to_vec()))
        .collect();
    let folder = Layer::owned("dialogue".to_owned(), files).unwrap_or_else(|e| panic!("{e:?}"));
    let mut ctx = story_preview::ctx().unwrap_or_else(|e| panic!("{e}"));
    ctx.use_layout(RIGHT_HANDED);
    let game = story_preview::game(ctx, folder).unwrap_or_else(|e| panic!("{e}"));
    let mut h = Core::new(game);
    h.watch_files(true);
    h.wait(FRAME_DT);
    (h, folder)
}

/// The preview over `trial.dlg`.
fn preview() -> (Core, Layer) {
    preview_of(&[("trial.dlg", TRIAL)])
}

/// The preview screen.
fn screen(h: &Core) -> &StoryPreview {
    h.screen::<StoryPreview>()
        .unwrap_or_else(|| panic!("no preview: {:?}", h.screens()))
}

/// What the engine's screen shows.
fn view(h: &Core) -> PreviewView {
    screen(h).preview().view(h.ctx())
}

/// What the game's part shows.
fn game_view(h: &Core) -> StoryPreviewView {
    screen(h).view(h.ctx())
}

/// The text the game's dialogue screen shows.
fn shown(h: &Core) -> String {
    let playing = screen(h).playing();
    let text = playing.and_then(|s| s.player().current().text.map(std::borrow::Cow::into_owned));
    text.unwrap_or_default()
}

/// Each action in turn, a frame each.
fn act(h: &mut Core, actions: &[Action]) {
    for &action in actions {
        h.act(action);
    }
}

#[test]
fn the_folders_scenes_are_listed_in_file_order() {
    let (h, _) = preview_of(&[("b.dlg", "@scene b\n> Bee.\n@end\n"), ("trial.dlg", TRIAL)]);
    let scenes: Vec<(String, String)> = view(&h)
        .scenes
        .into_iter()
        .map(|s| (s.file, s.id))
        .collect();
    let row = |file: &str, id: &str| (file.to_owned(), id.to_owned());
    assert_eq!(
        scenes,
        [
            row("b.dlg", "b"),
            row("trial.dlg", "trial"),
            row("trial.dlg", "trial_two")
        ]
    );
    assert_eq!(view(&h).problems, [], "the test scenes have no problem");
    assert_eq!(h.screens(), [StoryPreview::NAME]);
}

#[test]
fn a_scene_plays_in_the_games_dialogue_screen_with_its_faces_under_the_strip() {
    let (mut h, _) = preview();
    h.act(Action::Confirm);
    assert_eq!(shown(&h), "The rain had not stopped.");
    let strip = view(&h).strip.unwrap_or_else(|| panic!("no strip"));
    assert_eq!((strip.file.as_str(), strip.line), ("trial.dlg", 4));
    assert_eq!(strip.direction.as_deref(), Some("slowly"));
    // The knight's line: both faces named.
    act(&mut h, &[Action::Confirm, Action::Confirm]);
    assert_eq!(shown(&h), "You're late.");
    let faces = game_view(&h).faces;
    assert_eq!(
        faces.as_deref(),
        Some("Faces: test_lord[neutral]  test_knight[angry]")
    );
    assert!(h.undeclared().is_empty(), "{:?}", h.undeclared());
}

#[test]
fn with_a_character_switched_off_the_scene_plays_its_else_line() {
    let (mut h, _) = preview();
    h.act(Action::Confirm);
    // The Switches tab: the knight, there, is its first row.
    act(&mut h, &[Action::Cancel, Action::CursorRight]);
    assert_eq!(view(&h).tab, Tab::Switches);
    let names: Vec<String> = view(&h).switches.into_iter().map(|s| s.name).collect();
    assert_eq!(
        names,
        [
            "test_knight",
            "The lead is female",
            "The lead has a 12-letter name",
            "Strip: file, line, direction, faces",
        ]
    );
    h.act(Action::Confirm);
    assert!(view(&h).switch("test_knight").is_some_and(|s| !s.on));
    // The Lines tab: the line box, then the text boxes as they play now.
    h.act(Action::CursorRight);
    let lines: Vec<(u32, String)> = view(&h)
        .lines
        .into_iter()
        .map(|l| (l.line, l.words))
        .collect();
    let line = |n: u32, words: &str| (n, words.to_owned());
    assert_eq!(
        lines,
        [
            line(4, "The rain had not stopped."),
            line(11, "Nobody waits for you."),
            line(13, "Let's go, {lead|exile}."),
        ]
    );
    act(
        &mut h,
        &[Action::CursorDown, Action::CursorDown, Action::Confirm],
    );
    assert_eq!(shown(&h), "Nobody waits for you.");
}

#[test]
fn a_jump_to_a_line_shows_its_text_box_as_the_lines_before_set_it() {
    let (mut h, _) = preview();
    h.act(Action::Confirm);
    act(
        &mut h,
        &[Action::Cancel, Action::CursorRight, Action::CursorRight],
    );
    assert_eq!(view(&h).tab, Tab::Lines);
    h.act(Action::Confirm);
    assert!(view(&h).go_to.typing);
    h.type_text("8");
    h.act(Action::Confirm);
    assert!(!view(&h).open);
    let playing = screen(&h)
        .playing()
        .unwrap_or_else(|| panic!("nothing plays"));
    let now = playing.player().current();
    assert_eq!(now.text.as_deref(), Some("You're late."));
    let face = |p: Option<shuyi_ui::dialogue::Portrait<'_>>| {
        p.map(|p| (p.character.0.clone(), p.expression.to_owned()))
    };
    assert_eq!(face(now.left), Some(("test_lord".into(), "neutral".into())));
    assert_eq!(
        face(now.right),
        Some(("test_knight".into(), "angry".into()))
    );
    assert_eq!(now.caption.as_deref(), Some("Village of Heth, dusk"));
    let place = now.place.map(|p| format!("{p:?}"));
    assert!(
        place.is_some_and(|p| p.contains("test_small")),
        "{:?}",
        now.place
    );
}

#[test]
fn the_lead_switches_give_the_female_lead_and_the_longest_name() {
    let (mut h, _) = preview();
    h.act(Action::Confirm);
    let lead = |h: &Core| screen(h).playing().map(|s| s.player().lead().clone());
    assert_eq!(
        lead(&h).map(|l| (l.gender, l.name)),
        Some((LeadGender::Male, DEFAULT_NAME.into()))
    );
    // Female, then the long name: the scene plays again each time.
    act(
        &mut h,
        &[
            Action::Cancel,
            Action::CursorRight,
            Action::CursorDown,
            Action::Confirm,
        ],
    );
    act(&mut h, &[Action::CursorDown, Action::Confirm]);
    let lead = lead(&h).unwrap_or_else(|| panic!("nothing plays"));
    assert_eq!(
        (lead.gender, lead.name.as_str()),
        (LeadGender::Female, LONG_NAME)
    );
    assert_eq!(LONG_NAME.chars().count(), 12);
}

#[test]
fn a_saved_change_plays_on_the_next_frame_from_the_same_text_box() {
    let (mut h, folder) = preview();
    h.act(Action::Confirm);
    act(&mut h, &[Action::Confirm, Action::Confirm]);
    assert_eq!(shown(&h), "You're late.");
    let reworded = TRIAL.replace("You're late.", "You're late again.");
    h.change_file(folder, "trial.dlg", reworded.into_bytes());
    h.wait(FRAME_DT);
    assert_eq!(shown(&h), "You're late again.");
    assert_eq!(view(&h).playing.map(|p| p.text_box), Some(1));
    assert!(!h.ctx().debug_tools, "as a writer's optimized build");
}

#[test]
fn a_broken_file_shows_its_problem_and_keeps_the_last_good_scene_playing() {
    let (mut h, folder) = preview();
    h.act(Action::Confirm);
    act(&mut h, &[Action::Confirm, Action::Confirm]);
    // A parse error: the block isn't closed.
    let broken = TRIAL.replace("@endif\n", "");
    h.change_file(folder, "trial.dlg", broken.into_bytes());
    h.wait(FRAME_DT);
    assert_eq!(shown(&h), "You're late.", "the scene plays on");
    let problem = view(&h)
        .problems
        .first()
        .cloned()
        .unwrap_or_else(|| panic!("no problem"));
    assert_eq!(problem.file, "trial.dlg");
    assert!(problem.line.is_some(), "{problem:?}");
    // One of the game's checks: a speaker who is no character.
    let stranger = TRIAL.replace("test_knight: You're late.", "nobody: You're late.");
    h.change_file(folder, "trial.dlg", stranger.into_bytes());
    h.wait(FRAME_DT);
    let problems = view(&h).problems;
    let unknown = problems
        .iter()
        .find(|p| p.message == "unknown character \"nobody\"");
    let at = unknown.map(|p| (p.file.as_str(), p.line));
    assert_eq!(at, Some(("trial.dlg", Some(8))), "{problems:?}");
    // Mended, the problems go.
    h.change_file(folder, "trial.dlg", TRIAL.as_bytes().to_vec());
    h.wait(FRAME_DT);
    assert_eq!(view(&h).problems, []);
}

#[test]
fn unit_info_opens_the_names_and_the_faces_and_closes_them() {
    let (mut h, _) = preview();
    h.act(Action::Info);
    let page = game_view(&h).reference.unwrap_or_else(|| panic!("no page"));
    let entry = |a: &str| {
        page.rows.iter().find_map(|r| match r {
            ReferenceRow::Entry(token, what) if token == a => Some(what.clone()),
            _ => None,
        })
    };
    let names = &h.ctx().game.content.names.names;
    let (id, name) = names.iter().next().unwrap_or_else(|| panic!("no names"));
    assert_eq!(entry(&format!("{{n:{id}}}")).as_ref(), Some(name));
    let lord = entry("test_lord").unwrap_or_default();
    assert!(lord.contains("neutral") && lord.contains("sad"), "{lord}");
    assert!(entry("lead").is_some_and(|e| e.contains("neutral")));
    // The cursor scrolls; Cancel closes it, Unit info too.
    act(
        &mut h,
        &[Action::CursorDown, Action::CursorDown, Action::CursorUp],
    );
    assert_eq!(game_view(&h).reference.map(|r| r.first), Some(1));
    h.act(Action::Cancel);
    assert!(game_view(&h).reference.is_none());
    act(&mut h, &[Action::Info, Action::Info]);
    assert!(game_view(&h).reference.is_none());
    assert_eq!(view(&h).tab, Tab::Scenes, "the keys went to the page");
    let declared = screen(&h).actions().unwrap_or_default();
    assert!(declared.contains(&Action::Info));
    assert!(h.screen_mut::<StoryPreview>().is_some());
}

/// The scene `id` of the Chapter 1 script, as written there.
fn ch01_scene(id: &str) -> String {
    let file = trpg_content::bundle::file("dialogue/ch01.dlg")
        .unwrap_or_else(|| panic!("no Chapter 1 script: `cargo shuyi private-assets`"));
    let start = file
        .find(&format!("@scene {id}\n"))
        .unwrap_or_else(|| panic!("no scene {id}"));
    let end = file[start..]
        .find("\n@end\n")
        .map_or(file.len(), |e| start + e + 5);
    format!("{}\n", &file[start..end])
}

#[test]
fn a_trial_packet_of_two_scenes_plays_both_with_no_problem_about_the_others() {
    if skip_without_the_game() {
        return;
    }
    let packet = [
        ("ch01_intro.dlg", ch01_scene("ch01_intro")),
        ("ch01_prebattle.dlg", ch01_scene("ch01_prebattle")),
    ];
    let files: Vec<(&str, &str)> = packet.iter().map(|(f, t)| (*f, t.as_str())).collect();
    let (mut h, _) = preview_of(&files);
    assert_eq!(view(&h).problems, []);
    let ids: Vec<String> = view(&h).scenes.into_iter().map(|s| s.id).collect();
    assert_eq!(ids, ["ch01_intro", "ch01_prebattle"]);
    assert!(screen(&h).preview().game().scenes_not_here() > 0);
    let note = game_view(&h).only_here;
    assert_eq!(
        note.as_deref(),
        Some("Only the scenes in this folder are checked")
    );
    // Both play.
    h.act(Action::Confirm);
    assert_ne!(shown(&h), "");
    act(
        &mut h,
        &[Action::Cancel, Action::CursorDown, Action::Confirm],
    );
    assert_eq!(
        view(&h).playing.map(|p| p.scene),
        Some("ch01_prebattle".to_owned())
    );
    assert_ne!(shown(&h), "");
    // A battle's scene is drawn over its battle's map.
    assert!(screen(&h).playing().is_some_and(|s| s.battle().is_some()));
}

#[test]
fn every_script_of_the_game_plays_with_no_problem_and_every_rule_checked() {
    let paths = trpg_content::bundle::files_in("dialogue");
    let files: Vec<(&str, &str)> = paths
        .into_iter()
        .filter(|p| p.ends_with(trpg_content::dialogue::DIALOGUE_EXTENSION))
        .filter_map(|p| Some((p.strip_prefix("dialogue/")?, trpg_content::bundle::file(p)?)))
        .collect();
    assert!(files.len() > 5, "{files:?}");
    let (h, _) = preview_of(&files);
    assert_eq!(view(&h).problems, []);
    assert_eq!(screen(&h).preview().game().scenes_not_here(), 0);
    assert_eq!(game_view(&h).only_here, None);
}

#[test]
fn a_line_with_a_stand_in_shows_the_leads_name() {
    let (mut h, _) = preview();
    h.act(Action::Confirm);
    act(
        &mut h,
        &[
            Action::Confirm,
            Action::Confirm,
            Action::Confirm,
            Action::Confirm,
        ],
    );
    assert_eq!(shown(&h), format!("Let's go, {DEFAULT_NAME}."));
}

#[test]
fn two_problems_on_one_line_are_each_listed_once() {
    // Two unknown tokens: the format's checks and the game's find both;
    // the game's list leaves out what the format's already have.
    let broken = TRIAL.replace("The road goes on.", "The {nope} road {nada} goes on.");
    let (h, _) = preview_of(&[("trial.dlg", &broken)]);
    let problems: Vec<String> = view(&h).problems.into_iter().map(|p| p.message).collect();
    let unknown = |token: &str| {
        let message = format!("unknown token \"{{{token}}}\"");
        problems.iter().filter(|m| **m == message).count()
    };
    assert_eq!((unknown("nope"), unknown("nada")), (1, 1), "{problems:?}");
}
