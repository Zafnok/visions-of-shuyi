//! Scripted tests of the data's text in another language (ticket 0235),
//! through the real game: the test pack (`assets/lang/test/`) is English in
//! capitals for a few names, a tip and some lines of the test scene, so
//! what it translates shows in capitals and everything else in English.

use insta::assert_snapshot;
use trpg_content::LangCode;
use trpg_content::lang::TEST;
use trpg_core::{LeadGender, LeadProfile};
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;
use trpg_ui::screens::BattleScreen;
use trpg_ui::screens::battle::map_menu::MapEntry;
use trpg_ui::screens::battle::mode::{MenuEntry, Mode};

/// At the title in the test language, with the right-handed layout.
fn title() -> Harness {
    let mut h = Harness::with_layout(Layout::RightHanded);
    let Some(test) = LangCode::new(TEST) else {
        panic!("the test pack's code isn't a language code");
    };
    h.ctx_mut().lang = test;
    h
}

/// In Quick Battle in the test language, its banner closed.
fn quick_battle() -> Harness {
    let mut h = title();
    h.keys("Down f Left f f");
    assert_eq!(h.screens(), ["title", "battle"]);
    h
}

/// Whether the screen shows `text`.
fn shows(h: &Harness, text: &str) -> bool {
    h.snapshot().contains(text)
}

/// Plays the test scene (the debug menu's "Play test scene") for `lead`,
/// picking the first reply, and returns everything it showed: the screen
/// after each press.
fn test_scene(lead: LeadProfile) -> String {
    let mut h = title();
    h.ctx_mut().lead = lead;
    h.keys("F2 Down Down f");
    assert_eq!(h.screens(), ["title", "debug_menu", "dialogue"]);
    let mut shown = h.snapshot();
    for _ in 0..100 {
        h.keys("f");
        if h.top_screen() != "dialogue" {
            return shown;
        }
        shown.push_str(&h.snapshot());
    }
    panic!("the scene never ended");
}

#[test]
fn the_test_scene_opens_in_the_test_language() {
    let mut h = title();
    h.keys("F2 Down Down f f");
    assert_snapshot!(h.snapshot());
}

#[test]
fn a_scene_shows_the_packs_lines_and_english_for_the_rest() {
    let shown = test_scene(LeadProfile::new("Mara", LeadGender::Female));
    for text in [
        // The caption, narration and speech.
        "VILLAGE OF HETH, DUSK",
        "THE RAIN HAD NOT STOPPED FOR THREE DAYS.",
        "YOU'RE LATE.",
        // The lead's name fills `{lead}` in a translated line.
        "Mara! YOU CAME BACK FOR US.",
        // A reply, and one the pack doesn't have.
        "I SAID I WOULD. I KEEP MY WORD.",
        "Someone has to carry your arrows.",
        // Lines the pack doesn't have, a reaction among them.
        "Better late than... well.",
        "You do. It's why we follow you.",
        "Let's move.",
        // A speaker's name plate: the pack's name, and one it lacks.
        "TEST KNIGHT",
        "Test Lord",
    ] {
        assert!(shown.contains(text), "{text}");
    }
    for text in [
        "Village of Heth",
        "The rain had not",
        "You're late.",
        "Test Knight",
        // An orphaned entry (its line was reworded) shows nowhere.
        "IT IS ALWAYS ABOUT YOU.",
    ] {
        assert!(!shown.contains(text), "{text}");
    }
    assert!(shown.contains("It's always about you."));
}

/// The pack's line names the knight with a token English doesn't have:
/// it is filled with the pack's name for them.
#[test]
fn a_name_token_in_a_translated_line_uses_the_packs_name() {
    let shown = test_scene(LeadProfile::new("Mara", LeadGender::Female));
    assert!(shown.contains("WAS THAT ABOUT ME, TEST KNIGHT?"));
    assert!(!shown.contains("{n:"));
    assert!(!shown.contains("Was that about me?"));
}

#[test]
fn a_gendered_entry_shows_the_text_for_the_lead() {
    let hers = test_scene(LeadProfile::new("Mara", LeadGender::Female));
    assert!(hers.contains("Mara TIGHTENS HER GRIP ON THE SWORD. SHE WON'T LOSE ANYONE"));
    assert!(!hers.contains("HIS GRIP"));
    let his = test_scene(LeadProfile::new("Marek", LeadGender::Male));
    assert!(his.contains("Marek TIGHTENS HIS GRIP ON THE SWORD. HE WON'T LOSE ANYONE"));
    assert!(!his.contains("HER GRIP"));
}

