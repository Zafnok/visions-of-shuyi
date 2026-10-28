//! The `Magic` menu (ticket 0410, `docs/design/magic.md`): a unit's spell
//! list with what each spell can be cast on from where the unit stands, and
//! the cast target mode. One cursor steps through everything the chosen
//! spell can target (Nick): an enemy shows the attack forecast (0404's
//! [`Targeting`]), a hurt ally the heal's `HP 10 → 25`, an empty tile the
//! terrain change (`Forest → Burning (1 round)`). A spell active is a line
//! of the list after its spell (`Fire + Overcast`, Nick, 0442), cast at
//! enemies only. Every target is one the core accepts
//! ([`BattleState::check`]), so the UI never decides what is legal
//! (ADR-0004).

use crate::prelude::*;
use trpg_core::{
    BattleState, CastTarget, Command, CostError, EffectDuration, Equipped, Pos, SkillId, SpellDef,
    SpellId, SpellKind, TerrainId, UnitAction, UnitId,
};

use super::art_list::{Fit, Technique, cost_label, fit, reason_text};
use super::attack::{Targeting, aim_tile, reach};
use super::info::range_text;
use super::items::heal_preview;
use super::mode::{Line, Selection};
use super::skills::skill_name;
use crate::widgets::menu::{Menu, MenuItem, SuffixKind};
use crate::widgets::wrap_step;
use crate::words::Words;
use core::fmt::Display;

/// One line of the spell list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellChoice {
    /// The spell.
    pub spell: SpellId,
    /// The spell active it is cast with (`Fire + Overcast`), if any.
    pub active: Option<SkillId>,
    /// What it can be cast on from the destination, in `(y, x)` order of
    /// their tiles: enemies (an attack spell), hurt allies (a heal), empty
    /// tiles (a terrain effect); with an active, enemies only. Empty: the
    /// line is dimmed.
    pub targets: Vec<CastTarget>,
    /// Why the active's cost can't be paid (`1 uses left`), if it can't.
    pub blocked: Option<CostError>,
}

impl SpellChoice {
    /// What the line casts with: plainly, or its spell active.
    pub fn technique(&self) -> Technique {
        self.active
            .clone()
            .map_or(Technique::Attack, Technique::Active)
    }
}

/// Casting `spell` on `target`, with no active.
pub fn cast(spell: &SpellId, target: CastTarget) -> UnitAction {
    UnitAction::Cast {
        spell: spell.clone(),
        target,
        active: None,
    }
}

/// The tile `target` is on: a unit's, or the tile itself.
pub fn target_pos(state: &BattleState, target: CastTarget) -> Option<Pos> {
    match target {
        CastTarget::Unit(id) => state.unit(id).map(|u| u.pos),
        CastTarget::Tile(pos) => Some(pos),
    }
}

/// What `unit` could cast `spell` on from `dest`: the units and, for a
/// spell with a terrain effect, the tiles in its range that the core
/// accepts, in `(y, x)` order of their tiles.
pub fn cast_targets(
    state: &BattleState,
    unit: UnitId,
    dest: Pos,
    spell: &SpellDef,
) -> Vec<CastTarget> {
    let legal = |target: CastTarget| {
        let action = cast(&spell.id, target);
        state.check(&Command::Act { unit, dest, action }).is_ok()
    };
    let near = |pos: Pos| spell.in_range(Pos::manhattan(dest, pos));
    let units = state
        .units()
        .iter()
        .map(|u| (u.pos, CastTarget::Unit(u.id)));
    let tiles = state
        .map()
        .tiles
        .positions()
        .filter(|_| spell.terrain_effect.is_some())
        .map(|pos| (pos, CastTarget::Tile(pos)));
    let mut found: Vec<(Pos, CastTarget)> = units
        .chain(tiles)
        .filter(|&(pos, target)| near(pos) && legal(target))
        .collect();
    found.sort_by_key(|(p, _)| (p.y, p.x));
    found.into_iter().map(|(_, target)| target).collect()
}

