//! The game's content check, bot run and replay for Shuyi's tools (ticket
//! 1193): `check` is every validator of `trpg-content`; [`plan`] is the
//! bot run `playtest` plays (1240): a battle with the baseline bot
//! (`trpg-bots`), each try written as a [`PlayRecord`]; `replay` re-runs
//! one by the rules.

use serde::{Deserialize, Serialize};
use shuyi_assets::ContentError;
use shuyi_cli::PlaytestPlan;
use shuyi_cli::contract::replay::{Record, ReplayRun, turns};
use shuyi_core::Bot;
use shuyi_core::bots::playtest::{Entrant, Measure, play_try};
use trpg_bots::{BattleEntrant, BattleStage, Player};
use trpg_content::Content;
use trpg_core::{BattleState, Command, GameMode, Phase};

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

/// The game's bot run, as `playtest` plays it (ticket 1240, engine#2544):
/// battle `stage` (the story's first if `None`), the baseline bot on the
/// player's side in Classic, nothing judged, each try written as a
/// [`PlayRecord`].
pub fn plan(stage_id: Option<&str>) -> Result<PlaytestPlan, String> {
    let content = content()?;
    let battle = match stage_id {
        Some(id) => id.to_owned(),
        None => first_battle(&content)?,
    };
    let stage = stage(&content, &battle)?;
    let mode = GameMode::Classic;
    let entrant = BattleEntrant::new(BASELINE, mode, Player::Baseline, content.ai, &stage);
    let recorded = Recorded {
        entrant,
        battle: battle.clone(),
    };
    Ok(PlaytestPlan::new(
        format!("{battle} · {BASELINE} bot · {mode:?}"),
        vec![(BASELINE.to_owned(), Some(format!("{mode:?}")))],
        Box::new(|_| Vec::new()),
        Box::new(move |_, seed, turns| {
            play_try(&stage, &recorded, seed, turns).map_err(|e| format!("seed {seed}: {e}"))
        }),
    ))
}

/// The bot run's one entrant.
const BASELINE: &str = "baseline";

/// A battle's entrant whose tries are kept as [`PlayRecord`]s of
/// `battle`.
struct Recorded {
    entrant: BattleEntrant,
    battle: String,
}

impl Entrant<BattleStage, BattleState> for Recorded {
    fn name(&self) -> &str {
        self.entrant.name()
    }

    fn start(&self, stage: &BattleStage, seed: u64) -> BattleState {
        self.entrant.start(stage, seed)
    }

    fn bots(&self, seed: u64) -> Vec<(Phase, Box<dyn Bot<BattleState>>)> {
        self.entrant.bots(seed)
    }

    fn measure(&self) -> Box<dyn Measure<BattleState>> {
        self.entrant.measure()
    }

    fn record(&self, seed: u64, commands: &[Command]) -> Option<String> {
        let record = PlayRecord {
            version: VERSION,
            battle: self.battle.clone(),
            mode: self.entrant.mode(),
            seed,
            commands: commands.to_vec(),
        };
        ron::ser::to_string(&record).ok()
    }
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
pub(super) fn content() -> Result<Content, String> {
    trpg_content::load_embedded().map_err(|e| e.to_string())
}

/// The id of the story's first battle.
pub(super) fn first_battle(content: &Content) -> Result<String, String> {
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
