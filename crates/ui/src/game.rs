//! [`Game`]: the whole UI behind one call per frame. `app` feeds it raw key
//! and controller-button events and the frame time and blits the buffer it
//! returns; the test `Harness` drives it the same way without a window.
//!
//! The frame loop is Shuyi Engine's (`shuyi_ui::Game`, ticket 1109). This
//! module is how this game starts it: at the title ([`start`]), on a
//! dialogue scene ([`start_on_scene`]) or on a battle
//! ([`start_on_battle`]), with this game's [`hooks`] (the
//! layout picker on the first key, the debug menu on the Debug key).
//! [`system_menu`] is what a platform's own menu (the Playdate's) does.

use crate::prelude::*;
use shuyi_ui::{GameHooks, ScreenHook};
use trpg_content::battle_campaign;
use trpg_core::lead::DEFAULT_NAME;
use trpg_core::{BattleState, GameMode, LeadGender, LeadProfile};

use crate::debug::{self, DebugMenuScreen, ScenePreviewScreen};
use crate::screen::{Ctx, Keyboard, Screen, ScreenStack, Shuyi};
use crate::screens::{
    BattleScreen, LanguageListScreen, LayoutPickerScreen, TitleScreen, language_list, layout_picker,
};

pub use shuyi_ui::RawInputEvent;

pub mod system_menu;

/// This game's UI behind one call per frame ([`shuyi_ui::Game`]).
pub type Game = shuyi_ui::Game<Shuyi>;

/// The result of one frame ([`shuyi_ui::FrameOutput`]).
pub type FrameOutput<'a> = shuyi_ui::FrameOutput<'a, trpg_content::Variant>;

/// Where this game has a say in the frame loop: until a layout is chosen
/// the first key opens the layout picker (ticket 0226,
/// `docs/design/controls.md`), and in builds with debug tools the Debug key
/// opens the debug menu, except over a debug screen ([`debug::SCREENS`]).
pub fn hooks() -> GameHooks<Shuyi> {
    GameHooks {
        first_key: Some(ScreenHook {
            name: LayoutPickerScreen::NAME,
            open: |ctx| Box::new(layout_picker::first(ctx)),
        }),
        debug: Some(ScreenHook {
            name: DebugMenuScreen::NAME,
            open: |ctx| Box::new(debug::menu(ctx)),
        }),
        debug_screens: &debug::SCREENS,
    }
}

/// A game showing `root`, with this game's [`hooks`]. Debug screens (F2)
/// are on in debug builds.
pub fn new(ctx: Ctx, root: Box<dyn Screen<Shuyi>>) -> Game {
    with_atlas_widths(Game::new(ctx, root, hooks()))
}

/// `game`, printing wide what its font atlas has wide (ticket 0236), from
/// the start: a game a test drives has no look for the platform to take
/// them from (`shuyi_platform::App`, which gives the window
/// `CellWidths::of` the atlas when the game has loaded), so it would print
/// every character narrow. The wide characters only, not the atlas's ink
/// boxes, which place key caps: those stay the platform's to give, as they
/// were before wide glyphs.
fn with_atlas_widths(mut game: Game) -> Game {
    let wide = game.ctx().game.content.font.wide_chars();
    let widths = shuyi_assets::font::CellWidths::wide(wide);
    game.set_widths(widths);
    game
}

/// Uses the layout saved in `ctx.storage` (the settings') if none is in
/// use yet.
pub(crate) fn use_saved_layout(ctx: &mut Ctx) {
    if ctx.layout().is_none()
        && let Some(layout) = ctx.saved_layout()
    {
        ctx.use_layout(layout);
    }
}

