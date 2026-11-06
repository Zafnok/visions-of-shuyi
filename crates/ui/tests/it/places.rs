//! Conversations drawn over their place, darkened (ticket 0732,
//! `docs/design/look-and-feel.md`, *Places behind conversations*):
//! Chapter 1's scenes before and after its battle over the battle's map,
//! a scene during a battle over the battle map as it stands, and the
//! chapter's places of their own (ticket 0733).

use insta::assert_snapshot;
use trpg_core::{Faction, LeadGender, Pos, UnitId};
use trpg_ui::Ctx;
use trpg_ui::ShuyiCtx;
use trpg_ui::glyph_buffer::Rect;
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::map_view::{MapScene, Place};
use trpg_ui::screens::DialogueScreen;
use trpg_ui::screens::dialogue::glyph::PLACE_BRIGHTNESS;

use crate::playthrough::{battle, play_in_game, skip_scenes};

/// The dialogue screen playing (a battle's, or the game flow's).
fn dialogue(h: &Harness) -> &DialogueScreen {
    let Some(screen) = h.game().screen::<DialogueScreen>() else {
        panic!("no scene is playing: {:?}", h.screens());
    };
    screen
}

/// The place behind the talk now.
fn place(h: &Harness) -> Option<MapScene> {
    dialogue(h).view(h.game().ctx()).place
}

/// Chapter 1's field, with nobody on it, as the whole screen shows it
/// centred on `centre` with `ctx`'s map skin.
fn field(ctx: &Ctx, centre: Pos) -> MapScene {
    let def = &ctx.game.content.battles["ch01"];
    let view = ctx.game.map_skin.view_tiles(Rect::new(0, 0, 100, 32));
    Place::of_battle(&ctx.game.content, def).scene(centre, view)
}

/// The game opened on `scene`, as `--scene` does.
fn preview(scene: &str) -> Harness {
    let ctx = Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_layout(RIGHT_HANDED);
    let game = trpg_ui::game::start_on_scene(ctx, scene, LeadGender::Male)
        .unwrap_or_else(|e| panic!("{e}"));
    Harness::from_game(game)
}

/// From New Game, the letter (no place yet), then the Red Company on the
/// green: the battle's field at Harrowby, by the chapel, with nobody on it
/// yet. In the glyph look.
#[test]
fn the_scene_before_the_battle_is_drawn_over_its_field_at_harrowby() {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    h.keys("f f f Up f");
    assert_eq!(dialogue(&h).player().scene_id(), "ch01_intro");
    assert_eq!(place(&h), None);
    // Skip the letter (skipping stops at each reply choice: the first
    // reply is picked).
    for _ in 0..8 {
        if dialogue(&h).player().scene_id() != "ch01_intro" {
            break;
        }
        h.keys("d f");
    }
    assert_eq!(dialogue(&h).player().scene_id(), "ch01_prebattle");
    let ctx = h.game().ctx();
    assert_eq!(ctx.game.map_skin.name(), "glyph");
    let shown = place(&h);
    assert_eq!(shown, Some(field(ctx, Pos::new(3, 12))));
    assert!(shown.is_some_and(|s| s.units.is_empty()));
    // The first line typed out.
    h.wait(5.0);
    assert_snapshot!(h.snapshot());
}

/// After the battle, at the toll-fort.
#[test]
fn the_scene_after_the_battle_is_drawn_over_its_field_at_the_toll_fort() {
    let h = preview("ch01_victory");
    let ctx = h.game().ctx();
    assert_eq!(place(&h), Some(field(ctx, Pos::new(20, 3))));
}

/// Map file `map` as the whole screen shows it centred on `centre` with
/// `ctx`'s map skin.
fn map_place(ctx: &Ctx, map: &str, centre: Pos) -> MapScene {
    let view = ctx.game.map_skin.view_tiles(Rect::new(0, 0, 100, 32));
    Place::of_map(&ctx.game.content.maps[map]).scene(centre, view)
}

/// `scene` without anyone on it.
fn bare(scene: Option<MapScene>) -> Option<MapScene> {
    scene.map(|mut scene| {
        scene.units = Vec::new();
        scene
    })
}

/// The letter (ticket 0733): the opening captions on black, then the
/// yard behind the rented house at Harrowby, at dawn (with the lead and
/// Hollis on it: its opening, 0734, `openings.rs`). The first line over
/// it, in the glyph look.
#[test]
fn the_letter_is_read_in_the_yard_at_harrowby() {
    let mut h = preview("ch01_intro");
    for _ in 0..3 {
        assert_eq!(place(&h), None);
        h.wait(3.0);
        h.keys("f");
    }
    let ctx = h.game().ctx();
    assert_eq!(ctx.game.map_skin.name(), "glyph");
    let yard = map_place(ctx, "harrowby_yard", Pos::new(12, 8));
    assert_eq!(bare(place(&h)), Some(yard));
    assert_eq!(ctx.game.content.maps["harrowby_yard"].look.light, "dawn");
    h.wait(5.5);
    assert_snapshot!(h.snapshot());
}

/// The scene after the chapter (0733): the old lord's study at Veyne
/// Hall, at night, centred on the desk (with Dace in it: its opening,
/// 0734).
#[test]
fn the_last_scene_is_in_the_study_at_veyne_hall() {
    let h = preview("ch01_tbc");
    let ctx = h.game().ctx();
    let study = map_place(ctx, "veyne_study", Pos::new(12, 7));
    assert_eq!(bare(place(&h)), Some(study));
    let look = &ctx.game.content.maps["veyne_study"].look;
    assert_eq!(
        (look.tiles.as_str(), look.light.as_str()),
        ("indoor", "night")
    );
}

