//! `cargo xtask battler-import`: copies the bought combat pictures and
//! backdrops the game uses into `assets-private/game/battlers/` and
//! `assets-private/game/battlebacks/` under the public placeholders' names,
//! and writes each picture's sidecar (ticket 0413, ADR-0056).
//!
//! Which picture each class and character gets is the table [`BATTLERS`]
//! (`docs/design/look-and-feel.md` has it in words), and which backdrop is
//! which, [`BACKDROPS`]. A picture drawn on a flat background colour
//! (Shironejiya's) has that colour made clear on the way in, and a picture
//! shared by two fighters is recoloured for one of them ([`Tint`]).
//!
//! The output is bought art (ADR-0040): it goes to the private assets
//! checkout, never into this repository.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use trpg_content::battler::{BACKDROP_PX, BACKDROPS_DIR, BATTLERS_DIR};
use trpg_content::image::png_size;

use crate::font_atlas::{decode_png, encode_png};

/// Shown for `--help` and after a bad argument.
pub const USAGE: &str = "usage: cargo xtask battler-import [--list]\n\n\
Copies each bought combat picture the game uses from assets-private/library/ to\n\
assets-private/game/battlers/<id>.png with its sidecar <id>.ron, and each backdrop\n\
to assets-private/game/battlebacks/.\n\
--list  print which picture each class and character gets, and copy nothing\n\n\
Run `cargo xtask private-assets --library` first. Afterwards commit and push in\n\
assets-private/, then run `cargo xtask private-assets --pin`.";

/// The sorted bought packs, relative to the repo root.
pub const LIBRARY_DIR: &str = "assets-private/library";
/// The private assets the game is built with, relative to the repo root.
pub const GAME_DIR: &str = "assets-private/game";

/// Which way a bought picture faces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Faces {
    /// Toward the left edge.
    Left,
    /// Toward the right edge.
    Right,
}

/// One bought combat picture the game uses.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Picture {
    /// The class or character it is for: the sidecar's id.
    pub id: &'static str,
    /// The bought file, relative to [`LIBRARY_DIR`].
    pub source: &'static str,
    /// Which way it faces as bought.
    pub faces: Faces,
    /// How big it is drawn (`look-and-feel.md`, *Fighters with no Tiny
    /// Tales art*): numerator and denominator.
    pub scale: (u32, u32),
    /// Whether it is drawn on a flat background colour (its top-left
    /// pixel's) that must be made clear.
    pub keyed: bool,
    /// The recolours it gets (`look-and-feel.md`, *Combat screen*): a
    /// pixel takes the first that fits its colour.
    pub tints: &'static [Tint],
}

/// A recolour: every pixel whose hue is in `hues` (degrees, both ends
/// included) and that is at least `min_sat` saturated gets the hue `hue`,
/// with its saturation and its brightness multiplied (and kept to 1 at
/// most). Shading is kept: only the colour moves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tint {
    /// The hues it changes, in degrees from 0 to 360.
    pub hues: (f32, f32),
    /// The least saturation it changes, 0 to 1: greys and whites stay.
    pub min_sat: f32,
    /// The new hue, in degrees.
    pub hue: f32,
    /// What the saturation is multiplied by.
    pub sat: f32,
    /// What the brightness is multiplied by.
    pub val: f32,
    /// The first row of the picture it changes (0: the whole picture), to
    /// tell clothes from hair of the same colour.
    pub from_row: u32,
}

const fn tint(hues: (f32, f32), min_sat: f32, hue: f32, sat: f32, val: f32) -> Tint {
    Tint {
        hues,
        min_sat,
        hue,
        sat,
        val,
        from_row: 0,
    }
}

/// `tint` for the rows from `from_row` down only.
const fn below(tint: Tint, from_row: u32) -> Tint {
    Tint { from_row, ..tint }
}

