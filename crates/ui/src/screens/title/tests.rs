use super::*;
use crate::audio::{AudioRequest, pick_from_pool};
use crate::color::UiColor;
use crate::glyph_buffer::Cell;
use crate::input::Action;
use crate::screen::tests::ctx;
use Item::{Continue, Credits, LoadGame, NewGame, Options, QuickBattle, Quit};

/// The pool the Quick Battle's file names (`assets/battles/quick.ron`).
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
    use crate::console::{CONSOLE_H, CONSOLE_W};
    let c = ctx();
    let mut naming = crate::screens::LeadSelectScreen::new();
    let typing = FrameInput::new(vec![Action::CursorDown], 0.0, vec![]);
    naming.update(&mut ctx(), &typing);
    naming.update(&mut ctx(), &input(&[Action::Confirm]));
    let screens: [&dyn Screen; 8] = [
        &TitleScreen::new(&ctx()),
        &TitleScreen::with_quick_battle(&ctx()),
        &crate::screens::ModeSelectScreen::new(),
        &crate::screens::LeadSelectScreen::new(),
        &naming,
        &crate::screens::GameOverScreen::new(),
        &crate::screens::ToBeContinuedScreen,
        &CreditsScreen::new(&c),
    ];
    for screen in screens {
        let stale = Cell::new(
            'x',
            c.palette.get(UiColor::Enemy),
            c.palette.get(UiColor::Enemy),
        );
        let mut buf = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, stale);
        screen.draw(&c, &mut buf);
        let left = (0..i32::from(CONSOLE_H))
            .flat_map(|y| (0..i32::from(CONSOLE_W)).map(move |x| (x, y)))
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
    assert_eq!(view.title, "Visions of Shuyi");
    assert_eq!(view.subtitle, c.text("title.subtitle"));
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
    assert_eq!(view.prompt.as_deref(), Some("Press any key"));
    assert_eq!(
        (&view.menu, &view.help, &view.notice),
        (&None, &None, &None)
    );
    assert_eq!(view.title, "Visions of Shuyi");
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
        Some("PRESS ANY KEY")
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
    ];
    for (file, source) in sources {
        for name in ["UiColor", "Rect", "BoxStyle", "Cell"] {
            assert!(!names_word(source, name), "{file} names `{name}`");
        }
        // Their `draw` is one call to the skin.
        assert!(
            source.contains("glyph::paint(ctx, &self.view(ctx), buf);"),
            "{file}'s draw doesn't paint its view"
        );
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
