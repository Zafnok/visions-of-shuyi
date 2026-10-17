//! The entry points `shim/shim.c` calls: once at start, then once per
//! frame from the SDK's update callback.
//!
//! The front end shows "Loading…", loads the game on the game's own
//! stack, then runs it (ticket 1033): each frame reads the buttons,
//! calls `Game::frame` with the time since the last frame, and when the
//! picture changed redraws what changed with `trpg_ui::onebit` (ticket
//! 1055) and copies it to the screen. Once a minute it logs how fast that
//! went.

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use core::cell::UnsafeCell;
use core::ffi::c_int;

use trpg_content::font::ATLAS_1BIT_PATH;
use trpg_content::{FontAtlasDef, PaletteDef};
use trpg_ui::input::{Device, PadKind};
use trpg_ui::onebit::{self, FRAME_BYTES, Frame};
use trpg_ui::screen::{IdleMotion, Keyboard, Lessons};
use trpg_ui::{ConsoleSpec, Ctx, Game, GlyphBuffer, KeyPrompt, Palette};

use super::files::SdkFiles;
use super::{
    buttons, clear, draw_text, elapsed, game_stack, log, millis, reset_elapsed, set_refresh_rate,
    show, write_file,
};
use crate::pads::PlaydatePad;
use crate::report::{self, Minute};
use crate::storage::{self, FileStorage};

/// Frames a second the SDK runs the game at, at most (the crate README
/// says why).
const REFRESH_RATE: f32 = 30.0;

/// What the front end does on its next frame.
enum Step {
    /// Show the loading text; it reaches the screen at the end of the frame.
    ShowLoading,
    /// Load the game.
    Load,
    /// The game is running.
    Running(Box<Running>),
}

/// The running game and what the front end keeps beside it.
struct Running {
    game: Game,
    pad: PlaydatePad,
    font: onebit::Font<'static>,
    /// The picture last drawn to the screen, `None` before the first.
    shown: Option<GlyphBuffer>,
    /// The one-bit frame the renderer draws into.
    frame: Box<Frame>,
    /// How fast the game ran since the numbers were last logged.
    minute: Minute,
    /// The bench build's extra renders.
    #[cfg(feature = "bench")]
    bench: super::bench::Bench,
}

/// The front end's state. The SDK calls the update callback on its only
/// thread, so this is never touched from two places at once.
struct Front(UnsafeCell<Option<Step>>);

// SAFETY: the Playdate runs the game on one thread (see `Front`).
unsafe impl Sync for Front {}

static FRONT: Front = Front(UnsafeCell::new(None));

/// Called by the shim once, at `kEventInit`, before the first frame.
#[unsafe(no_mangle)]
pub extern "C" fn trpg_playdate_init() {
    game_stack::init();
    set_refresh_rate(REFRESH_RATE);
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
            // The debug build (the Simulator's) checks the game's storage
            // on the real files first.
            if cfg!(debug_assertions) {
                game_stack::run(|| storage::check(SdkFiles));
                log("storage: the conformance suite passed");
            }
            let start = millis();
            let running = game_stack::run(start_game)
                .unwrap_or_else(|e| panic!("the game did not start: {e}"));
            let load_ms = millis().wrapping_sub(start);
            let lines = report::loaded(load_ms, game_stack::high_water());
            let mut file = String::new();
            for line in &lines {
                log(line);
                file.push_str(line);
                file.push('\n');
            }
            write_file(report::FILE, file.as_bytes());
            *step = Step::Running(running);
            // The first frame's time starts now, not before loading.
            reset_elapsed();
            false
        }
        Step::Running(running) => {
            let dt = elapsed();
            reset_elapsed();
            let (current, pushed) = buttons();
            let events = running.pad.poll(current, pushed);
            let changed = game_stack::run(|| running.frame(&events, dt));
            running.minute.seconds += dt;
            if running.minute.is_over() {
                let screen = running.game.top_screen().unwrap_or("no screen");
                for line in running.minute.lines(screen, game_stack::high_water()) {
                    log(&line);
                }
                #[cfg(feature = "bench")]
                for line in running.bench.lines() {
                    log(&line);
                }
                running.minute = Minute::default();
            }
            changed
        }
    }
}

/// Loads the game's data and starts the game at the title, as the
/// Playdate runs it: its console, palette and buttons, no keyboard, its
/// files for storage.
fn start_game() -> Result<Box<Running>, String> {
    let mut ctx = Ctx::embedded().map_err(|e| e.to_string())?;
    ctx.console = ConsoleSpec::PLAYDATE;
    ctx.keyboard = Keyboard::Absent;
    ctx.device = Device::Pad(PadKind::Playdate);
    // The debug tools are made of the test files, which a build without
    // `std` does not have (ADR-0057).
    ctx.debug_tools = false;
    ctx.tips_enabled = true;
    ctx.lessons = Lessons::On;
    ctx.idle_motion = IdleMotion::On;
    ctx.music_seed = u64::from(millis());
    ctx.key_prompt = KeyPrompt::Waiting;
    let mut ctx = ctx.with_storage(Box::new(FileStorage::new(SdkFiles)));
    // After the storage: loading the settings applies their colour theme.
    ctx.palette = playdate_palette(&ctx.content.palette)?;
    for warning in ctx.take_warnings() {
        log(&warning);
    }
    // The font lives as long as the game.
    let atlas: &'static FontAtlasDef = Box::leak(Box::new(ctx.content.font.clone()));
    let bits = trpg_content::bundle::bytes(ATLAS_1BIT_PATH).unwrap_or_default();
    let font = onebit::Font::new(atlas, bits)?;
    let pad = PlaydatePad::new(ctx.content.keymap.stick);
    Ok(Box::new(Running {
        game: Game::start(ctx),
        pad,
        font,
        shown: None,
        frame: Box::new([0; FRAME_BYTES]),
        minute: Minute::default(),
        #[cfg(feature = "bench")]
        bench: super::bench::Bench::new(),
    }))
}

/// The Playdate's palette (ADR-0074), over the default theme's names.
fn playdate_palette(base: &PaletteDef) -> Result<Palette, String> {
    let def = trpg_content::ink::load_playdate(base).map_err(|errors| {
        let errors: alloc::vec::Vec<String> = errors.iter().map(ToString::to_string).collect();
        errors.join("; ")
    })?;
    // check-text: not player text (a build problem for the log)
    Palette::new(&def).map_err(|missing| alloc::format!("the Playdate palette lacks {missing:?}"))
}

impl Running {
    /// Runs one frame of the game and, if its picture changed, redraws
    /// the parts that did; true if the screen changed.
    fn frame(&mut self, events: &[trpg_ui::RawInputEvent], dt: f32) -> bool {
        let start = elapsed();
        // A game that has quit keeps its last frame: the Playdate's own
        // Menu button leaves it.
        let _quit = self.game.frame(events, dt).quit;
        let framed = elapsed();
        self.minute.frame.add((framed - start) * 1000.0);
        let buffer = self.game.buffer();
        if self.shown.as_ref() == Some(buffer) {
            return false;
        }
        if let Some(shown) = &mut self.shown {
            onebit::render_changed(shown, buffer, &self.font, &(), &mut self.frame);
            shown.clone_from(buffer);
        } else {
            onebit::render(buffer, &self.font, &(), &mut self.frame);
            self.shown = Some(buffer.clone());
        }
        let changed = show(&self.frame);
        self.minute.render.add((elapsed() - framed) * 1000.0);
        #[cfg(feature = "bench")]
        self.bench.measure(buffer, &self.font, &self.frame);
        changed
    }
}
