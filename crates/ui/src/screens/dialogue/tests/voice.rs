//! The dialogue screen's voices (ticket 0720): the test scene (`test.dlg`)
//! with 0238's test manifest, a clip for every line.

use trpg_content::{LineId, Variant};

use super::*;
use crate::debug::TEST_SCENE;
use crate::screen::tests::test_voices;

/// A context whose every line of the test scene has a clip, and the
/// test scene, at Instant text speed unless `typing`.
fn voiced(typing: bool) -> (Ctx, Scene) {
    let mut c = ctx();
    let count = c.game.content.dialogue.scenes[TEST_SCENE].lines().len();
    let (manifest, _) = test_voices(&c, &(0..count).collect::<Vec<_>>());
    c.set_voice_manifest(&manifest);
    let speed = if typing {
        TextSpeed::Normal
    } else {
        TextSpeed::Instant
    };
    c.change_settings(|s| s.text_speed = speed).unwrap();
    let scene = c.game.content.dialogue.scenes[TEST_SCENE].clone();
    (c, scene)
}

/// The test scene's screen, as the game shows it.
fn test_screen(c: &Ctx, scene: &Scene) -> DialogueScreen {
    DialogueScreen::told(c, scene, c.game.lead.clone(), &Present::Everyone)
}

/// The id of the test scene's line by `speaker` that starts with `text`.
fn line(scene: &Scene, speaker: &str, text: &str) -> LineId {
    let lines = scene.lines();
    let found = lines
        .iter()
        .find(|l| l.speaker == speaker && l.text.starts_with(text));
    found.expect("the test scene has the line").id.clone()
}

/// The voice requests queued since the last call.
fn voices(c: &mut Ctx) -> Vec<AudioRequest> {
    let voice = |r: &AudioRequest| {
        matches!(
            r,
            AudioRequest::PlayVoice { .. }
                | AudioRequest::StopVoice
                | AudioRequest::PreloadVoices { .. }
        )
    };
    c.audio.take().into_iter().filter(voice).collect()
}

fn play(line: &LineId) -> AudioRequest {
    AudioRequest::PlayVoice {
        line: line.clone(),
        variant: Variant::None,
    }
}

/// The lines asked for in `requests`, in order.
fn played(requests: &[AudioRequest]) -> Vec<LineId> {
    let line = |r: &AudioRequest| match r {
        AudioRequest::PlayVoice { line, .. } => Some(line.clone()),
        _ => None,
    };
    requests.iter().filter_map(line).collect()
}

/// Reads the scene to its end, picking the first reply at the choice:
/// every voice request on the way, and the lines whose boxes came up.
fn read_through(s: &mut DialogueScreen, c: &mut Ctx) -> (Vec<AudioRequest>, Vec<LineId>) {
    let mut requests = Vec::new();
    let mut boxes = Vec::new();
    let mut shown = None;
    for _ in 0..200 {
        let transition = press(s, c, Action::Confirm);
        requests.extend(voices(c));
        let line = s.player().line().cloned();
        if line != shown && !s.player().is_choosing() {
            boxes.extend(line.clone());
        }
        shown = line;
        if matches!(transition, Transition::Pop) {
            return (requests, boxes);
        }
    }
    panic!("the scene never ended");
}

/// Acceptance: each voiced line asks for its clip once, as its box comes
/// up, and moving on stops it before the next.
#[test]
fn each_line_plays_its_voice_once_and_moving_on_stops_it_first() {
    let (mut c, scene) = voiced(false);
    let mut s = test_screen(&c, &scene);
    let first = s.player().line().cloned().unwrap();
    s.update(&mut c, &frame(&[], 0.0, false));
    let start = voices(&mut c);
    assert_eq!(start[0], play(&first));
    assert!(matches!(start[1], AudioRequest::PreloadVoices { .. }));
    let (requests, boxes) = read_through(&mut s, &mut c);
    let expected: Vec<_> = boxes.into_iter().filter(|l| c.has_voice(l)).collect();
    assert!(expected.len() >= 5, "{expected:?}");
    assert_eq!(played(&requests), expected);
    // No voice is asked for while the last one plays (a choice stops
    // the line it answers, so its reaction's needs no second stop).
    let mut playing = true;
    for request in &requests {
        match request {
            AudioRequest::PlayVoice { .. } => {
                assert!(!playing, "{requests:?}");
                playing = true;
            }
            AudioRequest::StopVoice => playing = false,
            _ => {}
        }
    }
    // The last voice stops as the screen goes.
    assert_eq!(requests.last(), Some(&AudioRequest::StopVoice));
}

