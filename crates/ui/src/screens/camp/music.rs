//! Camp's music (ticket 0027, `docs/design/audio.md`, *Starting values*):
//! the `camp` pool's tracks **rotate** while camp is open. One plays to
//! its end and fades out, camp is quiet a moment, then the next one
//! starts. The first is picked at random each time camp opens; then the
//! order goes round the pool, so no track plays twice in a row.
//!
//! [`CampMusic`] is the schedule, by the game's clock: what to ask for,
//! and when. It plays nothing itself: the camp screen passes what it
//! asks for to [`Ctx::audio`](crate::screen::Ctx).
//!
//! The engine fades a track out only over the manifest's one fade
//! (`music_fade_ms`) and doesn't fade one in, so a track here stops that
//! long before its end and the next starts at full volume, not with
//! audio.md's 3 s fades: a workaround until the engine has both.

use crate::prelude::*;

/// What camp should ask for now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MusicCue {
    /// Play this music cue.
    Play(String),
    /// Let the music fade out.
    Stop,
}

/// Where the rotation is.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Phase {
    /// Track `index` plays until the clock reads `until`, when it starts
    /// to fade out.
    Playing { index: usize, until: f64 },
    /// Quiet until `until`, then track `index` plays.
    Quiet { index: usize, until: f64 },
}

/// The rotation of a pool's tracks (see the module docs).
#[derive(Debug, Clone, PartialEq)]
pub struct CampMusic {
    /// The pool's tracks, in order, each with how long it plays (s).
    tracks: Vec<(String, f64)>,
    /// How long the music takes to fade out (s).
    fade_s: f64,
    phase: Phase,
}

/// How long camp is quiet between two tracks, in seconds, after the fade
/// (audio.md, *Starting values*: 5 s, TUNABLE).
pub const QUIET_S: f64 = 5.0;

impl CampMusic {
    /// The rotation of `tracks` (cue, length in seconds), starting with
    /// track `first` (wrapped onto the pool) at clock time `now`, the
    /// music fading out over `fade_s`. `None` for no tracks. Returns the
    /// first cue to play too.
    pub fn start(
        tracks: Vec<(String, f64)>,
        first: usize,
        fade_s: f64,
        now: f64,
    ) -> Option<(Self, MusicCue)> {
        let index = first.checked_rem(tracks.len())?;
        let mut music = Self {
            tracks,
            fade_s: fade_s.max(0.0),
            phase: Phase::Quiet { index, until: now },
        };
        let cue = music.play(index, now);
        Some((music, cue))
    }

    /// The track playing or next to play.
    pub fn track(&self) -> &str {
        let (Phase::Playing { index, .. } | Phase::Quiet { index, .. }) = self.phase;
        self.tracks.get(index).map_or("", |(cue, _)| cue)
    }

    /// Whether a track is playing (not the quiet between two).
    pub fn is_playing(&self) -> bool {
        matches!(self.phase, Phase::Playing { .. })
    }

    /// Plays track `index` from clock time `now`.
    fn play(&mut self, index: usize, now: f64) -> MusicCue {
        let (cue, length) = self.tracks[index].clone();
        // engine-friction: engine#2341 (the fade-out starts the manifest's
        // one fade before the end, not audio.md's 3 s; no fade-in)
        let until = now + (length - self.fade_s).max(0.0);
        self.phase = Phase::Playing { index, until };
        MusicCue::Play(cue)
    }

    /// The clock reads `now`: what to ask for, if anything changes.
    pub fn tick(&mut self, now: f64) -> Option<MusicCue> {
        match self.phase {
            Phase::Playing { index, until } if now >= until => {
                let next = (index + 1) % self.tracks.len();
                let until = until + self.fade_s + QUIET_S;
                self.phase = Phase::Quiet { index: next, until };
                Some(MusicCue::Stop)
            }
            Phase::Quiet { index, until } if now >= until => Some(self.play(index, now)),
            _ => None,
        }
    }

    /// Starts the track playing again from `now`, from its beginning:
    /// something else played while camp was covered (a conversation's own
    /// music). In the quiet between tracks, nothing changes.
    pub fn resume(&mut self, now: f64) -> Option<MusicCue> {
        match self.phase {
            Phase::Playing { index, .. } => Some(self.play(index, now)),
            Phase::Quiet { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool() -> Vec<(String, f64)> {
        vec![("a".into(), 100.0), ("b".into(), 60.0), ("c".into(), 30.0)]
    }

    fn play(cue: &str) -> Option<MusicCue> {
        Some(MusicCue::Play(cue.into()))
    }

    #[test]
    fn camp_starts_with_the_track_picked() {
        let (music, cue) = CampMusic::start(pool(), 1, 0.5, 10.0).unwrap();
        assert_eq!(cue, MusicCue::Play("b".into()));
        assert_eq!(music.track(), "b");
        assert!(music.is_playing());
        // A pick past the pool wraps onto it; an empty pool has no music.
        let (music, _) = CampMusic::start(pool(), 4, 0.5, 0.0).unwrap();
        assert_eq!(music.track(), "b");
        assert!(CampMusic::start(vec![], 0, 0.5, 0.0).is_none());
    }

    /// A track plays until its fade ends with it, then camp is quiet for
    /// [`QUIET_S`], then the next track plays: round the pool, back to the
    /// first after the last.
    #[test]
    fn the_tracks_rotate_with_a_fade_and_a_quiet_between() {
        let (mut music, _) = CampMusic::start(pool(), 1, 0.5, 0.0).unwrap();
        assert_eq!(music.tick(59.0), None);
        assert_eq!(music.tick(59.5), Some(MusicCue::Stop));
        assert!(!music.is_playing());
        assert_eq!(music.track(), "c");
        assert_eq!(music.tick(64.9), None);
        assert_eq!(music.tick(65.0), play("c"));
        assert_eq!(music.tick(94.5), Some(MusicCue::Stop));
        assert_eq!(music.tick(100.0), play("a"));
        assert_eq!(music.track(), "a");
    }

    /// A long frame (camp covered by a conversation) moves the rotation on
    /// one step per tick, never skipping a track.
    #[test]
    fn a_late_tick_moves_on_one_step() {
        let (mut music, _) = CampMusic::start(pool(), 2, 0.5, 0.0).unwrap();
        assert_eq!(music.tick(500.0), Some(MusicCue::Stop));
        assert_eq!(music.tick(500.0), play("a"));
        assert_eq!(music.tick(500.0), None);
    }

    #[test]
    fn resuming_restarts_the_track_but_not_the_quiet() {
        let (mut music, _) = CampMusic::start(pool(), 0, 0.5, 0.0).unwrap();
        assert_eq!(music.resume(50.0), play("a"));
        assert_eq!(music.tick(149.0), None, "the track's whole length again");
        assert_eq!(music.tick(149.5), Some(MusicCue::Stop));
        assert_eq!(music.resume(150.0), None);
    }
}
