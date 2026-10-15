use super::*;
use crate::audio::{AudioRequest, pick_from_pool};
use crate::color::UiColor;
use crate::glyph_buffer::Cell;
use crate::input::Action;
use crate::screen::tests::ctx;
use crate::storage::Storage;
use Item::{Continue, Credits, LoadGame, NewGame, Options, QuickBattle, Quit};

/// The pool the Quick Battle's file names (`assets-test/battles/quick.ron`).
const QUICK_BATTLE_MUSIC_POOL: &str = "skirmish";

fn input(actions: &[Action]) -> FrameInput {
    FrameInput::new(actions.to_vec(), 0.0, vec![])
}

fn outcome(screen: &mut dyn Screen, actions: &[Action]) -> String {
    format!("{:?}", screen.update(&mut ctx(), &input(actions)))
}

#[test]
fn title_menu_transitions() {
    use Action::{Cancel, Confirm, CursorDown, CursorUp};
    let mut t = TitleScreen::new(&ctx());
    assert_eq!(t.name(), "title");
    assert!(!t.is_overlay());
    assert_eq!(outcome(&mut t, &[]), "None");
    assert_eq!(outcome(&mut t, &[Cancel]), "None");
    assert_eq!(outcome(&mut t, &[Confirm]), "Push(mode_select)");
    assert_eq!(outcome(&mut t, &[CursorDown, Confirm]), "Push(options)");
    assert_eq!(outcome(&mut t, &[CursorDown, Confirm]), "Push(credits)");
    assert_eq!(outcome(&mut t, &[CursorDown, Confirm]), "Quit");
    assert_eq!(outcome(&mut t, &[CursorUp, CursorUp, CursorUp]), "None");
    // Actions after the one that transitions are dropped.
    assert_eq!(outcome(&mut t, &[Confirm, CursorDown]), "Push(mode_select)");
    assert_eq!(outcome(&mut t, &[Confirm]), "Push(mode_select)");
}

#[test]
fn debug_title_offers_quick_battle() {
    use Action::{Confirm, CursorDown, CursorUp};
    let mut t = TitleScreen::with_quick_battle(&ctx());
    assert_eq!(
        t.items,
        [NewGame, LoadGame, QuickBattle, Options, Credits, Quit]
    );
    assert_eq!(outcome(&mut t, &[Confirm]), "Push(mode_select)");
    assert_eq!(
        outcome(&mut t, &[CursorDown, Confirm]),
        "Push(preparations)"
    );
    assert_eq!(outcome(&mut t, &[CursorDown, Confirm]), "Push(options)");
    assert_eq!(outcome(&mut t, &[CursorDown, Confirm]), "Push(credits)");
    assert_eq!(outcome(&mut t, &[CursorDown, Confirm]), "Quit");
    assert_eq!(
        outcome(&mut t, &[CursorUp, CursorUp, CursorUp, CursorUp, Confirm]),
        "Push(mode_select)"
    );
    // Without its chapter, Quick Battle does nothing.
    let mut c = ctx();
    c.content.chapters.clear();
    t.menu = TitleScreen::with_quick_battle(&ctx()).menu;
    let input = FrameInput::new(vec![CursorDown, Confirm], 0.0, vec![]);
    assert_eq!(format!("{:?}", t.update(&mut c, &input)), "None");
    assert_eq!(
        TitleScreen::new(&ctx()).items,
        [NewGame, LoadGame, Options, Credits, Quit]
    );
}

/// The music each update asks for, as cue names (`-` for a stop).
fn music_of(t: &mut TitleScreen, c: &mut Ctx, actions: &[Action]) -> Vec<String> {
    t.update(c, &input(actions));
    c.audio
        .take()
        .iter()
        .filter(|r| !matches!(r, AudioRequest::PlaySound { .. }))
        .map(|r| r.cue().unwrap_or("-").to_owned())
        .collect()
}

/// The sounds (not music) each update plays, as cue names.
fn sounds_of(s: &mut dyn Screen, c: &mut Ctx, actions: &[Action]) -> Vec<String> {
    s.update(c, &input(actions));
    c.audio
        .take()
        .iter()
        .filter(|r| matches!(r, AudioRequest::PlaySound { .. }))
        .filter_map(|r| r.cue().map(str::to_owned))
        .collect()
}

#[test]
fn menu_sounds() {
    use Action::{Cancel, Confirm, CursorDown, CursorUp};
    let mut c = ctx();
    let mut t = TitleScreen::new(&ctx());
    assert_eq!(
        sounds_of(&mut t, &mut c, &[CursorDown, CursorUp]),
        ["menu_move"; 2]
    );
    // Nothing to back out of: Cancel is silent.
    assert!(sounds_of(&mut t, &mut c, &[Cancel]).is_empty());
    assert_eq!(sounds_of(&mut t, &mut c, &[Confirm]), ["menu_select"]);
}

#[test]
fn title_music_plays_on_show_and_again_after_a_quick_battle() {
    use Action::{Confirm, CursorDown};
    let mut c = ctx();
    let mut t = TitleScreen::with_quick_battle(&ctx());
    assert_eq!(music_of(&mut t, &mut c, &[]), [TITLE_MUSIC]);
    assert!(music_of(&mut t, &mut c, &[CursorDown]).is_empty());
    let pool = c.content.audio.pools[QUICK_BATTLE_MUSIC_POOL].clone();
    let first = music_of(&mut t, &mut c, &[Confirm]);
    assert_eq!(first.len(), 1);
    assert!(pool.contains(&first[0]), "{first:?}");
    // Back on top after the battle: the title music again, once.
    assert_eq!(music_of(&mut t, &mut c, &[]), [TITLE_MUSIC]);
    assert!(music_of(&mut t, &mut c, &[]).is_empty());
}

/// New Game's battles change the music too, so the title asks for its
/// own again once it is back on top.
#[test]
fn title_music_plays_again_after_new_game() {
    let mut c = ctx();
    let mut t = TitleScreen::new(&ctx());
    assert_eq!(music_of(&mut t, &mut c, &[]), [TITLE_MUSIC]);
    // The mode screen keeps the title music: nothing asked.
    assert!(music_of(&mut t, &mut c, &[Action::Confirm]).is_empty());
    assert_eq!(music_of(&mut t, &mut c, &[]), [TITLE_MUSIC]);
    assert!(music_of(&mut t, &mut c, &[]).is_empty());
}

