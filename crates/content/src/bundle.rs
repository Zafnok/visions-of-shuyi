//! The asset bundle: every file under the repo's `assets/` directory, embedded
//! in the binary at compile time (ADR-0005). This is the only way `content`
//! reads data; there is no filesystem I/O.
//!
//! The layering is Shuyi Engine's (`shuyi_assets::bundle`, ticket 1103);
//! this module gives it the game's folders as layers ([`layers`]), installs
//! them once ([`install`]), and keeps the functions every loader calls,
//! which read the installed bundle.
//!
//! With the `private-assets` feature the bought art in `assets-private/game/`
//! is embedded too and laid over `assets/` (ADR-0040): a private file
//! replaces the public file at the same path, and a private-only file is
//! added. Tests and gates build without the feature, so they always read the
//! public placeholders.
//!
//! The game's own content from the private repository, in
//! `assets-private/content/`, is laid over `assets/` the same way whenever
//! the build finds it there (ADR-0117), with or without the bought art
//! (which lies over it). Every CI job that loads content checks it out, so
//! the gates read the real game; a build without the checkout (a fork) reads
//! the public files.
//!
//! A debug build, and a build with the `test-fixtures` feature, also embeds
//! the **test files** in `assets-test/` (ADR-0057): the characters, chapter,
//! battles and pictures of the tests and the debug tools. They are added
//! after the story's files ([`TEST_FILES`]); a build that ships has none.
//!
//! Every layer is a table of files `build.rs` writes, which holds no
//! `README.md`: those are notes for whoever edits the files, and no build
//! embeds them (ADR-0105).
//!
//! Without `std` (the Playdate, ADR-0070 §3) there are no test files, and
//! `assets/` is the **Playdate's bundle** (`playdate_layers`, ticket 1036):
//! `assets/` without the files `crates/content/playdate-bundle.txt` lists,
//! which it leaves out on purpose ([`left_out`]). The content is laid over
//! it as a table of its own, and with the `private-assets` feature the
//! bought art over both (ticket 1061).
//!
//! The **demo edition** (ticket 0914, ADR-0119) embeds the demo's tables
//! in place of `assets/`, the content and the bought art ([`demo_layers`]):
//! what `crates/content/demo/bundle.txt` keeps of them, the shared files
//! cut to what Chapter 1 names, and a file saying it is the demo
//! ([`is_demo`]). It has the `demo_edition` cfg, which `build.rs` sets when
//! the export names the demo's content root. Unlike the Playdate's, its
//! bundle doesn't say what it left out: those paths would name the later
//! chapters.
use crate::prelude::*;

use alloc::collections::BTreeMap;
pub use shuyi_assets::bundle::LoadFile;

use crate::error::ContentError;
use shuyi_assets::bundle as engine;
#[cfg(feature = "std")]
pub use shuyi_assets::bundle::reading;
pub use shuyi_assets::bundle::{Bundle, Layer, Watcher, story_only, watch};

/// Every file under `assets/`: its path (relative to `assets/`,
/// `/`-separated) and contents, sorted by path. Not in a packed build,
/// which reads them from the pack beside the program ([`use_pack`]).
#[cfg(any(all(feature = "std", not(demo_edition), not(feature = "packed")), test))]
static ASSETS: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/bundle_files.rs"));

/// [`ASSETS`] without the files the Playdate leaves out: the bundle without
/// `std`.
#[cfg(any(not(feature = "std"), feature = "playdate-bundle", test))]
static PLAYDATE: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/playdate_files.rs"));

/// The paths (sorted) of the files of `assets/` the Playdate leaves out.
#[cfg(any(not(feature = "std"), feature = "playdate-bundle", test))]
static PLAYDATE_LEFT_OUT: &[&str] = include!(concat!(env!("OUT_DIR"), "/playdate_left_out.rs"));

/// The Playdate's table of the bought art (ticket 1061): `build.rs` makes
/// it as [`PLAYDATE`], from `assets-private/game/`.
#[cfg(all(
    feature = "private-assets",
    any(not(feature = "std"), feature = "playdate-bundle")
))]
static PLAYDATE_PRIVATE: &[(&str, &[u8])] =
    include!(concat!(env!("OUT_DIR"), "/playdate_private_files.rs"));

/// The demo's table of `assets/` (ticket 0914): what it keeps, cut, and
/// its edition's file.
#[cfg(any(
    all(demo_edition, not(feature = "packed")),
    feature = "demo-bundle",
    test
))]
static DEMO: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/demo_files.rs"));

/// The demo's table of the bought art.
#[cfg(all(feature = "private-assets", any(demo_edition, feature = "demo-bundle")))]
static DEMO_PRIVATE: &[(&str, &[u8])] =
    include!(concat!(env!("OUT_DIR"), "/demo_private_files.rs"));

#[cfg(all(feature = "std", feature = "private-assets", not(demo_edition)))]
static PRIVATE: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/private_files.rs"));

/// Every file of `assets-private/content/` (ADR-0117), empty when the build
/// had no checkout. Not in the demo edition, whose table holds what it
/// keeps of it.
#[cfg(all(feature = "std", not(demo_edition), not(feature = "packed")))]
static CONTENT: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/content_files.rs"));

/// The Playdate's table of the content, made as [`PLAYDATE`] is.
#[cfg(any(not(feature = "std"), feature = "playdate-bundle", test))]
static PLAYDATE_CONTENT: &[(&str, &[u8])] =
    include!(concat!(env!("OUT_DIR"), "/playdate_content_files.rs"));

