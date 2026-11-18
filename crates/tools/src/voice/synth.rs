//! The programs `generate` runs, each behind a small trait so a tool is
//! swapped in one place (ticket 0721): a [`Synth`] reads lines in a voice,
//! an [`Ear`] says what a take says, and [`Audio`] measures, trims and
//! levels a take into the game's clip. The readers are the voice lab's
//! Python programs (ADR-0107 rule 3, `tools/voice/lab.py`), run as child
//! processes; `Tools::fake` stands in for all of them in tests.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

use super::audit::{self, Measure};
use super::files::Reader;
#[cfg(test)]
use crate::sfx;

/// One line for a reader to say.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Request {
    /// The take's file name in the output folder, without `.wav`.
    pub key: String,
    /// The words, as spelled for the reader.
    pub text: String,
    /// How it is said, if the script says.
    pub direction: Option<String>,
    /// The voice's reference clip.
    pub reference: PathBuf,
    /// The words the reference clip says.
    pub reference_text: String,
}

/// Something that reads lines in a voice.
pub trait Synth {
    /// What the manifest says made a clip: the tool and its model.
    fn made_by(&self) -> (String, String);

    /// Reads every request with random seed `seed`, writing each take as
    /// `<out>/<key>.wav`.
    fn read(&mut self, requests: &[Request], seed: u64, out: &Path) -> Result<(), String>;
}

/// Something that hears what a take says.
pub trait Ear {
    /// The words each request's take in `out` says, by key.
    fn hear(
        &mut self,
        requests: &[Request],
        out: &Path,
    ) -> Result<BTreeMap<String, String>, String>;
}

/// A clip's loudness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Level {
    /// Integrated loudness, LUFS (EBU R128).
    pub lufs: f64,
    /// True peak, dBTP.
    pub peak: f64,
}

/// Measures and finishes clips.
pub trait Audio {
    /// The loudness of `file` once trimmed.
    fn level(&self, file: &Path) -> Result<Level, String>;

    /// Writes `wav`, trimmed and `gain` dB louder, as the game's clip
    /// `ogg`: OGG Vorbis, mono, 44.1 kHz (ADR-0046 rule 2).
    fn finish(&self, wav: &Path, gain: f64, ogg: &Path) -> Result<(), String>;

    /// The length of the game's clip `ogg` and the pauses inside it
    /// (ticket 0762).
    fn measure(&self, ogg: &Path) -> Result<Measure, String>;
}

/// Everything `generate` runs.
pub struct Tools {
    /// Reads undirected lines of Qwen3's voices.
    pub qwen3: Box<dyn Synth>,
    /// Reads undirected lines of Chatterbox's voices.
    pub chatterbox: Box<dyn Synth>,
    /// Reads every directed line, whoever the reader (ADR-0109 rule 3).
    pub directed: Box<dyn Synth>,
    /// Checks the takes' words.
    pub ear: Box<dyn Ear>,
    /// Levels and encodes the takes.
    pub audio: Box<dyn Audio>,
}

impl Tools {
    /// The reader of a line of `reader`'s voice, directed or not.
    pub fn synth(&mut self, reader: Reader, directed: bool) -> &mut dyn Synth {
        match (reader, directed) {
            (_, true) => self.directed.as_mut(),
            (Reader::Qwen3, false) => self.qwen3.as_mut(),
            (Reader::Chatterbox, false) => self.chatterbox.as_mut(),
        }
    }

    /// The tool the manifest names for a take of `reader`'s voice.
    pub fn tool(&self, reader: Reader, directed: bool) -> String {
        let synth = match (reader, directed) {
            (_, true) => &self.directed,
            (Reader::Qwen3, false) => &self.qwen3,
            (Reader::Chatterbox, false) => &self.chatterbox,
        };
        synth.made_by().0
    }

