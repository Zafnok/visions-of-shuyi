//! Plays each frame's audio on the Playdate (ticket 1047, ADR-0084): the
//! sound requests and the music commands `trpg_ui::Game::frame` returns,
//! as the PC's player does them (ADR-0026, ADR-0037), through the SDK's
//! sample players and file players behind [`Speaker`].
//!
//! The sounds are in the `.pdx` (`audio/`), converted by `cargo xtask
//! playdate`; every sound file is loaded into memory when the game starts
//! and plays on one of [`VOICES`] sample players. The music is in the
//! `.pdx` too (`music/`), each track streamed from its file by a file
//! player, opened when the music asks to load it and closed when it
//! stops: at most two at once, the one fading out and the next. A track
//! the Playdate does not ship (not in `playdate-music.txt`) plays as
//! silence, with a warning.
//!
//! The music clock ([`Audio::music_playing`]) is the file player's own
//! position in its file, so a screen that keeps time with the music
//! follows what is heard.
//!
//! Only `src/pd/sound.rs` calls the SDK; everything here is plain code,
//! tested with a fake speaker.

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use trpg_content::audio::{self, AudioManifest};
use trpg_ui::audio::{AudioRequest, MusicCommand, splitmix64};

/// How many sounds can play at once: one sample player each. A sound
/// asked for while all play stops one of them, each in turn.
pub const VOICES: usize = 8;

/// What playing audio needs from the SDK.
pub trait Speaker {
    /// A sound loaded into memory.
    type Sample;
    /// A music file opened for streaming.
    type Track;

    /// Loads the sound at `path` in the `.pdx` (no extension); `None` if
    /// there is none.
    fn load_sample(&mut self, path: &str) -> Option<Self::Sample>;
    /// Plays `sample` once at `volume` (0–1) on sample player `voice`
    /// (below [`VOICES`]), stopping what it played.
    fn play_sample(&mut self, voice: usize, sample: &Self::Sample, volume: f32);
    /// Whether sample player `voice` is playing.
    fn voice_playing(&mut self, voice: usize) -> bool;
    /// Opens the music file at `path` in the `.pdx` (no extension) for
    /// streaming; `None` if there is none.
    fn open_track(&mut self, path: &str) -> Option<Self::Track>;
    /// Plays `track` from its start at `volume`, looped or once; false if
    /// it could not start.
    fn start_track(&mut self, track: &mut Self::Track, volume: f32, looped: bool) -> bool;
    /// Changes the volume of `track`.
    fn set_track_volume(&mut self, track: &mut Self::Track, volume: f32);
    /// Pauses or resumes `track` (the bench build's minutes without music).
    fn pause_track(&mut self, track: &mut Self::Track, paused: bool);
    /// Seconds into its file `track` is, while it plays; `None` once it has
    /// stopped (a track played once has ended).
    fn track_position(&mut self, track: &mut Self::Track) -> Option<f32>;
    /// Whether `track` ran out of data to play since it started.
    fn track_underran(&mut self, track: &mut Self::Track) -> bool;
    /// Stops `track` and closes its file.
    fn close_track(&mut self, track: Self::Track);
}

/// The audio player: the loaded sounds and the open music tracks.
pub struct Audio<S: Speaker> {
    /// Every sound file loaded, by its path in the manifest.
    samples: BTreeMap<String, S::Sample>,
    /// Each sound cue's volume (0–1) and its variants' files, which may be
    /// missing from `samples` (warned when loading).
    sounds: BTreeMap<String, (f32, Vec<String>)>,
    /// Each music cue's file, volume (0–1) and whether it loops.
    music: BTreeMap<String, (String, f32, bool)>,
    /// The tracks loading, playing or fading, by cue.
    tracks: BTreeMap<String, Track<S>>,
    /// The order tracks started in: the newest is the one heard.
    starts: u32,
    /// The voice a sound takes when all play: they are taken in turn, so
    /// it is roughly the one that started longest ago.
    oldest: usize,
    /// Picks sound variants (footsteps vary each step); never the game's
    /// simulation RNG (ADR-0019).
    variants: u64,
    /// The player's music and sound volumes (Options), 0–1.
    music_volume: f32,
    sound_volume: f32,
    /// Whether the music is paused (the bench build).
    paused: bool,
    /// Problems to log.
    warnings: Vec<String>,
}

