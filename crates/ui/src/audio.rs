//! Audio as data (ADR-0026): screens ask for sounds and music through
//! [`Ctx::audio`](shuyi_ui::Ctx::audio); [`Game`](crate::Game) hands the
//! requests, and the [`MusicState`]'s commands, to `app` in each
//! [`FrameOutput`](crate::FrameOutput). Nothing here plays anything, so
//! screens stay testable in the headless `Harness`. The mechanism is Shuyi
//! Engine's (`shuyi_ui::audio`, ticket 1109), with this game's voice clip
//! [`Variant`]s; which cues the menu sounds are is this game's
//! ([`AUDIO`]).
//!
//! Voice clips (ADR-0046) are asked for by dialogue line id, through
//! [`Ctx::play_voice`](shuyi_ui::Ctx::play_voice), which knows which lines
//! have a clip that may be played.
//!
//! ```
//! # use trpg_ui::audio::{AUDIO, AudioQueue, AudioRequest, MenuSound};
//! let mut audio = AudioQueue::new(AUDIO);
//! audio.menu(MenuSound::Move);
//! audio.play_music("title");
//! assert_eq!(audio.pending()[1], AudioRequest::PlayMusic { cue: "title".into() });
//! ```

use trpg_content::Variant;

pub use shuyi_ui::audio::{MusicClock, MusicCommand, MusicState, pick_from_pool, splitmix64};

/// Something a screen wants heard ([`shuyi_ui::audio::AudioRequest`]).
/// Cue ids are the ones in `assets/audio/audio.ron`.
pub type AudioRequest = shuyi_ui::audio::AudioRequest<Variant>;

/// The requests screens made this frame
/// ([`shuyi_ui::audio::AudioQueue`]).
pub type AudioQueue = shuyi_ui::audio::AudioQueue<Variant>;

pub use shuyi_ui::audio::{AudioConfig, MenuSound};

/// The map cursor's tick, once per tile it moves (the manifest plays it
/// quieter than `menu_move`).
pub const CURSOR_MOVE: &str = "cursor_move";

/// This game's menu sounds (`docs/design/audio.md`): the same three cues
/// mean the same thing on every screen. Confirm on something that can't
/// be chosen should get its own warning tone (Nick, ticket 0427); until
/// then it is `menu_cancel`.
pub const AUDIO: AudioConfig = AudioConfig {
    menu_move: "menu_move",
    menu_select: "menu_select",
    menu_cancel: "menu_cancel",
    menu_denied: "menu_cancel",
    cursor_move: CURSOR_MOVE,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_sounds_are_their_cues() {
        let mut q = AudioQueue::new(AUDIO);
        for sound in [
            MenuSound::Move,
            MenuSound::Select,
            MenuSound::Cancel,
            MenuSound::Denied,
        ] {
            q.menu(sound);
        }
        let cues: Vec<_> = q
            .take()
            .iter()
            .map(|r| r.cue().map(str::to_owned))
            .collect();
        let cue = |c: &str| Some(c.to_owned());
        assert_eq!(
            cues,
            [
                cue("menu_move"),
                cue("menu_select"),
                cue("menu_cancel"),
                cue("menu_cancel")
            ]
        );
        q.cursor_move();
        assert_eq!(q.take()[0].cue(), Some(CURSOR_MOVE));
    }

    /// The embedded audio manifest (with the real `skirmish` pool).
    fn manifest() -> trpg_content::AudioManifest {
        trpg_content::load_embedded().unwrap().audio
    }

    #[test]
    fn a_pool_pick_is_a_cue_of_the_skirmish_pool() {
        let m = manifest();
        let pool = &m.pools["skirmish"];
        for seed in [0, 1, 2, u64::MAX] {
            let cue = pick_from_pool(&m, "skirmish", seed).unwrap();
            assert!(pool.iter().any(|c| c == cue), "{cue}");
        }
    }

    /// Ticket 0809: the `victory` pool is the 13 Win Jingle versions, each
    /// played once, and two seeds can pick two of them. `defeat` loops.
    #[test]
    fn the_victory_pool_is_thirteen_stings_and_seeds_pick_different_ones() {
        let m = manifest();
        let pool = &m.pools["victory"];
        assert_eq!(pool.len(), 13);
        for cue in pool {
            let track = &m.music[cue];
            assert!(track.file.starts_with("win_jingle_"), "{cue}");
            assert!(!track.looped, "{cue}");
        }
        let picks: std::collections::BTreeSet<&str> = (0..8)
            .filter_map(|seed| pick_from_pool(&m, "victory", seed))
            .collect();
        assert!(picks.len() > 1, "{picks:?}");
        assert!(picks.iter().all(|p| pool.iter().any(|c| c == p)));
        assert!(m.music["defeat"].looped);
    }

    #[test]
    fn the_embedded_title_track_has_a_clock() {
        let m = manifest();
        let clock = MusicClock::from_elapsed(&m, "title", 140.0);
        let clock = clock.unwrap();
        assert!((clock.length - 133.743).abs() < 1e-3, "{clock:?}");
        assert!((clock.position - 6.257).abs() < 1e-3, "{clock:?}");
    }

    proptest::proptest! {
        // Each case loads the embedded content; a few dozen are plenty.
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(32))]

        /// Over enough seeds, every track in the pool comes up.
        #[test]
        fn every_track_in_a_pool_can_be_picked(start in proptest::prelude::any::<u64>()) {
            let m = manifest();
            let pool = &m.pools["skirmish"];
            let picked: std::collections::BTreeSet<&str> = (0..200)
                .filter_map(|i| pick_from_pool(&m, "skirmish", start.wrapping_add(i)))
                .collect();
            proptest::prop_assert_eq!(picked.len(), pool.len());
        }
    }
}
