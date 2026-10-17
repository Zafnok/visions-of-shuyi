//! The bench build (`cargo xtask playdate --bench`, ticket 1055): after
//! each frame drawn, it renders the same buffer whole again with the fast
//! renderer and with 1032's (`onebit::reference`), timing each phase of
//! both, and checks the three frames are the same. Once a minute the
//! front end logs [`Bench::lines`]. The extra renders slow the game down:
//! a build for measuring, never for playing.
//!
//! It also times the music (ticket 1047): every other minute the music is
//! paused, so the same work's times with and without the file player
//! decoding are side by side in the log.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use trpg_ui::GlyphBuffer;
use trpg_ui::onebit::{self, FRAME_BYTES, Frame, Phase};

use super::elapsed;
use crate::pictures::Pictures;
use crate::report::Phases;

/// The renders' scratch frames and times.
pub struct Bench {
    fast: Box<Frame>,
    slow: Box<Frame>,
    fast_phases: Phases,
    slow_phases: Phases,
    /// Frames the fast renderer drew unlike 1032's.
    unlike_reference: u32,
    /// Frames redrawn where they changed unlike their whole render.
    unlike_whole: u32,
    /// Whether the music is paused this minute.
    music_paused: bool,
}

impl Bench {
    pub fn new() -> Self {
        Self {
            fast: Box::new([0; FRAME_BYTES]),
            slow: Box::new([0; FRAME_BYTES]),
            fast_phases: Phases::default(),
            slow_phases: Phases::default(),
            unlike_reference: 0,
            unlike_whole: 0,
            music_paused: false,
        }
    }

    /// Whether the music is paused next minute: every other one.
    pub fn next_minute_paused(&mut self) -> bool {
        self.music_paused = !self.music_paused;
        self.music_paused
    }

    /// Renders `buffer` whole with both renderers, timing them, and
    /// checks both against `shown`, the frame the front end drew of it.
    pub fn measure(
        &mut self,
        buffer: &GlyphBuffer,
        font: &onebit::Font<'_>,
        pictures: &Pictures,
        shown: &Frame,
    ) {
        time(&mut self.slow_phases, |mut mark| {
            onebit::reference::render_marked(buffer, font, pictures, &mut self.slow, &mut mark);
        });
        time(&mut self.fast_phases, |mut mark| {
            onebit::render_marked(buffer, font, pictures, &mut self.fast, &mut mark);
        });
        if self.fast != self.slow {
            self.unlike_reference += 1;
        }
        if *shown != *self.fast {
            self.unlike_whole += 1;
        }
    }

    /// What to log, then starts counting afresh.
    pub fn lines(&mut self) -> Vec<String> {
        let mut lines = Vec::new();
        lines.extend(self.slow_phases.lines("Bench, 1032's render"));
        lines.extend(self.fast_phases.lines("Bench, fast render"));
        lines.push(format!(
            "Bench: {} frames unlike 1032's render, {} redrawn unlike a whole render",
            self.unlike_reference, self.unlike_whole
        ));
        lines.push(String::from(if self.music_paused {
            "Bench: the music was paused this minute"
        } else {
            "Bench: the music played this minute"
        }));
        self.fast_phases = Phases::default();
        self.slow_phases = Phases::default();
        self.unlike_reference = 0;
        self.unlike_whole = 0;
        lines
    }
}

/// Runs `render`, adding its time and each phase's to `phases`.
fn time(phases: &mut Phases, render: impl FnOnce(&mut dyn FnMut(Phase))) {
    let start = elapsed();
    let mut last = start;
    render(&mut |phase| {
        let now = elapsed();
        phases.add(phase, (now - last) * 1000.0);
        last = now;
    });
    phases.total.add((elapsed() - start) * 1000.0);
}
