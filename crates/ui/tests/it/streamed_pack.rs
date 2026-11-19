//! The start over a pack the web page streams in (ticket 1298, engine
//! ADR-0128 §3): the release's files, baked into a pack and read through a
//! source that answers `NotYet` for the bytes not in yet. The start waits
//! behind its loading card until the whole pack is in, then loads the
//! title. The first load reads every folder, so no folder is laid first
//! (`trpg_tools::game_tools::pack_bake`).

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use shuyi_assets::pack::{self, Content, Index, PackInfo, Source, SourceError};
use shuyi_platform::headless::HeadlessPlatform;
use shuyi_platform::{App, Millis};
use trpg_content::bundle::{Bundle, DISPLAY_ROOT, pack_layers, reading, skip_without_the_game};
use trpg_ui::screens::TitleScreen;
use trpg_ui::start::{Startup, start};

/// The loading card's title, as the binary's window has it.
const TITLE: &str = "Visions of Shuyi";

/// A pack whose first `got` bytes have arrived, as the page's stream hands
/// them over.
struct Streaming {
    bytes: &'static [u8],
    got: AtomicUsize,
}

impl Source for Streaming {
    fn read(&self, at: u64, into: &mut [u8]) -> Result<(), SourceError> {
        let at = usize::try_from(at).unwrap_or(usize::MAX);
        let end = at.saturating_add(into.len());
        if end > self.got.load(Ordering::SeqCst) {
            return Err(SourceError::NotYet);
        }
        into.copy_from_slice(&self.bytes[at..end]);
        Ok(())
    }

    fn arrived(&self, at: u64, len: u64) -> bool {
        let got = u64::try_from(self.got.load(Ordering::SeqCst)).unwrap_or(u64::MAX);
        at.saturating_add(len) <= got
    }
}

/// The files a release export bakes (`pack_layers`), as one pack.
fn release_pack() -> &'static [u8] {
    let baked = Bundle::new(pack_layers());
    let files: BTreeMap<String, Content> = baked
        .files_under("")
        .into_iter()
        .map(|path| {
            let bytes = baked
                .bytes(path)
                .unwrap_or_else(|| panic!("{path} unreadable"));
            (path.to_owned(), Content::Raw(bytes.into_owned()))
        })
        .collect();
    let info = PackInfo::new("t", None, shuyi_core::ContentHash::default());
    pack::write(&info, &files).leak()
}

/// Acceptance (1298): with every byte but the pack's last one in, the start
/// stays on its card frame after frame; once the last byte arrives, the
/// next frames load the game, at the title.
#[test]
fn the_start_waits_for_a_streamed_pack_then_starts() {
    if skip_without_the_game() {
        return;
    }
    let packed = release_pack();
    let index = Index::read(packed, "t").unwrap_or_else(|e| panic!("{e:?}"));
    let source: &'static Streaming = Box::leak(Box::new(Streaming {
        bytes: packed,
        got: AtomicUsize::new(packed.len() - 1),
    }));
    let streamed = Box::leak(Box::new(Bundle::new(vec![
        index.into_layer(DISPLAY_ROOT, source),
    ])));
    reading(streamed, || {
        let mut app = App::new(start(TITLE, Ok(Startup::Title)));
        let mut platform = HeadlessPlatform::new();
        for frame in 0..10 {
            let _ = app.frame(&mut platform, Millis(frame * 16));
            assert!(
                app.is_loading(),
                "frame {frame}: started before the pack was in"
            );
        }
        source.got.store(packed.len(), Ordering::SeqCst);
        let _ = app.frame(&mut platform, Millis(160));
        let _ = app.frame(&mut platform, Millis(176));
        assert_eq!(app.errors(), None);
        let game = app.into_game().unwrap_or_else(|| panic!("not loaded"));
        assert!(game.screen::<TitleScreen>().is_some(), "not at the title");
    });
}
