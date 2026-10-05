//! The Options screen (ticket 0805), opened from the title, the map menu
//! and Preparations: the player's [`Settings`] one per row, changed with
//! the cursor's left and right keys and saved at once. A volume goes from 0
//! to 100: left and right move its slider, Confirm opens a box to type the
//! exact number ([`NumberBox`]). `Layout` opens the layout picker
//! ([`LayoutPickerScreen::change`]) and `Key bindings` the
//! [`KeyBindingsScreen`]; with a campaign loaded, `Game mode` shows its
//! mode, and at Preparations switches Classic to Casual, never back and
//! never in the middle of a battle (`docs/design/death-and-difficulty.md`);
//! `Reset tips` shows every one-time tip again; `Restore defaults` puts
//! every setting back (custom keys are reset on the Key bindings screen,
//! and the layout stays). Whatever does something big asks first
//! (`docs/design/options.md`).
//!
//! **The look is a skin.** This module decides what the screen shows and
//! does, and says it as plain data: an [`OptionsView`]
//! ([`OptionsScreen::view`]). [`glyph::paint`] draws that view as glyphs.
//! Nothing here knows a cell, a colour or a position, so another look (a
//! bought UI pack) is another painter of the same view. Tests of what
//! happened read the view or the settings; only [`glyph`]'s tests read
//! cells.

pub mod glyph;
pub mod view;

pub use view::{OptionsView, QuestionView, RowView, ValueView};

use trpg_core::GameMode;

use super::{KeyBindingsScreen, LayoutPickerScreen, layout_picker};
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::{Action, TextKey, text_key, text_keys_help};
use crate::map_view::CursorStyle;
use crate::screen::{Ctx, FrameInput, ModeSwitch, Screen, Transition};
use crate::settings::{AnimSpeed, EnemyPhaseSpeed, MAX_VOLUME, Settings, TextSpeed};
use crate::tips::reset_tips;

/// Text key ([`Ctx::text`]) of the screen title.
pub const TITLE: &str = "options.title";
/// Text key of the message under the panel once the tips are reset.
pub const TIPS_RESET_MESSAGE: &str = "options.message.tips_reset";
/// Text key of the message once the defaults are back.
pub const RESTORED_MESSAGE: &str = "options.message.restored";
/// Text key of the message once the campaign is Casual.
pub const CASUAL_MESSAGE: &str = "options.message.casual";
/// Text key of the message if something couldn't be saved.
pub const NOT_SAVED_MESSAGE: &str = "options.message.not_saved";
/// Text key of the message when the mode can't be switched here.
pub const MODE_AT_PREP_MESSAGE: &str = "options.message.mode_at_prep";

/// How far the cursor's left and right keys move a volume's slider.
/// *Tunable.*
pub const VOLUME_STEP: u8 = 5;
/// The most digits the volume box takes.
const MAX_DIGITS: usize = 3;

/// One row of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    /// [`Settings::text_speed`].
    TextSpeed,
    /// [`Settings::anim_speed`].
    AnimSpeed,
    /// [`Settings::combat_animations`].
    CombatAnimations,
    /// [`Settings::enemy_phase_speed`].
    EnemyPhaseSpeed,
    /// [`Settings::auto_end_turn`].
    AutoEnd,
    /// [`Settings::fullscreen`].
    Fullscreen,
    /// [`Settings::cursor_style`].
    Cursor,
    /// [`Settings::music_volume`].
    MusicVolume,
    /// [`Settings::sound_volume`].
    SoundVolume,
    /// The right- or left-handed layout: opens the layout picker.
    Layout,
    /// Opens the Key bindings screen.
    KeyBindings,
    /// The campaign's mode (only with a campaign loaded).
    GameMode,
    /// Shows every one-time tip again.
    ResetTips,
    /// Puts every setting back to its default.
    RestoreDefaults,
}

