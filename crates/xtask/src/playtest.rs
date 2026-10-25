//! `cargo xtask playtest` (tickets 0505 and 0506, ADR-0033): bots play a
//! battle file many times, each try with fresh luck, and the report says
//! how the tries went and whether each persona met its target
//! (`docs/playtesting.md`, `docs/design/playtest-bots.md`).
//!
//! The tries, the checks and the table are the engine's playtest runner
//! (`shuyi_core::bots::playtest`), with the game's bots, measures and
//! targets from `trpg-bots`. This wrapper keeps the options, the threads,
//! the clock, the history and the game's own report lines. A report
//! depends only on the arguments: every seed range is judged once, sorted
//! by seed, so the thread count changes nothing but the speed lines.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use shuyi_core::bots::playtest::{
    Check, Entrant as _, Found, PlaytestReport, Tried, judge, play_tries,
};
use trpg_bots::{BattleEntrant, BattleStage, Player, Targets, personas_from_ron};
use trpg_content::{Content, battle_campaign};
use trpg_core::{BattleDef, GameMode, LeadGender, LeadProfile, Turn};

/// The command's usage text.
pub const USAGE: &str = "usage: cargo xtask playtest <battle-id> [options]\n\n\
Bots play the battle assets/battles/<battle-id>.ron many times, each try\n\
with fresh luck, and report how the tries went and whether each persona\n\
met its target (docs/playtesting.md). Exits with 1 when the battle is\n\
blocked: Casual succeeds in under 20% of tries.\n\n\
options:\n  \
--bot <bot>        who plays the player's side: baseline (default), casual,\n                     \
normal, hardcore, or all (the three personas)\n  \
--runs <n>         tries per bot (default 100)\n  \
--seed <n>         seed of the first try; try i plays seed + i (default 1)\n  \
--mode <mode>      the baseline's mode: classic or casual (default classic);\n                     \
a persona plays its own\n  \
--turn-cap <n>     stop a try after this many turns (default 60)\n  \
--json <path>      also write the report as JSON\n  \
--history <dir>    where every run is kept, to compare the next one with\n                     \
(default target/playtest-history/)\n  \
--threads <n>      threads to play on (default: every core)";

/// Where runs are kept, from the repo root. Not committed.
const DEFAULT_HISTORY: &str = "target/playtest-history";

/// The personas' settings and the targets, from the repo root (dev
/// tooling, not game content: ticket 0506).
const PERSONAS_FILE: &str = "crates/bots/personas.ron";
const TARGETS_FILE: &str = "crates/bots/targets.ron";

/// The lead's name in a playtest.
const LEAD_NAME: &str = "Lead";

/// Who plays the player's side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bot {
    /// The enemy AI on the player's side.
    Baseline,
    /// The Casual persona.
    Casual,
    /// The Normal persona.
    Normal,
    /// The Hardcore persona.
    Hardcore,
    /// The three personas.
    All,
}

impl Bot {
    fn parse(name: &str) -> Option<Bot> {
        match name {
            "baseline" => Some(Bot::Baseline),
            "casual" => Some(Bot::Casual),
            "normal" => Some(Bot::Normal),
            "hardcore" => Some(Bot::Hardcore),
            "all" => Some(Bot::All),
            _ => None,
        }
    }

    /// Its name, as `--bot` takes it.
    pub fn name(self) -> &'static str {
        match self {
            Bot::Baseline => "baseline",
            Bot::Casual => "casual",
            Bot::Normal => "normal",
            Bot::Hardcore => "hardcore",
            Bot::All => "all",
        }
    }

    /// The entrants it stands for, by name.
    fn entrants(self) -> Vec<&'static str> {
        match self {
            Bot::All => vec!["casual", "normal", "hardcore"],
            one => vec![one.name()],
        }
    }
}

/// The command's arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The battle's id (its file stem).
    pub battle: String,
    /// Tries per bot.
    pub runs: u32,
    /// Seed of the first try.
    pub seed: u64,
    /// The baseline's mode.
    pub mode: GameMode,
    /// A try stops when this turn has passed.
    pub turn_cap: Turn,
    /// Who plays.
    pub bot: Bot,
    /// Where to write the report as JSON.
    pub json: Option<PathBuf>,
    /// Where runs are kept; `None` is [`DEFAULT_HISTORY`] in the repo.
    pub history: Option<PathBuf>,
    /// Threads to play on; `None` is every core.
    pub threads: Option<usize>,
}