/// Without the debug tools the test pack is not a language: English.
#[test]
fn the_test_pack_counts_only_with_the_debug_tools() {
    let mut h = title();
    h.keys("F2 Down Down f f");
    assert!(shows(&h, "THE RAIN HAD NOT STOPPED"));
    let mut h = title();
    h.keys("F2 Down Down f");
    h.ctx_mut().debug_tools = false;
    h.wait(0.1);
    // The scene was made in the test language; the next one isn't.
    let ctx = h.ctx_mut();
    assert!(ctx.words().is_english());
    assert_eq!(ctx.language(), trpg_ui::Language::default());
}

#[test]
fn the_info_screen_in_the_test_language() {
    let mut h = quick_battle();
    // The lord: a class the pack lacks, a sword it has.
    h.keys("e");
    assert_snapshot!(h.snapshot());
    assert!(shows(&h, "Test Lord"));
    assert!(shows(&h, "IRON SWORD"));
    assert!(!shows(&h, "Iron Sword"));
    // Weapons the pack lacks stay English.
    assert!(shows(&h, "Weapon ranks"));
}

#[test]
fn names_on_the_map_panel_are_the_packs() {
    let mut h = quick_battle();
    let panel = |h: &Harness| h.snapshot();
    // The lord's name isn't in the pack.
    assert!(panel(&h).contains("Test Lord"));
    // Every player unit in turn: the knight is the pack's TEST KNIGHT, a
    // GUARD.
    let mut seen = String::new();
    for _ in 0..8 {
        h.keys("s");
        seen.push_str(&panel(&h));
    }
    for text in ["TEST KNIGHT", "GUARD", "TEST ARCHER", "Test Mage"] {
        assert!(seen.contains(text), "{text}");
    }
    for text in ["Test Knight", "Guard ", "Test Archer"] {
        assert!(!seen.contains(text), "{text}");
    }
}

/// A generic unit is named after its class, and terrain by the pack: a
/// brigand is a BRIGAND of class BRIGAND, with an axe whose entry is
/// stale.
#[test]
fn a_generic_unit_and_terrain_are_named_by_the_pack() {
    let mut h = quick_battle();
    let battle = h.battle().unwrap();
    let state = battle.state();
    let brigand = state
        .units()
        .iter()
        .find(|u| u.class.0 == "brigand")
        .map(|u| (u.pos, u.name.clone()))
        .unwrap();
    // The battle itself keeps English: only what is shown is translated.
    assert_eq!(brigand.1, "Brigand");
    let forest = state
        .terrain()
        .terrains
        .iter()
        .position(|t| t.name == "Forest");
    let tiles = &state.map().tiles;
    let forest_at = (0..i32::from(tiles.height()))
        .flat_map(|y| (0..i32::from(tiles.width())).map(move |x| trpg_core::Pos::new(x, y)))
        .find(|&p| tiles.get(p).map(|t| usize::from(t.0)) == forest);
    let cursor = h.cursor_tile().unwrap();
    let walk = |h: &mut Harness, from: trpg_core::Pos, to: trpg_core::Pos| {
        let (dx, dy) = (to.x - from.x, to.y - from.y);
        let step = |h: &mut Harness, n: i32, plus: &str, minus: &str| {
            for _ in 0..n.abs() {
                h.keys(if n > 0 { plus } else { minus });
            }
        };
        step(h, dx, "Right", "Left");
        step(h, dy, "Down", "Up");
    };
    walk(&mut h, cursor, brigand.0);
    assert_eq!(h.cursor_tile(), Some(brigand.0));
    assert!(shows(&h, "BRIGAND"));
    assert!(!shows(&h, "Brigand"));
    h.keys("e");
    assert!(shows(&h, "Iron Axe"));
    assert!(!shows(&h, "OLD AXE"));
    h.keys("d");
    if let Some(forest_at) = forest_at {
        walk(&mut h, brigand.0, forest_at);
        assert!(shows(&h, "FOREST"));
        assert!(!shows(&h, "Forest"));
    }
}

#[test]
fn a_tip_in_the_test_language() {
    let mut h = title();
    h.with_tips().keys("Down f Left f f");
    assert_snapshot!(h.snapshot());
    assert!(shows(&h, "YOUR MOVE"));
    // Key names are still the player's.
    assert!(shows(&h, "STEER THE CURSOR WITH arrows. PRESS f ON ONE OF"));
    assert!(!shows(&h, "Your move"));
}

