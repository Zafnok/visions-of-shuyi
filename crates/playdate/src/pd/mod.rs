//! The platform glue (the `sdk` feature): the SDK calls through
//! `shim/shim.c`, the allocator, the panic handler, the game's own stack
//! and the entry points the shim calls. The only place in this crate with
//! `unsafe` (ADR-0004 rule 5).
#![allow(unsafe_code)]

mod allocator;
mod entry;
mod game_stack;
mod panic;

use core::ffi::{c_char, c_int, c_uint, c_void};

use crate::text;

// One wrapper per SDK call, in `shim/shim.c`. The shim also has the
// elapsed-time timer and reading files, declared here once something
// uses them.
unsafe extern "C" {
    fn pd_realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    fn pd_log(text: *const c_char);
    fn pd_error(text: *const c_char);
    fn pd_millis() -> c_uint;
    fn pd_clear();
    fn pd_draw_text(text: *const c_char, len: usize, x: c_int, y: c_int);
    fn pd_file_open_write(name: *const c_char) -> *mut c_void;
    fn pd_file_write(file: *mut c_void, buf: *const c_void, len: c_uint) -> c_int;
    fn pd_file_close(file: *mut c_void) -> c_int;
}

/// Writes `line` to the console (the Simulator's, which also shows a
/// plugged-in device's).
pub fn log(line: &str) {
    let line = text::c_string(line);
    unsafe { pd_log(line.as_ptr().cast()) };
}

/// Milliseconds since some fixed moment: for timing, not for the date.
pub fn millis() -> u32 {
    unsafe { pd_millis() }
}

/// Clears the screen to white.
pub fn clear() {
    unsafe { pd_clear() };
}

/// Draws `line` in the SDK's current font, its top left at `(x, y)`.
pub fn draw_text(line: &str, x: i32, y: i32) {
    // `drawText` takes the length in characters, not bytes.
    let chars = line.chars().count();
    unsafe { pd_draw_text(line.as_ptr().cast(), chars, x, y) };
}

/// Replaces the file `name` in the game's data folder with `bytes`.
/// Errors are only logged: nothing so far depends on the file.
pub fn write_file(name: &str, bytes: &[u8]) {
    let c_name = text::c_string(name);
    let file = unsafe { pd_file_open_write(c_name.as_ptr().cast()) };
    if file.is_null() {
        log(&alloc::format!("could not open {name} to write it"));
        return;
    }
    let len = c_uint::try_from(bytes.len()).unwrap_or(c_uint::MAX);
    let written = unsafe { pd_file_write(file, bytes.as_ptr().cast(), len) };
    unsafe { pd_file_close(file) };
    if usize::try_from(written).ok() != Some(bytes.len()) {
        log(&alloc::format!("could not write all of {name}"));
    }
}
