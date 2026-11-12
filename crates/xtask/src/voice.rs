//! `cargo xtask voice`: makes the voice clips that are missing or stale,
//! and prints recording scripts (ticket 0721, ADR-0046 rule 7, ADR-0107,
//! ADR-0109, `docs/voice.md`). It runs on a developer's machine, never in
//! the game or in CI.
//!
//! - `status` lists, per scene, the clips to make and why, the recorded
//!   clips to record again and the clips to remove.
//! - `generate` makes them with the voice lab's readers, levels them and
//!   writes the manifest; `--dry-run` only counts.
//! - `script --character <id>` prints what an actor records.
//! - `audit` lists each clip's length, pace and longest pause inside, and
//!   flags those over the limits (ticket 0762); `generate` warns of each
//!   clip it made over them.

mod audit;
mod files;
mod plan;
mod synth;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

use trpg_content::Content;
use trpg_content::dialogue::{self, DIALOGUE_DIR, DIALOGUE_EXTENSION, DialogueTable};
use trpg_content::lesson::{self, LESSONS_PATH, LessonTable};
use trpg_content::voice::{self, MANIFEST_FILE, MadeBy, SOURCE_LANG, VoiceClip};

use files::{CAST_FILE, Cast, Reader, SAY_FILE, SENT_FILE, Say, Sent, SentMap};
use plan::{Inputs, Plan, Wanted};
pub use synth::Tools;
use synth::{Level, Request};

pub const USAGE: &str = "usage: cargo xtask voice <status | generate | script | audit> [options]\n\n\
  status                       per scene, the clips to make and why, the recorded clips\n                               \
  to record again and the clips to remove\n  \
  generate [--dry-run]         make them with the voice lab's readers and write the\n                               \
  manifest; --dry-run counts the lines and characters and makes nothing\n  \
  script --character <id>      a recording script for an actor: a character id, \">\"\n                               \
  for the narrator, or lead.m / lead.f\n  \
  audit                        each clip's length, pace and longest pause inside; flags\n                               \
  those over the limits\n\n\
options:\n  \
  --lang <code>                the language's folder (default en)\n  \
  --scene <id>                 one scene (a test scene too, or vix: the guide's lessons\n                               \
  and tips); default vix and every scene of assets/dialogue/\n  \
  --dir <folder>               the voice folder (default assets-private/voice)\n  \
  --lab <folder>               the voice lab (default $VOICE_LAB, else D:/voice-lab)";

/// The reference clips, relative to the repository (ADR-0107 rule 1).
pub const REFS_DIR: &str = "assets-src/audio/voice-refs";
/// The words every reference clip says (`assets-src/audio/voice-refs/README.md`).
pub const REFERENCE_TEXT: &str = "The road runs north past the mill and over the ford. We keep \
                                  moving until dark, and we eat when we get there.";
/// The seeds a line is tried with, in order, until a take says its words
/// (ADR-0109: up to four).
pub const SEEDS: [u64; 4] = [7, 11, 23, 42];

/// A command and its options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub command: Command,
    pub lang: String,
    pub scene: Option<String>,
    pub dir: Option<PathBuf>,
    pub lab: Option<PathBuf>,
}

/// What to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Status,
    Generate { dry_run: bool },
    Script { character: String },
    Audit,
}

/// Parses the arguments after `voice`.
pub fn parse(args: &[String]) -> Result<Args, String> {
    let mut command = match args.first().map(String::as_str) {
        Some("status") => Command::Status,
        Some("generate") => Command::Generate { dry_run: false },
        Some("script") => Command::Script {
            character: String::new(),
        },
        Some("audit") => Command::Audit,
        _ => return Err(USAGE.to_owned()),
    };
    let mut parsed = Args {
        command: Command::Status,
        lang: SOURCE_LANG.to_owned(),
        scene: None,
        dir: None,
        lab: None,
    };
    let mut rest = args[1..].iter();
    while let Some(flag) = rest.next() {
        let mut value = || {
            rest.next()
                .cloned()
                .ok_or_else(|| format!("{flag} needs a value"))
        };
        match (flag.as_str(), &mut command) {
            ("--lang", _) => parsed.lang = value()?,
            ("--scene", _) => parsed.scene = Some(value()?),
            ("--dir", _) => parsed.dir = Some(value()?.into()),
            ("--lab", Command::Generate { .. }) => parsed.lab = Some(value()?.into()),
            ("--dry-run", Command::Generate { dry_run }) => *dry_run = true,
            ("--character", Command::Script { character }) => *character = value()?,
            _ => return Err(format!("unknown option {flag}\n\n{USAGE}")),
        }
    }
    if matches!(&command, Command::Script { character } if character.is_empty()) {
        return Err(format!("script needs --character <id>\n\n{USAGE}"));
    }
    parsed.command = command;
    Ok(parsed)
}

