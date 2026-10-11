//! The Key bindings screen (tickets 0815 and 0816; `docs/design/controls.md`,
//! *Rebinding keys* and *Rebinding buttons*): every rebindable action with
//! its [`SLOTS`] key slots for the layout in use, or, on its controller
//! [`Side`], its [`SLOTS`] button slots. A row at the top switches between
//! the two. Confirm on a slot captures the next key or button pressed into
//! it; one taken from another slot moves; leaving is blocked while a
//! required action has no key or no button; Restore defaults asks first.
//!
//! The screen edits a copy of the layout's [`LayoutBindings`] and of the
//! [`PadBindings`] and hands them to [`Ctx`] when it closes, so it is steered
//! with the keys and buttons the player had when they opened it: moving
//! every cursor key elsewhere can't trap them here.
//!
//! No controller button is fixed, so the screen works with only a
//! controller: a button goes in the slot when it is let go, holding any
//! button backs out of a capture, and when the controller was used last
//! Confirm on a slot offers [`CHOICES`] (change it or clear it).
//!
//! Capture is the one place that reads raw key presses
//! ([`FrameInput::pressed_chords`]) and raw button presses
//! ([`FrameInput::pressed_buttons`]); which keys back out or clear a slot is
//! asked of `input` ([`is_capture_abort`], [`is_clear_slot`]), and a
//! button's name of [`PadKind::button_name`], never named here (the
//! `keyboard-input` skill).
//!
//! **The look is a skin** (ADR-0054). This module decides what the screen
//! shows and does, and says it as plain data: a [`KeyBindingsView`]
//! ([`KeyBindingsScreen::view`]). [`glyph::paint`] draws that view as
//! glyphs. Tests of what happened read the view; only [`glyph`]'s tests
//! read cells.

pub mod glyph;
pub mod view;

pub use view::{
    ChoicesView, GroupView, KeyBindingsView, QuestionView, RestoreView, RowView, SlotView,
    SwitchView,
};

use super::layout_picker;
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::{
    Action, Button, CAPTURE_BUTTON_PROMPT, CAPTURE_PROMPT, Chord, Device, Keymap, Layout,
    LayoutBindings, PadBindings, PadKind, SLOTS, capture_abort_key_name, clear_slot_key_name,
    is_capture_abort, is_clear_slot,
};
use crate::screen::{Ctx, FrameInput, Screen, Transition};
use crate::tips::fill_text;
use crate::widgets::help::{HelpKeys, NOT_MAPPED, SEPARATOR};

/// Every rebindable action with the text key of its player-facing label
/// (`Unit info`), in the order the screen lists them (Nick's pick, ticket
/// 0815): the required actions, then the optional ones, as in
/// `controls.md`'s *Required and optional* table.
pub const ROWS: [(Action, &str); 16] = [
    (Action::CursorUp, "key_bindings.action.cursor_up"),
    (Action::CursorDown, "key_bindings.action.cursor_down"),
    (Action::CursorLeft, "key_bindings.action.cursor_left"),
    (Action::CursorRight, "key_bindings.action.cursor_right"),
    (Action::Confirm, "key_bindings.action.confirm"),
    (Action::Cancel, "key_bindings.action.cancel"),
    (Action::EndTurn, "key_bindings.action.end_turn"),
    (Action::Select, "key_bindings.action.select"),
    (
        Action::ConfirmEndTurn,
        "key_bindings.action.confirm_end_turn",
    ),
    (Action::PrevUnit, "key_bindings.action.prev_unit"),
    (Action::NextUnit, "key_bindings.action.next_unit"),
    (Action::Info, "key_bindings.action.info"),
    (Action::DangerZone, "key_bindings.action.danger_zone"),
    (Action::ToggleAutoEnd, "key_bindings.action.toggle_auto_end"),
    (Action::Rewind, "key_bindings.action.rewind"),
    (Action::Menu, "key_bindings.action.menu"),
];

/// Which bindings the screen shows and edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The keys of the layout in use.
    Keyboard,
    /// The controller's buttons (one setup for both layouts).
    Controller,
}

impl Side {
    /// The text key of what a slot on this side shows while it waits.
    fn prompt(self) -> &'static str {
        match self {
            Self::Keyboard => CAPTURE_PROMPT,
            Self::Controller => CAPTURE_BUTTON_PROMPT,
        }
    }
}