#[test]
fn each_quick_battle_rolls_its_track_afresh() {
    let mut c = ctx();
    let mut t = TitleScreen::with_quick_battle(&ctx());
    let pool = c.content.audio.pools[QUICK_BATTLE_MUSIC_POOL].clone();
    let mut picked = std::collections::BTreeSet::new();
    for _ in 0..100 {
        // Down from New Game, then Confirm: Quick Battle.
        let cues = music_of(&mut t, &mut c, &[Action::CursorDown, Action::Confirm]);
        picked.extend(cues.into_iter().filter(|cue| cue != TITLE_MUSIC));
        t.menu = TitleScreen::with_quick_battle(&ctx()).menu;
    }
    // Same seed every time, yet the counter varies the pick.
    assert_eq!(picked.len(), pool.len(), "{picked:?}");
}

/// The n-th Quick Battle's track comes from the seed mixed with n.
#[test]
fn quick_battle_tracks_follow_the_seed_and_the_count() {
    let mut c = ctx();
    c.music_seed = 0xA5A5;
    let mut t = TitleScreen::with_quick_battle(&ctx());
    for n in 0..16 {
        let cues = music_of(&mut t, &mut c, &[Action::CursorDown, Action::Confirm]);
        let want = pick_from_pool(&c.content.audio, QUICK_BATTLE_MUSIC_POOL, 0xA5A5 ^ n);
        assert_eq!(cues.last().map(String::as_str), want, "battle {n}");
        t.menu = TitleScreen::with_quick_battle(&ctx()).menu;
    }
}

/// A context whose storage holds `suspend` as the suspend save and
/// `slot` in slot 3.
fn ctx_with_saves(suspend: Option<&str>, slot: Option<&str>) -> Ctx {
    let mut c = ctx();
    if let Some(text) = suspend {
        c.storage.write(save::SUSPEND_KEY, text).unwrap();
    }
    if let Some(text) = slot {
        c.storage.write(&save::slot_key(3), text).unwrap();
    }
    c
}

fn labels(t: &TitleScreen) -> Vec<(&str, bool)> {
    let items = t.menu.items().iter();
    items.map(|i| (i.label.as_str(), i.enabled)).collect()
}

#[test]
fn the_menu_follows_the_saves() {
    // No saves: no Continue, Load Game disabled.
    let none = [
        ("New Game", true),
        ("Load Game", false),
        ("Options", true),
        ("Credits", true),
        ("Quit", true),
    ];
    assert_eq!(labels(&TitleScreen::new(&ctx())), none);
    assert_eq!(labels(&TitleScreen::new(&ctx()).refreshed(&ctx())), none);
    // A slot with anything in it: Load Game.
    let c = ctx_with_saves(None, Some("junk"));
    let t = TitleScreen::new(&c).refreshed(&c);
    let saved = [
        ("New Game", true),
        ("Load Game", true),
        ("Options", true),
        ("Credits", true),
        ("Quit", true),
    ];
    assert_eq!(labels(&t), saved);
    assert_eq!(t.items, [NewGame, LoadGame, Options, Credits, Quit]);
    assert_eq!(t.menu.focus(), 0);
    // A suspend save: Continue, first and focused.
    let c = ctx_with_saves(Some("junk"), None);
    let t = TitleScreen::with_quick_battle(&c).refreshed(&c);
    assert_eq!(
        labels(&t),
        [
            ("Continue", true),
            ("New Game", true),
            ("Load Game", false),
            ("Quick Battle", true),
            ("Options", true),
            ("Credits", true),
            ("Quit", true)
        ]
    );
    assert_eq!(
        t.items,
        [
            Continue,
            NewGame,
            LoadGame,
            QuickBattle,
            Options,
            Credits,
            Quit
        ]
    );
    assert_eq!(t.menu.focus(), 0);
}

/// A refresh that changes nothing keeps the focus; one that changes a
/// line (here `Load Game` can now be chosen) starts at the top again.
#[test]
fn the_focus_stays_unless_the_menu_changed() {
    use Action::CursorDown;
    let mut c = ctx();
    let mut t = TitleScreen::new(&c);
    // Down from New Game skips the disabled Load Game: Options.
    t.update(&mut c, &input(&[CursorDown]));
    assert_eq!(t.menu.focus(), 2);
    t.refresh(&c);
    assert_eq!(t.menu.focus(), 2);
    c.storage.write(&save::slot_key(3), "junk").unwrap();
    t.refresh(&c);
    assert_eq!(t.menu.focus(), 0);
    assert!(t.menu.items()[1].enabled);
}

/// The labels follow the language at each refresh, and the focus
/// stays (ticket 0233).
#[test]
fn the_menu_is_labelled_in_the_language_in_use() {
    let mut c = ctx();
    let mut t = TitleScreen::with_quick_battle(&c);
    t.update(&mut c, &input(&[Action::CursorDown]));
    assert_eq!(t.menu.focus(), 2);
    c.lang = trpg_content::LangCode::new("test").unwrap();
    t.refresh(&c);
    assert_eq!(
        labels(&t),
        [
            ("NEW GAME", true),
            ("LOAD GAME", false),
            ("QUICK BATTLE", true),
            // Missing from the test pack: English.
            ("OPTIONS", true),
            ("Credits", true),
            ("QUIT", true)
        ]
    );
    assert_eq!(t.menu.focus(), 2);
}

#[test]
fn every_item_has_its_own_text() {
    let c = ctx();
    let all = [
        Continue,
        NewGame,
        LoadGame,
        QuickBattle,
        Options,
        Credits,
        Quit,
    ];
    let labels: std::collections::BTreeSet<&str> = all.iter().map(|i| c.text(i.key())).collect();
    assert_eq!(labels.len(), all.len());
}

