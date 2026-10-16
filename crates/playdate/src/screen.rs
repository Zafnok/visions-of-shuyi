//! The one-bit frame onto the Playdate's screen (ticket 1033):
//! `trpg_ui::onebit` already lays a [`Frame`] out as the SDK's frame
//! buffer (52-byte rows, the highest bit the left pixel, a set bit white),
//! so only the rows that changed are copied, and the SDK is told which.

use trpg_ui::onebit::{Frame, ROW_BYTES};

/// Copies every row of `frame` that differs from `screen` (the SDK's
/// frame buffer, laid out the same) into it, and returns the first and
/// last row that changed, or `None` if none did.
pub fn copy_changed(screen: &mut Frame, frame: &Frame) -> Option<(i32, i32)> {
    let mut changed: Option<(i32, i32)> = None;
    let (screen_rows, _) = screen.as_chunks_mut::<ROW_BYTES>();
    let (frame_rows, _) = frame.as_chunks::<ROW_BYTES>();
    for (y, (old, new)) in (0..).zip(screen_rows.iter_mut().zip(frame_rows)) {
        if old != new {
            *old = *new;
            changed = Some(changed.map_or((y, y), |(first, _)| (first, y)));
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use trpg_ui::onebit::FRAME_BYTES;

    use super::*;

    fn white() -> Box<Frame> {
        Box::new([u8::MAX; FRAME_BYTES])
    }

    #[test]
    fn only_rows_that_changed_are_copied_and_named() {
        let mut screen = white();
        let mut frame = white();
        assert_eq!(copy_changed(&mut screen, &frame), None);
        frame[3 * ROW_BYTES] = 0;
        frame[200 * ROW_BYTES + 51] = 0x0f;
        assert_eq!(copy_changed(&mut screen, &frame), Some((3, 200)));
        assert_eq!(screen, frame);
        assert_eq!(copy_changed(&mut screen, &frame), None);
        frame[239 * ROW_BYTES] = 1;
        assert_eq!(copy_changed(&mut screen, &frame), Some((239, 239)));
    }
}
