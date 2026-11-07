//! Voice clips through the Harness (ticket 0238, ADR-0046): a line is
//! asked for only when it has a clip that may be played and voices are on.
//! No game screen plays a voice yet (0720), so the "Voice test" debug tool
//! stands in for one. The voices' settings, their Options rows and the
//! New Game question (0826) are here too.

use trpg_content::{LineId, Variant};
use trpg_ui::AudioRequest;
use trpg_ui::ShuyiCtx;
use trpg_ui::debug::TEST_SCENE;
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::options::{Row, shown_rows};
use trpg_ui::settings::LogVoice;
use trpg_ui::storage::Storage;

/// `(id, text)` of the test scene's first four lines, none of which has a
/// token.
fn lines(h: &Harness) -> Vec<(LineId, String)> {
    let scene = &h.game().ctx().game.content.dialogue.scenes[TEST_SCENE];
    let lines = scene.lines();
    let pair = |l: &trpg_content::dialogue::Line| (l.id.clone(), l.text.to_owned());
    lines.iter().take(4).map(pair).collect()
}

/// A manifest with one clip per `(line, spoken)`.
fn manifest(clips: &[(&LineId, &str)]) -> String {
    let clip = |(line, spoken): &(&LineId, &str)| {
        format!(
            "(line: \"{line}\", spoken: \"{spoken}\", voice: \"v\", \
             made_by: Generated(tool: \"t\", model: \"m\", date: \"2026-10-03\")),\n"
        )
    };
    format!(
        "(clips: [\n{}])",
        clips.iter().map(clip).collect::<String>()
    )
}

/// The debug menu with "Voice test" focused (the last tool but three).
fn on_voice_test() -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.keys("F2 Up Up Up Up Up Up");
    assert_eq!(h.top_screen(), "debug_menu");
    h
}

fn play(line: &LineId) -> AudioRequest {
    AudioRequest::PlayVoice {
        line: line.clone(),
        variant: Variant::None,
    }
}

/// The voice requests of the run so far.
fn voice_requests(h: &Harness) -> Vec<AudioRequest> {
    let voice = |r: &AudioRequest| !matches!(r, AudioRequest::PlaySound { .. });
    h.audio_requests().into_iter().filter(voice).collect()
}

/// Acceptance: a line with a clip queues `PlayVoice`; a line without one,
/// with a stale one, or with voices off, queues nothing.
#[test]
fn only_lines_with_a_playable_clip_are_asked_for() {
    let mut h = on_voice_test();
    let lines = lines(&h);
    // No manifest: the tool has nothing to say.
    h.clear_audio().keys("f");
    assert_eq!(h.voices(), [] as [std::string::String; 0]);
    // The first line has a clip, the second a stale one, the third none,
    // the fourth a clip.
    let source = manifest(&[
        (&lines[0].0, &lines[0].1),
        (&lines[1].0, "What it said before a rewrite."),
        (&lines[3].0, &lines[3].1),
    ]);
    h.ctx_mut().set_voice_manifest(&source);
    assert_eq!(h.ctx_mut().take_warnings(), [] as [std::string::String; 0]);
    h.clear_audio().keys("f");
    let preload = AudioRequest::PreloadVoices {
        lines: vec![
            (lines[0].0.clone(), Variant::None),
            (lines[3].0.clone(), Variant::None),
        ],
    };
    assert_eq!(voice_requests(&h), [preload, play(&lines[0].0)]);
    h.clear_audio().keys("f");
    assert_eq!(voice_requests(&h), [play(&lines[3].0)]);
    assert_eq!(h.voices(), [lines[3].0.to_string()]);
    // Asked for directly, as a screen would: the stale and the missing
    // clip stay silent.
    h.clear_audio();
    for (line, _) in &lines {
        h.ctx_mut().play_voice(line);
    }
    h.wait(0.0);
    assert_eq!(h.voices(), [lines[0].0.to_string(), lines[3].0.to_string()]);
    // Voices off: nothing at all.
    h.ctx_mut().change_settings(|s| s.voices = false).unwrap();
    h.wait(0.0).clear_audio().keys("f");
    h.ctx_mut().play_voice(&lines[0].0);
    h.wait(0.0);
    assert_eq!(
        voice_requests(&h),
        [] as [shuyi_ui::audio::AudioRequest<trpg_content::Variant>; 0]
    );
    assert_eq!(h.sounds(), ["menu_select"]);
}

#[test]
fn stop_voice_reaches_the_frame_output() {
    let mut h = on_voice_test();
    h.clear_audio();
    h.ctx_mut().stop_voice();
    h.wait(0.0);
    assert_eq!(voice_requests(&h), [AudioRequest::StopVoice]);
    assert!(
        h.music_commands().is_empty(),
        "voices leave the music alone"
    );
}