/// The Church Knight's gold trim, tabard and shield in Red Company red.
const GOLD_TO_RED: [Tint; 1] = [tint((15.0, 60.0), 0.25, 2.0, 1.1, 0.9)];
/// The same in faded Veyne blue, for Hollis.
const GOLD_TO_BLUE: [Tint; 1] = [tint((15.0, 60.0), 0.25, 215.0, 0.8, 0.95)];
/// The Archer's blue and violet clothes in red, her green leaf brown, and
/// her blond hair dark: a Red Company archer, not Aske.
const ARCHER_TO_RED: [Tint; 3] = [
    tint((185.0, 290.0), 0.15, 2.0, 1.2, 1.0),
    tint((60.0, 170.0), 0.2, 30.0, 0.8, 0.6),
    tint((32.0, 59.0), 0.3, 22.0, 1.0, 0.5),
];
/// The female lead's pink top in her coat's blue, as her portrait has it.
/// Her hair is the same pink, above row 33; her skin's shadows are redder.
const PINK_TO_BLUE: [Tint; 1] = [below(tint((325.0, 352.0), 0.2, 212.0, 0.9, 0.9), 33)];

/// A Tiny Tales still battle picture: faces right, its own size.
const fn tiny(id: &'static str, source: &'static str) -> Picture {
    Picture {
        id,
        source,
        faces: Faces::Right,
        scale: (1, 1),
        keyed: false,
        tints: &[],
    }
}

/// `picture` drawn `num`/`den` times as big.
const fn scaled(picture: Picture, num: u32, den: u32) -> Picture {
    Picture {
        scale: (num, den),
        ..picture
    }
}

/// `picture` recoloured with `tints`.
const fn tinted(picture: Picture, tints: &'static [Tint]) -> Picture {
    Picture { tints, ..picture }
}

/// A Shironejiya picture: faces left, on a flat background, at 1.25×.
const fn shironejiya(id: &'static str, source: &'static str) -> Picture {
    Picture {
        id,
        source,
        faces: Faces::Left,
        scale: (5, 4),
        keyed: true,
        tints: &[],
    }
}

const ARCHER: &str = "tiny-tales/characters/heroes/Archer/battler.png";

/// The Church Knight: the Guard class and Hollis.
const fn church_knight(id: &'static str) -> Picture {
    tiny(
        id,
        "tiny-tales/characters/battler-classes/faith-and-evil/Human_Church_6__Church_Knight/battler.png",
    )
}

/// Every class and named character of the Quick Battle and Chapter 1 with
/// a bought picture (Nick's choices, ticket 0413). A class's picture is
/// what the Red Company's soldiers of that class look like; a named
/// character of the player's who shares it has their own row. The Rider
/// has none yet (Pixel Flag's Paladin isn't bought): it keeps the public
/// placeholder.
pub const BATTLERS: [Picture; 14] = [
    // The lead, by gender; the Exile class as the male lead.
    tiny(
        "exile",
        "tiny-tales/characters/heroes/FighterMale/battler.png",
    ),
    tinted(
        tiny(
            "lead_f",
            "tiny-tales/characters/heroes/FighterFemale/battler.png",
        ),
        &PINK_TO_BLUE,
    ),
    // Aske is the Archer hero as bought; the Red Company's archers are
    // the same picture in red with dark hair.
    tiny("poacher", ARCHER),
    tinted(tiny("archer", ARCHER), &ARCHER_TO_RED),
    // Rue is the Witch (face and combat picture match).
    tiny("heretic", "tiny-tales/characters/heroes/Witch/battler.png"),
    // The Tiny Tales class pictures are drawn smaller than its heroes:
    // the wizard and the cleric at 1.25× stand as tall.
    scaled(
        tiny(
            "mage",
            "tiny-tales/characters/battler-classes/faith-and-evil/Human_Church_7__Church_Wizard/battler.png",
        ),
        5,
        4,
    ),
    // Also Maud's.
    scaled(
        tiny(
            "cleric",
            "tiny-tales/characters/battler-classes/faith-and-evil/Human_Church_1__Church_Cleric/battler.png",
        ),
        5,
        4,
    ),
    // The Red Company's guard in red; Hollis in faded Veyne blue.
    tinted(church_knight("guard"), &GOLD_TO_RED),
    tinted(church_knight("retainer"), &GOLD_TO_BLUE),
    tiny(
        "frost_elemental",
        "tiny-tales/characters/battler-classes/elemental-forces/Ice_Elemental/battler.png",
    ),
    tiny(
        "fire_elemental",
        "tiny-tales/characters/battler-classes/elemental-forces/Fire_Elemental/battler.png",
    ),
    // The Brigand is the bearded axe bandit in its second colours, the
    // Raider the armoured axe fighter in red, Harl the bearded axe bandit
    // as drawn.
    shironejiya("brigand", "shironejiya/monster_s/m_s_bandanna05_b.png"),
    shironejiya("raider", "shironejiya/monster_s/m_s_fighter04_a.png"),
    shironejiya("red_captain", "shironejiya/monster_s/m_s_bandanna05.png"),
];

