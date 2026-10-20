//! Where the title's cinematic is in its time (ticket 0819): the title
//! song's own clock while it sounds, the frame clock while it doesn't, and
//! the song's silent ending counted out after a skip.
//!
//! quad-snd can't seek (ADR-0037), and a skip doesn't need to: the song
//! ends in silence, so "jump to the silence" is "stop the song, wait the
//! silence out, start it again".

use shuyi_ui::input::frame_seconds;

/// Seconds the cinematic waits at its start for the music to sound before
/// it runs on the frame clock instead (*tunable*).
pub const MUSIC_WAIT_SECS: f32 = 3.0;

/// The cinematic's time, and what moves it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SongTime {
    /// Seconds into the cinematic.
    t: f32,
    /// Seconds without the music's clock since it was last there (or
    /// since the start).
    waited: f32,
    /// Whether a skip stopped the song: the time runs on the frame clock
    /// to the cinematic's end, whatever the music's clock says.
    silence: bool,
}

impl SongTime {
    /// Seconds into the cinematic.
    pub fn t(&self) -> f32 {
        self.t
    }

    /// Whether a skip's silence is being counted out.
    pub fn in_silence(&self) -> bool {
        self.silence
    }

    /// Jumps to `t` (the logo shot's start) after a skip stopped the song:
    /// from here the time runs on the frame clock to the end.
    pub fn skip_to(&mut self, t: f32) {
        *self = Self {
            t,
            waited: 0.0,
            silence: true,
        };
    }

    /// Lets a frame of `dt` seconds pass (a time that isn't a number, or
    /// is below 0, counts as 0) in a cinematic `length` seconds long.
    /// `music` is how far into the title song the music is, if that is
    /// what sounds.
    ///
    /// - With `music`, the time is the music's, never smoothed.
    /// - Without, it holds for [`MUSIC_WAIT_SECS`], then runs on `dt`,
    ///   wrapping at `length`.
    /// - After a skip it runs on `dt` to `length`, then starts again at 0.
    ///
    /// Returns whether a skip's silence just ended: the caller asks for
    /// the title song again.
    pub fn advance(&mut self, music: Option<f32>, dt: f32, length: f32) -> bool {
        let dt = frame_seconds(dt);
        if self.silence {
            self.t += dt;
            if self.t < length {
                return false;
            }
            *self = Self::default();
            return true;
        }
        match music {
            Some(position) => {
                self.t = position;
                self.waited = 0.0;
            }
            None if self.waited < MUSIC_WAIT_SECS => self.waited += dt,
            None if length > 0.0 => self.t = (self.t + dt) % length,
            None => {}
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LENGTH: f32 = 10.0;

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn it_follows_the_music_exactly() {
        let mut s = SongTime::default();
        assert!(near(s.t(), 0.0));
        for position in [0.5, 0.52, 7.0, 0.1] {
            // The frame's own length doesn't count.
            assert!(!s.advance(Some(position), 0.25, LENGTH));
            assert!(near(s.t(), position), "{}", s.t());
        }
        assert!(!s.in_silence());
    }

    #[test]
    fn without_music_it_holds_then_runs_on_the_frame_clock_and_loops() {
        let mut s = SongTime::default();
        // Three seconds at the start.
        for _ in 0..6 {
            assert!(!s.advance(None, 0.5, LENGTH));
            assert!(near(s.t(), 0.0), "{}", s.t());
        }
        // Then it runs.
        s.advance(None, 0.5, LENGTH);
        assert!(near(s.t(), 0.5), "{}", s.t());
        s.advance(None, 9.0, LENGTH);
        assert!(near(s.t(), 9.5), "{}", s.t());
        // It wraps at the length, and never asks for the music.
        assert!(!s.advance(None, 1.0, LENGTH));
        assert!(near(s.t(), 0.5), "{}", s.t());
        // A frame that isn't a time, or goes backwards, adds nothing.
        for bad in [f32::NAN, f32::INFINITY, -1.0] {
            s.advance(None, bad, LENGTH);
            assert!(near(s.t(), 0.5), "{bad}: {}", s.t());
        }
        // A cinematic of no length stays put.
        s.advance(None, 1.0, 0.0);
        assert!(near(s.t(), 0.5), "{}", s.t());
    }

    #[test]
    fn it_switches_to_the_music_when_it_appears_and_holds_again_when_it_goes() {
        let mut s = SongTime::default();
        s.advance(None, 2.0, LENGTH);
        // The track loaded after two seconds.
        s.advance(Some(0.0), 0.5, LENGTH);
        s.advance(Some(4.0), 0.5, LENGTH);
        assert!(near(s.t(), 4.0));
        // The music gone: held where it was for three seconds, then on.
        s.advance(None, 2.9, LENGTH);
        assert!(near(s.t(), 4.0), "{}", s.t());
        s.advance(None, 0.2, LENGTH);
        assert!(near(s.t(), 4.0), "{}", s.t());
        s.advance(None, 0.2, LENGTH);
        assert!(near(s.t(), 4.2), "{}", s.t());
        // Late music wins over the frame clock.
        s.advance(Some(1.0), 0.5, LENGTH);
        assert!(near(s.t(), 1.0));
    }

    #[test]
    fn a_skip_counts_the_silence_out_then_asks_for_the_song() {
        let mut s = SongTime::default();
        s.advance(Some(3.0), 0.1, LENGTH);
        s.skip_to(8.0);
        assert!(s.in_silence());
        assert!(near(s.t(), 8.0));
        // The song is fading out: its clock is not followed, and nothing
        // is held.
        assert!(!s.advance(Some(3.1), 0.5, LENGTH));
        assert!(near(s.t(), 8.5), "{}", s.t());
        assert!(!s.advance(None, 1.25, LENGTH));
        assert!(near(s.t(), 9.75), "{}", s.t());
        // The end: back to the start, the song asked for once.
        assert!(s.advance(None, 0.25, LENGTH));
        assert_eq!(s, SongTime::default());
        // Until it sounds the time holds at 0 as at the start.
        assert!(!s.advance(None, 2.0, LENGTH));
        assert!(near(s.t(), 0.0));
        assert!(!s.advance(Some(0.3), 0.1, LENGTH));
        assert!(near(s.t(), 0.3));
    }
}
