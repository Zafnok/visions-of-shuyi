//! The entry points `shim/shim.c` calls: once at start, then once per
//! frame from the SDK's update callback.
//!
//! The first frame shows the loading card (ticket 1036, `crate::card`),
//! drawn with the game's font. The next loads the game on the game's own
//! stack, timing every file it reads from the bundle; then the game runs
//! (ticket 1033): each frame reads the buttons and the crank (ticket
//! 1035), hands the game an entry of the system menu chosen since the last
//! frame, calls `Game::frame` with the time since the last frame, and when
//! the picture changed redraws what changed with `trpg_ui::onebit` (ticket
//! 1055) and copies it to the screen. Once a minute it logs how fast that
//! went.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::UnsafeCell;
use core::ffi::c_int;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use trpg_ui::game::system_menu::{self, SystemMenuEntry};
use trpg_ui::input::{Crank, PadKind};
use trpg_ui::onebit::{self, FRAME_BYTES, Frame};
use trpg_ui::{ConsoleSizes, ConsoleSpec, Game, GlyphBuffer, RawInputEvent};

use super::files::SdkFiles;
use super::{
    MenuItem, add_menu_item, buttons, crank, elapsed, game_stack, log, millis, reset_elapsed,
    set_menu_item_title, set_refresh_rate, show, write_file,
};
use crate::menu::{self, Chosen, Titles};
use crate::pads::PlaydatePad;
use crate::report::{self, Minute, Reads};
use crate::storage::{self, FileStorage};
use crate::{card, start};

/// Frames a second the SDK runs the game at, at most (the crate README
/// says why).
const REFRESH_RATE: f32 = 30.0;

/// What the front end does on its next frame.
enum Step {
    /// Show the loading card; it reaches the screen at the end of the frame.
    ShowCard,
    /// Load the game, the card on the screen.
    Load(Box<Card>),
    /// The game is running.
    Running(Box<Running>),
}

/// The loading card on the screen, and what the game takes over from it.
struct Card {
    /// The game's font, read before the rest of the data.
    font: onebit::Font<'static>,
    /// The frame the card was drawn into.
    frame: Box<Frame>,
    /// The report's first line: when the card was shown.
    shown: String,
}

/// The running game and what the front end keeps beside it.
struct Running {
    game: Game,
    pad: PlaydatePad,
    crank: Crank,
    /// The game's entries in the system menu, once added, and their names.
    menu: [Option<MenuItem>; menu::ENTRIES],
    titles: Titles,
    font: onebit::Font<'static>,
    /// The picture last drawn to the screen, `None` before the first.
    shown: Option<GlyphBuffer>,
    /// The one-bit frame the renderer draws into.
    frame: Box<Frame>,
    /// How fast the game ran since the numbers were last logged.
    minute: Minute,
    /// The load report, until the game's first frame is shown: then it is
    /// logged and written with when that was.
    report: Option<Report>,
    /// The bench build's extra renders.
    #[cfg(feature = "bench")]
    bench: super::bench::Bench,
}

/// The load report.
struct Report {
    /// What the log says.
    log: Vec<String>,
    /// Every file read while loading, with its time: for the file only.
    each: Vec<String>,
}

/// The front end's state. The SDK calls the update callback on its only
/// thread, so this is never touched from two places at once.
struct Front(UnsafeCell<Option<Step>>);

// SAFETY: the Playdate runs the game on one thread (see `Front`).
unsafe impl Sync for Front {}

static FRONT: Front = Front(UnsafeCell::new(None));

/// The system menu's entry chosen since the last frame.
static CHOSEN: Chosen = Chosen::new();

/// The files read while the game loads; `None` outside it.
struct Watched(UnsafeCell<Option<Reads>>);

// SAFETY: the Playdate runs the game on one thread (see `Front`).
unsafe impl Sync for Watched {}

static READS: Watched = Watched(UnsafeCell::new(None));

/// `millis` when the game started.
static STARTED: AtomicU32 = AtomicU32::new(0);

/// Whether the game has loaded: from then on the debug build logs each
/// file read.
static LOADED: AtomicBool = AtomicBool::new(false);

/// Milliseconds since the game started.
fn since_start() -> u32 {
    millis().wrapping_sub(STARTED.load(Ordering::Relaxed))
}

/// Called by the shim once, at `kEventInit`, before the first frame.
#[unsafe(no_mangle)]
pub extern "C" fn trpg_playdate_init() {
    STARTED.store(millis(), Ordering::Relaxed);
    game_stack::init();
    set_refresh_rate(REFRESH_RATE);
    // Only fails when set already, which nothing else does.
    let _ = trpg_content::bundle::watch(on_read);
    // SAFETY: nothing else holds the state (see `Front`).
    unsafe { *FRONT.0.get() = Some(Step::ShowCard) };
}