/// Text key of the screen's title.
pub const TITLE: &str = "key_bindings.title";
/// Text key of the switch row's name for the keyboard side, which the
/// layout's name follows.
pub const KEYBOARD_SIDE: &str = "key_bindings.keyboard";
/// Text key of the switch row's name for the controller side.
pub const CONTROLLER_SIDE: &str = "key_bindings.controller";
/// Text key of the heading over the required actions, on the keyboard
/// side.
pub const REQUIRED_HEADING: &str = "key_bindings.must_have_key";
/// Text key of the heading over the required actions, on the controller
/// side.
pub const REQUIRED_BUTTON_HEADING: &str = "key_bindings.must_have_button";
/// Text key of the heading over the optional actions.
pub const OPTIONAL_HEADING: &str = "key_bindings.optional";
/// Text key of the row under the actions that puts the shown side's
/// defaults back.
pub const RESTORE_DEFAULTS: &str = "key_bindings.restore_defaults";
/// Text key of the help while a slot waits for a key.
pub const CAPTURE_HELP: &str = "key_bindings.capture_help";
/// Text key of the help for backing out of a capture with only a
/// controller.
pub const HOLD_TO_CANCEL: &str = "key_bindings.hold_to_cancel";
/// Text keys of what Confirm on a slot offers when the controller was used
/// last: put another key or button in it, or empty it.
pub const CHOICES: [&str; 2] = ["key_bindings.change", "key_bindings.clear"];
/// Text key of what shows when the key pressed for a slot is reserved.
pub const RESERVED_MESSAGE: &str = "key_bindings.reserved";
/// How long the row that just lost its key to another slot stays
/// highlighted, in seconds (*tunable*).
pub const MOVED_FLASH_SECS: f32 = 1.5;
/// How long a controller button is held to back out of a capture, in
/// seconds (*tunable*). Let go sooner, it goes in the slot.
pub const HOLD_TO_CANCEL_SECS: f32 = 1.0;

/// [`KeyBindingsScreen::row`] on Restore defaults, the row after the
/// actions.
const RESTORE_ROW: usize = ROWS.len();
/// [`KeyBindingsScreen::row`] on the keyboard / controller switch: above
/// the first action, so one up from it and one down from Restore defaults.
const SWITCH_ROW: usize = ROWS.len() + 1;

/// The actions that move the focus.
const CURSOR_ACTIONS: [Action; 4] = [
    Action::CursorUp,
    Action::CursorDown,
    Action::CursorLeft,
    Action::CursorRight,
];

/// The player-facing name of `action` ([`ROWS`]) in `ctx`'s language; its
/// keymap name for an action not on the screen (Debug).
pub fn label(ctx: &Ctx, action: Action) -> &str {
    ROWS.iter()
        .find(|&&(a, _)| a == action)
        .map_or(action.name(), |&(_, key)| ctx.text(key))
}

/// The blocked-leave message for `action` on `side` (`Give Cancel a key
/// first`, `Give Cancel a button first`).
pub fn blocked_message(ctx: &Ctx, side: Side, action: Action) -> String {
    let key = match side {
        Side::Keyboard => "key_bindings.blocked_key",
        Side::Controller => "key_bindings.blocked_button",
    };
    ctx.text_with(key, &[("action", &label(ctx, action))])
}

