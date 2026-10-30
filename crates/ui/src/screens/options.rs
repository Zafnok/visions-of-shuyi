//! The Options screen (ticket 0805), opened from the title, the map menu
//! and Preparations: Shuyi Engine's options screen
//! (`shuyi_ui::screens::options`, ticket 1112) with this game's rows
//! ([`rows`]), re-exported here under its old names.
//!
//! The player's [`Settings`] are one per row, changed with the cursor's
//! left and right keys. A change shows at once but is saved only when the
//! player keeps it: leaving with anything changed asks whether to keep
//! it, throw it away or stay (ticket 0839, `docs/design/options.md`). A
//! volume goes from 0 to 100: left and right move its slider, Confirm
//! opens a box to type the exact number ([`NumberBox`]). `Layout` opens
//! the layout picker ([`layout_picker::change`]) and `Key bindings` the
//! Key bindings screen; with a campaign loaded, `Game mode` shows its
//! mode, and at Preparations switches Classic to Casual, never back and
//! never in the middle of a battle (`docs/design/death-and-difficulty.md`),
//! and `Tutorial` its tutorial level, a value row like the settings
//! (ticket 0838, `docs/design/tutorial.md`); `Reset tips` shows every
//! one-time tip again; `Restore defaults` puts every setting back (custom
//! keys are reset on the Key bindings screen, and the layout stays).
//! Whatever does something big asks first (`docs/design/options.md`).
//!
//! On the Playdate (ticket 1045) the rows that mean nothing there are
//! left out ([`PC_ONLY`], and `Layout` with no keyboard).
//!
//! **The look is a skin** (ADR-0054): the engine's [`glyph::paint`] draws
//! the screen's [`OptionsView`], or on the Playdate [`playdate::paint`]
//! ([`SKINS`], ADR-0083).

pub mod playdate;

use crate::prelude::*;
pub use shuyi_ui::screens::options::*;

use shuyi_ui::screens::options;
use trpg_content::Theme;
use trpg_core::{GameMode, TutorialLevel};

use super::{key_bindings, layout_picker};
use crate::glyph_buffer::GlyphBuffer;
use crate::map_view::{CursorStyle, has_pictures};
use crate::screen::{Ctx, Keyboard, ModeSwitch, Shuyi, SkinSet};
use crate::screens::Skins;
use crate::settings::{AnimSpeed, EnemyPhaseSpeed, MapLook, MapZoom, Settings, TownsOpen};
use crate::tips::reset_tips;

/// The Options screen for this game
/// ([`shuyi_ui::screens::options::OptionsScreen`]).
pub type OptionsScreen = options::OptionsScreen<Shuyi>;

/// One row as the engine's screen takes it ([`options::OptionRow`]).
pub type OptionRow = options::OptionRow<Shuyi>;

/// Text key of the message under the panel once the tips are reset.
pub const TIPS_RESET_MESSAGE: &str = "options.message.tips_reset";
/// Text key of the message once the campaign is Casual.
pub const CASUAL_MESSAGE: &str = "options.message.casual";
/// Text key of the message when the mode can't be switched here.
pub const MODE_AT_PREP_MESSAGE: &str = "options.message.mode_at_prep";

/// The Options screen with this game's rows, its first row focused,
/// painted by the skin of the context's skin set.
pub fn screen() -> OptionsScreen {
    OptionsScreen::new(rows).painted_by(paint)
}

/// The screen's skins: the engine's glyph skin on the PC, and the
/// Playdate's.
pub const SKINS: Skins<OptionsView> = Skins {
    glyph: glyph::paint::<Shuyi>,
    playdate: playdate::paint,
};

/// Paints `view` with the skin of `ctx`'s skin set ([`SKINS`]).
fn paint(ctx: &Ctx, view: &OptionsView, buf: &mut GlyphBuffer) {
    SKINS.paint(ctx, view, buf);
}

/// The rows only the PC's screen shows (ticket 1045): what each does means
/// nothing on the Playdate's skins (`SkinSet::Playdate`). `Fullscreen`: its
/// screen is the device's, with no window. `Theme`: one bit has the
/// Playdate palette's inks, not a colour theme. `Map look` and `Map
/// zoom`: its map has one skin, which doesn't zoom, whatever the look.
/// `Intro`: it has no title cinematic yet (ticket 1044 adds it, and should
/// take the row out of this list). `Open towns on arrival`: it has no world
/// map yet (1007; the row comes with it).
pub const PC_ONLY: [Row; 6] = [
    Row::Fullscreen,
    Row::Theme,
    Row::MapLook,
    Row::MapZoom,
    Row::Intro,
    Row::Towns,
];