#[test]
fn the_first_update_and_every_return_look_for_saves() {
    use Action::{Confirm, CursorDown};
    let mut c = ctx_with_saves(None, Some("junk"));
    let mut t = TitleScreen::new(&ctx());
    assert!(!t.menu.items()[1].enabled);
    // The first update finds the save; the focus moves onto Load Game.
    assert_eq!(
        format!("{:?}", t.update(&mut c, &input(&[CursorDown, Confirm]))),
        "Push(save_slots)"
    );
    // Nothing changed while it was open: the focus stays.
    t.update(&mut c, &input(&[]));
    assert_eq!(t.menu.focus(), 1);
    // The saves don't change under the title itself: no look.
    c.storage.write(save::SUSPEND_KEY, "junk").unwrap();
    t.update(&mut c, &input(&[]));
    assert_eq!(t.items, [NewGame, LoadGame, Options, Credits, Quit]);
    // Back from another screen with a battle suspended: Continue,
    // focused.
    t.update(&mut c, &input(&[Confirm]));
    t.update(&mut c, &input(&[]));
    assert_eq!(
        t.items,
        [Continue, NewGame, LoadGame, Options, Credits, Quit]
    );
    assert_eq!(t.menu.focus(), 0);
}

#[test]
fn continue_with_a_save_that_cant_be_read_says_why() {
    use Action::{Confirm, CursorDown};
    let mut c = ctx_with_saves(Some("junk"), None);
    let mut t = TitleScreen::new(&c).refreshed(&c);
    assert_eq!(t.notice(), None);
    assert_eq!(outcome_in(&mut t, &mut c, &[Confirm]), "None");
    assert_eq!(t.notice(), Some("This save can't be read"));
    c.audio.take();
    // The save is still there, and so is Continue.
    assert_eq!(
        c.storage.read(save::SUSPEND_KEY),
        Ok(Some("junk".to_owned()))
    );
    assert_eq!(t.items[0], Continue);
    // No key, no change; the next key clears the notice.
    t.update(&mut c, &input(&[]));
    assert!(t.notice().is_some());
    t.update(&mut c, &input(&[CursorDown]));
    assert_eq!(t.notice(), None);
    c.audio.take();
    // Chosen: the select sound, then the refusal.
    t.menu = TitleScreen::new(&c).refreshed(&c).menu;
    assert_eq!(
        sounds_of(&mut t, &mut c, &[Confirm]),
        ["menu_select", MenuSound::Denied.cue()]
    );
}

fn outcome_in(screen: &mut dyn Screen, c: &mut Ctx, actions: &[Action]) -> String {
    format!("{:?}", screen.update(c, &input(actions)))
}

#[test]
fn texts_name_the_layout_keys() {
    let mut c = ctx();
    assert_eq!(TitleScreen::help(&c), "arrows move · f select · d back");
    c.use_layout(crate::input::Layout::LeftHanded);
    assert_eq!(TitleScreen::help(&c), "wasd move · j select · k back");
    c.use_layout(crate::input::Layout::RightHanded);
    c.keymap = crate::input::Keymap::new(
        crate::input::Action::ALL
            .iter()
            .flat_map(|&a| c.keymap.chords_for(a).into_iter().map(move |ch| (ch, a)))
            .filter(|&(_, a)| a != Action::Cancel),
        c.keymap.repeat(),
    );
    // Cancel with no key of its own (the fixed Esc isn't named).
    assert_eq!(
        TitleScreen::help(&c),
        "arrows move · f select · ! not mapped back"
    );
}

/// Opaque screens must paint every cell, not rely on `Game` clearing.
#[test]
fn screens_cover_the_whole_buffer() {
    use crate::console::ConsoleSpec;
    let c = ctx();
    let mut naming = crate::screens::LeadSelectScreen::new();
    let typing = FrameInput::new(vec![Action::CursorDown], 0.0, vec![]);
    naming.update(&mut ctx(), &typing);
    naming.update(&mut ctx(), &input(&[Action::Confirm]));
    // Game Over opens over the lost battle; it covers the buffer once
    // its animation is over (0830).
    let mut game_over = crate::screens::GameOverScreen::new(&c, false);
    game_over.skip();
    let screens: [&dyn Screen; 8] = [
        &TitleScreen::new(&ctx()),
        &TitleScreen::with_quick_battle(&ctx()),
        &crate::screens::ModeSelectScreen::new(&c),
        &crate::screens::LeadSelectScreen::new(),
        &naming,
        &game_over,
        &crate::screens::ToBeContinuedScreen,
        &CreditsScreen::new(&c),
    ];
    for screen in screens {
        let stale = Cell::new(
            'x',
            c.palette.get(UiColor::Enemy),
            c.palette.get(UiColor::Enemy),
        );
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
        screen.draw(&c, &mut buf);
        let left = (0..i32::from(ConsoleSpec::PC.rows))
            .flat_map(|y| (0..i32::from(ConsoleSpec::PC.cols)).map(move |x| (x, y)))
            .filter(|&(x, y)| buf.get(x, y) == Some(&stale))
            .count();
        assert_eq!(left, 0, "{} left stale cells", screen.name());
    }
}

/// The menu of `view` as `(label, enabled, focused)`.
fn shown(view: &TitleView) -> Vec<(&str, bool, bool)> {
    let items = view.menu.as_ref().map_or(&[][..], |m| &m.items[..]);
    items
        .iter()
        .map(|i| (i.label.as_str(), i.enabled, i.focused))
        .collect()
}

#[test]
fn the_first_frame_shows_the_names_the_menu_and_the_help() {
    let c = ctx();
    let t = TitleScreen::new(&c);
    let view = t.view(&c);
    assert!(c.content.title.backdrops.contains(&view.backdrop.unwrap()));
    assert_eq!(view.sparks, None);
    assert_eq!(view.prompt, None);
    assert_eq!(view.notice, None);
    assert_eq!(
        view.help.as_deref(),
        Some("arrows move · f select · d back")
    );
    assert_eq!(view.debug_hint.as_deref(), Some("F2 debug"));
    assert_eq!(
        shown(&view),
        [
            ("New Game", true, true),
            ("Load Game", false, false),
            ("Options", true, false),
            ("Credits", true, false),
            ("Quit", true, false),
        ]
    );
    assert_eq!(view.menu, Some(t.menu.view()));
}