impl Options {
    /// The defaults, for `battle`.
    pub fn new(battle: impl Into<String>) -> Self {
        Self {
            battle: battle.into(),
            runs: 100,
            seed: 1,
            mode: GameMode::Classic,
            turn_cap: 60,
            bot: Bot::Baseline,
            json: None,
            history: None,
            threads: None,
        }
    }
}

/// Parses the command's arguments.
pub fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options::new("");
    let mut battle: Option<&String> = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if !arg.starts_with("--") {
            if battle.is_some() {
                return Err(format!("unexpected argument: {arg}"));
            }
            battle = Some(arg);
            continue;
        }
        let value = iter
            .next()
            .ok_or_else(|| format!("{arg} requires a value"))?;
        let bad = || format!("{arg}: bad value \"{value}\"");
        match arg.as_str() {
            "--runs" => options.runs = positive(value).ok_or_else(bad)?,
            "--seed" => options.seed = value.parse().map_err(|_| bad())?,
            "--mode" => options.mode = parse_mode(value).ok_or_else(bad)?,
            "--turn-cap" => options.turn_cap = positive(value).ok_or_else(bad)?,
            "--bot" => options.bot = Bot::parse(value).ok_or_else(bad)?,
            "--json" => options.json = Some(PathBuf::from(value)),
            "--history" => options.history = Some(PathBuf::from(value)),
            "--threads" => {
                let threads = positive(value).and_then(|n| usize::try_from(n).ok());
                options.threads = Some(threads.ok_or_else(bad)?);
            }
            _ => return Err(format!("unknown option: {arg}")),
        }
    }
    battle
        .ok_or("no battle given")?
        .clone_into(&mut options.battle);
    Ok(options)
}

/// A number of at least 1.
fn positive(value: &str) -> Option<u32> {
    value.parse().ok().filter(|&n| n > 0)
}

fn parse_mode(name: &str) -> Option<GameMode> {
    match name {
        "classic" => Some(GameMode::Classic),
        "casual" => Some(GameMode::Casual),
        _ => None,
    }
}

fn mode_name(mode: GameMode) -> &'static str {
    match mode {
        GameMode::Classic => "Classic",
        GameMode::Casual => "Casual",
    }
}

/// One entrant of a run as the report shows it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Played {
    /// Its name.
    pub name: String,
    /// `Classic` or `Casual`.
    pub mode: String,
    /// Wall time per try, in seconds (the speed line; it differs between
    /// two runs with the same arguments).
    pub seconds_per_try: f64,
}

/// A run as kept in the history and written by `--json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Record {
    /// When it ran, e.g. `2026-10-01 14:05 UTC`.
    pub date: String,
    /// The git commit it ran on (short hash), or `unknown`.
    pub commit: String,
    /// The battle's id.
    pub battle: String,
    /// The battle's tier (its `difficulty`).
    pub tier: String,
    /// The turn cap.
    pub turn_cap: Turn,
    /// Who played, in order.
    pub played: Vec<Played>,
    /// The engine's report: every try's measures and the checks.
    pub report: PlaytestReport,
}

/// What a run found: the report's text, and whether the battle is blocked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ran {
    /// The report's text.
    pub text: String,
    /// Whether a persona fell under its floor (the exit code is 1).
    pub blocked: bool,
}

