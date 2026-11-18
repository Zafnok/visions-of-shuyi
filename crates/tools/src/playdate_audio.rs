//! The Playdate's sounds and music (ticket 1047, ADR-0084), baked by
//! `cargo xtask playdate` into the `.pdx`'s `Source/` folder before `pdc`
//! packs it.
//!
//! The bake is the engine's (`shuyi_cli::export::playdate::audio`, ticket
//! 1203). This module holds what is the game's: every file of every sound cue
//! in `assets/audio/audio.ron` ships, and of the music only
//! `crates/content/playdate-music.txt`'s. Sounds go under `Source/audio/` and
//! music under `Source/music/`, the paths
//! `trpg_content::audio::playdate_sound_path` and `playdate_music_path` give
//! the front end.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use shuyi_cli::export::playdate::audio as bake;
use shuyi_cli::game::GameAudio;
use trpg_content::audio::{self, AudioManifest};

/// The files to bake: each sound file of `manifest` once, and the file of
/// each music cue in `music` (read from `root/music/`), in path order. A cue
/// that isn't in `manifest`, a file that can't be read and two files on one
/// path are errors.
pub fn jobs(
    root: &Path,
    mut manifest: AudioManifest,
    music: &BTreeSet<String>,
) -> Result<Vec<bake::Job>, String> {
    if let Some(cue) = music.iter().find(|c| !manifest.music.contains_key(*c)) {
        return Err(format!("{cue:?} is not a music cue"));
    }
    manifest.music.retain(|cue, _| music.contains(cue));
    bake::jobs(&GameAudio::new(manifest, audio::files(None)), root)
}

/// The baked files in a `Source/` folder, for the build's summary line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Totals {
    /// Sound files and their bytes.
    pub sounds: (usize, u64),
    /// Music files and their bytes.
    pub music: (usize, u64),
}

impl Totals {
    /// The files under `source_dir`'s `audio/` and `music/` folders.
    pub fn on_disk(source_dir: &Path) -> Self {
        Self {
            sounds: files_under(&source_dir.join(audio::SOUND_DIR)),
            music: files_under(&source_dir.join(audio::MUSIC_DIR)),
        }
    }

    /// `audio: 22 sound files 0.3 MB, 20 tracks 16.7 MB`.
    pub fn line(self) -> String {
        let mb = |bytes: u64| {
            // Megabytes to one decimal place: far below f64's precision.
            #[allow(clippy::cast_precision_loss)]
            let mb = bytes as f64 / 1_000_000.0;
            format!("{mb:.1} MB")
        };
        format!(
            "audio: {} sound files {}, {} tracks {}",
            self.sounds.0,
            mb(self.sounds.1),
            self.music.0,
            mb(self.music.1)
        )
    }
}

/// How many files are under `dir`, and their bytes (none if it isn't
/// there).
fn files_under(dir: &Path) -> (usize, u64) {
    let Ok(entries) = fs::read_dir(dir) else {
        return (0, 0);
    };
    entries.flatten().fold((0, 0), |(count, bytes), entry| {
        let path = entry.path();
        if path.is_dir() {
            let (c, b) = files_under(&path);
            (count + c, bytes + b)
        } else {
            let len = entry.metadata().map_or(0, |m| m.len());
            (count + 1, bytes + len)
        }
    })
}