    /// The voice lab's programs, in the lab at `lab`, run with the script
    /// `script`, for language `language`.
    pub fn lab(lab: &Path, script: &Path, language: &str) -> Self {
        let lab = Lab {
            root: lab.to_path_buf(),
            script: script.to_path_buf(),
            language: language.to_owned(),
        };
        Self {
            qwen3: Box::new(Program {
                lab: lab.clone(),
                steps: &[("qwen", "qwen3")],
                tool: "Qwen3-TTS",
                model: "Qwen3-TTS-12Hz-1.7B-Base",
            }),
            chatterbox: Box::new(Program {
                lab: lab.clone(),
                steps: &[("chatter", "chatterbox")],
                tool: "Chatterbox",
                model: "ChatterboxMultilingualTTS v3",
            }),
            directed: Box::new(Program {
                lab: lab.clone(),
                steps: &[("qwen", "directed"), ("chatter", "convert")],
                tool: "Qwen3-TTS + Chatterbox",
                model: "Qwen3-TTS-12Hz-1.7B-VoiceDesign + ChatterboxVC",
            }),
            ear: Box::new(lab),
            audio: Box::new(Ffmpeg),
        }
    }

    #[cfg(test)]
    /// Stand-ins for tests: a tone per line, heard as the words it was
    /// asked to say unless its key is in `mumbled`, and leveled by copying.
    pub fn fake(mumbled: &[&str]) -> Self {
        let fake = || Box::new(Fake(Vec::new()));
        Self {
            qwen3: fake(),
            chatterbox: fake(),
            directed: fake(),
            ear: Box::new(FakeEar(mumbled.iter().map(|&k| k.to_owned()).collect())),
            audio: fake(),
        }
    }

    #[cfg(test)]
    /// [`fake`](Self::fake)'s tools, with the clips of `halting` measured
    /// with a second's pause inside: a key, or `<key>@<seed>` for that
    /// seed's take only.
    pub fn halting(mut self, halting: &[&str]) -> Self {
        self.audio = Box::new(Fake(halting.iter().map(|&k| k.to_owned()).collect()));
        self
    }
}

/// The voice lab (ADR-0107 rule 3): its Python environments, and the
/// script that runs the readers.
#[derive(Debug, Clone)]
struct Lab {
    root: PathBuf,
    script: PathBuf,
    language: String,
}

impl Lab {
    /// Runs `command` of the lab's script in environment `env` on a job of
    /// `requests` written to `<out>/job.json`. The readers' progress goes
    /// to the terminal.
    ///
    /// Not covered by mutation testing (`#[mutants::skip]`): it only spawns
    /// the lab's Python, which isn't installed where the tests run.
    #[mutants::skip]
    fn run(
        &self,
        env: &str,
        command: &str,
        requests: &[Request],
        seed: u64,
        out: &Path,
    ) -> Result<(), String> {
        let job = serde_json::json!({
            "seed": seed,
            "language": self.language,
            "out": out,
            "lines": requests,
        });
        let file = out.join("job.json");
        fs::write(&file, job.to_string())
            .map_err(|e| format!("writing {}: {e}", file.display()))?;
        let python = self.root.join("envs").join(env).join("Scripts/python.exe");
        let status = Command::new(&python)
            .arg(&self.script)
            .args([command.as_ref(), file.as_os_str()])
            .env("VOICE_LAB", &self.root)
            .env("HF_HOME", self.root.join("hf"))
            .env("HF_HUB_DISABLE_SYMLINKS", "1")
            .env("PYTHONUTF8", "1")
            .env("TEMP", self.root.join("tmp"))
            .env("TMP", self.root.join("tmp"))
            .status()
            .map_err(|e| {
                format!(
                    "running {} (is the voice lab set up?): {e}",
                    python.display()
                )
            })?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("{env}/{command} failed ({status})"))
        }
    }
}

impl Ear for Lab {
    #[mutants::skip]
    fn hear(
        &mut self,
        requests: &[Request],
        out: &Path,
    ) -> Result<BTreeMap<String, String>, String> {
        self.run("qwen", "hear", requests, 0, out)?;
        let file = out.join("heard.json");
        let text =
            fs::read_to_string(&file).map_err(|e| format!("reading {}: {e}", file.display()))?;
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", file.display()))
    }
}

