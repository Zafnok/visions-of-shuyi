//! The platform glue (the `sdk` feature): the SDK calls through
//! `shim/shim.c`, the allocator, the panic handler, the game's own stack
//! and the entry points the shim calls. The only place in this crate with
//! `unsafe` (ADR-0004 rule 5).
#![allow(unsafe_code)]

mod allocator;
#[cfg(feature = "bench")]
mod bench;
mod entry;
mod files;
mod game_stack;
mod panic;

use alloc::vec::Vec;
use core::ffi::{c_char, c_float, c_int, c_uint, c_void};

use trpg_ui::onebit::Frame;

use crate::text;

// One wrapper per SDK call, in `shim/shim.c`. The shim also has reading
// a file from the game's `.pdx`, declared here once something uses it.
unsafe extern "C" {
    fn pd_realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    fn pd_log(text: *const c_char);
    fn pd_error(text: *const c_char);
    fn pd_millis() -> c_uint;
    fn pd_reset_elapsed();
    fn pd_elapsed() -> c_float;
    fn pd_buttons(current: *mut c_uint, pushed: *mut c_uint);
    fn pd_crank_change() -> c_float;
    fn pd_crank_docked() -> c_int;
    fn pd_add_menu_item(title: *const c_char, entry: c_int) -> *mut c_void;
    fn pd_set_menu_item_title(item: *mut c_void, title: *const c_char);
    fn pd_set_refresh_rate(rate: c_float);
    fn pd_frame() -> *mut u8;
    fn pd_mark_updated_rows(start: c_int, end: c_int);
    fn pd_clear();
    fn pd_draw_text(text: *const c_char, len: usize, x: c_int, y: c_int);
    fn pd_file_open_write(name: *const c_char) -> *mut c_void;
    fn pd_file_open_read_data(name: *const c_char) -> *mut c_void;
    fn pd_file_read(file: *mut c_void, buf: *mut c_void, len: c_uint) -> c_int;
    fn pd_file_write(file: *mut c_void, buf: *const c_void, len: c_uint) -> c_int;
    fn pd_file_close(file: *mut c_void) -> c_int;
    fn pd_file_error() -> *const c_char;
    fn pd_file_kind(name: *const c_char) -> c_int;
    fn pd_file_list(
        path: *const c_char,
        each: extern "C" fn(name: *const c_char, userdata: *mut c_void),
        userdata: *mut c_void,
    ) -> c_int;
    fn pd_file_mkdir(path: *const c_char) -> c_int;
    fn pd_file_unlink(name: *const c_char) -> c_int;
    fn pd_file_rename(from: *const c_char, to: *const c_char) -> c_int;
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

/// Seconds since [`reset_elapsed`], from the SDK's high-resolution timer.
pub fn elapsed() -> f32 {
    unsafe { pd_elapsed() }
}

/// Starts [`elapsed`] again from 0.
pub fn reset_elapsed() {
    unsafe { pd_reset_elapsed() };
}

/// The buttons held now and the ones pressed since the last call, as the
/// SDK's `PDButtons` bits (read by `crate::pads`).
pub fn buttons() -> (u32, u32) {
    let (mut current, mut pushed) = (0, 0);
    unsafe { pd_buttons(&raw mut current, &raw mut pushed) };
    (current, pushed)
}

/// The degrees the crank turned since the last call (clockwise, which
/// winds it forwards, positive), and whether it is docked.
pub fn crank() -> (f32, bool) {
    let change = unsafe { pd_crank_change() };
    let docked = unsafe { pd_crank_docked() } != 0;
    (change, docked)
}

/// One of the game's entries in the system menu, as the SDK knows it,
/// with its name as given (kept while the SDK may read it).
pub struct MenuItem {
    item: *mut c_void,
    title: Vec<u8>,
}

/// Adds the game's entry `entry` (its index, handed back to
/// `trpg_playdate_menu_chosen` when chosen) to the system menu, named
/// `title`; `None` if the SDK refused (it takes at most three).
pub fn add_menu_item(title: &str, entry: usize) -> Option<MenuItem> {
    let title = text::c_string(title);
    let entry = c_int::try_from(entry).ok()?;
    let item = unsafe { pd_add_menu_item(title.as_ptr().cast(), entry) };
    (!item.is_null()).then_some(MenuItem { item, title })
}

/// Renames `item` in the system menu.
pub fn set_menu_item_title(item: &mut MenuItem, title: &str) {
    let title = text::c_string(title);
    unsafe { pd_set_menu_item_title(item.item, title.as_ptr().cast()) };
    item.title = title;
}

/// How many frames a second the SDK calls the update callback, at most.
pub fn set_refresh_rate(rate: f32) {
    unsafe { pd_set_refresh_rate(rate) };
}

/// Copies the rows of `frame` that changed onto the screen and tells the
/// SDK which; true if any did.
pub fn show(frame: &Frame) -> bool {
    // SAFETY: the SDK's frame buffer is `LCD_ROWSIZE` × `LCD_ROWS` bytes,
    // which `shim.c` checks is a `Frame`, and nothing else holds it
    // during the update callback.
    let screen = unsafe { &mut *pd_frame().cast::<Frame>() };
    match crate::screen::copy_changed(screen, frame) {
        Some((first, last)) => {
            unsafe { pd_mark_updated_rows(first, last) };
            true
        }
        None => false,
    }
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