/// Bakes the Playdate's sounds and music for the workspace at `root` into
/// `source_dir`, through the cache in `out` (`target/playdate/`).
///
/// Not covered by mutation testing (`#[mutants::skip]`): it joins the
/// manifest, [`jobs`] and the engine's bake (each tested) over the real files.
#[mutants::skip]
pub fn run(root: &Path, out: &Path, source_dir: &Path) -> Result<(), String> {
    let manifest = audio::load().map_err(|errors| {
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        errors.join("; ")
    })?;
    let music = audio::playdate_music(&manifest)?;
    bake::bake(
        &jobs(root, manifest, &music)?,
        &bake::cache_dir(out),
        source_dir,
    )
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// An empty folder for one test.
    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("xtask-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn write(path: &Path, bytes: &[u8]) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    /// `samples` (interleaved, `channels`) as a 16-bit PCM WAV at 44.1 kHz.
    fn pcm_wav(samples: &[i16], channels: u16) -> Vec<u8> {
        let data = u32::try_from(samples.len() * 2).unwrap();
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&channels.to_le_bytes());
        wav.extend_from_slice(&audio::SAMPLE_RATE.to_le_bytes());
        wav.extend_from_slice(&(audio::SAMPLE_RATE * u32::from(channels) * 2).to_le_bytes());
        wav.extend_from_slice(&(channels * 2).to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data.to_le_bytes());
        for s in samples {
            wav.extend_from_slice(&s.to_le_bytes());
        }
        wav
    }

    /// The real manifest with only the music cue `title`, its file `t.wav`.
    fn one_track() -> (AudioManifest, BTreeSet<String>) {
        let mut manifest = audio::load().unwrap();
        manifest.music.get_mut("title").unwrap().file = "t.wav".to_owned();
        (manifest, BTreeSet::from(["title".to_owned()]))
    }

    /// The real selection: the paths the front end opens, nothing else.
    #[test]
    fn every_sound_file_and_only_the_listed_music() {
        let manifest = audio::load().unwrap();
        let music = audio::playdate_music(&manifest).unwrap();
        let jobs = jobs(&repo(), manifest.clone(), &music).unwrap();
        let outs: BTreeSet<&str> = jobs.iter().map(|j| j.out.as_str()).collect();
        let sounds = manifest.sounds.values().flat_map(|cue| &cue.files);
        let sounds = sounds.map(|file| audio::playdate_sound_path(file));
        let tracks = music.iter().map(|cue| &manifest.music[cue].file);
        let tracks = tracks.map(|file| audio::playdate_music_path(file));
        let expected: BTreeSet<String> = sounds.chain(tracks).map(|p| p + ".wav").collect();
        assert_eq!(
            outs,
            expected.iter().map(String::as_str).collect::<BTreeSet<_>>()
        );
        assert!(outs.contains("audio/sfx/menu_move.wav"));
        assert!(outs.contains("music/new_sunrise_v1.wav"));
        // A track that is in the manifest and not on the list stays out.
        let left = manifest.music.iter().find(|(cue, _)| !music.contains(*cue));
        let (_, track) = left.expect("a track the Playdate doesn't ship");
        assert!(
            !outs.contains(format!("{}.wav", audio::playdate_music_path(&track.file)).as_str())
        );
    }

    #[test]
    fn a_cue_that_is_not_in_the_manifest_is_an_error() {
        let unknown = BTreeSet::from(["nope".to_owned()]);
        assert_eq!(
            jobs(&repo(), audio::load().unwrap(), &unknown).unwrap_err(),
            "\"nope\" is not a music cue"
        );
    }

    #[test]
    fn a_missing_file_is_an_error_naming_it() {
        let root = temp("audio-missing");
        let (manifest, music) = one_track();
        assert_eq!(
            jobs(&root, manifest.clone(), &music).unwrap_err(),
            "music \"title\": music/t.wav: no such file"
        );
        write(&root.join("music/t.wav"), &pcm_wav(&[0], 1));
        let mut manifest = manifest;
        let heal = manifest.sounds.get_mut("heal").unwrap();
        heal.files.push("sfx/nope.wav".to_owned());
        let err = jobs(&root, manifest, &music).unwrap_err();
        assert!(err.contains("sfx/nope.wav: no such file"), "{err}");
        let _ = fs::remove_dir_all(&root);
    }

    /// What ADR-0084 fixes and the front end's build reads, through the
    /// engine's bake: an IMA ADPCM WAV, mono at 44.1 kHz in 1,024-byte blocks,
    /// at its path. Three samples worked by hand: 0 opens the block at step 7,
    /// then 34 is code 7 and -34 code 15 (the old converter's test).
    #[test]
    fn a_track_bakes_to_the_format_and_path_the_build_packs() {
        let root = temp("audio-format");
        let (manifest, music) = one_track();
        // Stereo averages to the same three samples.
        let stereo = pcm_wav(&[0, 0, 30, 38, -34, -34], 2);
        write(&root.join("music/t.wav"), &stereo);
        let jobs = jobs(&root, manifest, &music).unwrap();
        let source = root.join("Source");
        bake::bake(&jobs, &bake::cache_dir(&root), &source).unwrap();
        let wav = fs::read(source.join("music/t.wav")).unwrap();
        let u16_at = |at: usize| u16::from_le_bytes([wav[at], wav[at + 1]]);
        let u32_at = |at: usize| u32::from(u16_at(at)) + (u32::from(u16_at(at + 2)) << 16);
        assert_eq!(wav.len(), 60 + 1024);
        assert_eq!((&wav[..4], &wav[8..16]), (&b"RIFF"[..], &b"WAVEfmt "[..]));
        assert_eq!(u16_at(20), 0x11, "IMA ADPCM");
        assert_eq!(u16_at(22), 1, "mono");
        assert_eq!(u32_at(24), 44_100);
        assert_eq!(u32_at(28), 22_125);
        assert_eq!(u16_at(32), 1024);
        assert_eq!(u16_at(38), 2041);
        assert_eq!(&wav[40..44], b"fact");
        assert_eq!(u32_at(48), 3, "the real length, for a gapless loop");
        assert_eq!(wav[64], 0xf7);
        // The sounds land where the front end opens them, and are counted.
        let totals = Totals::on_disk(&source);
        assert_eq!(totals.sounds, (jobs.len() - 1, totals.sounds.1));
        assert_eq!(totals.music, (1, 60 + 1024));
        let heal = audio::playdate_sound_path("sfx/heal.wav") + ".wav";
        assert!(source.join(heal).is_file());
        let _ = fs::remove_dir_all(&root);
    }

    /// The cache goes by the source's bytes, not its date: a changed file of
    /// the same length is baked again, in the same second or not.
    #[test]
    fn a_changed_source_is_baked_again() {
        let root = temp("audio-cache");
        let (manifest, music) = one_track();
        let source = root.join("Source");
        let cache = bake::cache_dir(&root);
        let baked = |samples: &[i16]| {
            write(&root.join("music/t.wav"), &pcm_wav(samples, 1));
            let jobs = jobs(&root, manifest.clone(), &music).unwrap();
            bake::bake(&jobs, &cache, &source).unwrap();
            fs::read(source.join("music/t.wav")).unwrap()
        };
        let first = baked(&[0, 34, -34]);
        let second = baked(&[0, -34, 34]);
        assert_ne!(first, second);
        assert_eq!(baked(&[0, 34, -34]), first);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn the_totals_count_files_in_nested_folders() {
        let dir = temp("audio-totals");
        write(&dir.join("audio/b.wav"), &[0; 5]);
        write(&dir.join("audio/sfx/a.wav"), &[0; 3]);
        write(&dir.join("audio/sfx/deep/c.wav"), &[0; 7]);
        write(&dir.join("music/t.wav"), &[0; 2]);
        let totals = Totals::on_disk(&dir);
        assert_eq!(totals.sounds, (3, 15));
        assert_eq!(totals.music, (1, 2));
        assert_eq!(Totals::on_disk(&dir.join("nothing")), Totals::default());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_summary_counts_each_kind() {
        let totals = Totals {
            sounds: (22, 340_000),
            music: (20, 16_740_000),
        };
        assert_eq!(
            totals.line(),
            "audio: 22 sound files 0.3 MB, 20 tracks 16.7 MB"
        );
    }
}
