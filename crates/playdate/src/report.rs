//! What the front end shows and logs: while the game's data loads
//! (ticket 1031), temporary text with the SDK's own font, replaced by the
//! loading card of ticket 1036; then, once a minute, how fast the game
//! runs (ticket 1033), and in the bench build each render phase's time
//! (ticket 1055).

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use trpg_ui::onebit::Phase;

use crate::stack;

/// Shown while the game's data loads.
pub const LOADING: &str = "Loading\u{2026}";

/// The file in the game's data folder the report is also written to, for
/// reading a Simulator run without its console (`Disk/Data/<bundle ID>/`).
pub const FILE: &str = "load-report.txt";

/// The report after loading: how long it took, and how much of the
/// game's stack it used (`None` where it is not measured: the Simulator
/// runs the game on the PC's stack).
pub fn loaded(load_ms: u32, stack_used: Option<usize>) -> [String; 2] {
    [load_time(load_ms), stack_mark(stack_used)]
}

/// The load time line.
pub fn load_time(load_ms: u32) -> String {
    let load_ms = usize::try_from(load_ms).unwrap_or(usize::MAX);
    format!("Game data loaded in {} ms", grouped(load_ms))
}

/// The game stack's high-water mark line.
pub fn stack_mark(stack_used: Option<usize>) -> String {
    match stack_used {
        Some(used) => format!(
            "Game stack used: {} of {} bytes",
            grouped(used),
            grouped(stack::SIZE)
        ),
        None => String::from("Game stack: measured on the device only"),
    }
}

/// How long something took, frame after frame: the fewest, the most and
/// the average milliseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Timing {
    count: u32,
    min_ms: f32,
    max_ms: f32,
    total_ms: f32,
}

impl Timing {
    /// Counts one more time of `ms` milliseconds.
    pub fn add(&mut self, ms: f32) {
        if self.count == 0 {
            self.min_ms = ms;
            self.max_ms = ms;
        } else {
            self.min_ms = self.min_ms.min(ms);
            self.max_ms = self.max_ms.max(ms);
        }
        self.count += 1;
        self.total_ms += ms;
    }

    /// How many times were counted.
    pub fn count(&self) -> u32 {
        self.count
    }

    /// The average time, 0 before any.
    pub fn avg_ms(&self) -> f32 {
        #[allow(clippy::cast_precision_loss)] // a minute's frames
        let count = self.count.max(1) as f32;
        self.total_ms / count
    }

    /// `name: min / avg / max ms, n = count`, or `name: none` before any.
    pub fn line(&self, name: &str) -> String {
        if self.count == 0 {
            return format!("{name}: none");
        }
        format!(
            "{name}: min {:.1} / avg {:.1} / max {:.1} ms, n = {}",
            self.min_ms,
            self.avg_ms(),
            self.max_ms,
            grouped(self.count as usize)
        )
    }
}

/// How long a render took, whole and phase by phase
/// ([`trpg_ui::onebit::render_marked`]), frame after frame.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Phases {
    /// Each whole render.
    pub total: Timing,
    /// Each phase, in [`Phase::ALL`]'s order.
    each: [Timing; Phase::ALL.len()],
}

impl Phases {
    /// Counts `ms` milliseconds of `phase`.
    pub fn add(&mut self, phase: Phase, ms: f32) {
        if let Some(i) = Phase::ALL.iter().position(|&p| p == phase) {
            self.each[i].add(ms);
        }
    }

    /// The total's [line](Timing::line), then each phase's average and
    /// most.
    pub fn lines(&self, name: &str) -> [String; 2] {
        let phases: Vec<String> = Phase::ALL
            .iter()
            .zip(&self.each)
            .map(|(phase, t)| format!("{} {:.1} / {:.1}", phase.name(), t.avg_ms(), t.max_ms))
            .collect();
        [
            self.total.line(name),
            format!("{name} by phase, avg / max ms: {}", phases.join(", ")),
        ]
    }
}

/// How fast the game ran over a stretch of time ([`Minute::lines`]).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Minute {
    /// Seconds since the stretch began.
    pub seconds: f32,
    /// Each `Game::frame` call.
    pub frame: Timing,
    /// Each frame drawn to the screen: rendered with the one-bit
    /// renderer and copied (only frames whose picture changed).
    pub render: Timing,
}