/// One row of the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    /// [`Settings::text_speed`].
    TextSpeed,
    /// [`ShuyiSettings::anim_speed`](crate::ShuyiSettings::anim_speed).
    AnimSpeed,
    /// [`ShuyiSettings::combat_animations`](crate::ShuyiSettings::combat_animations).
    CombatAnimations,
    /// [`ShuyiSettings::enemy_phase_speed`](crate::ShuyiSettings::enemy_phase_speed).
    EnemyPhaseSpeed,
    /// [`ShuyiSettings::auto_end_turn`](crate::ShuyiSettings::auto_end_turn).
    AutoEnd,
    /// [`Settings::fullscreen`].
    Fullscreen,
    /// [`ShuyiSettings::cursor_style`](crate::ShuyiSettings::cursor_style).
    Cursor,
    /// [`ShuyiSettings::map_look`](crate::ShuyiSettings::map_look) (only when
    /// the game has pictures).
    MapLook,
    /// [`ShuyiSettings::map_zoom`](crate::ShuyiSettings::map_zoom) (only when
    /// the map can be zoomed).
    MapZoom,
    /// [`ShuyiSettings::intro`](crate::ShuyiSettings::intro).
    Intro,
    /// [`ShuyiSettings::towns`](crate::ShuyiSettings::towns): on arrival or not.
    Towns,
    /// [`Settings::theme`].
    Theme,
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
    /// The campaign's tutorial level (only with a campaign loaded).
    Tutorial,
    /// Shows every one-time tip again.
    ResetTips,
    /// Puts every setting back to its default.
    RestoreDefaults,
}

impl Row {
    /// Every row, top to bottom.
    pub const ALL: [Row; 20] = [
        Row::TextSpeed,
        Row::AnimSpeed,
        Row::CombatAnimations,
        Row::EnemyPhaseSpeed,
        Row::AutoEnd,
        Row::Fullscreen,
        Row::Cursor,
        Row::MapLook,
        Row::MapZoom,
        Row::Intro,
        Row::Towns,
        Row::Theme,
        Row::MusicVolume,
        Row::SoundVolume,
        Row::Layout,
        Row::KeyBindings,
        Row::GameMode,
        Row::Tutorial,
        Row::ResetTips,
        Row::RestoreDefaults,
    ];

