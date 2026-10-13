//! Scripted tests of the battle screen through the real game (ADR-0007
//! layer 4), reached from the title screen's debug Quick Battle item.

use insta::assert_snapshot;
use trpg_content::{FontAtlasDef, Theme};
use trpg_core::{Faction, Pos};
use trpg_ui::audio::AudioRequest;
use trpg_ui::harness::{FRAME_DT, Harness};
use trpg_ui::input::Layout;
use trpg_ui::map_view::RangeKind;

/// At the title with the right-handed layout, then Quick Battle, still
/// under turn 1's `PLAYER PHASE` banner.
fn quick_battle_banner() -> Harness {
    let mut h = Harness::with_layout(Layout::RightHanded);
    // Quick Battle, then Preparations: Left wraps to `Fight!`.
    h.keys("Down f Left f");
    assert_eq!(h.screens(), ["title", "battle"]);
    h
}

/// [`quick_battle_banner`], and the banner closed.
fn quick_battle() -> Harness {
    let mut h = quick_battle_banner();
    h.keys("f");
    assert!(!shows(&h, "PHASE"));
    h
}

/// A battle opens on turn 1's `PLAYER PHASE` banner (ticket 0435): only
/// Confirm does anything while it is up, and it closes by itself after a
/// second.
#[test]
fn quick_battle_opens_on_the_player_phase_banner() {
    let mut h = quick_battle_banner();
    assert!(shows(&h, "PLAYER PHASE"));
    assert!(shows(&h, "Turn 1"));
    assert_eq!(help(&h), "f skip");
    // Other keys do nothing: no menu opens, and the cursor stays on the
    // lord at (3, 5).
    h.keys("Right d Space");
    assert!(shows(&h, "PLAYER PHASE"));
    let browsing = "f select · e info · s next unit · r rewind · d menu · Space end turn";
    // Confirm closes it, and does nothing else: the lord isn't selected.
    h.keys("f");
    assert!(!shows(&h, "PHASE"));
    assert_eq!(help(&h), browsing);
    assert_eq!(h.cursor_tile(), Some(LORD));
    // Left alone, it is gone after a second.
    let mut h = quick_battle_banner();
    h.wait(0.5);
    assert!(shows(&h, "PLAYER PHASE"));
    h.wait(0.6);
    assert!(!shows(&h, "PHASE"));
    assert_eq!(help(&h), browsing);
    assert_eq!(h.cursor_tile(), Some(LORD));
}

/// The Quick Battle screen at the start of the battle, its `PLAYER PHASE`
/// banner just closed (the cursor a little into its pulse): `test_small.map`
/// centred in the viewport, four ready player units and four enemies, all
/// at full HP, the cursor on the lord, the side panel showing the lord and
/// its tile, and the help line.
#[test]
fn quick_battle_renders() {
    let h = quick_battle();
    assert_eq!(h.screens(), ["title", "battle"]);
    assert_snapshot!(h.snapshot());
}

/// The Quick Battle in each colour theme (ticket 0806): the same frame,
/// with every colour still a palette name, and cells in the theme's
/// colours.
#[test]
fn quick_battle_in_every_theme() {
    for theme in Theme::ALL {
        let mut h = Harness::with_layout(Layout::RightHanded);
        h.ctx_mut().change_settings(|s| s.theme = theme).unwrap();
        h.keys("Down f Left f f");
        assert_eq!(h.screens(), ["title", "battle"]);
        let snapshot = h.snapshot();
        let legend = snapshot.split("--- overlays ---").next().unwrap_or("");
        assert!(!legend.contains('#'), "{theme:?} has a colour with no name");
        // The terrain is painted in this theme's colours.
        let def = &h.game().ctx().content.themes[&theme];
        let buf = h.game().buffer();
        let bg = |x, y| buf.get(x, y).map(|c| c.bg.to_hex());
        let grass = trpg_ui::color::Rgb::from(def.get("grass_bg").unwrap()).to_hex();
        assert!(
            (0..32).any(|y| (0..100).any(|x| bg(x, y) == Some(grass.clone()))),
            "{theme:?} shows no grass_bg"
        );
        insta::with_settings!({snapshot_suffix => format!("{theme:?}")}, {
            assert_snapshot!(snapshot);
        });
    }
}