/// The Key bindings screen for one layout and the controller. Cancel saves
/// and closes it, unless a required action has no key or no button.
#[derive(Debug, Clone)]
pub struct KeyBindingsScreen {
    /// The layout being edited.
    layout: Layout,
    /// The keymap the screen was opened with: what steers it (`Game` keeps
    /// using it until the edits are saved on leaving) and what its help
    /// text names.
    opened_with: Keymap,
    /// Which bindings are shown.
    side: Side,
    /// The edited copy of the layout's keys, saved on leaving.
    bindings: LayoutBindings,
    /// The edited copy of the controller's buttons, saved on leaving.
    pad: PadBindings,
    /// Focused row: an index into [`ROWS`], [`RESTORE_ROW`] or
    /// [`SWITCH_ROW`].
    row: usize,
    /// Focused slot (kept while on a row that has none).
    slot: usize,
    /// Whether the focused slot is waiting for a key or button.
    capturing: bool,
    /// The controller button held since it went down during a capture, and
    /// for how many seconds.
    held: Option<(Button, f32)>,
    /// The focused line of [`CHOICES`], while they are open.
    choice: Option<usize>,
    /// The message under the panel, if any.
    message: Option<String>,
    /// The action that just lost a key to another slot, and the seconds its
    /// row stays highlighted.
    moved: Option<(Action, f32)>,
    /// Whether the "restore the defaults?" question is open.
    asking_restore: bool,
    /// Set when a capture ends with a cursor key or button held: cursor
    /// moves are ignored until none is, because the one just pressed for
    /// the slot (or held to back out) may be one under the opened-with
    /// keymap, still down and repeating.
    await_release: bool,
}

