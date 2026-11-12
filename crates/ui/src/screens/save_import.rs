//! The Import screen (ticket 0873, `docs/design/options.md`, *Saves
//! outside the game*), opened by the import key on the Profile screen and
//! the Load Game list (the End turn key: on menus it has no other use;
//! *the agent's starting rule*). Three states:
//!
//! 1. **Waiting for the file**: on the web the browser's picker is open
//!    ([`Ctx::ask_for_file`], asked in the frame of the press); on a
//!    desktop the screen says to drop the file on the window. The game's
//!    system hook hands the file over ([`save_files::hear`]); a file
//!    dropped anywhere else is ignored.
//! 2. **Refused**: the file isn't an exported save file.
//! 3. **The report**, before anything is written: a line per save in the
//!    file with its verdict, the build that made it if another did, where
//!    it goes, and "Import these saves?" with the cursor on Cancel.
//!    Import writes ([`Report::apply`]) and closes with an [`ImportDone`]
//!    for the screen below, which shows its notice.
//!
//! **The look is a skin** (ADR-0054): this module builds an
//! [`ImportView`]; [`glyph`] paints it.

pub mod glyph;

use crate::prelude::*;

use shuyi_core::made::Part;
use shuyi_ui::storage::bundle::{BundleError, Verdict};

use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::profiles::MAX_PROFILES;
use crate::save::playtime_text;
use crate::save_files::import::{Channel, Destination, ImportFailed, Imported, Report};
use crate::save_files::{self, IMPORT_EXTENSION};
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::widgets::{Ask, AskView};

/// What the screen reacts to: the cursor (scrolling the saves, choosing
/// an answer), Confirm and Cancel.
pub const ACTIONS: &[Action] = &[
    Action::CursorUp,
    Action::CursorDown,
    Action::CursorLeft,
    Action::CursorRight,
    Action::Confirm,
    Action::Cancel,
];

/// The key of "Import these saves?".
const QUESTION: &str = "save_import.question";
/// The key of the text of a file that isn't one of saves.
const NOT_A_FILE: &str = "save_import.not_a_file";
/// The key of the text of a file the platform couldn't open.
const NOT_OPENED: &str = "save_import.not_opened";

/// Saves shown at once.
const VISIBLE: usize = 16;

/// What the screen pops with after an import, for the screen below.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportDone {
    /// The notice to show: what was imported, where.
    pub notice: String,
    /// The slot of this profile the save went into, if it went into one.
    pub slot: Option<usize>,
}

/// What the platform handed over, read in the next update.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Arrived {
    /// A file, its name and text.
    File(String, String),
    /// Something the screen refuses, with the key of why.
    Refused(&'static str),
    /// The player closed the picker.
    Cancelled,
}

/// Where the screen is.
#[derive(Debug, Clone)]
enum State {
    /// Waiting for the file: in the picker, or to be dropped.
    Waiting {
        /// Whether a picker was opened.
        picker: bool,
    },
    /// The file was refused: the key of why.
    Refused(&'static str),
    /// The report on the file.
    Report {
        /// What importing it would do.
        report: Box<Report>,
        /// "Import these saves?"; `None` where it can't go anywhere.
        ask: Option<Ask<bool>>,
        /// The first save shown.
        top: usize,
        /// Why the last import failed.
        message: Option<String>,
    },
}

/// The Import screen: see the [module docs](self).
#[derive(Debug, Clone)]
pub struct ImportScreen {
    channel: Channel,
    state: State,
    arrived: Option<Arrived>,
}

/// One save of the file, as the report shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRowView {
    /// `Slot 03`, or the suspend save's name.
    pub label: String,
    /// The chapter, if the save reads.
    pub chapter: Option<String>,
    /// The playtime, if the save reads.
    pub playtime: Option<String>,
    /// What this build can do with it.
    pub verdict: String,
    /// Whether it will be imported.
    pub ready: bool,
}

/// What the Import screen shows, for a skin.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportView {
    /// The panel's title.
    pub title: String,
    /// While waiting: what to do.
    pub waiting: Option<String>,
    /// The file's name, once it came.
    pub file: Option<String>,
    /// Where it goes.
    pub to: Option<String>,
    /// Whether [`to`](Self::to) is a warning (a whole profile through Load
    /// Game, or no room for it).
    pub warning: bool,
    /// Which build made it, if another did.
    pub made_by: Option<String>,
    /// The saves shown.
    pub rows: Vec<ImportRowView>,
    /// Which saves are shown, when not all fit.
    pub range: Option<String>,
    /// What happens to its options and key bindings.
    pub options: Option<String>,
    /// "Import these saves?".
    pub question: Option<AskView<bool>>,
    /// Why the file was refused, or the import failed.
    pub message: Option<String>,
    /// The bottom help line.
    pub help: String,
}

