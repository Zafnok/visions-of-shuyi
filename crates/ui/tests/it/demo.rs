//! The Chapter 1 demo (ticket 0914, `docs/design/demo.md`), played against
//! the demo's own bundle (`trpg_content::bundle::demo_layers`): what its
//! title offers, Chapter 1 from New Game to its win, both victory scenes,
//! no "Save your progress?", "Thanks for playing, {lead}" and "Export
//! save?". *Yes* offers one file, which the full game imports and goes on
//! to Chapter 2 from; *No* offers none. Both go back to the title.

use insta::assert_snapshot;
use shuyi_ui::{FileDrops, StoragePlace};
use trpg_content::bundle::{self, Bundle};
use trpg_content::new_campaign;
use trpg_core::{GameMode, SaveFile};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::save::{self, Slot, slot_key};
use trpg_ui::screens::{DemoEndScreen, DialogueScreen, SavePromptScreen, TitleScreen};
use trpg_ui::{Ctx, ShuyiCtx};

use crate::ch01_winnable::{chapter_1_fought, lead};

/// The demo's bundle, as the demo edition embeds it.
fn demo() -> &'static Bundle {
    Box::leak(Box::new(Bundle::new(bundle::demo_layers())))
}

/// The demo launched with a layout picked, as a player has it: no debug
/// tools.
fn launch() -> Harness {
    Harness::with_layout_and(RIGHT_HANDED, |ctx| ctx.debug_tools = false).with_shipped_story()
}

/// The labels of the title's menu.
fn title_menu(h: &Harness) -> Vec<String> {
    let title = h.game().screen::<TitleScreen>();
    let view = title
        .unwrap_or_else(|| panic!("no title"))
        .view(h.game().ctx());
    let menu = view.menu.unwrap_or_else(|| panic!("no menu"));
    let label = |i: &trpg_ui::widgets::MenuItemView| h.game().ctx().label(&i.label);
    menu.items.iter().map(label).collect()
}

/// From the battle's win on to the demo's end card: returns the scenes
/// played on the way, in order.
fn on_to_the_end_card(h: &mut Harness) -> Vec<String> {
    let mut scenes: Vec<String> = Vec::new();
    for _ in 0..60 {
        match h.top_screen() {
            "dialogue" => {
                let screen = h.game().screen::<DialogueScreen>();
                let id = screen.map(|d| d.player().scene_id().to_owned());
                if let Some(id) = id
                    && scenes.last() != Some(&id)
                {
                    scenes.push(id);
                }
                h.keys("d f");
            }
            "demo_end" => break,
            "save_prompt" => panic!("the demo asks \"Save your progress?\""),
            // The last fight plays out; banners and the results take a
            // press.
            _ => {
                h.wait(0.5).keys("f");
            }
        }
    }
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "demo_end"],
        "{}",
        h.snapshot()
    );
    scenes
}

/// The demo played from New Game to its end card, its answer to "Export
/// save?" given (`yes`), back at the title: the files it offered.
fn play_the_demo(yes: bool) -> Vec<(String, String)> {
    bundle::reading(demo(), || {
        let mut h = launch();
        assert!(h.game().ctx().game.demo);
        let ctx = h.game().ctx();
        let shown = title_menu(&h);
        for gone in ["title.load_game", "title.quick_battle"] {
            assert!(!shown.iter().any(|l| l == ctx.text(gone)), "{shown:?}");
        }
        assert_eq!(shown[0], ctx.text("title.new_game"));
        if yes {
            assert_snapshot!("demo_title", h.snapshot());
        }
        chapter_1_fought(&mut h);
        let scenes = on_to_the_end_card(&mut h);
        assert!(
            scenes.ends_with(&["ch01_victory".to_owned(), "ch01_tbc".to_owned()]),
            "both victory scenes: {scenes:?}"
        );
        let ctx = h.game().ctx();
        let card = h.game().screen::<DemoEndScreen>();
        let card = card.unwrap_or_else(|| panic!("no card")).view(ctx);
        let name = &h
            .flow()
            .and_then(|f| f.campaign())
            .map(|c| c.lead.name.clone());
        assert_eq!(name.as_deref(), Some(lead().name.as_str()));
        assert_eq!(card.heading, format!("Thanks for playing, {}", lead().name));
        if yes {
            assert_snapshot!("demo_end_card", h.snapshot());
        }
        h.keys("f");
        assert_eq!(h.top_screen(), SavePromptScreen::EXPORT_DEMO_NAME);
        if yes {
            assert_snapshot!("demo_export_question", h.snapshot());
        }
        assert_eq!(h.offered(), [] as [(String, String); 0]);
        h.keys(if yes { "f" } else { "Down f" });
        assert_eq!(h.screens(), ["title"], "{}", h.snapshot());
        h.offered().to_vec()
    })
}