    /// The text key ([`Ctx::text`](shuyi_ui::Ctx::text)) of its label,
    /// which also names it on the engine's screen.
    pub const fn key(self) -> &'static str {
        match self {
            Row::TextSpeed => "options.row.text_speed",
            Row::AnimSpeed => "options.row.anim_speed",
            Row::CombatAnimations => "options.row.combat_animations",
            Row::EnemyPhaseSpeed => "options.row.enemy_phase_speed",
            Row::AutoEnd => "options.row.auto_end",
            Row::Fullscreen => "options.row.fullscreen",
            Row::Cursor => "options.row.cursor",
            Row::MapLook => "options.row.map_look",
            Row::MapZoom => "options.row.map_zoom",
            Row::Intro => "options.row.intro",
            Row::Towns => "options.row.towns",
            Row::Theme => "options.row.theme",
            Row::MusicVolume => "options.row.music_volume",
            Row::SoundVolume => "options.row.sound_volume",
            Row::Layout => "options.row.layout",
            Row::KeyBindings => "options.row.key_bindings",
            Row::GameMode => "options.row.game_mode",
            Row::Tutorial => "options.row.tutorial",
            Row::ResetTips => "options.row.reset_tips",
            Row::RestoreDefaults => "options.row.restore_defaults",
        }
    }

    /// The row whose key is `key`.
    pub fn from_key(key: &str) -> Option<Row> {
        Row::ALL.into_iter().find(|r| r.key() == key)
    }

    /// Whether the cursor's left and right keys change its value.
    pub const fn is_setting(self) -> bool {
        !matches!(
            self,
            Row::Layout | Row::KeyBindings | Row::GameMode | Row::ResetTips | Row::RestoreDefaults
        )
    }

    /// The row as the engine's screen takes it: its value, and the first
    /// row of each group (`Layout`, `Reset tips`) after a blank one.
    pub fn option(self) -> OptionRow {
        let row = match self {
            Row::TextSpeed => OptionRow::text_speed(),
            Row::Fullscreen => OptionRow::fullscreen(),
            Row::MusicVolume => OptionRow::music_volume(),
            Row::SoundVolume => OptionRow::sound_volume(),
            Row::RestoreDefaults => OptionRow::restore_defaults(),
            Row::AnimSpeed => choice::<AnimSpeed>(self),
            Row::EnemyPhaseSpeed => choice::<EnemyPhaseSpeed>(self),
            Row::MapLook => choice::<MapLook>(self),
            Row::MapZoom => choice::<MapZoom>(self),
            Row::Theme => choice::<Theme>(self),
            Row::CombatAnimations => OptionRow::new(
                self.key(),
                RowValue::toggle(
                    |ctx| ctx.settings().game.combat_animations,
                    |ctx, on| ctx.preview_settings(|s| s.game.combat_animations = on),
                ),
            ),
            Row::AutoEnd => OptionRow::new(
                self.key(),
                RowValue::toggle(
                    |ctx| ctx.settings().game.auto_end_turn,
                    |ctx, on| ctx.preview_settings(|s| s.game.auto_end_turn = on),
                ),
            ),
            Row::Intro => OptionRow::new(
                self.key(),
                RowValue::toggle(
                    |ctx| ctx.settings().game.intro,
                    |ctx, on| ctx.preview_settings(|s| s.game.intro = on),
                ),
            ),
            Row::Towns => OptionRow::new(
                self.key(),
                RowValue::toggle(
                    |ctx| ctx.settings().game.towns == TownsOpen::OnArrival,
                    |ctx, on| {
                        let towns = if on {
                            TownsOpen::OnArrival
                        } else {
                            TownsOpen::OnConfirm
                        };
                        ctx.preview_settings(|s| s.game.towns = towns)
                    },
                ),
            ),
            Row::Cursor => cursor_row(),
            Row::Layout => layout_row(),
            Row::KeyBindings => key_bindings_row(),
            Row::GameMode => game_mode_row(),
            Row::Tutorial => tutorial_row(),
            Row::ResetTips => reset_tips_row(),
        };
        if matches!(self, Row::Layout | Row::ResetTips) {
            row.starting_group()
        } else {
            row
        }
    }
}

/// The cursor style: one of those offered under the map skin in use.
fn cursor_row() -> OptionRow {
    OptionRow::new(
        Row::Cursor.key(),
        RowValue::choice(
            |ctx| cursor_styles(ctx).iter().map(|&s| cursor_key(s)).collect(),
            |ctx| {
                let shown = shown_cursor_style(ctx);
                cursor_styles(ctx)
                    .iter()
                    .position(|&s| s == shown)
                    .unwrap_or(0)
            },
            |ctx, i| {
                let Some(&style) = cursor_styles(ctx).get(i) else {
                    return false;
                };
                ctx.preview_settings(|s| s.game.cursor_style = style)
            },
        ),
    )
}

/// The layout in use: Confirm opens the layout picker.
fn layout_row() -> OptionRow {
    OptionRow::new(
        Row::Layout.key(),
        RowValue::action(
            |ctx| {
                let layout = ctx.layout().map(|l| layout_picker::label(ctx, l));
                Some(layout.unwrap_or_default().to_owned())
            },
            |ctx| RowAction::Open(Box::new(layout_picker::change(ctx))),
            |_| HELP_OPEN,
        ),
    )
}

/// Confirm opens the Key bindings screen.
fn key_bindings_row() -> OptionRow {
    OptionRow::new(
        Row::KeyBindings.key(),
        RowValue::action(
            |_| None,
            |ctx| RowAction::Open(Box::new(key_bindings::screen(ctx))),
            |_| HELP_OPEN,
        ),
    )
}

/// The campaign's mode: at Preparations Confirm asks to switch Classic to Casual.
fn game_mode_row() -> OptionRow {
    OptionRow::new(
        Row::GameMode.key(),
        RowValue::action(
            |ctx| {
                let mode = ctx.game.campaign_mode?;
                Some(ctx.text(mode_key(mode)).to_owned())
            },
            switch_mode,
            |ctx| {
                if can_switch_mode(ctx) {
                    "options.help.switch"
                } else {
                    "options.help.none"
                }
            },
        ),
    )
}