impl ImportScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "save_import";

    /// Whether a screen offers Import: where the platform has files a
    /// player reaches (a desktop's drops, a browser's picker; not the
    /// Playdate).
    pub fn can_open(ctx: &Ctx) -> bool {
        save_files::can_export(&ctx.platform)
    }

    /// The screen asked from `channel`, waiting for the file: where the
    /// platform has a picker it is asked for now, in the frame of the
    /// press (a browser opens one only then). `None` where Import isn't
    /// offered.
    pub fn open(ctx: &mut Ctx, channel: Channel) -> Option<Self> {
        if !Self::can_open(ctx) {
            return None;
        }
        let picker = ctx.platform.can_pick_files();
        if picker {
            ctx.ask_for_file(&[IMPORT_EXTENSION]);
        }
        ctx.audio.menu(MenuSound::Select);
        Some(Self {
            channel,
            state: State::Waiting { picker },
            arrived: None,
        })
    }

    /// Whether the screen is waiting for the file.
    pub fn is_waiting(&self) -> bool {
        matches!(self.state, State::Waiting { .. })
    }

    /// The player picked the file `name` holding `text`.
    pub fn chosen(&mut self, name: &str, text: String) {
        self.arrive(Arrived::File(name.to_owned(), text));
    }

    /// The picker gave no file: the player closed it (`cancelled`), or it
    /// was refused.
    pub fn not_chosen(&mut self, cancelled: bool) {
        self.arrive(if cancelled {
            Arrived::Cancelled
        } else {
            Arrived::Refused(NOT_OPENED)
        });
    }

    /// The file `name` holding `bytes` was dropped on the window.
    pub fn dropped(&mut self, name: &str, bytes: Vec<u8>) {
        self.arrive(match String::from_utf8(bytes) {
            Ok(text) => Arrived::File(name.to_owned(), text),
            Err(_) => Arrived::Refused(NOT_A_FILE),
        });
    }

    /// Keeps what came for the next update, while waiting.
    fn arrive(&mut self, arrived: Arrived) {
        if self.is_waiting() {
            self.arrived = Some(arrived);
        }
    }

    /// Reads the file `name` holding `text` into the report.
    fn read(&mut self, ctx: &Ctx, name: &str, text: &str) {
        self.state = match Report::read(ctx, name, text, self.channel) {
            Ok(report) => {
                let answers = [(true, "save_import.import"), (false, "save_import.cancel")];
                let ask = (report.to != Destination::Full).then(|| {
                    Ask::new(QUESTION, answers)
                        .focused_on(false)
                        .cancelled_by(false)
                });
                State::Report {
                    report: Box::new(report),
                    ask,
                    top: 0,
                    message: None,
                }
            }
            Err(BundleError::Version { .. }) => State::Refused("save_import.not_this_version"),
            Err(_) => State::Refused(NOT_A_FILE),
        };
    }

    /// One action. Returns the screen's transition, if it is done.
    fn act(&mut self, ctx: &mut Ctx, action: Action) -> Option<Transition> {
        let back = |ctx: &mut Ctx| {
            ctx.audio.menu(MenuSound::Cancel);
            Transition::Pop
        };
        let State::Report {
            report,
            ask,
            top,
            message,
        } = &mut self.state
        else {
            let leaves =
                action == Action::Cancel || (action == Action::Confirm && !self.is_waiting());
            return leaves.then(|| back(ctx));
        };
        if matches!(action, Action::CursorUp | Action::CursorDown) {
            let last = report.saves.len().saturating_sub(VISIBLE);
            let moved = if action == Action::CursorDown {
                (*top + 1).min(last)
            } else {
                top.saturating_sub(1)
            };
            if moved != *top {
                *top = moved;
                ctx.audio.menu(MenuSound::Move);
            }
            return None;
        }
        let Some(ask) = ask else {
            return matches!(action, Action::Confirm | Action::Cancel).then(|| back(ctx));
        };
        if !ask.handle(action, &mut ctx.audio)? {
            return Some(back(ctx));
        }
        match report.apply(ctx) {
            Ok(done) => {
                ctx.audio.menu(MenuSound::Select);
                let slot = match done {
                    Imported::Slot(slot) => Some(slot),
                    _ => None,
                };
                let notice = done_text(ctx, &done);
                Some(Transition::pop_with(ImportDone { notice, slot }))
            }
            Err(failed) => {
                ctx.audio.menu(MenuSound::Denied);
                *message = Some(match failed {
                    ImportFailed::Full => full_text(ctx),
                    ImportFailed::Storage => ctx.text("save_import.failed").to_owned(),
                });
                None
            }
        }
    }

    /// The screen as it is now, as plain data for a skin to paint.
    pub fn view(&self, ctx: &Ctx) -> ImportView {
        let mut view = ImportView {
            title: ctx.text("save_import.title").to_owned(),
            waiting: None,
            file: None,
            to: None,
            warning: false,
            made_by: None,
            rows: Vec::new(),
            range: None,
            options: None,
            question: None,
            message: None,
            help: ctx.text_with("save_import.help.back", &[]),
        };
        match &self.state {
            State::Waiting { picker } => {
                let key = if *picker {
                    "save_import.choose"
                } else {
                    "save_import.drop"
                };
                view.waiting = Some(ctx.text(key).to_owned());
            }
            State::Refused(why) => view.message = Some(ctx.text(why).to_owned()),
            State::Report {
                report,
                ask,
                top,
                message,
            } => {
                report_view(ctx, report, *top, &mut view);
                view.question = ask.as_ref().map(|ask| ask.view(ctx));
                view.message.clone_from(message);
                if ask.is_some() {
                    view.help = ctx.text_with("save_import.help.report", &[]);
                }
            }
        }
        view
    }
}

