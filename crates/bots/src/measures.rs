//! What happened in one try of a battle.

use std::collections::BTreeMap;

use trpg_core::{
    BattleState, CombatOutcome, Event, Faction, ItemId, Outcome, Phase, Side, StatValue, Turn,
    UnitId,
};

/// The measures of one try (`docs/design/playtest-bots.md`): built from the
/// battle's start with [`BattleMeasures::new`], then told every applied
/// command's events with [`BattleMeasures::observe`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleMeasures {
    /// How the battle ended; `None` while it is on (a try stopped by the
    /// turn cap keeps `None`).
    pub outcome: Option<Outcome>,
    /// The last turn started.
    pub turns: Turn,
    /// Each player unit that fell (died in Classic, retreated in Casual) and
    /// the turn it fell on, in the order they fell.
    pub player_fallen: Vec<(UnitId, Turn)>,
    /// Consumables used by player units, counted per item.
    pub items_used: BTreeMap<ItemId, u32>,
    /// Commands applied, every side's.
    pub commands: u32,
    /// Commands applied in the Player phase.
    pub player_commands: u32,
    /// What each player unit did (ticket 0527), for the units that did
    /// anything: a unit missing did nothing.
    pub units: BTreeMap<UnitId, UnitTally>,
    /// The phase the next command is issued in.
    phase: Phase,
    /// Every unit's HP after the last command, standing units only: what a
    /// strike's HP taken is measured from.
    hp: BTreeMap<UnitId, StatValue>,
}

/// What one player unit did in a try (ticket 0527).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UnitTally {
    /// HP it took off enemies (a strike's overkill not counted).
    pub dealt: u32,
    /// Enemies it felled (or downed): the last to hurt them.
    pub kills: u32,
    /// Unit EXP it gained.
    pub exp: u32,
    /// Character levels it gained.
    pub levels: u32,
}

impl BattleMeasures {
    /// Measures of a try that starts at `state`.
    pub fn new(state: &BattleState) -> Self {
        Self {
            outcome: state.outcome(),
            turns: state.turn(),
            player_fallen: Vec::new(),
            items_used: BTreeMap::new(),
            commands: 0,
            player_commands: 0,
            units: BTreeMap::new(),
            phase: state.phase(),
            hp: hp_of(state),
        }
    }

    /// Counts one applied command: `events` are what it gave and
    /// `state_after` is the battle after it.
    pub fn observe(&mut self, state_after: &BattleState, events: &[Event]) {
        self.commands += 1;
        if self.phase == Phase::Player {
            self.player_commands += 1;
        }
        // Who acts in this command, and who last hurt each unit in it.
        let mut actor: Option<UnitId> = None;
        let mut last_hurt_by: BTreeMap<UnitId, UnitId> = BTreeMap::new();
        for event in events {
            match event {
                Event::CombatResolved {
                    attacker,
                    defender,
                    outcome,
                    ..
                } => {
                    actor = Some(*attacker);
                    self.combat(
                        state_after,
                        (*attacker, *defender),
                        outcome,
                        &mut last_hurt_by,
                    );
                }
                Event::SpellCast { unit, .. }
                | Event::SkillUsed { unit, .. }
                | Event::ArtUsed { unit, .. } => actor = Some(*unit),
                Event::Pushed { unit, damage, .. }
                | Event::CollisionDamage { unit, damage, .. } => {
                    let before = self.hp.get(unit).copied().unwrap_or(*damage);
                    let after = (before - damage).max(0);
                    self.hp.insert(*unit, after);
                    if let Some(by) = actor {
                        self.hurt(state_after, by, *unit, before - after, &mut last_hurt_by);
                    }
                }
                Event::UnitDowned { unit } => self.felled(state_after, *unit, &last_hurt_by),
                Event::ExpGained { unit, amount } => {
                    self.units.entry(*unit).or_default().exp += amount;
                }
                Event::LeveledUp { unit, .. } => {
                    self.units.entry(*unit).or_default().levels += 1;
                }
                Event::PhaseStarted { turn, phase } => {
                    self.turns = self.turns.max(*turn);
                    self.phase = *phase;
                }
                Event::UnitFell { unit } => {
                    self.felled(state_after, *unit, &last_hurt_by);
                    let fallen = state_after.fallen().iter().find(|u| u.id == *unit);
                    if fallen.is_some_and(|u| u.faction == Faction::Player) {
                        self.player_fallen.push((*unit, self.turns));
                    }
                }
                Event::ItemUsed { unit, item, .. } => {
                    if is_player(state_after, *unit) {
                        *self.items_used.entry(item.clone()).or_default() += 1;
                    }
                }
                Event::BattleEnded { outcome } => self.outcome = Some(*outcome),
                _ => {}
            }
        }
        self.hp = hp_of(state_after);
    }