/// A reader: commands of the lab's script, run one after the other.
struct Program {
    lab: Lab,
    /// Each command and the environment it runs in.
    steps: &'static [(&'static str, &'static str)],
    tool: &'static str,
    model: &'static str,
}

impl Synth for Program {
    fn made_by(&self) -> (String, String) {
        (self.tool.to_owned(), self.model.to_owned())
    }

    #[mutants::skip]
    fn read(&mut self, requests: &[Request], seed: u64, out: &Path) -> Result<(), String> {
        for (env, command) in self.steps {
            self.lab.run(env, command, requests, seed, out)?;
        }
        Ok(())
    }
}

/// How long a clip's silence may be at either end, seconds (*tunable*).
pub const KEEP_SILENCE_S: f64 = 0.1;
/// The level every undirected clip is brought to, LUFS (*tunable*): a
/// little louder than the music (`assets-src/audio/import.py`), as the
/// sounds are.
pub const VOICE_LUFS: f64 = -18.0;
/// The true peak no gain may push past, dBTP: the music's.
pub const PEAK_CEILING_DBTP: f64 = -1.0;
/// How far below its voice's level a directed clip may stay, dB
/// (*tunable*; ADR-0109 rule 4).
pub const DIRECTED_FLOOR_DB: f64 = -12.0;
/// How far above it, dB (*tunable*).
pub const DIRECTED_CEILING_DB: f64 = 6.0;

/// The gain that brings a clip at `level` to `target` LUFS without its
/// true peak passing the ceiling: `import.py`'s `gain_for`.
pub fn gain_for(level: Level, target: f64) -> f64 {
    (target - level.lufs).min(PEAK_CEILING_DBTP - level.peak)
}

/// The level a directed take at `take` is brought to, when its voice reads
/// at `voice`: [`VOICE_LUFS`] plus its distance from the voice, within the
/// floor and the ceiling, so a whisper stays quiet.
pub fn directed_target(take: Level, voice: Level) -> f64 {
    VOICE_LUFS + (take.lufs - voice.lufs).clamp(DIRECTED_FLOOR_DB, DIRECTED_CEILING_DB)
}

/// How alike a transcript must be to the words, 0 to 1 (*tunable*): the
/// lab's check passed a take at 0.85.
pub const SAME_WORDS: f64 = 0.85;

/// The most words a line may have to be checked by sound when its spelling
/// differs (*tunable*; ticket 0761).
pub const SHORT_LINE_WORDS: usize = 4;

/// Whether `heard` is `words`: their letters and digits, compared
/// character by character, are at least [`SAME_WORDS`] alike. A word a
/// transcriber spells otherwise (an invented name) costs a letter or two,
/// which a short line can't spare: one of at most [`SHORT_LINE_WORDS`]
/// words also passes when every word sounds the same ([`sound`]), so
/// "Mases" says "Mace's.".
pub fn says(heard: &str, words: &str) -> bool {
    let (a, b) = (plain(heard), plain(words));
    alike(&a, &b) >= SAME_WORDS || {
        let (a, b): (Vec<&str>, Vec<&str>) = (a.split(' ').collect(), b.split(' ').collect());
        b.len() <= SHORT_LINE_WORDS
            && a.len() == b.len()
            && a.iter().zip(&b).all(|(a, b)| sound(a) == sound(b))
    }
}

/// `s` lowercased, with its words of letters, digits and apostrophes
/// joined by single spaces.
fn plain(s: &str) -> String {
    let lower = s.to_lowercase().replace('\u{2019}', "'");
    let kept: String = lower
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '\'' {
                c
            } else {
                ' '
            }
        })
        .collect();
    kept.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A word's sound, a Metaphone-like key of a plain word: letters that