/// Where the tool reads and writes.
pub struct Folder {
    /// The language's voice folder, `voice/<lang>`.
    pub dir: PathBuf,
    /// The reference clips.
    pub refs: PathBuf,
}

/// The scripts as the tool needs them.
pub struct Scripts<'a> {
    /// The lines as written, the stand-ins for the name kept, and the
    /// guide's lines of the lessons and tips (scene `vix`, ticket 0476).
    pub dialogue: DialogueTable,
    /// The scenes to voice.
    pub scenes: BTreeSet<String>,
    /// The game's content: its names and characters.
    pub content: &'a Content,
}

impl<'a> Scripts<'a> {
    /// `content`'s scripts, voicing `scenes`.
    pub fn new(content: &'a Content, scenes: BTreeSet<String>) -> Self {
        Self {
            dialogue: voice::with_guide(
                dialogue::written(&content.dialogue, &content.stand_ins),
                &shipped_lessons(),
                &content.tips,
            ),
            scenes,
            content,
        }
    }
}

/// The lessons of a build that ships: never the test lesson (ADR-0057),
/// whose clip would be of a line that build doesn't have, so it would
/// refuse the whole manifest. Tips have no test files.
fn shipped_lessons() -> LessonTable {
    let source = trpg_content::bundle::file(LESSONS_PATH).unwrap_or_default();
    lesson::from_source(LESSONS_PATH, source, lesson::Refs::default()).unwrap_or_default()
}

/// The scenes of the story's scripts, `<root>/assets/dialogue/*.dlg` with
/// the content's over them (ADR-0117), and the guide's lines (ticket 0476).
pub fn story_scenes(root: &Path) -> Result<BTreeSet<String>, String> {
    let story = &crate::content_layer::STORY;
    let files = crate::content_layer::files_in(root, story, DIALOGUE_DIR, DIALOGUE_EXTENSION)?;
    let mut scenes = BTreeSet::from([voice::GUIDE_SCENE.to_owned()]);
    for (_, path) in files {
        let text =
            fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        let ids = text
            .lines()
            .filter_map(|l| l.trim().strip_prefix("@scene "))
            .map(|id| id.trim().to_owned());
        scenes.extend(ids);
    }
    Ok(scenes)
}

/// A language folder's files, read.
struct Loaded {
    cast: Cast,
    say: Say,
    clips: Vec<VoiceClip>,
    sent: SentMap,
    /// Each cast clip's fingerprint, by clip name.
    references: BTreeMap<String, String>,
}

/// The text of `file` in `dir`, or `None` if it isn't there.
fn read_optional(dir: &Path, file: &str) -> Result<Option<String>, String> {
    match fs::read_to_string(dir.join(file)) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("reading {}: {e}", dir.join(file).display())),
    }
}

/// Reads `folder`'s files: the cast must be there; no `say.ron`, manifest
/// or `sent.ron` is the same as an empty one.
fn load(folder: &Folder, scripts: &Scripts<'_>) -> Result<Loaded, String> {
    let characters = &scripts.content.characters;
    let cast_text = read_optional(&folder.dir, CAST_FILE)?
        .ok_or_else(|| format!("no {CAST_FILE} in {}", folder.dir.display()))?;
    let cast = files::cast(&cast_text, |id| {
        characters.can_speak(&trpg_core::CharacterId(id.to_owned()))
    })?;
    let say = read_optional(&folder.dir, SAY_FILE)?.map_or(Ok(Say::new()), |t| files::say(&t))?;
    let clips = read_optional(&folder.dir, MANIFEST_FILE)?
        .map_or(Ok(Vec::new()), |t| files::manifest(&t))?;
    let sent =
        read_optional(&folder.dir, SENT_FILE)?.map_or(Ok(SentMap::new()), |t| files::sent(&t))?;
    let mut references = BTreeMap::new();
    for voice in cast.voices.values() {
        let path = reference(folder, &voice.voice);
        let bytes =
            fs::read(&path).map_err(|e| format!("{CAST_FILE}: reading {}: {e}", path.display()))?;
        references.insert(voice.voice.clone(), files::fingerprint(&bytes));
    }
    Ok(Loaded {
        cast,
        say,
        clips,
        sent,
        references,
    })
}