    /// Counts the HP each strike of a combat between `fighters` (attacker,
    /// defender) took off its target.
    fn combat(
        &mut self,
        state: &BattleState,
        (attacker, defender): (UnitId, UnitId),
        outcome: &CombatOutcome,
        last_hurt_by: &mut BTreeMap<UnitId, UnitId>,
    ) {
        for strike in &outcome.strikes {
            let (by, target) = match strike.by {
                Side::Attacker => (attacker, defender),
                Side::Defender => (defender, attacker),
            };
            let after = strike.target_hp_after;
            let before = self
                .hp
                .get(&target)
                .copied()
                .unwrap_or(after + strike.damage);
            self.hp.insert(target, after);
            if strike.hit && !strike.healed {
                self.hurt(state, by, target, before - after, last_hurt_by);
            }
        }
    }

    /// `by` took `lost` HP off `target`: a player unit's dealt, if `target`
    /// is an enemy.
    fn hurt(
        &mut self,
        state: &BattleState,
        by: UnitId,
        target: UnitId,
        lost: StatValue,
        last_hurt_by: &mut BTreeMap<UnitId, UnitId>,
    ) {
        if lost <= 0 {
            return;
        }
        last_hurt_by.insert(target, by);
        if is_player(state, by) && faction(state, target) == Some(Faction::Enemy) {
            let dealt = u32::try_from(lost).unwrap_or(0);
            self.units.entry(by).or_default().dealt += dealt;
        }
    }

    /// `unit` fell or went down: a kill for the player unit that last hurt
    /// it, if it is an enemy.
    fn felled(
        &mut self,
        state: &BattleState,
        unit: UnitId,
        last_hurt_by: &BTreeMap<UnitId, UnitId>,
    ) {
        let Some(&by) = last_hurt_by.get(&unit) else {
            return;
        };
        if is_player(state, by) && faction(state, unit) == Some(Faction::Enemy) {
            self.units.entry(by).or_default().kills += 1;
        }
    }

    /// Every consumable used by player units.
    pub fn items_total(&self) -> u32 {
        self.items_used.values().sum()
    }
}

/// Whether `unit`, standing, fallen or down, is a player unit.
fn is_player(state: &BattleState, unit: UnitId) -> bool {
    faction(state, unit) == Some(Faction::Player)
}

/// The faction of `unit`, standing, fallen or down.
fn faction(state: &BattleState, unit: UnitId) -> Option<Faction> {
    let gone = || {
        let mut gone = state.fallen().iter().chain(state.down());
        gone.find(|u| u.id == unit)
    };
    state.unit(unit).or_else(gone).map(|u| u.faction)
}

/// Every standing unit's HP in `state`.
fn hp_of(state: &BattleState) -> BTreeMap<UnitId, StatValue> {
    state.units().iter().map(|u| (u.id, u.hp)).collect()
}

#[cfg(test)]
mod tests {
    use trpg_core::{Command, UnitAction};

    use super::*;
    use crate::testkit::{Scripted, attack, enemy, player, rigged, start, weak};

    /// Two turns of a battle on the test map's plain row: on turn 2 player
    /// 2 fells enemy 4, then enemy 3 fells player 1.
    fn two_falls() -> (Scripted, BattleMeasures) {
        let mut battle = Scripted::new(start(vec![
            weak(player(1, 5)),
            rigged(player(2, 2)),
            rigged(enemy(3, 6)),
            weak(enemy(4, 1)),
        ]));
        let start = battle.measures.clone();
        battle.next_player_phase();
        battle.act(2, 2, attack(4));
        battle.end_phase();
        battle.act(3, 6, attack(1));
        (battle, start)
    }