/// A desktop whose dropped files the game hears, with a save in slot 2 so
/// Load Game opens.
fn full_game_on_a_desktop(ctx: &mut Ctx) {
    ctx.platform.storage_place = StoragePlace::Folder("C:/saves".to_owned());
    ctx.platform.file_drops = FileDrops::Heard;
    let campaign = new_campaign(&ctx.game.content, GameMode::Classic, lead());
    let file = SaveFile::chapter_cleared(campaign);
    save::write(&mut ctx.storage, &slot_key(2), &file).unwrap_or_else(|e| panic!("{e}"));
}

/// Acceptance: the demo from New Game to its end, *Yes*: one file, a
/// save the full game imports (0873) and loads into Chapter 2.
#[test]
fn the_demo_ends_after_chapter_1_and_its_save_goes_on_in_the_full_game() {
    let offered = play_the_demo(true);
    let [(name, text)] = &offered[..] else {
        panic!("offered {offered:?}");
    };
    // Named as Load Game names slot 1 of this profile, "Player 1".
    assert_eq!(name, "visions-of-shuyi-player-1-1-slot-01.ron");
    for later in ["ch02", "Chapter 2"] {
        assert!(!text.contains(later), "the file names {later}: {text}");
    }

    // The full game: its import takes the file into the first empty slot,
    // and loading it starts Chapter 2.
    let mut h = Harness::with_layout_and(RIGHT_HANDED, full_game_on_a_desktop);
    assert!(!h.game().ctx().game.demo);
    h.wait(0.1).keys("Down f");
    assert_eq!(h.top_screen(), "save_slots", "{}", h.snapshot());
    h.keys("Space");
    assert_eq!(h.top_screen(), "save_import", "{}", h.snapshot());
    h.drop_file(name, text.clone().into_bytes()).wait(0.02);
    h.keys("Left f");
    assert_eq!(h.top_screen(), "save_slots", "{}", h.snapshot());
    let ctx = h.game().ctx();
    let slot = save::slot(&ctx.storage, &ctx.game.content, ctx.words(), 1);
    assert!(matches!(slot, Slot::Saved(_)), "{slot:?}");
    h.keys("f");
    let chapter = h.flow().and_then(|f| f.chapter()).map(|c| c.id.clone());
    assert_eq!(chapter.as_deref(), Some("ch02"), "{}", h.snapshot());
}

/// Acceptance: *No* goes back to the title with no file.
#[test]
fn no_to_export_save_offers_nothing() {
    assert_eq!(play_the_demo(false), [] as [(String, String); 0]);
}

/// Acceptance: the battle's suspend save the demo makes loads in the demo.
#[test]
fn the_demos_suspend_save_continues_in_the_demo() {
    bundle::reading(demo(), || {
        let mut h = launch();
        h.keys("f f f Up f");
        crate::playthrough::skip_scenes(&mut h);
        assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
        // The notes, the phase's banner and the first turn's scene.
        h.keys("f").wait(3.0).keys("f");
        crate::playthrough::skip_scenes(&mut h);
        // The map menu's `Suspend`, and Confirm on its question.
        h.keys("d Down Down Down Down f f");
        assert_eq!(h.screens(), ["title"], "{}", h.snapshot());
        let ctx = h.game().ctx();
        assert_eq!(title_menu(&h)[0], ctx.text("title.continue"));
        h.keys("f");
        assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
        let chapter = h.flow().and_then(|f| f.chapter()).map(|c| c.id.clone());
        assert_eq!(chapter.as_deref(), Some("ch01"));
    });
}
