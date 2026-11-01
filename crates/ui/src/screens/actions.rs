//! The actions each screen reacts to (`Screen::actions`, ticket 0140), in
//! one table so that `check --target` can read them with no screen open:
//! a screen's own state decides which of them it takes at a given moment.
//! In tests the Harness hands a screen only what it says here.

use crate::prelude::*;
use shuyi_ui::input::fit::ScreenActions;
use shuyi_ui::input::{NAVIGATION, UiAction, ui_actions};

use super::{
    BattleScreen, CampScreen, ClassChangeScreen, CreditsScreen, DialogueScreen, FamiliarityScreen,
    GameOverScreen, KeyBindingsScreen, LayoutPickerScreen, LeadSelectScreen, ModeSelectScreen,
    OptionsScreen, PreparationsScreen, ProfileScreen, ResultsScreen, SavePromptScreen,
    SlotPickerScreen, ToBeContinuedScreen, TownScreen, WorldMapScreen,
    battle::{guides::GuidesScreen, rewind::RewindScreen, shop::ShopScreen},
};
use crate::debug::{
    CinematicScreen, DebugMenuScreen, GlyphSamplerScreen, PortraitViewerScreen, SceneCameraScreen,
    ScenePreviewScreen, SpriteTestScreen,
};
use crate::flow::FlowScreen;
use crate::input::Action::{
    self, Cancel, Confirm, ConfirmEndTurn, CursorDown, CursorLeft, CursorRight, CursorUp,
    DangerZone, DialogueLog, EndTurn, Info, Menu, NextUnit, Options, PrevUnit, Rewind, RewindBack,
    RewindForward, Select, ToggleAutoEnd, WorldArmy, WorldFind, ZoomMap,
};
use crate::widgets;

/// A list moved up and down.
const LIST: &[Action] = &[CursorUp, CursorDown, Confirm, Cancel];
/// All four directions, Confirm and Cancel.
const GRID: &[Action] = &[
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    Confirm,
    Cancel,
];
/// A list that can't be left.
const NO_CANCEL: &[Action] = &[CursorUp, CursorDown, Confirm];

/// What the cinematic screen reacts to.
pub const CINEMATIC: &[Action] = &[Cancel];
/// What the portrait viewer screen reacts to.
pub const PORTRAIT_VIEWER: &[Action] = &[CursorUp, CursorDown, CursorLeft, CursorRight, Cancel];
/// What the scene preview screen reacts to.
pub const SCENE_PREVIEW: &[Action] = &[Confirm, Cancel];
/// What the sprite test screen reacts to.
pub const SPRITE_TEST: &[Action] = &[Cancel];
/// It shows itself as its first screen and reads nothing.
pub const FLOW: &[Action] = &[];
/// What the guides screen reacts to.
pub const GUIDES: &[Action] = &[CursorUp, CursorDown, Cancel];
/// What the battle screen reacts to.
pub const BATTLE: &[Action] = &[
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    Confirm,
    Cancel,
    Info,
    DangerZone,
    NextUnit,
    PrevUnit,
    EndTurn,
    Menu,
    ToggleAutoEnd,
    Rewind,
    RewindBack,
    Select,
    ConfirmEndTurn,
    ZoomMap,
    Options,
];
/// What the rewind screen reacts to.
pub const REWIND: &[Action] = &[
    CursorUp,
    CursorDown,
    Confirm,
    Cancel,
    Rewind,
    RewindBack,
    RewindForward,
];
/// What the shop screen reacts to.
pub const SHOP: &[Action] = GRID;
/// What the camp screen reacts to.
pub const CAMP: &[Action] = LIST;
/// What the class change screen reacts to.
pub const CLASS_CHANGE: &[Action] = GRID;
/// What the credits screen reacts to.
pub const CREDITS: &[Action] = LIST;
/// What the dialogue screen reacts to.
pub const DIALOGUE: &[Action] = &[
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    Confirm,
    EndTurn,
    Cancel,
    DialogueLog,
];
/// What the familiarity screen reacts to.
pub const FAMILIARITY: &[Action] = LIST;
/// Any action skips what is animating (`title::pressed`).
pub const GAME_OVER: &[Action] = &Action::ALL;
/// What the to be continued screen reacts to.
pub const TO_BE_CONTINUED: &[Action] = &[Confirm];
/// What the lead select screen reacts to.
pub const LEAD_SELECT: &[Action] = GRID;
/// What the mode select screen reacts to.
pub const MODE_SELECT: &[Action] = LIST;
/// What the preparations screen reacts to.
pub const PREPARATIONS: &[Action] = &[
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    Confirm,
    Cancel,
    Options,
];
/// What the results screen reacts to.
pub const RESULTS: &[Action] = &[Confirm, Cancel];
/// What the save prompt screen reacts to.
pub const SAVE_PROMPT: &[Action] = NO_CANCEL;
/// What the save slots screen reacts to.
pub const SAVE_SLOTS: &[Action] = GRID;
/// Any action skips the cinematic and clears the notice
/// (`title::pressed`).
pub const TITLE: &[Action] = &Action::ALL;
/// What the town screen reacts to.
pub const TOWN: &[Action] = GRID;
/// What the world map screen reacts to.
pub const WORLD_MAP: &[Action] = &[
    CursorUp,
    CursorDown,
    CursorLeft,
    CursorRight,
    Confirm,
    Cancel,
    PrevUnit,
    NextUnit,
    WorldArmy,
    WorldFind,
];

