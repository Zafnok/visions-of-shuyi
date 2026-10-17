//! The SDK's sound (ticket 1047): [`Speaker`] on the SDK's sample players
//! and file players, through `shim/shim.c`. What plays when is
//! `crate::audio`'s.

use core::ffi::{c_char, c_float, c_int, c_void};

use crate::audio::{Speaker, VOICES};
use crate::text;

unsafe extern "C" {
    fn pd_sample_load(path: *const c_char) -> *mut c_void;
    fn pd_sampleplayer_new() -> *mut c_void;
    fn pd_sampleplayer_play(player: *mut c_void, sample: *mut c_void, volume: c_float);
    fn pd_sampleplayer_is_playing(player: *mut c_void) -> c_int;
    fn pd_fileplayer_open(path: *const c_char) -> *mut c_void;
    fn pd_fileplayer_play(player: *mut c_void, looped: c_int) -> c_int;
    fn pd_fileplayer_set_volume(player: *mut c_void, volume: c_float);
    fn pd_fileplayer_pause(player: *mut c_void);
    fn pd_fileplayer_is_playing(player: *mut c_void) -> c_int;
    fn pd_fileplayer_offset(player: *mut c_void) -> c_float;
    fn pd_fileplayer_did_underrun(player: *mut c_void) -> c_int;
    fn pd_fileplayer_close(player: *mut c_void);
}

/// The SDK's speaker: [`VOICES`] sample players, made at the start.
pub struct Sdk {
    /// The sample players; a null one (the SDK had no memory) plays
    /// nothing.
    voices: [*mut c_void; VOICES],
}

/// A sound the SDK holds in memory, for the whole game.
pub struct Sample(*mut c_void);

/// A file player, open on its music file.
pub struct Track {
    player: *mut c_void,
    looped: bool,
}

impl Sdk {
    /// Makes the sample players.
    pub fn new() -> Self {
        Self {
            voices: core::array::from_fn(|_| unsafe { pd_sampleplayer_new() }),
        }
    }
}

impl Speaker for Sdk {
    type Sample = Sample;
    type Track = Track;

    fn load_sample(&mut self, path: &str) -> Option<Sample> {
        let path = text::c_string(path);
        let sample = unsafe { pd_sample_load(path.as_ptr().cast()) };
        (!sample.is_null()).then_some(Sample(sample))
    }

    fn play_sample(&mut self, voice: usize, sample: &Sample, volume: f32) {
        if let Some(&player) = self.voices.get(voice).filter(|p| !p.is_null()) {
            // SAFETY: a sample player the SDK made, and a sample it loaded
            // that is never freed.
            unsafe { pd_sampleplayer_play(player, sample.0, volume) };
        }
    }

    fn voice_playing(&mut self, voice: usize) -> bool {
        match self.voices.get(voice).filter(|p| !p.is_null()) {
            Some(&player) => unsafe { pd_sampleplayer_is_playing(player) != 0 },
            None => false,
        }
    }

    fn open_track(&mut self, path: &str) -> Option<Track> {
        let path = text::c_string(path);
        let player = unsafe { pd_fileplayer_open(path.as_ptr().cast()) };
        (!player.is_null()).then_some(Track {
            player,
            looped: false,
        })
    }

    fn start_track(&mut self, track: &mut Track, volume: f32, looped: bool) -> bool {
        track.looped = looped;
        unsafe {
            pd_fileplayer_set_volume(track.player, volume);
            pd_fileplayer_play(track.player, c_int::from(looped)) != 0
        }
    }

    fn set_track_volume(&mut self, track: &mut Track, volume: f32) {
        unsafe { pd_fileplayer_set_volume(track.player, volume) };
    }

    fn pause_track(&mut self, track: &mut Track, paused: bool) {
        unsafe {
            if paused {
                pd_fileplayer_pause(track.player);
            } else {
                pd_fileplayer_play(track.player, c_int::from(track.looped));
            }
        }
    }

    fn track_position(&mut self, track: &mut Track) -> Option<f32> {
        let playing = unsafe { pd_fileplayer_is_playing(track.player) != 0 };
        playing.then(|| unsafe { pd_fileplayer_offset(track.player) })
    }

    fn track_underran(&mut self, track: &mut Track) -> bool {
        unsafe { pd_fileplayer_did_underrun(track.player) != 0 }
    }

    fn close_track(&mut self, track: Track) {
        // SAFETY: the player is not used again: `track` is consumed.
        unsafe { pd_fileplayer_close(track.player) };
    }
}