/// The spell list's lines for `unit` at `dest`: the spells it has learned,
/// in id order, with what each could be cast on from there; after an attack
/// spell, a line for each of the unit's spell actives (`Fire + Overcast`).
pub fn spell_choices(state: &BattleState, unit: UnitId, dest: Pos) -> Vec<SpellChoice> {
    let Some(u) = state.unit(unit) else {
        return vec![];
    };
    let actives = u.usable_skills(state.classes(), state.skills());
    let mut lines = vec![];
    for def in u.learned.iter().filter_map(|id| state.spells().get(id)) {
        lines.push(SpellChoice {
            spell: def.id.clone(),
            active: None,
            targets: cast_targets(state, unit, dest, def),
            blocked: None,
        });
        if !def.is_attack() {
            continue;
        }
        let with = Equipped::Spell(def.id.clone());
        for skill in &actives {
            let technique = Technique::Active(skill.id.clone());
            let blocked = match fit(state, unit, dest, &with, &technique) {
                // Not a spell active: no line.
                Fit::No => continue,
                Fit::Cost(error) => Some(error),
                Fit::Hits | Fit::Far => None,
            };
            let targets = reach(state, unit, dest, &with, &technique);
            lines.push(SpellChoice {
                spell: def.id.clone(),
                active: Some(skill.id.clone()),
                targets: targets.into_iter().map(CastTarget::Unit).collect(),
                blocked,
            });
        }
    }
    lines
}

/// The lines of the heal list that pointing at the hurt ally `target`
/// opens (Nick, 0451): the heal spells `unit` has learned, in id order.
/// A heal that can be cast on `target` from `dest` has the hurt allies it
/// reaches from there, starting at `target`, then on in reading order; one
/// that can't is dimmed (no targets).
pub fn heal_choices(
    state: &BattleState,
    unit: UnitId,
    dest: Pos,
    target: UnitId,
) -> Vec<SpellChoice> {
    heal_spells(state, unit)
        .map(|def| {
            let mut targets = cast_targets(state, unit, dest, def);
            let at = targets.iter().position(|&t| t == CastTarget::Unit(target));
            match at {
                Some(at) => targets.rotate_left(at),
                None => targets.clear(),
            }
            SpellChoice {
                spell: def.id.clone(),
                active: None,
                targets,
                blocked: None,
            }
        })
        .collect()
}

/// The heal spells `unit` has learned, in id order.
fn heal_spells(state: &BattleState, unit: UnitId) -> impl Iterator<Item = &SpellDef> {
    let learned = state.unit(unit).into_iter().flat_map(|u| &u.learned);
    learned
        .filter_map(|id| state.spells().get(id))
        .filter(|def| matches!(def.kind, SpellKind::Heal { .. }))
}

/// Whether some heal spell of `unit` can be cast on `target` from `from`
/// (0451): the core accepts it (an ally, hurt, in range, a use left).
pub fn can_heal(state: &BattleState, unit: UnitId, from: Pos, target: UnitId) -> bool {
    heal_spells(state, unit).any(|def| {
        let action = cast(&def.id, CastTarget::Unit(target));
        let command = Command::Act {
            unit,
            dest: from,
            action,
        };
        state.check(&command).is_ok()
    })
}

/// The tile `sel`'s unit heals `target` from when the player points at it
/// (Nick, 0451): [`aim_tile`] with the tiles some heal reaches it from, the
/// same rule as an enemy's.
pub fn heal_tile(state: &BattleState, sel: &Selection, target: UnitId) -> Option<Pos> {
    aim_tile(sel, |from| can_heal(state, sel.unit, from, target))
}

/// Whether the action menu lists `Magic` for `unit`: it knows a spell.
pub fn knows_spells(state: &BattleState, unit: UnitId) -> bool {
    state
        .unit(unit)
        .is_some_and(|u| u.learned.iter().any(|id| state.spells().get(id).is_some()))
}

/// Whether `Magic` is enabled: some spell has something to be cast on.
pub fn can_cast(choices: &[SpellChoice]) -> bool {
    choices.iter().any(|c| !c.targets.is_empty())
}

/// Whether some attack spell reaches an enemy (the action menu then opens
/// on `Magic`, as it opens on `Attack` when a weapon does).
pub fn reaches_an_enemy(state: &BattleState, choices: &[SpellChoice]) -> bool {
    choices.iter().any(|c| {
        let attack = state
            .spells()
            .get(&c.spell)
            .is_some_and(SpellDef::is_attack);
        attack && c.targets.iter().any(|t| matches!(t, CastTarget::Unit(_)))
    })
}

