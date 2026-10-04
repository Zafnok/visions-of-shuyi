//! The battle map under each map skin (ADR-0038, ticket 0433): a skin
//! changes how the map looks and how many tiles fit, never the game.

use insta::assert_snapshot;
use trpg_core::{BattleState, Pos};
use trpg_ui::audio::AudioRequest;
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;
use trpg_ui::map_view::Facing;

/// What one step of a script left: the cursor's tile, the screens, and the
/// battle (if one is on the stack).
#[derive(Debug, PartialEq)]
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
    ("Down f", 0.0),
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

/// Plays [`SCRIPT`] with the map painted by the skin `skin`: what each
/// step left, and every sound and music request of the run.
fn play(skin: &str) -> (Vec<After>, Vec<AudioRequest>) {
    let mut h = Harness::with_layout(Layout::RightHanded);
    h.with_map_skin(skin);
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
    (after, h.audio_requests())
}

/// One long scripted Quick Battle, played under the glyph skin, under the
/// sprite skin and under the mixed skin (sprite units on glyph terrain),
/// ends the same way: the same battle, the cursor on the same tile and the
/// same screens after every step, the same sounds.
#[test]
fn the_skin_never_changes_the_game() {
    let (glyph, glyph_audio) = play("glyph");
    // The script did what it says: the lord moved and fought, the fight
    // was rewound, and the enemy phase came and went.
    let state = |i: usize| glyph[i].battle.clone().unwrap();
    let lord = |s: &BattleState| s.units()[0].pos;
    assert_eq!(glyph[0].screens, ["title", "battle"]);
    assert_eq!(lord(&state(6)), Pos::new(6, 4));
    assert_ne!(state(6), state(1));
    assert_eq!(lord(&state(10)), Pos::new(3, 5), "rewound");
    assert!(state(15).turn() >= 2, "turn {}", state(15).turn());
    let sounds = glyph_audio
        .iter()
        .filter(|r| matches!(r, AudioRequest::PlaySound { .. }));
    assert!(sounds.count() > 10);
    // Step by step, the same.
    for skin in ["sprite", "sprite_units"] {
        let (sprite, sprite_audio) = play(skin);
        for (g, s) in glyph.iter().zip(&sprite) {
            assert_eq!(g, s, "after {:?} under {skin}", g.step);
        }
        assert_eq!(glyph.len(), sprite.len(), "{skin}");
        assert_eq!(glyph_audio, sprite_audio, "{skin}");
    }
}

/// The Quick Battle at its start under the mixed skin, on the public
/// fixture of unit sheets: the terrain, the panels and the help as glyphs
/// (35 × 30 tiles of 16 px, as under the glyph skin), every unit a sprite
/// with its side's outline, standing on its 14-pixel HP bar.
#[test]
fn quick_battle_with_sprite_units_on_glyph_terrain() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    h.with_map_skin("sprite_units");
    h.keys("Down f f");
    assert_eq!(h.screens(), ["title", "battle"]);
    let mut glyphs = Harness::with_layout(Layout::RightHanded);
    glyphs.keys("Down f f");
    // What is shown is what the glyph skin shows.
    assert_eq!(h.map_text(), glyphs.map_text());
    assert_snapshot!(h.snapshot());
}

/// The same after the knight braced (a bonus) and the archer hit a brigand
/// with Pinning Shot (a penalty): both grey and darker, having acted; an
/// up arrow on the knight, dimmed with it; a down arrow on the brigand,
/// whose bar is part empty; and the mage, standing below the lord, cut at
/// its tile's top edge.
#[test]
fn quick_battle_with_sprite_units_after_two_actions() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    h.with_map_skin("sprite_units");
    h.keys("Down f f");
    // The knight, at (4, 6): Skill, Brace.
    h.keys("Right Down f f Up Up Up f f").wait(1.0);
    // The archer, at (2, 4): to (5, 4), Attack, Pinning Shot.
    h.keys("Left Left Left Up Up f Right Right Right");
    h.keys("f").wait(1.0);
    h.keys("f").wait(0.5);
    h.keys("Down Down f").wait(0.5);
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
    assert_snapshot!(h.snapshot());
}

/// What a walk showed of its unit, frame by frame: its tile, its facing,
/// its offset and its walking frame.
type Seen = Vec<(Pos, Facing, (f32, f32), u8)>;

/// In the Quick Battle under the skin `skin`, the lord walks three tiles
/// right and one up: its path, and what the scene shows of it every 50 ms
/// for a second from the Confirm that starts the walk.
fn lord_walk(skin: &str) -> (Vec<Pos>, Seen) {
    let mut h = Harness::with_layout(Layout::RightHanded);
    h.with_map_skin(skin);
    h.keys("Down f f f Right Right Right Up");
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