#[test]
fn the_view_follows_the_focus_the_saves_and_the_debug_tools() {
    use Action::CursorDown;
    let mut c = ctx_with_saves(Some("junk"), Some("junk"));
    let mut t = TitleScreen::with_quick_battle(&c).refreshed(&c);
    let focused = |t: &TitleScreen, c: &Ctx| {
        let view = t.view(c);
        let menu = view.menu.unwrap();
        menu.focused().map(|i| i.label.clone())
    };
    assert_eq!(focused(&t, &c).as_deref(), Some("Continue"));
    t.update(&mut c, &input(&[CursorDown, CursorDown]));
    assert_eq!(focused(&t, &c).as_deref(), Some("Load Game"));
    // Load Game can be chosen now, and Quick Battle is there.
    let view = t.view(&c);
    let labels: Vec<_> = shown(&view).into_iter().map(|(l, e, _)| (l, e)).collect();
    assert_eq!(
        labels,
        [
            ("Continue", true),
            ("New Game", true),
            ("Load Game", true),
            ("Quick Battle", true),
            ("Options", true),
            ("Credits", true),
            ("Quit", true)
        ]
    );
    // Without debug tools the hint isn't there.
    c.debug_tools = false;
    assert_eq!(t.view(&c).debug_hint, None);
}

/// The notice for a save that can't be read shows until the next key.
#[test]
fn the_notice_shows_until_the_next_key() {
    use Action::{Confirm, CursorDown};
    let mut c = ctx_with_saves(Some("junk"), None);
    let mut t = TitleScreen::new(&c).refreshed(&c);
    assert_eq!(t.view(&c).notice, None);
    t.update(&mut c, &input(&[Confirm]));
    let view = t.view(&c);
    assert_eq!(view.notice.as_deref(), Some("This save can't be read"));
    // The menu and help are still shown under it.
    assert!(view.menu.is_some() && view.help.is_some());
    t.update(&mut c, &input(&[]));
    assert!(t.view(&c).notice.is_some());
    t.update(&mut c, &input(&[CursorDown]));
    assert_eq!(t.view(&c).notice, None);
}

/// The web build's first frame: the prompt takes the menu's place, and the
/// help line isn't shown; the hint at the debug menu still is.
#[test]
fn the_prompt_stands_in_for_the_menu_until_a_key_is_pressed() {
    let mut c = ctx();
    c.key_prompt = KeyPrompt::Waiting;
    let mut t = TitleScreen::new(&c);
    let view = t.view(&c);
    assert_eq!(view.prompt.as_deref(), Some("Press any key or button"));
    assert_eq!(
        (&view.menu, &view.help, &view.notice),
        (&None, &None, &None)
    );
    assert_eq!(view.debug_hint.as_deref(), Some("F2 debug"));
    // A key ends the wait: the menu, the help and no prompt.
    c.key_prompt = KeyPrompt::Pressed;
    t.update(&mut c, &input(&[]));
    let view = t.view(&c);
    assert_eq!(view.prompt, None);
    assert!(view.menu.is_some() && view.help.is_some());
    // The prompt is said in the language in use.
    let mut c = ctx();
    c.key_prompt = KeyPrompt::Waiting;
    c.lang = trpg_content::LangCode::new("test").unwrap();
    assert_eq!(
        TitleScreen::new(&c).view(&c).prompt.as_deref(),
        Some("PRESS ANY KEY OR BUTTON")
    );
}

/// The logic modules of the converted screens say what they show, never
/// how it looks (ADR-0054): none of them names a colour, a rectangle, a
/// box style or a cell.
#[test]
fn the_logic_modules_name_no_cells_or_colours() {
    let sources = [
        ("title.rs", include_str!("../title.rs")),
        ("key_bindings.rs", include_str!("../key_bindings.rs")),
        ("layout_picker.rs", include_str!("../layout_picker.rs")),
        ("mode_select.rs", include_str!("../mode_select.rs")),
        ("familiarity.rs", include_str!("../familiarity.rs")),
        ("game_over.rs", include_str!("../game_over.rs")),
        ("save.rs", include_str!("../save.rs")),
        ("results.rs", include_str!("../results.rs")),
        ("credits.rs", include_str!("../credits.rs")),
        ("preparations.rs", include_str!("../preparations.rs")),
        ("lead_select.rs", include_str!("../lead_select.rs")),
        ("class_change.rs", include_str!("../class_change.rs")),
        ("battle/mod.rs", include_str!("../battle/mod.rs")),
    ];
    // Logic with no screen of its own to draw: the battle screen's modes
    // and the parts of it with a view of their own.
    let drawless = [
        ("battle/mode.rs", include_str!("../battle/mode.rs")),
        ("battle/forecast.rs", include_str!("../battle/forecast.rs")),
        ("battle/info.rs", include_str!("../battle/info.rs")),
        ("battle/playback.rs", include_str!("../battle/playback.rs")),
        ("battle/progress.rs", include_str!("../battle/progress.rs")),
        ("battle/rewind.rs", include_str!("../battle/rewind.rs")),
    ];
    for (file, source) in sources.iter().chain(&drawless) {
        // A picture is placed by the skin too (the view names it).
        for name in ["UiColor", "Rect", "BoxStyle", "Cell", "draw_portrait"] {
            assert!(!names_word(source, name), "{file} names `{name}`");
        }
    }
    for (file, source) in sources {
        // Their `draw` is one call to the skin (a file with two screens,
        // one for each; the save screens' skin has a painter for each).
        let draws = source.matches("fn draw(").count();
        let paints = source
            .lines()
            .filter(|l| l.contains("glyph::paint") && l.contains("(ctx, &self.view(ctx), buf);"))
            .count();
        assert!(
            draws > 0 && paints == draws,
            "{file}'s draw doesn't paint its view"
        );
    }
    for (file, source) in drawless {
        // Nothing of theirs draws: the battle skin paints their views.
        assert!(!source.contains("GlyphBuffer"), "{file} draws");
    }
}

/// Whether `word` appears in `source` as a whole identifier.
fn names_word(source: &str, word: &str) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    source.match_indices(word).any(|(at, _)| {
        let before = source[..at].chars().next_back();
        let after = source[at + word.len()..].chars().next();
        !before.is_some_and(ident) && !after.is_some_and(ident)
    })
}