/// One music track.
struct Track<S: Speaker> {
    /// The open file; `None` if the `.pdx` has no such track.
    file: Option<S::Track>,
    /// The cue's own volume, 0–1.
    volume: f32,
    /// The fade's gain, 0–1.
    gain: f32,
    looped: bool,
    /// When it started, in [`Audio::starts`]' order; `None` before.
    started: Option<u32>,
}

impl<S: Speaker> Audio<S> {
    /// Loads every sound file of `manifest` from the `.pdx`. A file that is
    /// not there is left out with a warning; its cue plays its other
    /// variants, or nothing. `seed` seeds the variant picker.
    pub fn load(speaker: &mut S, manifest: &AudioManifest, seed: u64) -> Self {
        let mut samples = BTreeMap::new();
        let mut warnings = Vec::new();
        let mut sounds = BTreeMap::new();
        for (cue, sound) in &manifest.sounds {
            for file in &sound.files {
                if samples.contains_key(file) {
                    continue;
                }
                let path = audio::playdate_sound_path(file);
                match speaker.load_sample(&path) {
                    Some(sample) => {
                        samples.insert(file.clone(), sample);
                    }
                    None => warnings.push(format!("sound \"{cue}\": {path} is not in the .pdx")),
                }
            }
            sounds.insert(cue.clone(), (percent(sound.volume), sound.files.clone()));
        }
        let music = manifest
            .music
            .iter()
            .map(|(cue, m)| (cue.clone(), (m.file.clone(), percent(m.volume), m.looped)))
            .collect();
        Self {
            samples,
            sounds,
            music,
            tracks: BTreeMap::new(),
            starts: 0,
            oldest: 0,
            variants: seed,
            music_volume: 1.0,
            sound_volume: 1.0,
            paused: false,
            warnings,
        }
    }

    /// How many sound files are loaded.
    pub fn samples(&self) -> usize {
        self.samples.len()
    }

    /// Sets the player's volumes (Options, 0–1): every sound and track
    /// plays at its own volume times these. A change to the music's
    /// reaches the tracks playing.
    pub fn set_volumes(&mut self, speaker: &mut S, music: f32, sound: f32) {
        self.sound_volume = sound;
        if (music - self.music_volume).abs() <= f32::EPSILON {
            return;
        }
        self.music_volume = music;
        for track in self.tracks.values_mut() {
            if let (Some(_), Some(file)) = (track.started, &mut track.file) {
                speaker.set_track_volume(file, track.volume * track.gain * music);
            }
        }
    }

    /// Plays one frame's sound `requests` (the music requests are already
    /// `music`'s commands) and applies the `music` commands.
    pub fn play(&mut self, speaker: &mut S, requests: &[AudioRequest], music: &[MusicCommand]) {
        for request in requests {
            if let AudioRequest::PlaySound { cue, volume } = request {
                self.play_sound(speaker, cue, *volume);
            }
        }
        for command in music {
            self.music(speaker, command);
        }
    }

    fn play_sound(&mut self, speaker: &mut S, cue: &str, volume: f32) {
        let Some((own, files)) = self.sounds.get(cue) else {
            self.warnings.push(format!("unknown sound cue \"{cue}\""));
            return;
        };
        let volume = own * volume * self.sound_volume;
        if files.is_empty() || volume <= 0.0 {
            return;
        }
        self.variants = self.variants.wrapping_add(1);
        let len = u64::try_from(files.len()).unwrap_or(1);
        let pick = usize::try_from(splitmix64(self.variants) % len).unwrap_or(0);
        let Some(sample) = files.get(pick).and_then(|file| self.samples.get(file)) else {
            return; // Not in the .pdx (warned when loading).
        };
        let voice = (0..VOICES)
            .find(|&v| !speaker.voice_playing(v))
            .unwrap_or(self.oldest);
        if voice == self.oldest {
            self.oldest = (voice + 1) % VOICES;
        }
        speaker.play_sample(voice, sample, volume);
    }

