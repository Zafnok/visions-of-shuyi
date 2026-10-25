//! `cargo shuyi autobalance --stage <battle-id>` (ticket 0509): the plan
//! the engine's command searches. The battle file, its kind's knobs
//! (`trpg_bots::knobs`), 0506's three personas judged by `targets.ron`'s
//! bands for the battle's tier, one try of a persona on a changed battle,
//! and `--apply` writing the battle file back. The options, the threads,
//! the search and the report are the engine's (engine#2127, engine#2541).

use std::path::Path;
use std::sync::Arc;

use shuyi_cli::contract::autobalance::AutobalanceInput;
use shuyi_cli::{AutobalancePlan, Tunable};
use shuyi_core::bots::playtest::play_tries;
use trpg_bots::{BattleEntrant, Persona, Targets, TunedBattle, knobs, personas_from_ron};
use trpg_content::battle::BATTLES_DIR;
use trpg_content::battle::tune::BattleText;
use trpg_content::bundle;

use crate::playtest::{PERSONAS_FILE, TARGETS_FILE, stage};

/// Who the search lands in their bands: the personas, never the baseline.
const ENTRANTS: [&str; 3] = ["casual", "normal", "hardcore"];

/// The battles `--stage` takes, by id.
pub fn stages() -> Vec<String> {
    let content = trpg_content::load_embedded();
    content.map_or_else(|_| Vec::new(), |c| c.battles.into_keys().collect())
}

/// The plan of battle `stage`, the personas' settings and targets read
/// from under `root`, where `--apply` writes the battle file too.
pub fn plan(
    root: &Path,
    stage_id: Option<&str>,
    input: &AutobalanceInput,
) -> Result<AutobalancePlan, String> {
    if let Some(option) = input.start.first() {
        return Err(format!(
            "autobalance takes no game option, not {}",
            option.flag
        ));
    }
    let id = stage_id.ok_or("name the battle to balance with --stage")?;
    let content = Arc::new(trpg_content::load_embedded().map_err(|e| format!("content: {e}"))?);
    let path = format!("{BATTLES_DIR}/{id}.ron");
    let source = bundle::file(&path).ok_or_else(|| format!("no battle \"{id}\""))?;
    let shown = bundle::display_path(&path);
    let text = BattleText::parse(&shown, id, source).map_err(|e| e.to_string())?;
    let battle = TunedBattle::new(text, Arc::clone(&content));
    let def = battle.load()?;

    let read =
        |file: &str| std::fs::read_to_string(root.join(file)).map_err(|e| format!("{file}: {e}"));
    let all = personas_from_ron(&read(PERSONAS_FILE)?)?;
    let personas = ENTRANTS
        .iter()
        .map(|&name| all.get(name).copied().ok_or(name))
        .collect::<Result<Vec<Persona>, _>>()
        .map_err(|name| format!("no persona \"{name}\" in {PERSONAS_FILE}"))?;
    let targets = Targets::from_ron(&read(TARGETS_FILE)?)?;
    let tier = def.difficulty;

    let play = Box::new(move |battle: &TunedBattle, entrant: usize, seed, turns| {
        let stage = stage(&content, &battle.load()?);
        let name = ENTRANTS[entrant];
        let persona = BattleEntrant::persona(name, personas[entrant], content.ai, &stage);
        let mut tries =
            play_tries(&stage, &persona, seed..seed + 1, turns).map_err(|e| e.to_string())?;
        tries.pop().ok_or_else(|| "no try".to_owned())
    });
    let file = root.join(&shown);
    let write = Box::new(move |battle: &TunedBattle| {
        std::fs::write(&file, battle.text.text()).map_err(|e| format!("{shown}: {e}"))?;
        Ok(vec![shown.clone()])
    });
    let knobs = knobs(&battle)?;
    Ok(AutobalancePlan::new(
        format!("{id} · {tier:?} tier · {:?}", def.kind),
        ENTRANTS.iter().map(|&n| (n.to_owned(), None)).collect(),
        Box::new(move |names| targets.checks(tier, names)),
        Box::new(Tunable::new(battle, knobs, play, write)),
    ))
}

#[cfg(test)]
mod tests;