    #[test]
    fn measures_start_from_the_state() {
        let (_, start) = two_falls();
        assert_eq!(start.outcome, None);
        assert_eq!(start.turns, 1);
        assert_eq!(start.player_fallen, vec![]);
        assert_eq!(start.items_total(), 0);
        assert_eq!((start.commands, start.player_commands), (0, 0));
    }

    #[test]
    fn measures_count_fallen_player_units_only() {
        let (battle, _) = two_falls();
        let fallen: Vec<UnitId> = battle.state.fallen().iter().map(|u| u.id).collect();
        assert_eq!(fallen, [UnitId(4), UnitId(1)]);
        assert_eq!(battle.measures.player_fallen, [(UnitId(1), 2)]);
        assert_eq!(battle.measures.outcome, None);
    }

    #[test]
    fn measures_ignore_a_fall_of_a_unit_the_state_doesnt_know() {
        let (battle, _) = two_falls();
        let mut measures = battle.measures.clone();
        measures.observe(&battle.state, &[Event::UnitFell { unit: UnitId(99) }]);
        assert_eq!(measures.player_fallen, battle.measures.player_fallen);
    }

    #[test]
    fn measures_count_items_used_by_player_units() {
        let mut battle = Scripted::new(start(vec![player(1, 5), player(2, 4), enemy(3, 0)]));
        let potion = ItemId::new("potion");
        let elixir = ItemId::new("elixir");
        let use_on = |target| UnitAction::UseItem {
            pack_index: 0,
            target: UnitId(target),
        };
        battle.act(1, 5, use_on(1));
        assert_eq!(battle.measures.items_used, [(potion.clone(), 1)].into());
        battle.act(2, 4, use_on(1));
        battle.next_player_phase();
        battle.act(1, 5, use_on(2));
        let used = [(elixir.clone(), 1), (potion.clone(), 2)];
        assert_eq!(battle.measures.items_used, used.into());
        assert_eq!(battle.measures.items_total(), 3);

        // Only player units' items count (the battle lets no one else use
        // one, so the events are made up).
        let by = |unit, target| Event::ItemUsed {
            unit: UnitId(unit),
            item: potion.clone(),
            target: UnitId(target),
        };
        let mut measures = battle.measures.clone();
        measures.observe(&battle.state, &[by(3, 1), by(99, 1)]);
        assert_eq!(measures.items_total(), 3);
        measures.observe(&battle.state, &[by(2, 3)]);
        assert_eq!(measures.items_used[&potion], 3);
    }

    #[test]
    fn measures_count_an_item_used_by_a_player_unit_that_then_fell() {
        let (battle, _) = two_falls();
        let mut measures = battle.measures.clone();
        let by = |unit| Event::ItemUsed {
            unit: UnitId(unit),
            item: ItemId::new("potion"),
            target: UnitId(unit),
        };
        // Player 1 and enemy 4 have both fallen.
        measures.observe(&battle.state, &[by(4)]);
        assert_eq!(measures.items_total(), 0);
        measures.observe(&battle.state, &[by(1)]);
        assert_eq!(measures.items_total(), 1);
    }

    #[test]
    fn measures_turns_is_the_last_turn_started() {
        let mut battle = Scripted::new(start(vec![player(1, 5), enemy(3, 0)]));
        assert_eq!(battle.measures.turns, 1);
        battle.act(1, 5, UnitAction::Wait);
        assert_eq!(battle.measures.turns, 1);
        battle.next_player_phase();
        battle.next_player_phase();
        assert_eq!(battle.state.turn(), 3);
        assert_eq!(battle.measures.turns, 3);
        // A turn never goes back.
        let old = Event::PhaseStarted {
            turn: 2,
            phase: Phase::Enemy,
        };
        battle.measures.observe(&battle.state, &[old]);
        assert_eq!(battle.measures.turns, 3);
    }

