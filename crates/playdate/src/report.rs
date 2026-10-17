//! What the front end logs: how the start went (ticket 1031), with when
//! the loading card and the game's first frame were shown and which files
//! took longest to load (ticket 1036); then, once a minute, how fast the
//! game runs (ticket 1033), and in the bench build each render phase's
//! time (ticket 1055).

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use trpg_ui::onebit::Phase;

use crate::stack;

/// The file in the game's data folder the report is also written to, for
/// reading a Simulator run without its console (`Disk/Data/<bundle ID>/`).
/// It also lists every file read while loading, with its time.
pub const FILE: &str = "load-report.txt";

/// How many of the slowest files the log names.
pub const SLOWEST: usize = 10;

/// When the loading card was shown, in milliseconds after the game
/// started.
pub fn card_shown(ms: u32) -> String {
    format!("Loading card shown {} ms after start", grouped(ms as usize))
}

/// When the game's first frame (the title) was shown, in milliseconds
/// after the game started.
pub fn game_shown(ms: u32) -> String {
    format!(
        "First frame of the game shown {} ms after start",
        grouped(ms as usize)
    )
}

/// The files the game reads from its bundle while it loads, each with when
/// (`shuyi_assets::bundle::watch`): a file's time is from its read to the
/// next read, the parsing and checking of what it holds. Only roughly: a
/// loader may read several files before it parses them. The steps of the
/// start that read no file are marked (`mark`), so their time is theirs.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Reads {
    /// Each read, in order: the path (or a step's name in brackets),
    /// whether the bundle has it (`None` for a step), and when
    /// (milliseconds after loading started).
    reads: Vec<(String, Option<bool>, f32)>,
}

/// One file's reads, or one step, for the report.
#[derive(Debug, Clone, Copy, PartialEq)]
struct FileTime {
    /// From each of its reads to the next read, summed.
    ms: f32,
    /// How many times it was read.
    reads: u32,
    /// Whether the bundle has it; `None` for a step.
    found: Option<bool>,
}

impl Reads {
    /// Notes a read of `path`, `at_ms` milliseconds after loading started.
    pub fn add(&mut self, path: &str, found: bool, at_ms: f32) {
        self.reads.push((String::from(path), Some(found), at_ms));
    }

    /// Notes that the step `name` of the start began, `at_ms` milliseconds
    /// after loading started: the time to the next read is its own.
    pub fn mark(&mut self, name: &str, at_ms: f32) {
        self.reads.push((format!("({name})"), None, at_ms));
    }

    /// Each file read and each step, by name, when loading ended at
    /// `end_ms`.
    fn files(&self, end_ms: f32) -> BTreeMap<&str, FileTime> {
        let mut files: BTreeMap<&str, FileTime> = BTreeMap::new();
        let ends = self.reads.iter().skip(1).map(|r| r.2).chain([end_ms]);
        for ((path, found, at), end) in self.reads.iter().zip(ends) {
            let file = files.entry(path).or_insert(FileTime {
                ms: 0.0,
                reads: 0,
                found: *found,
            });
            file.ms += (end - at).max(0.0);
            file.reads += 1;
        }
        files
    }

    /// The files by time, the slowest first (by path where equal).
    fn by_time(&self, end_ms: f32) -> Vec<(&str, FileTime)> {
        let mut files: Vec<(&str, FileTime)> = self.files(end_ms).into_iter().collect();
        files.sort_by(|a, b| b.1.ms.total_cmp(&a.1.ms).then(a.0.cmp(b.0)));
        files
    }

