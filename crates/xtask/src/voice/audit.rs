//! How a clip sounds against its words (ticket 0762): its length, its pace
//! in seconds per letter, and the pauses inside it, read from ffmpeg's
//! `silencedetect`. `cargo xtask voice audit` lists every clip; `generate`
//! warns of each clip it made that is over a limit.

use std::fmt::Write as _;

/// How quiet a stretch must be to count as a pause, dB (*tunable*): the
/// clips sit at −18 LUFS, and a reader's breaths and room noise between
/// words stay under −40 dB.
pub const PAUSE_NOISE_DB: i32 = -40;
/// The shortest stretch of quiet `silencedetect` reports, seconds.
pub const PAUSE_SHORTEST_S: f64 = 0.15;
/// The longest pause a clip may have inside it, seconds (*tunable*, the
/// agent's starting value, judged in the Act 1 playtest): the 90th
/// percentile of Chapter 1's and the guide's lines with no direction and
/// no `...` or `--` before ticket 0762 (0.64) plus a margin. The ticket
/// started at 0.6, which flagged 13 of those 75 plain lines.
pub const LONGEST_PAUSE_S: f64 = 0.8;
/// The slowest pace a clip may have, seconds per letter (*tunable*): the
/// 90th percentile of Chapter 1's and the guide's clips before ticket 0762
/// (0.157) plus a margin. Their median was 0.095.
pub const SLOWEST_PACE_S: f64 = 0.17;
/// The fewest letters a line has for its pace to be judged: a line of a
/// word or two is mostly the reader's breath in and out.
pub const PACE_LETTERS: usize = 20;
/// Two stretches of quiet this close are one pause (ffmpeg splits one when
/// a click passes the threshold), seconds.
const JOINED_S: f64 = 0.05;
/// A pause this close to either end of the clip is the trimmed silence
/// there, not a pause inside it, seconds.
const EDGE_S: f64 = 0.02;

/// What ffmpeg measured of a clip.
#[derive(Debug, Clone, PartialEq)]
pub struct Measure {
    /// Its length, seconds.
    pub length_s: f64,
    /// The pauses inside it (not at either end), seconds each.
    pub pauses: Vec<f64>,
}

/// The filter that logs a clip's pauses.
pub fn silence_filter() -> String {
    format!("silencedetect=noise={PAUSE_NOISE_DB}dB:d={PAUSE_SHORTEST_S}")
}

/// The number after `label` in `line`, if it has one.
fn after(line: &str, label: &str) -> Option<f64> {
    let at = line.find(label)? + label.len();
    line[at..].split_whitespace().next()?.parse().ok()
}

/// The pauses inside a clip `length_s` long, from `silencedetect`'s log
/// `log`: stretches of quiet a hair apart joined, those at either end left
/// out. A stretch with no end runs to the end of the clip.
pub fn inner_pauses(log: &str, length_s: f64) -> Vec<f64> {
    let mut quiet: Vec<(f64, f64)> = Vec::new();
    let mut open = None;
    for line in log.lines() {
        if let Some(start) = after(line, "silence_start:") {
            open = Some(start);
        } else if let Some(end) = after(line, "silence_end:") {
            quiet.push((open.take().unwrap_or(0.0), end));
        }
    }
    if let Some(start) = open {
        quiet.push((start, length_s));
    }
    let mut joined: Vec<(f64, f64)> = Vec::new();
    for (start, end) in quiet {
        match joined.last_mut() {
            Some(last) if start - last.1 < JOINED_S => last.1 = end,
            _ => joined.push((start, end)),
        }
    }
    joined
        .into_iter()
        .filter(|&(start, end)| start > EDGE_S && end < length_s - EDGE_S)
        .map(|(start, end)| end - start)
        .collect()
}

/// The letters and digits of `spoken`: what its pace is measured by.
pub fn letters(spoken: &str) -> usize {
    spoken.chars().filter(|c| c.is_alphanumeric()).count()
}

