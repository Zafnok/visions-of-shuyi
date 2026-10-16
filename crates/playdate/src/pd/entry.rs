//! The entry points `shim/shim.c` calls: once at start, then once per
//! frame from the SDK's update callback.
//!
//! So far (ticket 1031) the front end shows "Loading…", loads the game's
//! data on the game's stack, then shows and logs how long that took and
//! how much of the stack it used.

use alloc::boxed::Box;
use alloc::string::String;
use core::cell::UnsafeCell;
use core::ffi::c_int;

use super::{clear, draw_text, game_stack, log, millis, write_file};
use crate::report;

/// What the front end does on its next frame.
enum Step {
    /// Show the loading text; it reaches the screen at the end of the frame.
    ShowLoading,
    /// Load the game's data.
    Load,
    /// Loaded: the report is on the screen.
    Loaded {
        /// The game's data, kept as the game will keep it.
        _content: Box<trpg_content::Content>,
        /// When the stack's mark was last logged, in milliseconds.
        logged_at: u32,
        /// The stack's mark when it was last logged.
        logged_used: Option<usize>,
    },
}

/// The front end's state. The SDK calls the update callback on its only
/// thread, so this is never touched from two places at once.
struct Front(UnsafeCell<Option<Step>>);

// SAFETY: the Playdate runs the game on one thread (see `Front`).
unsafe impl Sync for Front {}

static FRONT: Front = Front(UnsafeCell::new(None));

/// Line height of the SDK's system font, in pixels.
const LINE: i32 = 20;

/// Called by the shim once, at `kEventInit`, before the first frame.
#[unsafe(no_mangle)]
pub extern "C" fn trpg_playdate_init() {
    game_stack::init();
    // SAFETY: nothing else holds the state (see `Front`).
    unsafe { *FRONT.0.get() = Some(Step::ShowLoading) };
}

/// Called by the shim every frame; returns 1 when the screen changed.
#[unsafe(no_mangle)]
pub extern "C" fn trpg_playdate_update() -> c_int {
    // SAFETY: nothing else holds the state (see `Front`), and nothing
    // below calls back in here.
    let Some(step) = (unsafe { &mut *FRONT.0.get() }) else {
        return 0;
    };
    c_int::from(update(step))
}

/// One frame; true when the screen changed.
fn update(step: &mut Step) -> bool {
    match step {
        Step::ShowLoading => {
            clear();
            draw_text(report::LOADING, 8, 8);
            *step = Step::Load;
            true
        }
        Step::Load => {
            let start = millis();
            let content = game_stack::run(trpg_content::load_embedded)
                .unwrap_or_else(|e| panic!("the game's data did not load: {e}"));
            let load_ms = millis().wrapping_sub(start);
            let used = game_stack::high_water();
            let lines = report::loaded(load_ms, used);
            clear();
            let mut file = String::new();
            for (row, line) in (0..).zip(&lines) {
                log(line);
                draw_text(line, 8, 8 + row * LINE);
                file.push_str(line);
                file.push('\n');
            }
            write_file(report::FILE, file.as_bytes());
            *step = Step::Loaded {
                _content: Box::new(content),
                logged_at: millis(),
                logged_used: used,
            };
            true
        }
        Step::Loaded {
            logged_at,
            logged_used,
            ..
        } => {
            // While debugging: the stack's mark once a second, when it moved.
            if cfg!(debug_assertions) && millis().wrapping_sub(*logged_at) >= 1000 {
                *logged_at = millis();
                let used = game_stack::high_water();
                if used != *logged_used {
                    *logged_used = used;
                    log(&report::stack_mark(used));
                }
            }
            false
        }
    }
}
