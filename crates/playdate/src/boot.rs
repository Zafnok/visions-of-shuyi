//! The start as the front end runs it (ticket 1086): one step per update
//! callback, the loading card on the screen, so that the Playdate OS sees
//! the run loop come back between them. It ends a game whose update
//! callback has not returned for 10 seconds, and the whole start in one
//! callback took longer than that on the device.
//!
//! The steps are the game's data ([`start::data`]), the player's settings
//! and saves ([`start::with_storage`]), the sounds (as many in a callback
//! as [`Host::out_of_time`] allows) and the title. Plain code: `src/pd/
//! entry.rs` runs it on the SDK, and the tests and `chapter_1.rs` on fakes.

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;

use trpg_ui::{Ctx, Game, Storage};

use crate::audio::{Audio, Speaker};
use crate::start;

/// The load report's name of the step that reads the player's files.
pub const STORAGE: &str = "the player's settings and saves";
/// The load report's name of the step that loads the sounds.
pub const SOUNDS: &str = "the sounds";
/// The load report's name of the step that starts the title.
pub const TITLE: &str = "the title";

/// What the start needs from the platform.
pub trait Host {
    /// The SDK's sound players.
    type Speaker: Speaker;
    /// A seed for the title's music and the sounds' variants.
    fn seed(&mut self) -> u64;
    /// The player's settings and saves.
    fn storage(&mut self) -> Box<dyn Storage>;
    /// The sound players, once, before the sounds load.
    fn speaker(&mut self) -> Self::Speaker;
    /// Notes in the load report that the step `name` began.
    fn mark(&mut self, name: &str);
    /// Logs a problem found while starting.
    fn warn(&mut self, warning: &str);
    /// Whether this update callback has run long enough: the sounds load
    /// until it has, then go on in the next.
    fn out_of_time(&mut self) -> bool;
}

/// The step the next update callback runs.
pub enum Boot<S: Speaker> {
    /// The game's data.
    Data,
    /// The player's settings and saves, over the data.
    Storage(Box<Ctx>),
    /// The sounds, some of them loaded.
    Sounds(Box<Loaded<S>>),
    /// The title, everything loaded.
    Title(Box<Loaded<S>>),
}

/// The game's context and its sounds, while they load.
pub struct Loaded<S: Speaker> {
    ctx: Ctx,
    audio: Audio<S>,
    speaker: S,
}

/// The game at its title, with its sounds and the players they play on.
pub struct Started<S: Speaker> {
    /// The game, its title on the top of its stack.
    pub game: Game,
    /// The sounds, loaded, and the music.
    pub audio: Audio<S>,
    /// The sound players.
    pub speaker: S,
}

/// What a step leaves for the next callback.
pub enum Next<S: Speaker> {
    /// The step to run next.
    Step(Boot<S>),
    /// The game, started.
    Started(Box<Started<S>>),
}