impl Row {
    /// Every row, top to bottom.
    pub const ALL: [Row; 14] = [
        Row::TextSpeed,
        Row::AnimSpeed,
        Row::CombatAnimations,
        Row::EnemyPhaseSpeed,
        Row::AutoEnd,
        Row::Fullscreen,
        Row::Cursor,
        Row::MusicVolume,
        Row::SoundVolume,
        Row::Layout,
        Row::KeyBindings,
        Row::GameMode,
        Row::ResetTips,
        Row::RestoreDefaults,
    ];

    /// The text key ([`Ctx::text`]) of its label.
    pub const fn key(self) -> &'static str {
        match self {
            Row::TextSpeed => "options.row.text_speed",
            Row::AnimSpeed => "options.row.anim_speed",
            Row::CombatAnimations => "options.row.combat_animations",
            Row::EnemyPhaseSpeed => "options.row.enemy_phase_speed",
            Row::AutoEnd => "options.row.auto_end",
            Row::Fullscreen => "options.row.fullscreen",
            Row::Cursor => "options.row.cursor",
            Row::MusicVolume => "options.row.music_volume",
            Row::SoundVolume => "options.row.sound_volume",
            Row::Layout => "options.row.layout",
            Row::KeyBindings => "options.row.key_bindings",
            Row::GameMode => "options.row.game_mode",
            Row::ResetTips => "options.row.reset_tips",
            Row::RestoreDefaults => "options.row.restore_defaults",
        }
    }

    /// Whether the cursor's left and right keys change its value.
    pub const fn is_setting(self) -> bool {
        !matches!(
            self,
            Row::Layout | Row::KeyBindings | Row::GameMode | Row::ResetTips | Row::RestoreDefaults
        )
    }

    /// Whether a blank row comes before it (the first row of a group).
    const fn starts_group(self) -> bool {
        matches!(self, Row::Layout | Row::ResetTips)
    }
}

/// The text key of a cursor style's name (`docs/design/look-and-feel.md`).
pub const fn cursor_key(style: CursorStyle) -> &'static str {
    match style {
        CursorStyle::Corners => "options.cursor.corners",
        CursorStyle::LargeCorners => "options.cursor.large_corners",
        CursorStyle::TileGlow => "options.cursor.tile_glow",
    }
}

/// Every cursor style, in the order the row steps through them.
const CURSOR_STYLES: [CursorStyle; 3] = [
    CursorStyle::Corners,
    CursorStyle::LargeCorners,
    CursorStyle::TileGlow,
];

/// The text key of a game mode's name.
const fn mode_key(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Classic => "options.mode.classic",
        GameMode::Casual => "options.mode.casual",
    }
}

/// The text key of an on/off value.
const fn on_off_key(on: bool) -> &'static str {
    if on {
        "options.value.on"
    } else {
        "options.value.off"
    }
}

/// `all`'s value one step after (`forward`) or before `current`, stopping
/// at the ends (`wrap`: going round instead).
fn stepped<T: Copy + PartialEq>(all: &[T], current: T, forward: bool, wrap: bool) -> T {
    let n = all.len();
    let i = all.iter().position(|&v| v == current).unwrap_or(0);
    let to = match (forward, wrap) {
        (true, true) => (i + 1) % n,
        // Past the last value there is none: it stays (below).
        (true, false) => i + 1,
        (false, true) => (i + n - 1) % n,
        (false, false) => i.saturating_sub(1),
    };
    all.get(to).copied().unwrap_or(current)
}

/// A volume one slider step ([`VOLUME_STEP`]) up or down, within 0 to
/// [`MAX_VOLUME`].
fn stepped_volume(volume: u8, forward: bool) -> u8 {
    if forward {
        volume.saturating_add(VOLUME_STEP).min(MAX_VOLUME)
    } else {
        volume.saturating_sub(VOLUME_STEP)
    }
}

/// The box that takes a volume's exact number. From the keyboard the
/// digits are typed (the game's keys do nothing meanwhile; the text box's
/// fixed keys finish, delete and cancel). Opened with a controller button
/// it holds the volume, and the cursor steps it by one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberBox {
    /// What is typed so far (on a controller, the number as stepped).
    digits: String,
    /// Whether a controller opened it.
    pad: bool,
}

