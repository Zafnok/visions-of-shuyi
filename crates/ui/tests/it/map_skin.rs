//! The battle map under each map skin (ADR-0038, ticket 0433): a skin
//! changes how the map looks and how many tiles fit, never the game.

use insta::assert_snapshot;
use trpg_core::{BattleState, Pos};
use trpg_ui::audio::AudioRequest;
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::map_view::{Facing, range};
use trpg_ui::screens::battle::layout::MAP_VIEW;
use trpg_ui::settings::MapZoom;

/// Sets the player's map zoom, as the Options screen does.
fn at_zoom(h: &mut Harness, zoom: MapZoom) {
    let saved = h.ctx_mut().change_settings(|s| s.game.map_zoom = zoom);
    assert_eq!(saved, Ok(()));
}

/// What one step of a script left: the cursor's tile, the screens, and the
/// battle (if one is on the stack).
#[derive(Debug, Clone, PartialEq)]
struct After {
    step: &'static str,
    cursor: Option<Pos>,
    screens: Vec<&'static str>,
    battle: Option<BattleState>,
}

/// One step: keys pressed (each a frame, then released), then seconds of
/// waiting.
type Step = (&'static str, f32);

/// The Quick Battle from the title: the lord walks up to the brigand and
/// attacks it, the fight is rewound, the turn ends, the enemy phase plays,
/// and turn 2 starts.
const SCRIPT: [Step; 16] = [
    ("Down f Left f f", 0.0),
    ("f", 0.0),
    ("f Right Right Right Up", 0.0),
    ("f", 1.0),
    ("f", 0.5),
    ("f", 0.5),
    ("f", 30.0),
    ("Left Down", 0.0),
    ("r", 0.5),
    ("f", 0.0),
    ("f", 0.0),
    ("Space", 0.0),
    ("Space", 0.0),
    ("f", 30.0),
    ("f", 1.0),
    ("Down Down", 0.0),
];

/// The Quick Battle's id: what a battle file's `light` is kept under.
const QUICK: &str = "quick";

/// Has the Quick Battle fought in the light `light`, as its battle file's
/// `light:` would (ticket 0438).
fn light_quick_battle(h: &mut Harness, light: &str) {
    let lights = &mut h.ctx_mut().game.content.battle_lights;
    lights.insert(QUICK.to_owned(), light.to_owned());
}

/// What a run of [`SCRIPT`] left and asked for.
#[derive(Debug, Clone, PartialEq)]
struct Run {
    /// What each step left.
    after: Vec<After>,
    /// Every sound and music request, with the frame that asked.
    heard: Vec<(u32, AudioRequest)>,
    /// The skin's walking speeds, in tiles per second (normal, Fast
    /// animations, held): a skin's own (Nick, 2026-10-04 and 2026-11-25),
    /// so a footstep falls when the unit is seen entering its tile.
    walk: (f32, f32, f32),
}

impl Run {
    /// The requests in order without their frames, and without the fades'
    /// volume steps (the title's crackle, the place's bed, 1332): one a
    /// frame, they count frames, not what is heard.
    fn sounds(&self) -> Vec<&AudioRequest> {
        let heard = self.heard.iter().map(|(_, r)| r);
        heard
            .filter(|r| !matches!(r, AudioRequest::SoundVolume { .. }))
            .collect()
    }
}

/// Plays [`SCRIPT`] with the map painted by the skin `skin` at the zoom
/// `zoom`, in the light `light` if one is given.
fn play_in(skin: &str, zoom: MapZoom, light: Option<&str>) -> Run {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    at_zoom(&mut h, zoom);
    h.with_map_skin(skin);
    if let Some(light) = light {
        light_quick_battle(&mut h, light);
    }
    let mut after = Vec::new();
    for (keys, seconds) in SCRIPT {
        h.keys(keys);
        if seconds > 0.0 {
            h.wait(seconds);
        }
        let battle = h.battle();
        after.push(After {
            step: keys,
            cursor: battle.map(|b| b.cursor().pos),
            screens: h.screens(),
            battle: battle.map(|b| b.state().clone()),
        });
    }
    let heard = h.audio_log();
    let heard = heard.into_iter().map(|(f, r)| (f, r.clone())).collect();
    let walk = h.ctx().game.map_skin.walk();
    let walk = (walk.normal, walk.fast, walk.held);
    Run { after, heard, walk }
}

/// [`play_in`] the map's own light.
fn play(skin: &str, zoom: MapZoom) -> Run {
    play_in(skin, zoom, None)
}

/// One long scripted Quick Battle, played under the glyph skin, under the
/// sprite skins (plain tiles, and the tileset with layers and looks) and
/// under the mixed skin (sprite units on glyph terrain), at each zoom and
/// in another light, ends the same way: the same battle, the cursor on the
/// same tile and the same screens after every step, the same sounds in
/// the same order. Every skin that walks at the same speed asks for every
/// sound on the same frame (1358): a walk's footsteps follow the walk on
/// screen, and the sprites walk slower than the glyphs (Nick kept the two
/// speeds, 2026-11-25), so only the walks may move a sound.
#[test]
fn the_skin_never_changes_the_game() {
    let glyph = play("glyph", MapZoom::X1);
    // The glyph skin doesn't zoom: the same run whatever the setting.
    assert_eq!(play("glyph", MapZoom::X2), glyph);
    // The script did what it says: the lord moved and fought, the fight
    // was rewound, and the enemy phase came and went.
    let state = |i: usize| glyph.after[i].battle.clone().unwrap();
    let lord = |s: &BattleState| s.units()[0].pos;
    assert_eq!(glyph.after[0].screens, ["title", "game_flow", "battle"]);
    assert_eq!(lord(&state(6)), Pos::new(6, 4));
    assert_ne!(state(6), state(1));
    assert_eq!(lord(&state(10)), Pos::new(3, 5), "rewound");
    assert!(state(15).turn() >= 2, "turn {}", state(15).turn());
    let sounds = glyph.sounds().into_iter();
    let played = sounds.filter(|r| matches!(r, AudioRequest::PlaySound { .. }));
    assert!(played.count() > 10);
    // Under plain tiles, under tiles with layers between them, and under
    // unit sprites on glyph terrain, each at both zooms (ticket 0439: at
    // 2× the view is a quarter of the tiles); then in the dusk light
    // (ticket 0438: shaded ground, and a shaded backdrop behind the
    // fight).
    let mut runs = Vec::new();
    for skin in ["sprite", "test_auto", "sprite_units"] {
        for zoom in MapZoom::ALL {
            runs.push((format!("{skin} at {zoom:?}"), play(skin, zoom)));
        }
    }
    for skin in ["test_auto", "glyph"] {
        let dusk = play_in(skin, MapZoom::X1, Some("dusk"));
        runs.push((format!("{skin} at dusk"), dusk));
    }
    // The first run at each walking speed, which the others at that speed
    // match frame for frame.
    let mut by_walk = vec![glyph.clone()];
    for (name, run) in &runs {
        // Step by step, the same.
        for (g, s) in glyph.after.iter().zip(&run.after) {
            assert_eq!(g, s, "after {:?} under {name}", g.step);
        }
        assert_eq!(glyph.after.len(), run.after.len(), "{name}");
        assert_eq!(glyph.sounds(), run.sounds(), "{name}");
        let Some(same_walk) = by_walk.iter().find(|r| r.walk == run.walk) else {
            by_walk.push(run.clone());
            continue;
        };
        for (g, s) in same_walk.heard.iter().zip(&run.heard) {
            assert_eq!(g, s, "{name}");
        }
        assert_eq!(same_walk.heard, run.heard, "{name}");
    }
    // Both walking speeds were played, the glyphs' and the sprites', and
    // each was matched frame for frame by another run.
    assert_eq!(by_walk.len(), 2, "{:?}", by_walk.iter().map(|r| r.walk));
    for (i, walk) in by_walk.iter().enumerate() {
        let same = runs.iter().filter(|(_, r)| r.walk == walk.walk).count();
        assert!(same > usize::from(i > 0), "{:?}", walk.walk);
    }
}

/// The Quick Battle with the lord selected, fought at dusk (its battle
/// file's `light`, here set by hand) under the public tileset with layers:
/// the scene names the light; every picture of the ground (each tile's
/// own, the pictures between tiles, the fort and the bridge) carries
/// dusk's shade; the ranges' washes, the units, their bars and the cursor
/// are as at noon. Under the glyph skin the frame is noon's.
#[test]
fn quick_battle_at_dusk_shades_the_ground_only() {
    let start = |skin: &str, light: Option<&str>| {
        let mut h = Harness::with_layout(RIGHT_HANDED);
        at_zoom(&mut h, MapZoom::X1);
        h.with_map_skin(skin);
        if let Some(light) = light {
            light_quick_battle(&mut h, light);
        }
        h.keys("Down f Left f f f f");
        assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
        h
    };
    let noon = start("test_auto", None);
    let dusk = start("test_auto", Some("dusk"));
    let scene = dusk.map_scene().unwrap();
    assert_eq!(scene.look.light, "dusk");
    assert_eq!(noon.map_scene().unwrap().look.light, "noon");
    // What is shown is the same but for the light's name.
    let text = dusk.map_text();
    assert!(
        text.lines().next().unwrap().ends_with(" light dusk"),
        "{text}"
    );
    assert_eq!(text.replacen(" light dusk", "", 1), noon.map_text());
    // Line by line, the frame is noon's with a shade on the ground's
    // pictures: the tileset's own image, not one colour, under the glyphs.
    let (lit, plain) = (dusk.snapshot_of(MAP_VIEW), noon.snapshot_of(MAP_VIEW));
    assert_eq!(lit.lines().count(), plain.lines().count());
    let shade = " shade=350:620,560,460";
    let mut shaded = 0;
    for (lit, plain) in lit.lines().zip(plain.lines()) {
        if lit == plain {
            assert!(!lit.contains("shade="), "{lit}");
            continue;
        }
        assert_eq!(lit.replacen(shade, "", 1), plain);
        assert!(lit.starts_with("under sprite "), "{lit}");
        assert!(lit.contains(" tilesets/test_auto.png "), "{lit}");
        assert!(!lit.contains("solid="), "{lit}");
        shaded += 1;
    }
    // Every tile of the 15 × 10 map, and the pictures between them.
    assert!(shaded > 150, "{shaded}");
    // The units aren't the tileset image's ground: none is shaded.
    let units = lit.lines().filter(|l| l.contains("tilesets/test_units"));
    assert!(units.clone().count() > 0);
    assert!(units.into_iter().all(|l| !l.contains("shade=")));
    assert_snapshot!(lit);
    // The glyph skin has no pictures to shade.
    assert_eq!(
        start("glyph", Some("dusk")).snapshot(),
        start("glyph", None).snapshot()
    );
}

/// The Quick Battle with the lord selected, under the public tileset with
/// layers (16 × 16 tiles, as the bought art's, so the view is the glyph
/// skin's): every tile its own picture; the lines round the water and
/// round the woods as pictures between tiles, cut at the map's edge; the
/// fort's frame and the bridge's rails on their tiles; the lord's ranges
/// as each tile's own shape in the range's colour; and the units as
/// sprites.
#[test]
fn quick_battle_with_layers_between_tiles() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    at_zoom(&mut h, MapZoom::X1);
    h.with_map_skin("test_auto");
    // Past Preparations (Fight!) and the battle's notes; then the lord.
    h.keys("Down f Left f f f f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    let mut glyphs = Harness::with_layout(RIGHT_HANDED);
    glyphs.keys("Down f Left f f f f");
    // What is shown is what the glyph skin shows.
    assert_eq!(h.map_text(), glyphs.map_text());
    let scene = h.map_scene().unwrap();
    assert_ne!(scene.tinted(range::MOVE), [] as [trpg_core::Pos; 0]);
    assert_eq!(scene.look.tiles, "outdoor");
    assert_snapshot!(h.snapshot_of(MAP_VIEW));
}

/// The Quick Battle at its start under the mixed skin, on the public
/// fixture of unit sheets: the terrain, the panels and the help as glyphs
/// (35 × 30 tiles of 16 px, as under the glyph skin), every unit a sprite
/// with its side's outline, standing on its 14-pixel HP bar.
#[test]
fn quick_battle_with_sprite_units_on_glyph_terrain() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.with_map_skin("sprite_units");
    h.keys("Down f Left f f f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    let mut glyphs = Harness::with_layout(RIGHT_HANDED);
    glyphs.keys("Down f Left f f f");
    // What is shown is what the glyph skin shows.
    assert_eq!(h.map_text(), glyphs.map_text());
    assert_snapshot!(h.snapshot_of(MAP_VIEW));
}

/// The same after the knight braced (a bonus) and the archer hit a brigand
/// with Pinning Shot (a penalty): both grey and darker, having acted; an
/// up arrow on the knight, dimmed with it; a down arrow on the brigand,
/// whose bar is part empty; and the mage, standing below the lord, cut at
/// its tile's top edge.
#[test]
fn quick_battle_with_sprite_units_after_two_actions() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.with_map_skin("sprite_units");
    h.keys("Down f Left f f f");
    // The knight, at (4, 6): Skill, Brace.
    h.keys("Right Down f f Up Up Up f f").wait(1.0);
    // The archer, at (2, 4): to (5, 4), Arts, Pinning Shot.
    h.keys("Left Left Left Up Up f Right Right Right");
    h.keys("f").wait(1.0);
    h.keys("Down f").wait(0.5);
    h.keys("Down f").wait(0.5);
    h.keys("f").wait(30.0);
    h.keys("Down Down");
    let scene = h.map_scene().unwrap();
    let unit = |x, y| scene.unit_at(Pos::new(x, y)).unwrap();
    let (knight, archer, brigand) = (unit(4, 6), unit(5, 4), unit(7, 4));
    assert!(
        knight.acted && knight.effects.bonus && !knight.effects.penalty,
        "{knight:?}"
    );
    assert!(archer.acted && !archer.has_effect(), "{archer:?}");
    assert!(
        brigand.effects.penalty && !brigand.effects.bonus,
        "{brigand:?}"
    );
    assert!(brigand.hp.0 < brigand.hp.1, "{brigand:?}");
    assert_snapshot!(h.snapshot_of(MAP_VIEW));
}

