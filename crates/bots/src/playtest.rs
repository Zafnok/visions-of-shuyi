//! The battle on the engine's playtest runner (ticket 0506): what a try
//! measures ([`BattleMeasure`]), who plays ([`BattleEntrant`]: the
//! baseline and the personas) and what they should achieve ([`Targets`],
//! `crates/bots/targets.ron`, turned into the engine's [`Check`]s for a
//! battle's tier). Playing the tries, judging them and the table are the
//! engine's (`shuyi_core::bots::playtest`).

use std::collections::{BTreeMap, HashMap};

use serde::Deserialize;
use shuyi_core::bots::playtest::{Band, Check, Condition, Entrant, Measure, Measures, Want};
use shuyi_core::{Bot, Report};
use trpg_core::{
    AiWeights, BattleSetup, BattleState, Command, Difficulty, Event, GameMode, Outcome, Phase,
    UnitId,
};

use crate::{BaselineBot, BattleMeasures, Persona, PersonaBot};

/// A battle as the runner plays it: its setup in each game mode (the army
/// a mode fields may differ), every try reseeding it.
#[derive(Debug, Clone)]
pub struct BattleStage {
    /// The battle in Classic.
    pub classic: BattleSetup,
    /// The battle in Casual.
    pub casual: BattleSetup,
}

impl BattleStage {
    /// The battle in `mode`.
    pub fn setup(&self, mode: GameMode) -> &BattleSetup {
        match mode {
            GameMode::Classic => &self.classic,
            GameMode::Casual => &self.casual,
        }
    }
}

/// Who plays the Player phase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Player {
    /// The enemy AI on the player's side ([`BaselineBot`]).
    Baseline,
    /// A persona ([`PersonaBot`]).
    Persona(Persona),
}

/// One named player of a battle for the engine's runner: its mode, its bot
/// for the Player phase, and the enemy AI for every other phase.
#[derive(Debug, Clone)]
pub struct BattleEntrant {
    name: String,
    mode: GameMode,
    player: Player,
    ai: AiWeights,
    /// Every unit's name and whether it is a lord, reinforcements included.
    cast: BTreeMap<UnitId, (String, bool)>,
    /// The measures of a try before its first command.
    start: BattleMeasures,
}

impl BattleEntrant {
    /// `player`, called `name`, playing `stage` in `mode`, the enemy AI
    /// (and the baseline) scoring with `ai`.
    pub fn new(
        name: impl Into<String>,
        mode: GameMode,
        player: Player,
        ai: AiWeights,
        stage: &BattleStage,
    ) -> Self {
        let (state, _) = BattleState::new(stage.setup(mode).clone());
        let waiting = state.reinforcements().iter().map(|r| &r.unit);
        let cast = state.units().iter().chain(waiting);
        BattleEntrant {
            name: name.into(),
            mode,
            player,
            ai,
            cast: cast.map(|u| (u.id, (u.name.clone(), u.is_lord))).collect(),
            start: BattleMeasures::new(&state),
        }
    }

    /// A persona, playing in its own mode.
    pub fn persona(
        name: impl Into<String>,
        persona: Persona,
        ai: AiWeights,
        stage: &BattleStage,
    ) -> Self {
        Self::new(name, persona.mode, Player::Persona(persona), ai, stage)
    }

    /// The mode it plays in.
    pub fn mode(&self) -> GameMode {
        self.mode
    }
}

impl Entrant<BattleStage, BattleState> for BattleEntrant {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self, stage: &BattleStage, seed: u64) -> BattleState {
        let mut setup = stage.setup(self.mode).clone();
        setup.seed = seed;
        BattleState::new(setup).0
    }

    fn bots(&self, seed: u64) -> Vec<(Phase, Box<dyn Bot<BattleState>>)> {
        let player: Box<dyn Bot<BattleState>> = match &self.player {
            Player::Baseline => Box::new(BaselineBot::new(self.ai)),
            Player::Persona(persona) => Box::new(PersonaBot::new(persona, self.ai, seed)),
        };
        vec![
            (Phase::Player, player),
            (Phase::Enemy, Box::new(BaselineBot::new(self.ai))),
            (Phase::Other, Box::new(BaselineBot::new(self.ai))),
        ]
    }

    fn measure(&self) -> Box<dyn Measure<BattleState>> {
        Box::new(BattleMeasure {
            measures: self.start.clone(),
            cast: self.cast.clone(),
            mode: self.mode,
        })
    }
}