    /// What the log says: how many reads of how many files, which the
    /// bundle lacks, and the [`SLOWEST`] slowest files and steps.
    pub fn summary(&self, end_ms: f32) -> Vec<String> {
        let files = self.by_time(end_ms);
        let missing: Vec<&str> = files
            .iter()
            .filter(|(_, f)| f.found == Some(false))
            .map(|(path, _)| *path)
            .collect();
        let reads = self.reads.iter().filter(|r| r.1.is_some()).count();
        let read = files.iter().filter(|(_, f)| f.found.is_some()).count();
        let mut lines = Vec::from([format!(
            "Bundle: {} reads of {} files, {} not in the bundle",
            grouped(reads),
            grouped(read),
            missing.len()
        )]);
        if !missing.is_empty() {
            lines.push(format!("Not in the bundle: {}", missing.join(", ")));
        }
        let slowest: Vec<String> = files
            .iter()
            .take(SLOWEST)
            .map(|(path, f)| format!("{path} {:.0}", f.ms))
            .collect();
        if !slowest.is_empty() {
            lines.push(format!(
                "Slowest files, ms to the next read: {}",
                slowest.join(", ")
            ));
        }
        lines
    }

    /// Every file read and step, the slowest first: its time, and how many
    /// times it was read when more than once.
    pub fn each(&self, end_ms: f32) -> Vec<String> {
        self.by_time(end_ms)
            .into_iter()
            .map(|(path, f)| {
                let missing = if f.found == Some(false) {
                    ", not in the bundle"
                } else {
                    ""
                };
                let reads = if f.reads > 1 {
                    format!(", {} reads", f.reads)
                } else {
                    String::new()
                };
                format!("  {path}: {:.1} ms{reads}{missing}", f.ms)
            })
            .collect()
    }
}

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
    fn the_start_lines_name_when_the_card_and_the_game_were_shown() {
        assert_eq!(card_shown(85), "Loading card shown 85 ms after start");
        assert_eq!(
            game_shown(3_812),
            "First frame of the game shown 3,812 ms after start"
        );
    }

    /// A file's time runs to the next read; the last one's to the end.
    #[test]
    fn each_file_takes_the_time_to_the_next_read() {
        let mut reads = Reads::default();
        reads.add("data/palette.ron", true, 0.0);
        reads.add("data/terrain.ron", true, 10.0);
        reads.add("data/palette.ron", true, 40.0);
        reads.add("lang/fr/ui.ron", false, 42.5);
        reads.add("dialogue/ch01.dlg", true, 43.0);
        reads.mark("the title", 113.0);
        assert_eq!(
            reads.summary(143.0),
            [
                "Bundle: 5 reads of 4 files, 1 not in the bundle",
                "Not in the bundle: lang/fr/ui.ron",
                "Slowest files, ms to the next read: dialogue/ch01.dlg 70, \
                 (the title) 30, data/terrain.ron 30, data/palette.ron 12, lang/fr/ui.ron 0",
            ]
        );
        assert_eq!(
            reads.each(143.0),
            [
                "  dialogue/ch01.dlg: 70.0 ms",
                "  (the title): 30.0 ms",
                "  data/terrain.ron: 30.0 ms",
                "  data/palette.ron: 12.5 ms, 2 reads",
                "  lang/fr/ui.ron: 0.5 ms, not in the bundle",
            ]
        );
    }

    #[test]
    fn the_summary_names_only_the_slowest_files() {
        let mut reads = Reads::default();
        for i in 0..15_u8 {
            reads.add(&format!("f{i:02}"), true, f32::from(i) * f32::from(i));
        }
        let summary = reads.summary(225.0);
        assert_eq!(summary.len(), 2, "{summary:?}");
        assert_eq!(
            summary[0],
            "Bundle: 15 reads of 15 files, 0 not in the bundle"
        );
        assert_eq!(
            summary[1],
            "Slowest files, ms to the next read: f14 29, f13 27, f12 25, f11 23, \
             f10 21, f09 19, f08 17, f07 15, f06 13, f05 11"
        );
        assert_eq!(reads.each(225.0).len(), 15);
        assert_eq!(
            Reads::default().summary(0.0),
            ["Bundle: 0 reads of 0 files, 0 not in the bundle"]
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