/// Each frame carries the voice volume setting for `app` (ticket 0826):
/// 0 is silent.
#[test]
fn frames_carry_the_voice_volume() {
    for (setting, volume) in [(50, 0.5), (0, 0.0), (100, 1.0)] {
        let mut ctx = trpg_ui::Ctx::embedded().unwrap();
        ctx.change_settings(|s| s.voice_volume = setting).unwrap();
        let mut game = trpg_ui::game::start(ctx);
        let out = game.frame(&[], 0.0).voice_volume;
        assert!((out - volume).abs() < f32::EPSILON, "{setting}: {out}");
    }
}

/// The test scene's first line has a clip: what a build with a `voice/`
/// folder hands over at launch.
fn with_a_voice(h: &mut Harness) -> LineId {
    let lines = lines(h);
    let source = manifest(&[(&lines[0].0, &lines[0].1)]);
    h.ctx_mut().set_voice_manifest(&source);
    lines[0].0.clone()
}

/// From the title: New Game, Classic, "Not familiar".
fn past_familiarity(h: &mut Harness) {
    h.keys("f f f");
}

/// Acceptance (0826): with voice clips, New Game asks "Play with
/// AI-generated voices?" after the familiarity question, `Yes`
/// highlighted; the answer is the Voices setting, saved; Cancel goes back
/// a question, and the lead screen's Cancel comes back to it.
#[test]
fn the_new_game_asks_about_voices_and_the_answer_is_the_setting() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    with_a_voice(&mut h);
    past_familiarity(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "voices_question"]);
    insta::assert_snapshot!(h.snapshot());
    // No: voices off, on to the lead.
    h.keys("Down f");
    assert_eq!(h.top_screen(), "lead_select");
    assert!(!h.game().ctx().settings().voices);
    let saved = h.game().ctx().storage.read(trpg_ui::settings::SETTINGS_KEY);
    let saved = saved.ok().flatten().unwrap_or_default();
    assert!(saved.contains("voices: Some(false),"), "{saved}");
    // Back to the question, then Yes (the highlighted answer): on again.
    h.keys("d");
    assert_eq!(h.top_screen(), "voices_question");
    h.keys("f");
    assert_eq!(h.top_screen(), "lead_select");
    assert!(h.game().ctx().settings().voices);
    // Cancel on the question goes back to the familiarity question.
    h.keys("d d");
    assert_eq!(h.screens(), ["title", "game_flow", "familiarity"]);
}

/// Acceptance (0826): with no voice clips found, New Game doesn't ask,
/// and the lead screen's Cancel goes back to the familiarity question.
#[test]
fn without_voice_clips_the_new_game_doesnt_ask() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    past_familiarity(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "lead_select"]);
    h.keys("d");
    assert_eq!(h.top_screen(), "familiarity");
    assert!(h.game().ctx().settings().voices, "on until answered");
}

/// Options from the title, focused on `row`.
fn options_on(row: Row) -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.keys("Down Down f");
    assert_eq!(h.screens(), ["title", "options"]);
    let rows = shown_rows(h.game().ctx());
    let at = rows.iter().position(|&r| r == row);
    let at = at.unwrap_or_else(|| panic!("no {row:?} row in {rows:?}"));
    h.keys(&vec!["Down"; at].join(" "));
    h
}

/// Acceptance (0826): the Voices row turns voices off (stopping the clip
/// playing; none is asked for after) and on; the voice volume changes
/// what `app` plays at, 0 silent; the log's row turns hover voices off;
/// all three survive a restart.
#[test]
fn the_voice_rows_switch_set_the_volume_and_persist() {
    let mut h = options_on(Row::Voices);
    let line = with_a_voice(&mut h);
    let close = |a: f32, b: f32| (a - b).abs() < 1e-6;
    assert!(close(h.voice_volume(), 0.8), "{}", h.voice_volume());
    // Off: the voice playing stops, and nothing plays.
    h.ctx_mut().play_voice(&line);
    h.wait(0.0).clear_audio().keys("Left");
    assert!(!h.game().ctx().settings().voices);
    assert_eq!(voice_requests(&h), [AudioRequest::StopVoice]);
    h.clear_audio();
    h.ctx_mut().play_voice(&line);
    h.wait(0.0);
    assert_eq!(voice_requests(&h), [] as [AudioRequest; 0]);
    // The volume: the slider, then 0 by the box.
    h.keys("Down Left Left");
    assert!(close(h.voice_volume(), 0.7), "{}", h.voice_volume());
    h.keys("f").type_text("0").keys("Enter");
    assert!(close(h.voice_volume(), 0.0), "{}", h.voice_volume());
    // Voice on hover in the log, under the volume: off.
    h.keys("Down Left");
    assert_eq!(h.game().ctx().settings().game.log_voice, LogVoice::Off);
    // Kept (Cancel asks, two left is Keep), then a restart.
    h.keys("d Left Left f");
    let mut h = Harness::with_storage(h.into_storage());
    h.wait(trpg_ui::harness::FRAME_DT);
    let s = h.game().ctx().settings().clone();
    assert_eq!(
        (s.voices, s.voice_volume, s.game.log_voice),
        (false, 0, LogVoice::Off)
    );
    assert!(close(h.voice_volume(), 0.0), "{}", h.voice_volume());
}