/// The campaign's tutorial level, a value row like the settings.
fn tutorial_row() -> OptionRow {
    OptionRow::new(
        Row::Tutorial.key(),
        RowValue::choice(
            // None without a campaign: the row shows nothing.
            |ctx| match ctx.game.campaign_tutorial {
                Some(_) => TUTORIAL_LEVELS.map(tutorial_key).to_vec(),
                None => Vec::new(),
            },
            |ctx| {
                let level = ctx.game.campaign_tutorial;
                TUTORIAL_LEVELS
                    .iter()
                    .position(|&l| Some(l) == level)
                    .unwrap_or(0)
            },
            |ctx, i| {
                let to = TUTORIAL_LEVELS.get(i).copied();
                let changed = to.is_some() && ctx.game.campaign_tutorial != to;
                if changed {
                    ctx.game.campaign_tutorial = to;
                }
                changed
            },
        ),
    )
    // The campaign's, not a setting: kept or put back with them.
    .unsaved()
}

/// Asks, then shows every one-time tip again.
fn reset_tips_row() -> OptionRow {
    OptionRow::new(
        Row::ResetTips.key(),
        RowValue::action(
            |_| None,
            |_| {
                RowAction::Ask(options::Question {
                    key: Question::ResetTips.key(),
                    yes: |ctx| {
                        if reset_tips(&mut ctx.storage).is_ok() {
                            TIPS_RESET_MESSAGE
                        } else {
                            NOT_SAVED_MESSAGE
                        }
                    },
                })
            },
            |_| "options.help.reset",
        ),
    )
}

impl From<Row> for &'static str {
    fn from(row: Row) -> Self {
        row.key()
    }
}

/// The rows shown, as the engine's screen takes them: every one, `Game
/// mode` and `Tutorial` only with a campaign loaded, `Map look` only when
/// the game has pictures and `Map zoom` only when it has them or the map
/// skin in use can zoom; `Layout` only with a keyboard, and none of
/// [`PC_ONLY`] on the Playdate.
pub fn rows(ctx: &Ctx) -> Vec<OptionRow> {
    shown_rows(ctx).into_iter().map(Row::option).collect()
}

/// The rows shown ([`rows`]), top to bottom.
pub fn shown_rows(ctx: &Ctx) -> Vec<Row> {
    let campaign = ctx.game.campaign_mode.is_some();
    let tutorial = ctx.game.campaign_tutorial.is_some();
    let pictures = has_pictures(&ctx.game.content);
    let zooms = pictures || ctx.game.map_skin.zoom().is_some();
    let keyboard = ctx.platform.keyboard == Keyboard::Present;
    let pc = ctx.game.skin_set == SkinSet::Glyph;
    Row::ALL
        .into_iter()
        .filter(|&r| r != Row::GameMode || campaign)
        .filter(|&r| r != Row::Tutorial || tutorial)
        .filter(|&r| r != Row::MapLook || pictures)
        .filter(|&r| r != Row::MapZoom || zooms)
        .filter(|&r| r != Row::Layout || keyboard)
        .filter(|r| pc || !PC_ONLY.contains(r))
        .collect()
}

/// What `row` shows beside its name ([`options::OptionsScreen::value`]).
pub fn value(ctx: &Ctx, row: Row) -> ValueView {
    OptionsScreen::value(ctx, &row.option())
}

/// A setting of the game's that is one of a few named values, stepped in
/// the order of [`ALL`](Self::ALL).
trait Named: Copy + PartialEq + 'static {
    /// Every value, in the order the row steps through them.
    const ALL: &'static [Self];
    /// The text key of its name.
    fn key(self) -> &'static str;
    /// Its value in `s`.
    fn get(s: &Settings) -> Self;
    /// Puts `self` in `s`.
    fn put(self, s: &mut Settings);
}

impl Named for AnimSpeed {
    const ALL: &'static [Self] = &AnimSpeed::ALL;
    fn key(self) -> &'static str {
        AnimSpeed::key(self)
    }
    fn get(s: &Settings) -> Self {
        s.game.anim_speed
    }
    fn put(self, s: &mut Settings) {
        s.game.anim_speed = self;
    }
}

impl Named for EnemyPhaseSpeed {
    const ALL: &'static [Self] = &EnemyPhaseSpeed::ALL;
    fn key(self) -> &'static str {
        EnemyPhaseSpeed::key(self)
    }
    fn get(s: &Settings) -> Self {
        s.game.enemy_phase_speed
    }
    fn put(self, s: &mut Settings) {
        s.game.enemy_phase_speed = self;
    }
}