#[cfg(all(feature = "std", any(debug_assertions, feature = "test-fixtures")))]
static TEST: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/test_files.rs"));

#[cfg(all(
    feature = "std",
    feature = "private-assets",
    any(debug_assertions, feature = "test-fixtures")
))]
static PRIVATE_TEST: &[(&str, &[u8])] =
    include!(concat!(env!("OUT_DIR"), "/private_test_files.rs"));

/// Whether this build embeds the test files (`assets-test/`, ADR-0057): a
/// debug build, or one with the `test-fixtures` feature, with `std`.
/// `false` in every build that ships.
pub const TEST_FILES: bool = cfg!(all(
    feature = "std",
    any(debug_assertions, feature = "test-fixtures")
));

/// Prefix used when showing an asset path to a human (`assets/data/x.ron`).
pub const DISPLAY_ROOT: &str = "assets";

/// The same prefix for a file that comes from the private assets
/// (`assets-private/game/portraits/x.png`).
pub const PRIVATE_DISPLAY_ROOT: &str = "assets-private/game";

/// The same prefix for a file of the game's content from the private
/// repository (`assets-private/content/dialogue/ch02.dlg`, ADR-0117).
pub const CONTENT_DISPLAY_ROOT: &str = "assets-private/content";

/// The same prefix for a test file (`assets-test/maps/test_small.map`).
pub const TEST_DISPLAY_ROOT: &str = "assets-test";

/// The same prefix for a test file that is bought art
/// (`assets-private/test/portraits/test_lord/neutral.png`).
pub const PRIVATE_TEST_DISPLAY_ROOT: &str = "assets-private/test";

/// The game's embedded directories, in the order a file is read from them:
/// the bought art, the content, `assets/`, then the test files' bought art
/// and the test files, each when this build has it. Without `std`, the
/// Playdate's (`playdate_layers`); in the demo edition, the demo's
/// ([`demo_layers`]).
pub fn layers() -> Vec<Layer> {
    #[cfg(demo_edition)]
    let mut layers = demo_layers();
    #[cfg(not(demo_edition))]
    let mut layers = {
        #[cfg(all(feature = "std", not(feature = "packed")))]
        let content = content_layer(CONTENT);
        // The pack holds the content laid over `assets/` (ADR-0120).
        #[cfg(feature = "packed")]
        let content = None;
        #[cfg(not(feature = "std"))]
        let content = content_layer(PLAYDATE_CONTENT);
        story_layers(content)
    };
    layers.extend(test_layers());
    layers
}

/// [`layers`] with `content` as the content's layer (ADR-0117), between the
/// bought art and `assets/`: a file there replaces `assets/`'s at its path.
/// The layer is marked private in the engine's sense: its files may not be
/// handed out, and a file no layer has is still shown under `assets/`. It
/// isn't bought art: [`is_private`] is false for it, and it needs no
/// credit (ADR-0051). A test lays a folder of its own there.
#[cfg(not(demo_edition))]
pub fn layers_with(content: Option<Layer>) -> Vec<Layer> {
    let mut layers = story_layers(content);
    layers.extend(test_layers());
    layers
}

/// The bought art, `content` and `assets/`, each when this build has it.
#[cfg(not(demo_edition))]
fn story_layers(content: Option<Layer>) -> Vec<Layer> {
    let mut layers = vec![
        #[cfg(all(feature = "std", feature = "private-assets"))]
        Layer::table(PRIVATE_DISPLAY_ROOT, PRIVATE).private(),
        #[cfg(all(not(feature = "std"), feature = "private-assets"))]
        Layer::table(PRIVATE_DISPLAY_ROOT, PLAYDATE_PRIVATE).private(),
    ];
    layers.extend(content.map(Layer::private));
    layers.extend([
        #[cfg(all(feature = "std", not(feature = "packed")))]
        Layer::table(DISPLAY_ROOT, ASSETS),
        #[cfg(feature = "packed")]
        packed_layer(),
        #[cfg(not(feature = "std"))]
        playdate_layer(),
    ]);
    layers
}

/// The test files' bought art and the test files, each when this build
/// has them (ADR-0057).
fn test_layers() -> Vec<Layer> {
    vec![
        #[cfg(all(
            feature = "std",
            feature = "private-assets",
            any(debug_assertions, feature = "test-fixtures")
        ))]
        Layer::table(PRIVATE_TEST_DISPLAY_ROOT, PRIVATE_TEST)
            .private()
            .test(),
        #[cfg(all(feature = "std", any(debug_assertions, feature = "test-fixtures")))]
        Layer::table(TEST_DISPLAY_ROOT, TEST).test(),
    ]
}

/// The content's layer of `table`: none when it is empty (a build without
/// the checkout, or with a checkout of nothing but notes).
#[cfg(all(not(demo_edition), not(feature = "packed")))]
fn content_layer(table: &'static [(&'static str, &'static [u8])]) -> Option<Layer> {
    (!table.is_empty()).then(|| Layer::table(CONTENT_DISPLAY_ROOT, table))
}

