use std::collections::BTreeMap;

use insta::assert_snapshot;
use shuyi_assets::lang::LangCode;
use trpg_content::Lang;

use super::*;
use crate::harness::Harness;
use crate::input::LEFT_HANDED;
use crate::widgets::menu::text_of;

fn code(tag: &str) -> LangCode {
    LangCode::parse(tag).unwrap()
}

/// English and three packs a player could have: Chinese and Japanese made
/// by a machine, each with its own note as the design doc words it, and
/// French made by a person, with none. Not the game's translations
/// (0719, 0726): what the list does with such packs.
fn with_packs(ctx: &mut Ctx) {
    let english = trpg_content::lang::english_ui().clone();
    let pack = |name: &str, made_by: &str, note: Option<&str>| {
        let info = format!("(name: {name:?}, made_by: {made_by})");
        let ui = note.map_or_else(
            || "[]".to_owned(),
            |note| {
                let source = &english[MACHINE_NOTE];
                format!("[(key: {MACHINE_NOTE:?}, source: {source:?}, text: {note:?})]")
            },
        );
        let made =
            trpg_content::lang::pack_from_sources(("lang.ron", &info), ("ui.ron", &ui), &english);
        made.unwrap_or_else(|errors| panic!("{errors:?}"))
    };
    let packs = BTreeMap::from([
        (
            code("fr"),
            pack("Français", r#"Human("A. Translator")"#, None),
        ),
        (code("ja"), pack("日本語", "Machine", Some("(機械翻訳)"))),
        (
            code("zh-Hans"),
            pack("简体中文", "Machine", Some("(机器翻译，母语者审校)")),
        ),
    ]);
    ctx.game.content.lang = std::sync::Arc::new(Lang::new(english, packs));
}

/// A first launch with [`with_packs`]'s languages, the system's languages
/// `system`, debug tools off (so no test pack).
fn first_launch(system: &[&str]) -> Harness {
    let system: Vec<LangCode> = system.iter().map(|tag| code(tag)).collect();
    Harness::first_launch_and(|ctx| {
        with_packs(ctx);
        ctx.debug_tools = false;
        ctx.set_system_languages(system);
    })
}

/// The list's view in `h`.
fn list(h: &Harness) -> SavePromptView {
    let screen = h.screen::<LanguageListScreen>().expect("the list is open");
    screen.view(h.game().ctx())
}

fn labels(view: &SavePromptView) -> Vec<String> {
    view.menu
        .items
        .iter()
        .map(|i| text_of(&i.label).to_owned())
        .collect()
}

fn focused(view: &SavePromptView) -> String {
    text_of(&view.menu.focused().unwrap().label).to_owned()
}

/// The design doc: each language by its own name, a machine translation
/// with its note, in its own language; a person's translation with none.
#[test]
fn every_language_by_its_own_name_with_the_machine_note() {
    let h = first_launch(&[]);
    assert_eq!(h.screens(), ["title", "layout_picker", "language_list"]);
    let view = list(&h);
    assert_eq!(view.question, "Language");
    assert_eq!(
        labels(&view),
        [
            "English",
            "Français",
            "日本語 (機械翻訳)",
            "简体中文 (机器翻译，母语者审校)",
        ]
    );
}

/// The computer's language is highlighted when the game has a pack for
/// it (Chinese Windows: Chinese), English otherwise; one press keeps it.
#[test]
fn the_systems_language_is_highlighted_english_otherwise() {
    let h = first_launch(&["zh-CN"]);
    assert_eq!(focused(&list(&h)), "简体中文 (机器翻译，母语者审校)");
    let h = first_launch(&["de-DE", "ko"]);
    assert_eq!(focused(&list(&h)), "English");
    let mut h = first_launch(&["ja-JP"]);
    h.keys("f");
    assert_eq!(h.game().ctx().settings().language, Some(code("ja")));
    assert_eq!(h.screens(), ["title", "layout_picker"]);
}

/// Ticket 0825's order: the language, then the layout; once both are
/// picked neither is asked again, and a launch with the language picked
/// but no layout asks only the layout, on the first key.
#[test]
fn the_language_is_asked_once_before_the_layout() {
    let mut h = first_launch(&[]);
    h.keys("s f");
    assert_eq!(h.game().ctx().settings().language, Some(code("fr")));
    assert_eq!(h.screens(), ["title", "layout_picker"]);
    // Closed before the layout: the next launch skips the list.
    let mut h = Harness::with_storage(h.into_storage());
    assert_eq!(h.screens(), ["title"]);
    h.keys("f");
    assert_eq!(h.screens(), ["title", "layout_picker"]);
    h.keys("s f");
    assert_eq!(h.game().ctx().layout(), Some(LEFT_HANDED));
    assert_eq!(h.screens(), ["title"]);
    let h = Harness::with_storage(h.into_storage());
    assert_eq!(h.screens(), ["title"]);
    assert_eq!(h.game().ctx().settings().language, Some(code("fr")));
    assert_eq!(h.game().ctx().lang, code("fr"));
}

/// With no keyboard (the Playdate) the language is asked too, and no
/// layout after it.
#[test]
fn without_a_keyboard_the_language_is_asked_and_no_layout() {
    let mut h = first_launch_without_a_keyboard();
    assert_eq!(h.screens(), ["title", "language_list"]);
    h.act(crate::input::Action::Confirm);
    assert_eq!(h.screens(), ["title"]);
    assert_eq!(h.game().ctx().settings().language, Some(code("en")));
}

fn first_launch_without_a_keyboard() -> Harness {
    Harness::first_launch_and(|ctx| {
        with_packs(ctx);
        ctx.platform.keyboard = crate::screen::Keyboard::Absent;
    })
}

/// Old saved settings (with a layout, from before the list) load with
/// English and no list: Options changes it.
#[test]
fn a_player_from_before_the_list_is_not_asked() {
    let h = Harness::with_layout_and(LEFT_HANDED, with_packs);
    assert_eq!(h.screens(), ["title"]);
    assert_eq!(h.game().ctx().settings().language, None);
    assert_eq!(h.game().ctx().lang, code("en"));
}

/// With only English and the test pack (a developer's), there is nothing
/// to pick: no list, the title as before.
#[test]
fn with_english_alone_nothing_is_asked() {
    let h = Harness::new();
    assert_eq!(h.screens(), ["title"]);
    assert!(!asks(h.game().ctx()));
}

/// The list's look, with the languages a debug build has: English, and
/// the test pack (machine-made) with its note. (The fixture's Chinese and
/// Japanese aren't drawn: the atlas has only the glyphs the game's packs
/// use, 0236.)
#[test]
fn language_list_snapshot() {
    let h = Harness::with_screen(Box::new(LanguageListScreen::new(
        &crate::screen::tests::ctx(),
    )));
    assert_snapshot!(h.snapshot());
}

#[test]
fn language_list_declares_its_actions() {
    let h = first_launch(&[]);
    let screen = h.screen::<LanguageListScreen>().unwrap();
    crate::screens::actions::assert_declares(screen, crate::screens::actions::LANGUAGE_LIST);
}