/// sound alike written one way, vowels after the first letter dropped (a
/// first vowel is `a`), doubled sounds once, apostrophes gone.
pub fn sound(word: &str) -> String {
    let w: Vec<char> = word.chars().filter(|c| c.is_alphanumeric()).collect();
    let mut key = String::new();
    let mut i = 0;
    while i < w.len() {
        let next = w.get(i + 1).copied();
        let (codes, step): (&[char], usize) = match (w[i], next) {
            ('p', Some('h')) => (&['f'], 2),
            ('c' | 's', Some('h')) => (&['x'], 2),
            ('t', Some('h')) => (&['0'], 2),
            ('c', Some('e' | 'i' | 'y')) | ('z', _) => (&['s'], 1),
            ('c' | 'q', _) => (&['k'], 1),
            ('x', _) => (&['k', 's'], 1),
            ('a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'w' | 'h', _) if i == 0 => (&['a'], 1),
            ('a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'w' | 'h', _) => (&[], 1),
            _ => (std::slice::from_ref(&w[i]), 1),
        };
        for &code in codes {
            if !key.ends_with(code) {
                key.push(code);
            }
        }
        i += step;
    }
    key
}

/// How alike two plain strings are, 0 to 1: one less their edit distance
/// over the longer one's length.
fn alike(a: &str, b: &str) -> f64 {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let longest = a.len().max(b.len());
    if longest == 0 {
        return 1.0;
    }
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut diagonal = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let replace = diagonal + usize::from(ca != cb);
            diagonal = row[j + 1];
            row[j + 1] = replace.min(row[j] + 1).min(diagonal + 1);
        }
    }
    // Lines are a few hundred characters at most.
    let float = |n: usize| f64::from(u32::try_from(n).unwrap_or(u32::MAX));
    1.0 - float(row[b.len()]) / float(longest)
}

/// ffmpeg's filters that make a take the game's: mono at 44.1 kHz, with at
/// most [`KEEP_SILENCE_S`] of silence at either end.
pub fn trim_filters() -> String {
    let trim = format!(
        "silenceremove=start_periods=1:start_threshold=-50dB:start_silence={KEEP_SILENCE_S}"
    );
    format!(
        "aformat=channel_layouts=mono,aresample=44100:resampler=soxr,{trim},areverse,{trim},areverse"
    )
}

/// The number after the last `label` in ffmpeg's `ebur128` log `log`.
fn number_after(log: &str, label: &str) -> Option<f64> {
    let at = log.rfind(label)? + label.len();
    let word = log[at..].split_whitespace().next()?;
    if word == "-inf" {
        Some(f64::NEG_INFINITY)
    } else {
        word.parse().ok()
    }
}

/// The loudness in the summary of ffmpeg's `ebur128=peak=true` log.
pub fn parse_level(log: &str) -> Option<Level> {
    let summary = &log[log.rfind("Summary:")?..];
    Some(Level {
        lufs: number_after(summary, "I:")?,
        peak: number_after(summary, "Peak:")?,
    })
}

/// [`Audio`] by ffmpeg, as the music's import does.
struct Ffmpeg;

impl Ffmpeg {
    /// Runs ffmpeg with `args`, returning its log.
    ///
    /// Not covered by mutation testing: see [`crate::voice_test::ffmpeg`].
    #[mutants::skip]
    fn run(args: &[&std::ffi::OsStr]) -> Result<String, String> {
        let output = Command::new(crate::voice_test::ffmpeg())
            .args(["-hide_banner", "-nostats"])
            .args(args)
            .output()
            .map_err(|e| format!("running ffmpeg (is it installed?): {e}"))?;
        let log = String::from_utf8_lossy(&output.stderr).into_owned();
        if output.status.success() {
            Ok(log)
        } else {
            Err(format!("ffmpeg failed: {}", log.trim()))
        }
    }
}

impl Audio for Ffmpeg {
    #[mutants::skip]
    fn level(&self, file: &Path) -> Result<Level, String> {
        // Padded to half a second, so a short line fills a 400 ms window.
        let chain = format!("{},apad=whole_dur=0.5,ebur128=peak=true", trim_filters());
        let log = Self::run(&[
            "-i".as_ref(),
            file.as_os_str(),
            "-af".as_ref(),
            chain.as_ref(),
            "-f".as_ref(),
            "null".as_ref(),
            "-".as_ref(),
        ])?;
        parse_level(&log)
            .ok_or_else(|| format!("no loudness in ffmpeg's log of {}", file.display()))
    }