/// What the Quick Battle's map shows at its start, as data (ADR-0038): the
/// terrain, the units and the cursor on the lord. Moving the cursor one
/// tile changes its line and nothing else.
#[test]
fn quick_battle_map_text() {
    let mut h = quick_battle();
    let start = h.map_text();
    assert_snapshot!(start);
    h.keys("Right");
    let moved = h.map_text();
    assert_eq!(start.lines().count(), moved.lines().count());
    let changed: Vec<(&str, &str)> = start
        .lines()
        .zip(moved.lines())
        .filter(|(a, b)| a != b)
        .collect();
    assert_eq!(changed.len(), 1, "{changed:?}");
    let (was, is) = changed[0];
    assert!(was.starts_with("cursor (3,5) corners "), "{was}");
    assert!(is.starts_with("cursor (4,5) corners "), "{is}");
}

#[test]
fn back_opens_the_map_menu() {
    let mut h = quick_battle();
    // With the lord selected, back only drops the selection.
    h.keys("f Up d");
    assert_eq!(
        help(&h),
        "f select · e info · s next unit · r rewind · d menu · Space end turn"
    );
    // Browsing: the map menu; back closes it.
    h.keys("d");
    assert_eq!(help(&h), "arrows choose · f confirm · d back");
    h.keys("d");
    assert_eq!(h.screens(), ["title", "battle"]);
    assert_eq!(
        help(&h),
        "f select · e info · s next unit · r rewind · d menu · Space end turn"
    );
}

/// The text of row `y`, trimmed.
fn row(h: &Harness, y: i32) -> String {
    let buf = h.game().buffer();
    (0..100)
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect::<String>()
        .trim()
        .to_owned()
}

/// Whether some row of the screen contains `text`.
fn shows(h: &Harness, text: &str) -> bool {
    (0..32).any(|y| row(h, y).contains(text))
}

/// Lets frames pass with no keys until `text` shows (the enemies act one
/// by one before the player's next phase).
///
/// # Panics
///
/// If it doesn't show within `max` seconds.
fn wait_for(h: &mut Harness, text: &str, max: f32) {
    let mut waited = 0.0;
    while !shows(h, text) {
        assert!(waited < max, "no {text:?} after {max} s:\n{}", h.snapshot());
        h.wait(FRAME_DT);
        waited += FRAME_DT;
    }
}

/// Each player unit in turn (the one under the cursor, then the next ready
/// one) selected, kept where it stands, and told to Wait (no enemy is in
/// reach, so the menu opens on Wait, the mage's too).
fn wait_all(h: &mut Harness, units: usize) {
    for i in 0..units {
        if i > 0 {
            h.keys("s");
        }
        h.keys("f f");
        assert_eq!(help(h), "arrows choose · f confirm · d back");
        h.keys("f");
    }
}

#[test]
fn a_full_turn_of_waits_ends_with_auto_end_and_starts_turn_two() {
    let mut h = quick_battle();
    // Auto-end on (off by default): a message says so.
    h.keys("Shift+Space");
    assert!(row(&h, 30).starts_with("Auto-end: ON"), "{}", row(&h, 30));
    wait_all(&mut h, 4);
    // The enemy phase's banner, then the enemies act, then the player's
    // turn 2.
    assert!(shows(&h, "ENEMY PHASE"), "{}", h.snapshot());
    assert!(shows(&h, "Turn 1"));
    h.wait(1.1);
    assert!(!shows(&h, "PHASE"));
    wait_for(&mut h, "PLAYER PHASE", 30.0);
    assert!(shows(&h, "Turn 2"));
    h.wait(1.1);
    assert!(!shows(&h, "PHASE"));
    // Everyone is ready again, the lord selectable.
    let units = h.map_scene().unwrap().units;
    let ready = units
        .iter()
        .filter(|u| u.faction == Faction::Player && !u.acted);
    assert_eq!(ready.count(), 4);
    assert_eq!(unit(&h, 3, 5), Some(shown("Lo", false)));
    assert_eq!(
        help(&h),
        "f select · e info · s next unit · r rewind · d menu · Space end turn"
    );
}

