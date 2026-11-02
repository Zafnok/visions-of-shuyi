//! `cargo xtask skirmish new` (ticket 0510, `docs/playtesting.md`,
//! *Generating skirmishes*): generates a skirmish from a request
//! ([`trpg_bots::generate`]), lands the three personas in their bands with
//! the engine's autobalance, and writes the battle, its map and its place
//! on the world map, or its entry in the random pool. Files are written
//! only once the result passes content validation and its bands.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError};

use shuyi_cli::contract::Answer;
use shuyi_cli::contract::autobalance::{
    AutobalanceInput, AutobalanceOutput, autobalance, parse_args as search_args, text,
};
use shuyi_cli::frame_png::{Script, Shot};
use shuyi_cli::game::{GameLang, GameOption, GameTools, unsupported};
use shuyi_cli::{AutobalancePlan, GameArg};
use shuyi_core::bots::playtest::Tries;
use trpg_bots::{
    Kind, Request, TunedBattle, Units, add_place, add_to_pool, generate, generated_knobs,
};
use trpg_content::battle::tune::BattleText;
use trpg_content::battle::{BATTLES_DIR, pool_source_fits};
use trpg_content::map::parse_map;
use trpg_content::worldmap::{self, WORLDMAPS_DIR, WorldRefs};
use trpg_content::{Content, bundle};
use trpg_core::{BattleDef, SkirmishRules};

use crate::game_tools::balance::tuned_plan;

/// The command's usage text.
pub const USAGE: &str = "usage: cargo xtask skirmish new --at <x,y> --level <n> [options]\n\n\
Generates a skirmish on a world map, lands the three bots in their bands\n\
and writes the battle, its map and its place (docs/playtesting.md,\n\
Generating skirmishes). Run it optimised: the bots think a lot.\n\n\
options:\n  \
--at <x,y>         the cell on the world map (required)\n  \
--level <n>        the battle's level: enemies around it, the bots' army at it\n                     \
(required)\n  \
--units <what>     mix (default), a class or a weapon type: mage, axe\n  \
--chapters <list>  the chapters it shows in, by number or id: 2 or ch02,ch03\n                     \
(default all); the bots' army is the earliest's\n  \
--kind <kind>      fixed (default): a place of its own; random: an entry in\n                     \
the map's random pool, on no place\n  \
--tier <tier>      easy or normal (default normal; random is always easy)\n  \
--map <id>         the world map (default the story's first, else a test map)\n  \
--name <text>      the place's name on the map (default Skirmish)\n  \
--seed <n>         the generator's seed (default 1)\n  \
--force            write the files even outside the bands\n  \
--tries, --quick, --budget, --time, --turns, --threads, --entrant <v>\n                     \
the search's, as cargo shuyi autobalance has them";

/// The command's arguments.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Options {
    at: Option<(i32, i32)>,
    level: Option<u32>,
    units: Option<Units>,
    chapters: Vec<String>,
    random: bool,
    tier: Option<String>,
    map: Option<String>,
    name: Option<String>,
    seed: u64,
    force: bool,
    /// The search's own options, as given.
    search: Vec<String>,
}