    #[mutants::skip]
    fn finish(&self, wav: &Path, gain: f64, ogg: &Path) -> Result<(), String> {
        let chain = format!("{},volume={gain:.2}dB", trim_filters());
        Self::run(&[
            "-y".as_ref(),
            "-i".as_ref(),
            wav.as_os_str(),
            "-af".as_ref(),
            chain.as_ref(),
            "-ac".as_ref(),
            "1".as_ref(),
            "-ar".as_ref(),
            "44100".as_ref(),
            "-c:a".as_ref(),
            "libvorbis".as_ref(),
            "-q:a".as_ref(),
            "4".as_ref(),
            "-map_metadata".as_ref(),
            "-1".as_ref(),
            ogg.as_os_str(),
        ])
        .map(drop)
    }

    #[mutants::skip]
    fn measure(&self, ogg: &Path) -> Result<Measure, String> {
        let bytes = fs::read(ogg).map_err(|e| format!("reading {}: {e}", ogg.display()))?;
        let millis = trpg_content::audio::ogg_length_ms(&bytes)
            .ok_or_else(|| format!("{} is not an OGG Vorbis file", ogg.display()))?;
        let length_s = f64::from(millis) / 1000.0;
        let log = Self::run(&[
            "-i".as_ref(),
            ogg.as_os_str(),
            "-af".as_ref(),
            audit::silence_filter().as_ref(),
            "-f".as_ref(),
            "null".as_ref(),
            "-".as_ref(),
        ])?;
        Ok(Measure {
            length_s,
            pauses: audit::inner_pauses(&log, length_s),
        })
    }
}

/// A reader and audio for tests: a short phrase of tones per line, each
/// measured a second long, with a second's pause inside if its key is
/// listed.
#[cfg(test)]
struct Fake(Vec<String>);

#[cfg(test)]
impl Synth for Fake {
    fn made_by(&self) -> (String, String) {
        ("test tones".to_owned(), "none".to_owned())
    }

    fn read(&mut self, requests: &[Request], _seed: u64, out: &Path) -> Result<(), String> {
        for (i, request) in requests.iter().enumerate() {
            let wav = out.join(format!("{}.wav", request.key));
            fs::write(&wav, sfx::wav_bytes(&sfx::test_phrase(i)))
                .map_err(|e| format!("writing {}: {e}", wav.display()))?;
        }
        Ok(())
    }
}

#[cfg(test)]
impl Audio for Fake {
    fn level(&self, _file: &Path) -> Result<Level, String> {
        Ok(Level {
            lufs: -20.0,
            peak: -3.0,
        })
    }

    fn finish(&self, wav: &Path, _gain: f64, ogg: &Path) -> Result<(), String> {
        fs::copy(wav, ogg)
            .map(drop)
            .map_err(|e| format!("copying to {}: {e}", ogg.display()))
    }

    fn measure(&self, ogg: &Path) -> Result<Measure, String> {
        let name = |p: Option<&std::ffi::OsStr>| p.map(|s| s.to_string_lossy().into_owned());
        let key = name(ogg.file_stem()).unwrap_or_default();
        let seed = name(ogg.parent().and_then(Path::file_name)).unwrap_or_default();
        let halting = self.0.contains(&key) || self.0.contains(&format!("{key}@{seed}"));
        Ok(Measure {
            length_s: 1.0,
            pauses: if halting { vec![1.0] } else { Vec::new() },
        })
    }
}

/// An ear for tests: hears each take say its words, unless its key is
/// mumbled.
#[cfg(test)]
struct FakeEar(Vec<String>);

#[cfg(test)]
impl Ear for FakeEar {
    fn hear(
        &mut self,
        requests: &[Request],
        _out: &Path,
    ) -> Result<BTreeMap<String, String>, String> {
        Ok(requests
            .iter()
            .map(|r| {
                let heard = if self.0.contains(&r.key) {
                    "mumble"
                } else {
                    r.text.as_str()
                };
                (r.key.clone(), heard.to_owned())
            })
            .collect())
    }
}
