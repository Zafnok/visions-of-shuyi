//! A place's sound bed under the music (ticket 1332, `audio.md`,
//! *Ambience*): the Quick Battle's ambience, the test files' meadow,
//! through the headless harness, whose speaker holds the sounds playing.

use trpg_content::ambience::AMBIENCE;
use trpg_ui::audio::AudioRequest;
use trpg_ui::audio::groups::MUSIC;
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::battle::mode::Mode;

/// The bed the Quick Battle's meadow plays at noon.
const MEADOW: &str = "bed_summer_meadow";

/// The Quick Battle from the title, its banners closed.
fn quick_battle(setup: impl FnOnce(&mut trpg_ui::screen::Ctx)) -> Harness {
    let mut h = Harness::with_layout_and(RIGHT_HANDED, setup);
    h.keys("Down f Left f").keys("f f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h
}

/// Whether the bed is playing (the title's campfire may still be fading
/// out beside it).
fn playing(h: &Harness) -> bool {
    h.sounds_playing().iter().any(|s| s.cue == MEADOW)
}

/// The bed's handle while it plays.
fn bed_id(h: &Harness) -> Option<trpg_ui::audio::SoundId> {
    h.sounds_playing()
        .iter()
        .find(|s| s.cue == MEADOW)
        .map(|s| s.id)
}

/// The volumes the bed was set to since the audio was last cleared.
fn bed_volumes(h: &Harness) -> Vec<f32> {
    let log = h.audio_log();
    let set = log.iter().filter_map(|(_, r)| match r {
        AudioRequest::SoundVolume { volume, .. } => Some(*volume),
        _ => None,
    });
    set.collect()
}

/// Acceptance: the bed starts with the battle, from silence up to full
/// over half a second, and fades out with it as it closes.
#[test]
fn the_bed_starts_with_the_battle_and_fades_out_with_it() {
    let mut h = quick_battle(|_| {});
    assert!(playing(&h));
    let started = h.audio_log().into_iter().find_map(|(_, r)| match r {
        AudioRequest::StartSound { cue, volume, .. } if cue == MEADOW => Some(*volume),
        _ => None,
    });
    assert_eq!(started, Some(0.0), "from silence");
    h.wait(1.0);
    assert_eq!(bed_volumes(&h).last(), Some(&1.0), "full after the fade");
    // The map menu's Suspend, and yes: back at the title, the bed fading.
    h.clear_audio();
    h.keys("d Down Down Down Down f f");
    assert_eq!(h.screens(), ["title"]);
    h.wait(0.25);
    let fading = bed_volumes(&h);
    assert!(fading.iter().any(|&v| 0.0 < v && v < 1.0), "{fading:?}");
    h.wait(0.5);
    assert!(!playing(&h), "gone after the fade");
}

/// Acceptance: the bed fades to silence while a fight's stage shows and
/// comes back after it, the same sound all along (its loop goes on).
#[test]
fn the_bed_stops_while_the_fight_shows_and_is_back_after_it() {
    let mut h = quick_battle(|_| {});
    h.wait(1.0);
    let bed = bed_id(&h);
    assert!(bed.is_some());
    // The lord beside the brigand, Attack, and the fight.
    h.keys("f Right Right Right Up f").wait(0.5).keys("f");
    h.clear_audio();
    h.keys("f").wait(0.7);
    let battle = h.battle().map(|b| matches!(b.mode(), Mode::Combat(_)));
    assert_eq!(battle, Some(true), "the fight shows");
    assert_eq!(bed_volumes(&h).last(), Some(&0.0), "silent in the fight");
    assert_eq!(bed_id(&h), bed, "still looping");
    // Cancel skips the rest of the fight: the map, and the bed back.
    h.clear_audio();
    h.keys("d").wait(1.0);
    assert_eq!(bed_volumes(&h).last(), Some(&1.0), "back after the fight");
    assert_eq!(bed_id(&h), bed, "the same loop");
}

/// Acceptance: the Ambience volume starts 10 under the music's, and with
/// it at 0 no bed is started.
#[test]
fn the_ambience_volume_starts_10_under_the_music_and_at_0_starts_nothing() {
    let h = Harness::with_layout(RIGHT_HANDED);
    let ctx = h.game().ctx();
    assert_eq!(ctx.volume(AMBIENCE) + 10, ctx.volume(MUSIC));
    let h = quick_battle(|ctx| {
        let set = ctx.change_settings(|s| s.volumes.set(AMBIENCE, 0));
        assert!(set.is_ok(), "{set:?}");
    });
    assert!(!playing(&h));
    let started = h
        .audio_log()
        .into_iter()
        .filter(|(_, r)| r.cue() == Some(MEADOW));
    assert_eq!(started.count(), 0);
}

/// The bed a scene's `@ambience quiet` plays, beside the battle's meadow.
const QUIET: &str = "bed_quiet";

/// Ticket 1383: a scene over the battle's map (`@place battle`) that names
/// its own ambience plays that bed, not the battle's; `@ambience none`
/// stops it and the music goes on; the next `@place` plays the place's
/// bed again.
#[test]
fn a_scene_over_the_battle_plays_the_ambience_it_names() {
    use trpg_content::dialogue::{AmbienceLine, MusicLine, PlaceLine, Present, Scene, Stage, Step};
    use trpg_core::{LeadGender, LeadProfile, Pos};
    use trpg_ui::ShuyiCtx;
    use trpg_ui::map_view::Place;
    use trpg_ui::screens::DialogueScreen;

    let mut ctx = trpg_ui::Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_layout(RIGHT_HANDED);
    // A second bed: the meadow's file under another cue.
    let sounds = &mut ctx.game.content.audio.sounds;
    let meadow = sounds[MEADOW].clone();
    sounds.insert(QUIET.into(), meadow);
    let quiet = [("noon".to_owned(), QUIET.to_owned())].into();
    let table = &mut ctx.game.content.ambience;
    table.ambiences.insert("quiet".into(), quiet);
    let narrate = |text: &str| Step::Narrate {
        direction: None,
        text: text.into(),
        line: trpg_content::LineId::default(),
    };
    let fort = || {
        Step::Directive(Stage::Setting(PlaceLine::Battle {
            at: Pos::new(11, 5),
            bright: None,
        }))
    };
    let ambience = |id: Option<&str>| {
        Step::Directive(Stage::Ambience(
            id.map_or(AmbienceLine::None, |id| AmbienceLine::Bed(id.into())),
        ))
    };
    let scene = Scene {
        id: "red_company".into(),
        steps: vec![
            fort(),
            Step::Music(MusicLine::Cue("village".into())),
            ambience(Some("quiet")),
            narrate("Quiet."),
            ambience(None),
            narrate("Silence."),
            fort(),
            narrate("The field again."),
        ],
    };
    let field = Place::of_battle(&ctx.game.content, &ctx.game.content.battles["test"]);
    let lead = LeadProfile::new("Ellery", LeadGender::Male);
    let names = ctx.game.content.names.clone();
    let screen = DialogueScreen::new(&scene, lead, names, &Present::Everyone).with_battle(field);
    let mut h = Harness::from_game(trpg_ui::game::new(ctx, Box::new(screen)));
    let cues = |h: &Harness| -> Vec<String> {
        let playing = h.sounds_playing().iter();
        playing.map(|s| s.cue.clone()).collect()
    };
    h.wait(1.0);
    assert_eq!(cues(&h), [QUIET], "the scene's, not the battle's");
    h.clear_audio();
    h.keys("f f").wait(1.0);
    assert_eq!(cues(&h), [] as [&str; 0], "no bed");
    let log = h.audio_log();
    let music = log.iter().filter(|(_, r)| {
        matches!(
            r,
            AudioRequest::StopMusic | AudioRequest::StopMusicWith { .. }
        )
    });
    assert_eq!(music.count(), 0, "the music goes on: {log:?}");
    h.keys("f f").wait(1.0);
    assert_eq!(cues(&h), [MEADOW], "the battle's again");
}