/// A cast clip's reference file.
fn reference(folder: &Folder, clip: &str) -> PathBuf {
    folder.refs.join(format!("{clip}.flac"))
}

/// The plan for `scripts` and `loaded`, with `tools` naming the readers.
fn plan_of(scripts: &Scripts<'_>, loaded: &Loaded, tools: &Tools) -> Plan {
    plan::plan(&Inputs {
        dialogue: &scripts.dialogue,
        scenes: &scripts.scenes,
        names: &scripts.content.names,
        cast: &loaded.cast,
        say: &loaded.say,
        clips: &loaded.clips,
        sent: &loaded.sent,
        references: &loaded.references,
        tool: &|reader, directed| tools.tool(reader, directed),
    })
}

/// `status`'s report.
pub fn status(folder: &Folder, scripts: &Scripts<'_>, tools: &Tools) -> Result<String, String> {
    let loaded = load(folder, scripts)?;
    let plan = plan_of(scripts, &loaded, tools);
    let mut rows: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut row = |line: &trpg_content::dialogue::LineId, text: String| {
        rows.entry(line.scene_id().to_owned())
            .or_default()
            .push(text);
    };
    for (i, why) in &plan.make {
        let w = &plan.wanted[*i];
        row(
            &w.line,
            format!("  make       {:<34} {:<12} {}", w.file(), w.key, why.says()),
        );
    }
    for (i, why) in &plan.record {
        let w = &plan.wanted[*i];
        row(
            &w.line,
            format!("  re-record  {:<34} {:<12} {}", w.file(), w.key, why.says()),
        );
    }
    for clip in &plan.remove {
        let file = trpg_content::voice::clip_file(&clip.line, clip.variant);
        let recorded = matches!(clip.made_by, MadeBy::Recorded { .. });
        let note = if recorded { "  (recorded)" } else { "" };
        row(
            &clip.line,
            format!(
                "  remove     {file:<34} {:<12} its line is gone{note}",
                clip.voice
            ),
        );
    }
    for uncast in &plan.uncast {
        let (line, speaker) = (&uncast.line, &uncast.speaker);
        row(
            line,
            format!("  no voice   {line:<34} {speaker:<12} not in {CAST_FILE}"),
        );
    }
    let mut out = String::new();
    for (scene, lines) in rows {
        let _ = writeln!(out, "{scene}");
        for line in lines {
            let _ = writeln!(out, "{line}");
        }
    }
    let _ = write!(
        out,
        "{} clips to make, {} to re-record, {} to remove, {} lines with no voice",
        plan.make.len(),
        plan.record.len(),
        plan.remove.len(),
        plan.uncast.len()
    );
    Ok(out)
}

/// `script --character <id>`: every clip the voice under cast key
/// `character` says (a character id, `>`, or a key such as `lead.m`), in
/// scene and script order.
pub fn script(
    folder: &Folder,
    scripts: &Scripts<'_>,
    tools: &Tools,
    character: &str,
) -> Result<String, String> {
    let loaded = load(folder, scripts)?;
    let plan = plan_of(scripts, &loaded, tools);
    let mut out = format!(
        "Recording script: {character}\n\
         Deliver each line as OGG Vorbis, mono, 44.1 kHz, under the file name shown,\n\
         in the voice folder's language folder (docs/voice.md).\n"
    );
    let mine = shuyi_assets::voice::plan::recording_script(&plan, character);
    for w in &mine {
        let _ = write!(out, "\n{}\n  scene:  {}\n", w.file(), w.line.scene_id());
        if let Some((speaker, words)) = &w.before {
            let speaker = match speaker.as_str() {
                dialogue::NARRATION_SPEAKER => "narrator",
                dialogue::REPLY_SPEAKER => "the lead (a reply)",
                other => other,
            };
            let _ = writeln!(out, "  before: {speaker}: {words}");
        }
        if let Some(direction) = &w.direction {
            let _ = writeln!(out, "  how:    {direction}");
        }
        let _ = writeln!(out, "  say:    {}", w.spoken);
    }
    let _ = write!(out, "\n{} lines", mine.len());
    Ok(out)
}