/// A scene played over the battle map darkens it: every map cell around
/// the talk is the battle's own, at [`PLACE_BRIGHTNESS`].
#[test]
fn the_battle_map_behind_a_battles_scene_is_darkened() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.keys("Down f Left f f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    let before = h.game().buffer().clone();
    h.keys("F2 Down Down Down f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle", "dialogue"]);
    assert!(dialogue(&h).view(h.game().ctx()).overlay);
    let during = h.game().buffer();
    let mut terrain = 0;
    // Between the portraits' frames, above the text box.
    for (x, y) in (2..19).flat_map(|y| (36..64).map(move |x| (x, y))) {
        let (Some(was), Some(now)) = (before.get(x, y), during.get(x, y)) else {
            continue;
        };
        if !matches!(was.glyph, '^' | '♣' | '♠' | '▲') || now.glyph != was.glyph {
            continue;
        }
        terrain += 1;
        let darkened = (
            was.fg.scale(PLACE_BRIGHTNESS),
            was.bg.scale(PLACE_BRIGHTNESS),
        );
        assert_eq!((now.fg, now.bg), darkened, "cell {x},{y}");
    }
    assert!(terrain > 10, "only {terrain} terrain cells");
}

/// The study is darkened less than the other places (Nick, 0738: "Darken
/// less in this scene only"): its `@place` line asks for 60%; every other
/// Chapter 1 scene is at the usual brightness. After Dace's pacing, the
/// first line over it, in the glyph look.
#[test]
fn the_study_is_darkened_less_than_the_other_places() {
    let mut h = preview("ch01_tbc");
    assert_eq!(dialogue(&h).view(h.game().ctx()).bright, Some(0.6));
    h.wait(5.0);
    assert!(!dialogue(&h).is_opening());
    assert_eq!(dialogue(&h).view(h.game().ctx()).bright, Some(0.6));
    assert_snapshot!(h.snapshot());
    for scene in ["ch01_intro", "ch01_prebattle", "ch01_victory"] {
        let mut h = preview(scene);
        for _ in 0..4 {
            assert_eq!(dialogue(&h).view(h.game().ctx()).bright, None, "{scene}");
            h.wait(3.0).keys("f");
        }
    }
}

/// After Chapter 1 is won (0738): the toll-fort at noon (the beat sheet's
/// time), with the party standing where the battle left them, on no side,
/// and no enemy; the field starts bright and darkens over half a second
/// before the first line comes up. The first line, in the glyph look.
#[test]
fn the_victory_shows_who_is_left_standing_and_fades_into_the_dark() {
    let mut h = Harness::with_layout(RIGHT_HANDED).with_shipped_story();
    h.keys("f f f Up f");
    skip_scenes(&mut h);
    play_in_game(&mut h, &crate::ch01_winnable::winning_commands());
    let end = battle(&h);
    let mut stood: Vec<(UnitId, Pos)> = end
        .units()
        .iter()
        .filter(|u| u.faction == Faction::Player)
        .map(|u| (u.id, u.pos))
        .collect();
    stood.sort();
    assert_eq!(stood.len(), 6, "nobody lost");
    // Harl's last words, the VICTORY banner and the results.
    for _ in 0..60 {
        match h.top_screen() {
            "dialogue" if dialogue(&h).player().scene_id() == "ch01_victory" => break,
            "dialogue" => skip_scenes(&mut h),
            _ => {
                h.wait(0.5).keys("f");
            }
        }
    }
    assert_eq!(dialogue(&h).player().scene_id(), "ch01_victory");
    // Bright, with no talk yet.
    assert!(dialogue(&h).is_opening());
    let start = dialogue(&h).view(h.game().ctx());
    let darkened = start.opening.map_or(1.0, |o| o.darkened);
    assert!(darkened < 0.1, "{darkened}");
    let field = start.place.unwrap_or_else(|| panic!("no place"));
    let mut shown: Vec<(UnitId, Pos)> = field.units.iter().map(|u| (u.id, u.pos)).collect();
    shown.sort();
    assert_eq!(shown, stood);
    assert!(field.units.iter().all(|u| !u.sided));
    // Noon (the beat sheet: "the toll-fort, noon").
    let ctx = h.game().ctx();
    assert_eq!(
        ctx.game
            .content
            .battle_look(&ctx.game.content.battles["ch01"])
            .light,
        "noon"
    );
    assert_eq!(field.look.light, "noon");
    // Half way into the darkening, then the talk.
    h.wait(0.25);
    let half = dialogue(&h)
        .view(h.game().ctx())
        .opening
        .map_or(1.0, |o| o.darkened);
    assert!((0.3..0.8).contains(&half), "{half}");
    h.wait(0.3);
    assert!(!dialogue(&h).is_opening());
    let talk = dialogue(&h).view(h.game().ctx());
    assert!(talk.text.lines[0].starts_with("The gate stands open."));
    let behind = talk.place.map(|p| p.units.len());
    assert_eq!(behind, Some(6), "they stay behind the talk");
    h.wait(5.0);
    assert_snapshot!(h.snapshot());
}