#[test]
fn names_word_finds_whole_identifiers_only() {
    assert!(names_word("use crate::color::UiColor;", "UiColor"));
    assert!(names_word("let r: Rect = x;", "Rect"));
    assert!(names_word("Cell::new(' ')", "Cell"));
    assert!(!names_word("RectSomething and MyCell and Cells", "Cell"));
    assert!(!names_word("RectSomething", "Rect"));
    assert!(!names_word("", "Rect"));
}

fn tick(dt: f32) -> FrameInput {
    FrameInput::new(vec![], dt, vec![])
}

/// A launch with `c`'s storage: a new title, updated once.
fn launch(c: &mut Ctx) -> Option<ImageId> {
    let mut t = TitleScreen::new(c);
    t.update(c, &input(&[]));
    t.view(c).backdrop
}

#[test]
fn the_backdrop_is_picked_at_the_start_and_kept_for_the_launch() {
    let mut c = ctx();
    let all = c.content.title.backdrops.clone();
    assert!(all.len() >= 2);
    let mut t = TitleScreen::new(&c);
    let picked = t.view(&c).backdrop.unwrap();
    assert!(all.contains(&picked));
    assert_eq!(c.storage.computer().read(BACKDROP_KEY), Ok(None));
    // The first update remembers it for the next launch.
    t.update(&mut c, &input(&[]));
    assert_eq!(t.view(&c).backdrop, Some(picked));
    assert_eq!(
        c.storage.computer().read(BACKDROP_KEY),
        Ok(Some(picked.path().to_owned()))
    );
    // Later updates, another seed and other screens in between don't
    // change it.
    c.music_seed ^= 0xABCD;
    t.update(&mut c, &input(&[Action::CursorDown]));
    t.refresh(&c);
    t.update(&mut c, &input(&[]));
    assert_eq!(t.view(&c).backdrop, Some(picked));
    // It is stored once: a later change of what is stored stays.
    c.storage
        .computer_mut()
        .write(BACKDROP_KEY, "other")
        .unwrap();
    t.update(&mut c, &input(&[]));
    assert_eq!(
        c.storage.computer().read(BACKDROP_KEY),
        Ok(Some("other".to_owned()))
    );
    // While the prompt waits it is there too.
    let mut c = ctx();
    c.key_prompt = KeyPrompt::Waiting;
    let mut t = TitleScreen::new(&c);
    t.update(&mut c, &input(&[]));
    assert!(t.view(&c).backdrop.is_some());
    assert!(t.view(&c).prompt.is_some());
    assert!(c.storage.computer().read(BACKDROP_KEY).unwrap().is_some());
}

#[test]
fn the_next_launch_never_shows_the_same_backdrop() {
    let mut c = ctx();
    let all = c.content.title.backdrops.clone();
    let mut seen = std::collections::BTreeSet::new();
    let mut last = None;
    for seed in 0..200 {
        c.music_seed = seed;
        let picked = launch(&mut c);
        assert!(picked.is_some_and(|p| all.contains(&p)));
        assert_ne!(picked, last, "seed {seed}");
        seen.extend(picked.map(ImageId::path));
        last = picked;
    }
    assert_eq!(seen.len(), all.len(), "every backdrop gets its turn");
    // The pick follows the seed: the same launch twice is the same pick.
    let pick_with = |seed: u64, stored: &str| {
        let mut c = ctx();
        c.music_seed = seed;
        c.storage
            .computer_mut()
            .write(BACKDROP_KEY, stored)
            .unwrap();
        launch(&mut c)
    };
    let first = all[0].path();
    assert_eq!(pick_with(5, first), pick_with(5, first));
    assert!((0..50).any(|seed| pick_with(seed, first) != pick_with(5, first)));
    // A stored name that is no backdrop (an old build's) rules nothing out.
    let any: std::collections::BTreeSet<_> = (0..200)
        .filter_map(|seed| pick_with(seed, "title/gone.png"))
        .collect();
    assert_eq!(any.len(), all.len());
}

#[test]
fn one_backdrop_is_shown_every_launch_and_none_is_none() {
    let mut c = ctx();
    c.content.title.backdrops.truncate(1);
    let only = c.content.title.backdrops[0];
    for seed in 0..5 {
        c.music_seed = seed;
        assert_eq!(launch(&mut c), Some(only));
    }
    let mut c = ctx();
    c.content.title.backdrops.clear();
    assert_eq!(launch(&mut c), None);
    assert_eq!(c.storage.computer().read(BACKDROP_KEY), Ok(None));
}

#[test]
fn the_sparks_follow_how_long_the_title_was_updated() {
    let mut c = ctx();
    let mut t = TitleScreen::new(&c);
    // Only with idle motion on (the game; not a test that doesn't ask).
    t.update(&mut c, &tick(0.25));
    assert_eq!(t.view(&c).sparks, None);
    c.idle_motion = IdleMotion::On;
    assert_eq!(t.view(&c).sparks, Some(250));
    t.update(&mut c, &tick(0.5));
    assert_eq!(t.view(&c).sparks, Some(750));
    // A frame that isn't a time, or goes backwards, adds nothing.
    for bad in [f32::NAN, f32::INFINITY, -1.0] {
        t.update(&mut c, &tick(bad));
        assert_eq!(t.view(&c).sparks, Some(750), "{bad}");
    }
    // It runs while the prompt waits, too, from 0.
    let mut c = ctx();
    c.idle_motion = IdleMotion::On;
    c.key_prompt = KeyPrompt::Waiting;
    let mut t = TitleScreen::new(&c);
    assert_eq!(t.view(&c).sparks, Some(0));
    t.update(&mut c, &tick(1.0));
    assert_eq!(t.view(&c).sparks, Some(1000));
    // A very long time stops at the biggest number.
    t.time = 1.0e30;
    assert_eq!(t.view(&c).sparks, Some(u32::MAX));
}

/// A frame of `dt` seconds with the title song `position` seconds in.
fn song_tick(t: &mut TitleScreen, c: &mut Ctx, position: f32, dt: f32) {
    c.music_clock = Some(crate::audio::MusicClock {
        cue: TITLE_MUSIC.to_owned(),
        position,
        length: 133.743,
    });
    t.update(c, &tick(dt));
}

