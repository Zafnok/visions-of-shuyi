//! What the game gives the platform shell: the window, where saves are kept
//! and the voice clips.

use std::future::Future;
use std::path::PathBuf;
use std::pin::pin;
use std::task::{Context, Poll, Waker};

use shuyi_app::read_voice_manifest;
use shuyi_app::storage::{data_dir, web_prefix};
use trpg_content::voice::{MANIFEST_FILE, clip_file};
use trpg_content::{LineId, Variant};
use trpg_ui::{Ctx, ShuyiCtx};

use super::{APP, STORAGE_NAME, WINDOW, config, look};

/// Runs a future that never waits on anything external.
fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = pin!(future);
    let mut cx = Context::from_waker(Waker::noop());
    loop {
        if let Poll::Ready(v) = future.as_mut().poll(&mut cx) {
            return v;
        }
    }
}

#[test]
fn the_window_has_an_icon() {
    let conf = shuyi_app::window_conf(&WINDOW);
    assert!(conf.icon.is_some());
    assert_eq!(WINDOW.icon, APP.window_icon);
    assert_eq!(conf.window_title, "Visions of Shuyi");
}

/// Acceptance (1199): the name and the storage name players' saves depend
/// on, as literals, and the version `release.yml` tags.
#[test]
fn who_the_game_is() {
    assert_eq!(APP.name, "Visions of Shuyi");
    assert_eq!(APP.storage, "tactical-rpg");
    assert_eq!(APP.version, env!("CARGO_PKG_VERSION"));
}

/// Acceptance (1110): saves written before the engine's platform shell
/// are found after it, in the same folder and under the same prefix.
#[test]
fn saves_stay_where_they_were() {
    assert_eq!(config().storage_name, STORAGE_NAME);
    assert_eq!(STORAGE_NAME, "tactical-rpg");
    assert_eq!(web_prefix(STORAGE_NAME), "tactical-rpg/");
    let dir = data_dir(STORAGE_NAME);
    assert!(
        dir.as_ref().is_some_and(|d| d.ends_with("tactical-rpg")),
        "{dir:?}"
    );
    #[cfg(target_os = "windows")]
    assert_eq!(
        dir,
        std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("tactical-rpg"))
    );
    #[cfg(not(target_os = "windows"))]
    let _: Option<PathBuf> = dir;
}

/// The three lines of `assets-test/dialogue/test.dlg` that have a test clip
/// (`crates/app/tests/voice/`), and a fourth that doesn't.
const LINES: [&str; 4] = [
    "test_21833604",
    "test_7de7b254",
    "test_01439969",
    "test_5da5d147",
];

/// The test voice folder: laid out as `voice/` is beside the game.
const TEST_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/voice/en");

fn line(i: usize) -> LineId {
    LineId::new(LINES[i])
}

/// Acceptance: with no `voice/` folder the game starts and logs nothing
/// about voices.
#[test]
fn without_a_voice_folder_there_are_no_voices_and_no_warnings() {
    let nowhere = tempfile::tempdir().unwrap();
    let dir = format!("{}/voice/en", nowhere.path().display());
    assert_eq!(block_on(read_voice_manifest(&dir, MANIFEST_FILE)), Ok(None));
    let mut ctx = Ctx::embedded().unwrap();
    assert_eq!(ctx.take_warnings(), [] as [std::string::String; 0]);
    assert!(!ctx.has_voice(&line(0)));
}

/// The test folder is read as the real one is: its manifest is valid,
/// every clip may be played, and every file is where the player looks for
/// it, an OGG the length reader understands.
#[test]
fn the_test_voice_folder_gives_the_game_its_three_clips() {
    let manifest = block_on(read_voice_manifest(TEST_DIR, MANIFEST_FILE));
    let manifest = manifest.unwrap().unwrap();
    let mut ctx = Ctx::embedded().unwrap();
    ctx.set_voice_manifest(&manifest);
    assert_eq!(ctx.take_warnings(), [] as [&str; 0]);
    for (i, id) in LINES.iter().enumerate().take(3) {
        assert!(ctx.has_voice(&line(i)), "{id}");
        let file = format!("{TEST_DIR}/{}", clip_file(&line(i), Variant::None));
        let bytes = std::fs::read(&file).unwrap_or_default();
        let length = trpg_content::audio::ogg_length_ms(&bytes);
        let short = length.is_some_and(|ms| (300..2000).contains(&ms));
        assert!(short, "{file}: {length:?}");
    }
    assert!(!ctx.has_voice(&line(3)));
    // The cast beside it is valid too.
    let cast = std::fs::read_to_string(format!("{TEST_DIR}/cast.ron")).unwrap();
    let characters = Some(&ctx.game.content.characters);
    let cast = trpg_content::voice::cast_from_source("cast.ron", &cast, characters);
    assert_eq!(cast.map(|c| c.voices.len()), Ok(3));
}

/// The platform shell reads the game's font and images from its content.
#[test]
fn the_font_and_images_are_the_contents() {
    let ctx = Ctx::embedded().unwrap();
    let config = config();
    let content = &ctx.game.content;
    let look = look(&ctx.game);
    assert!(std::ptr::eq(look.font, &raw const content.font));
    assert!(std::ptr::eq(look.images, &raw const content.images));
    assert!((config.files)(config.font_png).is_some());
}