/// A game starting at the title screen. If no layout is in use yet, the
/// saved one (the settings') is loaded from `ctx.storage`; if none is
/// saved (first launch), the first key press opens the layout picker.
/// A platform without a keyboard ([`Keyboard::Absent`]) never opens
/// it. A first launch with no language picked either asks for it first
/// ([`language_list::asks`], ticket 0825): the language list opens at
/// once, the layout picker under it on a keyboard.
pub fn start(mut ctx: Ctx) -> Game {
    use_saved_layout(&mut ctx);
    let title = if ctx.debug_tools {
        TitleScreen::with_quick_battle(&ctx)
    } else {
        TitleScreen::new(&ctx)
    };
    let mut stack = ScreenStack::new(Box::new(title));
    if language_list::asks(&ctx) {
        if ctx.platform.keyboard == Keyboard::Present {
            stack.push(Box::new(layout_picker::first(&ctx)));
        }
        stack.push(Box::new(LanguageListScreen::new(&ctx)));
    }
    with_atlas_widths(Game::with_stack(ctx, stack, hooks()))
}

/// A game opened straight on the dialogue scene `scene`, for whoever
/// writes scripts (ticket 0723): it plays with everyone there and the
/// default lead of `gender`; when it ends, the [`ScenePreviewScreen`]
/// plays it again or quits. The saved layout is used; if none is saved
/// the layout picker opens first, as in [`start`].
///
/// # Errors
///
/// If there is no such scene: the error lists the scenes there are.
pub fn start_on_scene(mut ctx: Ctx, scene: &str, gender: LeadGender) -> Result<Game, String> {
    let Some(found) = ctx.game.content.dialogue.get(scene).cloned() else {
        let ids: Vec<&str> = ctx
            .game
            .content
            .dialogue
            .scenes
            .keys()
            .map(String::as_str)
            .collect();
        // check-text: not player text (a developer's error)
        return Err(format!(
            "no scene \"{scene}\" in assets/dialogue/; the scenes are: {}",
            ids.join(", ")
        ));
    };
    use_saved_layout(&mut ctx);
    ctx.game.lead = LeadProfile::new(DEFAULT_NAME, gender);
    let preview = ScenePreviewScreen::new(found);
    let playing = preview.play(&ctx);
    let mut stack = ScreenStack::new(Box::new(preview));
    stack.push(Box::new(playing));
    if ctx.layout().is_none() && ctx.platform.keyboard == Keyboard::Present {
        stack.push(Box::new(layout_picker::first(&ctx)));
    }
    Ok(with_atlas_widths(Game::with_stack(ctx, stack, hooks())))
}

