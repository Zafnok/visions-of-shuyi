use super::*;

use crate::audio::AudioRequest;
use crate::screen::tests::ctx;

const FRAME: f32 = 0.1;

/// The volumes the bed was set to in `requests`.
fn volumes(requests: &[AudioRequest]) -> Vec<f32> {
    let set = requests.iter().filter_map(|r| match r {
        AudioRequest::SoundVolume { volume, .. } => Some(*volume),
        _ => None,
    });
    set.collect()
}

/// The cues started in `requests`, each with its volume.
fn starts(requests: &[AudioRequest]) -> Vec<(String, f32)> {
    let started = requests.iter().filter_map(|r| match r {
        AudioRequest::StartSound { cue, volume, .. } => Some((cue.clone(), *volume)),
        _ => None,
    });
    started.collect()
}

/// `frames` frames of `owner` asking for `cue`.
fn frames(ctx: &mut Ctx, owner: &'static str, cue: Option<&str>, hushed: bool, frames: u32) {
    for _ in 0..frames {
        play(ctx, owner, cue, hushed, FRAME);
    }
}

#[test]
fn a_bed_starts_silent_fades_in_and_isnt_started_again() {
    let mut c = ctx();
    frames(&mut c, "a", Some("day"), false, 7);
    let asked = c.audio.take();
    assert_eq!(starts(&asked), [("day".to_owned(), 0.0)]);
    let v = volumes(&asked);
    assert_eq!(v.len(), 5, "full after half a second: {v:?}");
    assert!(v.windows(2).all(|w| w[0] < w[1]), "{v:?}");
    assert_eq!(v.last(), Some(&1.0));
    assert_eq!(c.game.ambience.cue(), Some("day"));
}

#[test]
fn hushed_it_fades_to_silence_and_back_without_restarting() {
    let mut c = ctx();
    frames(&mut c, "a", Some("day"), false, 6);
    c.audio.take();
    frames(&mut c, "a", Some("day"), true, 6);
    let asked = c.audio.take();
    assert_eq!(starts(&asked), []);
    assert_eq!(volumes(&asked).last(), Some(&0.0));
    frames(&mut c, "a", Some("day"), false, 6);
    let asked = c.audio.take();
    assert_eq!(starts(&asked), []);
    assert_eq!(volumes(&asked).last(), Some(&1.0));
}

#[test]
fn another_bed_fades_the_one_playing_out_and_none_fades_it_out() {
    let mut c = ctx();
    frames(&mut c, "a", Some("day"), false, 6);
    let day = c.audio.take();
    frames(&mut c, "b", Some("night"), false, 1);
    assert_eq!(starts(&c.audio.take()), [("night".to_owned(), 0.0)]);
    // The day bed fades over the half second, then stops.
    c.audio.advance(0.6);
    let stopped = c.audio.take();
    let day_id = day.iter().find_map(|r| match r {
        AudioRequest::StartSound { id, .. } => Some(*id),
        _ => None,
    });
    assert!(stopped.contains(&AudioRequest::StopSound {
        id: day_id.unwrap()
    }));
    frames(&mut c, "b", None, false, 1);
    assert_eq!(c.game.ambience.cue(), None);
}

#[test]
fn leaving_fades_out_only_the_bed_the_screen_started() {
    let mut c = ctx();
    frames(&mut c, "battle", Some("day"), false, 6);
    // A scene over the battle at the same place keeps the battle's bed,
    // and its leaving doesn't stop it.
    frames(&mut c, "dialogue", Some("day"), false, 3);
    leave(&mut c, "dialogue");
    assert_eq!(c.game.ambience.cue(), Some("day"));
    leave(&mut c, "battle");
    assert_eq!(c.game.ambience.cue(), None);
}

#[test]
fn with_the_ambience_volume_at_0_no_bed_starts() {
    let mut c = ctx();
    assert!(c.preview_volume(AMBIENCE, 0));
    frames(&mut c, "a", Some("day"), false, 3);
    assert_eq!(starts(&c.audio.take()), []);
    assert_eq!(c.game.ambience.cue(), None);
}

#[test]
fn a_place_without_a_bed_of_its_light_plays_its_noon_one() {
    let mut c = ctx();
    let table = &mut c.game.content.ambience;
    let beds = [("noon", "day"), ("night", "crickets")];
    let beds = beds.map(|(l, b)| (l.to_owned(), b.to_owned())).into();
    table.ambiences.insert("field".into(), beds);
    assert_eq!(bed(&c, Some("field"), "night").as_deref(), Some("crickets"));
    assert_eq!(bed(&c, Some("field"), "dusk").as_deref(), Some("day"));
    assert_eq!(bed(&c, None, "noon"), None);
}

/// The Playdate's skins turn the beds off until the bench says they run
/// well there.
#[test]
fn the_playdate_plays_no_bed() {
    use crate::screen::ShuyiCtx;
    let mut c = ctx();
    c.use_playdate_skins();
    frames(&mut c, "a", Some("day"), false, 3);
    assert_eq!(starts(&c.audio.take()), []);
    assert_eq!(c.game.ambience.cue(), None);
}