impl KeyBindingsScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "key_bindings";

    /// The screen for the layout in use (right-handed if none is chosen
    /// yet), focused on the first action's first slot, showing the side the
    /// player pressed last: the controller's buttons after a button press,
    /// else the keys.
    pub fn new(ctx: &Ctx) -> Self {
        let layout = ctx.layout().unwrap_or(Layout::RightHanded);
        let side = match ctx.device {
            Device::Keyboard => Side::Keyboard,
            Device::Pad(_) => Side::Controller,
        };
        Self {
            layout,
            opened_with: ctx.keymap.clone(),
            side,
            bindings: ctx.layout_bindings(layout),
            pad: ctx.pad_bindings(),
            row: 0,
            slot: 0,
            capturing: false,
            held: None,
            choice: None,
            message: None,
            moved: None,
            asking_restore: false,
            await_release: false,
        }
    }

    /// The same screen editing `bindings` instead of the saved ones.
    #[cfg(test)]
    fn with_bindings(mut self, bindings: LayoutBindings) -> Self {
        self.bindings = bindings;
        self
    }

    /// The edited keys (saved when the screen closes).
    pub fn bindings(&self) -> &LayoutBindings {
        &self.bindings
    }

    /// The edited controller buttons (saved when the screen closes).
    pub fn pad_bindings(&self) -> &PadBindings {
        &self.pad
    }

    /// Which bindings are shown.
    pub fn side(&self) -> Side {
        self.side
    }

    /// The focused action and slot; `None` on Restore defaults and on the
    /// keyboard / controller switch.
    pub fn focus(&self) -> Option<(Action, usize)> {
        ROWS.get(self.row).map(|&(action, _)| (action, self.slot))
    }

    /// Whether the focus is on the keyboard / controller switch.
    pub fn is_on_switch(&self) -> bool {
        self.row == SWITCH_ROW
    }

    /// Whether the focused slot is waiting for a key or button.
    pub fn is_capturing(&self) -> bool {
        self.capturing
    }

    /// The focused line of [`CHOICES`], while they are open.
    pub fn choice(&self) -> Option<usize> {
        self.choice
    }

    /// Whether the screen is asking before it restores the defaults.
    pub fn is_asking_restore(&self) -> bool {
        self.asking_restore
    }

    /// The question asked before restoring the shown side's defaults.
    pub fn restore_question(&self, ctx: &Ctx) -> String {
        match self.side {
            Side::Keyboard => {
                let layout = layout_picker::label(ctx, self.layout);
                ctx.text_with("key_bindings.restore_keys", &[("layout", &layout)])
            }
            Side::Controller => ctx.text("key_bindings.restore_buttons").to_owned(),
        }
    }

    /// The message under the panel, if any.
    pub fn message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    /// The names in `action`'s slots on the shown side; buttons as the
    /// controller used last names them (an Xbox pad's names until one is
    /// used).
    fn slot_names(&self, ctx: &Ctx, action: Action) -> [Option<String>; SLOTS] {
        match self.side {
            Side::Keyboard => self
                .bindings
                .slots(action)
                .map(|c| c.map(|c| c.to_string())),
            Side::Controller => {
                let kind = match ctx.device {
                    Device::Pad(kind) => kind,
                    Device::Keyboard => PadKind::default(),
                };
                let name = |b| kind.button_name(b).to_owned();
                self.pad.slots(action).map(|b| b.map(name))
            }
        }
    }

    /// Whether `action` has nothing in any slot on the shown side.
    fn is_unmapped(&self, action: Action) -> bool {
        match self.side {
            Side::Keyboard => self.bindings.is_unmapped(action),
            Side::Controller => self.pad.is_unmapped(action),
        }
    }

    /// The first row of the screen whose required action has nothing on
    /// `side`.
    fn first_unmapped(&self, side: Side) -> Option<Action> {
        let unmapped = match side {
            Side::Keyboard => self.bindings.unmapped_required(),
            Side::Controller => self.pad.unmapped_required(),
        };
        let mut rows = ROWS.iter().map(|&(action, _)| action);
        rows.find(|action| unmapped.contains(action))
    }

    /// Counts the moved-row highlight down.
    fn tick(&mut self, dt: f32) {
        if let Some((_, left)) = &mut self.moved {
            *left -= dt;
            // A NaN frame time ends the highlight too.
            if left.is_nan() || *left <= 0.0 {
                self.moved = None;
            }
        }
    }

    /// Moves the focus one row up or down, wrapping: the actions, Restore
    /// defaults, the switch, and the first action again.
    fn move_row(&mut self, ctx: &mut Ctx, down: bool) {
        let rows = ROWS.len() + 2;
        self.row = if down {
            (self.row + 1) % rows
        } else {
            (self.row + rows - 1) % rows
        };
        self.message = None;
        ctx.audio.menu(MenuSound::Move);
    }

    /// Left or right: on the switch, shows the keyboard (left) or the
    /// controller (right); on a slot, moves the focus one slot, stopping at
    /// the ends. Does nothing on Restore defaults.
    fn move_across(&mut self, ctx: &mut Ctx, right: bool) {
        if self.is_on_switch() {
            let side = if right {
                Side::Controller
            } else {
                Side::Keyboard
            };
            self.show(ctx, side);
            return;
        }
        if self.focus().is_none() {
            return;
        }
        let to = if right {
            (self.slot + 1).min(SLOTS - 1)
        } else {
            self.slot.saturating_sub(1)
        };
        if to != self.slot {
            self.slot = to;
            self.message = None;
            ctx.audio.menu(MenuSound::Move);
        }
    }

    /// Shows `side`, if it isn't shown already.
    fn show(&mut self, ctx: &mut Ctx, side: Side) {
        if side != self.side {
            self.side = side;
            self.message = None;
            self.moved = None;
            ctx.audio.menu(MenuSound::Move);
        }
    }

    /// Confirm: on a slot, starts capturing (after the controller was used,
    /// opens the [`CHOICES`] instead); on Restore defaults, asks whether to
    /// restore; on the switch, shows the other side.
    fn confirm(&mut self, ctx: &mut Ctx) {
        self.message = None;
        if self.is_on_switch() {
            let other = match self.side {
                Side::Keyboard => Side::Controller,
                Side::Controller => Side::Keyboard,
            };
            self.show(ctx, other);
            return;
        }
        if self.focus().is_none() {
            self.asking_restore = true;
        } else if matches!(ctx.device, Device::Pad(_)) {
            self.choice = Some(0);
        } else {
            self.capturing = true;
        }
        ctx.audio.menu(MenuSound::Select);
    }

    /// An action while the [`CHOICES`] are open: up and down move between
    /// them, Confirm takes the focused one (capture, or empty the slot) and
    /// Cancel closes them. Returns whether they closed.
    fn answer_choice(&mut self, ctx: &mut Ctx, action: Action) -> bool {
        let Some(line) = self.choice else {
            return true;
        };
        match action {
            Action::CursorUp | Action::CursorDown => {
                self.choice = Some((line + 1) % CHOICES.len());
                ctx.audio.menu(MenuSound::Move);
                return false;
            }
            Action::Confirm if line == 0 => {
                self.capturing = true;
                ctx.audio.menu(MenuSound::Select);
            }
            Action::Confirm => {
                if !self.clear_slot(ctx) {
                    ctx.audio.menu(MenuSound::Select);
                }
            }
            Action::Cancel => ctx.audio.menu(MenuSound::Cancel),
            _ => return false,
        }
        self.choice = None;
        true
    }

    /// An action while the restore question is open: Confirm restores the
    /// shown side's defaults, Cancel backs out; either closes the question
    /// (and returns `true`). Nothing else does anything.
    fn answer_restore(&mut self, ctx: &mut Ctx, action: Action) -> bool {
        match action {
            Action::Confirm => {
                let def = &ctx.content.keymap;
                match self.side {
                    Side::Keyboard => self.bindings = LayoutBindings::defaults(def, self.layout),
                    Side::Controller => self.pad = PadBindings::defaults(def),
                }
                self.moved = None;
                ctx.audio.menu(MenuSound::Select);
            }
            Action::Cancel => ctx.audio.menu(MenuSound::Cancel),
            _ => return false,
        }
        self.asking_restore = false;
        true
    }

    /// Empties the focused slot on the shown side. Returns whether it held
    /// anything.
    fn clear_slot(&mut self, ctx: &mut Ctx) -> bool {
        let Some((action, slot)) = self.focus() else {
            return false;
        };
        if self.slot_names(ctx, action)[slot].is_none() {
            return false;
        }
        match self.side {
            Side::Keyboard => self.bindings.clear(action, slot),
            Side::Controller => self.pad.clear(action, slot),
        }
        self.message = None;
        ctx.audio.menu(MenuSound::Select);
        true
    }

    /// Ends the capture with nothing changed.
    fn abort_capture(&mut self, ctx: &mut Ctx) {
        self.end_capture();
        ctx.audio.menu(MenuSound::Cancel);
    }

    /// Ends the capture, clearing what belongs to it.
    fn end_capture(&mut self) {
        self.capturing = false;
        self.held = None;
        self.message = None;
    }

    /// The capture took something that was in slot `from`, if any: ends it
    /// and lights up the action that lost it.
    fn captured(&mut self, ctx: &mut Ctx, from: Option<(Action, usize)>) {
        self.end_capture();
        self.moved = from.map(|(lost, _)| (lost, MOVED_FLASH_SECS));
        ctx.audio.menu(MenuSound::Select);
    }

    /// A key pressed while capturing. Returns whether capturing ended.
    fn capture(&mut self, ctx: &mut Ctx, chord: Chord) -> bool {
        if is_clear_slot(chord) {
            return false;
        }
        if is_capture_abort(chord) {
            self.abort_capture(ctx);
            return true;
        }
        let Some((action, slot)) = self.focus() else {
            self.end_capture();
            return true;
        };
        if self.side == Side::Controller {
            // A slot waiting for a button takes no key.
            return false;
        }
        let Ok(from) = self.bindings.bind(action, slot, chord) else {
            self.message = Some(ctx.text(RESERVED_MESSAGE).to_owned());
            ctx.audio.menu(MenuSound::Denied);
            return false;
        };
        self.captured(ctx, from);
        true
    }

    /// This frame's controller buttons while capturing: the first button
    /// to go down is watched. Let go before [`HOLD_TO_CANCEL_SECS`], it
    /// goes in a slot that waits for a button; held that long, it backs out
    /// of the capture with nothing changed.
    fn capture_buttons(&mut self, ctx: &mut Ctx, input: &FrameInput) {
        let fresh = self.held.is_none();
        if fresh {
            self.held = input.pressed_buttons().first().map(|&b| (b, 0.0));
        }
        let Some((button, held_secs)) = self.held else {
            return;
        };
        if input.released_buttons().contains(&button) {
            self.held = None;
            self.tapped(ctx, button);
            return;
        }
        // The frame it went down in doesn't count; nor does a NaN frame.
        let secs = if fresh {
            0.0
        } else {
            held_secs + input.dt.max(0.0)
        };
        self.held = Some((button, secs));
        if secs >= HOLD_TO_CANCEL_SECS {
            self.abort_capture(ctx);
        }
    }

    /// `button` was pressed and let go while capturing: on the controller
    /// side it goes in the focused slot.
    fn tapped(&mut self, ctx: &mut Ctx, button: Button) {
        let (Side::Controller, Some((action, slot))) = (self.side, self.focus()) else {
            return;
        };
        // The focused action is rebindable and its slot exists.
        if let Ok(from) = self.pad.bind(action, slot, button) {
            self.captured(ctx, from);
        }
    }

    /// Cancel: saves and returns `true` if every required action has a key
    /// and a button; otherwise shows the side that lacks one and names the
    /// first action on the screen that does (the shown side first).
    fn leave(&mut self, ctx: &mut Ctx) -> bool {
        let other = match self.side {
            Side::Keyboard => Side::Controller,
            Side::Controller => Side::Keyboard,
        };
        let blocked = [self.side, other]
            .into_iter()
            .find_map(|side| Some((side, self.first_unmapped(side)?)));
        if let Some((side, action)) = blocked {
            self.side = side;
            self.message = Some(blocked_message(ctx, side, action));
            ctx.audio.menu(MenuSound::Denied);
            return false;
        }
        // If saving fails the keys and buttons still apply for this session.
        if self.bindings != ctx.layout_bindings(self.layout) {
            let _ = ctx.set_layout_bindings(self.layout, self.bindings.clone());
        }
        if self.pad != ctx.pad_bindings() {
            let _ = ctx.set_pad_bindings(self.pad.clone());
        }
        ctx.audio.menu(MenuSound::Cancel);
        true
    }

    /// The bottom help line, naming the keys the screen was opened with
    /// (or their buttons, on a controller): the same as `ctx`'s until the
    /// screen closes.
    pub fn help(&self, ctx: &Ctx) -> String {
        let km = HelpKeys::new(&self.opened_with, ctx.device);
        let on_pad = matches!(ctx.device, Device::Pad(_));
        if self.capturing {
            let what = ctx.text(match self.side {
                Side::Keyboard => CAPTURE_HELP,
                Side::Controller => CAPTURE_BUTTON_PROMPT,
            });
            let hold = on_pad || self.side == Side::Controller;
            let hold = hold.then(|| ctx.text(HOLD_TO_CANCEL));
            let back = (!on_pad).then(|| {
                let key = capture_abort_key_name();
                ctx.text_with("key_bindings.help_capture_back", &[("key", &key)])
            });
            let hints = [Some(what), hold, back.as_deref()];
            return hints
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join(SEPARATOR);
        }
        if self.asking_restore {
            return fill_text(ctx.text("key_bindings.help_restore"), km, &[]);
        }
        if self.choice.is_some() {
            return ctx.text_with("key_bindings.help_choice", &[]);
        }
        if self.is_on_switch() {
            return ctx.text_with("key_bindings.help_switch", &[]);
        }
        let on_slot = self.focus().is_some();
        if on_slot && on_pad {
            return ctx.text_with("key_bindings.help_slot_pad", &[]);
        }
        if on_slot {
            let clear = clear_slot_key_name();
            let text = ctx.text("key_bindings.help_slot");
            return fill_text(text, km, &[("clear", &clear)]);
        }
        fill_text(ctx.text("key_bindings.help_restore_row"), km, &[])
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): the switch, the slot columns, every action with
    /// what its slots hold, the restore row, the message, the question or
    /// the choices open, and the help line, all in the player's language
    /// and naming their keys.
    pub fn view(&self, ctx: &Ctx) -> KeyBindingsView {
        let columns = (0..SLOTS).map(|j| match self.side {
            Side::Keyboard => ctx.text_with("key_bindings.key_column", &[("n", &(j + 1))]),
            Side::Controller => ctx.text_with("key_bindings.button_column", &[("n", &(j + 1))]),
        });
        let mut groups: Vec<GroupView> = Vec::new();
        for (i, &(action, _)) in ROWS.iter().enumerate() {
            let required = action.is_required();
            if groups.last().is_none_or(|g| g.required != required) {
                groups.push(GroupView {
                    heading: self.group_heading(ctx, required),
                    required,
                    rows: Vec::new(),
                });
            }
            if let Some(group) = groups.last_mut() {
                group.rows.push(self.row_view(ctx, i));
            }
        }
        let km = HelpKeys::new(&self.opened_with, ctx.device);
        KeyBindingsView {
            title: ctx.text(TITLE).to_owned(),
            switch: SwitchView {
                keyboard: format!(
                    "{}{SEPARATOR}{}",
                    ctx.text(KEYBOARD_SIDE),
                    layout_picker::label(ctx, self.layout)
                ),
                controller: ctx.text(CONTROLLER_SIDE).to_owned(),
                shown: self.side,
                focused: self.is_on_switch(),
            },
            columns: columns.collect(),
            groups,
            restore: RestoreView {
                label: ctx.text(RESTORE_DEFAULTS).to_owned(),
                focused: self.row == RESTORE_ROW,
            },
            not_mapped: NOT_MAPPED.to_owned(),
            message: self.message.clone(),
            question: self.asking_restore.then(|| QuestionView {
                text: self.restore_question(ctx),
                answers: fill_text(ctx.text("key_bindings.restore_answers"), km, &[]),
            }),
            choices: self.choice.map(|focused| ChoicesView {
                lines: CHOICES.map(|key| ctx.text(key).to_owned()).to_vec(),
                focused,
                slot: self.slot,
            }),
            help: self.help(ctx),
        }
    }

    /// The heading over the required (or the optional) actions.
    fn group_heading(&self, ctx: &Ctx, required: bool) -> String {
        match (required, self.side) {
            (true, Side::Keyboard) => ctx.text(REQUIRED_HEADING).to_owned(),
            (true, Side::Controller) => ctx.text(REQUIRED_BUTTON_HEADING).to_owned(),
            (false, _) => ctx.text(OPTIONAL_HEADING).to_owned(),
        }
    }

    /// Row `i` of [`ROWS`], as the screen shows it.
    fn row_view(&self, ctx: &Ctx, i: usize) -> RowView {
        let (action, label) = ROWS[i];
        let label = ctx.text(label);
        let focused = self.row == i;
        let fixed = match self.side {
            Side::Keyboard => Keymap::fixed_chords_for(action)
                .iter()
                .map(ToString::to_string)
                .collect(),
            Side::Controller => Vec::new(),
        };
        let slots = self.slot_names(ctx, action).into_iter().enumerate();
        let slots = slots.map(|(j, name)| match name {
            _ if focused && self.slot == j && self.capturing => {
                SlotView::Capturing(ctx.text(self.side.prompt()).to_owned())
            }
            Some(name) => SlotView::Bound(name),
            None => SlotView::Empty,
        });
        let unmapped = self.is_unmapped(action);
        RowView {
            action,
            label: label.to_owned(),
            fixed,
            slots: slots.collect(),
            focus: focused.then_some(self.slot),
            unmapped,
            blocks_leaving: unmapped && action.is_required(),
            lost_key: self.moved.is_some_and(|(lost, _)| lost == action),
        }
    }
}