#[test]
fn a_full_turn_of_waits_then_space_ends_the_turn() {
    let mut h = quick_battle();
    // Auto-end is off by default (ticket 0420).
    assert!(row(&h, 30).ends_with("Shift+Space auto-end: OFF"));
    wait_all(&mut h, 4);
    assert!(!shows(&h, "PHASE"));
    // Nobody ready: Space ends the turn without asking.
    h.keys("Space");
    assert!(shows(&h, "ENEMY PHASE"));
    // Confirm skips each banner.
    h.keys("f");
    assert!(!shows(&h, "ENEMY PHASE"));
    wait_for(&mut h, "PLAYER PHASE", 30.0);
    assert!(shows(&h, "Turn 2"));
    h.keys("f");
    assert!(!shows(&h, "PHASE"));
    assert_eq!(unit(&h, 3, 5), Some(shown("Lo", false)));
}

#[test]
fn space_twice_ends_the_turn_with_units_ready() {
    let mut h = quick_battle();
    wait_all(&mut h, 1);
    h.keys("Space");
    assert!(shows(&h, "End turn with 3 units ready?"));
    assert!(shows(&h, "f yes / d no"));
    // No: back to the map.
    h.keys("d");
    assert!(!shows(&h, "units ready?"));
    // Double-tap Space.
    h.keys("Space Space");
    assert!(shows(&h, "ENEMY PHASE"));
    h.keys("f");
    wait_for(&mut h, "PLAYER PHASE", 30.0);
    h.keys("f");
    assert!(!shows(&h, "PHASE"));
    // Turn 2: the objective says so.
    h.keys("d Down f");
    assert!(shows(&h, "Rout the enemy"));
    assert!(shows(&h, "Turn 2"));
}

#[test]
fn info_and_danger_zone_keys() {
    let mut h = quick_battle();
    h.keys("e");
    assert!(shows(&h, "Weapon ranks"));
    assert!(shows(&h, "Test Lord"));
    h.keys("s");
    assert!(shows(&h, "Test Mage"));
    h.keys("d");
    assert!(!shows(&h, "Weapon ranks"));
    assert!(row(&h, 30).ends_with("w danger zone: OFF · Shift+Space auto-end: OFF"));
    h.keys("w");
    assert!(row(&h, 30).ends_with("w danger zone: ON · Shift+Space auto-end: OFF"));
}

/// The info screen, the map menu, the end-turn prompt and a banner only
/// draw glyphs the font has.
#[test]
fn new_boxes_draw_only_glyphs_in_the_font() {
    let font = FontAtlasDef::load().unwrap_or_default();
    let check = |h: &Harness| {
        let snap = h.snapshot();
        let glyphs = snap.split("\n--- colours ---").next().unwrap_or("");
        for g in glyphs.chars().filter(|&c| c != '\n') {
            assert!(font.glyph_rect(g).is_some(), "{g:?} missing from the font");
        }
    };
    let mut h = quick_battle();
    for keys in ["e", "d d", "f", "d d", "d Down f", "d d Space", "Space"] {
        h.keys(keys);
        check(&h);
    }
    assert!(shows(&h, "ENEMY PHASE"));
}

#[test]
fn every_glyph_drawn_is_in_the_font() {
    let font = FontAtlasDef::load().unwrap_or_default();
    let snap = quick_battle().snapshot();
    let glyphs = snap.split("\n--- colours ---").next().unwrap_or("");
    for g in glyphs.chars().filter(|&c| c != '\n') {
        assert!(font.glyph_rect(g).is_some(), "{g:?} missing from the font");
    }
}