/// What the view shows of the cinematic: its time in milliseconds and its
/// brightness.
fn playing(t: &TitleScreen, c: &Ctx) -> Option<(u32, u8)> {
    t.view(c).cinematic.map(|shot| (shot.t_ms, shot.brightness))
}

/// The title with the cinematic playing alone, the song `position` in.
fn in_cinematic(c: &mut Ctx, position: f32) -> TitleScreen {
    let mut t = TitleScreen::new(c);
    song_tick(&mut t, c, position, IDLE_SECS);
    song_tick(&mut t, c, position, FADE_OUT_SECS);
    assert_eq!(t.phase, Phase::Cinematic);
    c.audio.take();
    t
}

#[test]
fn a_part_of_full_is_rounded_and_stays_in_range() {
    assert_eq!(of_full(0.0), 0);
    assert_eq!(of_full(0.25), 64);
    assert_eq!(of_full(0.5), 128);
    assert_eq!(of_full(1.0), 255);
    assert_eq!(of_full(7.0), 255);
    assert_eq!(of_full(-1.0), 0);
    assert_eq!(of_full(f32::NAN), 0);
}

#[test]
fn a_frame_is_its_share_of_a_fade() {
    assert!((share(0.25, 0.5) - 0.5).abs() < f32::EPSILON);
    assert!((share(0.5, 2.0) - 0.25).abs() < f32::EPSILON);
    assert!((share(0.5, FADE_OUT_SECS) - 0.5 / FADE_OUT_SECS).abs() < f32::EPSILON);
}

#[test]
fn a_key_an_action_or_a_button_is_a_press() {
    let chord = crate::input::Chord::parse("z").unwrap();
    assert!(!pressed(&input(&[])));
    assert!(!pressed(&tick(5.0)));
    assert!(pressed(&input(&[Action::CursorDown])));
    assert!(pressed(&input(&[]).with_typing(vec![chord], vec![])));
    assert!(pressed(&input(&[]).with_pad(true)));
    // Typed characters come with their key's press; alone they are none.
    assert!(!pressed(&input(&[]).with_typing(vec![], vec!['z'])));
}

/// The menu for [`IDLE_SECS`], the fade for [`FADE_OUT_SECS`], then the
/// cinematic alone, at the song's time.
#[test]
fn left_alone_the_menu_fades_out_over_the_cinematic() {
    let mut c = ctx();
    let mut t = TitleScreen::new(&c);
    let first = t.view(&c);
    assert_eq!((&first.cinematic, first.menu_opacity), (&None, FULL));
    song_tick(&mut t, &mut c, 0.0, 0.0);
    song_tick(&mut t, &mut c, 14.9, 14.9);
    assert_eq!(playing(&t, &c), None);
    assert_eq!(t.view(&c).menu_opacity, FULL);
    // The fade starts dark, under the whole title.
    song_tick(&mut t, &mut c, 15.0, 0.1);
    assert_eq!(playing(&t, &c), Some((15_000, 0)));
    assert_eq!(t.view(&c).menu_opacity, FULL);
    assert!(t.view(&c).menu.is_some() && t.view(&c).help.is_some());
    song_tick(&mut t, &mut c, 15.25, 0.25);
    assert_eq!(playing(&t, &c), Some((15_250, 64)));
    assert_eq!(t.view(&c).menu_opacity, 191);
    song_tick(&mut t, &mut c, 15.75, 0.5);
    assert_eq!(playing(&t, &c), Some((15_750, 191)));
    assert_eq!(t.view(&c).menu_opacity, 64);
    assert!(t.view(&c).menu.is_some());
    // Alone: no menu, notice or help line. The debug hint stays.
    t.notice = Some("x".to_owned());
    song_tick(&mut t, &mut c, 16.0, 0.25);
    assert_eq!(playing(&t, &c), Some((16_000, FULL)));
    let view = t.view(&c);
    assert_eq!(
        (&view.menu, &view.help, &view.notice),
        (&None, &None, &None)
    );
    assert_eq!(view.menu_opacity, 0);
    assert!(view.debug_hint.is_some());
    // It follows the song, wherever that goes.
    song_tick(&mut t, &mut c, 80.5, 0.0);
    assert_eq!(playing(&t, &c), Some((80_500, FULL)));
    // The player it shows is the title cinematic, on this launch's backdrop.
    let want = CinematicPlayer::new(&c, TITLE_CINEMATIC, None).unwrap();
    let shown = t.view(&c).cinematic.unwrap().player;
    assert_eq!(*shown, want.with_backdrop(t.backdrop));
    assert!(t.backdrop.is_some());
    // Nothing was asked of the audio but the song, once.
    assert_eq!(
        c.audio.take(),
        [AudioRequest::PlayMusic {
            cue: TITLE_MUSIC.to_owned()
        }]
    );
}

/// At the logo shot the menu fades in over [`FADE_IN_SECS`] and the
/// sparks go on from the cinematic's; it stays until the song loops.
#[test]
fn the_menu_fades_in_on_the_logo_and_out_at_the_loop() {
    let mut c = ctx();
    c.idle_motion = IdleMotion::On;
    let mut t = in_cinematic(&mut c, 20.0);
    song_tick(&mut t, &mut c, 116.999, 0.1);
    assert_eq!(playing(&t, &c), Some((116_999, FULL)));
    song_tick(&mut t, &mut c, 117.0, 0.1);
    let view = t.view(&c);
    assert_eq!((&view.cinematic, view.menu_opacity), (&None, 0));
    assert!(view.menu.is_some() && view.help.is_some());
    assert_eq!(view.sparks, Some(117_000));
    song_tick(&mut t, &mut c, 117.125, 0.125);
    assert_eq!(t.view(&c).menu_opacity, 64);
    assert_eq!(t.view(&c).sparks, Some(117_125));
    song_tick(&mut t, &mut c, 117.5, 0.375);
    assert_eq!(t.view(&c).menu_opacity, FULL);
    // Far longer than 15 seconds: it waits for the loop.
    song_tick(&mut t, &mut c, 133.7, 40.0);
    assert_eq!(playing(&t, &c), None);
    assert_eq!(t.view(&c).menu_opacity, FULL);
    song_tick(&mut t, &mut c, 0.01, 0.05);
    assert_eq!(playing(&t, &c), Some((10, 0)));
    assert_eq!(t.phase, Phase::FadingOut { f: 0.0 });
}