impl Named for MapLook {
    const ALL: &'static [Self] = &MapLook::ALL;
    fn key(self) -> &'static str {
        MapLook::key(self)
    }
    fn get(s: &Settings) -> Self {
        s.game.map_look
    }
    fn put(self, s: &mut Settings) {
        s.game.map_look = self;
    }
}

impl Named for MapZoom {
    const ALL: &'static [Self] = &MapZoom::ALL;
    fn key(self) -> &'static str {
        MapZoom::key(self)
    }
    fn get(s: &Settings) -> Self {
        s.game.map_zoom
    }
    fn put(self, s: &mut Settings) {
        s.game.map_zoom = self;
    }
}

impl Named for Theme {
    const ALL: &'static [Self] = &Theme::ALL;
    fn key(self) -> &'static str {
        Theme::key(self)
    }
    fn get(s: &Settings) -> Self {
        s.theme
    }
    fn put(self, s: &mut Settings) {
        s.theme = self;
    }
}

/// The row of `row`'s setting `T`, one of its named values.
fn choice<T: Named>(row: Row) -> OptionRow {
    OptionRow::new(
        row.key(),
        RowValue::choice(
            |_| T::ALL.iter().map(|&v| v.key()).collect(),
            |ctx| {
                let now = T::get(ctx.settings());
                T::ALL.iter().position(|&v| v == now).unwrap_or(0)
            },
            |ctx, i| {
                let Some(&to) = T::ALL.get(i) else {
                    return false;
                };
                ctx.preview_settings(|s| to.put(s))
            },
        ),
    )
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

/// The cursor styles the row offers: every one, but no `Tile glow` while
/// pictures paint the whole map, where a cursor never glows (Nick,
/// 2026-10-13, ticket 0444).
fn cursor_styles(ctx: &Ctx) -> &'static [CursorStyle] {
    match ctx.game.map_skin.zoom() {
        Some(_) => &[CursorStyle::Corners, CursorStyle::LargeCorners],
        None => &CURSOR_STYLES,
    }
}

/// The cursor style the row shows: the setting, or `Corners` (what the
/// map draws) while the setting isn't offered.
fn shown_cursor_style(ctx: &Ctx) -> CursorStyle {
    let style = ctx.settings().game.cursor_style;
    if cursor_styles(ctx).contains(&style) {
        style
    } else {
        CursorStyle::Corners
    }
}

/// The text key of a game mode's name.
const fn mode_key(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Classic => "options.mode.classic",
        GameMode::Casual => "options.mode.casual",
    }
}

/// Whether Confirm on `Game mode` would ask to switch to Casual: a
/// Classic campaign, at Preparations.
fn can_switch_mode(ctx: &Ctx) -> bool {
    ctx.game.campaign_mode == Some(GameMode::Classic) && ctx.game.mode_switch == ModeSwitch::Open
}

/// Confirm on `Game mode`: at Preparations, asks to switch Classic to
/// Casual. Casual never goes back to Classic, and Classic only switches at
/// Preparations: the row says so.
fn switch_mode(ctx: &mut Ctx) -> RowAction<Shuyi> {
    if can_switch_mode(ctx) {
        return RowAction::Ask(options::Question {
            key: Question::SwitchToCasual.key(),
            yes: |ctx| {
                ctx.game.switch_to_casual();
                CASUAL_MESSAGE
            },
        });
    }
    let classic = ctx.game.campaign_mode == Some(GameMode::Classic);
    RowAction::Refuse(classic.then_some(MODE_AT_PREP_MESSAGE))
}

/// Every tutorial level, in the order the row steps through them.
const TUTORIAL_LEVELS: [TutorialLevel; 3] = [
    TutorialLevel::Full,
    TutorialLevel::Short,
    TutorialLevel::None,
];

/// The text key of a tutorial level's name.
pub const fn tutorial_key(level: TutorialLevel) -> &'static str {
    match level {
        TutorialLevel::Full => "options.tutorial.full",
        TutorialLevel::Short => "options.tutorial.short",
        TutorialLevel::None => "options.tutorial.none",
    }
}

/// The questions this game's rows ask before something that can't be
/// taken back with one key, by the text key the screen names them by
/// ([`OptionsScreen::asking`]).
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

#[cfg(test)]
mod tests;