/// The demo's layers (ticket 0914, ADR-0119): its bought art over its
/// `assets/` (the content laid over `assets/`, cut), with no test files.
/// The bundle of the demo edition; with the `demo-bundle` feature, for the
/// test that plays the demo against it ([`reading`]).
#[cfg(any(demo_edition, feature = "demo-bundle", test))]
pub fn demo_layers() -> Vec<Layer> {
    vec![
        #[cfg(feature = "private-assets")]
        Layer::table(PRIVATE_DISPLAY_ROOT, DEMO_PRIVATE).private(),
        #[cfg(not(all(demo_edition, feature = "packed", not(feature = "demo-bundle"))))]
        Layer::table(DISPLAY_ROOT, DEMO),
        #[cfg(all(demo_edition, feature = "packed", not(feature = "demo-bundle")))]
        packed_layer(),
    ]
}

/// The stamp of the pack beside the program (ADR-0120): the game's
/// version, so a pack of another version is refused and said to be baked
/// again. The tools bake with it and a packed build opens the pack with it.
pub const PACK_STAMP: &str = concat!("visions-of-shuyi ", env!("CARGO_PKG_VERSION"));

/// What a release export bakes into the pack beside the program (ADR-0120):
/// the files this build reads from `assets/` with the content laid over
/// them, or in the demo edition the demo's table. The bought art stays in
/// the program (the `private-assets` feature, ADR-0040), and no build that
/// ships has the test files. Not in a packed build, which has no tables
/// of them to bake.
#[cfg(all(feature = "std", not(feature = "packed")))]
pub fn pack_layers() -> Vec<Layer> {
    #[cfg(demo_edition)]
    return vec![Layer::table(DISPLAY_ROOT, DEMO)];
    #[cfg(not(demo_edition))]
    {
        let mut layers: Vec<Layer> = content_layer(CONTENT).into_iter().collect();
        layers.push(Layer::table(DISPLAY_ROOT, ASSETS));
        layers
    }
}

/// The pack beside the program a packed build reads in place of the
/// tables of `assets/` and the content ([`use_pack`]).
#[cfg(any(feature = "packed", all(test, feature = "std")))]
static PACKED: std::sync::OnceLock<Layer> = std::sync::OnceLock::new();

/// Lays `pack`, the pack beside the program opened as a layer, where a
/// packed build reads `assets/` and the content from (ADR-0120): before
/// anything reads a file ([`install`]). Only the first call counts.
#[cfg(any(feature = "packed", all(test, feature = "std")))]
pub fn use_pack(pack: Layer) {
    let _ = PACKED.set(pack);
}

/// The pack's layer, or an empty one if the program had none: the game
/// then shows why in place of its title (`crates/app/src/main.rs`).
#[cfg(any(feature = "packed", all(test, feature = "std")))]
fn packed_layer() -> Layer {
    PACKED
        .get()
        .copied()
        .unwrap_or(Layer::table(DISPLAY_ROOT, &[]))
}

/// `assets/` as the Playdate embeds it: without the files
/// `playdate-bundle.txt` lists, which it leaves out.
#[cfg(any(not(feature = "std"), feature = "playdate-bundle", test))]
const fn playdate_layer() -> Layer {
    Layer::table(DISPLAY_ROOT, PLAYDATE).leaving_out(PLAYDATE_LEFT_OUT)
}

/// The Playdate build's layers (ticket 1036): [`layers`] without `std`.
/// With it (the `playdate-bundle` feature), for the test that plays
/// Chapter 1 against them ([`reading`]). The content lies over `assets/`,
/// and with the `private-assets` feature the bought art over both, as on
/// the PC (ticket 1061, ADR-0117).
#[cfg(any(not(feature = "std"), feature = "playdate-bundle", test))]
pub fn playdate_layers() -> Vec<Layer> {
    let mut layers = vec![
        #[cfg(all(
            feature = "private-assets",
            any(not(feature = "std"), feature = "playdate-bundle")
        ))]
        Layer::table(PRIVATE_DISPLAY_ROOT, PLAYDATE_PRIVATE).private(),
    ];
    layers.extend(content_layer(PLAYDATE_CONTENT).map(Layer::private));
    layers.push(playdate_layer());
    layers
}

/// The game's bundle, the process-wide one: installed from [`layers`] the
/// first time anything reads a file ([`crate::load_embedded`] does first).
/// Inside [`reading`], the bundle this thread reads instead.
pub fn install() -> &'static Bundle {
    engine::get_or_install(|| Bundle::new(layers()))
}

/// Whether the game's bundle is the Chapter 1 demo's (ticket 0914): it
/// holds the demo's edition file, which no other bundle has. The demo
/// ends after Chapter 1 and its title has no entry past it.
pub fn is_demo() -> bool {
    file(crate::edition::EDITION_FILE).is_some_and(|text| text == crate::edition::DEMO)
}

/// Whether this build leaves the file at `path` (relative to `assets/`)
/// out on purpose: the Playdate's, for what it does without (ticket 1036).
/// A loader then does without the file instead of reporting it missing.
pub fn left_out(path: &str) -> bool {
    install().left_out(path)
}

/// Returns the UTF-8 contents of the embedded file at `path` (relative to
/// `assets/`, `/`-separated), or `None` if it is missing or not valid UTF-8.
pub fn file(path: &str) -> Option<&'static str> {
    install().file(path)
}

/// The test files' own file at `path`, with its human-facing name, when
/// the story has a file there too: the entries a data table gains in a
/// build with the test files (ADR-0057). [`file()`] reads the story's.
/// `None` without test files, and for a path only one of the two has.
pub fn test_part(path: &str) -> Option<(String, &'static str)> {
    install().test_part(path)
}

