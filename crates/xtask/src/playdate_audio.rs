//! The Playdate's sounds and music (ticket 1047, ADR-0084), converted by
//! `cargo xtask playdate` into the `.pdx`'s `Source/` folder before `pdc`
//! packs it.
//!
//! Every file of every sound cue in `assets/audio/audio.ron`, and the music
//! the Playdate ships (`crates/content/playdate-music.txt`), is decoded
//! (OGG Vorbis with `lewton`, WAV with `hound`), mixed down to mono and
//! written as an IMA ADPCM WAV at its own 44.1 kHz: sounds under
//! `Source/audio/`, music under `Source/music/`, at the paths
//! `trpg_content::audio::playdate_sound_path` and `playdate_music_path`
//! give the front end. `pdc` turns each into a `.pda`.
//!
//! Decoding the music takes seconds, so each converted file is kept in
//! `target/playdate/audio-v<FORMAT>/` and made again only when its source
//! is newer; [`FORMAT`] changes with the conversion, so a new one never
//! reuses an old file.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use trpg_content::audio::{self, AudioManifest, SAMPLE_RATE};

/// The conversion's version: the cache folder is named after it, so a
/// change to what is written must change it.
pub const FORMAT: u32 = 1;

/// Bytes in one ADPCM block. Both tools the SDK's docs name (ffmpeg's
/// `adpcm_ima_wav` and Audacity, through libsndfile) write 1,024-byte
/// blocks for mono at 44.1 kHz, so it is what `pdc` is known to read.
pub const BLOCK_BYTES: usize = 1024;

/// Samples in one mono block: the one in its header, then two a byte.
pub const BLOCK_SAMPLES: usize = (BLOCK_BYTES - 4) * 2 + 1;

/// One file to convert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    /// The file it is made from.
    pub source: PathBuf,
    /// Where it goes under `Source/`, with `.wav`: `audio/sfx/heal.wav`.
    pub out: String,
}

/// The files to convert: each sound file of `manifest` (under
/// `root/assets/audio/`) once, and the file of each music cue in `music`
/// (under `root/music/`), in path order. Two files that would land on the
/// same path are an error.
pub fn jobs(
    root: &Path,
    manifest: &AudioManifest,
    music: &BTreeSet<String>,
) -> Result<Vec<Job>, String> {
    let mut jobs: BTreeMap<String, Job> = BTreeMap::new();
    let sounds = manifest.sounds.values().flat_map(|cue| &cue.files);
    let sounds = sounds.map(|file| {
        let source = root.join("assets").join(audio::SOUND_DIR).join(file);
        (source, audio::playdate_sound_path(file))
    });
    let mut tracks = Vec::new();
    for cue in music {
        let track = manifest
            .music
            .get(cue)
            .ok_or_else(|| format!("{cue:?} is not a music cue"))?;
        let source = root.join(audio::MUSIC_DIR).join(&track.file);
        tracks.push((source, audio::playdate_music_path(&track.file)));
    }
    for (source, path) in sounds.chain(tracks) {
        let out = format!("{path}.wav");
        match jobs.get(&out) {
            Some(job) if job.source != source => {
                return Err(format!(
                    "{} and {} would both be {out} on the Playdate",
                    job.source.display(),
                    source.display()
                ));
            }
            Some(_) => {}
            None => {
                jobs.insert(out.clone(), Job { source, out });
            }
        }
    }
    Ok(jobs.into_values().collect())
}

/// The mono samples of the audio file `bytes` named `name` (`.ogg` or
/// `.wav`), at [`SAMPLE_RATE`]: a stereo file's channels are averaged.
pub fn decode(name: &str, bytes: &[u8]) -> Result<Vec<i16>, String> {
    let is_wav = Path::new(name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("wav"));
    let (channels, rate, samples) = if is_wav {
        decode_wav(bytes)
    } else {
        decode_ogg(bytes)
    }
    .map_err(|e| format!("{name}: {e}"))?;
    if rate != SAMPLE_RATE {
        return Err(format!("{name}: {rate} Hz; must be {SAMPLE_RATE} Hz"));
    }
    Ok(mono(&samples, channels))
}

