//! What the player's units left to act could still do this turn, for the
//! lessons (ticket 0464, `docs/design/tutorial.md`, *The lessons*): a lesson
//! that asks for something none of them can do doesn't open, or is put
//! away. Every answer is the core's reach and the menus' own checks, never
//! the screen's cells.

use trpg_content::DoneWhen;
use trpg_core::{BattleState, CastTarget, Faction, Phase, Unit};

use super::art_list::{art_lines, can_use_art};
use super::magic::{knows_spells, spell_choices};
use super::mode::Selection;

/// What the units left to act could do, each worked out the first time a
/// lesson asks and kept until the battle changes (the screen starts a new
/// one after every command and rewind).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Doable {
    /// Some unit is left to act.
    unit: Option<bool>,
    /// Some unit left could reach an enemy.
    attack: Option<bool>,
    /// Some unit left has an art it could use from a tile it can reach.
    art: Option<bool>,
    /// Some unit left could aim a spell at a tile from a tile it can reach.
    tile_spell: Option<bool>,
}

impl Doable {
    /// Whether some unit left to act in `state` could do `done` this turn.
    /// What takes no unit ([`DoneWhen::needs_unit`]) can always be done.
    pub fn can(&mut self, state: &BattleState, done: DoneWhen) -> bool {
        match done {
            DoneWhen::ForecastOpened | DoneWhen::Attacked => *self
                .attack
                .get_or_insert_with(|| any_left(state, reaches_enemy)),
            DoneWhen::ArtsOpened => *self.art.get_or_insert_with(|| any_left(state, has_art)),
            DoneWhen::TileAimed => *self
                .tile_spell
                .get_or_insert_with(|| any_left(state, aims_tile)),
            d if d.needs_unit() => *self
                .unit
                .get_or_insert_with(|| any_left(state, |_, _| true)),
            _ => true,
        }
    }
}

/// Whether some unit of the player's left to act in the player's phase,
/// selected where it stands, passes `test`.
fn any_left(state: &BattleState, test: fn(&BattleState, &Selection) -> bool) -> bool {
    state.phase() == Phase::Player
        && state
            .units()
            .iter()
            .filter(|u| u.faction == Faction::Player && !u.acted)
            .filter_map(|u| Selection::new(state, u.id))
            .any(|sel| test(state, &sel))
}

/// Whether `sel` could reach a unit not the player's: on a tile it can move
/// through or attack after moving (what the danger zone and the threat
/// arrows use, and the Attacking lesson's trigger).
pub fn reaches_enemy(state: &BattleState, sel: &Selection) -> bool {
    let reached = |u: &Unit| {
        u.faction != Faction::Player && (sel.moves.contains(u.pos) || sel.attack.contains(u.pos))
    };
    state.units().iter().any(reached)
}

/// Whether `sel` has an art its menu would let it use from some tile it
/// can stop on.
fn has_art(state: &BattleState, sel: &Selection) -> bool {
    reaches_enemy(state, sel)
        && sel
            .reach
            .stoppable()
            .iter()
            .any(|dest| can_use_art(&art_lines(state, sel.unit, dest)))
}

/// Whether `sel` has a spell it could aim at a tile from some tile it can
/// stop on.
fn aims_tile(state: &BattleState, sel: &Selection) -> bool {
    let on_tile = |t: &CastTarget| matches!(t, CastTarget::Tile(_));
    knows_spells(state, sel.unit)
        && sel.reach.stoppable().iter().any(|dest| {
            spell_choices(state, sel.unit, dest)
                .iter()
                .any(|c| c.targets.iter().any(on_tile))
        })
}

#[cfg(test)]
mod tests {
    use trpg_content::Action;
    use trpg_core::{Command, Pos, UnitId};

    use super::*;

    /// Chapter 1 as a new game starts it (the shipped New Game).
    fn chapter_1() -> BattleState {
        let content = trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"));
        let lead = trpg_core::LeadProfile::new("Ellery", trpg_core::LeadGender::Male);
        let campaign = trpg_content::new_campaign(&content, trpg_core::GameMode::Classic, lead);
        let setup = campaign.battle_setup(&content.battles["ch01"], &content.tables());
        BattleState::new(setup).0
    }

    /// The id of the player's unit on `at`.
    fn on(state: &BattleState, at: Pos) -> UnitId {
        let u = state.units().iter().find(|u| u.pos == at);
        u.unwrap_or_else(|| panic!("nobody on {at:?}")).id
    }

    /// Makes the unit on `at` wait where it stands.
    fn wait(state: &mut BattleState, at: Pos) {
        let unit = on(state, at);
        let cmd = Command::Act {
            unit,
            dest: at,
            action: trpg_core::UnitAction::Wait,
        };
        state.apply(&cmd).unwrap_or_else(|e| panic!("{e:?}"));
    }

    const SERGEANT: Pos = Pos::new(6, 11);
    const HERETIC: Pos = Pos::new(4, 13);

    fn can(state: &BattleState, done: DoneWhen) -> bool {
        Doable::default().can(state, done)
    }

    /// Chapter 1, turn 1: only the Sergeant reaches an enemy (and has an
    /// art for it); the Heretic has a spell for the forest. Each goes when
    /// its unit has acted; with nobody left, nothing that takes a unit can
    /// be done, and what takes none always can.
    #[test]
    fn chapter_1_turn_1_can_do_what_its_units_left_can() {
        let mut s = chapter_1();
        for done in [
            DoneWhen::UnitSelected,
            DoneWhen::ForecastOpened,
            DoneWhen::Attacked,
            DoneWhen::ArtsOpened,
            DoneWhen::TileAimed,
        ] {
            assert!(can(&s, done), "{done:?}");
        }
        wait(&mut s, SERGEANT);
        assert!(!can(&s, DoneWhen::Attacked));
        assert!(!can(&s, DoneWhen::ForecastOpened));
        assert!(!can(&s, DoneWhen::ArtsOpened));
        assert!(can(&s, DoneWhen::TileAimed));
        assert!(can(&s, DoneWhen::ThreatPointed));
        wait(&mut s, HERETIC);
        assert!(!can(&s, DoneWhen::TileAimed));
        assert!(can(&s, DoneWhen::UnitMoved));
        let left: Vec<Pos> = s
            .units()
            .iter()
            .filter(|u| u.faction == Faction::Player && !u.acted)
            .map(|u| u.pos)
            .collect();
        for at in left {
            wait(&mut s, at);
        }
        assert_eq!(s.phase(), Phase::Player);
        for done in [
            DoneWhen::UnitSelected,
            DoneWhen::ActionMenuOpened,
            DoneWhen::SafePointed,
        ] {
            assert!(!can(&s, done), "{done:?}");
        }
        for done in [
            DoneWhen::Pressed(Action::CursorUp),
            DoneWhen::CursorMoved,
            DoneWhen::RewindOpened,
            DoneWhen::TurnEnded,
            DoneWhen::OptionsOpened,
        ] {
            assert!(can(&s, done), "{done:?}");
        }
    }

    /// An answer is kept: the screen starts a new `Doable` when the battle
    /// changes.
    #[test]
    fn an_answer_is_kept_until_a_new_one_is_started() {
        let mut s = chapter_1();
        let mut d = Doable::default();
        assert!(d.can(&s, DoneWhen::Attacked));
        wait(&mut s, SERGEANT);
        assert!(d.can(&s, DoneWhen::Attacked));
        assert!(!Doable::default().can(&s, DoneWhen::Attacked));
    }
}