/// Today's date, `YYYY-MM-DD`, in UTC.
pub fn today() -> String {
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    civil_date(secs / 86_400)
}

/// The date `days` after 1970-01-01 (Howard Hinnant's `civil_from_days`).
pub fn civil_date(days: u64) -> String {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + u64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// `generate`: removes the clips `status` would remove, then makes the
/// clips to make, trying each line's seeds until a take says its words.
/// With `dry_run`, only counts. Returns the report.
pub fn generate(
    folder: &Folder,
    scripts: &Scripts<'_>,
    tools: &mut Tools,
    dry_run: bool,
    date: &str,
) -> Result<String, String> {
    let mut loaded = load(folder, scripts)?;
    let plan = plan_of(scripts, &loaded, tools);
    let names = &scripts.content.names;
    let jobs: Vec<Job<'_>> = plan
        .make
        .iter()
        .map(|&(i, _)| Job::new(&plan.wanted[i], &loaded.say, names))
        .collect();
    if dry_run {
        return Ok(counts(&jobs, plan.remove.len()));
    }
    for clip in &plan.remove {
        let file = trpg_content::voice::clip_file(&clip.line, clip.variant);
        match fs::remove_file(folder.dir.join(&file)) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                return Err(format!("removing {file}: {e}"));
            }
            _ => {}
        }
        loaded.sent.remove(&file);
    }
    loaded.clips.retain(|c| !plan.remove.contains(c));
    save(folder, &loaded)?;
    let mut failed = Vec::new();
    let mut flagged = Vec::new();
    let mut made = 0;
    // Every directed line goes to the one directed reader, whoever reads
    // the rest of its voice's lines (ADR-0109 rule 3).
    let by = [
        (Reader::Qwen3, false),
        (Reader::Chatterbox, false),
        (Reader::Qwen3, true),
    ];
    for (reader, directed) in by {
        let group: Vec<&Job<'_>> = jobs
            .iter()
            .filter(|w| w.direction.is_some() == directed)
            .filter(|w| directed || loaded.cast.voices[&w.key].settings == reader)
            .collect();
        if group.is_empty() {
            continue;
        }
        let batch = make(folder, &mut loaded, tools, (reader, directed), &group, date)?;
        made += batch.made;
        failed.extend(batch.failed);
        flagged.extend(batch.flagged);
    }
    let mut out = format!(
        "voice: made {made} clips, removed {}, in {}",
        plan.remove.len(),
        folder.dir.display()
    );
    if !failed.is_empty() {
        let _ = write!(
            out,
            "\nno take said the words of {} lines (tried {} seeds); not saved:\n  {}",
            failed.len(),
            SEEDS.len(),
            failed.join("\n  ")
        );
    }
    for row in &flagged {
        let _ = write!(out, "\nwarning: {}: {}", row.file, row.flags());
    }
    Ok(out)
}

/// `--dry-run`'s report: how many clips and characters `jobs` sends, the
/// directed ones apart (each costs more).
fn counts(jobs: &[Job<'_>], removed: usize) -> String {
    let count = |directed: bool| {
        let these = jobs.iter().filter(|w| w.direction.is_some() == directed);
        these.fold((0, 0), |(n, chars), w| {
            (n + 1, chars + w.sent.chars().count())
        })
    };
    let (plain, plain_chars) = count(false);
    let (directed, directed_chars) = count(true);
    format!(
        "voice --dry-run: would make {} clips: {plain} undirected ({plain_chars} characters) and \
         {directed} directed ({directed_chars} characters); would remove {removed}; made nothing",
        plain + directed
    )
}

/// A clip to make, with the words its reader is sent.
struct Job<'a> {
    /// The clip.
    wanted: &'a Wanted,
    /// Its words as sent to the reader ([`plan::sent`]).
    sent: String,
}