/// The side panel's rows 1..9, inside the border, trimmed.
fn panel(h: &Harness) -> Vec<String> {
    let buf = h.game().buffer();
    (1..9)
        .map(|y| {
            (71..99)
                .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
                .collect::<String>()
                .trim()
                .to_owned()
        })
        .collect()
}

/// The lord's starting tile.
const LORD: Pos = Pos::new(3, 5);

/// The label of the unit the map shows on `(x, y)`, and whether it has
/// acted; `None` for an empty tile.
fn unit(h: &Harness, x: i32, y: i32) -> Option<(String, bool)> {
    h.unit_at(Pos::new(x, y)).map(|u| (u.label, u.acted))
}

/// A unit shown with `label`, acted or not, for comparing with [`unit`].
fn shown(label: &str, acted: bool) -> (String, bool) {
    (label.to_owned(), acted)
}

#[test]
fn arrows_move_the_cursor_and_the_panel_follows() {
    let mut h = quick_battle();
    // The cursor on the lord at (3, 5).
    assert_eq!(h.cursor_tile(), Some(LORD));
    assert_eq!(panel(&h)[4], "Test Lord");
    h.keys("Right Right Right");
    assert_eq!(h.cursor_tile(), Some(Pos::new(6, 5)));
    assert_eq!(panel(&h)[0], "Plain");
    assert_eq!(panel(&h)[4], "");
    // Held: stops at the map's right edge (x = 13).
    h.hold("Right", 1.0);
    assert_eq!(h.cursor_tile(), Some(Pos::new(13, 5)));
    assert_eq!(panel(&h)[0], "Plain");
    // Two tiles left is a fort.
    h.keys("Left Left");
    assert_eq!(panel(&h)[0], "Fort");
    assert_eq!(panel(&h)[1], "DEF +2  AVO +20");
}

#[test]
fn next_unit_jumps_between_ready_units() {
    let mut h = quick_battle();
    // Reading order: archer (2, 4), lord (3, 5), mage (3, 6), knight (4, 6).
    h.keys("s");
    assert_eq!(panel(&h)[4], "Test Mage");
    h.keys("s");
    assert_eq!(panel(&h)[4], "Test Knight");
    h.keys("s");
    assert_eq!(panel(&h)[4], "Test Archer");
    h.keys("s");
    assert_eq!(panel(&h)[4], "Test Lord");
    h.keys("a");
    assert_eq!(panel(&h)[4], "Test Archer");
}

/// The key-help line, left of the right-aligned debug hint.
fn help(h: &Harness) -> String {
    let buf = h.game().buffer();
    (0..90)
        .map(|x| buf.get(x, 31).map_or(' ', |c| c.glyph))
        .collect::<String>()
        .trim()
        .to_owned()
}

/// The lord (at (3, 5)) selected, the path steered two tiles right onto
/// the fort at (5, 5).
fn lord_path() -> Harness {
    let mut h = quick_battle();
    h.keys("f Right Right");
    h
}

/// The lord selected: blue move and red attack ranges, the path from the
/// lord's tile edge to an arrowhead on the fort, no cursor frame there,
/// a double-line panel border.
#[test]
fn selected_unit_with_ranges_and_path_snapshot() {
    let h = lord_path();
    assert_eq!(help(&h), "arrows move · f move here · d cancel");
    assert_snapshot!(h.snapshot());
}

/// The lord walked to the fort, its action menu open beside it.
#[test]
fn action_menu_snapshot() {
    let mut h = lord_path();
    h.keys("f").wait(0.5);
    assert_eq!(help(&h), "arrows choose · f confirm · d back");
    assert_snapshot!(h.snapshot());
}