/// Fills `view` with `report`, its saves from `top`.
fn report_view(ctx: &Ctx, report: &Report, top: usize, view: &mut ImportView) {
    view.file = Some(ctx.text_with("save_import.file", &[("file", &report.file)]));
    let (to, warning) = match &report.to {
        Destination::NewProfile { name, warned } => {
            let key = if *warned {
                "save_import.to.profile_warning"
            } else {
                "save_import.to.profile"
            };
            (ctx.text_with(key, &[("name", name)]), *warned)
        }
        Destination::Full => (full_text(ctx), true),
        Destination::Slot { .. } => (ctx.text("save_import.to.slot").to_owned(), false),
    };
    view.to = Some(to);
    view.warning = warning;
    view.made_by = report.plan.mismatch.as_ref().map(|mismatch| {
        let game = mismatch.parts.iter().find(|d| d.part == Part::Game);
        match game {
            Some(d) => ctx.text_with("save_import.made_by", &[("made", &d.made), ("now", &d.now)]),
            None => ctx.text_with(
                "save_import.made_by_build",
                &[("now", &save_files::VERSION)],
            ),
        }
    });
    let shown = top..(top + VISIBLE).min(report.saves.len());
    if report.saves.len() > VISIBLE {
        let (first, last, count) = (shown.start + 1, shown.end, report.saves.len());
        view.range = Some(ctx.text_with(
            "save.range",
            &[("first", &first), ("last", &last), ("count", &count)],
        ));
    }
    view.rows = report.saves[shown]
        .iter()
        .map(|line| ImportRowView {
            label: match line.slot {
                Some(slot) => ctx.text_with("save_import.slot", &[("slot", &two_digits(slot))]),
                None => ctx.text("save_import.suspend").to_owned(),
            },
            chapter: line.chapter.clone(),
            playtime: line.playtime_s.map(playtime_text),
            verdict: verdict_text(ctx, line.verdict, &report.to),
            ready: line.verdict == Verdict::Ready,
        })
        .collect();
    view.options = report.options.map(|verdict| match report.to {
        Destination::Slot { .. } => ctx.text("save_import.options_left_out").to_owned(),
        _ => ctx.text_with(
            "save_import.options",
            &[("verdict", &verdict_text(ctx, verdict, &Destination::Full))],
        ),
    });
}

/// `slot` as two digits, as Load Game numbers its slots.
fn two_digits(slot: usize) -> String {
    format!("{slot:02}")
}

/// A verdict in words: where a ready save goes when it goes into a slot.
fn verdict_text(ctx: &Ctx, verdict: Verdict, to: &Destination) -> String {
    match verdict {
        Verdict::Ready => match to {
            Destination::Slot { slot, replaces } => {
                let key = if *replaces {
                    "save_import.ready_replaces"
                } else {
                    "save_import.ready_into"
                };
                ctx.text_with(key, &[("slot", &two_digits(*slot))])
            }
            _ => ctx.text("save_import.ready").to_owned(),
        },
        Verdict::Incompatible { found, wants } => ctx.text_with(
            "save_import.incompatible",
            &[("found", &found), ("wants", &wants)],
        ),
        _ => ctx.text("save_import.unreadable").to_owned(),
    }
}

/// "There can be at most 8 profiles…".
fn full_text(ctx: &Ctx) -> String {
    ctx.text_with("save_import.to.full", &[("n", &MAX_PROFILES)])
}

/// The notice after an import.
fn done_text(ctx: &Ctx, done: &Imported) -> String {
    match done {
        Imported::Profile {
            name,
            saves,
            refused,
        } => ctx.text_with(
            "save_import.done.profile",
            &[("name", name), ("imported", saves), ("refused", refused)],
        ),
        Imported::Slot(slot) => {
            ctx.text_with("save_import.done.slot", &[("slot", &two_digits(*slot))])
        }
        Imported::Nothing { refused } => {
            ctx.text_with("save_import.done.none", &[("refused", refused)])
        }
    }
}

impl Screen<Shuyi> for ImportScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn actions(&self) -> Option<Vec<Action>> {
        Some(ACTIONS.to_vec())
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        match self.arrived.take() {
            Some(Arrived::File(name, text)) => self.read(ctx, &name, &text),
            Some(Arrived::Refused(why)) => self.state = State::Refused(why),
            Some(Arrived::Cancelled) => {
                ctx.audio.menu(MenuSound::Cancel);
                return Transition::Pop;
            }
            None => {}
        }
        for &action in &input.actions {
            if let Some(done) = self.act(ctx, action) {
                return done;
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn core::any::Any> {
        Some(self)
    }
}