impl NumberBox {
    /// The number in the box, at most [`MAX_VOLUME`]; `None` while empty.
    pub fn value(&self) -> Option<u8> {
        let typed: u32 = self.digits.parse().ok()?;
        Some(u8::try_from(typed.min(u32::from(MAX_VOLUME))).unwrap_or(MAX_VOLUME))
    }

    /// The digits in the box.
    pub fn digits(&self) -> &str {
        &self.digits
    }
}

/// A yes/no question the screen asks before something that can't be taken
/// back with one key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Question {
    /// Classic → Casual.
    SwitchToCasual,
    /// Every one-time tip shown again.
    ResetTips,
    /// Every setting back to its default.
    RestoreDefaults,
}

impl Question {
    /// The text key of the question.
    pub const fn key(self) -> &'static str {
        match self {
            Question::SwitchToCasual => "options.question.casual",
            Question::ResetTips => "options.question.reset_tips",
            Question::RestoreDefaults => "options.question.restore",
        }
    }
}

/// The Options screen. Cancel closes it.
#[derive(Debug, Clone)]
pub struct OptionsScreen {
    /// The focused row.
    row: Row,
    /// The question open, if any.
    asking: Option<Question>,
    /// The volume box, while it is open on the focused volume row.
    typing: Option<NumberBox>,
    /// The text key of the message under the panel, if any.
    message: Option<&'static str>,
}