/// An OGG Vorbis file's channels, sample rate and interleaved samples.
fn decode_ogg(bytes: &[u8]) -> Result<(u16, u32, Vec<i16>), String> {
    let mut reader =
        lewton::inside_ogg::OggStreamReader::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let channels = u16::from(reader.ident_hdr.audio_channels);
    let rate = reader.ident_hdr.audio_sample_rate;
    let mut samples = Vec::new();
    while let Some(packet) = reader.read_dec_packet_itl().map_err(|e| e.to_string())? {
        samples.extend(packet);
    }
    Ok((channels, rate, samples))
}

/// A PCM (16-bit) or float (32-bit) WAV file's channels, sample rate and
/// interleaved samples.
fn decode_wav(bytes: &[u8]) -> Result<(u16, u32, Vec<i16>), String> {
    let reader = hound::WavReader::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
    let spec = reader.spec();
    let samples: Result<Vec<i16>, hound::Error> = match (spec.sample_format, spec.bits_per_sample) {
        (hound::SampleFormat::Int, 16) => reader.into_samples::<i16>().collect(),
        (hound::SampleFormat::Float, 32) => reader
            .into_samples::<f32>()
            .map(|s| s.map(float_to_i16))
            .collect(),
        (format, bits) => {
            return Err(format!(
                "{bits}-bit {format:?} samples; must be 16-bit PCM or 32-bit float"
            ));
        }
    };
    Ok((
        spec.channels,
        spec.sample_rate,
        samples.map_err(|e| e.to_string())?,
    ))
}

/// A float sample (−1 to 1) as a 16-bit one, clamped.
fn float_to_i16(sample: f32) -> i16 {
    // Clamped into the i16 range first, so the cast never truncates.
    #[allow(clippy::cast_possible_truncation)]
    let sample = (sample * 32_767.0).round().clamp(-32_768.0, 32_767.0) as i16;
    sample
}

/// Interleaved `samples` of `channels` channels averaged into one.
fn mono(samples: &[i16], channels: u16) -> Vec<i16> {
    let channels = usize::from(channels.max(1));
    samples
        .chunks(channels)
        .map(|frame| {
            let sum: i32 = frame.iter().map(|&s| i32::from(s)).sum();
            // The average of i16s is an i16.
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let mean = (sum / frame.len() as i32) as i16;
            mean
        })
        .collect()
}

/// IMA ADPCM's step sizes.
const STEPS: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408, 449,
    494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066, 2272,
    2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630, 9493,
    10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794, 32767,
];

/// How each code moves the step index.
const INDEX_MOVES: [i32; 8] = [-1, -1, -1, -1, 2, 4, 6, 8];

/// The IMA ADPCM coder's state: the last sample decoded and the step index.
#[derive(Debug, Clone, Copy, Default)]
struct Coder {
    predictor: i32,
    index: i32,
}

impl Coder {
    /// The step for the current index.
    fn step(self) -> i32 {
        STEPS[usize::try_from(self.index).unwrap_or(0)]
    }

    /// The 4-bit code for `sample`, moving the state as a decoder would.
    fn encode(&mut self, sample: i16) -> u8 {
        let mut diff = i32::from(sample) - self.predictor;
        let mut code = 0;
        if diff < 0 {
            code = 8;
            diff = -diff;
        }
        let mut step = self.step();
        for bit in [4, 2, 1] {
            if diff >= step {
                code |= bit;
                diff -= step;
            }
            step >>= 1;
        }
        self.decode(code);
        code
    }

    /// Moves the state by `code` and returns the sample it stands for.
    fn decode(&mut self, code: u8) -> i16 {
        let step = self.step();
        let mut delta = step >> 3;
        if code & 4 != 0 {
            delta += step;
        }
        if code & 2 != 0 {
            delta += step >> 1;
        }
        if code & 1 != 0 {
            delta += step >> 2;
        }
        if code & 8 != 0 {
            delta = -delta;
        }
        self.predictor = (self.predictor + delta).clamp(-32_768, 32_767);
        self.index = (self.index + INDEX_MOVES[usize::from(code & 7)]).clamp(0, 88);
        // Clamped to the i16 range above.
        #[allow(clippy::cast_possible_truncation)]
        let sample = self.predictor as i16;
        sample
    }
}