/// How often [`Minute`]'s numbers are logged, in seconds.
pub const MINUTE_SECONDS: f32 = 60.0;

impl Minute {
    /// Whether the stretch is a minute long.
    pub fn is_over(&self) -> bool {
        self.seconds >= MINUTE_SECONDS
    }

    /// The numbers to log: the screen on top (`screen`), frames per
    /// second, the frame and render times, and the stack's mark.
    pub fn lines(&self, screen: &str, stack_used: Option<usize>) -> [String; 4] {
        let frames = self.frame.count();
        #[allow(clippy::cast_precision_loss)] // a minute's frames
        let fps = if self.seconds > 0.0 {
            frames as f32 / self.seconds
        } else {
            0.0
        };
        [
            format!(
                "On {screen} for {:.0} seconds: {fps:.1} frames a second",
                self.seconds
            ),
            self.frame.line("Game::frame"),
            self.render.line("Render and copy"),
            stack_mark(stack_used),
        ]
    }
}

/// `n` with a comma between each group of three digits.
pub fn grouped(n: usize) -> String {
    let digits = format!("{n}");
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_grouped_by_thousands() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(3_708), "3,708");
        assert_eq!(grouped(65_536), "65,536");
        assert_eq!(grouped(1_234_567), "1,234,567");
    }

    #[test]
    fn the_report_names_the_load_time_and_the_stack_used() {
        assert_eq!(
            loaded(3_708, Some(15_256)),
            [
                "Game data loaded in 3,708 ms",
                "Game stack used: 15,256 of 65,536 bytes"
            ]
        );
    }

    #[test]
    fn a_timing_keeps_the_fewest_the_most_and_the_average() {
        let mut t = Timing::default();
        assert_eq!(t.line("Render"), "Render: none");
        t.add(2.0);
        assert_eq!(
            t.line("Render"),
            "Render: min 2.0 / avg 2.0 / max 2.0 ms, n = 1"
        );
        t.add(4.0);
        t.add(0.5);
        assert_eq!(t.count(), 3);
        assert_eq!(
            t.line("Render"),
            "Render: min 0.5 / avg 2.2 / max 4.0 ms, n = 3"
        );
    }

    #[test]
    fn a_minute_names_the_screen_the_frame_rate_and_the_times() {
        let mut m = Minute::default();
        assert!(!m.is_over());
        for _ in 0..1_800 {
            m.frame.add(1.0);
        }
        m.render.add(12.0);
        m.seconds = 60.0;
        assert!(m.is_over());
        assert_eq!(
            m.lines("title", Some(20_000)),
            [
                "On title for 60 seconds: 30.0 frames a second",
                "Game::frame: min 1.0 / avg 1.0 / max 1.0 ms, n = 1,800",
                "Render and copy: min 12.0 / avg 12.0 / max 12.0 ms, n = 1",
                "Game stack used: 20,000 of 65,536 bytes",
            ]
        );
        assert_eq!(
            Minute::default().lines("title", None)[0],
            "On title for 0 seconds: 0.0 frames a second"
        );
    }

    #[test]
    fn phases_are_timed_each_and_whole() {
        let mut p = Phases::default();
        assert_eq!(p.lines("Fast")[0], "Fast: none");
        for ms in [1.0, 3.0] {
            p.total.add(ms * 10.0);
            p.add(Phase::Ground, ms);
            p.add(Phase::Glyphs, ms * 2.0);
        }
        assert_eq!(
            p.lines("Fast"),
            [
                "Fast: min 10.0 / avg 20.0 / max 30.0 ms, n = 2",
                "Fast by phase, avg / max ms: ground 2.0 / 3.0, backdrop 0.0 / 0.0, \
                 backgrounds 0.0 / 0.0, under items 0.0 / 0.0, edges 0.0 / 0.0, \
                 glyphs 4.0 / 6.0, over items 0.0 / 0.0, invert 0.0 / 0.0",
            ]
        );
    }

    #[test]
    fn the_simulator_report_says_the_stack_is_measured_on_the_device() {
        assert_eq!(
            loaded(5, None),
            [
                "Game data loaded in 5 ms",
                "Game stack: measured on the device only"
            ]
        );
    }
}