    #[test]
    fn measures_count_commands_and_the_player_phases_commands() {
        let mut battle = Scripted::new(start(vec![player(1, 5), enemy(3, 0)]));
        battle.act(1, 5, UnitAction::Wait);
        assert_eq!(
            (battle.measures.commands, battle.measures.player_commands),
            (1, 1)
        );
        // Ending the Player phase is the player's command; what follows is
        // not.
        battle.end_phase();
        assert_eq!(battle.state.phase(), Phase::Enemy);
        battle.act(3, 0, UnitAction::Wait);
        battle.next_player_phase();
        let enemy_side = battle.measures.commands - battle.measures.player_commands;
        assert_eq!(battle.measures.player_commands, 2);
        assert_eq!(enemy_side, 2);
        battle.step(&Command::EndPhase);
        assert_eq!(battle.measures.player_commands, 3);
    }

    /// Ticket 0527: what each player unit did.
    #[test]
    fn measures_count_each_player_units_dealt_kills_and_exp() {
        // Player 1 takes 4 HP off enemy 3 and fells it; player 2 waits.
        let mut first = weak(enemy(3, 6));
        first.hp = 4;
        let mut battle = Scripted::new(start(vec![
            rigged(player(1, 5)),
            player(2, 0),
            first,
            enemy(4, 8),
        ]));
        let events = battle
            .state
            .apply(&Command::Act {
                unit: UnitId(1),
                dest: crate::testkit::tile(5),
                action: attack(3),
            })
            .unwrap();
        battle.measures.observe(&battle.state, &events);
        let exp = events.iter().find_map(|e| match e {
            Event::ExpGained { unit, amount } if *unit == UnitId(1) => Some(*amount),
            _ => None,
        });
        let levels = events
            .iter()
            .filter(|e| matches!(e, Event::LeveledUp { unit, .. } if *unit == UnitId(1)))
            .count();
        let tally = battle.measures.units[&UnitId(1)];
        assert_eq!((tally.dealt, tally.kills), (4, 1));
        assert!(tally.exp > 0);
        assert_eq!(Some(tally.exp), exp);
        assert_eq!(tally.levels, u32::try_from(levels).unwrap());
        assert!(!battle.measures.units.contains_key(&UnitId(2)));
    }

    /// A combat of `attacker` on `defender` with `strikes` (by, damage, the
    /// target's HP after).
    fn combat(attacker: u32, defender: u32, strikes: &[(Side, StatValue, StatValue)]) -> Event {
        let strikes = strikes
            .iter()
            .map(|&(by, damage, target_hp_after)| trpg_core::Strike {
                by,
                hit: damage > 0,
                crit: false,
                damage,
                healed: false,
                reaction: None,
                target_hp_after,
            })
            .collect();
        let forecast = battle_forecast();
        Event::CombatResolved {
            attacker: UnitId(attacker),
            defender: UnitId(defender),
            forecast,
            outcome: CombatOutcome {
                strikes,
                attacker_hp: 0,
                defender_hp: 0,
            },
        }
    }

    /// Any forecast: the measures never read it.
    fn battle_forecast() -> trpg_core::Forecast {
        let state = start(vec![player(1, 5), enemy(3, 6)]);
        state.threat_forecast(UnitId(1), UnitId(3)).unwrap()
    }

    #[test]
    fn measures_count_the_hp_taken_not_the_overkill() {
        let battle = Scripted::new(start(vec![player(1, 5), enemy(3, 6), enemy(4, 7)]));
        let hp = battle.state.unit(UnitId(3)).unwrap().hp;
        let mut measures = battle.measures.clone();
        // Enemy 3 attacks player 1, which counters for 5, then for more than
        // the HP left: only what was left counts.
        let strikes = [
            (Side::Defender, 5, hp - 5),
            (Side::Attacker, 3, 10),
            (Side::Defender, 99, 0),
        ];
        measures.observe(&battle.state, &[combat(3, 1, &strikes)]);
        assert_eq!(measures.units[&UnitId(1)].dealt, u32::try_from(hp).unwrap());
        // The enemy's strike on player 1 counts for nobody.
        assert!(!measures.units.contains_key(&UnitId(3)));
        // The kill is the last to hurt it, in the same command.
        measures.observe(
            &battle.state,
            &[
                combat(3, 1, &[(Side::Defender, 2, 0)]),
                Event::UnitFell { unit: UnitId(3) },
            ],
        );
        assert_eq!(measures.units[&UnitId(1)].kills, 1);
        // A fall with nobody hurting it in that command is nobody's kill.
        measures.observe(&battle.state, &[Event::UnitFell { unit: UnitId(4) }]);
        assert_eq!(measures.units[&UnitId(1)].kills, 1);
    }

