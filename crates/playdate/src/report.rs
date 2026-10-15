//! What the front end shows and logs while it only loads the game's data
//! (ticket 1031): temporary text with the SDK's own font, replaced by the
//! loading card of ticket 1036.

use alloc::format;
use alloc::string::String;

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
