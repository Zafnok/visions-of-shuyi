//! Which lines are kept before a scene starts (ticket 1293), on the test
//! chapter (`assets-test/chapters/test.ron`), its intro made plain and
//! voiced.

use super::*;
use core::fmt::Write as _;

use crate::AudioRequest;
use crate::screen::tests::ctx;
use crate::screen::{FrameInput, Screen};
use crate::screens::TitleScreen;

/// Puts scene `id` in `c`'s content as plain narration, one line per
/// text; its lines' ids and texts.
fn plain_scene(c: &mut Ctx, id: &str, texts: &[&str]) -> Vec<(LineId, String)> {
    let mut source = format!("@scene {id}\n");
    for t in texts {
        let _ = writeln!(source, "> {t}");
    }
    source.push_str("@end\n");
    let (parsed, errors) = trpg_content::dialogue::parse_dlg("t.dlg", &source);
    assert_eq!(errors, []);
    let scene = parsed.into_iter().next().map(|p| p.scene);
    let scene = scene.unwrap_or_else(|| panic!("no scene in {source}"));
    let lines = scene.lines();
    let lines = lines.iter().map(|l| (l.id.clone(), l.text.to_owned()));
    let lines = lines.collect();
    c.game.content.dialogue.scenes.insert(id.into(), scene);
    lines
}

/// The test context with the test chapter's intro four plain lines, a
/// chapter `test2` after it whose intro is four more, every line with a
/// clip; the two intros' line ids.
fn voiced() -> (Ctx, Vec<LineId>, Vec<LineId>) {
    let mut c = ctx();
    let one = plain_scene(&mut c, "test_intro", &["One.", "Two.", "Three.", "Four."]);
    let two = plain_scene(&mut c, "test_two", &["Five.", "Six.", "Seven.", "Eight."]);
    let chapters = &mut c.game.content.chapters;
    let mut next = chapters["test"].clone();
    next.id = "test2".into();
    next.intro_scenes = vec!["test_two".into()];
    next.next = None;
    chapters.insert("test2".into(), next);
    if let Some(test) = chapters.get_mut("test") {
        test.next = Some("test2".into());
    }
    let mut clips = String::new();
    for (line, text) in one.iter().chain(&two) {
        let _ = writeln!(
            clips,
            "(line: \"{line}\", spoken: \"{text}\", voice: \"v\", made_by: Recorded(actor: \"a\")),"
        );
    }
    c.set_voice_manifest(&format!("(clips: [\n{clips}])"));
    assert_eq!(c.take_warnings(), [] as [String; 0]);
    c.audio.take();
    let ids = |lines: Vec<(LineId, String)>| lines.into_iter().map(|(l, _)| l).collect();
    (c, ids(one), ids(two))
}

fn keep(lines: &[LineId]) -> AudioRequest {
    AudioRequest::KeepVoices {
        lines: lines
            .iter()
            .map(|l| (l.clone(), trpg_content::Variant::None))
            .collect(),
    }
}

/// The sets kept since the last call.
fn kept(c: &mut Ctx) -> Vec<AudioRequest> {
    let requests = c.audio.take().into_iter();
    requests
        .filter(|r| matches!(r, AudioRequest::KeepVoices { .. }))
        .collect()
}

/// A chapter save of New Game's campaign, its first chapter cleared.
fn first_chapter_cleared(c: &Ctx) -> SaveFile {
    let campaign = new_campaign(&c.game.content, GameMode::Classic, c.game.lead.clone());
    SaveFile::chapter_cleared(campaign)
}

#[test]
fn new_game_keeps_its_first_chapters_first_intro_lines() {
    let (c, one, _) = voiced();
    assert_eq!(opening_lines(&c, Start::NewGame), one[..3]);
}

#[test]
fn a_chapter_save_keeps_the_next_chapters_first_lines() {
    let (c, _, two) = voiced();
    let file = first_chapter_cleared(&c);
    assert_eq!(opening_lines(&c, Start::Save(&file)), two[..3]);
}

#[test]
fn a_save_that_starts_no_scene_keeps_nothing() {
    let (mut c, _, _) = voiced();
    // On the world map: the map is back, no scene.
    let campaign = first_chapter_cleared(&c).campaign;
    let on_map = SaveFile::world_map(campaign);
    assert_eq!(opening_lines(&c, Start::Save(&on_map)), [] as [LineId; 0]);
    // The last chapter cleared: "To be continued".
    if let Some(test) = c.game.content.chapters.get_mut("test") {
        test.next = None;
    }
    let file = first_chapter_cleared(&c);
    assert_eq!(opening_lines(&c, Start::Save(&file)), [] as [LineId; 0]);
}

#[test]
fn a_scene_with_nothing_to_say_for_the_army_is_passed_over() {
    let (mut c, _, two) = voiced();
    // A scene whose only line is for someone not in the army is skipped,
    // as the flow skips it.
    let src = "@scene test_kell\n@if test_absent\n> Never.\n@endif\n@end\n";
    let (parsed, errors) = trpg_content::dialogue::parse_dlg("t.dlg", src);
    assert_eq!(errors, []);
    let scene = parsed.into_iter().next().map(|p| p.scene);
    let scene = scene.unwrap_or_else(|| panic!("no scene in {src}"));
    c.game
        .content
        .dialogue
        .scenes
        .insert("test_kell".into(), scene);
    let ids = ["test_kell".to_owned(), "test_two".to_owned()];
    let campaign = first_chapter_cleared(&c).campaign;
    let first = first_told(&c, Some(&campaign), &ids).map(|s| s.first_lines());
    assert_eq!(first.as_deref(), Some(&two[..3]));
}