/// Every screen the game can show, for `check --target`: the game's by the
/// table above, the engine's by the widgets they declare themselves with.
pub fn every_screen() -> Vec<ScreenActions<Action>> {
    let own: [(&'static str, &[Action]); 25] = [
        (CinematicScreen::NAME, CINEMATIC),
        (PortraitViewerScreen::NAME, PORTRAIT_VIEWER),
        (ScenePreviewScreen::NAME, SCENE_PREVIEW),
        (SpriteTestScreen::NAME, SPRITE_TEST),
        (FlowScreen::NAME, FLOW),
        (GuidesScreen::NAME, GUIDES),
        (BattleScreen::NAME, BATTLE),
        (RewindScreen::NAME, REWIND),
        (ShopScreen::NAME, SHOP),
        (CampScreen::NAME, CAMP),
        (ClassChangeScreen::NAME, CLASS_CHANGE),
        (CreditsScreen::NAME, CREDITS),
        (DialogueScreen::NAME, DIALOGUE),
        (FamiliarityScreen::NAME, FAMILIARITY),
        (GameOverScreen::NAME, GAME_OVER),
        (ToBeContinuedScreen::NAME, TO_BE_CONTINUED),
        (LeadSelectScreen::NAME, LEAD_SELECT),
        (ModeSelectScreen::NAME, MODE_SELECT),
        (PreparationsScreen::NAME, PREPARATIONS),
        (ResultsScreen::NAME, RESULTS),
        (SavePromptScreen::NAME, SAVE_PROMPT),
        (SlotPickerScreen::NAME, SAVE_SLOTS),
        ("title", TITLE), // check-text: not player text (the screen's name)
        (TownScreen::NAME, TOWN),
        (WorldMapScreen::NAME, WORLD_MAP),
    ];
    let mut screens: Vec<ScreenActions<Action>> = own
        .into_iter()
        .map(|(name, actions)| ScreenActions::declared(name, actions.to_vec()))
        .collect();
    let navigation = || ui_actions::<Action>(NAVIGATION);
    let menu = || ui_actions::<Action>(widgets::Menu::<()>::ACTIONS);
    screens.extend([
        ScreenActions::declared(OptionsScreen::NAME, navigation()),
        ScreenActions::declared(KeyBindingsScreen::NAME, navigation()),
        ScreenActions::declared(ProfileScreen::NAME, navigation()),
        ScreenActions::declared(LayoutPickerScreen::NAME, menu()),
        ScreenActions::declared(DebugMenuScreen::NAME, menu()),
        ScreenActions::declared(GlyphSamplerScreen::NAME, ui_actions(&[UiAction::Cancel])),
        ScreenActions::declared(SceneCameraScreen::NAME, navigation()),
    ]);
    screens
}

/// `screen` declares just `actions`.
#[cfg(test)]
pub(crate) fn assert_declares(
    screen: &dyn crate::screen::Screen<crate::screen::Shuyi>,
    actions: &[Action],
) {
    assert_eq!(
        screen.actions().as_deref(),
        Some(actions),
        "{}",
        screen.name()
    );
}
