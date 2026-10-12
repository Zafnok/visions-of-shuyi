//! `cargo xtask battler-import`: copies the bought combat pictures and
//! backdrops the game uses into `assets-private/game/battlers/` and
//! `assets-private/game/battlebacks/` under the public placeholders' names,
//! and writes each picture's sidecar (ticket 0413, ADR-0056).
//!
//! Which picture each class and character gets is the table [`BATTLERS`]
//! (`docs/design/look-and-feel.md` has it in words), and which backdrop is
//! which, [`BACKDROPS`]. A picture drawn on a flat background colour
//! (Shironejiya's) has that colour made clear on the way in.
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

/// A Tiny Tales still battle picture: faces right, its own size.
const fn tiny(id: &'static str, source: &'static str) -> Picture {
    Picture {
        id,
        source,
        faces: Faces::Right,
        scale: (1, 1),
        keyed: false,
    }
}

/// A Shironejiya picture: faces left, on a flat background, at 1.25×.
const fn shironejiya(id: &'static str, source: &'static str) -> Picture {
    Picture {
        id,
        source,
        faces: Faces::Left,
        scale: (5, 4),
        keyed: true,
    }
}

/// Every class and named character of the Quick Battle and Chapter 1 with
/// a bought picture. The Rider has none yet (Pixel Flag's Paladin isn't
/// bought): it keeps the public placeholder.
pub const BATTLERS: [Picture; 12] = [
    // The lead, by gender; the Exile class as the male lead.
    tiny(
        "exile",
        "tiny-tales/characters/heroes/FighterMale/battler.png",
    ),
    tiny(
        "lead_f",
        "tiny-tales/characters/heroes/FighterFemale/battler.png",
    ),
    // Aske is the Archer hero; so is every archer until the class has its
    // own picture.
    tiny("archer", "tiny-tales/characters/heroes/Archer/battler.png"),
    // Rue is the Witch (face and combat picture match).
    tiny("heretic", "tiny-tales/characters/heroes/Witch/battler.png"),
    tiny(
        "mage",
        "tiny-tales/characters/battler-classes/faith-and-evil/Human_Church_7__Church_Wizard/battler.png",
    ),
    tiny(
        "guard",
        "tiny-tales/characters/battler-classes/faith-and-evil/Human_Church_6__Church_Knight/battler.png",
    ),
    // Also Maud's.
    tiny(
        "cleric",
        "tiny-tales/characters/battler-classes/faith-and-evil/Human_Church_1__Church_Cleric/battler.png",
    ),
    tiny(
        "frost_elemental",
        "tiny-tales/characters/battler-classes/elemental-forces/Ice_Elemental/battler.png",
    ),
    tiny(
        "fire_elemental",
        "tiny-tales/characters/battler-classes/elemental-forces/Fire_Elemental/battler.png",
    ),
    // Nick, 2026-10-11 (ticket 0413): the Brigand is the bearded axe
    // bandit in its second colours, the Raider the armoured axe fighter in
    // red, Harl the bearded axe bandit as drawn.
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

/// `--list`: one line per picture and backdrop.
pub fn list(pictures: &[Picture], backdrops: &[(&str, &str)]) -> String {
    let mut out = String::new();
    for p in pictures {
        let (num, den) = p.scale;
        let _ = writeln!(
            out,
            "{:<18} {BATTLERS_DIR}/{}.png  faces {:?}, {num}/{den}  {}",
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
        if p.keyed {
            let (w, h, mut pixels) =
                decode_png(&bytes).map_err(|e| format!("{}: {e}", p.source))?;
            key_out(&mut pixels);
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
