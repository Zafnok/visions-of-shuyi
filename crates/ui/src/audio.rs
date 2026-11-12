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
//! assert_eq!(audio.pending()[1], AudioRequest::play_music("title", None));
//! ```

use trpg_content::Variant;

pub use shuyi_ui::audio::{
    MusicClock, MusicCommand, MusicState, MusicSwitch, pick_from_pool, splitmix64,
};

/// Something a screen wants heard ([`shuyi_ui::audio::AudioRequest`]).
/// Cue ids are the ones in `assets/audio/audio.ron`.
pub type AudioRequest = shuyi_ui::audio::AudioRequest<Variant>;

/// The requests screens made this frame
/// ([`shuyi_ui::audio::AudioQueue`]).
pub type AudioQueue = shuyi_ui::audio::AudioQueue<Variant>;

pub use shuyi_ui::audio::{AudioConfig, MenuSound};

/// The sound groups and a frame's gains (engine #318, ADR-0091 there):
/// what a platform crate plays each cue at.
pub use shuyi_ui::audio::{Gain, Gains, GroupGains, SoundGroups, groups};

/// The map cursor's tick, once per tile it moves (the manifest plays it
/// quieter than `menu_move`).
pub const CURSOR_MOVE: &str = "cursor_move";

/// This game's menu sounds (`docs/design/audio.md`): the same three cues
/// mean the same thing on every screen. Confirm on something that can't
/// be chosen should get its own warning tone (Nick, ticket 0427); until
/// then it is `menu_cancel`.
pub const AUDIO: AudioConfig = AudioConfig {
    move_cue: "menu_move",
    select_cue: "menu_select",
    cancel_cue: "menu_cancel",
    denied_cue: "menu_cancel",
    cursor_cue: CURSOR_MOVE,
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
        q.cursor_tick();
        assert_eq!(q.take()[0].cue(), Some(CURSOR_MOVE));
    }

    /// A made-up manifest: a `pool` of three looped tracks, each 10 s.
    fn manifest() -> trpg_content::AudioManifest {
        use trpg_content::{CreditRef, MusicCue};
        let mut m = trpg_content::AudioManifest::default();
        let cues = ["one", "two", "three"];
        for cue in cues {
            let track = MusicCue {
                file: format!("{cue}.ogg"),
                volume: 100,
                looped: true,
                length_ms: 10_000,
                credit: CreditRef::Own,
                group: None,
            };
            m.music.insert(cue.to_owned(), track);
        }
        m.pools
            .insert("pool".to_owned(), cues.map(str::to_owned).to_vec());
        m
    }

    #[test]
    fn a_pool_pick_is_a_cue_of_the_pool() {
        let m = manifest();
        let pool = &m.pools["pool"];
        for seed in [0, 1, 2, u64::MAX] {
            let cue = pick_from_pool(&m, "pool", seed).unwrap();
            assert!(pool.iter().any(|c| c == cue), "{cue}");
        }
        assert_eq!(pick_from_pool(&m, "no_such_pool", 0), None);
    }

    /// Ticket 0809: the `victory` pool is the 13 Win Jingle versions, each
    /// played once, and two seeds can pick two of them. `defeat` loops. A
    /// test of the game's own cue table (ADR-0118).
    #[test]
    fn the_victory_pool_is_thirteen_stings_and_seeds_pick_different_ones() {
        if trpg_content::bundle::skip_without_the_game() {
            return;
        }
        let m = trpg_content::load_embedded().unwrap().audio;
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

    /// A looped track's clock is the time played, round its length.
    #[test]
    fn a_looped_track_has_a_clock() {
        let m = manifest();
        let clock = MusicClock::from_elapsed(&m, "two", 25.5);
        let clock = clock.unwrap();
        assert!((clock.length - 10.0).abs() < 1e-3, "{clock:?}");
        assert!((clock.position - 5.5).abs() < 1e-3, "{clock:?}");
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(32))]

        /// Over enough seeds, every track in the pool comes up.
        #[test]
        fn every_track_in_a_pool_can_be_picked(start in proptest::prelude::any::<u64>()) {
            let m = manifest();
            let pool = &m.pools["pool"];
            let picked: std::collections::BTreeSet<&str> = (0..200)
                .filter_map(|i| pick_from_pool(&m, "pool", start.wrapping_add(i)))
                .collect();
            proptest::prop_assert_eq!(picked.len(), pool.len());
        }
    }
}