#[test]
fn select_move_and_wait_dims_the_unit_and_keeps_its_label_case() {
    let mut h = lord_path();
    assert_eq!(unit(&h, 3, 5), Some(shown("Lo", false)));
    h.keys("f").wait(0.5).keys("f");
    // The lord stands on the fort, x + 2, marked as acted (each map skin
    // shows that its own way: the glyph skin dims it), its label as it was.
    assert_eq!(unit(&h, 5, 5), Some(shown("Lo", true)));
    assert_eq!(unit(&h, 3, 5), None);
    assert_eq!(panel(&h)[4], "Test Lord");
    // Browsing again, on a unit that can't act.
    assert_eq!(
        help(&h),
        "arrows move · e info · s next unit · r rewind · d menu · Space end turn"
    );
    // It is no longer selectable.
    h.keys("f");
    assert_eq!(
        help(&h),
        "arrows move · e info · s next unit · r rewind · d menu · Space end turn"
    );
}

#[test]
fn cancelling_the_menu_then_the_selection_restores_the_unit() {
    let mut h = quick_battle();
    // (Select and back: the cursor's pulse restarts, which the banner's
    // time had moved on.)
    h.keys("f d");
    let before = h.snapshot();
    h.keys("f Right Right f").wait(0.5);
    assert_eq!(unit(&h, 5, 5), Some(shown("Lo", false)));
    assert_eq!(unit(&h, 3, 5), None);
    // Back to the steered path: the lord back on its tile.
    h.keys("d");
    assert_eq!(unit(&h, 3, 5), Some(shown("Lo", false)));
    assert_eq!(unit(&h, 5, 5), None);
    assert_eq!(help(&h), "arrows move · f move here · d cancel");
    // Back to browsing, the cursor on the lord (its pulse restarted, as
    // when the battle opened): the screen exactly as it was.
    h.keys("d");
    assert_eq!(h.cursor_tile(), Some(LORD));
    assert_eq!(
        help(&h),
        "f select · e info · s next unit · r rewind · d menu · Space end turn"
    );
    assert_eq!(h.snapshot(), before);
}

#[test]
fn confirm_on_an_enemy_toggles_its_range() {
    let mut h = quick_battle();
    // To the brigand at (8, 2).
    h.keys("Right Right Right Right Right Up Up Up");
    assert_eq!(panel(&h)[4], "Brigand");
    let reach = Pos::new(8, 7);
    assert!(h.tints_at(reach).is_empty());
    h.keys("f");
    assert_eq!(
        h.tints_at(reach),
        [RangeKind::Attack],
        "(8, 7) is in its range"
    );
    assert!(
        help(&h).ends_with("d hide range · Space end turn"),
        "{}",
        help(&h)
    );
    h.keys("f");
    assert!(h.tints_at(reach).is_empty());
    h.keys("f d");
    assert!(h.tints_at(reach).is_empty());
    assert_eq!(h.top_screen(), "battle");
}

#[test]
fn the_lord_fights_the_near_brigand_on_turn_one() {
    let mut h = quick_battle();
    // The lord to (6, 4), beside the brigand at (7, 4).
    h.keys("f Right Right Right Up f").wait(0.5);
    // Attack: the forecast against the brigand with the equipped iron
    // sword. Two swords reach: left/right swap between them (0442).
    h.keys("f");
    assert_eq!(panel(&h)[1], "Test Lord     Brigand");
    assert_eq!(
        help(&h),
        "Left/Right swap · Up/Down target · f attack · d back"
    );
    // Attack, then Cancel skips the playback.
    h.keys("f");
    assert_eq!(help(&h), "d skip · hold f fast");
    h.keys("d");
    // The lord's EXP bar; Confirm finishes it and it closes.
    assert_eq!(help(&h), "f skip · hold f fast");
    assert!(shows(&h, "EXP"));
    h.keys("f");
    // The lord has acted, at (6, 4).
    assert_eq!(unit(&h, 6, 4), Some(shown("Lo", true)));
    assert!(
        help(&h).ends_with("s next unit · r rewind · d menu · Space end turn"),
        "{}",
        help(&h)
    );
}

