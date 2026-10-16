//! The game's own stack (ADR-0070 §4): its size, and how much of it was
//! used.
//!
//! The device gives the C stack about 10 KB and the game needs about 15 KB
//! (ticket 1026), so the front end runs the game on [`SIZE`] bytes from the
//! heap (`pd::game_stack`). The block is filled with [`PAINT`] first; the
//! stack grows down from its top, so the painted bytes left at the bottom
//! say how deep it ever went ([`high_water`]).

/// The game's stack, in bytes: about four times the 15 KB measured.
pub const SIZE: usize = 64 * 1024;

/// The byte the stack is filled with before the game first runs on it.
pub const PAINT: u8 = 0xA5;

/// The bottom bytes of the stack that count as an overflow once used:
/// the game has gone, or is about to go, past its stack.
pub const RED_ZONE: usize = 256;

/// The bytes of `stack` (lowest address first) ever used: everything
/// above the painted bytes at its bottom.
pub fn high_water(stack: &[u8], paint: u8) -> usize {
    let untouched = stack.iter().take_while(|&&byte| byte == paint).count();
    stack.len() - untouched
}

/// Whether `used` bytes of a stack of `size` bytes reach its red zone.
pub fn overflowed(used: usize, size: usize) -> bool {
    used > size.saturating_sub(RED_ZONE)
}

/// The starting stack pointer for a stack of `size` bytes at `base`: its
/// top, rounded down to 8 bytes as the ARM procedure call standard asks.
pub fn top(base: usize, size: usize) -> usize {
    (base + size) & !7
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_untouched_stack_has_used_nothing() {
        assert_eq!(high_water(&[PAINT; 64], PAINT), 0);
    }

    #[test]
    fn the_used_bytes_are_those_above_the_paint() {
        let mut stack = [PAINT; 64];
        stack[40] = 0;
        stack[63] = 1;
        assert_eq!(high_water(&stack, PAINT), 24);
        stack[0] = 7;
        assert_eq!(high_water(&stack, PAINT), 64);
    }

    #[test]
    fn a_painted_byte_written_by_the_game_still_counts_above_the_deepest_write() {
        let mut stack = [PAINT; 16];
        stack[4] = 0;
        // Bytes 5..16 kept the paint's value, but byte 4 was written.
        assert_eq!(high_water(&stack, PAINT), 12);
    }

    #[test]
    fn the_red_zone_is_the_bottom_of_the_stack() {
        assert!(!overflowed(SIZE - RED_ZONE, SIZE));
        assert!(overflowed(SIZE - RED_ZONE + 1, SIZE));
        assert!(overflowed(SIZE, SIZE));
        assert!(!overflowed(15_256, SIZE));
        assert!(overflowed(1, 100));
    }

    #[test]
    fn the_top_is_eight_byte_aligned_and_inside_the_block() {
        assert_eq!(top(0x2000_0000, SIZE), 0x2001_0000);
        assert_eq!(top(0x2000_0004, 100), 0x2000_0068);
        assert_eq!(top(0x2000_0003, 16) % 8, 0);
        assert!(top(0x2000_0003, 16) <= 0x2000_0013);
    }
}