/// A spell's line: name, uses left of its uses per battle, and its numbers,
/// e.g. `Fire    6/10  Mt5 Hit90 Rng1-2` or `Heal    8/8   HP+10 Rng1`.
pub fn spell_label(
    words: Words<'_>,
    def: &SpellDef,
    name: &str,
    left: u8,
    name_w: usize,
) -> String {
    let numbers = match &def.kind {
        SpellKind::Attack { might, hit, .. } => {
            words.text_with("battle.spell.attack", &[("might", might), ("hit", hit)])
        }
        SpellKind::Heal { heal_power } => {
            words.text_with("battle.spell.heal", &[("power", heal_power)])
        }
    };
    words.text_with(
        "battle.spell.line",
        &[
            ("name", &format!("{name:<name_w$}")),
            ("left", &format!("{left:>2}")),
            ("uses", &format!("{:<2}", def.uses)),
            ("numbers", &numbers),
            ("range", &range_text(def.min_range, def.max_range)),
        ],
    )
}

/// The spell list: one line per choice, dimmed when it has nothing to be
/// cast on (always so at 0 uses), focused on the equipped spell if it can
/// be cast. A line with a spell active reads just `Fire + Overcast`, with
/// what the active costs after it (`+1 use`), or why that can't be paid:
/// the spell's uses and numbers are on its own line, above.
pub fn spell_menu(
    state: &BattleState,
    words: Words<'_>,
    unit: UnitId,
    choices: &[SpellChoice],
) -> Menu<Line> {
    let defs = || {
        choices
            .iter()
            .filter_map(|c| Some((c, state.spells().get(&c.spell)?)))
    };
    let name_w = defs().map(|(_, d)| words.spell(d).chars().count()).max();
    let caster = state.unit(unit);
    let items = defs()
        .map(|(c, def)| {
            let label = match &c.active {
                None => {
                    let left = caster.map_or(0, |u| u.spells.uses_left(&c.spell));
                    spell_label(words, def, words.spell(def), left, name_w.unwrap_or(0))
                }
                Some(active) => {
                    let active = skill_name(state, words, active);
                    let keys = [
                        ("spell", &words.spell(def) as &dyn Display),
                        ("active", &active),
                    ];
                    words.text_with("battle.spell.with_active", &keys)
                }
            };
            let item = MenuItem::new(label)
                .with_value(Line::Spell(c.clone()))
                .with_enabled(!c.targets.is_empty());
            match (&c.blocked, &c.active) {
                (Some(error), _) => item.with_suffix(
                    format!(" {}", reason_text(words, error)),
                    SuffixKind::Warning,
                ),
                (None, Some(_)) => {
                    let cost = cost_label(state, words, &c.technique());
                    item.with_suffix(format!(" {cost}"), SuffixKind::Aside)
                }
                (None, None) => item,
            }
        })
        .collect();
    let equipped = caster.and_then(|u| u.loadout.equipped_spell());
    let menu = Menu::new(items);
    let focus = menu.items().iter().position(|i| match &i.value {
        Line::Spell(c) => Some(&c.spell) == equipped && c.active.is_none(),
        _ => false,
    });
    match focus {
        Some(i) => menu.focused(i),
        None => menu,
    }
}

/// Picking what a spell is cast on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastTargeting {
    /// The caster and its path (it stands at the path's end).
    pub sel: Selection,
    /// The spell list it was opened from (Cancel goes back to it).
    pub menu: Menu<Line>,
    /// The line being cast.
    pub chosen: SpellChoice,
    /// The target under the cursor, an index of the line's targets.
    pub index: usize,
    /// The forecast against the units an attack spell reaches (with the
    /// line's spell active, if it has one); shown while the cursor is on
    /// one of them.
    attack: Option<Targeting>,
}