impl<'a> Job<'a> {
    fn new(wanted: &'a Wanted, say: &Say, names: &trpg_content::names::Names) -> Self {
        let sent = plan::sent(&wanted.spoken, say, names);
        Self { wanted, sent }
    }
}

impl std::ops::Deref for Job<'_> {
    type Target = Wanted;

    fn deref(&self) -> &Wanted {
        self.wanted
    }
}

/// Writes the manifest and `sent.ron`.
fn save(folder: &Folder, loaded: &Loaded) -> Result<(), String> {
    fs::create_dir_all(&folder.dir)
        .map_err(|e| format!("creating {}: {e}", folder.dir.display()))?;
    for (file, text) in [
        (MANIFEST_FILE, files::manifest_text(&loaded.clips)),
        (SENT_FILE, files::sent_text(&loaded.sent)),
    ] {
        let path = folder.dir.join(file);
        fs::write(&path, text).map_err(|e| format!("writing {}: {e}", path.display()))?;
    }
    Ok(())
}

/// How many batches this process has made: each takes a scratch folder of
/// its own.
static RUNS: AtomicUsize = AtomicUsize::new(0);

/// What a batch of [`make`] did.
struct Batch {
    /// How many clips it made.
    made: usize,
    /// The files of the lines no take said right.
    failed: Vec<String>,
    /// The clips it made that are over the audit's limits.
    flagged: Vec<audit::Row>,
}

/// Makes `group`'s clips, all read by one reader (`by`: the cast's reader
/// and whether they are directed). A take that doesn't say the words is
/// tried again with the next seed; so is one over the audit's limits
/// (ticket 0762), and the line keeps its best take, the one least over.
fn make(
    folder: &Folder,
    loaded: &mut Loaded,
    tools: &mut Tools,
    by: (Reader, bool),
    group: &[&Job<'_>],
    date: &str,
) -> Result<Batch, String> {
    let (tool, model) = tools.synth(by.0, by.1).made_by();
    let run = RUNS.fetch_add(1, Ordering::Relaxed);
    let scratch = std::env::temp_dir().join(format!("trpg-voice-{}-{run}", std::process::id()));
    let mut pending: Vec<&Job<'_>> = group.to_vec();
    // Each line's best take so far, by file.
    let mut best: BTreeMap<String, audit::Row> = BTreeMap::new();
    let mut voice_levels: BTreeMap<String, Level> = BTreeMap::new();
    for seed in SEEDS {
        if pending.is_empty() {
            break;
        }
        let out = scratch.join(seed.to_string());
        fs::create_dir_all(&out).map_err(|e| format!("creating {}: {e}", out.display()))?;
        let requests: Vec<Request> = pending.iter().map(|w| request(folder, loaded, w)).collect();
        tools.synth(by.0, by.1).read(&requests, seed, &out)?;
        let heard = tools.ear.hear(&requests, &out)?;
        let mut missed = Vec::new();
        for (w, request) in pending.into_iter().zip(&requests) {
            let said = heard.get(&request.key).map_or("", String::as_str);
            if !synth::says(said, &w.spoken) && !synth::says(said, &w.sent) {
                missed.push(w);
                continue;
            }
            let take = out.join(format!("{}.wav", request.key));
            let level = tools.audio.level(&take)?;
            let target = if by.1 {
                let key = request.reference.display().to_string();
                let voice = if let Some(&level) = voice_levels.get(&key) {
                    level
                } else {
                    let level = tools.audio.level(&request.reference)?;
                    voice_levels.insert(key, level);
                    level
                };
                synth::directed_target(level, voice)
            } else {
                synth::VOICE_LUFS
            };
            let finished = out.join(format!("{}.ogg", request.key));
            tools
                .audio
                .finish(&take, synth::gain_for(level, target), &finished)?;
            let file = w.file();
            let row = audit::judge(&file, &w.spoken, &tools.audio.measure(&finished)?);
            if row.flagged() {
                missed.push(w);
            }
            if best.get(&file).is_some_and(|b| b.over() <= row.over()) {
                continue;
            }
            let ogg = folder.dir.join(&file);
            if let Some(parent) = ogg.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("creating {}: {e}", parent.display()))?;
            }
            fs::copy(&finished, &ogg).map_err(|e| format!("writing {}: {e}", ogg.display()))?;
            record(loaded, w, (&tool, &model), date);
            best.insert(file, row);
        }
        save(folder, loaded)?;
        pending = missed;
    }
    // Scratch files in the system's temporary folder: failing to tidy up
    // is not a failure of the command.
    let _ = fs::remove_dir_all(&scratch);
    Ok(Batch {
        made: best.len(),
        failed: pending
            .iter()
            .map(|w| w.file())
            .filter(|file| !best.contains_key(file))
            .collect(),
        flagged: best.into_values().filter(audit::Row::flagged).collect(),
    })
}