/// Each backdrop's file name in `game/battlebacks/` (the public
/// placeholder's) and the bought picture, relative to [`LIBRARY_DIR`].
pub const BACKDROPS: [(&str, &str); 2] = [
    (
        "fields.png",
        "tiny-tales/battle-backgrounds/Battleback_FieldsB_Day.png",
    ),
    (
        "mountains.png",
        "tiny-tales/battle-backgrounds/Battleback_MountainA_Day.png",
    ),
];

/// The sidecar of `picture` (`assets/battlers/README.md`).
pub fn sidecar(picture: &Picture) -> String {
    let (num, den) = picture.scale;
    format!(
        "(\n    id: \"{id}\",\n    image: \"{id}.png\",\n    faces: {faces:?},\n    scale: ({num}, {den}),\n)\n",
        id = picture.id,
        faces = picture.faces,
    )
}

/// Makes every pixel of RGBA8 `pixels` that has the first pixel's colour
/// clear.
pub fn key_out(pixels: &mut [u8]) {
    let Some(key) = pixels.first_chunk::<4>().copied() else {
        return;
    };
    for pixel in pixels.as_chunks_mut::<4>().0 {
        if *pixel == key {
            *pixel = [0; 4];
        }
    }
}

/// The hue (degrees, 0 to 360), saturation and brightness (0 to 1) of
/// the colour `rgb`.
pub fn to_hsv([r, g, b]: [u8; 3]) -> (f32, f32, f32) {
    let [r, g, b] = [r, g, b].map(|c| f32::from(c) / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let spread = max - min;
    if spread <= 0.0 {
        return (0.0, 0.0, max);
    }
    let sixth = if max <= r {
        ((g - b) / spread).rem_euclid(6.0)
    } else if max <= g {
        (b - r) / spread + 2.0
    } else {
        (r - g) / spread + 4.0
    };
    (60.0 * sixth, spread / max, max)
}

/// The colour of hue `h` (degrees), saturation `s` and brightness `v`
/// (0 to 1).
pub fn to_rgb(hue: f32, sat: f32, val: f32) -> [u8; 3] {
    let chroma = val * sat;
    let sixth = hue.rem_euclid(360.0) / 60.0;
    let second = chroma * (1.0 - (sixth.rem_euclid(2.0) - 1.0).abs());
    let (r, g, b) = if sixth < 1.0 {
        (chroma, second, 0.0)
    } else if sixth < 2.0 {
        (second, chroma, 0.0)
    } else if sixth < 3.0 {
        (0.0, chroma, second)
    } else if sixth < 4.0 {
        (0.0, second, chroma)
    } else if sixth < 5.0 {
        (second, 0.0, chroma)
    } else {
        (chroma, 0.0, second)
    };
    let low = val - chroma;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 0..=255
    [r, g, b].map(|c| ((c + low) * 255.0).round().clamp(0.0, 255.0) as u8)
}

/// Recolours the RGBA8 `pixels` of a picture `width` pixels wide with
/// `tints`: each pixel that isn't clear takes the first tint that fits it;
/// the others stay as they are.
pub fn recolour(pixels: &mut [u8], width: u32, tints: &[Tint]) {
    let width = (width as usize).max(1);
    for (i, pixel) in pixels.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let row = u32::try_from(i / width).unwrap_or(u32::MAX);
        let [red, green, blue, alpha] = *pixel;
        if alpha == 0 {
            continue;
        }
        let (hue, sat, val) = to_hsv([red, green, blue]);
        let fits =
            |t: &&Tint| t.hues.0 <= hue && hue <= t.hues.1 && sat >= t.min_sat && row >= t.from_row;
        if let Some(t) = tints.iter().find(fits) {
            let [red, green, blue] = to_rgb(t.hue, (sat * t.sat).min(1.0), (val * t.val).min(1.0));
            *pixel = [red, green, blue, alpha];
        }
    }
}