/// Moves the cursor from where the map shows it to `to` with the arrow keys.
fn go(h: &mut Harness, to: Pos) {
    let Some(at) = h.cursor_tile() else {
        panic!("no cursor on the map");
    };
    let (dx, dy) = (to.x - at.x, to.y - at.y);
    let (x, y) = (
        if dx < 0 { "Left" } else { "Right" },
        if dy < 0 { "Up" } else { "Down" },
    );
    for _ in 0..dx.abs() {
        h.keys(x);
    }
    for _ in 0..dy.abs() {
        h.keys(y);
    }
}

/// Ticket 0442, with the default keys: the lord uses a Combat Art from the
/// arts menu, and the mage casts `Fire + Overcast` from the spell list. A
/// forecast opened by pointing at the enemy has neither: Cancel, then the
/// menu.
#[test]
fn an_art_from_the_arts_menu_and_a_spell_active_from_the_spell_list() {
    use trpg_core::{ArtId, SkillId, SpellId, UnitId};

    let mut h = quick_battle();
    let state = |h: &Harness| h.battle().unwrap().state().clone();
    let (lord, mage) = (UnitId(1), UnitId(8));
    // The lord points at the brigand at (7, 4): it walks beside it and the
    // forecast opens with the iron sword, and no arts list.
    h.keys("f");
    go(&mut h, Pos::new(7, 4));
    h.keys("f").wait(0.5);
    assert_eq!(panel(&h)[1], "Test Lord     Brigand");
    assert_eq!(
        help(&h),
        "Left/Right swap · Up/Down target · f attack · d back"
    );
    assert!(!shows(&h, "Guard Break"));
    // Cancel: the action menu, on Attack. Arts is the next line.
    h.keys("d");
    assert_eq!(help(&h), "arrows choose · f confirm · d back");
    assert!(shows(&h, "Arts"));
    h.keys("Down f");
    assert!(shows(&h, "Iron Sword 20/20"));
    assert!(shows(&h, "Flowing Cut     −2 dur  E"));
    assert!(shows(&h, "Guard Break     −4 dur  D"));
    // Guard Break: its forecast on the brigand, which can't counter.
    h.keys("Down f");
    assert_eq!(panel(&h)[0], "Guard Break (20 → 16)");
    assert_eq!(panel(&h)[1], "Test Lord     Brigand");
    assert!(shows(&h, "no counter"));
    assert!(!shows(&h, "Flowing Cut"));
    // Cancel goes back to the arts menu, on Guard Break; Confirm again.
    h.keys("d");
    assert!(shows(&h, "Flowing Cut     −2 dur  E"));
    h.keys("f f");
    assert_eq!(help(&h), "d skip · hold f fast");
    h.keys("d f");
    let after = state(&h);
    let sword = after.unit(lord).unwrap().loadout.weapon(0).unwrap();
    assert_eq!(sword.durability_left, 16);
    assert!(matches!(
        h.battle().unwrap().history().commands().last(),
        Some(trpg_core::Command::Act {
            action: trpg_core::UnitAction::Attack { art: Some(art), .. },
            ..
        }) if *art == ArtId::new("guard_break")
    ));
    // The mage at (3, 6) points at the brigand too: the forecast of Fire,
    // with nothing to pick Overcast with. Cancel: the menu, on Magic.
    go(&mut h, Pos::new(3, 6));
    h.keys("f");
    go(&mut h, Pos::new(7, 4));
    h.keys("f").wait(0.5);
    assert_eq!(panel(&h)[1], "Test Mage     Brigand");
    assert!(!shows(&h, "Overcast"));
    h.keys("d");
    assert_eq!(help(&h), "arrows choose · f confirm · d back");
    // Magic: the spell list, with a line for Overcast after each attack
    // spell.
    h.keys("f");
    assert!(shows(&h, "Fire + Overcast  +1 use"));
    assert!(shows(&h, "Frost + Overcast  +1 use"));
    h.keys("Down f");
    assert_eq!(panel(&h)[0], "Overcast");
    assert_eq!(panel(&h)[1], "Test Mage     Brigand");
    assert_eq!(help(&h), "arrows next target · f cast · d back");
    h.keys("f");
    assert!(matches!(
        h.battle().unwrap().history().commands().last(),
        Some(trpg_core::Command::Act {
            action: trpg_core::UnitAction::Cast { active: Some(active), .. },
            ..
        }) if *active == SkillId::new("overcast")
    ));
    // The cast and Overcast's extra use.
    let uses = state(&h)
        .unit(mage)
        .unwrap()
        .spells
        .uses_left(&SpellId::new("fire"));
    assert_eq!(uses, 8);
}