/// A game opened straight on the battle `id` at its start, fought by its
/// own characters in Classic, with no chapter around it: what `cargo shuyi
/// bench`'s scenarios measure (ticket 1244). The saved layout is used, as
/// in [`start`].
///
/// # Errors
///
/// If there is no such battle: the error lists the battles there are.
pub fn start_on_battle(mut ctx: Ctx, id: &str) -> Result<Game, String> {
    let content = &ctx.game.content;
    let Some(def) = content.battles.get(id) else {
        let ids: Vec<&str> = content.battles.keys().map(String::as_str).collect();
        // check-text: not player text (a developer's error)
        return Err(format!(
            "no battle \"{id}\" in assets/battles/; the battles are: {}",
            ids.join(", ")
        ));
    };
    let campaign = battle_campaign(content, def, GameMode::Classic, ctx.game.lead.clone());
    let state = BattleState::new(campaign.battle_setup(def, &content.tables())).0;
    let battle = BattleScreen::new(state)
        .with_look(content.battle_look(def))
        .in_battle(&def.id);
    use_saved_layout(&mut ctx);
    Ok(new(ctx, Box::new(battle)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::AudioRequest;
    use crate::console::ConsoleSizes;
    use crate::glyph_buffer::GlyphBuffer;
    use crate::input::{Action, Button, Chord, Key, LEFT_HANDED, PadKind, RIGHT_HANDED};
    use crate::screen::{FrameInput, KeyPrompt, ShuyiCtx};

    use crate::screen::tests::ctx;
    use crate::storage::Storage;

    fn down(key: Key) -> RawInputEvent {
        RawInputEvent::Down(Chord::plain(key))
    }

    fn tap(game: &mut Game, key: Key) -> bool {
        let quit = game.frame(&[down(key)], 0.0).quit;
        game.frame(&[RawInputEvent::Up(key)], 0.0);
        quit
    }

    #[test]
    fn starts_at_the_title_already_drawn() {
        let game = start(ctx());
        assert_eq!(game.top_screen(), Some("title"));
        assert!(!game.quit_requested());
        let expected = {
            let mut g = start(ctx());
            g.frame(&[], 0.0);
            g.buffer().clone()
        };
        assert_eq!(game.buffer(), &expected);
        assert_eq!(game.buffer().spec(), crate::ConsoleSpec::PC);
    }

    fn first_launch() -> Ctx {
        Ctx::embedded().unwrap()
    }

    /// Ticket 0226: until a layout is chosen, the first key (bound or
    /// not) opens the picker and does nothing else; the next keys are the
    /// picker's.
    #[test]
    fn with_no_layout_the_first_key_opens_the_picker() {
        let mut game = start(first_launch());
        assert_eq!(game.screens(), ["title"]);
        assert_eq!(game.ctx().layout(), None);
        // Releases and text alone don't.
        game.frame(&[RawInputEvent::Up(Key::F), RawInputEvent::Text('f')], 0.0);
        assert_eq!(game.screens(), ["title"]);
        // `s` would move the picker down and `f` pick: neither acts.
        game.frame(&[down(Key::S), down(Key::F)], 0.0);
        assert_eq!(game.screens(), ["title", "layout_picker"]);
        assert_eq!(game.ctx().layout(), None);
        assert!(!game.input().is_held(Action::Confirm));
        game.frame(&[RawInputEvent::Up(Key::S), RawInputEvent::Up(Key::F)], 0.0);
        // Open already: keys reach it, and no second picker opens. (Had
        // the first `s` moved the highlight, this one would wrap it back.)
        tap(&mut game, Key::S);
        assert_eq!(game.screens(), ["title", "layout_picker"]);
        tap(&mut game, Key::F);
        assert_eq!(game.screens(), ["title"]);
        assert_eq!(game.ctx().layout(), Some(LEFT_HANDED));
        // Chosen: keys act again and the picker stays away.
        tap(&mut game, Key::J);
        assert_eq!(game.screens(), ["title", "game_flow", "mode_select"]);
    }

    /// Ticket 1033: a platform without a keyboard (the Playdate) starts on
    /// its pad and never opens the layout picker, even for a key press;
    /// its buttons work with no layout chosen.
    #[test]
    fn without_a_keyboard_no_layout_picker_opens() {
        use crate::input::Device;
        let playdate = Device::Pad(PadKind::Playdate);
        let mut ctx = first_launch();
        ctx.platform.keyboard = Keyboard::Absent;
        ctx.device = playdate;
        ctx.use_playdate_buttons();
        let mut game = start(ctx);
        game.frame(&[], 0.0);
        assert_eq!(game.ctx().device, playdate, "help names its buttons");
        game.frame(&[down(Key::Z)], 0.0);
        assert_eq!(game.screens(), ["title"]);
        let press = |b| RawInputEvent::PadDown(b, PadKind::Playdate, 0);
        game.frame(&[press(Button::East)], 0.0);
        game.frame(&[RawInputEvent::PadUp(Button::East, 0)], 0.0);
        // The flow pushes its first screen on its first frame.
        game.frame(&[], 0.0);
        assert_eq!(game.screens(), ["title", "game_flow", "mode_select"]);
        assert_eq!(game.ctx().layout(), None);
        assert_eq!(game.ctx().device, playdate);
    }

    /// A scene preview without a keyboard opens no picker either.
    #[test]
    fn a_scene_preview_without_a_keyboard_opens_no_picker() {
        let mut ctx = first_launch();
        ctx.platform.keyboard = Keyboard::Absent;
        let scene = ctx
            .game
            .content
            .dialogue
            .scenes
            .keys()
            .next()
            .cloned()
            .unwrap();
        let game = start_on_scene(ctx, &scene, LeadGender::Female).unwrap();
        assert!(!game.screens().contains(&LayoutPickerScreen::NAME));
    }

    /// A controller button closes the asked-for picker without choosing,
    /// bound or not, and does nothing else; the next key asks again.
    #[test]
    fn a_pad_button_closes_the_picker_without_choosing() {
        for button in [Button::South, Button::DpadDown, Button::RightStickPress] {
            let mut game = start(first_launch());
            tap(&mut game, Key::Z);
            assert_eq!(game.screens(), ["title", "layout_picker"]);
            pad_tap(&mut game, button);
            assert_eq!(game.screens(), ["title"], "{button:?}");
            assert_eq!(game.ctx().layout(), None);
            assert_eq!(game.ctx().saved_layout(), None);
            tap(&mut game, Key::Z);
            assert_eq!(game.screens(), ["title", "layout_picker"]);
        }
    }

    /// With no layout a controller plays as usual, and the picker opens
    /// over whatever screen the first key finds.
    #[test]
    fn the_picker_opens_over_any_screen() {
        let mut game = start(first_launch());
        pad_tap(&mut game, Button::South);
        assert_eq!(game.screens(), ["title", "game_flow", "mode_select"]);
        tap(&mut game, Key::D);
        assert_eq!(
            game.screens(),
            ["title", "game_flow", "mode_select", "layout_picker"]
        );
        tap(&mut game, Key::F);
        assert_eq!(game.screens(), ["title", "game_flow", "mode_select"]);
        assert_eq!(game.ctx().layout(), Some(RIGHT_HANDED));
        assert_eq!(game.input().keymap(), &game.ctx().keymap);
        tap(&mut game, Key::D);
        assert_eq!(game.screens(), ["title"]);
    }

    #[test]
    fn a_saved_layout_skips_the_picker() {
        let mut ctx = first_launch();
        ctx.storage.write("layout", "LeftHanded").unwrap();
        let game = start(ctx);
        assert_eq!(game.screens(), ["title"]);
        assert_eq!(game.ctx().layout(), Some(LEFT_HANDED));
        assert_eq!(game.input().keymap(), &game.ctx().keymap);
    }

    #[test]
    fn a_layout_in_use_beats_the_saved_one() {
        let mut ctx = ctx();
        ctx.storage.write("layout", "LeftHanded").unwrap();
        let game = start(ctx);
        assert_eq!(game.screens(), ["title"]);
        assert_eq!(game.ctx().layout(), Some(RIGHT_HANDED));
    }

    #[test]
    fn picking_a_layout_switches_the_input_keys() {
        let mut game = start(first_launch());
        tap(&mut game, Key::Z);
        // Picker keys: `s` moves down, `j` picks.
        tap(&mut game, Key::S);
        tap(&mut game, Key::J);
        assert_eq!(game.screens(), ["title"]);
        assert_eq!(game.ctx().layout(), Some(LEFT_HANDED));
        assert_eq!(game.input().keymap(), &game.ctx().keymap);
        // Left-handed: `j` confirms, `f` does nothing, `k` backs out.
        tap(&mut game, Key::F);
        assert_eq!(game.screens(), ["title"]);
        tap(&mut game, Key::J);
        assert_eq!(game.screens(), ["title", "game_flow", "mode_select"]);
        tap(&mut game, Key::K);
        assert_eq!(game.screens(), ["title"]);
        let ctx = game.into_ctx();
        assert_eq!(ctx.saved_layout(), Some(LEFT_HANDED));
    }

    /// `c` as `app` starts it: the title waits for a key or a button.
    fn waiting(mut c: Ctx) -> Ctx {
        c.key_prompt = KeyPrompt::Waiting;
        c
    }

    /// Ticket 0226, first launch: a key at the prompt opens the picker
    /// (and ends the wait); a button goes to the menu without it.
    #[test]
    fn the_first_press_at_the_prompt_decides_about_the_picker() {
        let mut game = start(waiting(first_launch()));
        assert_eq!(game.screens(), ["title"]);
        tap(&mut game, Key::F);
        assert_eq!(game.screens(), ["title", "layout_picker"]);
        assert_eq!(game.ctx().key_prompt, KeyPrompt::Pressed);
        assert_eq!(game.ctx().layout(), None);
        let mut game = start(waiting(first_launch()));
        pad_tap(&mut game, Button::South);
        assert_eq!(game.screens(), ["title"]);
        assert_eq!(game.ctx().key_prompt, KeyPrompt::Pressed);
        // The menu now: the same button chooses New Game.
        pad_tap(&mut game, Button::South);
        assert_eq!(game.screens(), ["title", "game_flow", "mode_select"]);
    }

    fn pad_tap(game: &mut Game, button: Button) -> bool {
        let quit = game
            .frame(&[RawInputEvent::PadDown(button, PadKind::Xbox, 0)], 0.0)
            .quit;
        game.frame(&[RawInputEvent::PadUp(button, 0)], 0.0);
        quit
    }

    /// Ticket 0219: with the default buttons, the bottom face button
    /// confirms and the right one cancels.
    #[test]
    fn pad_buttons_reach_the_top_screen() {
        let mut game = start(ctx());
        assert!(!pad_tap(&mut game, Button::South));
        assert_eq!(game.screens(), ["title", "game_flow", "mode_select"]);
        pad_tap(&mut game, Button::East);
        assert_eq!(game.screens(), ["title"]);
        // Unbound buttons do nothing.
        pad_tap(&mut game, Button::RightStickPress);
        assert_eq!(game.screens(), ["title"]);
        // D-pad up wraps to Quit; South chooses it.
        pad_tap(&mut game, Button::DpadUp);
        assert!(pad_tap(&mut game, Button::South));
    }

    /// Ticket 0220: picking a layout (new bindings) doesn't forget the
    /// device pressed last. (The rest of 0220, the device in the frame of
    /// the press, is the engine's `the_context_knows_the_device_pressed_last`.)
    #[test]
    fn picking_a_layout_keeps_the_device_pressed_last() {
        use crate::input::Device;
        let mut game = start(first_launch());
        pad_tap(&mut game, Button::DpadDown);
        game.ctx_mut().choose_layout(LEFT_HANDED).unwrap();
        game.frame(&[], 0.0);
        assert_eq!(game.input().keymap(), &game.ctx().keymap);
        assert_eq!(game.ctx().device, Device::Pad(PadKind::Xbox));
    }

    /// Ticket 1034: on the Playdate's buttons A's Confirm comes when it is
    /// let go (or after the chord window), with no button going down that
    /// frame: screens still see it as pad input (a letter grid, not a
    /// typing box).
    #[test]
    fn a_chord_buttons_late_action_is_pad_input() {
        let seen = std::rc::Rc::default();
        let mut ctx = ctx();
        ctx.use_playdate_buttons();
        let mut game = new(ctx, Box::new(Spy(std::rc::Rc::clone(&seen))));
        let down = |b| RawInputEvent::PadDown(b, PadKind::Playdate, 0);
        game.frame(&[down(Button::East)], 0.0);
        game.frame(&[RawInputEvent::PadUp(Button::East, 0)], 0.0);
        game.frame(&[], 0.0);
        // A key afterwards is keyboard input again.
        game.frame(&[RawInputEvent::Down(Chord::plain(Key::F))], 0.0);
        let seen = seen.borrow();
        assert!(seen[0].actions.is_empty() && seen[0].pad_pressed());
        assert_eq!(seen[1].actions, [Action::Confirm]);
        assert!(seen[1].pad_pressed(), "Confirm, let go");
        assert!(!seen[2].pad_pressed(), "nothing");
        assert!(!seen[3].pad_pressed(), "a key");
    }

    /// The button that ends the title's wait does nothing else, also when
    /// a chord makes its action wait until it is let go (ticket 1034).
    #[test]
    fn the_button_that_ends_the_wait_gives_nothing_when_let_go() {
        let mut ctx = waiting(ctx());
        ctx.use_playdate_buttons();
        let mut game = start(ctx);
        let down = |b| RawInputEvent::PadDown(b, PadKind::Playdate, 0);
        game.frame(&[down(Button::East)], 0.0);
        game.frame(&[RawInputEvent::PadUp(Button::East, 0)], 0.0);
        game.frame(&[], 1.0);
        assert_eq!(game.ctx().key_prompt, KeyPrompt::Pressed);
        assert_eq!(game.screens(), ["title"]);
        game.frame(&[down(Button::East)], 0.0);
        game.frame(&[RawInputEvent::PadUp(Button::East, 0)], 0.0);
        // The flow pushes its first screen on its first frame.
        game.frame(&[], 0.0);
        assert_eq!(game.screens(), ["title", "game_flow", "mode_select"]);
    }

    #[test]
    fn escape_is_ignored_by_the_first_launch_picker() {
        let mut game = start(first_launch());
        tap(&mut game, Key::Z);
        assert_eq!(
            game.input().keymap().action(Chord::plain(Key::Escape)),
            Some(Action::Cancel)
        );
        tap(&mut game, Key::Escape);
        assert_eq!(game.screens(), ["title", "layout_picker"]);
    }

    #[test]
    fn events_reach_the_top_screen() {
        let mut game = start(ctx());
        assert!(!tap(&mut game, Key::F));
        assert_eq!(game.screens(), ["title", "game_flow", "mode_select"]);
        tap(&mut game, Key::D);
        assert_eq!(game.screens(), ["title"]);
    }

    #[test]
    fn debug_key_opens_the_debug_menu_once() {
        let mut game = start(ctx()).with_debug_screens(true);
        tap(&mut game, Key::F2);
        assert_eq!(game.screens(), ["title", "debug_menu"]);
        tap(&mut game, Key::F2);
        assert_eq!(game.screens(), ["title", "debug_menu"]);
        // Not over a debug tool either.
        tap(&mut game, Key::F);
        assert_eq!(game.screens(), ["title", "debug_menu", "glyph_sampler"]);
        tap(&mut game, Key::F2);
        assert_eq!(game.screens(), ["title", "debug_menu", "glyph_sampler"]);
        tap(&mut game, Key::D);
        tap(&mut game, Key::Down);
        tap(&mut game, Key::F);
        assert_eq!(game.screens(), ["title", "debug_menu", "portrait_viewer"]);
        tap(&mut game, Key::F2);
        assert_eq!(game.screens(), ["title", "debug_menu", "portrait_viewer"]);
        tap(&mut game, Key::D);
        tap(&mut game, Key::D);
        assert_eq!(game.screens(), ["title"]);
    }

    #[test]
    fn debug_screens_follow_the_build() {
        let game = start(Ctx::embedded().unwrap());
        assert_eq!(game.ctx().debug_tools, crate::screen::DEBUG_TOOLS);
        let names = |ctx: Ctx| {
            let mut game = start(ctx);
            tap(&mut game, Key::Down);
            tap(&mut game, Key::F);
            game.screens()
        };
        let mut release = ctx();
        release.debug_tools = false;
        // Down + f chose Options: there is no Quick Battle.
        assert_eq!(names(release), ["title", "options"]);
        let mut debug = ctx();
        debug.debug_tools = true;
        assert_eq!(names(debug), ["title", "game_flow", "preparations"]);
    }

    /// Records what the screen saw.
    struct Spy(std::rc::Rc<std::cell::RefCell<Vec<FrameInput>>>);

    impl Screen<Shuyi> for Spy {
        fn name(&self) -> &'static str {
            "spy"
        }
        fn update(&mut self, _: &mut Ctx, input: &FrameInput) -> crate::screen::Transition {
            self.0.borrow_mut().push(input.clone());
            crate::screen::Transition::None
        }
        fn draw(&self, _: &Ctx, _: &mut GlyphBuffer) {}
    }

    /// The Harness types as the app does: each character with its key
    /// (Shift for capitals), one character per press.
    #[test]
    fn the_harness_types_text_with_its_keys() {
        let seen = std::rc::Rc::default();
        let game = new(ctx(), Box::new(Spy(std::rc::Rc::clone(&seen))));
        let mut h = crate::harness::Harness::from_game(game);
        h.type_text("Ma -é");
        let seen = seen.borrow();
        let typed: Vec<(Vec<String>, Vec<char>)> = seen
            .iter()
            .filter(|i| !i.text().is_empty())
            .map(|i| {
                let keys = i.pressed_chords().iter().map(ToString::to_string).collect();
                (keys, i.text().to_vec())
            })
            .collect();
        let expect = |k: &[&str], c| (k.iter().map(|&s| s.to_owned()).collect(), vec![c]);
        assert_eq!(
            typed,
            [
                expect(&["Shift+m"], 'M'),
                expect(&["a"], 'a'),
                expect(&["Space"], ' '),
                expect(&["-"], '-'),
                expect(&[], 'é'),
            ]
        );
    }

    /// A won battle asks for its victory sting once, at the banner (0809),
    /// and pops back to the title, which asks for its music again.
    #[test]
    fn the_title_music_returns_after_a_battle() {
        use crate::screens::battle::testing::battle_with;
        use crate::screens::battle::{BattleScreen, quick_battle};
        use trpg_core::{Objective, Pos, UnitId};

        let c = ctx();
        // The run's first pick from the pool.
        let sting = crate::audio::pick_from_pool(&c.game.content.audio, "victory", c.music_seed);
        let sting = AudioRequest::play_music(sting.unwrap().to_owned(), None);
        let quick = quick_battle(&c.game.content).unwrap();
        // The lord next to a brigand on 1 HP, which one hit routs.
        let mut units = quick.units().to_vec();
        units.retain(|u| u.id == UnitId(1) || u.id == UnitId(4));
        units[0].pos = Pos::new(7, 2);
        units[1].hp = 1;
        let rout = Objective::Rout { turn_limit: None };
        let battle = battle_with(&c, quick.map().clone(), units, rout);
        let mut stack = ScreenStack::new(Box::new(TitleScreen::with_quick_battle(&c)));
        stack.push(Box::new(BattleScreen::new(battle)));
        let mut game = Game::with_stack(c, stack, hooks());
        let mut music = Vec::new();
        // The battle's sounds (0424) and the EXP bar's (0605) aside.
        let music_of = |audio: &[AudioRequest]| {
            let music = |r: &&AudioRequest| {
                !matches!(
                    r,
                    AudioRequest::PlaySound { .. }
                        | AudioRequest::StartSound { .. }
                        | AudioRequest::StopSound { .. }
                )
            };
            audio.iter().filter(music).cloned().collect::<Vec<_>>()
        };
        // Select the lord, stay, Attack, the brigand, confirm the forecast.
        for key in [Key::F; 5] {
            let quit = game.frame(&[down(key)], 0.0).quit;
            assert!(!quit);
            music.extend(music_of(game.frame(&[RawInputEvent::Up(key)], 0.5).audio));
        }
        for _ in 0..60 {
            music.extend(music_of(game.frame(&[], 0.5).audio));
        }
        assert_eq!(music, [sting]);
        assert_eq!(game.screens(), ["title", "battle"]);
        // Past the VICTORY banner, back at the title (the battle screen's
        // sound handles go with it).
        assert_eq!(music_of(game.frame(&[down(Key::F)], 0.0).audio), []);
        assert_eq!(game.screens(), ["title"]);
        let title = AudioRequest::play_music("title", None);
        let back = game.frame(&[RawInputEvent::Up(Key::F)], 0.0).audio;
        assert_eq!(back[0], title);
        // And the title's campfire with it (ticket 0922).
        assert_eq!(
            back[1..].iter().map(AudioRequest::cue).collect::<Vec<_>>(),
            [Some("title_campfire")]
        );
        assert_eq!(game.frame(&[], 0.1).audio, []);
    }
}
