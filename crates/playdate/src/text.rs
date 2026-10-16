//! Text handed to the SDK, which takes C strings (`NUL`-terminated).

use alloc::vec::Vec;
use core::fmt;

/// `text` as a C string: its bytes without any `NUL`, then one `NUL`.
pub fn c_string(text: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = text.bytes().filter(|&byte| byte != 0).collect();
    bytes.push(0);
    bytes
}

/// A C string of at most `N - 1` bytes written with [`fmt::Write`],
/// without allocating: the panic handler's message, which must not fail
/// when the heap is what ran out. Text that does not fit is cut at a
/// character boundary; `NUL`s are left out.
pub struct FixedText<const N: usize> {
    bytes: [u8; N],
    len: usize,
    /// Something did not fit: nothing more is written, so the text is
    /// always a beginning of what was written.
    full: bool,
}

impl<const N: usize> FixedText<N> {
    /// Empty text.
    pub const fn new() -> Self {
        Self {
            bytes: [0; N],
            len: 0,
            full: false,
        }
    }

    /// The text so far, without its `NUL`.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    /// The text so far with its `NUL`, for the SDK.
    pub fn as_c_bytes(&self) -> &[u8] {
        &self.bytes[..=self.len]
    }
}

impl<const N: usize> Default for FixedText<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> fmt::Write for FixedText<N> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if self.full {
            return Ok(());
        }
        for ch in text.chars().filter(|&ch| ch != '\0') {
            let mut utf8 = [0; 4];
            let encoded = ch.encode_utf8(&mut utf8).as_bytes();
            // One byte stays free for the `NUL`.
            if self.len + encoded.len() >= N {
                self.full = true;
                break;
            }
            self.bytes[self.len..self.len + encoded.len()].copy_from_slice(encoded);
            self.len += encoded.len();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::fmt::Write as _;

    #[test]
    fn a_c_string_ends_in_one_nul_and_has_no_other() {
        assert_eq!(c_string("ab"), b"ab\0");
        assert_eq!(c_string("a\0b"), b"ab\0");
        assert_eq!(c_string(""), b"\0");
    }

    #[test]
    fn fixed_text_keeps_what_fits_and_ends_in_nul() {
        let mut text = FixedText::<8>::new();
        write!(text, "panic: {}", 42).unwrap();
        assert_eq!(text.as_bytes(), b"panic: ");
        assert_eq!(text.as_c_bytes(), b"panic: \0");
    }

    #[test]
    fn fixed_text_cuts_at_a_character_boundary_and_drops_nuls() {
        let mut text = FixedText::<5>::new();
        write!(text, "a\0b\u{2026}c").unwrap();
        // "ab" + "…" (3 bytes) would leave no room for the NUL, and
        // nothing after what did not fit is written.
        assert_eq!(text.as_bytes(), b"ab");
        write!(text, "d").unwrap();
        assert_eq!(text.as_bytes(), b"ab");
        let mut text = FixedText::<6>::default();
        write!(text, "ab\u{2026}c").unwrap();
        assert_eq!(text.as_bytes(), "ab\u{2026}".as_bytes());
        assert_eq!(text.as_c_bytes().last(), Some(&0));
    }
}