#[test]
fn tips_show_when_switched_on() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    h.with_tips().keys("Down f Left f");
    // The start tip waits for the `PLAYER PHASE` banner.
    assert!(shows(&h, "PLAYER PHASE"));
    assert!(!shows(&h, "Your move"));
    h.keys("f");
    assert!(shows(&h, "Your move"));
    assert_eq!(help(&h), "f close");
    h.keys("f");
    assert!(!shows(&h, "Your move"));
}

/// The Quick Battle's dialogue triggers (0705): its rogue arrives on the
/// fort in turn 2's enemy phase, and turn 3 starts with a scene, once its
/// banner is closed.
#[test]
fn the_rogue_arrives_and_turn_three_opens_with_a_scene() {
    let mut h = quick_battle();
    // The fort at (12, 3), empty.
    assert_eq!(unit(&h, 12, 3), None);
    // Turn 1 ends with everyone ready; both banners skipped.
    h.keys("Space Space f");
    wait_for(&mut h, "PLAYER PHASE", 30.0);
    h.keys("f");
    assert!(!shows(&h, "PHASE"));
    // Turn 2's enemy phase (the rogue arrives under its banner).
    h.keys("Space Space");
    assert!(shows(&h, "ENEMY PHASE"));
    h.keys("f");
    wait_for(&mut h, "PLAYER PHASE", 30.0);
    assert!(shows(&h, "Turn 3"));
    assert_eq!(h.screens(), ["title", "battle"]);
    // Closing the banner plays the scene over the map.
    h.keys("f");
    assert_eq!(h.screens(), ["title", "battle", "dialogue"]);
    h.wait(2.0);
    assert!(shows(
        &h,
        "Turn 3. A rogue has slipped into the fort to the east."
    ));
    h.keys("f");
    assert_eq!(h.screens(), ["title", "battle"]);
    assert_eq!(
        help(&h),
        "f select · e info · s next unit · r rewind · d menu · Space end turn"
    );
    assert_eq!(
        unit(&h, 12, 3).map(|(label, _)| label).as_deref(),
        Some("Ro")
    );
}

/// The music cues the run asked for, in order.
fn music(h: &Harness) -> Vec<String> {
    h.audio_requests()
        .into_iter()
        .filter_map(|r| match r {
            AudioRequest::PlayMusic { cue } => Some(cue),
            _ => None,
        })
        .collect()
}

/// Plays the Quick Battle in `h` through a fight, a rewind, the enemy phase
/// and into turn 2, and checks that nothing asked for music or stopped it
/// after the battle's start.
fn nothing_changes_the_music(h: &mut Harness) {
    let cues = h.audio_requests();
    play_into_turn_two(h);
    let music = |all: Vec<AudioRequest>| -> Vec<AudioRequest> {
        let sound = |r: &AudioRequest| matches!(r, AudioRequest::PlaySound { .. });
        all.into_iter().filter(|r| !sound(r)).collect()
    };
    assert_eq!(music(h.audio_requests()), music(cues));
}