    fn music(&mut self, speaker: &mut S, command: &MusicCommand) {
        match command {
            MusicCommand::Load { cue } => {
                self.stop_track(speaker, cue);
                let Some((file, volume, looped)) = self.music.get(cue) else {
                    self.warnings.push(format!("unknown music cue \"{cue}\""));
                    return;
                };
                let path = audio::playdate_music_path(file);
                let file = speaker.open_track(&path);
                if file.is_none() {
                    self.warnings
                        .push(format!("music \"{cue}\": {path} is not in the .pdx"));
                }
                let track = Track {
                    file,
                    volume: *volume,
                    gain: 1.0,
                    looped: *looped,
                    started: None,
                };
                self.tracks.insert(cue.clone(), track);
            }
            MusicCommand::Start { cue } => {
                let Some(track) = self.tracks.get_mut(cue) else {
                    return;
                };
                track.gain = 1.0;
                let Some(file) = &mut track.file else {
                    return;
                };
                if speaker.start_track(file, track.volume * self.music_volume, track.looped) {
                    self.starts += 1;
                    track.started = Some(self.starts);
                    if self.paused {
                        speaker.pause_track(file, true);
                    }
                } else {
                    self.warnings
                        .push(format!("music \"{cue}\": the SDK could not start it"));
                }
            }
            MusicCommand::Gain { cue, gain } => {
                if let Some(track) = self.tracks.get_mut(cue) {
                    track.gain = *gain;
                    if let (Some(_), Some(file)) = (track.started, &mut track.file) {
                        let volume = track.volume * track.gain * self.music_volume;
                        speaker.set_track_volume(file, volume);
                    }
                }
            }
            MusicCommand::Stop { cue } => self.stop_track(speaker, cue),
        }
    }

    /// Stops and closes `cue`'s track, if it has one.
    fn stop_track(&mut self, speaker: &mut S, cue: &str) {
        if let Some(Track {
            file: Some(file), ..
        }) = self.tracks.remove(cue)
        {
            speaker.close_track(file);
        }
    }

    /// Stops all music (the game quit).
    pub fn stop_all(&mut self, speaker: &mut S) {
        let cues: Vec<String> = self.tracks.keys().cloned().collect();
        for cue in cues {
            self.stop_track(speaker, &cue);
        }
    }

    /// The music heard now: its cue and how many seconds into its file it
    /// is, for `Game::set_music_playing`. `None` in silence: nothing
    /// asked for, a track the `.pdx` lacks, or one played once that has
    /// ended. During a fade it is the track fading out; the next one
    /// starts when the fade ends.
    pub fn music_playing(&mut self, speaker: &mut S) -> Option<(&str, f64)> {
        let (cue, track) = self
            .tracks
            .iter_mut()
            .filter(|(_, track)| track.started.is_some())
            .max_by_key(|(_, track)| track.started)?;
        let position = speaker.track_position(track.file.as_mut()?)?;
        Some((cue.as_str(), f64::from(position)))
    }

    /// Pauses or resumes the music (the bench build's minutes without it).
    pub fn pause_music(&mut self, speaker: &mut S, paused: bool) {
        if self.paused == paused {
            return;
        }
        self.paused = paused;
        for track in self.tracks.values_mut() {
            if let (Some(_), Some(file)) = (track.started, &mut track.file) {
                speaker.pause_track(file, paused);
            }
        }
    }

    /// The minute report's line about the music: the track heard and
    /// whether it ran out of data since it started.
    pub fn report(&mut self, speaker: &mut S) -> String {
        let heard = self
            .tracks
            .iter_mut()
            .filter(|(_, track)| track.started.is_some())
            .max_by_key(|(_, track)| track.started);
        let Some((
            cue,
            Track {
                file: Some(file), ..
            },
        )) = heard
        else {
            return String::from("Music: none");
        };
        let underran = if speaker.track_underran(file) {
            "it ran out of data"
        } else {
            "no gaps"
        };
        format!("Music: {cue}, {underran}")
    }