fn parse_args(args: &[String]) -> Result<Options, String> {
    let mut options = Options {
        seed: 1,
        ..Options::default()
    };
    let mut iter = args.iter();
    match iter.next().map(String::as_str) {
        Some("new") => {}
        Some(other) => return Err(format!("unknown subcommand: {other}")),
        None => return Err("no subcommand given".to_owned()),
    }
    while let Some(arg) = iter.next() {
        if arg == "--force" {
            options.force = true;
            continue;
        }
        let value = iter
            .next()
            .ok_or_else(|| format!("{arg} requires a value"))?;
        let bad = || format!("{arg}: bad value \"{value}\"");
        match arg.as_str() {
            "--at" => options.at = Some(parse_cell(value).ok_or_else(bad)?),
            "--level" => options.level = Some(value.parse().map_err(|_| bad())?),
            "--units" => options.units = Some(parse_units(value)),
            "--chapters" => options.chapters = value.split(',').map(str::to_owned).collect(),
            "--kind" => {
                options.random = match value.as_str() {
                    "fixed" => false,
                    "random" => true,
                    _ => return Err(bad()),
                };
            }
            "--tier" => {
                let tier = match value.as_str() {
                    "easy" => "Easy",
                    "normal" => "Normal",
                    _ => return Err(bad()),
                };
                options.tier = Some(tier.to_owned());
            }
            "--map" => options.map = Some(value.clone()),
            "--name" => options.name = Some(value.clone()),
            "--seed" => options.seed = value.parse().map_err(|_| bad())?,
            "--tries" | "--quick" | "--budget" | "--time" | "--turns" | "--threads"
            | "--entrant" => options.search.extend([arg.clone(), value.clone()]),
            _ => return Err(format!("unknown option: {arg}")),
        }
    }
    if options.at.is_none() || options.level.is_none() {
        return Err("--at and --level are required".to_owned());
    }
    if options.random && !options.chapters.is_empty() {
        return Err("--chapters: a random skirmish spawns from the pool, in no chapter".to_owned());
    }
    if options.random && options.tier.as_deref() == Some("Normal") {
        return Err("--tier: a random skirmish is always easy".to_owned());
    }
    Ok(options)
}

fn parse_cell(value: &str) -> Option<(i32, i32)> {
    let (x, y) = value.split_once(',')?;
    Some((x.trim().parse().ok()?, y.trim().parse().ok()?))
}

fn parse_units(value: &str) -> Units {
    if value == "mix" {
        Units::Mix
    } else {
        Units::Only(value.to_owned())
    }
}

/// The command: its exit code.
pub fn command(root: &Path, args: &[String]) -> u8 {
    if args.iter().any(|a| a == "--help") {
        println!("{USAGE}");
        return 0;
    }
    let options = match parse_args(args) {
        Ok(options) => options,
        Err(e) => {
            eprintln!("{e}\n\n{USAGE}");
            return 2;
        }
    };
    match run(root, &options) {
        Ok(ran) => {
            print!("{}", ran.report);
            for file in &ran.written {
                println!("wrote {file}");
            }
            if ran.written.is_empty() {
                eprintln!(
                    "skirmish: outside its bands, nothing written (another --seed, or --force)"
                );
                return 1;
            }
            0
        }
        Err(e) => {
            eprintln!("skirmish: {e}");
            1
        }
    }
}

/// What a run did.
#[derive(Debug)]
struct Ran {
    /// The search's report.
    report: String,
    /// The files written, none if the search missed its bands.
    written: Vec<String>,
}

/// The world map a request without `--map` goes on: the story's first, else
/// the first of the test files', with the story having no map yet.
fn default_map(content: &Content) -> Result<String, String> {
    let shipped = |id: &&String| {
        !bundle::display_path(&format!("{WORLDMAPS_DIR}/{id}.ron")).starts_with("assets-test")
    };
    let mut maps = content.worldmaps.keys();
    let first = content.worldmaps.keys().find(shipped);
    first
        .or_else(|| maps.next())
        .cloned()
        .ok_or_else(|| "the game has no world map".to_owned())
}

/// `--chapters` as chapter ids: a number is the chapter with that number.
fn chapter_ids(content: &Content, chapters: &[String]) -> Result<Vec<String>, String> {
    chapters
        .iter()
        .map(|given| match given.parse::<u32>() {
            Ok(number) => content
                .chapters
                .values()
                .find(|c| c.number == Some(number))
                .map(|c| c.id.clone())
                .ok_or_else(|| format!("no chapter {number}")),
            Err(_) => Ok(given.clone()),
        })
        .collect()
}

/// `path` under `root`, where a test run writes instead of the repository.
fn file_at(root: &Path, path: &str) -> PathBuf {
    root.join(path)
}

/// Where a file of the same data folder as `world` goes: the test files'
/// beside a test map, the story's beside a story map.
fn beside(world: &str, path: &str) -> String {
    let data = world.split('/').next().unwrap_or("assets");
    format!("{data}/{path}")
}