/// Returns the raw bytes of the embedded file at `path` (relative to
/// `assets/`, `/`-separated), e.g. an image, or `None` if it is missing.
pub fn bytes(path: &str) -> Option<&'static [u8]> {
    install().bytes(path)
}

/// Lists the paths (relative to `assets/`, `/`-separated, sorted) of every
/// file directly inside directory `dir`. A missing directory yields an empty
/// list. Use `""` for the bundle root.
pub fn files_in(dir: &str) -> Vec<&'static str> {
    install().files_in(dir)
}

/// Parses every file of extension `ext` (without the dot) directly inside
/// `dir` with `parse`, into a map by file stem: the whole map, or every
/// error of every file (a file that is not UTF-8 is one).
pub fn load_each<T>(
    dir: &str,
    ext: &str,
    parse: impl FnMut(&LoadFile<'_>) -> Result<T, Vec<ContentError>>,
) -> Result<BTreeMap<String, T>, Vec<ContentError>> {
    install().load_each(dir, ext, parse)
}

/// The files of a [`load_each`] whose parse gave `Some`, for a loader that
/// reports each file's problems itself and then checks the files together;
/// the files that are not UTF-8 go to `errors` (and then none are kept).
pub fn parsed<T>(
    loaded: Result<BTreeMap<String, Option<T>>, Vec<ContentError>>,
    errors: &mut Vec<ContentError>,
) -> BTreeMap<String, T> {
    match loaded {
        Ok(files) => files
            .into_iter()
            .filter_map(|(stem, value)| Some((stem, value?)))
            .collect(),
        Err(not_text) => {
            errors.extend(not_text);
            BTreeMap::new()
        }
    }
}

/// Lists the paths (relative to `assets/`, `/`-separated, sorted) of every
/// file inside directory `dir` or any directory below it. A missing
/// directory yields an empty list. Use `""` for the whole bundle.
pub fn files_under(dir: &str) -> Vec<&'static str> {
    install().files_under(dir)
}

/// The human-facing name of an asset path, e.g. `assets/data/palette.ron`
/// (`assets-private/game/...` for a file read from the private assets,
/// `assets-test/...` for a test file).
pub fn display_path(path: &str) -> String {
    install().display_path(path)
}

/// The human-facing name of the test file at `path`, there or not: where a
/// tool that makes a test file writes it.
pub fn test_display_path(path: &str) -> String {
    format!("{TEST_DISPLAY_ROOT}/{path}")
}

/// Whether `shown` (a human-facing name, from [`display_path`]) names a
/// file of the private assets: bought art (ADR-0040), the game's or the
/// test files'. Also in a build without them.
pub fn is_private(shown: &str) -> bool {
    [PRIVATE_DISPLAY_ROOT, PRIVATE_TEST_DISPLAY_ROOT]
        .iter()
        .any(|root| engine::shown_under(shown, root))
}

/// Whether the game's own content is here (ADR-0118): its chapters, read
/// as a build that ships has them, without the test files. In `assets/`
/// until ticket 0160 moves it, then only in a checkout of the private
/// repository (ADR-0117). A test that needs the real content asks
/// `skip_without_the_game`.
pub fn has_the_game() -> bool {
    let bundle = install();
    story_only(|| {
        bundle
            .files_in("chapters")
            .iter()
            .any(|p| p.rsplit_once('.').is_some_and(|(_, ext)| ext == "ron"))
    })
}

/// Whether a build has the helpers for the tests of the real content
/// (ADR-0118): the builds with the test files ([`TEST_FILES`]), never one
/// that ships.
macro_rules! for_tests {
    ($($item:item)*) => {
        $(
            #[cfg(all(feature = "std", any(debug_assertions, feature = "test-fixtures")))]
            $item
        )*
    };
}

