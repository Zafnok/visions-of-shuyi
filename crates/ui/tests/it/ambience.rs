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