fn run(root: &Path, options: &Options) -> Result<Ran, String> {
    let mut content = trpg_content::load_embedded().map_err(|e| format!("content: {e}"))?;
    let request = request_of(&content, options)?;
    let map = request.map.clone();
    let kind = request.kind;
    let world = content
        .worldmaps
        .get(&map)
        .cloned()
        .ok_or_else(|| format!("no world map \"{map}\""))?;
    let made = generate(&content, &world, &request)?;

    let world_path = bundle::display_path(&format!("{WORLDMAPS_DIR}/{map}.ron"));
    let battle_path = beside(&world_path, &format!("{BATTLES_DIR}/{}.ron", made.id));
    let map_path = beside(&world_path, &format!("maps/{}.map", made.id));
    if file_at(root, &battle_path).exists() {
        return Err(format!(
            "{battle_path} already exists: delete it to generate it again"
        ));
    }
    let parsed = parse_map(&map_path, &made.map_text, &content.terrain.display)
        .map_err(|e| errors_text(&e))?;
    content.maps.insert(made.id.clone(), parsed);
    let content = Arc::new(content);

    let authored =
        BattleText::parse(&battle_path, &made.id, &made.battle_text).map_err(|e| e.to_string())?;
    let battle = TunedBattle::new(authored, Arc::clone(&content));
    let def = battle.load()?;
    // Every other file's change is known now: no search for a request that
    // can't be written.
    let (other_path, other_text) = match kind {
        Kind::Fixed => {
            let source = read(root, &world_path)?;
            let placed = add_place(&source, &world, &request, &made.id)?;
            check_world(&content, &map, &world_path, &placed, &def)?;
            (world_path, placed)
        }
        Kind::Random => {
            let path = beside(&world_path, &format!("skirmishes/{map}.ron"));
            let source = read(root, &path)
                .map_err(|e| format!("world map \"{map}\" has no random pool: {e}"))?;
            (path, add_to_pool(&source, &made.id)?)
        }
    };

    let knobs = generated_knobs(&battle, options.random)?;
    let found: Arc<Mutex<Option<String>>> = Arc::default();
    let keep = Arc::clone(&found);
    let write = Box::new(move |changed: &TunedBattle| {
        *keep.lock().unwrap_or_else(PoisonError::into_inner) = Some(changed.text.text());
        Ok(Vec::new())
    });
    let plan = tuned_plan(root, &made.id, battle, knobs, write)?;
    let answer = search(plan, &options.search)?;
    let Some(output) = answer.result.as_ref() else {
        return Err(answer.error.unwrap_or_else(|| "no answer".to_owned()));
    };
    // The engine's apply line speaks of its own write, which only hands the
    // text back: what this command writes it says itself.
    let report: String = text(output)
        .split_inclusive('\n')
        .filter(|line| !line.starts_with("apply:"))
        .collect();
    if !output.in_bands && !options.force {
        return Ok(Ran {
            report,
            written: Vec::new(),
        });
    }

    let changed = found.lock().unwrap_or_else(PoisonError::into_inner).take();
    let battle_text = changed.unwrap_or_else(|| made.battle_text.clone());
    let def = trpg_content::battle::from_source(
        &battle_path,
        &made.id,
        &battle_text,
        &content.battle_refs(),
    )
    .map_err(|e| errors_text(&e))?;
    match kind {
        Kind::Fixed => check_world(&content, &map, &other_path, &other_text, &def)?,
        Kind::Random => check_pool(&other_path, &other_text, &battle_text, &def)?,
    }
    let files = [
        (map_path, made.map_text.as_str()),
        (battle_path, battle_text.as_str()),
        (other_path, other_text.as_str()),
    ];
    let written = write_files(root, &files)?;
    Ok(Ran { report, written })
}