#[test]
fn the_set_is_sent_once_while_it_holds_and_again_when_it_changes() {
    let (mut c, one, two) = voiced();
    let mut voices = KeptVoices::default();
    assert!(voices.due(&c));
    voices.keep(one.clone(), &mut c);
    assert_eq!(kept(&mut c), [keep(&one)]);
    assert!(!voices.due(&c));
    // The same set: nothing new.
    voices.keep(one.clone(), &mut c);
    assert_eq!(kept(&mut c), []);
    // Lines twice and lines without a clip are left out.
    let mut more = two.clone();
    more.extend([two[0].clone(), LineId::new("test_00000000")]);
    voices.keep(more, &mut c);
    assert_eq!(kept(&mut c), [keep(&two)]);
    // Covered: sent again, even unchanged.
    voices.covered();
    voices.keep(two.clone(), &mut c);
    assert_eq!(kept(&mut c), [keep(&two)]);
    // Nothing to keep: the set is let go.
    voices.keep(Vec::new(), &mut c);
    assert_eq!(kept(&mut c), [keep(&[])]);
    // An empty set when none was kept is not sent.
    voices.covered();
    voices.keep(Vec::new(), &mut c);
    assert_eq!(kept(&mut c), []);
}

#[test]
fn with_voices_off_nothing_is_kept_and_the_set_is_due_once_on() {
    let (mut c, one, _) = voiced();
    c.change_settings(|s| s.voices = false).unwrap();
    c.audio.take();
    let mut voices = KeptVoices::default();
    assert!(!voices.due(&c));
    voices.keep(one.clone(), &mut c);
    assert_eq!(kept(&mut c), []);
    c.change_settings(|s| s.voices = true).unwrap();
    assert!(voices.due(&c));
    voices.keep(one.clone(), &mut c);
    assert_eq!(kept(&mut c), [keep(&one)]);
}

/// Acceptance (1293): the title's first frame keeps the opening scene's
/// first 3 lines; it sends them again only once back on top.
#[test]
fn the_title_keeps_new_games_first_lines() {
    let (mut c, one, _) = voiced();
    let mut title = TitleScreen::new(&c);
    let frame = || FrameInput::new(vec![], 0.0, vec![]);
    title.update(&mut c, &frame());
    assert_eq!(kept(&mut c), [keep(&one[..3])]);
    title.update(&mut c, &frame());
    assert_eq!(kept(&mut c), []);
    title.uncovered(&mut c, None);
    title.update(&mut c, &frame());
    assert_eq!(kept(&mut c), [keep(&one[..3])]);
}

/// What the flow keeps as it shows a screen that isn't a scene, by where
/// it is (CI's mutants, PR #757): New Game's opening while the game is
/// set up, the next chapter's at the save prompt, the chapter camp goes
/// on to, nothing before a battle.
#[test]
fn the_flow_keeps_the_scene_after_the_screen_it_shows() {
    use super::super::{FlowScreen, Stage};
    let (c, one, two) = voiced();
    let mut flow = FlowScreen::empty();
    flow.chapter = c.game.content.chapters.get("test").cloned();
    flow.campaign = Some(first_chapter_cleared(&c).campaign);
    let at = |flow: &mut FlowScreen, stage: Stage| {
        flow.stage = stage;
        flow.coming_lines(&c)
    };
    assert_eq!(at(&mut flow, Stage::Mode), one[..3]);
    assert_eq!(at(&mut flow, Stage::SavePrompt), two[..3]);
    assert_eq!(at(&mut flow, Stage::Slots(true)), two[..3]);
    assert_eq!(at(&mut flow, Stage::Battle), [] as [LineId; 0]);
    // Camp breaks into the chapter it was opened for, or the battle.
    assert_eq!(at(&mut flow, Stage::Camp), [] as [LineId; 0]);
    flow.after_camp = Some("test2".into());
    assert_eq!(at(&mut flow, Stage::Camp), two[..3]);
    // The last chapter cleared: no chapter, nothing at the save prompt.
    flow.chapter = flow.chapter.map(|mut ch| {
        ch.next = None;
        ch
    });
    assert_eq!(at(&mut flow, Stage::SavePrompt), [] as [LineId; 0]);
}

/// Lines are looked for only with voices on and clips found: a build
/// without clips (the Playdate) skips the work.
#[test]
fn lines_are_wanted_only_with_voices_on_and_clips_found() {
    let (mut c, _, _) = voiced();
    assert!(KeptVoices::wanted(&c));
    c.change_settings(|s| s.voices = false).unwrap();
    assert!(!KeptVoices::wanted(&c));
    let plain = ctx();
    assert!(plain.settings().voices && !plain.has_voices());
    assert!(!KeptVoices::wanted(&plain));
    assert!(!KeptVoices::default().due(&plain));
}