impl OptionsScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "options";

    /// The screen with its first row focused.
    pub fn new() -> Self {
        Self {
            row: Row::TextSpeed,
            asking: None,
            typing: None,
            message: None,
        }
    }

    /// The rows shown: every one, `Game mode` only with a campaign loaded.
    pub fn rows(ctx: &Ctx) -> Vec<Row> {
        let campaign = ctx.campaign_mode.is_some();
        Row::ALL
            .into_iter()
            .filter(|&r| r != Row::GameMode || campaign)
            .collect()
    }

    /// The focused row.
    pub fn focus(&self) -> Row {
        self.row
    }

    /// The question open, if any.
    pub fn asking(&self) -> Option<Question> {
        self.asking
    }

    /// The volume box, while it is open.
    pub fn typing(&self) -> Option<&NumberBox> {
        self.typing.as_ref()
    }

    /// The text key of the message under the panel, if any.
    pub fn message(&self) -> Option<&str> {
        self.message
    }

    /// Whether Confirm on `Game mode` would ask to switch to Casual: a
    /// Classic campaign, at Preparations.
    fn can_switch_mode(ctx: &Ctx) -> bool {
        ctx.campaign_mode == Some(GameMode::Classic) && ctx.mode_switch == ModeSwitch::Open
    }

    /// What `row` shows beside its name: the setting, the layout in use,
    /// the campaign's mode; nothing for the rows that only do something.
    /// (The volume box, while open, is the screen's:
    /// [`view`](Self::view).)
    pub fn value(ctx: &Ctx, row: Row) -> ValueView {
        let s = ctx.settings();
        let adjustable = row.is_setting();
        let words = |text: &str| ValueView::Text {
            text: text.to_owned(),
            adjustable,
        };
        let volume = |level| ValueView::Volume {
            level,
            max: MAX_VOLUME,
        };
        let key = match row {
            Row::TextSpeed => s.text_speed.key(),
            Row::AnimSpeed => s.anim_speed.key(),
            Row::CombatAnimations => on_off_key(s.combat_animations),
            Row::EnemyPhaseSpeed => s.enemy_phase_speed.key(),
            Row::AutoEnd => on_off_key(s.auto_end_turn),
            Row::Fullscreen => on_off_key(s.fullscreen),
            Row::Cursor => cursor_key(s.cursor_style),
            Row::MusicVolume => return volume(s.music_volume),
            Row::SoundVolume => return volume(s.sound_volume),
            Row::Layout => {
                return words(ctx.layout().map(layout_picker::label).unwrap_or_default());
            }
            Row::GameMode => match ctx.campaign_mode {
                Some(mode) => mode_key(mode),
                None => return ValueView::None,
            },
            Row::KeyBindings | Row::ResetTips | Row::RestoreDefaults => return ValueView::None,
        };
        words(ctx.text(key))
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): every row with its name and value, the row in
    /// focus, the volume box if it is open, the message, the question and
    /// the help line, all in the player's language and naming their keys.
    pub fn view(&self, ctx: &Ctx) -> OptionsView {
        let rows = Self::rows(ctx).into_iter().map(|row| {
            let focused = row == self.row;
            let value = match self.typing.as_ref().filter(|_| focused) {
                Some(typing) => ValueView::NumberBox {
                    digits: typing.digits.clone(),
                    typing: !typing.pad,
                },
                None => Self::value(ctx, row),
            };
            RowView {
                row,
                label: ctx.text(row.key()).to_owned(),
                value,
                focused,
                starts_group: row.starts_group(),
            }
        });
        OptionsView {
            title: ctx.text(TITLE).to_owned(),
            rows: rows.collect(),
            message: self.message.map(|key| ctx.text(key).to_owned()),
            question: self.asking.map(|question| QuestionView {
                text: ctx.text(question.key()).to_owned(),
                answers: ctx.text_with("options.question.answers", &[]),
            }),
            help: self.help(ctx),
        }
    }

    /// Moves the focus one row up or down, wrapping.
    fn move_row(&mut self, ctx: &mut Ctx, down: bool) {
        self.row = stepped(&Self::rows(ctx), self.row, down, true);
        self.message = None;
        ctx.audio.menu(MenuSound::Move);
    }

    /// Steps the focused setting one value on (`forward`) or back; `wrap`
    /// goes round at the ends.
    fn change(&mut self, ctx: &mut Ctx, forward: bool, wrap: bool) {
        let row = self.row;
        self.set(ctx, |s| step_setting(s, row, forward, wrap));
    }

    /// Lets `change` edit the settings; if it changed anything, saves them
    /// and plays the move sound (at the new volume, for the sound volume).
    fn set(&mut self, ctx: &mut Ctx, change: impl FnOnce(&mut Settings)) {
        let before = ctx.settings().clone();
        let saved = ctx.change_settings(change);
        if *ctx.settings() == before {
            return;
        }
        self.message = saved.is_err().then_some(NOT_SAVED_MESSAGE);
        ctx.audio.menu(MenuSound::Move);
    }

    /// The focused volume row's setting.
    fn volume(&self, ctx: &Ctx) -> u8 {
        match self.row {
            Row::MusicVolume => ctx.settings().music_volume,
            _ => ctx.settings().sound_volume,
        }
    }

    /// Opens the volume box on the focused volume row: empty, to type in;
    /// or, opened with a controller button (`pad`), holding the volume.
    fn open_box(&mut self, ctx: &mut Ctx, pad: bool) {
        let digits = if pad {
            self.volume(ctx).to_string()
        } else {
            String::new()
        };
        self.typing = Some(NumberBox { digits, pad });
        ctx.audio.menu(MenuSound::Select);
    }

    /// Closes the volume box. `keep`: its number becomes the volume (an
    /// empty box keeps the volume as it was).
    fn close_box(&mut self, ctx: &mut Ctx, keep: bool) {
        let value = self.typing.take().and_then(|b| b.value()).filter(|_| keep);
        let Some(value) = value else {
            ctx.audio.menu(MenuSound::Cancel);
            return;
        };
        let row = self.row;
        ctx.audio.menu(MenuSound::Select);
        self.set(ctx, |s| match row {
            Row::MusicVolume => s.music_volume = value,
            _ => s.sound_volume = value,
        });
    }

    /// One frame of the volume box.
    fn type_number(&mut self, ctx: &mut Ctx, input: &FrameInput) {
        if self.typing.as_ref().is_some_and(|b| b.pad) {
            for &action in &input.actions {
                let up = match action {
                    Action::CursorUp | Action::CursorRight => true,
                    Action::CursorDown | Action::CursorLeft => false,
                    Action::Confirm => return self.close_box(ctx, true),
                    Action::Cancel => return self.close_box(ctx, false),
                    _ => continue,
                };
                let Some(b) = &mut self.typing else { return };
                let now = b.value().unwrap_or(0);
                let to = if up {
                    now.saturating_add(1).min(MAX_VOLUME)
                } else {
                    now.saturating_sub(1)
                };
                if to != now {
                    b.digits = to.to_string();
                    ctx.audio.menu(MenuSound::Move);
                }
            }
            return;
        }
        for &c in input.text() {
            let Some(b) = &mut self.typing else { return };
            if c.is_ascii_digit() && b.digits.len() < MAX_DIGITS {
                b.digits.push(c);
            } else {
                ctx.audio.menu(MenuSound::Denied);
            }
        }
        for &chord in input.pressed_chords() {
            match text_key(chord) {
                Some(TextKey::Done) => return self.close_box(ctx, true),
                Some(TextKey::Cancel) => return self.close_box(ctx, false),
                Some(TextKey::Delete) => {
                    let Some(b) = &mut self.typing else { return };
                    if b.digits.pop().is_some() {
                        ctx.audio.menu(MenuSound::Cancel);
                    }
                }
                None => {}
            }
        }
    }

    /// Confirm on the focused row (`pad`: with a controller button).
    fn confirm(&mut self, ctx: &mut Ctx, pad: bool) -> Transition {
        self.message = None;
        match self.row {
            Row::MusicVolume | Row::SoundVolume => self.open_box(ctx, pad),
            Row::Layout => {
                ctx.audio.menu(MenuSound::Select);
                return Transition::Push(Box::new(LayoutPickerScreen::change(ctx)));
            }
            Row::KeyBindings => {
                ctx.audio.menu(MenuSound::Select);
                return Transition::Push(Box::new(KeyBindingsScreen::new(ctx)));
            }
            Row::GameMode if Self::can_switch_mode(ctx) => {
                self.asking = Some(Question::SwitchToCasual);
                ctx.audio.menu(MenuSound::Select);
            }
            // Casual never goes back to Classic, and Classic only switches
            // at Preparations: the row says so.
            Row::GameMode => {
                if ctx.campaign_mode == Some(GameMode::Classic) {
                    self.message = Some(MODE_AT_PREP_MESSAGE);
                }
                ctx.audio.menu(MenuSound::Denied);
            }
            Row::ResetTips => {
                self.asking = Some(Question::ResetTips);
                ctx.audio.menu(MenuSound::Select);
            }
            Row::RestoreDefaults => {
                self.asking = Some(Question::RestoreDefaults);
                ctx.audio.menu(MenuSound::Select);
            }
            _ => self.change(ctx, true, true),
        }
        Transition::None
    }

    /// An action while `question` is open: Confirm does it, Cancel backs
    /// out; either closes the question (and returns `true`). Nothing else
    /// does anything.
    fn answer(&mut self, ctx: &mut Ctx, question: Question, action: Action) -> bool {
        match action {
            Action::Confirm => {
                self.message = Some(match question {
                    Question::SwitchToCasual => {
                        ctx.switch_to_casual();
                        CASUAL_MESSAGE
                    }
                    Question::ResetTips => {
                        if reset_tips(&mut *ctx.storage).is_ok() {
                            TIPS_RESET_MESSAGE
                        } else {
                            NOT_SAVED_MESSAGE
                        }
                    }
                    Question::RestoreDefaults => {
                        let saved = ctx.change_settings(|s| *s = s.restored());
                        if saved.is_ok() {
                            RESTORED_MESSAGE
                        } else {
                            NOT_SAVED_MESSAGE
                        }
                    }
                });
                ctx.audio.menu(MenuSound::Select);
            }
            Action::Cancel => ctx.audio.menu(MenuSound::Cancel),
            _ => return false,
        }
        self.asking = None;
        true
    }

    /// The text key of the help line for the focused row: what Confirm
    /// does there, if anything.
    fn help_key(&self, ctx: &Ctx) -> &'static str {
        if self.asking.is_some() {
            return "options.help.question";
        }
        match self.row {
            Row::Layout | Row::KeyBindings => "options.help.open",
            Row::MusicVolume | Row::SoundVolume => "options.help.volume",
            Row::GameMode if Self::can_switch_mode(ctx) => "options.help.switch",
            Row::GameMode => "options.help.none",
            Row::ResetTips => "options.help.reset",
            Row::RestoreDefaults => "options.help.restore",
            _ => "options.help.change",
        }
    }

    /// The bottom help line, naming the keys of the active keymap (or
    /// their buttons, on a controller).
    pub fn help(&self, ctx: &Ctx) -> String {
        match &self.typing {
            // Typing: the text box's own keys.
            Some(b) if !b.pad => text_keys_help(),
            Some(_) => ctx.text_with("options.help.number_pad", &[]),
            None => ctx.text_with(self.help_key(ctx), &[]),
        }
    }
}