    /// Problems found since the last call, for the log.
    pub fn take_warnings(&mut self) -> Vec<String> {
        core::mem::take(&mut self.warnings)
    }
}

/// A manifest volume (percent) as a 0–1 factor.
fn percent(volume: u8) -> f32 {
    f32::from(volume) / 100.0
}

#[cfg(test)]
mod tests {
    use trpg_content::audio::{CreditRef, MusicCue, SoundCue};

    use super::*;

    /// What the fake speaker was told.
    #[derive(Debug, Clone, PartialEq)]
    enum Call {
        Load(String),
        Play(usize, String, f32),
        Open(String),
        Start(String, f32, bool),
        Volume(String, f32),
        Pause(String, bool),
        Close(String),
    }

    /// A speaker that records its calls. Its sample players are busy
    /// while `busy` says, its files are the paths in `files`.
    #[derive(Default)]
    struct Fake {
        calls: Vec<Call>,
        files: Vec<&'static str>,
        busy: [bool; VOICES],
        /// Where each open track is, by path; `None` once it has ended.
        positions: BTreeMap<String, Option<f32>>,
        refuse_start: bool,
        underran: bool,
    }

    impl Speaker for Fake {
        type Sample = String;
        type Track = String;

        fn load_sample(&mut self, path: &str) -> Option<String> {
            self.calls.push(Call::Load(path.into()));
            self.files.contains(&path).then(|| path.into())
        }

        fn play_sample(&mut self, voice: usize, sample: &String, volume: f32) {
            self.busy[voice] = true;
            self.calls.push(Call::Play(voice, sample.clone(), volume));
        }

        fn voice_playing(&mut self, voice: usize) -> bool {
            self.busy[voice]
        }

        fn open_track(&mut self, path: &str) -> Option<String> {
            self.calls.push(Call::Open(path.into()));
            self.files.contains(&path).then(|| {
                self.positions.insert(path.into(), Some(0.0));
                path.into()
            })
        }

        fn start_track(&mut self, track: &mut String, volume: f32, looped: bool) -> bool {
            self.calls.push(Call::Start(track.clone(), volume, looped));
            !self.refuse_start
        }

        fn set_track_volume(&mut self, track: &mut String, volume: f32) {
            self.calls.push(Call::Volume(track.clone(), volume));
        }

        fn pause_track(&mut self, track: &mut String, paused: bool) {
            self.calls.push(Call::Pause(track.clone(), paused));
        }

        fn track_position(&mut self, track: &mut String) -> Option<f32> {
            self.positions.get(track.as_str()).copied().flatten()
        }

        fn track_underran(&mut self, _track: &mut String) -> bool {
            self.underran
        }

        fn close_track(&mut self, track: String) {
            self.calls.push(Call::Close(track));
        }
    }

    /// The test manifest: `click` (volume 50), `step` with two variants,
    /// one not in the `.pdx`, `blip` and `bloop` sharing a file, and the
    /// tracks `theme` (looped, volume 80), `jingle` (once) and `gone` (not
    /// in the `.pdx`).
    fn manifest() -> AudioManifest {
        let mut m = AudioManifest::default();
        let sound = |files: &[&str], volume| {
            SoundCue::new(
                files.iter().map(|f| (*f).to_owned()).collect(),
                volume,
                CreditRef::Own,
            )
        };
        m.sounds
            .insert("click".into(), sound(&["sfx/click.wav"], 50));
        m.sounds.insert(
            "step".into(),
            sound(&["sfx/step_a.ogg", "sfx/step_b.ogg"], 100),
        );
        m.sounds
            .insert("blip".into(), sound(&["sfx/blip.wav"], 100));
        m.sounds
            .insert("bloop".into(), sound(&["sfx/blip.wav"], 40));
        let track = |file: &str, volume, looped| MusicCue {
            file: file.into(),
            volume,
            looped,
            length_ms: 10_000,
            credit: CreditRef::Own,
        };
        m.music.insert("theme".into(), track("theme.ogg", 80, true));
        m.music
            .insert("jingle".into(), track("jingle.ogg", 100, false));
        m.music.insert("gone".into(), track("gone.ogg", 100, true));
        m
    }