for_tests! {
    /// Whether CI built this: every CI job that runs a test of the real
    /// content checks the content out (ADR-0117 §3), so there a test never
    /// skips.
    fn in_ci() -> bool {
        option_env!("CI").is_some()
    }

    /// The private repository's `content/tests/` on disk (ADR-0118): the
    /// content's own test data, which no build embeds.
    pub const CONTENT_TESTS: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets-private/content/tests");

    /// For a test that needs the game's own content, which a checkout
    /// without access to the private repository doesn't have (ADR-0118):
    /// `true`, after saying so, when the content is not here
    /// ([`has_the_game`]), and the test returns at once. `false` when it is.
    /// In CI a missing content is a failure, never a skip.
    ///
    /// # Panics
    ///
    /// In CI, when the content is not here.
    pub fn skip_without_the_game() -> bool {
        !there_or_skip(has_the_game(), "the game's content")
    }

    /// For a test of the real content that checks it against the content's
    /// own test data (ADR-0118 §3): the folder [`CONTENT_TESTS`], when it
    /// and the game are here. `None`, after saying so, when not, and the
    /// test returns at once; in CI that is a failure, as for
    /// `skip_without_the_game`.
    ///
    /// # Panics
    ///
    /// In CI, when either is not here.
    pub fn content_tests() -> Option<std::path::PathBuf> {
        let dir = std::path::PathBuf::from(CONTENT_TESTS);
        let here = has_the_game() && dir.is_dir();
        there_or_skip(here, "the game's content and its test data").then_some(dir)
    }

    /// The private repository's `story/` on disk (ADR-0117, ticket 0159):
    /// the story's documents, which no build embeds.
    pub const STORY_DOCS: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets-private/story");

    /// For a test that checks the game against one of the story's documents
    /// (`names.md`, `writers-guide.md`): its path in [`STORY_DOCS`], when it
    /// is there. `None`, after saying so, when not, and the test returns at
    /// once; in CI that is a failure, as for `skip_without_the_game`.
    ///
    /// # Panics
    ///
    /// In CI, when the document is not here.
    pub fn story_doc(name: &str) -> Option<std::path::PathBuf> {
        let path = std::path::Path::new(STORY_DOCS).join(name);
        there_or_skip(path.is_file(), "the story's documents").then_some(path)
    }

    /// `here`; when it is false, a panic in CI, else a note that the test
    /// is skipped because it needs `what`.
    fn there_or_skip(here: bool, what: &str) -> bool {
        if here {
            return true;
        }
        assert!(
            !in_ci(),
            "this test needs {what}, and this CI job didn't check it out (ADR-0117 §3)"
        );
        std::eprintln!(
            "skipped: this test needs {what}, which this checkout doesn't have \
             (`cargo shuyi private-assets`, ADR-0118)"
        );
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bundle of `layers`, for [`reading`].
    fn leaked(layers: Vec<Layer>) -> &'static Bundle {
        Box::leak(Box::new(Bundle::new(layers)))
    }

    /// The game is here when a story layer has a chapter: not when only the
    /// test files have one, nor with only a chapter's notes.
    #[test]
    fn the_game_is_here_when_a_story_layer_has_a_chapter() {
        static CHAPTER: &[(&str, &[u8])] = &[("chapters/one.ron", b"()")];
        static NOTES: &[(&str, &[u8])] = &[("chapters/notes.md", b"notes")];
        let with = |layers| reading(leaked(layers), has_the_game);
        assert!(with(vec![Layer::table(DISPLAY_ROOT, CHAPTER)]));
        assert!(with(vec![
            Layer::table(CONTENT_DISPLAY_ROOT, CHAPTER).private(),
            Layer::table(DISPLAY_ROOT, NOTES),
        ]));
        assert!(!with(vec![Layer::table(DISPLAY_ROOT, NOTES)]));
        assert!(!with(vec![
            Layer::table(DISPLAY_ROOT, NOTES),
            Layer::table(TEST_DISPLAY_ROOT, CHAPTER).test(),
        ]));
    }

    /// Here, the game's content is in `assets/` or the checkout, as on disk,
    /// and its test data in the checkout.
    #[test]
    fn the_game_is_here_as_the_files_on_disk_say() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let on_disk = [DISPLAY_ROOT, CONTENT_DISPLAY_ROOT].iter().any(|layer| {
            std::fs::read_dir(root.join(layer).join("chapters")).is_ok_and(|mut files| {
                files.any(|f| f.is_ok_and(|f| f.path().extension().is_some_and(|e| e == "ron")))
            })
        });
        assert_eq!(has_the_game(), on_disk);
        assert_eq!(skip_without_the_game(), !on_disk);
        let tests = root.join(CONTENT_DISPLAY_ROOT).join("tests");
        let expected = (on_disk && tests.is_dir()).then_some(tests);
        let found = content_tests().map(|dir| dir.canonicalize().unwrap());
        assert_eq!(found, expected.map(|dir| dir.canonicalize().unwrap()));
    }

    /// Without the game a test skips, but in CI it fails.
    #[test]
    fn without_the_game_a_test_skips_but_fails_in_ci() {
        let empty = leaked(vec![Layer::table(DISPLAY_ROOT, &[])]);
        let skipped = std::panic::catch_unwind(|| reading(empty, skip_without_the_game));
        let data = std::panic::catch_unwind(|| reading(empty, content_tests));
        if in_ci() {
            assert!(skipped.is_err() && data.is_err());
        } else {
            assert!(matches!(skipped, Ok(true)));
            assert!(matches!(data, Ok(None)));
        }
    }

    #[test]
    fn reads_existing_file() {
        let text = file("data/palette.ron");
        assert!(text.is_some_and(|t| t.contains("player")));
    }

    #[test]
    fn missing_file_is_none() {
        assert_eq!(file("data/does_not_exist.ron"), None);
    }

    #[test]
    fn reads_binary_file() {
        let png = bytes("fonts/atlas.png").unwrap_or_default();
        assert!(png.starts_with(b"\x89PNG"));
        assert_eq!(bytes("fonts/nope.png"), None);
        assert_eq!(
            bytes("data/palette.ron").map(<[u8]>::len),
            file("data/palette.ron").map(str::len)
        );
    }

    #[test]
    fn lists_files_in_directory() {
        let files = files_in("data");
        assert!(files.contains(&"data/palette.ron"));
        assert!(files.iter().all(|p| p.starts_with("data/")));
        let mut sorted = files.clone();
        sorted.sort_unstable();
        assert_eq!(files, sorted);
    }

    #[test]
    fn root_and_missing_directories() {
        assert!(files_in("").iter().all(|p| !p.contains('/')));
        assert_eq!(files_in("no_such_dir"), [] as [&str; 0]);
    }

    #[test]
    fn lists_files_under_a_directory_and_below() {
        let all = files_under("");
        for path in ["data/palette.ron", "fonts/atlas.png", "audio/sfx/heal.wav"] {
            assert!(all.contains(&path), "{path}");
        }
        let mut sorted = all.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(all, sorted);
        let audio = files_under("audio");
        assert!(audio.contains(&"audio/audio.ron"));
        assert!(audio.contains(&"audio/sfx/heal.wav"));
        assert!(audio.iter().all(|p| p.starts_with("audio/")));
        // `data` holds one folder: the colour themes' palettes.
        let data = files_under("data");
        assert!(data.contains(&"data/palette.ron"));
        assert!(data.contains(&"data/palettes/moonlit.ron"));
        assert!(files_in("data").iter().all(|f| data.contains(f)));
        assert!(!files_in("data").contains(&"data/palettes/moonlit.ron"));
        assert_eq!(files_under("no_such_dir"), [] as [&str; 0]);
    }

    /// The paths (relative to `dir`, `/`-separated) of every file on disk
    /// in `dir` and below it.
    fn on_disk(dir: &std::path::Path, prefix: &str, files: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = format!("{prefix}{}", entry.file_name().to_str().unwrap());
            if entry.path().is_dir() {
                on_disk(&entry.path(), &format!("{path}/"), files);
            } else {
                files.push(path);
            }
        }
    }

    /// Each table is its folder on disk, file for file and byte for byte,
    /// without the READMEs (ADR-0105). The content's too (ADR-0117),
    /// without its `tests/` (ADR-0118): empty
    /// with no checkout, and a build made before the checkout was fetched
    /// fails here, so the gates never quietly read the public files.
    #[test]
    fn a_layer_is_its_folder_without_the_readmes() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let content = root.join(CONTENT_DISPLAY_ROOT);
        let mut layers = vec![("assets", ASSETS, true), ("assets-test", TEST, true)];
        if content.is_dir() {
            // The content's notes are its only file before 0159 and 0160.
            layers.push((CONTENT_DISPLAY_ROOT, CONTENT, false));
        } else {
            assert!(
                CONTENT.is_empty(),
                "{CONTENT_DISPLAY_ROOT}/ is gone: build again"
            );
        }
        for (folder, table, has_readme) in layers {
            let folder = root.join(folder);
            let mut files = Vec::new();
            on_disk(&folder, "", &mut files);
            let readme = files.iter().any(|f| f.ends_with("README.md"));
            assert!(readme || !has_readme, "{files:?}");
            files.retain(|f| !f.ends_with("README.md"));
            // The content's own test data is read from disk (ADR-0118).
            if std::ptr::eq(table, CONTENT) {
                files.retain(|f| !f.starts_with("tests/"));
            }
            files.sort_unstable();
            let paths: Vec<&str> = table.iter().map(|(path, _)| *path).collect();
            assert_eq!(
                paths,
                files,
                "the build's table isn't {}: a checkout fetched after the build? \
                 `cargo clean -p trpg-content`, then build again",
                folder.display()
            );
            for (path, bytes) in table {
                assert_eq!(*bytes, std::fs::read(folder.join(path)).unwrap(), "{path}");
            }
        }
    }

    /// The demo's bundle (ticket 0914) loads with no error, says it is
    /// the demo, and holds Chapter 1 and no later chapter: no file of it
    /// names one (the edition's export checks the program the same way,
    /// `shuyi.ron`'s canaries).
    #[test]
    fn the_demos_bundle_loads_chapter_1_alone_and_says_it_is_the_demo() {
        if skip_without_the_game() {
            return;
        }
        let demo: &'static Bundle = Box::leak(Box::new(Bundle::new(demo_layers())));
        let chapters = reading(demo, || {
            assert!(is_demo());
            let content = crate::load_embedded().unwrap_or_else(|e| panic!("{e}"));
            assert!(content.worldmaps.is_empty());
            content.chapters.into_keys().collect::<Vec<_>>()
        });
        assert_eq!(chapters, ["ch01"]);
        assert!(!is_demo(), "the whole game isn't the demo");
        for path in demo.files_under("") {
            let text = demo.file(path).unwrap_or_default();
            for later in [
                "ch02",
                "ch05",
                "ch08",
                "greywater_kitchen",
                "defector",
                "Chapter 2",
            ] {
                assert!(!text.contains(later), "{path} names {later}");
            }
        }
        for path in ["dialogue/ch02.dlg", "maps/ch02.map", "skirmishes/act1.ron"] {
            assert_eq!(demo.bytes(path), None, "{path}");
            assert!(!demo.left_out(path), "the demo names no file it left out");
        }
    }

    /// No README in any listing or read, with or without the test files;
    /// they stay in the repository, beside the files they describe.
    #[test]
    fn the_bundle_lists_and_reads_no_readme() {
        let is_readme = |path: &&&str| path.rsplit('/').next() == Some("README.md");
        assert_eq!(files_under("").iter().find(is_readme), None);
        assert_eq!(story_only(|| files_under("")).iter().find(is_readme), None);
        let readme = "maps/README.md";
        assert_eq!(file(readme), None);
        assert_eq!(bytes(readme), None);
        assert_eq!(files_in("maps").iter().find(is_readme), None);
        assert!(!left_out(readme));
        let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(manifest.join("../../assets").join(readme).is_file());
    }

    /// The Playdate's table is `assets/` without the files it leaves out,
    /// each with the same bytes but the pictures it draws one-bit (ticket
    /// 1040): those are one-bit pictures made from that PNG. It names the
    /// left-out files as left out.
    #[test]
    fn the_playdate_table_is_assets_without_what_it_leaves_out() {
        let mut all = Vec::from_iter(content_layer(CONTENT));
        all.push(Layer::table(DISPLAY_ROOT, ASSETS));
        let all = Bundle::new(all);
        let playdate = Bundle::new(playdate_layers());
        let kept = playdate.files_under("");
        assert!(kept.len() > 50, "{} files", kept.len());
        for path in all.files_under("") {
            let left_out = PLAYDATE_LEFT_OUT.contains(&path);
            assert_eq!(playdate.left_out(path), left_out, "{path}");
            assert_eq!(kept.contains(&path), !left_out, "{path}");
            if left_out {
                continue;
            }
            if crate::playdate_images::kind_of(path).is_some() {
                let png = all.bytes(path).and_then(crate::image::png_size);
                let bits = playdate.bytes(path).unwrap_or_default();
                let source = shuyi_assets::bitmap::bitmap_sizes(bits).map(|(_, source)| source);
                let source = source.map(|(w, h)| (u32::from(w), u32::from(h)));
                assert_eq!(source, png, "{path}");
            } else {
                assert_eq!(playdate.bytes(path), all.bytes(path), "{path}");
            }
        }
        assert!(PLAYDATE_LEFT_OUT.iter().all(|p| all.bytes(p).is_some()));
        let mut sorted = PLAYDATE_LEFT_OUT.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted, PLAYDATE_LEFT_OUT);
        // What `playdate-bundle.txt` lists.
        for path in ["icon/icon.ico", "audio/sfx/heal.wav"] {
            assert!(playdate.left_out(path), "{path}");
        }
        for path in [
            "data/palette.ron",
            "audio/audio.ron",
            "fonts/atlas_1bit.bin",
        ] {
            assert!(playdate.bytes(path).is_some(), "{path}");
        }
        // One-bit pictures at the size the Playdate shows them, in their
        // styles (`playdate_images.rs`); the logo a lettering, a block of
        // rim round its 52 × 31 blocks, 3 pixels each.
        let bitmap = |path| {
            let bytes = playdate.bytes(path).unwrap_or_default();
            let bitmap = shuyi_assets::bitmap::Bitmap::parse(bytes).ok();
            bitmap.map(|b| (b.width(), b.height(), b.styles(), b.mask().is_some()))
        };
        assert_eq!(
            bitmap("portraits/lead_f/happy.png"),
            Some((64, 64, 4, true))
        );
        assert_eq!(bitmap("battlers/default.png"), Some((96, 144, 1, true)));
        assert_eq!(bitmap("battlebacks/fields.png"), Some((400, 295, 1, false)));
        assert_eq!(
            bitmap("title/fields_b_dusk.png"),
            Some((400, 295, 1, false))
        );
        assert_eq!(bitmap("title/logo.png"), Some((162, 99, 1, true)));
        assert_ne!(
            playdate.bytes("title/logo.png"),
            all.bytes("title/logo.png")
        );
        // The PC's bundle leaves nothing out.
        assert!(!left_out("audio/sfx/heal.wav"));
    }

    #[test]
    fn display_path_prefixes_assets() {
        assert_eq!(display_path("data/palette.ron"), "assets/data/palette.ron");
    }

    /// Only a name under one of the two private folders, whether this build
    /// has them or not.
    #[test]
    fn a_name_under_a_private_folder_is_private() {
        assert!(is_private("assets-private/game/x.png"));
        assert!(is_private("assets-private/test/x.png"));
        assert!(!is_private("assets-private/games/x.png"));
        assert!(!is_private("assets-private/game"));
        assert!(!is_private("assets-private/library/x.png"));
        assert!(!is_private("x/assets-private/game/x.png"));
        assert!(!is_private("assets/x.png"));
        assert!(!is_private("assets-test/x.png"));
    }

    fn names(layers: &[Layer]) -> Vec<&'static str> {
        layers.iter().map(|layer| layer.name).collect()
    }

    /// Without the `private-assets` feature (every gate), the content when
    /// the build has it (ADR-0117: every CI job that loads content does),
    /// `assets/` and the test files are the only layers: a gate never reads
    /// the bought art.
    #[cfg(not(feature = "private-assets"))]
    #[test]
    fn gates_read_the_content_the_public_assets_and_the_test_files() {
        let layers = layers();
        let mut expected = Vec::from_iter((!CONTENT.is_empty()).then_some(CONTENT_DISPLAY_ROOT));
        expected.extend([DISPLAY_ROOT, TEST_DISPLAY_ROOT]);
        assert_eq!(names(&layers), expected);
        let art = |layer: &&Layer| is_private(&format!("{}/x.png", layer.name));
        assert_eq!(layers.iter().find(art).map(|layer| layer.name), None);
        assert_eq!(TEST_FILES, layers.iter().any(|layer| layer.test));
    }

    /// A file of the content replaces `assets/`'s at its path and is shown
    /// under the content's root; the bought art lies over both.
    #[test]
    fn the_content_lies_between_the_bought_art_and_assets() {
        static CONTENT: &[(&str, &[u8])] = &[("data/palette.ron", b"the content's")];
        let content = Layer::table(CONTENT_DISPLAY_ROOT, CONTENT);
        let bundle = Bundle::new(layers_with(Some(content)));
        assert_eq!(bundle.file("data/palette.ron"), Some("the content's"));
        let shown = bundle.display_path("data/palette.ron");
        assert_eq!(shown, "assets-private/content/data/palette.ron");
        assert!(bundle.is_private(&shown));
        assert!(!is_private(&shown));
        let missing = bundle.display_path("data/nothing.ron");
        assert_eq!(missing, "assets/data/nothing.ron");
        let names = names(bundle.layers());
        let at = |name| names.iter().position(|n| *n == name);
        assert_eq!(
            at(CONTENT_DISPLAY_ROOT),
            Some(usize::from(cfg!(feature = "private-assets")))
        );
        assert!(at(CONTENT_DISPLAY_ROOT) < at(DISPLAY_ROOT));
        // A table with files is a layer under the content's root; none without.
        let layer = content_layer(CONTENT).map(|layer| (layer.name, layer.private));
        assert_eq!(layer, Some((CONTENT_DISPLAY_ROOT, false)));
        assert_eq!(content_layer(&[]).map(|layer| layer.name), None);
    }

    /// The installed bundle is the game's layers, in their order.
    #[test]
    fn the_installed_bundle_is_the_games_layers() {
        assert_eq!(names(install().layers()), names(&layers()));
        assert!(core::ptr::eq(install(), install()));
    }

    /// The data tables a build with the test files adds entries to: the
    /// only paths the story's files and the test files may share.
    const TABLES: [&str; 5] = [
        "data/camp.ron",
        "data/characters.ron",
        "data/lessons.ron",
        "data/names.ron",
        "data/supports.ron",
    ];

    /// A test file never stands in for a story file: but for the data
    /// tables, which are merged by id, no path is in both.
    #[test]
    fn only_the_data_tables_are_in_both_the_story_and_the_test_files() {
        let story = story_only(|| files_under(""));
        let tests = Bundle::new(layers().into_iter().filter(|l| l.test).collect());
        let shared: Vec<&str> = tests
            .files_under("")
            .into_iter()
            .filter(|path| story.contains(path))
            .collect();
        assert_eq!(shared, TABLES);
        for path in TABLES {
            let (shown, part) = test_part(path).unwrap();
            assert_eq!(shown, format!("assets-test/{path}"));
            assert_ne!(Some(part), file(path));
            assert_eq!(display_path(path), format!("assets/{path}"));
        }
        assert_eq!(test_part("maps/test_small.map"), None);
        assert_eq!(test_part("data/palette.ron"), None);
    }

    #[test]
    fn a_test_file_is_read_and_shown_under_its_own_root() {
        let map = "maps/test_small.map";
        assert!(file(map).is_some());
        assert!(bytes(map).is_some());
        assert_eq!(display_path(map), "assets-test/maps/test_small.map");
        assert!(files_in("maps").contains(&map));
        assert!(files_under("").contains(&map));
        assert!(!is_private(&display_path(map)));
        assert_eq!(test_display_path(map), display_path(map));
        assert_eq!(test_display_path("x/new.png"), "assets-test/x/new.png");
    }

    #[test]
    fn story_only_hides_the_test_files_from_this_thread_until_it_returns() {
        let map = "maps/test_small.map";
        let inside = story_only(|| {
            // A nested call leaves them hidden when it returns.
            story_only(|| ());
            (
                file(map),
                bytes(map),
                files_in("maps").contains(&map),
                files_under("").contains(&map),
                display_path(map),
                test_part(TABLES[0]),
                file("data/palette.ron").is_some(),
            )
        });
        assert_eq!(
            inside,
            (
                None,
                None,
                false,
                false,
                "assets/maps/test_small.map".to_owned(),
                None,
                true
            )
        );
        assert!(file(map).is_some());
        // Also after a panic inside it.
        assert!(std::panic::catch_unwind(|| story_only(|| panic!("inside"))).is_err());
        assert!(file(map).is_some());
        // Another thread is never affected.
        let elsewhere = story_only(|| std::thread::spawn(move || file(map).is_some()).join());
        assert!(matches!(elsewhere, Ok(true)));
    }

    /// Ticket 1265: what the release export bakes is every file of
    /// `assets/` (with the content over it), the bytes the game embeds,
    /// and no bought art or test file.
    #[test]
    fn the_pack_layers_are_the_shipped_files_of_assets() {
        let packed = Bundle::new(pack_layers());
        let keymap = "data/keymap.ron";
        assert_eq!(packed.bytes(keymap), install().bytes(keymap));
        let paths = packed.files_under("");
        let assets: Vec<&str> = ASSETS.iter().map(|(path, _)| *path).collect();
        for path in &assets {
            assert!(paths.contains(path), "{path}");
        }
        for path in &paths {
            assert!(!is_private(&packed.display_path(path)), "{path}");
            assert!(packed.test_part(path).is_none(), "{path}");
        }
    }

    /// Ticket 1265: a packed build reads the pack the program laid in, and
    /// one that laid none reads an empty `assets/` (its start then says
    /// why). Only the first pack laid counts.
    #[test]
    fn a_packed_build_reads_the_pack_it_was_given() {
        static FILES: &[(&str, &[u8])] = &[("data/x.ron", b"(a: 1)")];
        static OTHER: &[(&str, &[u8])] = &[("data/y.ron", b"(b: 2)")];
        let empty = Bundle::new(vec![packed_layer()]);
        assert!(PACKED.get().is_some() || empty.files_under("").is_empty());
        use_pack(Layer::table(DISPLAY_ROOT, FILES));
        use_pack(Layer::table(DISPLAY_ROOT, OTHER));
        let read = Bundle::new(vec![packed_layer()]);
        assert_eq!(read.files_under(""), ["data/x.ron"]);
        assert_eq!(read.bytes("data/x.ron"), Some(&b"(a: 1)"[..]));
        assert_eq!(read.display_path("data/x.ron"), "assets/data/x.ron");
    }
}
