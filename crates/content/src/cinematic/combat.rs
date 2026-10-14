//! The combat shot (ticket 0832, ADR-0062): one exchange between two units
//! of a battle, **worked out by the rules**. Unlike a map shot's action
//! (ADR-0061), nothing about the fight is authored: the file names who
//! attacks whom, from where and with which luck, and [`fight`] runs the
//! attack through `core` from the battle's start (a [`Command`] →
//! [`Event`]s), so the numbers are the game's own. The loader checks the
//! names ([`check`]); [`check_fights`] then runs every fight once the whole
//! content is loaded, so a rebalance that makes one impossible fails the
//! content check, not the title. The format is described in
//! `assets/cinematics/README.md`.

use std::collections::BTreeMap;

use trpg_core::{
    BattleState, CastTarget, Command, Equipped, Event, GameMode, LeadGender, LeadProfile, Phase,
    Pos, Unit, UnitAction, UnitId, WEAPON_SLOTS, lead::DEFAULT_NAME,
};

use super::{CINEMATICS_DIR, EXTENSION, Refs, Shot, action};
use crate::Content;
use crate::bundle;
use crate::chapter::battle_campaign;
use crate::error::{ContentError, ContentErrors};

/// What a [`Shot::Combat`] shows: one unit attacking another at the start
/// of a battle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CombatShot {
    /// The battle, by id (`assets/battles/`).
    pub battle: String,
    /// Who attacks: its unit id in the battle.
    pub attacker: UnitId,
    /// Who is attacked.
    pub target: UnitId,
    /// The tile the attacker strikes from; `None`: where it starts.
    pub from: Option<Pos>,
    /// The seed of the combat's luck; `None`: the battle's own.
    pub seed: Option<u64>,
}

/// A combat as the rules worked it out.
#[derive(Debug, Clone)]
pub struct Fight {
    /// Every unit as the attack starts.
    pub before: Vec<Unit>,
    /// What the attack did: at least one [`Event::CombatResolved`] with a
    /// strike in it.
    pub events: Vec<Event>,
    /// The battle after the attack.
    pub after: BattleState,
}

/// What `unit` can attack `target` with, in the order they are tried:
/// what it has equipped, then each other weapon it carries, first slot
/// first.
pub(super) fn attacks(unit: &Unit, target: UnitId) -> Vec<UnitAction> {
    let weapon = |slot| UnitAction::Attack {
        target,
        slot,
        active: None,
        art: None,
    };
    let loadout = &unit.loadout;
    let equipped = match &loadout.equipped {
        Some(Equipped::Spell(spell)) => Some(UnitAction::Cast {
            spell: spell.clone(),
            target: CastTarget::Unit(target),
            active: None,
        }),
        Some(Equipped::Weapon(slot)) => Some(weapon(*slot)),
        None => None,
    };
    let others = (0..WEAPON_SLOTS)
        .filter(|&slot| loadout.weapon(slot).is_some() && Some(slot) != loadout.equipped_slot())
        .map(weapon);
    equipped.into_iter().chain(others).collect()
}

/// The combat of `shot` as the rules work it out, with `lead` as the
/// lead: the battle as it starts (the player's side as a battle played on
/// its own has it), with the shot's seed for its luck and the attacker
/// moved to the shot's `from`; the phases ended until it is the
/// attacker's; then the attacker attacks the target from where it stands
/// with the first of what it carries that the rules allow (what it has
/// equipped first). An error, in plain words, if the rules allow none, or
/// if the attack fights no combat.
pub fn fight(content: &Content, shot: &CombatShot, lead: LeadProfile) -> Result<Fight, String> {
    let def =
        (content.battles.get(&shot.battle)).ok_or(format!("no battle \"{}\"", shot.battle))?;
    let mut setup =
        battle_campaign(content, def, GameMode::Classic, lead).battle_setup(def, &content.tables());
    setup.seed = shot.seed.unwrap_or(def.seed);
    let missing = |id: UnitId| format!("unit {} is not in the battle", id.0);
    let attacker = setup.units.iter_mut().find(|u| u.id == shot.attacker);
    let attacker = attacker.ok_or_else(|| missing(shot.attacker))?;
    if let Some(from) = shot.from {
        attacker.pos = from;
    }
    let (mut state, _) = BattleState::new(setup);
    let refused = |e| format!("the rules refuse the attack: {e}");
    let unit = state
        .unit(shot.attacker)
        .ok_or_else(|| missing(shot.attacker))?;
    // At most one round of phases until the attacker's.
    let phase = Phase::of(unit.faction);
    for _ in Phase::ALL {
        if state.phase() == phase {
            break;
        }
        state.apply(&Command::EndPhase).map_err(refused)?;
    }
    let unit = state
        .unit(shot.attacker)
        .ok_or_else(|| missing(shot.attacker))?;
    let act = |action| Command::Act {
        unit: shot.attacker,
        dest: unit.pos,
        action,
    };
    let tried: Vec<Command> = attacks(unit, shot.target).into_iter().map(act).collect();
    let first_refusal = tried.first().map(|cmd| state.check(cmd));
    let allowed = tried.iter().find(|cmd| state.check(cmd).is_ok()).cloned();
    let Some(cmd) = allowed else {
        return Err(match first_refusal {
            Some(Err(e)) => refused(e),
            _ => format!("unit {} has nothing to attack with", shot.attacker.0),
        });
    };
    let before = state.units().to_vec();
    let events = state.apply(&cmd).map_err(refused)?;
    let struck = events
        .iter()
        .any(|e| matches!(e, Event::CombatResolved { outcome, .. } if !outcome.strikes.is_empty()));
    if !struck {
        return Err("the attack fights no combat".to_owned());
    }
    Ok(Fight {
        before,
        events,
        after: state,
    })
}