/// Quick Battle (`Pool("skirmish")`) plays one track from the pool, asked
/// for once at the start, and nothing in the battle (combat, rewind, the
/// enemy phase, the next turn) changes it.
#[test]
fn quick_battle_keeps_one_skirmish_track() {
    let content = trpg_content::load_embedded().unwrap();
    let pool = &content.audio.pools["skirmish"];
    let mut h = quick_battle();
    let cues = music(&h);
    assert_eq!(cues.len(), 2, "{cues:?}");
    assert_eq!(cues[0], "title");
    assert!(pool.contains(&cues[1]), "{cues:?}");
    nothing_changes_the_music(&mut h);
    assert_eq!(music(&h), cues);
}

/// A battle whose file names a cue plays it, asked for once at the start,
/// and keeps it the same way.
#[test]
fn a_battle_keeps_the_cue_its_file_names() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    let quick = h.ctx_mut().content.battles.get_mut("quick").unwrap();
    quick.music = trpg_core::BattleMusic::Cue("battle_bright".into());
    // The track starts on the Preparations screen and stays on.
    h.keys("Down f");
    assert_eq!(music(&h), ["title", "battle_bright"]);
    h.keys("Left f f");
    assert_eq!(music(&h), ["title", "battle_bright"]);
    nothing_changes_the_music(&mut h);
    assert_eq!(music(&h), ["title", "battle_bright"]);
    // The track started once and was never stopped or started again.
    let starts = h.music_commands().iter().filter(
        |c| matches!(c, trpg_ui::audio::MusicCommand::Start { cue } if cue == "battle_bright"),
    );
    assert_eq!(starts.count(), 1, "{:?}", h.music_commands());
}

/// The lord fights, the fight is rewound, the turn ends through the enemy
/// phase, and turn 2 opens.
fn play_into_turn_two(h: &mut Harness) {
    // The lord attacks the brigand in reach, and the combat plays out.
    h.keys("f Right Right Right Up f")
        .wait(0.5)
        .keys("f")
        .wait(0.5);
    h.keys("f").wait(0.5).keys("f").wait(30.0);
    // Rewind the fight (the hurt lord would fall in the enemy phase), then
    // end the turn through the enemy phase.
    h.keys("r").wait(0.5).keys("f f");
    assert!(!shows(h, "Rewind"), "{}", h.snapshot());
    h.keys("Space Space");
    assert!(shows(h, "ENEMY PHASE"), "{}", h.snapshot());
    h.keys("f");
    wait_for(h, "PLAYER PHASE", 30.0);
    h.keys("f");
    assert!(!shows(h, "PHASE"));
    h.keys("d Down f");
    assert!(shows(h, "Turn 2"));
    assert_eq!(h.screens(), ["title", "battle"]);
}

/// The move loop's menus sound like every other menu (ticket 0425).
#[test]
fn battle_menus_sound() {
    let mut h = quick_battle();
    h.clear_audio();
    // Select the lord, keep it where it stands: the action menu opens.
    h.keys("f f");
    assert_eq!(h.sounds(), ["menu_select", "menu_select"]);
    assert_eq!(help(&h), "arrows choose · f confirm · d back");
    h.clear_audio().keys("Down d d");
    assert_eq!(h.sounds(), ["menu_move", "menu_cancel", "menu_cancel"]);
    // Browsing: back opens the map menu (opening sounds as select).
    h.clear_audio().keys("d Down d");
    assert_eq!(h.sounds(), ["menu_select", "menu_move", "menu_cancel"]);
    // The info screen: opened, cycled, closed.
    h.clear_audio().keys("e Down e");
    assert_eq!(h.sounds(), ["menu_select", "menu_move", "menu_cancel"]);
    // Confirm on a unit that can't be selected (no threat to show for a
    // player unit that has acted) plays nothing: Wait with the lord first.
    h.clear_audio().keys("f f f");
    assert_eq!(h.sounds(), ["menu_select"; 3]);
    h.clear_audio().keys("f");
    assert!(h.sounds().is_empty(), "{:?}", h.sounds());
}