    #[test]
    fn measures_count_a_unit_downed_as_a_kill_and_a_spell_or_art_as_acting() {
        let battle = Scripted::new(start(vec![player(1, 5), enemy(3, 6), enemy(4, 7)]));
        let hp = battle.state.unit(UnitId(4)).unwrap().hp;
        let mut measures = battle.measures.clone();
        measures.observe(
            &battle.state,
            &[
                combat(1, 4, &[(Side::Attacker, hp, 0)]),
                Event::UnitDowned { unit: UnitId(4) },
            ],
        );
        assert_eq!(measures.units[&UnitId(1)].kills, 1);
        let pushed = |damage| Event::Pushed {
            unit: UnitId(3),
            from: crate::testkit::tile(6),
            to: crate::testkit::tile(6),
            collided: None,
            damage,
        };
        let cast = Event::SpellCast {
            unit: UnitId(1),
            spell: trpg_core::SpellId::new("fire"),
            target: trpg_core::CastTarget::Unit(UnitId(3)),
        };
        measures.observe(&battle.state, &[cast, pushed(1)]);
        let art = Event::ArtUsed {
            unit: UnitId(1),
            art: trpg_core::ArtId::new("smash"),
            weapon: ItemId::new("iron_sword"),
            durability_before: 10,
            durability_after: 8,
        };
        measures.observe(&battle.state, &[art, pushed(2)]);
        assert_eq!(
            measures.units[&UnitId(1)].dealt,
            u32::try_from(hp).unwrap() + 3
        );
    }

    #[test]
    fn measures_count_push_damage_for_the_unit_acting_and_levels() {
        let battle = Scripted::new(start(vec![player(1, 5), enemy(3, 6), enemy(4, 7)]));
        let mut measures = battle.measures.clone();
        let pushed = Event::Pushed {
            unit: UnitId(3),
            from: crate::testkit::tile(6),
            to: crate::testkit::tile(6),
            collided: Some(crate::testkit::tile(7)),
            damage: 2,
        };
        let crashed = Event::CollisionDamage {
            unit: UnitId(4),
            by: UnitId(3),
            damage: 3,
        };
        // Damage with nobody acting counts for nobody.
        measures.observe(&battle.state, std::slice::from_ref(&pushed));
        assert!(measures.units.is_empty());
        let skill = Event::SkillUsed {
            unit: UnitId(1),
            skill: trpg_core::SkillId::new("shove"),
        };
        measures.observe(&battle.state, &[skill, pushed, crashed]);
        assert_eq!(measures.units[&UnitId(1)].dealt, 5);
        let level = |unit| Event::LeveledUp {
            unit: UnitId(unit),
            level: 2,
            gains: trpg_core::StatGains::default(),
        };
        let exp = Event::ExpGained {
            unit: UnitId(1),
            amount: 30,
        };
        measures.observe(&battle.state, &[exp, level(1), level(1)]);
        assert_eq!(measures.units[&UnitId(1)].exp, 30);
        assert_eq!(measures.units[&UnitId(1)].levels, 2);
    }

    #[test]
    fn measures_record_the_outcome() {
        // The last enemy falls: victory.
        let mut battle = Scripted::new(start(vec![rigged(player(1, 5)), weak(enemy(2, 6))]));
        battle.act(1, 5, attack(2));
        assert_eq!(battle.state.outcome(), Some(Outcome::Victory));
        assert_eq!(battle.measures.outcome, Some(Outcome::Victory));
        assert_eq!(battle.measures.player_fallen, vec![]);

        // The last player unit falls: defeat, on turn 1.
        let mut battle = Scripted::new(start(vec![weak(player(1, 5)), rigged(enemy(2, 6))]));
        battle.end_phase();
        battle.act(2, 6, attack(1));
        assert_eq!(battle.measures.outcome, Some(Outcome::Defeat));
        assert_eq!(battle.measures.player_fallen, [(UnitId(1), 1)]);
    }
}