impl CastTargeting {
    /// Targeting for `chosen`, a line of `menu`, starting on a unit (0462):
    /// the one `sel`'s path was aimed at if the spell reaches it, else the
    /// first unit in reading order, else the first tile. `None` if it has
    /// no target.
    pub fn new(
        state: &BattleState,
        sel: Selection,
        menu: Menu<Line>,
        chosen: SpellChoice,
    ) -> Option<Self> {
        let choice = &chosen;
        let aimed = sel.target.map(CastTarget::Unit);
        let start = choice
            .targets
            .iter()
            .position(|t| Some(*t) == aimed)
            .or_else(|| {
                let unit = |t: &CastTarget| matches!(t, CastTarget::Unit(_));
                choice.targets.iter().position(unit)
            })
            .unwrap_or(0);
        let first = *choice.targets.get(start)?;
        let enemies: Vec<UnitId> = choice
            .targets
            .iter()
            .filter_map(|t| match t {
                CastTarget::Unit(id) => Some(*id),
                CastTarget::Tile(_) => None,
            })
            .collect();
        // No forecast can be made of a heal: its units get the HP line.
        let with = Equipped::Spell(choice.spell.clone());
        let mut attack = Targeting::with(state, sel.clone(), with, choice.technique(), enemies);
        if let (Some(attack), CastTarget::Unit(id)) = (&mut attack, first) {
            attack.aim(id, state);
        }
        Some(Self {
            sel,
            menu,
            chosen,
            index: start,
            attack,
        })
    }

    /// The line being cast.
    fn used(&self) -> &SpellChoice {
        // Its targets are never empty ([`Self::new`]).
        &self.chosen
    }

    /// The spell being cast.
    pub fn spell(&self) -> &SpellId {
        &self.used().spell
    }

    /// The target under the cursor.
    pub fn target(&self) -> CastTarget {
        self.used().targets[self.index]
    }

    /// Everything the spell can be cast on.
    pub fn targets(&self) -> &[CastTarget] {
        &self.used().targets
    }

    /// The attack forecast, while the cursor is on a unit an attack spell
    /// hits.
    pub fn forecast(&self) -> Option<&Targeting> {
        match self.target() {
            CastTarget::Unit(_) => self.attack.as_ref(),
            CastTarget::Tile(_) => None,
        }
    }

    /// Moves to the next target (or the previous one), wrapping; on an
    /// enemy, the forecast follows.
    pub fn cycle(&mut self, forward: bool, state: &BattleState) {
        self.index = wrap_step(self.index, self.used().targets.len(), forward);
        if let (CastTarget::Unit(id), Some(attack)) = (self.target(), &mut self.attack) {
            attack.aim(id, state);
        }
    }

    /// The command that casts the spell.
    pub fn command(&self) -> Command {
        match self.forecast() {
            Some(attack) => attack.command(),
            None => Command::Act {
                unit: self.sel.unit,
                dest: self.sel.dest(),
                action: cast(self.spell(), self.target()),
            },
        }
    }

    /// The tiles the spell can be cast on, each with the terrain it would
    /// become.
    pub fn tile_changes(&self, state: &BattleState) -> Vec<(Pos, TerrainId)> {
        let effect = state
            .spells()
            .get(self.spell())
            .and_then(|def| def.terrain_effect.as_ref());
        let Some(effect) = effect else {
            return vec![];
        };
        self.targets()
            .iter()
            .filter_map(|t| match t {
                CastTarget::Tile(pos) => Some((*pos, effect.to)),
                CastTarget::Unit(_) => None,
            })
            .collect()
    }

    /// The preview line of a heal (`Heal on Rex: HP 10 → 25`) or a tile
    /// cast (`Forest → Burning (1 round)`, `Sea → Ice`); none on an enemy,
    /// which has the forecast.
    pub fn preview(&self, state: &BattleState, words: Words<'_>) -> Option<String> {
        let def = state.spells().get(self.spell())?;
        match self.target() {
            CastTarget::Unit(id) => {
                let target = state.unit(id)?;
                let action = cast(self.spell(), self.target());
                let gain = state.preview_heal(self.sel.unit, self.sel.dest(), &action)?;
                Some(heal_preview(words, words.spell(def), target, gain))
            }
            CastTarget::Tile(pos) => {
                let effect = def.terrain_effect.as_ref()?;
                let name = |id| state.terrain().get(id).map(|t| words.terrain(t));
                let from = name(*state.map().tiles.get(pos)?)?;
                let to = name(effect.to)?;
                let change = format!("{from} → {to}");
                Some(match effect.lasts {
                    EffectDuration::Permanent => change,
                    EffectDuration::UntilCastersNextPhase { .. } => {
                        words.text_with("battle.spell.one_round", &[("change", &change)])
                    }
                })
            }
        }
    }
}
