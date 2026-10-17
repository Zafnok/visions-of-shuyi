//! How a unit looks on the Playdate's map, through the one-bit renderer.

use trpg_content::Ink;
use trpg_core::{Faction, Pos};

use super::*;
use crate::map_view::playdate::tests::{
    blank, c, p, painted, pd_ctx, plains, shown, tile_pixels, unit,
};
use crate::onebit::{ink_is_white, is_white};

/// `u` painted on plains, on tile (1, 1): cells (3, 1) and (4, 1), pixels
/// 24..40 × 16..32.
fn on_plains(u: UnitView) -> GlyphBuffer {
    let ctx = pd_ctx();
    let mut scene = plains(&ctx);
    scene.push_unit(u.at(p(1, 1)));
    painted(&ctx, &scene)
}

const CELL: (i32, i32) = (3, 1);
const TILE: PxRect = Rect::new(24, 16, 16, 16);

fn cells(buf: &GlyphBuffer) -> (Cell, Cell) {
    (*buf.get(3, 1).unwrap(), *buf.get(4, 1).unwrap())
}

#[test]
fn each_side_has_its_look() {
    let inks = Inks::of(&pd_ctx().palette);
    let look = |faction, acted| {
        let mut u = unit(faction, p(0, 0));
        u.acted = acted;
        look(&u, &inks)
    };
    let solid = |tile, letters| Look {
        tile: Some(c(tile)),
        letters: c(letters),
        outline: None,
    };
    assert_eq!(look(Faction::Player, false), solid(Ink::Black, Ink::White));
    assert_eq!(
        look(Faction::Enemy, false),
        solid(Ink::Grey, Ink::BlackEdged)
    );
    let acted = Look {
        tile: Some(c(Ink::White)),
        letters: c(Ink::Black),
        outline: Some(c(Ink::Black)),
    };
    for faction in [
        Faction::Player,
        Faction::Enemy,
        Faction::Ally,
        Faction::Neutral,
    ] {
        assert_eq!(look(faction, true), acted, "{faction:?}");
    }
    let letters = Look {
        tile: None,
        letters: c(Ink::BlackEdged),
        outline: None,
    };
    assert_eq!(look(Faction::Ally, false), letters);
    assert_eq!(look(Faction::Neutral, false), letters);
}

/// Your unit: the letters white in a solid black tile. At full HP its HP
/// line is black on black, so it doesn't show.
#[test]
fn your_unit_is_white_letters_in_a_black_tile() {
    let buf = on_plains(unit(Faction::Player, Pos::new(0, 0)));
    let (white, black) = (c(Ink::White), c(Ink::Black));
    assert_eq!(
        cells(&buf),
        (Cell::new('B', white, black), Cell::new('r', white, black))
    );
    let frame = shown(&buf);
    for ((x, y), ink, _) in tile_pixels(['B', 'r'], CELL) {
        assert_eq!(is_white(&frame, x, y), ink, "({x}, {y})");
    }
}

/// An enemy: the letters on a grey (50% checker) tile, each with a white
/// edge, its HP line black along the bottom.
#[test]
fn an_enemy_is_white_edged_letters_on_a_grey_tile() {
    let buf = on_plains(unit(Faction::Enemy, Pos::new(0, 0)));
    let (edged, grey) = (c(Ink::BlackEdged), c(Ink::Grey));
    assert_eq!(
        cells(&buf),
        (Cell::new('B', edged, grey), Cell::new('r', edged, grey))
    );
    let frame = shown(&buf);
    for ((x, y), ink, near) in tile_pixels(['B', 'r'], CELL) {
        let hp = y >= 30 && (25..39).contains(&x);
        let white = !ink && !hp && (near || ink_is_white(Ink::Grey, x, y));
        assert_eq!(is_white(&frame, x, y), white, "({x}, {y})");
    }
}

/// A unit that has acted: an outline round black letters on white.
#[test]
fn a_unit_that_has_acted_is_outlined() {
    for faction in [Faction::Player, Faction::Enemy] {
        let mut u = unit(faction, Pos::new(0, 0));
        u.acted = true;
        let buf = on_plains(u);
        let (white, black) = (c(Ink::White), c(Ink::Black));
        assert_eq!(
            cells(&buf),
            (Cell::new('B', black, white), Cell::new('r', black, white))
        );
        let sides: Vec<Overlay> = buf
            .overlays()
            .into_iter()
            .filter(|o| o.layer == Layer::Under)
            .collect();
        let side = |x, y, w, h| Overlay::new(Rect::new(x, y, w, h), black, Layer::Under);
        assert_eq!(
            sides,
            [
                side(24, 16, 16, 1),
                side(24, 31, 16, 1),
                side(24, 16, 1, 16),
                side(39, 16, 1, 16),
            ]
        );
        let frame = shown(&buf);
        for ((x, y), ink, _) in tile_pixels(['B', 'r'], CELL) {
            let edge = x == 24 || x == 39 || y == 16 || y >= 30;
            assert_eq!(
                is_white(&frame, x, y),
                !ink && !edge,
                "{faction:?} ({x}, {y})"
            );
        }
    }
}