/// Runs the command: plays the tries, keeps the run in the history, writes
/// the JSON file if asked, and returns the report.
pub fn run(repo_root: &Path, options: &Options) -> Result<Ran, String> {
    let content = trpg_content::load_embedded().map_err(|e| format!("content: {e}"))?;
    let read = |file: &str| {
        std::fs::read_to_string(repo_root.join(file)).map_err(|e| format!("{file}: {e}"))
    };
    let record = Record {
        date: utc_stamp(now()),
        commit: git_commit(repo_root),
        ..play(
            &content,
            &read(PERSONAS_FILE)?,
            &read(TARGETS_FILE)?,
            options,
        )?
    };
    let default_history = repo_root.join(DEFAULT_HISTORY);
    let history = options.history.as_deref().unwrap_or(&default_history);
    let file = history_file(history, options);
    let previous = last_record(&file);
    append(&file, &record).map_err(|e| format!("{}: {e}", file.display()))?;
    if let Some(path) = &options.json {
        let json = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
        std::fs::write(path, json + "\n").map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(Ran {
        text: render(&record, previous.as_ref()),
        blocked: record.report.blocked,
    })
}

/// The battle `def` as the bots play it. The army: the characters of the
/// battle's player slots, fresh from the character data (as the debug
/// Quick Battle fields them).
pub fn stage(content: &Content, def: &BattleDef) -> BattleStage {
    let setup = |mode| {
        let lead = LeadProfile::new(LEAD_NAME, LeadGender::Male);
        battle_campaign(content, def, mode, lead).battle_setup(def, &content.tables())
    };
    BattleStage {
        classic: setup(GameMode::Classic),
        casual: setup(GameMode::Casual),
    }
}

/// Plays every try of `options` with the personas of `personas` (the text
/// of `personas.ron`) and judges them with `targets` (`targets.ron`'s):
/// the record, but for its date and commit.
pub fn play(
    content: &Content,
    personas: &str,
    targets: &str,
    options: &Options,
) -> Result<Record, String> {
    let Some(def) = content.battles.get(&options.battle) else {
        let known: Vec<&str> = content.battles.keys().map(String::as_str).collect();
        return Err(format!(
            "no battle \"{}\" in assets/battles (there: {})",
            options.battle,
            known.join(", ")
        ));
    };
    let stage = stage(content, def);
    let personas = personas_from_ron(personas)?;
    let targets = Targets::from_ron(targets)?;
    let entrants = options
        .bot
        .entrants()
        .into_iter()
        .map(|name| match personas.get(name) {
            Some(persona) => Ok(BattleEntrant::persona(name, *persona, content.ai, &stage)),
            None if name == "baseline" => Ok(BattleEntrant::new(
                name,
                options.mode,
                Player::Baseline,
                content.ai,
                &stage,
            )),
            None => Err(format!("no persona \"{name}\" in {PERSONAS_FILE}")),
        })
        .collect::<Result<Vec<_>, String>>()?;
    let names: Vec<&str> = entrants.iter().map(BattleEntrant::name).collect();
    let checks = targets.checks(def.difficulty, &names);

    let (tries, took) = play_on_threads(&stage, &entrants, options)?;
    let report = judge(tries, &checks).map_err(|e| e.to_string())?;
    let runs = f64::from(options.runs);
    let played = entrants
        .iter()
        .zip(took)
        .map(|(entrant, took)| Played {
            name: entrant.name().to_owned(),
            mode: mode_name(entrant.mode()).to_owned(),
            seconds_per_try: took.as_secs_f64() / runs,
        })
        .collect();
    Ok(Record {
        date: String::new(),
        commit: String::new(),
        battle: options.battle.clone(),
        tier: format!("{:?}", def.difficulty),
        turn_cap: options.turn_cap,
        played,
        report,
    })
}

/// One entrant's tries, by name.
type NamedTries = (String, Vec<Tried>);

/// A try done on a thread: its entrant's index, the try, and how long it
/// took; or why it stopped.
type Done = Result<(usize, Vec<Tried>, Duration), String>;

/// Every entrant's tries, played try by try on the threads (each try is
/// timed), with the time each entrant's tries took in all.
fn play_on_threads(
    stage: &BattleStage,
    entrants: &[BattleEntrant],
    options: &Options,
) -> Result<(Vec<NamedTries>, Vec<Duration>), String> {
    let cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let threads = options.threads.unwrap_or(cores).max(1);
    let work: Vec<(usize, u64)> = (0..entrants.len())
        .flat_map(|e| (0..options.runs).map(move |i| (e, u64::from(i))))
        .collect();
    let work = &work;
    let done: Vec<Done> = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|worker| {
                scope.spawn(move || {
                    work.iter()
                        .skip(worker)
                        .step_by(threads)
                        .map(|&(e, i)| {
                            let entrant = &entrants[e];
                            let seed = options.seed.wrapping_add(i);
                            let started = Instant::now();
                            let one = play_tries(stage, entrant, seed..seed + 1, options.turn_cap)
                                .map_err(|err| format!("{}: try {seed}: {err}", entrant.name()))?;
                            Ok((e, one, started.elapsed()))
                        })
                        .collect::<Vec<Done>>()
                })
            })
            .collect();
        let joined = workers.into_iter().flat_map(|worker| match worker.join() {
            Ok(done) => done,
            Err(panic) => std::panic::resume_unwind(panic),
        });
        joined.collect()
    });
    let mut tries: Vec<NamedTries> = entrants
        .iter()
        .map(|e| (e.name().to_owned(), Vec::new()))
        .collect();
    let mut took = vec![Duration::ZERO; entrants.len()];
    for one in done {
        let (e, mut played, elapsed) = one?;
        tries[e].1.append(&mut played);
        took[e] += elapsed;
    }
    Ok((tries, took))
}

