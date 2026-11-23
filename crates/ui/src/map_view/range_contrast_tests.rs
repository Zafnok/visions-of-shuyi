//! The move and attack ranges are easy to tell apart (ticket 1319): in
//! every colour theme, tinted over the ground the player sees them on,
//! the two colours differ by a minimum contrast, at the weakest strength
//! any range style tints them.

use shuyi_ui::grid::glyph::OVERLAY_BLEND;
use shuyi_ui::grid::ranges::{GLINT_BASE, SHIMMER_DEPTH, SHOWN_BLEND};
use trpg_content::palette::{Theme, load_theme};

use crate::color::Rgb;

/// The bought grass tile's colour (its most common pixel, measured from
/// a `frame-png` picture of Chapter 1 with the bought art): the picture
/// look's ground in every theme, since a theme never recolours a picture.
/// The gates never read the art, so it is written out here.
const GRASS_PICTURE: Rgb = Rgb::new(0x59, 0xcf, 0x93);

/// The glyph look's grounds the ranges lie on most: the palette names.
const GLYPH_GROUNDS: [&str; 3] = ["grass_bg", "forest_bg", "road_bg"];

/// The least CIE76 ΔE between the tinted move and attack tiles. Before
/// 1319 the default theme's were 16 apart over the grass picture.
const MIN_DELTA_E: f32 = 30.0;

/// The least gap between their hues, in degrees (CIELAB). Before 1319
/// the default theme's were 32° apart over the grass picture: blue and
/// red both read as murky teal-browns.
const MIN_HUE_GAP: f32 = 75.0;

/// The weakest a range style tints a tile: the shimmer's trough, the
/// glint's base, the still tint.
fn weakest_strength() -> f32 {
    [OVERLAY_BLEND, GLINT_BASE, SHOWN_BLEND - SHIMMER_DEPTH]
        .into_iter()
        .fold(f32::INFINITY, f32::min)
}

/// `colour` in CIELAB (D65).
fn lab(colour: Rgb) -> [f32; 3] {
    let linear = |channel: u8| {
        let v = f32::from(channel) / 255.0;
        if v <= 0.040_45 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    let [red, green, blue] = [colour.r, colour.g, colour.b].map(linear);
    // CIE XYZ, each over the white point's.
    let xyz = [
        (0.4124 * red + 0.3576 * green + 0.1805 * blue) / 0.950_47,
        0.2126 * red + 0.7152 * green + 0.0722 * blue,
        (0.0193 * red + 0.1192 * green + 0.9505 * blue) / 1.088_83,
    ];
    let [fx, fy, fz] = xyz.map(|t| {
        if t > 216.0 / 24389.0 {
            t.cbrt()
        } else {
            (24389.0 / 27.0 * t + 16.0) / 116.0
        }
    });
    [116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz)]
}

/// The ΔE and the hue gap in degrees between `a` and `b`.
fn contrast(a: Rgb, b: Rgb) -> (f32, f32) {
    let (a, b) = (lab(a), lab(b));
    let delta_e = a
        .iter()
        .zip(&b)
        .map(|(p, q)| (p - q).powi(2))
        .sum::<f32>()
        .sqrt();
    let hue = |c: [f32; 3]| c[2].atan2(c[1]).to_degrees();
    let gap = (hue(a) - hue(b)).rem_euclid(360.0);
    (delta_e, gap.min(360.0 - gap))
}

#[test]
fn the_lab_conversion_matches_known_values() {
    let near = |c: Rgb, want: [f32; 3]| {
        let got = lab(c);
        assert!(
            got.iter().zip(&want).all(|(g, w)| (g - w).abs() < 0.1),
            "{got:?}"
        );
    };
    near(Rgb::new(255, 255, 255), [100.0, 0.0, 0.0]);
    near(Rgb::new(0, 0, 0), [0.0, 0.0, 0.0]);
    near(Rgb::new(255, 0, 0), [53.24, 80.09, 67.20]);
    // A grey this dark is on the straight parts of both curves.
    near(Rgb::new(5, 5, 5), [1.371, 0.0, 0.0]);
    let (red, blue) = (Rgb::new(255, 0, 0), Rgb::new(0, 0, 255));
    for (a, b) in [(red, blue), (blue, red)] {
        let (delta_e, gap) = contrast(a, b);
        assert!((delta_e - 176.3).abs() < 0.5, "{delta_e}");
        assert!((gap - 93.7).abs() < 0.5, "{gap}");
    }
}

#[test]
fn the_weakest_tint_is_the_shimmers_trough() {
    assert!((weakest_strength() - (SHOWN_BLEND - SHIMMER_DEPTH)).abs() < 1e-6);
    assert!(
        weakest_strength() > 0.3,
        "the test would judge a faint tint"
    );
}

#[test]
fn move_and_attack_ranges_are_easy_to_tell_apart_in_every_theme() {
    let t = weakest_strength();
    let mut failures = Vec::new();
    for theme in Theme::ALL {
        let def = load_theme(theme).unwrap();
        let colour = |name: &str| Rgb::from(def.get(name).unwrap());
        let (moving, attack) = (colour("move_range"), colour("attack_range"));
        let grounds = GLYPH_GROUNDS
            .iter()
            .map(|&name| (name, colour(name)))
            .chain([("the grass picture", GRASS_PICTURE)]);
        for (name, ground) in grounds {
            let (delta_e, gap) = contrast(ground.lerp(moving, t), ground.lerp(attack, t));
            if delta_e < MIN_DELTA_E || gap < MIN_HUE_GAP {
                failures.push(format!(
                    "{theme:?} over {name}: ΔE {delta_e:.1}, hue gap {gap:.0}°"
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

#[test]
fn every_theme_has_the_same_range_colours() {
    // Nick, ticket 1319: a theme doesn't change the ranges, so a washed-out
    // theme doesn't wash them out too.
    let ranges = |theme| {
        let def = load_theme(theme).unwrap();
        ["move_range", "attack_range", "heal_range"].map(|name| def.get(name).unwrap())
    };
    let default = ranges(Theme::default());
    for theme in Theme::ALL {
        assert_eq!(ranges(theme), default, "{theme:?}");
    }
}