impl Screen for KeyBindingsScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        self.tick(input.dt);
        if self.capturing {
            // This frame's actions and repeats are ignored: the key or
            // button is for the slot.
            if !input.pressed_chords().iter().any(|&c| self.capture(ctx, c)) {
                self.capture_buttons(ctx, input);
            }
            if !self.capturing {
                self.await_release = CURSOR_ACTIONS.iter().any(|&a| input.is_held(a));
            }
            return Transition::None;
        }
        if self.asking_restore {
            for &action in &input.actions {
                if self.answer_restore(ctx, action) {
                    break;
                }
            }
            return Transition::None;
        }
        if self.choice.is_some() {
            for &action in &input.actions {
                if self.answer_choice(ctx, action) {
                    break;
                }
            }
            return Transition::None;
        }
        if input.pressed_chords().iter().any(|&c| is_clear_slot(c)) {
            self.clear_slot(ctx);
        }
        self.await_release &= CURSOR_ACTIONS.iter().any(|&a| input.is_held(a));
        for &action in &input.actions {
            match action {
                cursor if self.await_release && CURSOR_ACTIONS.contains(&cursor) => {}
                Action::CursorUp => self.move_row(ctx, false),
                Action::CursorDown => self.move_row(ctx, true),
                Action::CursorLeft => self.move_across(ctx, false),
                Action::CursorRight => self.move_across(ctx, true),
                Action::Confirm => {
                    self.confirm(ctx);
                    if self.capturing || self.asking_restore || self.choice.is_some() {
                        // The key that confirmed isn't the key to bind (or
                        // the answer), and nothing after it this frame
                        // counts.
                        break;
                    }
                }
                Action::Cancel if self.leave(ctx) => return Transition::Pop,
                _ => {}
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