/// The report's text: a header, the engine's table (the checks and every
/// try), then the game's own lines per entrant, with `previous`' numbers
/// as `was …` if there was a run before.
pub fn render(record: &Record, previous: Option<&Record>) -> String {
    let mut out = String::new();
    let names: Vec<&str> = record.played.iter().map(|p| p.name.as_str()).collect();
    // Writing to a `String` can't fail.
    let _ = writeln!(
        out,
        "{} · {} tier · {} · turn cap {}",
        record.battle,
        record.tier,
        names.join(", "),
        record.turn_cap
    );
    let _ = write!(out, "{}", record.report);
    out.push('\n');
    let width = names.iter().map(|n| n.chars().count()).max().unwrap_or(0);
    for played in &record.played {
        let Some(entrant) = record
            .report
            .entrants
            .iter()
            .find(|e| e.name == played.name)
        else {
            continue;
        };
        let lines = Lines::of(&played.mode, &entrant.tries);
        let pad = " ".repeat(width + 13);
        let _ = writeln!(
            out,
            "{:<width$} · {:<7} · won {}/{} · {} per try  {} · most: {}",
            played.name,
            played.mode,
            lines.won,
            lines.count,
            lines.falls_name,
            lines.falls_per_try(),
            lines.fell_most.as_deref().unwrap_or("nobody"),
        );
        let _ = writeln!(
            out,
            "{pad}items per try  median {}  max {} · turns median {} · {:.2} s per try{}",
            lines.items_median,
            lines.items_max,
            lines.turns_median,
            played.seconds_per_try,
            was(&record.report, previous, &played.name),
        );
    }
    if let Some(p) = previous {
        let _ = writeln!(out, "previous  {} · commit {}", p.date, p.commit);
    }
    out
}

/// The game's numbers of one entrant's tries.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Lines {
    count: usize,
    won: i64,
    /// `deaths` in Classic, `retreats` in Casual.
    falls_name: &'static str,
    /// Tries in which 0, 1, 2, and 3 or more player units fell.
    falls: [u32; 4],
    /// The unit that fell in the most tries, and how many (`Kael 41×`).
    fell_most: Option<String>,
    items_median: i64,
    items_max: i64,
    turns_median: i64,
}

impl Lines {
    fn of(mode: &str, tries: &[Tried]) -> Lines {
        let falls_name = if mode == "Casual" {
            "retreats"
        } else {
            "deaths"
        };
        let measure = |name: &str| -> Vec<i64> {
            tries
                .iter()
                .map(|t| t.measures.get(name).copied().unwrap_or(0))
                .collect()
        };
        let mut falls = [0; 4];
        for n in measure(falls_name) {
            falls[usize::try_from(n.clamp(0, 3)).unwrap_or(3)] += 1;
        }
        let mut fell: BTreeMap<&str, u32> = BTreeMap::new();
        for t in tries {
            let units = t.measures.keys().filter_map(|k| k.strip_prefix("fell "));
            for unit in units.filter(|u| !u.ends_with(" turn")) {
                *fell.entry(unit).or_default() += 1;
            }
        }
        // Most often first, then by name.
        let most = fell
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)));
        let items = measure("items");
        Lines {
            count: tries.len(),
            won: measure("won").iter().sum(),
            falls_name,
            falls,
            fell_most: most.map(|(unit, n)| format!("{unit} {n}×")),
            items_median: median(items.clone()),
            items_max: items.into_iter().max().unwrap_or(0),
            turns_median: median(measure("turns")),
        }
    }

    /// `0: 30  1: 21  2: 25  3+: 24`.
    fn falls_per_try(&self) -> String {
        let [none, one, two, more] = self.falls;
        format!("0: {none}  1: {one}  2: {two}  3+: {more}")
    }
}