/// Whether the action menu is open with `Equip` focused.
fn on_equip(h: &Harness) -> bool {
    match h.battle().map(BattleScreen::mode) {
        Some(Mode::ActionMenu { menu, entries, .. }) => {
            entries.get(menu.focus()) == Some(&MenuEntry::Equip)
        }
        _ => false,
    }
}

/// The lord's Equip list: painted in the player's language, while the
/// battle plays the same list as in English.
#[test]
fn a_weapon_list_is_painted_in_the_test_language() {
    let equip_list = |mut h: Harness| {
        // Select the lord, walk to the fort, then go to Equip.
        h.keys("f Right Right f").wait(0.5);
        for _ in 0..5 {
            if !on_equip(&h) {
                h.keys("Down");
            }
        }
        assert!(on_equip(&h));
        h.keys("f");
        h
    };
    let mut english = Harness::with_layout(Layout::RightHanded);
    english.keys("Down f Left f f");
    let english = equip_list(english);
    let test = equip_list(quick_battle());
    assert!(shows(&english, "* Iron Sword "));
    assert!(shows(&english, "Steel Sword"));
    assert!(shows(&test, "* IRON SWORD "));
    assert!(!shows(&test, "Iron Sword"));
    // A weapon the pack lacks.
    assert!(shows(&test, "Steel Sword"));
    // What is played is the same in both: the list and its focus.
    let mode = |h: &Harness| h.battle().map(|b| b.mode().clone());
    assert!(matches!(mode(&test), Some(Mode::EquipMenu { .. })));
    assert_eq!(mode(&test), mode(&english));
}

/// A fight: the weapon list, the forecast with its arts, the combat's
/// fighters and the rewind list all name things in the player's language.
/// The combat is made by a command applied between frames, in the
/// language of the frame before.
#[test]
fn a_fight_names_everything_in_the_test_language() {
    let mut h = quick_battle();
    // The lord to (6, 4), beside the brigand at (7, 4); Attack.
    h.keys("f Right Right Right Up f").wait(0.5);
    h.keys("f");
    assert!(shows(&h, "IRON SWORD "));
    assert!(shows(&h, "Steel Sword"));
    assert!(!shows(&h, "Iron Sword"));
    // The forecast, and the lord's arts beside it.
    h.keys("f");
    assert!(shows(&h, "Test Lord     BRIGAND"));
    assert!(shows(&h, "IRON SWORD"));
    assert!(shows(&h, "GUARD BREAK"));
    assert!(shows(&h, "Flowing Cut"));
    assert!(!shows(&h, "Guard Break"));
    // The combat.
    h.keys("f");
    assert!(shows(&h, "BRIGAND"));
    assert!(!shows(&h, "Brigand"));
    h.keys("d f");
    // The rewind list.
    h.keys("r");
    assert!(shows(&h, "attacked BRIGAND"));
    assert!(!shows(&h, "Brigand"));
}

#[test]
fn the_unit_list_is_painted_in_the_test_language() {
    let mut h = quick_battle();
    h.keys("d");
    let on_units = |h: &Harness| match h.battle().map(BattleScreen::mode) {
        Some(Mode::MapMenu { menu, entries }) => {
            entries.get(menu.focus()) == Some(&MapEntry::Units)
        }
        _ => false,
    };
    for _ in 0..8 {
        if !on_units(&h) {
            h.keys("Down");
        }
    }
    assert!(on_units(&h));
    h.keys("f");
    assert!(matches!(
        h.battle().map(BattleScreen::mode),
        Some(Mode::UnitList { .. })
    ));
    assert!(shows(&h, "TEST KNIGHT "));
    assert!(shows(&h, "TEST ARCHER "));
    assert!(shows(&h, "Test Lord "));
    assert!(!shows(&h, "Test Knight"));
}

/// A scene over the battle map (the debug menu's overlay) is in the
/// player's language too.
#[test]
fn a_scene_over_the_map_is_in_the_test_language() {
    let mut h = quick_battle();
    h.keys("F2 Down Down Down f");
    assert_eq!(h.screens(), ["title", "battle", "dialogue"]);
    h.keys("f");
    assert!(shows(&h, "VILLAGE OF HETH, DUSK"));
    assert!(shows(&h, "THE RAIN HAD NOT STOPPED FOR THREE DAYS."));
}