/// `--list`: one line per picture and backdrop.
pub fn list(pictures: &[Picture], backdrops: &[(&str, &str)]) -> String {
    let mut out = String::new();
    for p in pictures {
        let (num, den) = p.scale;
        let recoloured = if p.tints.is_empty() {
            ""
        } else {
            ", recoloured"
        };
        let _ = writeln!(
            out,
            "{:<18} {BATTLERS_DIR}/{}.png  faces {:?}, {num}/{den}{recoloured}  {}",
            p.id, p.id, p.faces, p.source
        );
    }
    for (name, source) in backdrops {
        let _ = writeln!(out, "{:<18} {BACKDROPS_DIR}/{name}  {source}", "backdrop");
    }
    out
}

/// Copies `pictures` and `backdrops` from the library under `root` into
/// its private game assets and returns what to print.
pub fn run(
    root: &Path,
    pictures: &[Picture],
    backdrops: &[(&str, &str)],
) -> Result<String, String> {
    let library = root.join(LIBRARY_DIR);
    let game = root.join(GAME_DIR);
    if !library.is_dir() || !game.is_dir() {
        return Err(format!(
            "{LIBRARY_DIR}/ or {GAME_DIR}/ is missing: run `cargo xtask private-assets --library` first"
        ));
    }
    let read = |source: &str| {
        let path = library.join(source);
        fs::read(&path).map_err(|e| format!("reading {}: {e}", path.display()))
    };
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for p in pictures {
        let mut bytes = read(p.source)?;
        if p.keyed || !p.tints.is_empty() {
            let (w, h, mut pixels) =
                decode_png(&bytes).map_err(|e| format!("{}: {e}", p.source))?;
            if p.keyed {
                key_out(&mut pixels);
            }
            recolour(&mut pixels, w, p.tints);
            bytes = encode_png(w, h, &pixels)?;
        } else if png_size(&bytes).is_none() {
            return Err(format!("{} is not a PNG image", p.source));
        }
        files.push((format!("{BATTLERS_DIR}/{}.png", p.id), bytes));
        files.push((
            format!("{BATTLERS_DIR}/{}.ron", p.id),
            sidecar(p).into_bytes(),
        ));
    }
    for (name, source) in backdrops {
        let bytes = read(source)?;
        if png_size(&bytes) != Some(BACKDROP_PX) {
            return Err(format!(
                "{source} is not a {}×{} px backdrop",
                BACKDROP_PX.0, BACKDROP_PX.1
            ));
        }
        files.push((format!("{BACKDROPS_DIR}/{name}"), bytes));
    }
    for (path, bytes) in &files {
        let path = game.join(path);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
        }
        fs::write(&path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))?;
    }
    Ok(format!(
        "battler-import: wrote {} pictures to {GAME_DIR}/{BATTLERS_DIR}/ and {} backdrops to \
         {GAME_DIR}/{BACKDROPS_DIR}/; commit and push in assets-private/, then run \
         `cargo xtask private-assets --pin`",
        pictures.len(),
        backdrops.len()
    ))
}

#[cfg(test)]
mod tests {
    use trpg_content::ImageTable;
    use trpg_content::battler::{Facing, parse_battler};

    use super::*;

    const LILAC: [u8; 4] = [200, 170, 210, 255];
    const SKIN: [u8; 4] = [220, 180, 150, 255];

    /// A 2×2 picture: background, figure, figure, background.
    fn on_lilac() -> Vec<u8> {
        encode_png(2, 2, &[LILAC, SKIN, SKIN, LILAC].concat()).unwrap()
    }

    const TWO: [Picture; 2] = [
        tiny("guard", "tt/knight.png"),
        shironejiya("brigand", "s/b.png"),
    ];
    const BACK: [(&str, &str); 1] = [("fields.png", "tt/fields.png")];

    fn root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("xtask-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let library = root.join(LIBRARY_DIR);
        fs::create_dir_all(library.join("tt")).unwrap();
        fs::create_dir_all(library.join("s")).unwrap();
        fs::write(library.join("tt/knight.png"), on_lilac()).unwrap();
        fs::write(library.join("s/b.png"), on_lilac()).unwrap();
        let fields = encode_png(336, 248, &vec![9; 336 * 248 * 4]).unwrap();
        fs::write(library.join("tt/fields.png"), fields).unwrap();
        fs::create_dir_all(root.join(GAME_DIR)).unwrap();
        root
    }