/// The scene's start tells `app` the next three lines, and the first
/// lines of each reply's reaction at a choice.
#[test]
fn the_lines_that_can_come_next_are_preloaded() {
    let (mut c, scene) = voiced(false);
    let mut s = test_screen(&c, &scene);
    s.update(&mut c, &frame(&[], 0.0, false));
    let preloaded = |requests: Vec<AudioRequest>| match requests.last() {
        Some(AudioRequest::PreloadVoices { lines }) => {
            lines.iter().map(|(l, _)| l.clone()).collect::<Vec<_>>()
        }
        other => panic!("no preload: {other:?}"),
    };
    let next = [
        line(&scene, "test_knight", "You're late."),
        line(&scene, "test_lord", "Better late"),
        line(&scene, "test_knight", "Than never."),
    ];
    assert_eq!(preloaded(voices(&mut c)), next);
    // At the choice: the first line of each reply's reaction.
    while !s.player().is_choosing() {
        press(&mut s, &mut c, Action::Confirm);
    }
    let reactions = [
        line(&scene, "test_archer", "You do."),
        line(&scene, "test_archer", "I carry my own"),
        line(&scene, "test_archer", "Right. Moving."),
    ];
    assert_eq!(preloaded(voices(&mut c)), reactions);
}

/// Acceptance: skipping asks for no skipped line's voice and stops the
/// one playing.
#[test]
fn skipping_stops_the_voice_and_says_no_skipped_line() {
    let (mut c, scene) = voiced(false);
    let mut s = test_screen(&c, &scene);
    s.update(&mut c, &frame(&[], 0.0, false));
    voices(&mut c);
    press(&mut s, &mut c, Action::Cancel);
    press(&mut s, &mut c, Action::Confirm);
    assert!(s.player().is_choosing());
    let requests = voices(&mut c);
    assert_eq!(requests.first(), Some(&AudioRequest::StopVoice));
    assert_eq!(played(&requests), [] as [LineId; 0]);
}

/// Acceptance: with voices off, nothing is asked for.
#[test]
fn voices_off_asks_for_nothing() {
    let (mut c, scene) = voiced(false);
    c.change_settings(|s| s.voices = false).unwrap();
    // Turning them off stops any that plays.
    voices(&mut c);
    let mut s = test_screen(&c, &scene);
    s.update(&mut c, &frame(&[], 0.0, false));
    let (mut requests, _) = read_through(&mut s, &mut c);
    requests.extend(voices(&mut c));
    assert_eq!(requests, []);
}

/// Acceptance: narration has a voice.
#[test]
fn narration_is_voiced() {
    let (mut c, scene) = voiced(false);
    let mut s = test_screen(&c, &scene);
    assert!(s.player().current().narration);
    s.update(&mut c, &frame(&[], 0.0, false));
    let rain = line(
        &scene,
        trpg_content::dialogue::NARRATION_SPEAKER,
        "The rain",
    );
    assert_eq!(played(&voices(&mut c)), [rain]);
}

/// Acceptance: a character's speech has a voice.
#[test]
fn speech_is_voiced() {
    let (mut c, scene) = voiced(false);
    let mut s = test_screen(&c, &scene);
    s.update(&mut c, &frame(&[], 0.0, false));
    voices(&mut c);
    press(&mut s, &mut c, Action::Confirm);
    let late = line(&scene, "test_knight", "You're late.");
    assert_eq!(played(&voices(&mut c)), [late]);
}

/// Acceptance: the lead's scripted speech has a voice.
#[test]
fn the_leads_speech_is_voiced() {
    let (mut c, scene) = voiced(false);
    let mut s = test_screen(&c, &scene);
    let (requests, _) = read_through(&mut s, &mut c);
    let lead = line(&scene, LEAD_ID, "Let's move.");
    assert!(c.has_voice(&lead));
    assert!(played(&requests).contains(&lead), "{requests:?}");
}

