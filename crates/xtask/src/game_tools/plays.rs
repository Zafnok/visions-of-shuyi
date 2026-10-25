//! The game's content check, bots and replay for Shuyi's tools (ticket
//! 1193): `check` is every validator of `trpg-content`; `bots` plays the
//! story's first battle with the baseline bot (`trpg-bots`), each play
//! written as a [`PlayRecord`]; `replay` re-runs one by the rules.

use serde::{Deserialize, Serialize};
use shuyi_assets::ContentError;
use shuyi_cli::contract::bots::{BotsInput, BotsOutput, Play, UNFINISHED, tally};
use shuyi_cli::contract::replay::{Record, ReplayRun, turns};
use shuyi_core::Bot;
use shuyi_core::bots::play_observed;
use shuyi_core::bots::playtest::Entrant as _;
use trpg_bots::{BattleEntrant, BattleStage, Player};
use trpg_content::Content;
use trpg_core::{BattleState, Command, GameMode};

/// The [`PlayRecord`] format's version.
const VERSION: u32 = 1;

/// A battle played: what rebuilds it and every command applied, nothing
/// about the look (ADR-0038 §5). RON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayRecord {
    /// The format's version ([`VERSION`]).
    pub version: u32,
    /// The battle, by id (`assets/battles/`).
    pub battle: String,
    /// The mode it was played in.
    pub mode: GameMode,
    /// Its seed.
    pub seed: u64,
    /// Every command applied, in order.
    pub commands: Vec<Command>,
}

/// Every error of every validator of the game's content.
pub fn check() -> Vec<ContentError> {
    trpg_content::load_embedded()
        .err()
        .map_or_else(Vec::new, |e| e.0)
}

/// `input`'s games of the story's first battle, the baseline bot on the
/// player's side in Classic.
pub fn bots(input: &BotsInput) -> Result<BotsOutput, String> {
    let content = content()?;
    let battle = first_battle(&content)?;
    let stage = stage(&content, &battle)?;
    let mode = GameMode::Classic;
    let entrant = BattleEntrant::new("baseline", mode, Player::Baseline, content.ai, &stage);
    let seeds = input.seed..input.seed + u64::from(input.games);
    let plays = seeds.map(|seed| {
        // As the engine's `play_tries`, keeping the commands for the record.
        let mut rules = entrant.start(&stage, seed);
        let mut bots = entrant.bots(seed);
        let mut by_side: Vec<_> = bots
            .iter_mut()
            .map(|(side, bot)| (*side, bot.as_mut() as &mut dyn Bot<BattleState>))
            .collect();
        let mut measure = entrant.measure();
        let report = play_observed(&mut rules, &mut by_side, input.turns, |r, c, e| {
            measure.observe(r, c, e);
        })
        .map_err(|e| format!("seed {seed}: {e}"))?;
        let outcome = report
            .outcome
            .map_or(UNFINISHED.to_owned(), |o| format!("{o:?}"));
        let commands = u32::try_from(report.commands.len()).unwrap_or(u32::MAX);
        let mut play = Play::new(seed, outcome, report.turns, commands);
        play.measures = measure.finish(&report, &rules);
        let record = PlayRecord {
            version: VERSION,
            battle: battle.clone(),
            mode,
            seed,
            commands: report.commands,
        };
        play.record = Some(ron::ser::to_string(&record).map_err(|e| e.to_string())?);
        Ok(play)
    });
    Ok(tally(plays.collect::<Result<_, String>>()?))
}

/// `record`, a [`PlayRecord`], re-run by the rules from its battle and seed.
pub fn replay(record: &Record) -> Result<ReplayRun, String> {
    let played: PlayRecord =
        ron::from_str(&record.text).map_err(|e| format!("{}: {e}", record.file))?;
    if played.version != VERSION {
        return Err(format!(
            "{}: a record of version {}, this game reads {VERSION}",
            record.file, played.version
        ));
    }
    let content = content()?;
    let mut setup = stage(&content, &played.battle)?.setup(played.mode).clone();
    setup.seed = played.seed;
    turns(&record.file, BattleState::new(setup).0, &played.commands)
}

/// The game's content, loaded and checked.
fn content() -> Result<Content, String> {
    trpg_content::load_embedded().map_err(|e| e.to_string())
}

/// The id of the story's first battle.
fn first_battle(content: &Content) -> Result<String, String> {
    let first = &content.new_game.first_chapter;
    let chapter = content.chapters.get(first);
    let chapter = chapter.ok_or_else(|| format!("no chapter \"{first}\""))?;
    Ok(chapter.battle.clone())
}

/// The battle `battle` as the bots play it.
fn stage(content: &Content, battle: &str) -> Result<BattleStage, String> {
    let def = content.battles.get(battle);
    let def = def.ok_or_else(|| format!("no battle \"{battle}\" in assets/battles"))?;
    Ok(crate::playtest::stage(content, def))
}