/// Steps the setting `row` shows; other rows have none.
fn step_setting(s: &mut Settings, row: Row, forward: bool, wrap: bool) {
    match row {
        Row::TextSpeed => s.text_speed = stepped(&TextSpeed::ALL, s.text_speed, forward, wrap),
        Row::AnimSpeed => s.anim_speed = stepped(&AnimSpeed::ALL, s.anim_speed, forward, wrap),
        Row::EnemyPhaseSpeed => {
            s.enemy_phase_speed =
                stepped(&EnemyPhaseSpeed::ALL, s.enemy_phase_speed, forward, wrap);
        }
        // On/off rows read `Off  On` left to right.
        Row::CombatAnimations => {
            s.combat_animations = stepped(&[false, true], s.combat_animations, forward, wrap);
        }
        Row::AutoEnd => s.auto_end_turn = stepped(&[false, true], s.auto_end_turn, forward, wrap),
        Row::Fullscreen => s.fullscreen = stepped(&[false, true], s.fullscreen, forward, wrap),
        Row::Cursor => s.cursor_style = stepped(&CURSOR_STYLES, s.cursor_style, forward, wrap),
        Row::MusicVolume => s.music_volume = stepped_volume(s.music_volume, forward),
        Row::SoundVolume => s.sound_volume = stepped_volume(s.sound_volume, forward),
        Row::Layout | Row::KeyBindings | Row::GameMode | Row::ResetTips | Row::RestoreDefaults => {}
    }
}

impl Default for OptionsScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for OptionsScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if self.typing.is_some() {
            // The game's keys do nothing while the box is open.
            self.type_number(ctx, input);
            return Transition::None;
        }
        for &action in &input.actions {
            if let Some(question) = self.asking {
                // The answer ends this frame's keys.
                if self.answer(ctx, question, action) {
                    break;
                }
                continue;
            }
            match action {
                Action::CursorUp => self.move_row(ctx, false),
                Action::CursorDown => self.move_row(ctx, true),
                Action::CursorLeft => self.change(ctx, false, false),
                Action::CursorRight => self.change(ctx, true, false),
                Action::Confirm => {
                    let transition = self.confirm(ctx, input.pad_pressed());
                    if !matches!(transition, Transition::None) {
                        return transition;
                    }
                    if self.asking.is_some() || self.typing.is_some() {
                        // The key that asked isn't the answer, and the
                        // key that opened the box types nothing.
                        break;
                    }
                }
                Action::Cancel => {
                    ctx.audio.menu(MenuSound::Cancel);
                    return Transition::Pop;
                }
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