    fn fake() -> Fake {
        Fake {
            files: [
                "audio/sfx/click",
                "audio/sfx/step_a",
                "audio/sfx/blip",
                "music/theme",
                "music/jingle",
            ]
            .into(),
            ..Fake::default()
        }
    }

    fn loaded() -> (Fake, Audio<Fake>) {
        let mut speaker = fake();
        let mut audio = Audio::load(&mut speaker, &manifest(), 1);
        audio.take_warnings();
        speaker.calls.clear();
        (speaker, audio)
    }

    fn sound(cue: &str, volume: f32) -> AudioRequest {
        AudioRequest::PlaySound {
            cue: cue.into(),
            volume,
        }
    }

    fn load(cue: &str) -> MusicCommand {
        MusicCommand::Load { cue: cue.into() }
    }

    fn start(cue: &str) -> MusicCommand {
        MusicCommand::Start { cue: cue.into() }
    }

    fn stop(cue: &str) -> MusicCommand {
        MusicCommand::Stop { cue: cue.into() }
    }

    fn gain(cue: &str, gain: f32) -> MusicCommand {
        MusicCommand::Gain {
            cue: cue.into(),
            gain,
        }
    }

    fn frame(
        speaker: &mut Fake,
        audio: &mut Audio<Fake>,
        requests: &[AudioRequest],
        music: &[MusicCommand],
    ) -> Vec<Call> {
        speaker.calls.clear();
        audio.play(speaker, requests, music);
        core::mem::take(&mut speaker.calls)
    }

    #[test]
    fn every_sound_file_is_loaded_once_at_the_start() {
        let mut speaker = fake();
        let mut audio = Audio::load(&mut speaker, &manifest(), 1);
        let loads: Vec<Call> = speaker.calls.clone();
        assert_eq!(
            loads,
            [
                Call::Load("audio/sfx/blip".into()),
                Call::Load("audio/sfx/click".into()),
                Call::Load("audio/sfx/step_a".into()),
                Call::Load("audio/sfx/step_b".into()),
            ]
        );
        assert_eq!(audio.samples(), 3);
        assert_eq!(
            audio.take_warnings(),
            ["sound \"step\": audio/sfx/step_b is not in the .pdx"]
        );
        assert_eq!(audio.take_warnings(), [] as [std::string::String; 0]);
    }

    #[test]
    fn a_sound_plays_at_its_volume_times_the_request_and_the_option() {
        let (mut speaker, mut audio) = loaded();
        assert_eq!(
            frame(&mut speaker, &mut audio, &[sound("click", 1.0)], &[]),
            [Call::Play(0, "audio/sfx/click".into(), 0.5)]
        );
        audio.set_volumes(&mut speaker, 1.0, 0.5);
        assert_eq!(
            frame(&mut speaker, &mut audio, &[sound("bloop", 0.5)], &[]),
            [Call::Play(1, "audio/sfx/blip".into(), 0.1)]
        );
        // At no volume nothing plays; an unknown cue is a warning.
        audio.set_volumes(&mut speaker, 1.0, 0.0);
        assert_eq!(
            frame(&mut speaker, &mut audio, &[sound("click", 1.0)], &[]),
            []
        );
        assert_eq!(
            frame(&mut speaker, &mut audio, &[sound("nope", 1.0)], &[]),
            []
        );
        assert_eq!(audio.take_warnings(), ["unknown sound cue \"nope\""]);
    }

    #[test]
    fn a_variant_missing_from_the_pdx_plays_nothing() {
        let (mut speaker, mut audio) = loaded();
        let mut played = Vec::new();
        for _ in 0..40 {
            played.extend(frame(&mut speaker, &mut audio, &[sound("step", 1.0)], &[]));
            speaker.busy = [false; VOICES];
        }
        // Both variants are picked: the one in the .pdx plays.
        assert!(played.len() > 5 && played.len() < 35, "{}", played.len());
        assert!(
            played
                .iter()
                .all(|c| *c == Call::Play(0, "audio/sfx/step_a".into(), 1.0))
        );
        assert_eq!(audio.take_warnings(), [] as [std::string::String; 0]);
    }

