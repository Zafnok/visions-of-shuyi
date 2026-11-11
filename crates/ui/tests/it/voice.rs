//! Voice clips through the Harness (ticket 0238, ADR-0046): a line is
//! asked for only when it has a clip that may be played and voices are on.
//! No game screen plays a voice yet (0720), so the "Voice test" debug tool
//! stands in for one. The voices' settings, their Options rows and the
//! New Game question (0826) are here too.

use shuyi_ui::audio::loudness::loudness;
use trpg_content::{LineId, Variant};
use trpg_ui::AudioRequest;
use trpg_ui::ShuyiCtx;
use trpg_ui::audio::Gain;
use trpg_ui::audio::groups::{MASTER, MUSIC, VOICE};
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

/// The debug menu with "Voice test" focused (the last tool but four).
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

/// Each frame carries the voice volume setting for `app` (ticket 0826),
/// on the engine's loudness curve (engine #318): 0 is silent.
#[test]
fn frames_carry_the_voice_volume() {
    for (setting, gain) in [(50, loudness(50)), (0, Gain::SILENT), (100, Gain::FULL)] {
        let mut ctx = trpg_ui::Ctx::embedded().unwrap();
        ctx.change_settings(|s| s.volumes.set(VOICE, setting))
            .unwrap();
        let mut game = trpg_ui::game::start(ctx);
        let out = game.frame(&[], 0.0).gains.voice();
        assert_eq!(out, gain, "{setting}");
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
    // The old 80, on the engine's loudness curve (engine #318).
    assert_eq!(h.gain(VOICE), loudness(97));
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
    assert_eq!(h.gain(VOICE), loudness(87));
    h.keys("f").type_text("0").keys("Enter");
    assert_eq!(h.gain(VOICE), Gain::SILENT);
    // Voice on hover in the log, under the volume: off.
    h.keys("Down Left");
    assert_eq!(h.game().ctx().settings().game.log_voice, LogVoice::Off);
    // Kept (Cancel asks, two left is Keep), then a restart.
    h.keys("d Left Left f");
    let mut h = Harness::with_storage(h.into_storage());
    h.wait(trpg_ui::harness::FRAME_DT);
    let s = h.game().ctx().settings().clone();
    assert_eq!(
        (s.voices, s.volumes.get(VOICE), s.game.log_voice),
        (false, Some(0), LogVoice::Off)
    );
    assert_eq!(h.gain(VOICE), Gain::SILENT);
}

/// How far the music dips under a voice: `audio.ron`'s `ducks` rule
/// (ticket 0865), 6 dB on the loudness curve.
const DUCK_TO: u8 = 90;

/// The music volume these tests set: not full, so the dip is seen to be
/// a fraction of the player's own volume.
const MUSIC_VOLUME: u8 = 70;

/// The debug menu's Voice test, with the test scene's first line voiced,
/// each clip `seconds` long, the music at [`MUSIC_VOLUME`] and the master
/// volume full; then `settings` changed. Returns the voiced line.
fn ducking_setup(
    seconds: f32,
    settings: impl FnOnce(&mut trpg_ui::settings::Settings),
) -> (Harness, LineId) {
    let mut h = on_voice_test();
    let line = with_a_voice(&mut h);
    h.voice_length(seconds);
    let changed = h.ctx_mut().change_settings(|s| {
        s.volumes.set(MUSIC, MUSIC_VOLUME);
        s.volumes.set(MASTER, 100);
        settings(s);
    });
    assert!(changed.is_ok(), "{changed:?}");
    h.wait(0.0);
    assert_eq!(h.gain(MUSIC), loudness(MUSIC_VOLUME), "full before");
    (h, line)
}

/// Acceptance (0865): the music's gain goes down to `loudness(to)` of the
/// player's volume while a voiced line plays (over `attack_ms`), stays
/// there, and comes back over `release_ms` after the line ends.
#[test]
fn the_music_dips_under_a_voiced_line_and_comes_back() {
    let (mut h, line) = ducking_setup(1.0, |_| {});
    let full = loudness(MUSIC_VOLUME);
    let ducked = full.times(loudness(DUCK_TO));
    h.ctx_mut().play_voice(&line);
    h.wait(0.0);
    assert_eq!(h.voices(), [line.to_string()]);
    // Past the 150 ms attack, and still while the line plays.
    h.wait(0.3);
    assert_eq!(h.gain(MUSIC), ducked);
    h.wait(0.5);
    assert_eq!(h.gain(MUSIC), ducked, "down while the line plays");
    // The line ended at 1 s: partway back during the 600 ms release.
    h.wait(0.5);
    let back = h.gain(MUSIC);
    assert!(ducked < back && back < full, "{back:?}");
    // Past the release: full again.
    h.wait(0.5);
    assert_eq!(h.gain(MUSIC), full);
}

/// The music's gain at every frame of two seconds after the test line is
/// asked for.
fn music_gains_after_asking(h: &mut Harness, line: &LineId) -> Vec<Gain> {
    h.ctx_mut().play_voice(line);
    (0..120)
        .map(|_| h.frame(&[], trpg_ui::harness::FRAME_DT).gain(MUSIC))
        .collect()
}

/// Acceptance (0865): with voices off, the line is never played and the
/// music never moves.
#[test]
fn with_voices_off_the_music_doesnt_dip() {
    let (mut h, line) = ducking_setup(1.0, |s| s.voices = false);
    let gains = music_gains_after_asking(&mut h, &line);
    assert_eq!(h.voices(), [] as [String; 0]);
    assert!(
        gains.iter().all(|&g| g == loudness(MUSIC_VOLUME)),
        "{gains:?}"
    );
}

/// Acceptance (0865): a voice the player turned down to 0 plays, but
/// lowers nothing.
#[test]
fn with_the_voice_volume_at_0_the_music_doesnt_dip() {
    let (mut h, line) = ducking_setup(1.0, |s| s.volumes.set(VOICE, 0));
    let gains = music_gains_after_asking(&mut h, &line);
    assert_eq!(h.voices(), [line.to_string()], "the clip still plays");
    assert!(
        gains.iter().all(|&g| g == loudness(MUSIC_VOLUME)),
        "{gains:?}"
    );
}

/// Acceptance (0865): a line with no clip plays nothing and lowers nothing.
#[test]
fn a_line_with_no_clip_doesnt_dip_the_music() {
    let (mut h, _) = ducking_setup(1.0, |_| {});
    let unvoiced = lines(&h)[2].0.clone();
    let gains = music_gains_after_asking(&mut h, &unvoiced);
    assert_eq!(h.voices(), [] as [String; 0]);
    assert!(
        gains.iter().all(|&g| g == loudness(MUSIC_VOLUME)),
        "{gains:?}"
    );
}