impl<S: Speaker> Boot<S> {
    /// Runs this step with `host`.
    ///
    /// # Errors
    ///
    /// If the game's data or the Playdate's palette fails to load.
    pub fn step<H: Host<Speaker = S>>(self, host: &mut H) -> Result<Next<S>, String> {
        Ok(Next::Step(match self {
            Self::Data => {
                let ctx = start::data(host.seed())?;
                host.mark(STORAGE);
                Self::Storage(Box::new(ctx))
            }
            Self::Storage(ctx) => {
                let mut ctx = start::with_storage(*ctx, host.storage())?;
                for warning in ctx.take_warnings() {
                    host.warn(&warning);
                }
                host.mark(SOUNDS);
                let audio = Audio::new(&ctx.game.content.audio, host.seed());
                let speaker = host.speaker();
                Self::Sounds(Box::new(Loaded {
                    ctx,
                    audio,
                    speaker,
                }))
            }
            Self::Sounds(mut loaded) => {
                let Loaded { audio, speaker, .. } = &mut *loaded;
                while audio.load_next(speaker) {
                    if host.out_of_time() {
                        return Ok(Next::Step(Self::Sounds(loaded)));
                    }
                }
                for warning in audio.take_warnings() {
                    host.warn(&format!("audio: {warning}"));
                }
                host.mark(TITLE);
                Self::Title(loaded)
            }
            Self::Title(loaded) => {
                let Loaded {
                    ctx,
                    audio,
                    speaker,
                } = *loaded;
                let game = trpg_ui::game::start(ctx);
                return Ok(Next::Started(Box::new(Started {
                    game,
                    audio,
                    speaker,
                })));
            }
        }))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use alloc::vec::Vec;

    use trpg_ui::MemoryStorage;

    use super::*;

    /// Sound players that play nothing: every file is there.
    #[derive(Debug, Default)]
    pub(crate) struct Silent;

    impl Speaker for Silent {
        type Sample = ();
        type Track = ();

        fn load_sample(&mut self, _path: &str) -> Option<()> {
            Some(())
        }

        fn play_sample(&mut self, _voice: usize, _sample: &(), _volume: f32) {}

        fn voice_playing(&mut self, _voice: usize) -> bool {
            false
        }

        fn open_track(&mut self, _path: &str) -> Option<()> {
            Some(())
        }

        fn start_track(&mut self, _track: &mut (), _volume: f32, _looped: bool) -> bool {
            true
        }

        fn set_track_volume(&mut self, _track: &mut (), _volume: f32) {}

        fn pause_track(&mut self, _track: &mut (), _paused: bool) {}

        fn track_position(&mut self, _track: &mut ()) -> Option<f32> {
            None
        }

        fn track_underran(&mut self, _track: &mut ()) -> bool {
            false
        }

        fn close_track(&mut self, _track: ()) {}
    }

    /// A host with the player's files in memory and no sound, noting the
    /// report's marks; each callback is out of time after one sound when
    /// `slow`.
    #[derive(Debug, Default)]
    pub(crate) struct TestHost {
        pub(crate) slow: bool,
        pub(crate) marks: Vec<String>,
        pub(crate) warnings: Vec<String>,
    }

    impl Host for TestHost {
        type Speaker = Silent;

        fn seed(&mut self) -> u64 {
            1
        }

        fn storage(&mut self) -> Box<dyn Storage> {
            Box::new(MemoryStorage::new())
        }

        fn speaker(&mut self) -> Silent {
            Silent
        }

        fn mark(&mut self, name: &str) {
            self.marks.push(name.into());
        }

        fn warn(&mut self, warning: &str) {
            self.warnings.push(warning.into());
        }

        fn out_of_time(&mut self) -> bool {
            self.slow
        }
    }

    /// Runs the start with `host` to the title, one step a call: the game,
    /// and the name of each step run (`Sounds` once per callback).
    pub(crate) fn boot(host: &mut TestHost) -> (Started<Silent>, Vec<&'static str>) {
        let mut boot = Boot::Data;
        let mut steps = Vec::new();
        loop {
            steps.push(match boot {
                Boot::Data => "data",
                Boot::Storage(_) => "storage",
                Boot::Sounds(_) => "sounds",
                Boot::Title(_) => "title",
            });
            match boot.step(host).unwrap_or_else(|e| panic!("{e}")) {
                Next::Step(next) => boot = next,
                Next::Started(started) => return (*started, steps),
            }
        }
    }

    /// The sound files of the game's manifest, each once.
    fn sound_files(ctx: &Ctx) -> usize {
        let mut files: Vec<&String> = ctx
            .game
            .content
            .audio
            .sounds
            .values()
            .flat_map(|sound| &sound.files)
            .collect();
        files.sort_unstable();
        files.dedup();
        files.len()
    }

    /// Data, storage, sounds and the title, one callback each, the
    /// report's marks between them; then the title is up with every sound.
    #[test]
    fn the_steps_run_in_order_one_per_call_to_the_title() {
        let mut host = TestHost::default();
        let (started, steps) = boot(&mut host);
        assert_eq!(steps, ["data", "storage", "sounds", "title"]);
        assert_eq!(host.marks, [STORAGE, SOUNDS, TITLE]);
        assert_eq!(host.warnings, Vec::<String>::new());
        assert_eq!(started.game.top_screen(), Some("title"));
        let files = sound_files(started.game.ctx());
        assert!(files > 1);
        assert_eq!(started.audio.samples(), files);
    }

    /// A callback that runs out of time stops the sounds; the next one
    /// goes on with them.
    #[test]
    fn the_sounds_go_on_in_the_next_callback_when_one_runs_out_of_time() {
        let mut host = TestHost {
            slow: true,
            ..TestHost::default()
        };
        let (started, steps) = boot(&mut host);
        let files = sound_files(started.game.ctx());
        // One sound a callback, then one that finds none left.
        let mut expected = Vec::from(["data", "storage"]);
        expected.extend(core::iter::repeat_n("sounds", files + 1));
        expected.push("title");
        assert_eq!(steps, expected);
        assert_eq!(host.marks, [STORAGE, SOUNDS, TITLE]);
        assert_eq!(started.audio.samples(), files);
    }
}