/// Allies and neutral units (none in Chapter 1): their letters alone, with
/// a white edge, on the terrain and whatever range is on it.
#[test]
fn allies_and_neutral_units_are_their_letters_alone() {
    let ctx = pd_ctx();
    for faction in [Faction::Ally, Faction::Neutral] {
        let mut scene = plains(&ctx);
        scene.tint([p(1, 1)], crate::map_view::RangeKind::Move);
        scene.push_unit(unit(faction, p(1, 1)));
        let buf = painted(&ctx, &scene);
        let (edged, dots) = (c(Ink::BlackEdged), c(Ink::Dots));
        assert_eq!(
            cells(&buf),
            (Cell::new('B', edged, dots), Cell::new('r', edged, dots))
        );
    }
}

/// The HP line: along the bottom of the tile, a pixel in from each side, as
/// long as the HP left in black and the rest white; none for someone who
/// isn't fighting.
#[test]
fn the_hp_line_is_as_long_as_the_hp_left() {
    let (black, white) = (c(Ink::Black), c(Ink::White));
    let line = |hp: (i32, i32), marks: bool| {
        let mut u = unit(Faction::Enemy, Pos::new(0, 0));
        u.hp = hp;
        u.marks = marks;
        let buf = on_plains(u);
        buf.overlays()
            .into_iter()
            .map(|o| (o.rect, o.color, o.layer))
            .collect::<Vec<_>>()
    };
    let bar = |x, w, color| (Rect::new(x, 30, w, 2), color, Layer::Over);
    assert_eq!(line((30, 30), true), [bar(25, 14, black)]);
    assert_eq!(line((15, 30), true), [bar(25, 7, black), bar(32, 7, white)]);
    assert_eq!(line((0, 30), true), [bar(25, 14, white)]);
    assert!(line((15, 30), false).is_empty());
    // On your black tile the HP lost is a white gap.
    let mut hurt = unit(Faction::Player, Pos::new(0, 0));
    hurt.hp = (15, 30);
    let frame = shown(&on_plains(hurt));
    assert!(!is_white(&frame, 31, 31));
    assert!(is_white(&frame, 32, 31));
    assert!(is_white(&frame, 38, 30));
    assert!(!is_white(&frame, 39, 31));
}

/// A unit a battle note is about, or one a blow just landed on, is
/// flipped; a falling unit is gone past the middle of its fall.
#[test]
fn a_picked_out_or_struck_unit_is_flipped_and_a_fallen_one_gone() {
    let invert = |u: UnitView| {
        on_plains(u)
            .overlays()
            .into_iter()
            .filter(|o| o.color == c(Ink::Invert))
            .map(|o| o.rect)
            .collect::<Vec<_>>()
    };
    let base = unit(Faction::Enemy, Pos::new(0, 0));
    assert!(invert(base.clone()).is_empty());
    assert_eq!(invert(base.clone().highlighted(true)), [TILE]);
    for (flash, flipped) in [(0.5, true), (1.0, true), (0.49, false), (f32::NAN, false)] {
        let struck = UnitView {
            flash,
            ..base.clone()
        };
        assert_eq!(!invert(struck).is_empty(), flipped, "{flash}");
    }
    let ctx = pd_ctx();
    let plain = painted(&ctx, &plains(&ctx));
    for (fade, gone) in [(0.5, true), (2.0, true), (0.49, false), (f32::NAN, false)] {
        let buf = on_plains(base.clone().fading(fade));
        assert_eq!(buf == plain, gone, "{fade}");
    }
}

/// A label of one letter still fills the tile; one of three shows two.
#[test]
fn a_short_label_still_fills_its_tile() {
    let mut u = unit(Faction::Player, Pos::new(0, 0));
    u.label = "B".into();
    let (white, black) = (c(Ink::White), c(Ink::Black));
    assert_eq!(cells(&on_plains(u.clone())).1, Cell::new(' ', white, black));
    u.label = "Bri".into();
    assert_eq!(cells(&on_plains(u)).1, Cell::new('r', white, black));
}

/// A unit's items are clipped: here to the tile's left 12 pixels.
#[test]
fn a_units_items_are_clipped() {
    let ctx = pd_ctx();
    let inks = Inks::of(&ctx.palette);
    let mut u = unit(Faction::Enemy, Pos::new(0, 0));
    u.acted = true;
    u.hp = (15, 30);
    u.highlight = true;
    let mut buf = blank();
    let clip = Rect::new(0, 0, 20, 16);
    draw_unit(&mut buf, &ctx.palette, &inks, &u, (1, 0), clip);
    let rects: Vec<Rect> = buf.overlays().iter().map(|o| o.rect).collect();
    let r = Rect::new;
    // Top, bottom, left (no right side); the HP left and lost; the flip.
    assert_eq!(
        rects,
        [
            r(8, 0, 12, 1),
            r(8, 15, 12, 1),
            r(8, 0, 1, 16),
            r(9, 14, 7, 2),
            r(16, 14, 4, 2),
            r(8, 0, 12, 16),
        ]
    );
}
