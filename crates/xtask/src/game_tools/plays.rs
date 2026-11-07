//! The game's content check, bots and replay for Shuyi's tools (ticket
//! 1193): `check` is every validator of `trpg-content`; `playtest` with no
//! checks plays the story's first battle with the baseline bot
//! (`trpg-bots`), each play written as a [`PlayRecord`]; `replay` re-runs
//! one by the rules, or the battle record a crash report holds (ticket
//! 1245, [`BattleRecord`]).

use serde::{Deserialize, Serialize};
use shuyi_assets::ContentError;
use shuyi_cli::PlaytestPlan;
use shuyi_cli::contract::replay::{Record, ReplayRun, turns};
use shuyi_core::Bot;
use shuyi_core::bots::play_observed;
use shuyi_core::bots::playtest::{Entrant as _, PlayedTry, Tried};
use trpg_bots::{BattleEntrant, BattleStage, Player};
use trpg_content::Content;
use trpg_core::{BattleState, Command, GameMode};
use trpg_ui::crash::{self, BattleRecord};

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

/// The bot run: `battle` (by id), else the story's first battle, the
/// baseline bot on the player's side in Classic, as one entrant with no
/// checks.
pub fn plan(battle: Option<&str>) -> Result<PlaytestPlan, String> {
    let content = content()?;
    let battle = battle.map_or_else(|| first_battle(&content), |b| Ok(b.to_owned()))?;
    let stage = stage(&content, &battle)?;
    let mode = GameMode::Classic;
    let entrant = BattleEntrant::new("baseline", mode, Player::Baseline, content.ai, &stage);
    let title = format!("{battle} · the baseline bot in {mode:?}");
    let play = Box::new(move |_: usize, seed, turns| {
        // As the engine's `play_try`, keeping the commands for the record.
        let mut rules = entrant.start(&stage, seed);
        let mut bots = entrant.bots(seed);
        let mut by_side: Vec<_> = bots
            .iter_mut()
            .map(|(side, bot)| (*side, bot.as_mut() as &mut dyn Bot<BattleState>))
            .collect();
        let mut measure = entrant.measure();
        let report = play_observed(&mut rules, &mut by_side, turns, |r, c, e| {
            measure.observe(r, c, e);
        })
        .map_err(|e| format!("seed {seed}: {e}"))?;
        let tried = Tried {
            seed,
            outcome: report.outcome.map(|o| format!("{o:?}")),
            turns: report.turns,
            measures: measure.finish(&report, &rules),
        };
        let commands = u32::try_from(report.commands.len()).unwrap_or(u32::MAX);
        let record = PlayRecord {
            version: VERSION,
            battle: battle.clone(),
            mode,
            seed,
            commands: report.commands,
        };
        let mut played = PlayedTry::new(tried, commands);
        played.record = Some(ron::ser::to_string(&record).map_err(|e| e.to_string())?);
        Ok(played)
    });
    let entrants = vec![("baseline".to_owned(), None)];
    Ok(PlaytestPlan::new(
        title,
        entrants,
        Box::new(|_| Vec::new()),
        play,
    ))
}

/// `record`, a [`PlayRecord`], re-run by the rules from its battle and
/// seed; or a crash report's [`BattleRecord`], from its first state.
pub fn replay(record: &Record) -> Result<ReplayRun, String> {
    if let Ok(crashed) = ron::from_str::<BattleRecord>(&record.text) {
        return replay_crash(&record.file, crashed);
    }
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

/// A crash report's battle record re-run from its first state, to the
/// command the rules were applying when it crashed.
fn replay_crash(file: &str, mut crashed: BattleRecord) -> Result<ReplayRun, String> {
    if crashed.version != crash::VERSION {
        return Err(format!(
            "{file}: a crash record of version {}, this game reads {}",
            crashed.version,
            crash::VERSION
        ));
    }
    crashed.history.restore_tables(&content()?.tables());
    turns(file, crashed.history.state_at(0), &crashed.commands())
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
    crate::playtest::stage(content, def)
}