/// `samples` (mono, [`SAMPLE_RATE`]) as an IMA ADPCM WAV file: blocks of
/// [`BLOCK_BYTES`], the last padded with silence, and a `fact` chunk with
/// the real number of samples.
pub fn adpcm_wav(samples: &[i16]) -> Vec<u8> {
    let blocks = samples.len().div_ceil(BLOCK_SAMPLES).max(1);
    let mut data = Vec::with_capacity(blocks * BLOCK_BYTES);
    let mut coder = Coder::default();
    for block in 0..blocks {
        let start = block * BLOCK_SAMPLES;
        let sample = |i: usize| samples.get(start + i).copied().unwrap_or(0);
        // The header: the block's first sample as is, and the step index.
        let first = sample(0);
        coder.predictor = i32::from(first);
        data.extend_from_slice(&first.to_le_bytes());
        data.push(u8::try_from(coder.index).unwrap_or(0));
        data.push(0);
        for pair in 0..(BLOCK_BYTES - 4) {
            let low = coder.encode(sample(1 + pair * 2));
            let high = coder.encode(sample(2 + pair * 2));
            // Two 4-bit codes: the first in the low half of the byte.
            data.push(low + (high << 4));
        }
    }
    let u32_of = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
    let block_bytes = u16::try_from(BLOCK_BYTES).unwrap_or(u16::MAX);
    let block_samples = u16::try_from(BLOCK_SAMPLES).unwrap_or(u16::MAX);
    // A whole number of blocks a second, rounded down, as ffmpeg writes it.
    let bytes_per_second = SAMPLE_RATE * u32::from(block_bytes) / u32::from(block_samples);
    let mut wav = Vec::with_capacity(60 + data.len());
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&u32_of(52 + data.len()).to_le_bytes());
    wav.extend_from_slice(b"WAVEfmt ");
    wav.extend_from_slice(&20u32.to_le_bytes());
    wav.extend_from_slice(&0x11u16.to_le_bytes()); // IMA ADPCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    wav.extend_from_slice(&bytes_per_second.to_le_bytes());
    wav.extend_from_slice(&block_bytes.to_le_bytes());
    wav.extend_from_slice(&4u16.to_le_bytes()); // bits a sample
    wav.extend_from_slice(&2u16.to_le_bytes()); // the extra bytes below
    wav.extend_from_slice(&block_samples.to_le_bytes());
    wav.extend_from_slice(b"fact");
    wav.extend_from_slice(&4u32.to_le_bytes());
    wav.extend_from_slice(&u32_of(samples.len()).to_le_bytes());
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&u32_of(data.len()).to_le_bytes());
    wav.extend_from_slice(&data);
    wav
}

/// The converted files in a `Source/` folder, for the build's summary line.
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

/// The cache of converted files, `target/playdate/audio-v<FORMAT>/`.
pub fn cache_dir(out: &Path) -> PathBuf {
    out.join(format!("audio-v{FORMAT}"))
}

/// Whether `cached` was made after `source` changed, so it can be used.
fn fresh(cached: &Path, source: &Path) -> bool {
    let modified = |path: &Path| fs::metadata(path).and_then(|m| m.modified()).ok();
    match (modified(cached), modified(source)) {
        (Some(cached), Some(source)) => cached >= source,
        _ => false,
    }
}

/// Converts `jobs` into `source_dir` (the `.pdx`'s `Source/`), through the
/// cache in `cache`.
pub fn convert(jobs: &[Job], cache: &Path, source_dir: &Path) -> Result<(), String> {
    for job in jobs {
        let cached = cache.join(&job.out);
        if !fresh(&cached, &job.source) {
            let bytes =
                fs::read(&job.source).map_err(|e| format!("read {}: {e}", job.source.display()))?;
            let name = job.source.to_string_lossy();
            let wav = adpcm_wav(&decode(&name, &bytes)?);
            write(&cached, &wav)?;
        }
        let to = source_dir.join(&job.out);
        if let Some(dir) = to.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
        }
        fs::copy(&cached, &to)
            .map_err(|e| format!("copy {} to {}: {e}", cached.display(), to.display()))?;
    }
    Ok(())
}

/// Writes `bytes` to `path`, making its folder.
fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("create {}: {e}", dir.display()))?;
    }
    fs::write(path, bytes).map_err(|e| format!("write {}: {e}", path.display()))
}

/// Converts the Playdate's sounds and music for the workspace at `root`
/// into `source_dir`, through the cache in `out` (`target/playdate/`).
///
/// Not covered by mutation testing (`#[mutants::skip]`): it joins the
/// manifest, [`jobs`] and [`convert`] (each tested) over the real files.
#[mutants::skip]
pub fn run(root: &Path, out: &Path, source_dir: &Path) -> Result<(), String> {
    let manifest = audio::load().map_err(|errors| {
        let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
        errors.join("; ")
    })?;
    let music = audio::playdate_music(&manifest)?;
    convert(&jobs(root, &manifest, &music)?, &cache_dir(out), source_dir)
}