/// The press that skips the cinematic plays the skip sound, stops the
/// song and does nothing else; the next one is the menu's.
#[test]
fn a_press_skips_the_cinematic_and_nothing_else() {
    let mut c = ctx();
    let mut t = in_cinematic(&mut c, 40.0);
    assert_eq!(outcome_in(&mut t, &mut c, &[Action::Confirm]), "None");
    assert_eq!(
        c.audio.take(),
        [
            AudioRequest::PlaySound {
                cue: TITLE_SKIP_SOUND.to_owned(),
                volume: 1.0
            },
            AudioRequest::StopMusic
        ]
    );
    assert_eq!(playing(&t, &c), None);
    assert_eq!(t.view(&c).menu_opacity, FULL);
    assert_eq!(t.menu.focus(), 0);
    // The time is the logo's, and runs on the frame clock from there
    // whatever the fading song's clock says.
    assert!((t.cinematic_time() - 117.0).abs() < 1e-3);
    song_tick(&mut t, &mut c, 40.3, 0.25);
    assert!((t.cinematic_time() - 117.25).abs() < 1e-3);
    // Cursor moves skip too, without moving or sounding like a menu.
    let mut t = in_cinematic(&mut c, 40.0);
    let moved = sounds_of(&mut t, &mut c, &[Action::CursorDown, Action::Confirm]);
    assert_eq!(moved, [TITLE_SKIP_SOUND]);
    assert_eq!(t.menu.focus(), 0);
    assert_eq!(
        outcome_in(&mut t, &mut c, &[Action::Confirm]),
        "Push(mode_select)"
    );
}

/// After a skip the song is asked for again when its silence would have
/// ended, and the menu, left alone, fades once the picture is past the
/// logo.
#[test]
fn the_song_starts_again_after_the_skips_silence() {
    let mut c = ctx();
    let mut t = in_cinematic(&mut c, 40.0);
    t.update(&mut c, &input(&[Action::Cancel]));
    c.audio.take();
    c.music_clock = None;
    t.update(&mut c, &tick(16.7));
    assert!(c.audio.take().is_empty());
    assert_eq!(playing(&t, &c), None);
    assert!((t.cinematic_time() - 133.7).abs() < 1e-3);
    // The end of the silence: the song, once, and the time at 0.
    t.update(&mut c, &tick(0.05));
    assert_eq!(
        c.audio.take(),
        [AudioRequest::PlayMusic {
            cue: TITLE_MUSIC.to_owned()
        }]
    );
    assert!(t.cinematic_time().abs() < f32::EPSILON);
    assert_eq!(t.phase, Phase::FadingOut { f: 0.0 });
    t.update(&mut c, &tick(0.05));
    assert!(c.audio.take().is_empty());
}

/// Any press with the menu up starts the 15 seconds again, and takes the
/// menu off the "until the loop" rule; one during the fade brings the
/// menu back and acts on it.
#[test]
fn a_press_restarts_the_wait_and_ends_the_fade() {
    use Action::CursorDown;
    let mut c = ctx();
    let mut t = TitleScreen::new(&c);
    song_tick(&mut t, &mut c, 14.0, 14.0);
    t.update(&mut c, &input(&[CursorDown]));
    assert_eq!(t.phase, Phase::MENU);
    assert_eq!(t.menu.focus(), 2);
    song_tick(&mut t, &mut c, 28.9, 14.9);
    assert_eq!(playing(&t, &c), None);
    song_tick(&mut t, &mut c, 29.5, 0.6);
    assert_eq!(playing(&t, &c), Some((29_500, 0)));
    song_tick(&mut t, &mut c, 30.0, 0.5);
    assert_eq!(playing(&t, &c), Some((30_000, 128)));
    // During the fade.
    t.update(&mut c, &input(&[CursorDown]));
    assert_eq!(t.phase, Phase::MENU);
    assert_eq!(t.menu.focus(), 3);
    assert_eq!(t.view(&c).menu_opacity, FULL);
    // On the logo: the 15 seconds, not the loop; and while they run out
    // on the logo, the menu stays.
    let mut t = in_cinematic(&mut c, 20.0);
    song_tick(&mut t, &mut c, 117.0, 0.1);
    song_tick(&mut t, &mut c, 117.5, 0.2);
    assert_eq!(t.view(&c).menu_opacity, 102);
    t.update(&mut c, &input(&[CursorDown]));
    assert_eq!(t.phase, Phase::MENU);
    song_tick(&mut t, &mut c, 133.0, 15.5);
    assert_eq!(playing(&t, &c), None);
    song_tick(&mut t, &mut c, 0.5, 1.0);
    assert_eq!(playing(&t, &c), Some((500, 0)));
    // Pressed again in the silence: the loop goes by under the menu.
    let mut t = in_cinematic(&mut c, 20.0);
    song_tick(&mut t, &mut c, 125.0, 0.1);
    t.update(&mut c, &input(&[CursorDown]));
    song_tick(&mut t, &mut c, 5.0, 13.7);
    assert_eq!(playing(&t, &c), None);
    song_tick(&mut t, &mut c, 6.3, 1.3);
    assert_eq!(playing(&t, &c), Some((6_300, 0)));
}

/// Back from the game the title starts over: the menu, the song asked
/// for, the cinematic's time at 0 until the song sounds.
#[test]
fn the_title_starts_over_when_it_is_shown_again() {
    let mut c = ctx();
    let mut t = in_cinematic(&mut c, 60.0);
    // Skip, then New Game: another screen is open over the title.
    t.update(&mut c, &input(&[Action::Confirm]));
    assert_eq!(
        outcome_in(&mut t, &mut c, &[Action::Confirm]),
        "Push(mode_select)"
    );
    c.audio.take();
    c.music_clock = None;
    assert_eq!(music_of(&mut t, &mut c, &[]), [TITLE_MUSIC]);
    assert_eq!(t.phase, Phase::MENU);
    assert!(t.cinematic_time().abs() < f32::EPSILON);
    assert!(!t.song.in_silence());
}