/// What a walk showed of its unit, frame by frame: its tile, its facing,
/// its offset and its walking frame.
type Seen = Vec<(Pos, Facing, (f32, f32), u8)>;

/// In the Quick Battle under the skin `skin`, the lord walks three tiles
/// right and one up: its path, and what the scene shows of it every 50 ms
/// for a second from the Confirm that starts the walk.
fn lord_walk(skin: &str) -> (Vec<Pos>, Seen) {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.with_map_skin(skin);
    h.keys("Down f Left f f f f Right Right Right Up");
    let path = h.path();
    let lord = h.battle().map(|b| b.state().units()[0].id);
    h.keys("f");
    let mut seen = Vec::new();
    for _ in 0..20 {
        let scene = h.map_scene();
        let unit = scene.as_ref().zip(lord).and_then(|(s, id)| s.unit(id));
        seen.extend(unit.map(|u| (u.pos, u.facing, u.offset, u.frame)));
        h.wait(0.05);
    }
    (path, seen)
}

/// Ticket 0440, on the public fixture of unit sheets. Read from the scene
/// (ADR-0038): the lord is on each tile of its path in turn, turned the
/// way it goes and partway to the next tile, and ends on the last one
/// facing the camera. The glyph skin's walk is the same path, quicker.
#[test]
fn a_scripted_move_walks_along_its_path_under_the_sprite_skin() {
    let (path, seen) = lord_walk("sprite_units");
    assert_eq!(seen.len(), 20);
    let at = |x, y| Pos::new(x, y);
    assert_eq!(path, [at(3, 5), at(4, 5), at(5, 5), at(6, 5), at(6, 4)]);
    let mut tiles: Vec<(Pos, Facing)> = seen.iter().map(|&(pos, f, ..)| (pos, f)).collect();
    tiles.dedup();
    assert_eq!(
        tiles,
        [
            (at(3, 5), Facing::Right),
            (at(4, 5), Facing::Right),
            (at(5, 5), Facing::Right),
            (at(6, 5), Facing::Up),
            (at(6, 4), Facing::Down),
        ]
    );
    for &(pos, facing, (dx, dy), frame) in &seen {
        assert!(frame <= 2, "{seen:?}");
        match facing {
            Facing::Right => assert!((0.0..1.0).contains(&dx) && dy == 0.0, "{seen:?}"),
            Facing::Up => assert!(dx == 0.0 && dy <= 0.0 && dy > -1.0, "{seen:?}"),
            // Arrived: no offset.
            _ => assert_eq!((pos, dx, dy), (at(6, 4), 0.0, 0.0), "{seen:?}"),
        }
    }
    // It was seen between tiles, and with each foot forward.
    assert!(seen.iter().any(|&(_, _, (dx, _), _)| dx > 0.2));
    assert!(seen.iter().any(|&(_, _, (_, dy), _)| dy < -0.2));
    assert!(seen.iter().any(|&(.., frame)| frame == 0));
    assert!(seen.iter().any(|&(.., frame)| frame == 2));
    // The scene is the same under the other sprite skin.
    assert_eq!(lord_walk("sprite"), (path.clone(), seen.clone()));
    // Under the glyph skin the walk is the same path, sooner over: 12
    // tiles a second, not 6.
    let (glyph_path, glyph_seen) = lord_walk("glyph");
    assert_eq!(glyph_path, path);
    let walking = |seen: &Seen| seen.iter().filter(|s| s.1 != Facing::Down).count();
    assert_eq!((walking(&seen), walking(&glyph_seen)), (13, 7));
    let mut glyph_tiles: Vec<Pos> = glyph_seen.iter().map(|s| s.0).collect();
    glyph_tiles.dedup();
    assert_eq!(glyph_tiles, path);
}
