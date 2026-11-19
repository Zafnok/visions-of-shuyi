//! The game's palettes and scenes reloading as their files change, in a
//! build with debug tools (ticket 1209): a palette and the test scene laid
//! over the game's files as folders read from disk are, changed through
//! the headless backend's `change_file`, as `cargo shuyi run --watch`
//! changes them.

use std::collections::BTreeMap;

use shuyi_assets::bundle::Layer;
use trpg_content::bundle::{self, Bundle};
use trpg_content::palette::PALETTE_PATH;
use trpg_ui::color::UiColor;
use trpg_ui::game::Game;
use trpg_ui::harness::{Core, FRAME_DT, Harness};
use trpg_ui::hot::{HotContent, reloads};
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::DialogueScreen;
use trpg_ui::{Ctx, ShuyiCtx};

/// The test scene's file (`assets-test/dialogue/test.dlg`).
const SCENE: &str = "dialogue/test.dlg";

/// The test scene's first narration, as the file says it.
const FIRST: &str = "The rain had not stopped for three days.";

/// The text of the game's file at `path`.
fn file(path: &str) -> String {
    bundle::file(path)
        .unwrap_or_else(|| panic!("no {path}"))
        .into_owned()
}

/// A folder read from disk, shown as `name`, holding the game's file at
/// `path`.
fn folder(name: &str, path: &str) -> Layer {
    let files = BTreeMap::from([(path.to_owned(), file(path).into_bytes())]);
    Layer::owned(name.to_owned(), files).unwrap_or_else(|e| panic!("{e:?}"))
}

/// The game's files with `assets/`'s palette and the test scene over them,
/// as folders read from disk.
struct Disk {
    bundle: &'static Bundle,
    assets: Layer,
    test: Layer,
}

impl Disk {
    fn new() -> Self {
        let assets = folder(bundle::DISPLAY_ROOT, PALETTE_PATH);
        let test = folder(bundle::TEST_DISPLAY_ROOT, SCENE);
        let mut layers = vec![assets, test];
        layers.extend(bundle::layers());
        let bundle = Box::leak(Box::new(Bundle::new(layers)));
        Self {
            bundle,
            assets,
            test,
        }
    }

    /// Runs `f` with the game reading these files.
    fn read<T>(&self, f: impl FnOnce() -> T) -> T {
        bundle::reading(self.bundle, f)
    }
}

/// The game at the title with the module installed.
fn title() -> Game {
    let ctx = Ctx::embedded().unwrap_or_else(|e| panic!("{e}"));
    let game = trpg_ui::game::start(ctx.with_layout(RIGHT_HANDED));
    HotContent::install(game).unwrap_or_else(|e| panic!("{e}"))
}

/// The game at the title, with its debug tools on.
fn game() -> Harness {
    Harness::from_game(title())
}

/// The debug menu's "Play test scene", its first narration fully shown,
/// and that text.
fn play_test_scene(h: &mut Harness) -> String {
    h.keys("F2 Down Down f f");
    let Some(screen) = h.game().screen::<DialogueScreen>() else {
        panic!("no scene is playing: {:?}", h.screens());
    };
    screen.view(h.game().ctx()).text.lines.join(" ")
}

/// The palette file with `text` coloured `hex`.
fn palette_with_text(hex: &str) -> String {
    let line = |l: &str| {
        if l.trim_start().starts_with("\"text\":") {
            format!("    \"text\": \"{hex}\",")
        } else {
            l.to_owned()
        }
    };
    file(PALETTE_PATH)
        .lines()
        .map(line)
        .collect::<Vec<_>>()
        .join("\n")
}

/// The text colour the game paints with, as its channels.
fn text_color(h: &Core) -> (u8, u8, u8) {
    let c = h.ctx().palette.get(UiColor::Text);
    (c.r, c.g, c.b)
}

#[test]
fn the_palettes_and_the_scenes_reload_and_nothing_else() {
    assert!(reloads(PALETTE_PATH));
    assert!(reloads("data/palettes/moonlit.ron"));
    assert!(reloads(SCENE));
    assert!(!reloads("dialogue/notes.txt"));
    assert!(!reloads("dialogue/old/test.dlg.bak"));
    assert!(!reloads("maps/test_small.map"));
    assert!(!reloads("battles/test_small.ron"));
    assert!(!reloads("data/palettes"));
}

#[test]
fn a_changed_palette_is_painted_with_on_the_next_frame() {
    let disk = Disk::new();
    disk.read(|| {
        let mut h = game();
        let names: Vec<_> = h.game().modules().iter().map(|m| m.name).collect();
        assert!(names.contains(&"hot_content"), "{names:?}");
        assert_ne!(text_color(&h), (0x12, 0x34, 0x56));
        h.change_file(
            disk.assets,
            PALETTE_PATH,
            palette_with_text("#123456").into_bytes(),
        );
        h.wait(FRAME_DT);
        assert_eq!(text_color(&h), (0x12, 0x34, 0x56));
        assert_eq!(h.ctx().reload_errors(), [] as [String; 0]);
    });
}

#[test]
fn a_changed_scene_plays_the_next_time_it_is_played() {
    let disk = Disk::new();
    disk.read(|| {
        let mut h = game();
        let reworded = file(SCENE).replace(FIRST, "The rain stopped at last.");
        h.change_file(disk.test, SCENE, reworded.into_bytes());
        h.wait(FRAME_DT);
        assert_eq!(play_test_scene(&mut h), "The rain stopped at last.");
    });
}

#[test]
fn a_broken_file_keeps_the_old_scene_and_names_its_file_and_line() {
    let disk = Disk::new();
    disk.read(|| {
        let mut h = game();
        let good = file(SCENE);
        // A block that isn't closed.
        let broken = good.replacen("@endif\n", "", 1);
        h.change_file(disk.test, SCENE, broken.into_bytes());
        h.wait(FRAME_DT);
        let errors = h.ctx().reload_errors().to_vec();
        let named = errors
            .iter()
            .any(|e| e.starts_with("assets-test/dialogue/test.dlg:"));
        assert!(named, "{errors:?}");
        assert_eq!(h.platform().reload_errors(), errors, "drawn over the frame");
        assert_eq!(play_test_scene(&mut h), FIRST, "the old scene plays");
        // Mended, the error goes.
        h.change_file(disk.test, SCENE, good.into_bytes());
        h.wait(FRAME_DT);
        assert_eq!(h.ctx().reload_errors(), [] as [String; 0]);
    });
}

#[test]
fn without_debug_tools_the_game_never_asks_what_changed() {
    let disk = Disk::new();
    disk.read(|| {
        // As in a build without them: the Harness turns them on.
        let mut h = Core::new(title().with_debug_screens(false));
        let before = text_color(&h);
        h.change_file(
            disk.assets,
            PALETTE_PATH,
            palette_with_text("#123456").into_bytes(),
        );
        h.wait(FRAME_DT);
        assert_eq!(h.platform().changed_asks(), 0);
        assert_eq!(text_color(&h), before);
    });
}