/// Without a title cinematic the title is its menu, as it always was.
#[test]
fn without_a_cinematic_the_menu_stays() {
    let mut c = ctx();
    c.content.cinematics.remove(TITLE_CINEMATIC);
    let mut t = TitleScreen::new(&c);
    for _ in 0..3 {
        song_tick(&mut t, &mut c, 50.0, 100.0);
        let view = t.view(&c);
        assert_eq!((&view.cinematic, view.menu_opacity), (&None, FULL));
        assert!(view.menu.is_some());
    }
    assert_eq!(
        outcome_in(&mut t, &mut c, &[Action::Confirm]),
        "Push(mode_select)"
    );
    // One that goes away under a playing title leaves the menu.
    let mut c = ctx();
    let mut t = in_cinematic(&mut c, 30.0);
    c.content.cinematics.remove(TITLE_CINEMATIC);
    t.refresh(&c);
    song_tick(&mut t, &mut c, 31.0, 1.0);
    assert_eq!(t.phase, Phase::MENU);
    assert!(t.view(&c).menu.is_some());
}

/// The cinematic is made again each time the title is shown, the options
/// included (they change the colours it is painted in).
#[test]
fn the_cinematic_is_made_again_when_the_title_is_shown_again() {
    use Action::{Confirm, CursorDown, CursorUp};
    let mut c = ctx();
    let mut t = TitleScreen::new(&c);
    assert!(t.player.is_none(), "made by the first update");
    t.update(&mut c, &input(&[]));
    let first = t.player.clone().unwrap();
    assert_eq!(first.timeline().length_ms(), 133_743);
    assert_eq!(first.timeline().last_start_ms(), 117_000);
    // Nothing changed under the title itself: the same one.
    t.update(&mut c, &input(&[]));
    assert!(Rc::ptr_eq(&first, t.player.as_ref().unwrap()));
    // Options, and back.
    assert_eq!(
        outcome_in(&mut t, &mut c, &[CursorDown, Confirm]),
        "Push(options)"
    );
    c.content
        .cinematics
        .get_mut(TITLE_CINEMATIC)
        .unwrap()
        .length_ms = 120_000;
    t.update(&mut c, &input(&[]));
    assert_eq!(t.player.as_ref().unwrap().timeline().length_ms(), 120_000);
    // New Game, and back.
    assert_eq!(
        outcome_in(&mut t, &mut c, &[CursorUp, Confirm]),
        "Push(mode_select)"
    );
    c.content
        .cinematics
        .get_mut(TITLE_CINEMATIC)
        .unwrap()
        .length_ms = 125_000;
    t.update(&mut c, &input(&[]));
    assert_eq!(t.player.as_ref().unwrap().timeline().length_ms(), 125_000);
    // `refreshed` makes it at once.
    assert!(TitleScreen::new(&c).refreshed(&c).player.is_some());
}

/// The cinematic's lead is the lead of the most recent save, looked up
/// each time the title is shown; with no save, the player's own default.
#[test]
fn the_cinematics_lead_follows_the_latest_save() {
    use crate::save::tests::save_of;
    use trpg_core::{LeadGender, LeadProfile};
    let mut c = ctx();
    let mut t = TitleScreen::new(&c);
    t.update(&mut c, &input(&[]));
    let nobody = CinematicPlayer::new(&c, TITLE_CINEMATIC, None).unwrap();
    assert_eq!(t.player.as_ref().unwrap().lead(), nobody.lead());
    let mara = save_of(&c, "Mara", LeadGender::Female);
    save::write(&mut c.storage, &save::slot_key(2), &mara).unwrap();
    // Not under the title itself; when it is shown again.
    t.update(&mut c, &input(&[]));
    assert_eq!(t.player.as_ref().unwrap().lead(), nobody.lead());
    t.update(&mut c, &input(&[Action::Confirm]));
    t.update(&mut c, &input(&[]));
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    assert_eq!(t.player.as_ref().unwrap().lead(), &lead);
    assert_ne!(&lead, nobody.lead());
}

/// Before the first press there is no cinematic, however long it takes.
#[test]
fn the_prompt_waits_without_a_cinematic() {
    let mut c = ctx();
    c.key_prompt = KeyPrompt::Waiting;
    let mut t = TitleScreen::new(&c);
    t.update(&mut c, &tick(100.0));
    let view = t.view(&c);
    assert_eq!((&view.cinematic, view.menu_opacity), (&None, FULL));
    assert!(view.prompt.is_some());
    assert!(c.audio.take().is_empty());
    // The press: the song and the menu; the 15 seconds start now.
    c.key_prompt = KeyPrompt::Pressed;
    assert_eq!(music_of(&mut t, &mut c, &[Action::Confirm]), [TITLE_MUSIC]);
    assert_eq!(t.phase, Phase::MENU);
    song_tick(&mut t, &mut c, 14.9, 14.9);
    assert_eq!(playing(&t, &c), None);
    song_tick(&mut t, &mut c, 15.0, 0.2);
    assert_eq!(playing(&t, &c), Some((15_000, 0)));
}

/// A frame that isn't a time, or goes backwards, moves no timer.
#[test]
fn a_frame_that_is_not_a_time_moves_nothing() {
    let mut c = ctx();
    let mut t = TitleScreen::new(&c);
    song_tick(&mut t, &mut c, 14.0, 14.0);
    let before = t.phase;
    for bad in [f32::NAN, f32::INFINITY, -5.0] {
        song_tick(&mut t, &mut c, 14.0, bad);
        assert_eq!(t.phase, before, "{bad}");
    }
    song_tick(&mut t, &mut c, 15.0, 1.0);
    song_tick(&mut t, &mut c, 15.5, 0.5);
    let fading = t.phase;
    assert_eq!(fading, Phase::FadingOut { f: 0.5 });
    for bad in [f32::NAN, f32::INFINITY, -5.0] {
        song_tick(&mut t, &mut c, 15.5, bad);
        assert_eq!(t.phase, fading, "{bad}");
    }
}