/// A try's [`BattleMeasures`] as the engine's named measures:
///
/// - `won`: 1 if the battle was won, else 0;
/// - `lord fell`: 1 if a lord fell, else 0;
/// - `deaths` and `retreats`: the player units that fell, as deaths in
///   Classic and retreats in Casual (the other is 0);
/// - `items`: battle-pack items the player's units used;
/// - `turns`: the last turn played (the turn cap if that stopped it);
/// - for each player unit that fell, `fell <name>` (1) and `fell <name>
///   turn` (the turn it fell on).
pub struct BattleMeasure {
    measures: BattleMeasures,
    cast: BTreeMap<UnitId, (String, bool)>,
    mode: GameMode,
}

impl Measure<BattleState> for BattleMeasure {
    fn observe(&mut self, state: &BattleState, _: &Command, events: &[Event]) {
        self.measures.observe(state, events);
    }

    fn finish(self: Box<Self>, report: &Report<BattleState>, _: &BattleState) -> Measures {
        let m = &self.measures;
        let fallen = i64::try_from(m.player_fallen.len()).unwrap_or(i64::MAX);
        // The cap stops a try as the turn after it starts: the last played
        // is the one before.
        let stopped = u32::from(report.outcome.is_none());
        let mut out = Measures::from([
            (
                "won".to_owned(),
                i64::from(m.outcome == Some(Outcome::Victory)),
            ),
            ("items".to_owned(), i64::from(m.items_total())),
            (
                "turns".to_owned(),
                i64::from(m.turns.saturating_sub(stopped)),
            ),
            (
                "deaths".to_owned(),
                if self.mode == GameMode::Classic {
                    fallen
                } else {
                    0
                },
            ),
            (
                "retreats".to_owned(),
                if self.mode == GameMode::Casual {
                    fallen
                } else {
                    0
                },
            ),
        ]);
        let mut lord_fell = false;
        for &(id, turn) in &m.player_fallen {
            let (name, lord) = self
                .cast
                .get(&id)
                .cloned()
                .unwrap_or_else(|| (format!("unit {}", id.0), false));
            lord_fell |= lord;
            out.insert(format!("fell {name}"), 1);
            out.insert(format!("fell {name} turn"), i64::from(turn));
        }
        out.insert("lord fell".to_owned(), i64::from(lord_fell));
        out
    }
}

/// What the personas should achieve (`docs/design/playtest-bots.md`), as
/// `targets.ron` holds it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Targets {
    /// What makes a try a success, per persona.
    pub success: BTreeMap<String, Vec<Condition>>,
    /// The band of successes wanted, per tier and persona.
    pub bands: HashMap<Difficulty, BTreeMap<String, Band>>,
    /// The share of successes under which the change is blocked, per
    /// persona that has one.
    pub floors: BTreeMap<String, u32>,
    /// A persona whose median of a measure must be lower than another's.
    pub faster: Faster,
}

/// [`Targets::faster`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Faster {
    /// The persona that should be faster.
    pub entrant: String,
    /// The persona it is compared with.
    pub than: String,
    /// The measure.
    pub measure: String,
    /// The tries both medians are taken over.
    pub among: Vec<Condition>,
}

impl Targets {
    /// Reads `targets.ron`.
    pub fn from_ron(text: &str) -> Result<Targets, String> {
        ron::from_str(text).map_err(|e| format!("targets: {e}"))
    }

    /// The checks of a battle of `tier` played by `entrants`: each one's
    /// success band, with its floor, then the faster check when both of its
    /// personas play. An entrant with no target (the baseline) gets none.
    pub fn checks(&self, tier: Difficulty, entrants: &[&str]) -> Vec<Check> {
        let bands = self.bands.get(&tier);
        let mut checks: Vec<Check> = entrants
            .iter()
            .filter_map(|&name| {
                Some(Check::Rate {
                    entrant: name.to_owned(),
                    success: self.success.get(name)?.clone(),
                    band: *bands?.get(name)?,
                    floor: self.floors.get(name).copied(),
                })
            })
            .collect();
        let faster = &self.faster;
        if entrants.contains(&faster.entrant.as_str()) && entrants.contains(&faster.than.as_str()) {
            checks.push(Check::Median {
                entrant: faster.entrant.clone(),
                measure: faster.measure.clone(),
                among: faster.among.clone(),
                want: Want::Lower,
                than: faster.than.clone(),
            });
        }
        checks
    }
}

#[cfg(test)]
mod tests;