    #[test]
    fn a_sidecar_is_one_the_game_reads() {
        let text = sidecar(&TWO[1]);
        assert_eq!(
            text,
            "(\n    id: \"brigand\",\n    image: \"brigand.png\",\n    faces: Left,\n    scale: (5, 4),\n)\n"
        );
        let png = on_lilac();
        let images = ImageTable::from_files([("battlers/brigand.png", &png[..])]).unwrap();
        let battler = parse_battler("brigand.ron", "brigand", &text, &images).unwrap();
        assert_eq!((battler.faces, battler.scale), (Facing::Left, (5, 4)));
        let knight = sidecar(&TWO[0]);
        assert!(
            knight.contains("faces: Right,\n    scale: (1, 1),"),
            "{knight}"
        );
    }

    #[test]
    fn keying_clears_the_first_pixels_colour_only() {
        let mut pixels = [LILAC, SKIN, SKIN, LILAC].concat();
        key_out(&mut pixels);
        assert_eq!(pixels, [[0; 4], SKIN, SKIN, [0; 4]].concat());
        let mut empty: Vec<u8> = Vec::new();
        key_out(&mut empty);
        assert!(empty.is_empty());
    }

    #[test]
    fn pictures_sidecars_and_backdrops_are_written_keyed_where_asked() {
        let root = root("battlers");
        let said = run(&root, &TWO, &BACK).unwrap();
        assert!(
            said.starts_with("battler-import: wrote 2 pictures"),
            "{said}"
        );
        assert!(said.contains("and 1 backdrops"), "{said}");
        let game = root.join(GAME_DIR);
        let pixels = |name: &str| decode_png(&fs::read(game.join(name)).unwrap()).unwrap().2;
        // As bought; and with its background cleared.
        assert_eq!(
            fs::read(game.join("battlers/guard.png")).unwrap(),
            on_lilac()
        );
        assert_eq!(
            pixels("battlers/brigand.png"),
            [[0; 4], SKIN, SKIN, [0; 4]].concat()
        );
        assert_eq!(
            fs::read_to_string(game.join("battlers/guard.ron")).unwrap(),
            sidecar(&TWO[0])
        );
        assert_eq!(
            fs::read_to_string(game.join("battlers/brigand.ron")).unwrap(),
            sidecar(&TWO[1])
        );
        assert_eq!(pixels("battlebacks/fields.png").len(), 336 * 248 * 4);
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn colours_go_to_hue_saturation_brightness_and_back() {
        let close = |a: (f32, f32, f32), b: (f32, f32, f32)| {
            (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.001 && (a.2 - b.2).abs() < 0.001
        };
        let cases: [([u8; 3], (f32, f32, f32)); 10] = [
            ([255, 0, 0], (0.0, 1.0, 1.0)),
            ([255, 255, 0], (60.0, 1.0, 1.0)),
            ([0, 255, 0], (120.0, 1.0, 1.0)),
            ([0, 255, 255], (180.0, 1.0, 1.0)),
            ([0, 0, 255], (240.0, 1.0, 1.0)),
            ([255, 0, 255], (300.0, 1.0, 1.0)),
            ([255, 0, 51], (348.0, 1.0, 1.0)),
            ([51, 102, 204], (220.0, 0.75, 0.8)),
            ([204, 153, 51], (40.0, 0.75, 0.8)),
            ([102, 204, 153], (150.0, 0.5, 0.8)),
        ];
        for (rgb, hsv) in cases {
            assert!(close(to_hsv(rgb), hsv), "{rgb:?}: {:?}", to_hsv(rgb));
            assert_eq!(to_rgb(hsv.0, hsv.1, hsv.2), rgb, "{hsv:?}");
        }
        // Greys have no hue and no saturation.
        assert_eq!(to_hsv([0, 0, 0]), (0.0, 0.0, 0.0));
        assert_eq!(to_hsv([255, 255, 255]), (0.0, 0.0, 1.0));
        assert!(close(to_hsv([102, 102, 102]), (0.0, 0.0, 0.4)));
        assert_eq!(to_rgb(123.0, 0.0, 0.4), [102, 102, 102]);
        // A hue outside 0 to 360 wraps round.
        assert_eq!(to_rgb(360.0, 1.0, 1.0), [255, 0, 0]);
        assert_eq!(to_rgb(-120.0, 1.0, 1.0), [0, 0, 255]);
        assert_eq!(to_rgb(480.0, 1.0, 1.0), [0, 255, 0]);
        // Each sixth of the wheel, half way.
        assert_eq!(to_rgb(30.0, 1.0, 1.0), [255, 128, 0]);
        assert_eq!(to_rgb(90.0, 1.0, 1.0), [128, 255, 0]);
        assert_eq!(to_rgb(150.0, 1.0, 1.0), [0, 255, 128]);
        assert_eq!(to_rgb(210.0, 1.0, 1.0), [0, 128, 255]);
        assert_eq!(to_rgb(270.0, 1.0, 1.0), [128, 0, 255]);
        assert_eq!(to_rgb(330.0, 1.0, 1.0), [255, 0, 128]);
    }

    #[test]
    fn a_tint_moves_the_hues_it_names_and_keeps_the_shading() {
        // Gold (40°, 0.75, 0.8) to blue, a little duller and darker.
        let to_blue = [tint((15.0, 60.0), 0.25, 220.0, 0.8, 0.5)];
        let gold = [204, 153, 51, 255];
        let pale_gold = [204, 187, 170, 255]; // 30°, saturation 1/6
        let green = [102, 204, 153, 200];
        let clear_gold = [204, 153, 51, 0];
        let mut pixels = [gold, pale_gold, green, clear_gold].concat();
        recolour(&mut pixels, 2, &to_blue);
        // (220°, 0.6, 0.4): 102 × (0.4, 0.6, 1) rounded.
        assert_eq!(pixels[..4], [41, 61, 102, 255]);
        // Too pale, another hue, clear: untouched.
        assert_eq!(pixels[4..], [pale_gold, green, clear_gold].concat());
        // The ends of the range are in it (pure green is exactly 120°, fully
        // saturated); just outside isn't.
        let pure = [0, 255, 0, 255];
        let changed = |t: Tint| {
            let mut p = pure.to_vec();
            recolour(&mut p, 1, &[t]);
            p != pure
        };
        let over = |hues| changed(tint(hues, 0.5, 0.0, 1.0, 1.0));
        assert!(over((120.0, 130.0)) && over((110.0, 120.0)));
        assert!(!over((120.5, 130.0)) && !over((110.0, 119.5)));
        assert!(changed(tint((0.0, 360.0), 1.0, 0.0, 1.0, 1.0)));
        let mut p = gold.to_vec();
        recolour(&mut p, 1, &[tint((0.0, 360.0), 0.76, 0.0, 1.0, 1.0)]);
        assert_eq!(p, gold);
        // A tint for the lower rows leaves the rows above alone: of four
        // gold pixels two wide, the second row; one wide, the last two.
        let lower = [below(tint((0.0, 360.0), 0.0, 120.0, 1.0, 1.0), 1)];
        let green_now = [51, 204, 51, 255];
        let mut p = [gold; 4].concat();
        recolour(&mut p, 2, &lower);
        assert_eq!(p, [gold, gold, green_now, green_now].concat());
        let lowest = [below(lower[0], 2)];
        let mut p = [gold; 4].concat();
        recolour(&mut p, 1, &lowest);
        assert_eq!(p, [gold, gold, green_now, green_now].concat());
        let mut p = [gold; 4].concat();
        recolour(&mut p, 2, &lowest);
        assert_eq!(p, [gold; 4].concat());
        // Saturation and brightness stop at 1.
        let mut p = gold.to_vec();
        recolour(&mut p, 1, &[tint((0.0, 360.0), 0.0, 0.0, 2.0, 2.0)]);
        assert_eq!(p, [255, 0, 0, 255]);
        // The first tint that fits wins; with none, nothing changes.
        let mut p = gold.to_vec();
        let two = [
            tint((0.0, 360.0), 0.0, 120.0, 1.0, 1.0),
            tint((0.0, 360.0), 0.0, 240.0, 1.0, 1.0),
        ];
        recolour(&mut p, 1, &two);
        assert_eq!(p, [51, 204, 51, 255]);
        let mut p = gold.to_vec();
        recolour(&mut p, 1, &[]);
        assert_eq!(p, gold);
    }

    #[test]
    fn a_tinted_picture_is_recoloured_on_the_way_in() {
        const RED: [Tint; 1] = [tint((0.0, 360.0), 0.1, 0.0, 1.0, 1.0)];
        let root = root("battlers-tint");
        let pictures = [
            tinted(tiny("guard", "tt/knight.png"), &RED),
            scaled(tiny("cleric", "tt/knight.png"), 5, 4),
        ];
        run(&root, &pictures, &BACK).unwrap();
        let game = root.join(GAME_DIR);
        let pixels = decode_png(&fs::read(game.join("battlers/guard.png")).unwrap())
            .unwrap()
            .2;
        // Not keyed: the background is recoloured with the figure.
        let red = |c: [u8; 4]| {
            let (_, sat, val) = to_hsv([c[0], c[1], c[2]]);
            let [r, g, b] = to_rgb(0.0, sat, val);
            [r, g, b, 255]
        };
        assert_eq!(
            pixels,
            [red(LILAC), red(SKIN), red(SKIN), red(LILAC)].concat()
        );
        assert_ne!(red(SKIN), SKIN);
        let cleric = fs::read_to_string(game.join("battlers/cleric.ron")).unwrap();
        assert!(cleric.contains("scale: (5, 4),"), "{cleric}");
        assert_eq!(
            fs::read(game.join("battlers/cleric.png")).unwrap(),
            on_lilac()
        );
        assert!(list(&pictures, &BACK).starts_with(
            "guard              battlers/guard.png  faces Right, 1/1, recoloured  tt/knight.png\n\
             cleric             battlers/cleric.png  faces Right, 5/4  tt/knight.png\n"
        ));
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn a_missing_checkout_a_missing_file_and_a_wrong_file_are_errors() {
        let empty =
            std::env::temp_dir().join(format!("xtask-battlers-none-{}", std::process::id()));
        let _ = fs::remove_dir_all(&empty);
        let e = run(&empty, &TWO, &BACK).unwrap_err();
        assert!(e.contains("cargo xtask private-assets --library"), "{e}");
        fs::create_dir_all(empty.join(LIBRARY_DIR)).unwrap();
        assert!(run(&empty, &TWO, &BACK).is_err());
        let _ = fs::remove_dir_all(&empty);

        let root = root("battlers-bad");
        let missing = [tiny("guard", "tt/none.png")];
        let e = run(&root, &missing, &BACK).unwrap_err();
        assert!(e.starts_with("reading ") && e.contains("none.png"), "{e}");
        let library = root.join(LIBRARY_DIR);
        fs::write(library.join("tt/text.png"), b"not a picture").unwrap();
        let e = run(&root, &[tiny("guard", "tt/text.png")], &BACK).unwrap_err();
        assert_eq!(e, "tt/text.png is not a PNG image");
        let e = run(&root, &[shironejiya("b", "tt/text.png")], &BACK).unwrap_err();
        assert!(e.starts_with("tt/text.png: "), "{e}");
        // A backdrop of the wrong size.
        let e = run(&root, &TWO, &[("fields.png", "tt/knight.png")]).unwrap_err();
        assert_eq!(e, "tt/knight.png is not a 336×248 px backdrop");
        // Nothing was written by the runs that failed.
        assert!(!root.join(GAME_DIR).join("battlers").exists());
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn the_list_names_every_picture_and_backdrop() {
        assert_eq!(
            list(&TWO, &BACK),
            "guard              battlers/guard.png  faces Right, 1/1  tt/knight.png\n\
             brigand            battlers/brigand.png  faces Left, 5/4  s/b.png\n\
             backdrop           battlebacks/fields.png  tt/fields.png\n"
        );
    }

    #[test]
    fn the_games_table_names_real_ids_once_each_and_the_placeholders_backdrops() {
        let content = trpg_content::load_embedded().unwrap();
        let mut seen = Vec::new();
        for p in BATTLERS {
            assert!(!seen.contains(&p.id), "{} twice", p.id);
            seen.push(p.id);
            let class = content
                .classes
                .get(&trpg_core::ClassId(p.id.to_owned()))
                .is_some();
            let id = trpg_core::CharacterId(p.id.to_owned());
            let character = trpg_content::tileset::LEAD_PICTURES.contains(&p.id)
                || content.characters.characters.contains_key(&id);
            assert!(class || character, "{} is no class or character", p.id);
            let (num, den) = p.scale;
            assert!(den > 0 && num >= den && num <= 2 * den, "{}", p.id);
        }
        // Each bought backdrop replaces a public placeholder of that name.
        for (name, _) in BACKDROPS {
            let path = format!("{BACKDROPS_DIR}/{name}");
            assert!(content.images.id(&path).is_some(), "{path}");
        }
    }
}