/// `options` as a request of the generator.
fn request_of(content: &Content, options: &Options) -> Result<Request, String> {
    let map = match &options.map {
        Some(map) => map.clone(),
        None => default_map(content)?,
    };
    let (kind, tier) = if options.random {
        (Kind::Random, "Easy")
    } else {
        (Kind::Fixed, "Normal")
    };
    Ok(Request {
        map,
        at: options.at.unwrap_or_default(),
        level: options.level.unwrap_or_default(),
        units: options.units.clone().unwrap_or(Units::Mix),
        chapters: chapter_ids(content, &options.chapters)?,
        kind,
        tier: options.tier.clone().unwrap_or_else(|| tier.to_owned()),
        seed: options.seed,
        name: options
            .name
            .clone()
            .unwrap_or_else(|| "Skirmish".to_owned()),
    })
}

/// Writes each `(path, text)` under `root`: the paths written.
fn write_files(root: &Path, files: &[(String, &str)]) -> Result<Vec<String>, String> {
    let mut written = Vec::new();
    for (path, text) in files {
        let file = file_at(root, path);
        if let Some(folder) = file.parent() {
            std::fs::create_dir_all(folder).map_err(|e| format!("{path}: {e}"))?;
        }
        std::fs::write(&file, text).map_err(|e| format!("{path}: {e}"))?;
        written.push(path.clone());
    }
    Ok(written)
}

fn read(root: &Path, path: &str) -> Result<String, String> {
    std::fs::read_to_string(file_at(root, path)).map_err(|e| format!("{path}: {e}"))
}

fn errors_text<E: ToString>(errors: &[E]) -> String {
    let lines: Vec<String> = errors.iter().map(ToString::to_string).collect();
    lines.join("\n")
}

/// Whether world map text `text` passes its checks with battle `def` in the
/// tables.
fn check_world(
    content: &Content,
    map: &str,
    path: &str,
    text: &str,
    def: &BattleDef,
) -> Result<(), String> {
    let mut battles = content.battles.clone();
    battles.insert(def.id.clone(), def.clone());
    let refs = WorldRefs {
        battles: &battles,
        chapters: &content.chapters,
        dialogue: &content.dialogue,
        characters: &content.characters,
        items: &content.items,
        audio: &content.audio,
    };
    worldmap::from_source(path, map, text, refs)
        .map(|_| ())
        .map_err(|e| errors_text(&e))
}

/// Whether pool text `text` still parses and `battle_text` fits it.
fn check_pool(path: &str, text: &str, battle_text: &str, def: &BattleDef) -> Result<(), String> {
    let rules: SkirmishRules = ron::from_str(text).map_err(|e| format!("{path}: {e}"))?;
    if pool_source_fits(battle_text, def, rules.size.1) {
        Ok(())
    } else {
        Err(format!("{}: doesn't fit the random pool of {path}", def.id))
    }
}

/// One plan for the engine's `autobalance`, which asks a game for it: the
/// parts of a game it doesn't use are never asked.
struct Generating(Mutex<Option<AutobalancePlan>>);

impl GameTools for Generating {
    fn frame_options(&self) -> &[GameOption] {
        &[]
    }

    fn frame(&self, _: &[GameArg], _: &Script, _: Option<u32>) -> Result<Shot, String> {
        Err(unsupported("frame-png", "frame"))
    }

    fn lang(&self) -> Result<GameLang, String> {
        Err(unsupported("lang-status", "lang"))
    }

    fn playtest_defaults(&self) -> Tries {
        Tries::new(100, 1, 60)
    }

    fn autobalance_plan(
        &self,
        _: Option<&str>,
        _: &AutobalanceInput,
    ) -> Result<AutobalancePlan, String> {
        let mut plan = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        plan.take().ok_or_else(|| "the plan is spent".to_owned())
    }
}

/// The engine's search of `plan`, with the search's options `given`.
fn search(plan: AutobalancePlan, given: &[String]) -> Result<Answer<AutobalanceOutput>, String> {
    let mut args = given.to_vec();
    args.push("--apply".to_owned());
    let input = search_args(&args, &[])?;
    let game = Generating(Mutex::new(Some(plan)));
    Ok(autobalance(&game, &input))
}

#[cfg(test)]
mod tests;