#[cfg(test)]
mod tests {
    use std::f64::consts::TAU;

    use super::*;

    /// A 440 Hz tone, `len` samples at 44.1 kHz, at half volume.
    fn tone(len: usize) -> Vec<i16> {
        (0..len)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let t = i as f64 / f64::from(SAMPLE_RATE);
                #[allow(clippy::cast_possible_truncation)]
                let s = ((TAU * 440.0 * t).sin() * 16_000.0) as i16;
                s
            })
            .collect()
    }

    fn u16_at(bytes: &[u8], at: usize) -> u16 {
        u16::from_le_bytes([bytes[at], bytes[at + 1]])
    }

    fn u32_at(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
    }

    /// The samples of an ADPCM WAV written by [`adpcm_wav`], as the
    /// standard IMA decoder reads them (blocks only, padding included).
    fn decode_adpcm(wav: &[u8]) -> Vec<i16> {
        let data = &wav[60..];
        let mut out = Vec::new();
        for block in data.chunks(BLOCK_BYTES) {
            let first = i16::from_le_bytes([block[0], block[1]]);
            let mut coder = Coder {
                predictor: i32::from(first),
                index: i32::from(block[2]),
            };
            out.push(first);
            for &byte in &block[4..] {
                out.push(coder.decode(byte & 0x0f));
                out.push(coder.decode(byte >> 4));
            }
        }
        out
    }

    /// The test tone through the whole conversion: its WAV header, its
    /// length and its sound.
    #[test]
    fn a_test_tone_becomes_an_ima_adpcm_wav_with_its_length() {
        // 1.5 blocks: the last is padded.
        let samples = tone(BLOCK_SAMPLES * 3 / 2);
        let pcm = hound_wav(&samples, 1);
        let decoded = decode("tone.wav", &pcm).unwrap();
        assert_eq!(decoded, samples);
        let wav = adpcm_wav(&decoded);
        assert_eq!(&wav[..4], b"RIFF");
        assert_eq!(u32_at(&wav, 4) as usize, wav.len() - 8);
        assert_eq!(&wav[8..16], b"WAVEfmt ");
        assert_eq!(u32_at(&wav, 16), 20);
        assert_eq!(u16_at(&wav, 20), 0x11, "IMA ADPCM");
        assert_eq!(u16_at(&wav, 22), 1, "mono");
        assert_eq!(u32_at(&wav, 24), 44_100);
        assert_eq!(u32_at(&wav, 28), 22_125);
        assert_eq!(u16_at(&wav, 32), 1024);
        assert_eq!(u16_at(&wav, 34), 4);
        assert_eq!(u16_at(&wav, 36), 2);
        assert_eq!(u16_at(&wav, 38), 2041);
        assert_eq!(&wav[40..44], b"fact");
        assert_eq!(u32_at(&wav, 48) as usize, samples.len());
        assert_eq!(&wav[52..56], b"data");
        assert_eq!(u32_at(&wav, 56) as usize, 2 * BLOCK_BYTES);
        assert_eq!(wav.len(), 60 + 2 * BLOCK_BYTES);
        // It sounds like the tone: once the coder has caught up with the
        // tone's start, each sample is close; the padding is silent.
        let back = decode_adpcm(&wav);
        assert_eq!(back.len(), 2 * BLOCK_SAMPLES);
        let worst = samples
            .iter()
            .zip(&back)
            .skip(64)
            .map(|(a, b)| (i32::from(*a) - i32::from(*b)).abs())
            .max()
            .unwrap();
        assert!(worst < 300, "{worst}");
        assert!(back[samples.len() + 64..].iter().all(|s| s.abs() < 64));
    }

    /// The standard IMA ADPCM tables and steps, worked by hand: the step
    /// at index 10 is 19.
    #[test]
    fn the_coder_follows_the_ima_standard() {
        let at = |index| Coder {
            predictor: 100,
            index,
        };
        // Codes 0–3 step the index down one; 4–7 up 2, 4, 6, 8.
        for (code, index) in [
            (0, 9),
            (1, 9),
            (2, 9),
            (3, 9),
            (4, 12),
            (5, 14),
            (6, 16),
            (7, 18),
        ] {
            let mut coder = at(10);
            coder.decode(code);
            assert_eq!(coder.index, index, "code {code}");
        }
        // 7: 19/8 + 19 + 19/2 + 19/4 = 2 + 19 + 9 + 4; 15 is its negative.
        assert_eq!(at(10).decode(7), 134);
        assert_eq!(at(10).decode(15), 66);
        assert_eq!(at(10).decode(0), 102);
        // The same sample again is code 0, not a negative 0.
        assert_eq!(at(10).encode(100), 0);
        assert_eq!(at(10).encode(134), 7);
        assert_eq!(at(10).encode(66), 15);
        // A byte holds two codes, the first in its low half.
        let wav = adpcm_wav(&[0, 34, -34]);
        assert_eq!(wav[64] & 0x0f, 7);
        assert_eq!(wav[64] >> 4, 0x0f);
    }

    #[test]
    fn a_block_starts_with_its_first_sample_exactly() {
        let samples = tone(BLOCK_SAMPLES * 2);
        let wav = adpcm_wav(&samples);
        let back = decode_adpcm(&wav);
        assert_eq!(back[0], samples[0]);
        assert_eq!(back[BLOCK_SAMPLES], samples[BLOCK_SAMPLES]);
        // Nothing at all is still one (silent) block.
        let empty = adpcm_wav(&[]);
        assert_eq!(empty.len(), 60 + BLOCK_BYTES);
        assert_eq!(u32_at(&empty, 48), 0);
        // A full-scale square wave never wraps around: once caught up,
        // each half stays on its side.
        let square: Vec<i16> = (0..4000)
            .map(|i| if i / 50 % 2 == 0 { i16::MAX } else { i16::MIN })
            .collect();
        let back = decode_adpcm(&adpcm_wav(&square));
        assert!(
            back[90..100].iter().all(|&s| s < -20_000),
            "{:?}",
            &back[90..100]
        );
        assert!(
            back[140..150].iter().all(|&s| s > 20_000),
            "{:?}",
            &back[140..150]
        );
    }

    /// `samples` (interleaved, `channels`) as a 16-bit PCM WAV.
    fn hound_wav(samples: &[i16], channels: u16) -> Vec<u8> {
        let spec = hound::WavSpec {
            channels,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut bytes = Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(&mut bytes, spec).unwrap();
        for &s in samples {
            writer.write_sample(s).unwrap();
        }
        writer.finalize().unwrap();
        bytes.into_inner()
    }

    #[test]
    fn stereo_is_averaged_and_float_is_scaled() {
        let pcm = hound_wav(&[100, 300, -100, -301, 7, 7], 2);
        assert_eq!(decode("a.wav", &pcm), Ok(vec![200, -200, 7]));
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut bytes = Cursor::new(Vec::new());
        let mut writer = hound::WavWriter::new(&mut bytes, spec).unwrap();
        for s in [0.0f32, 0.5, -1.0, 2.0] {
            writer.write_sample(s).unwrap();
        }
        writer.finalize().unwrap();
        assert_eq!(
            decode("f.wav", &bytes.into_inner()),
            Ok(vec![0, 16_384, -32_767, 32_767])
        );
    }

    #[test]
    fn a_file_it_cannot_play_is_an_error_naming_it() {
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 22_050,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut bytes = Cursor::new(Vec::new());
        hound::WavWriter::new(&mut bytes, spec)
            .unwrap()
            .finalize()
            .unwrap();
        assert_eq!(
            decode("slow.wav", &bytes.into_inner()),
            Err("slow.wav: 22050 Hz; must be 44100 Hz".to_owned())
        );
        assert!(
            decode("x.ogg", b"not ogg")
                .unwrap_err()
                .starts_with("x.ogg: ")
        );
        assert!(
            decode("x.wav", b"not wav")
                .unwrap_err()
                .starts_with("x.wav: ")
        );
    }

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// One real file of each kind, decoded: the game's own sound (WAV) and
    /// a track's jingle (OGG), as long as the manifest says.
    #[test]
    fn the_games_files_decode() {
        let heal = fs::read(root().join("assets/audio/sfx/heal.wav")).unwrap();
        assert_ne!(decode("heal.wav", &heal).unwrap(), [] as [i16; 0]);
        let jingle = fs::read(root().join("music/win_jingle_brass.ogg")).unwrap();
        let samples = decode("win_jingle_brass.ogg", &jingle).unwrap();
        let manifest = audio::load().unwrap();
        let ms = samples.len() as u64 * 1000 / u64::from(SAMPLE_RATE);
        let length = u64::from(manifest.music["victory_brass"].length_ms);
        assert!(ms.abs_diff(length) <= 1, "{ms} vs {length}");
    }

    #[test]
    fn every_sound_file_once_and_the_listed_music() {
        let root = Path::new("/repo");
        let manifest = audio::load().unwrap();
        let music = BTreeSet::from(["title".to_owned()]);
        let jobs = jobs(root, &manifest, &music).unwrap();
        let sounds: Vec<&Job> = jobs
            .iter()
            .filter(|j| j.out.starts_with("audio/"))
            .collect();
        let files: BTreeSet<&String> = manifest.sounds.values().flat_map(|s| &s.files).collect();
        assert_eq!(sounds.len(), files.len());
        assert!(jobs.contains(&Job {
            source: root.join("assets/audio/sfx/menu_move.wav"),
            out: "audio/sfx/menu_move.wav".to_owned(),
        }));
        let tracks: Vec<&Job> = jobs
            .iter()
            .filter(|j| j.out.starts_with("music/"))
            .collect();
        assert_eq!(
            tracks,
            [&Job {
                source: root.join("music/new_sunrise_v1.ogg"),
                out: "music/new_sunrise_v1.wav".to_owned(),
            }]
        );
        let unknown = BTreeSet::from(["nope".to_owned()]);
        assert_eq!(
            jobs_err(root, &manifest, &unknown),
            "\"nope\" is not a music cue"
        );
    }

    fn jobs_err(root: &Path, manifest: &AudioManifest, music: &BTreeSet<String>) -> String {
        jobs(root, manifest, music).unwrap_err()
    }

    #[test]
    fn two_files_on_one_playdate_path_are_an_error() {
        let mut manifest = audio::load().unwrap();
        if let Some(cue) = manifest.sounds.get_mut("heal") {
            cue.files.push("sfx/heal.ogg".to_owned());
        }
        let err = jobs_err(Path::new("r"), &manifest, &BTreeSet::new());
        assert!(
            err.ends_with("would both be audio/sfx/heal.wav on the Playdate"),
            "{err}"
        );
    }

    #[test]
    fn the_conversion_is_cached_until_its_source_changes() {
        let dir = std::env::temp_dir().join(format!("xtask-playdate-audio-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let source = dir.join("in/beep.wav");
        write(&source, &hound_wav(&tone(3000), 1)).unwrap();
        let job = Job {
            source: source.clone(),
            out: "audio/beep.wav".to_owned(),
        };
        let cache = cache_dir(&dir);
        assert!(cache.ends_with(format!("audio-v{FORMAT}")));
        let out = dir.join("Source");
        convert(std::slice::from_ref(&job), &cache, &out).unwrap();
        let totals = Totals::on_disk(&out);
        assert_eq!(totals.sounds, (1, 60 + 2 * BLOCK_BYTES as u64));
        assert_eq!(totals.music, (0, 0));
        assert_eq!(
            fs::read(out.join("audio/beep.wav")).unwrap(),
            fs::read(cache.join("audio/beep.wav")).unwrap()
        );
        // A cached file newer than its source is used as it is.
        write(&cache.join("audio/beep.wav"), b"cached").unwrap();
        convert(std::slice::from_ref(&job), &cache, &out).unwrap();
        assert_eq!(fs::read(out.join("audio/beep.wav")).unwrap(), b"cached");
        // A source newer than the cache is converted again.
        std::thread::sleep(std::time::Duration::from_millis(20));
        write(&source, &hound_wav(&tone(10), 1)).unwrap();
        convert(&[job], &cache, &out).unwrap();
        assert_eq!(
            fs::read(out.join("audio/beep.wav")).unwrap().len(),
            60 + BLOCK_BYTES
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_totals_count_files_in_nested_folders() {
        let dir = std::env::temp_dir().join(format!("xtask-audio-totals-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        write(&dir.join("audio/b.wav"), &[0; 5]).unwrap();
        write(&dir.join("audio/sfx/a.wav"), &[0; 3]).unwrap();
        write(&dir.join("audio/sfx/deep/c.wav"), &[0; 7]).unwrap();
        write(&dir.join("music/t.wav"), &[0; 2]).unwrap();
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