    #[test]
    fn a_sound_takes_a_free_voice_else_the_oldest() {
        let (mut speaker, mut audio) = loaded();
        let voices = |speaker: &mut Fake, audio: &mut Audio<Fake>, n| {
            let mut voices = Vec::new();
            for _ in 0..n {
                for call in frame(speaker, audio, &[sound("blip", 1.0)], &[]) {
                    if let Call::Play(voice, ..) = call {
                        voices.push(voice);
                    }
                }
            }
            voices
        };
        assert_eq!(
            voices(&mut speaker, &mut audio, 10),
            [0, 1, 2, 3, 4, 5, 6, 7, 0, 1]
        );
        // Voice 5 finished: the next sound takes it, then the oldest.
        speaker.busy[5] = false;
        assert_eq!(voices(&mut speaker, &mut audio, 3), [5, 2, 3]);
    }

    #[test]
    fn a_track_is_opened_started_faded_and_closed() {
        let (mut speaker, mut audio) = loaded();
        assert_eq!(
            frame(
                &mut speaker,
                &mut audio,
                &[],
                &[load("theme"), start("theme")]
            ),
            [
                Call::Open("music/theme".into()),
                Call::Start("music/theme".into(), 0.8, true)
            ]
        );
        assert_eq!(audio.music_playing(&mut speaker), Some(("theme", 0.0)));
        speaker.positions.insert("music/theme".into(), Some(2.5));
        assert_eq!(audio.music_playing(&mut speaker), Some(("theme", 2.5)));
        // A switch: the next opens while this one fades.
        assert_eq!(
            frame(
                &mut speaker,
                &mut audio,
                &[],
                &[load("jingle"), gain("theme", 0.5)]
            ),
            [
                Call::Open("music/jingle".into()),
                Call::Volume("music/theme".into(), 0.4)
            ]
        );
        // Still the fading track until the next starts.
        assert_eq!(audio.music_playing(&mut speaker), Some(("theme", 2.5)));
        assert_eq!(
            frame(
                &mut speaker,
                &mut audio,
                &[],
                &[stop("theme"), start("jingle")]
            ),
            [
                Call::Close("music/theme".into()),
                Call::Start("music/jingle".into(), 1.0, false)
            ]
        );
        assert_eq!(audio.music_playing(&mut speaker), Some(("jingle", 0.0)));
        // Played once and ended: silence.
        speaker.positions.insert("music/jingle".into(), None);
        assert_eq!(audio.music_playing(&mut speaker), None);
    }

    #[test]
    fn a_track_the_pdx_lacks_is_silence_with_a_warning() {
        let (mut speaker, mut audio) = loaded();
        assert_eq!(
            frame(
                &mut speaker,
                &mut audio,
                &[],
                &[load("gone"), start("gone"), gain("gone", 0.5)]
            ),
            [Call::Open("music/gone".into())]
        );
        assert_eq!(audio.music_playing(&mut speaker), None);
        assert_eq!(
            frame(
                &mut speaker,
                &mut audio,
                &[],
                &[stop("gone"), load("nope"), start("nope")]
            ),
            []
        );
        assert_eq!(
            audio.take_warnings(),
            [
                "music \"gone\": music/gone is not in the .pdx",
                "unknown music cue \"nope\""
            ]
        );
        speaker.refuse_start = true;
        frame(
            &mut speaker,
            &mut audio,
            &[],
            &[load("theme"), start("theme")],
        );
        assert_eq!(audio.music_playing(&mut speaker), None);
        assert_eq!(
            audio.take_warnings(),
            ["music \"theme\": the SDK could not start it"]
        );
    }