/// Every problem with a combat shot on battle `battle` of `attacker`
/// against `target` from `from`, as the loader sees it, and the shot (its
/// units' ids 0 where a name is wrong): the battle exists; both units are
/// on its map at its start, named as a map shot's action names them; they
/// are two; `from` is on the map, a tile the attacker's movement type can
/// enter, and not where another unit starts (the rules never move a unit
/// there, so they don't check it). Whether the rules allow the attack is
/// [`check_fights`]'s to say.
pub(super) fn check(
    refs: Refs<'_>,
    (battle, attacker, target): (&str, &str, &str),
    from: Option<(i32, i32)>,
    seed: Option<u64>,
) -> (CombatShot, Vec<String>) {
    let mut shot = CombatShot {
        battle: battle.to_owned(),
        attacker: UnitId(0),
        target: UnitId(0),
        from: from.map(|(x, y)| Pos::new(x, y)),
        seed,
    };
    let Some(def) = refs.battles.get(battle) else {
        return (shot, vec![format!("no battle \"{battle}\"")]);
    };
    let mut problems = Vec::new();
    let cast = action::cast(def, refs);
    let mut id = |name: &str| {
        let found = cast.get(name).map(|actor| actor.id);
        if found.is_none() {
            problems.push(format!(
                "no unit \"{name}\" on the map of battle \"{battle}\" at its start"
            ));
        }
        found.unwrap_or(UnitId(0))
    };
    shot.attacker = id(attacker);
    shot.target = id(target);
    let attacker_id = cast.get(attacker).map(|actor| actor.id);
    if attacker == target {
        problems.push(format!("\"{attacker}\" can't attack itself"));
    }
    if let Some(pos) = shot.from {
        let tiles = &def.map.tiles;
        let (x, y) = (pos.x, pos.y);
        let class = cast.get(attacker).and_then(|actor| actor.class);
        match tiles.get(pos) {
            None => problems.push(format!(
                "from ({x}, {y}) is not on the map of battle \"{battle}\" ({}×{} tiles)",
                tiles.width(),
                tiles.height()
            )),
            Some(&terrain) => {
                let move_cost = class.map(|c| refs.terrain.move_cost(terrain, c.movement_type));
                if let (Some(None), Some(class)) = (move_cost, class) {
                    problems.push(format!(
                        "from ({x}, {y}): \"{attacker}\" ({}) can't stand there",
                        class.name
                    ));
                }
            }
        }
        // The first name of each other unit there (an enemy character
        // has two).
        let mut there: BTreeMap<UnitId, &str> = BTreeMap::new();
        for (name, actor) in &cast {
            if actor.pos == pos && Some(actor.id) != attacker_id {
                there.entry(actor.id).or_insert(name);
            }
        }
        for name in there.values() {
            problems.push(format!("from ({x}, {y}) is where \"{name}\" starts"));
        }
    }
    (shot, problems)
}

/// `content` if the rules allow the fight of every combat shot of its
/// cinematics ([`fight`], with the lead a new game has by default), else
/// one error per shot they refuse, against its cinematic file.
pub fn check_fights(content: Content) -> Result<Content, ContentErrors> {
    let lead = LeadProfile::new(DEFAULT_NAME, LeadGender::Male);
    let mut errors = Vec::new();
    for (id, def) in &content.cinematics {
        let file = bundle::display_path(&format!("{CINEMATICS_DIR}/{id}{EXTENSION}"));
        for (i, shot) in def.shots.iter().enumerate() {
            let Shot::Combat(combat) = &shot.shot else {
                continue;
            };
            if let Err(why) = fight(&content, combat, lead.clone()) {
                errors.push(ContentError::new(&file, format!("shot {}: {why}", i + 1)));
            }
        }
    }
    if errors.is_empty() {
        Ok(content)
    } else {
        Err(ContentErrors(errors))
    }
}