/// The request for `w`'s take.
fn request(folder: &Folder, loaded: &Loaded, w: &Job<'_>) -> Request {
    Request {
        key: plan::key(w),
        text: w.sent.clone(),
        direction: w.direction.clone(),
        reference: reference(folder, &loaded.cast.voices[&w.key].voice),
        reference_text: REFERENCE_TEXT.to_owned(),
    }
}

/// Writes `w`'s new clip into the manifest and `sent.ron`.
fn record(loaded: &mut Loaded, w: &Job<'_>, (tool, model): (&str, &str), date: &str) {
    let clip_name = loaded.cast.voices[&w.key].voice.clone();
    let clip = VoiceClip {
        line: w.line.clone(),
        variant: w.variant,
        spoken: w.spoken.clone(),
        direction: w.direction.clone(),
        voice: clip_name.clone(),
        made_by: MadeBy::Generated {
            tool: tool.to_owned(),
            model: model.to_owned(),
            date: date.to_owned(),
        },
    };
    loaded
        .clips
        .retain(|c| (&c.line, c.variant) != (&w.line, w.variant));
    loaded.clips.push(clip);
    let reference = loaded
        .references
        .get(&clip_name)
        .cloned()
        .unwrap_or_default();
    loaded.sent.insert(
        w.file(),
        Sent {
            text: w.sent.clone(),
            reference,
        },
    );
}

/// `audit`: every clip of the manifest in `scripts`' scenes, judged by its
/// length, pace and pauses (the [`audit`](mod@audit) module). Returns the report.
pub fn audit(folder: &Folder, scripts: &Scripts<'_>, tools: &Tools) -> Result<String, String> {
    let clips = read_optional(&folder.dir, MANIFEST_FILE)?
        .map_or(Ok(Vec::new()), |t| files::manifest(&t))?;
    let mut rows = Vec::new();
    let ours = clips
        .iter()
        .filter(|c| scripts.scenes.contains(c.line.scene_id()));
    for clip in ours {
        let file = trpg_content::voice::clip_file(&clip.line, clip.variant);
        let measure = tools.audio.measure(&folder.dir.join(&file))?;
        rows.push(audit::judge(&file, &clip.spoken, &measure));
    }
    rows.sort_by(|a, b| a.file.cmp(&b.file));
    Ok(audit::report(&rows))
}

/// Runs `args` from the repository at `root`. Returns the report.
pub fn run(root: &Path, args: &Args) -> Result<String, String> {
    let content = trpg_content::load_embedded().map_err(|e| e.to_string())?;
    let mut scripts = Scripts::new(&content, BTreeSet::new());
    scripts.scenes = match &args.scene {
        Some(id) if scripts.dialogue.get(id).is_some() => BTreeSet::from([id.clone()]),
        Some(id) => return Err(format!("no scene \"{id}\"")),
        None => story_scenes(root)?,
    };
    let voice = args
        .dir
        .clone()
        .unwrap_or_else(|| root.join("assets-private/voice"));
    let folder = Folder {
        dir: voice.join(&args.lang),
        refs: root.join(REFS_DIR),
    };
    let lab = args.lab.clone().unwrap_or_else(|| {
        std::env::var_os("VOICE_LAB").map_or_else(|| PathBuf::from("D:/voice-lab"), PathBuf::from)
    });
    let mut tools = Tools::lab(&lab, &root.join("tools/voice/lab.py"), &args.lang);
    match &args.command {
        Command::Status => status(&folder, &scripts, &tools),
        Command::Generate { dry_run } => {
            generate(&folder, &scripts, &mut tools, *dry_run, &today())
        }
        Command::Script { character } => script(&folder, &scripts, &tools, character),
        Command::Audit => audit(&folder, &scripts, &tools),
    }
}

#[cfg(test)]
mod tests;