    #[test]
    fn the_music_option_reaches_the_track_playing() {
        let (mut speaker, mut audio) = loaded();
        audio.set_volumes(&mut speaker, 0.5, 1.0);
        frame(&mut speaker, &mut audio, &[], &[load("theme")]);
        // Not started yet: nothing to change.
        speaker.calls.clear();
        audio.set_volumes(&mut speaker, 0.25, 1.0);
        assert_eq!(speaker.calls, []);
        assert_eq!(
            frame(
                &mut speaker,
                &mut audio,
                &[],
                &[start("theme"), gain("theme", 0.5)]
            ),
            [
                Call::Start("music/theme".into(), 0.2, true),
                Call::Volume("music/theme".into(), 0.1)
            ]
        );
        speaker.calls.clear();
        audio.set_volumes(&mut speaker, 0.5, 1.0);
        audio.set_volumes(&mut speaker, 0.5, 0.5);
        // The cue's 0.8 times the fade's 0.5 times the option's 0.5.
        assert_eq!(speaker.calls, [Call::Volume("music/theme".into(), 0.2)]);
    }

    /// Two tracks started without a stop between (the music never does
    /// that, but the clock must still say which sounds): the newer one.
    #[test]
    fn the_track_heard_is_the_one_started_last() {
        let (mut speaker, mut audio) = loaded();
        frame(
            &mut speaker,
            &mut audio,
            &[],
            &[load("theme"), start("theme")],
        );
        frame(
            &mut speaker,
            &mut audio,
            &[],
            &[load("jingle"), start("jingle")],
        );
        assert_eq!(audio.music_playing(&mut speaker), Some(("jingle", 0.0)));
        assert_eq!(audio.report(&mut speaker), "Music: jingle, no gaps");
        frame(
            &mut speaker,
            &mut audio,
            &[],
            &[stop("theme"), start("theme")],
        );
        frame(
            &mut speaker,
            &mut audio,
            &[],
            &[load("theme"), start("theme")],
        );
        assert_eq!(audio.music_playing(&mut speaker), Some(("theme", 0.0)));
    }

    #[test]
    fn loading_a_track_again_restarts_it_and_stop_all_closes_everything() {
        let (mut speaker, mut audio) = loaded();
        frame(
            &mut speaker,
            &mut audio,
            &[],
            &[load("theme"), start("theme"), load("jingle")],
        );
        assert_eq!(
            frame(&mut speaker, &mut audio, &[], &[load("theme")]),
            [
                Call::Close("music/theme".into()),
                Call::Open("music/theme".into())
            ]
        );
        // Not started again yet: silence.
        assert_eq!(audio.music_playing(&mut speaker), None);
        speaker.calls.clear();
        audio.stop_all(&mut speaker);
        assert_eq!(
            speaker.calls,
            [
                Call::Close("music/jingle".into()),
                Call::Close("music/theme".into())
            ]
        );
        assert_eq!(audio.music_playing(&mut speaker), None);
    }

    #[test]
    fn the_bench_pauses_the_music_heard_and_any_that_starts() {
        let (mut speaker, mut audio) = loaded();
        frame(
            &mut speaker,
            &mut audio,
            &[],
            &[load("theme"), start("theme"), load("jingle")],
        );
        speaker.calls.clear();
        audio.pause_music(&mut speaker, true);
        audio.pause_music(&mut speaker, true);
        assert_eq!(speaker.calls, [Call::Pause("music/theme".into(), true)]);
        assert_eq!(
            frame(
                &mut speaker,
                &mut audio,
                &[],
                &[stop("theme"), start("jingle")]
            ),
            [
                Call::Close("music/theme".into()),
                Call::Start("music/jingle".into(), 1.0, false),
                Call::Pause("music/jingle".into(), true)
            ]
        );
        speaker.calls.clear();
        audio.pause_music(&mut speaker, false);
        assert_eq!(speaker.calls, [Call::Pause("music/jingle".into(), false)]);
    }

    #[test]
    fn the_report_names_the_track_heard_and_its_gaps() {
        let (mut speaker, mut audio) = loaded();
        assert_eq!(audio.report(&mut speaker), "Music: none");
        frame(
            &mut speaker,
            &mut audio,
            &[],
            &[load("theme"), start("theme")],
        );
        assert_eq!(audio.report(&mut speaker), "Music: theme, no gaps");
        speaker.underran = true;
        assert_eq!(
            audio.report(&mut speaker),
            "Music: theme, it ran out of data"
        );
    }
}