/// Acceptance: a reply the player picks stays silent, though it has a
/// clip; its reaction is voiced.
#[test]
fn reply_text_is_never_voiced() {
    let (mut c, scene) = voiced(false);
    let replies: Vec<_> = scene
        .lines()
        .iter()
        .filter(|l| l.speaker == trpg_content::dialogue::REPLY_SPEAKER)
        .map(|l| l.id.clone())
        .collect();
    assert_eq!(replies.len(), 3);
    assert!(replies.iter().all(|r| c.has_voice(r)));
    let mut s = test_screen(&c, &scene);
    let (requests, _) = read_through(&mut s, &mut c);
    let said = played(&requests);
    assert!(said.iter().all(|l| !replies.contains(l)), "{said:?}");
    assert!(said.contains(&line(&scene, "test_archer", "You do.")));
    // Nor in the log: the cursor on a reply says nothing.
    let mut s = test_screen(&c, &scene);
    while !s.player().is_choosing() {
        press(&mut s, &mut c, Action::Confirm);
    }
    press(&mut s, &mut c, Action::Confirm);
    voices(&mut c);
    press(&mut s, &mut c, Action::DialogueLog);
    press(&mut s, &mut c, Action::CursorUp);
    assert_eq!(played(&voices(&mut c)), [] as [LineId; 0]);
}

/// Acceptance: the first press on a typing line fills in the box and
/// leaves the voice playing; the next goes on and stops it.
#[test]
fn the_press_that_fills_the_box_leaves_the_voice_playing() {
    let (mut c, scene) = voiced(true);
    let mut s = test_screen(&c, &scene);
    s.update(&mut c, &frame(&[], 0.0, false));
    voices(&mut c);
    assert!(!s.is_revealed());
    press(&mut s, &mut c, Action::Confirm);
    assert!(s.is_revealed());
    assert_eq!(voices(&mut c), []);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(voices(&mut c).first(), Some(&AudioRequest::StopVoice));
}

/// Acceptance (0729): the log's cursor says the line it moves onto,
/// unless *Voice on hover in the log* is off; a jump says the line as
/// its box comes up again.
#[test]
fn the_logs_cursor_and_jumps_say_their_lines() {
    let (mut c, scene) = voiced(false);
    let rain = line(
        &scene,
        trpg_content::dialogue::NARRATION_SPEAKER,
        "The rain",
    );
    let late = line(&scene, "test_knight", "You're late.");
    let mut s = test_screen(&c, &scene);
    s.update(&mut c, &frame(&[], 0.0, false));
    press(&mut s, &mut c, Action::Confirm);
    press(&mut s, &mut c, Action::Confirm);
    voices(&mut c);
    // Opening the log says nothing; moving onto a line says it.
    press(&mut s, &mut c, Action::DialogueLog);
    assert_eq!(voices(&mut c), []);
    press(&mut s, &mut c, Action::CursorUp);
    assert_eq!(voices(&mut c), [AudioRequest::StopVoice, play(&late)]);
    press(&mut s, &mut c, Action::CursorUp);
    assert_eq!(voices(&mut c), [AudioRequest::StopVoice, play(&rain)]);
    // The jump shows the box again: stopped, said, the next preloaded.
    press(&mut s, &mut c, Action::Confirm);
    let jumped = voices(&mut c);
    assert_eq!(jumped[..2], [AudioRequest::StopVoice, play(&rain)]);
    assert!(matches!(jumped[2], AudioRequest::PreloadVoices { .. }));
    // With the setting off, the cursor says nothing.
    c.change_settings(|s| s.game.log_voice = LogVoice::Off)
        .unwrap();
    press(&mut s, &mut c, Action::DialogueLog);
    press(&mut s, &mut c, Action::CursorDown);
    assert_eq!(voices(&mut c), []);
}

/// Showing a scene again (a rewind past it, a replay) says its lines as
/// the first time.
#[test]
fn a_scene_shown_again_says_its_lines_again() {
    let (mut c, scene) = voiced(false);
    let mut first = test_screen(&c, &scene);
    let (said, _) = read_through(&mut first, &mut c);
    let mut again = test_screen(&c, &scene);
    let (said_again, _) = read_through(&mut again, &mut c);
    assert_eq!(played(&said), played(&said_again));
}