/// One clip, judged.
#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    /// Its file, relative to the language's folder.
    pub file: String,
    /// Its length, seconds.
    pub length_s: f64,
    /// The letters of its words.
    pub letters: usize,
    /// Seconds per letter.
    pub pace: f64,
    /// Its longest pause inside, seconds (0 if none).
    pub longest_pause: f64,
    /// Its pace is slower than [`SLOWEST_PACE_S`] (a line of at least
    /// [`PACE_LETTERS`] letters).
    pub slow: bool,
    /// A pause is longer than [`LONGEST_PAUSE_S`].
    pub halting: bool,
}

impl Row {
    /// Whether it is over a limit.
    pub const fn flagged(&self) -> bool {
        self.slow || self.halting
    }

    /// How far over the limits it is: its pause and its pace (when
    /// judged) over their limits, the larger; over 1 is flagged.
    pub fn over(&self) -> f64 {
        let pace = if self.letters >= PACE_LETTERS {
            self.pace / SLOWEST_PACE_S
        } else {
            0.0
        };
        (self.longest_pause / LONGEST_PAUSE_S).max(pace)
    }

    /// What is wrong with it, for a report.
    pub fn flags(&self) -> String {
        let mut flags = Vec::new();
        if self.halting {
            flags.push(format!("pause {:.2} s", self.longest_pause));
        }
        if self.slow {
            flags.push(format!("pace {:.3} s/letter", self.pace));
        }
        flags.join(", ")
    }
}

/// Judges the clip `file`, which says `spoken`, as `measure` measured it.
pub fn judge(file: &str, spoken: &str, measure: &Measure) -> Row {
    let letters = letters(spoken);
    // Lines are a few hundred letters at most.
    let pace = measure.length_s / f64::from(u32::try_from(letters.max(1)).unwrap_or(u32::MAX));
    let longest_pause = measure.pauses.iter().copied().fold(0.0, f64::max);
    Row {
        file: file.to_owned(),
        length_s: measure.length_s,
        letters,
        pace,
        longest_pause,
        slow: letters >= PACE_LETTERS && pace > SLOWEST_PACE_S,
        halting: longest_pause > LONGEST_PAUSE_S,
    }
}

/// The value `percent` of the way up `sorted`, nearest rank.
fn percentile(sorted: &[f64], percent: usize) -> f64 {
    let Some(last) = sorted.len().checked_sub(1) else {
        return 0.0;
    };
    sorted[(last * percent + 50) / 100]
}

/// The audit's report: every row, the flagged ones marked, then the
/// pace's spread and the count over each limit.
pub fn report(rows: &[Row]) -> String {
    let mut out = format!(
        "  {:<44} {:>7} {:>7} {:>9} {:>7}\n",
        "clip", "length", "letters", "s/letter", "pause"
    );
    for row in rows {
        let mark = if row.flagged() { '!' } else { ' ' };
        let _ = write!(
            out,
            "{mark} {:<44} {:>6.2}s {:>7} {:>9.3} {:>6.2}s",
            row.file, row.length_s, row.letters, row.pace, row.longest_pause
        );
        if row.flagged() {
            let _ = write!(out, "  {}", row.flags());
        }
        out.push('\n');
    }
    let mut paces: Vec<f64> = rows
        .iter()
        .filter(|r| r.letters >= PACE_LETTERS)
        .map(|r| r.pace)
        .collect();
    paces.sort_by(f64::total_cmp);
    let count = |f: fn(&Row) -> bool| rows.iter().filter(|r| f(r)).count();
    let _ = write!(
        out,
        "{} clips; pace of lines of {PACE_LETTERS}+ letters: median {:.3}, 90th percentile {:.3} \
         s/letter\n{} flagged: {} with a pause over {LONGEST_PAUSE_S} s, {} slower than \
         {SLOWEST_PACE_S} s/letter",
        rows.len(),
        percentile(&paces, 50),
        percentile(&paces, 90),
        count(Row::flagged),
        count(|r| r.halting),
        count(|r| r.slow),
    );
    out
}