/// Called by the shim when the player chooses the game's system menu entry
/// `entry` (its index in [`SystemMenuEntry::ALL`]), between two frames.
#[unsafe(no_mangle)]
pub extern "C" fn trpg_playdate_menu_chosen(entry: c_int) {
    CHOSEN.set(entry);
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

/// Told of every file the game reads from its bundle: noted with its time
/// while loading; once loaded, logged in the debug build. The card's font
/// is read before either.
fn on_read(path: &str, found: bool) {
    // SAFETY: one thread (see `Front`), and nothing here reads files.
    if let Some(reads) = unsafe { &mut *READS.0.get() } {
        reads.add(path, found, elapsed() * 1000.0);
    } else if cfg!(debug_assertions) && LOADED.load(Ordering::Relaxed) {
        let missing = if found { "" } else { ", not in the bundle" };
        log(&alloc::format!(
            "bundle: read {path} while running{missing}"
        ));
    }
}

/// One frame; true when the screen changed.
fn update(step: &mut Step) -> bool {
    match step {
        Step::ShowCard => {
            let mut frame = Box::new([0; FRAME_BYTES]);
            let font = game_stack::run(|| {
                let font = card::font().unwrap_or_else(|e| panic!("no font for the card: {e}"));
                onebit::render(&card::buffer(ConsoleSpec::PLAYDATE), &font, &(), &mut frame);
                font
            });
            let changed = show(&frame);
            let shown = report::card_shown(since_start());
            log(&shown);
            *step = Step::Load(Box::new(Card { font, frame, shown }));
            changed
        }
        Step::Load(_) => {
            let Step::Load(card) = core::mem::replace(step, Step::ShowCard) else {
                return false;
            };
            *step = Step::Running(load(card));
            // The first frame's time starts now, not before loading.
            reset_elapsed();
            false
        }
        Step::Running(running) => {
            let dt = elapsed();
            reset_elapsed();
            let (current, pushed) = buttons();
            let mut events = running.pad.poll(current, pushed);
            let (turned, docked) = crank();
            let kind = PadKind::Playdate;
            events.extend(RawInputEvent::from_crank(
                &mut running.crank,
                turned,
                docked,
                kind,
            ));
            let chosen = CHOSEN.take();
            let changed = game_stack::run(|| {
                if let Some(entry) = chosen {
                    system_menu::choose(&mut running.game, entry);
                }
                running.frame(&events, dt)
            });
            running.name_menu();
            if changed && let Some(report) = running.report.take() {
                write_report(&report);
            }
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

/// Loads the game behind the card, noting every file it reads, and logs
/// how long that took.
fn load(mut card: Box<Card>) -> Box<Running> {
    // The debug build (the Simulator's) checks the game's storage on the
    // real files first.
    if cfg!(debug_assertions) {
        game_stack::run(|| storage::check(SdkFiles));
        log("storage: the conformance suite passed");
    }
    let shown = core::mem::take(&mut card.shown);
    // SAFETY: one thread (see `Front`).
    unsafe { *READS.0.get() = Some(Reads::default()) };
    reset_elapsed();
    let start = millis();
    let mut running = game_stack::run(|| start_game(card))
        .unwrap_or_else(|e| panic!("the game did not start: {e}"));
    let load_ms = millis().wrapping_sub(start);
    let end_ms = elapsed() * 1000.0;
    // SAFETY: one thread (see `Front`).
    let reads = unsafe { (*READS.0.get()).take() }.unwrap_or_default();
    LOADED.store(true, Ordering::Relaxed);
    let mut lines = Vec::from([shown]);
    lines.extend(report::loaded(load_ms, game_stack::high_water()));
    lines.extend(reads.summary(end_ms));
    for line in &lines {
        log(line);
    }
    let each = reads.each(end_ms);
    if cfg!(debug_assertions) {
        for line in &each {
            log(line);
        }
    }
    running.report = Some(Report { log: lines, each });
    running
}

/// Logs when the game's first frame was shown, and writes the whole load
/// report to its file.
fn write_report(report: &Report) {
    let shown = report::game_shown(since_start());
    log(&shown);
    let mut file = String::new();
    for line in report.log.iter().chain([&shown]) {
        file.push_str(line);
        file.push('\n');
    }
    file.push_str("Every file read while loading, ms to the next read:\n");
    for line in &report.each {
        file.push_str(line);
        file.push('\n');
    }
    write_file(report::FILE, file.as_bytes());
}

/// Notes in the load report that the step `name` of the start began.
fn mark(name: &str) {
    // SAFETY: one thread (see `Front`).
    if let Some(reads) = unsafe { &mut *READS.0.get() } {
        reads.mark(name, elapsed() * 1000.0);
    }
}

/// Loads the game's data and starts the game at the title, as the
/// Playdate runs it (`crate::start`), with its files for storage, drawn
/// with the card's font into its frame.
fn start_game(card: Box<Card>) -> Result<Box<Running>, String> {
    let Card { font, frame, .. } = *card;
    let ctx = start::data(u64::from(millis()))?;
    mark("the player's settings and saves");
    let mut ctx = start::with_storage(ctx, Box::new(FileStorage::new(SdkFiles)))?;
    for warning in ctx.take_warnings() {
        log(&warning);
    }
    let pad = PlaydatePad::new(ctx.game.content.keymap.stick);
    let crank = Crank::new(ctx.game.content.keymap.crank);
    mark("the title");
    let mut running = Box::new(Running {
        game: trpg_ui::game::start(ctx),
        pad,
        crank,
        menu: [None, None, None],
        titles: Titles::default(),
        font,
        shown: None,
        frame,
        minute: Minute::default(),
        report: None,
        #[cfg(feature = "bench")]
        bench: super::bench::Bench::new(),
    });
    running.name_menu();
    Ok(running)
}

impl Running {
    /// Adds the game's entries to the system menu the first time, and
    /// renames those whose name changed since (another language).
    fn name_menu(&mut self) {
        let ctx = self.game.ctx();
        let now = SystemMenuEntry::ALL.map(|e| e.label(ctx));
        for i in self.titles.update(now) {
            let title = self.titles.get(i);
            match &mut self.menu[i] {
                Some(item) => set_menu_item_title(item, title),
                slot @ None => *slot = add_menu_item(title, i),
            }
        }
    }

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