/// The median of `values`, the lower middle one of an even count; 0 for
/// none.
fn median(mut values: Vec<i64>) -> i64 {
    values.sort_unstable();
    values
        .get(values.len().saturating_sub(1) / 2)
        .copied()
        .unwrap_or(0)
}

/// ` · was 47/100 succeed` (or ` · was won 63/100` for an entrant with no
/// check): `name`'s headline number in `previous`, if it played there.
fn was(report: &PlaytestReport, previous: Option<&Record>, name: &str) -> String {
    let Some(previous) = previous else {
        return String::new();
    };
    let rate = |report: &PlaytestReport| {
        report
            .checks
            .iter()
            .find_map(|j| match (&j.check, &j.found) {
                (
                    Check::Rate { entrant, .. },
                    Found::Rate {
                        successes, count, ..
                    },
                ) if entrant == name => Some((*successes, *count)),
                _ => None,
            })
    };
    if rate(report).is_some() {
        return rate(&previous.report)
            .map_or_else(String::new, |(s, n)| format!(" · was {s}/{n} succeed"));
    }
    let Some(entrant) = previous.report.entrants.iter().find(|e| e.name == name) else {
        return String::new();
    };
    let won: i64 = entrant
        .tries
        .iter()
        .filter_map(|t| t.measures.get("won"))
        .sum();
    format!(" · was won {won}/{}", entrant.tries.len())
}

/// The history file of `options`' battle and bot in `dir` (and mode, for
/// the baseline): one run per line, oldest first.
fn history_file(dir: &Path, options: &Options) -> PathBuf {
    let name = match options.bot {
        Bot::Baseline => format!(
            "{}-baseline-{}.jsonl",
            options.battle,
            mode_name(options.mode).to_lowercase()
        ),
        bot => format!("{}-{}.jsonl", options.battle, bot.name()),
    };
    dir.join(name)
}

/// The last run kept in `file`, if any.
fn last_record(file: &Path) -> Option<Record> {
    let text = std::fs::read_to_string(file).ok()?;
    text.lines()
        .rev()
        .find_map(|line| serde_json::from_str(line).ok())
}

/// Adds `record` to `file`, creating it and its folder if needed.
fn append(file: &Path, record: &Record) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let line = serde_json::to_string(record)?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)?;
    writeln!(file, "{line}")
}

/// The short hash of the commit checked out in `repo_root`, or `unknown`.
fn git_commit(repo_root: &Path) -> String {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["rev-parse", "--short", "HEAD"])
        .output();
    let hash = output
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned());
    hash.filter(|h| !h.is_empty())
        .unwrap_or_else(|| "unknown".to_owned())
}

/// Seconds since 1970-01-01 00:00 UTC.
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// `secs` since 1970-01-01 00:00 UTC as `2026-10-01 14:05 UTC`.
fn utc_stamp(secs: u64) -> String {
    let leap =
        |year: u64| year.is_multiple_of(4) && !year.is_multiple_of(100) || year.is_multiple_of(400);
    let mut days = secs / 86_400;
    let mut year = 1970;
    // Bounded, so a wrong count of days can't spin forever.
    for candidate in 1970..=9999 {
        year = candidate;
        let length = if leap(year) { 366 } else { 365 };
        if days < length {
            break;
        }
        days -= length;
    }
    let february = if leap(year) { 29 } else { 28 };
    let mut month = 1;
    for length in [31, february, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31] {
        if days < length {
            break;
        }
        days -= length;
        month += 1;
    }
    let minutes = secs % 86_400 / 60;
    format!(
        "{year}-{month:02}-{:02} {:02}:{:02} UTC",
        days + 1,
        minutes / 60,
        minutes % 60
    )
}

#[cfg(test)]
mod tests;
