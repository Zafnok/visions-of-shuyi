use crate::bundle::{Bundle, Layer};
use shuyi_assets::dialogue::DialogueFile;
use trpg_core::{AiWeights, ArtTable, ItemTable, SkillTable};

use super::*;
use crate::dialogue::Stage;
use crate::{ai, art, item, skill};

/// The registry's kinds, each with its formats in the order they are
/// tried. The PNG pixels' kind is there only when a crate built with this
/// one turns the engine's `png` feature on, so it is left out.
#[test]
fn the_registry_lists_its_kinds_and_their_formats() {
    let listing = format!("{:?}", install()).replace(r#"("image pixels", ["png"]), "#, "");
    let kinds = [
        r#"("image", ["png", "one-bit"])"#,
        r#"("sound", ["wav", "ogg-vorbis"])"#,
        r#"("music track", ["ogg-vorbis"])"#,
        r#"("audio manifest", ["audio-manifest"])"#,
        r#"("credits", ["credits"])"#,
        r#"("font atlas", ["font-atlas"])"#,
        r#"("palette", ["palette"])"#,
        r#"("lights", ["lights"])"#,
        r#"("names table", ["names"])"#,
        r#"("dialogue", ["dlg"])"#,
        r#"("dialogue", ["dlg"])"#,
        r#"("keymap", ["keymap"])"#,
        r#"("map", ["map"])"#,
        r#"("cinematic", ["cinematic"])"#,
        r#"("AI weights", ["ron"])"#,
        r#"("art table", ["ron"])"#,
        r#"("item table", ["ron"])"#,
        r#"("skill table", ["ron"])"#,
    ];
    let want = format!(
        "Formats {{ kinds: [{}], limits: Limits {{ max_bytes: {} }} }}",
        kinds.join(", "),
        usize::MAX
    );
    assert_eq!(listing, want);
}

#[test]
fn the_games_formats_are_registered_on_the_standard_ones_without_a_conflict() {
    let mut formats = Formats::standard();
    assert_eq!(register(&mut formats), Ok(()));
    // The game's directives are a kind beside the standard `.dlg`'s.
    assert_eq!(formats.ids::<DialogueFile<Stage>>(), ["dlg"]);
    assert_eq!(formats.ids::<DialogueFile>(), ["dlg"]);
}

/// Its module's parser of a data table.
type Parser<T> = fn(&str, &str) -> Result<T, Vec<ContentError>>;

/// Each data table read through the registry is what its module's parser
/// reads from the real file.
#[test]
fn each_table_is_its_parser_on_the_real_file() {
    fn same<T: DataTable + PartialEq + core::fmt::Debug>(path: &str, parse: Parser<T>) {
        let source = bundle::file(path).unwrap_or_default();
        let parsed = parse(&bundle::display_path(path), source);
        assert!(parsed.is_ok(), "{path}: {parsed:?}");
        assert_eq!(table::<T>(path), parsed, "{path}");
    }
    same::<AiWeights>(ai::AI_PATH, ai::from_source);
    same::<ArtTable>(art::ARTS_PATH, art::from_source);
    same::<ItemTable>(item::ITEMS_PATH, item::from_source);
    same::<SkillTable>(skill::SKILLS_PATH, skill::from_source);
}

#[test]
fn a_table_reports_its_parsers_errors() {
    let file = AssetPath::new("data/ai.ron").shown_as("assets/data/ai.ron");
    let read = TableRon::<AiWeights>::default().decode(&file, b"(damage: -1)", &DecodeCx::new());
    let parsed = ai::from_source("assets/data/ai.ron", "(damage: -1)");
    assert_eq!(read.err(), parsed.err());
    let errors = TableRon::<AiWeights>::default()
        .decode(&file, b"\xff", &DecodeCx::new())
        .err()
        .unwrap_or_default();
    assert_eq!(
        errors[0].to_string(),
        "assets/data/ai.ron: file is not valid UTF-8"
    );
}

#[test]
fn a_missing_file_is_not_found_as_the_loaders_always_said() {
    let errors = load::<Table<AiWeights>>("data/nope.ron")
        .err()
        .unwrap_or_default();
    let shown: Vec<String> = errors.iter().map(ToString::to_string).collect();
    assert_eq!(
        shown,
        ["assets/data/nope.ron: file not found in asset bundle"]
    );
}

static DIALOGUE: &[(&str, &[u8])] = &[
    ("dialogue/a.dlg", b"@scene a\nnot a line\n@end\n"),
    ("dialogue/b.dlg", b"@scene b\n> It rains.\n@end\n"),
    ("dialogue/deeper/c.dlg", b"@scene c\n> Not read.\n@end\n"),
    ("dialogue/README.md", b"# Not dialogue\n"),
];

/// `load_in` reads each file of the kind in the folder (not below it), and
/// a file that fails to read leaves the others read.
#[test]
fn load_in_keeps_the_files_that_read_when_one_fails() {
    // The registry is built from the game's own files first.
    install();
    let fixture: &'static Bundle = Box::leak(Box::new(Bundle::new(vec![Layer::table(
        "fixture", DIALOGUE,
    )])));
    let read = bundle::reading(fixture, || load_in::<DialogueFile<Stage>>("dialogue"));
    let paths: Vec<&str> = read.keys().copied().collect();
    assert_eq!(paths, ["dialogue/a.dlg", "dialogue/b.dlg"]);
    let errors = read["dialogue/a.dlg"]
        .as_ref()
        .err()
        .cloned()
        .unwrap_or_default();
    assert!(
        errors.iter().all(|e| e.file == "fixture/dialogue/a.dlg"),
        "{errors:?}"
    );
    assert!(!errors.is_empty());
    let scenes = read["dialogue/b.dlg"].as_ref().map(|f| f.scenes.len());
    assert_eq!(scenes, Ok(1));
}
