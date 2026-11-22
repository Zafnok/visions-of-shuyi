//! When the guide's tips and lessons fire (0406, 0457, 1291): the checks
//! of the battle's state, its mode and a command's events that fire a
//! [`TipTrigger`]. The [`Guide`](super::guide::Guide) queues what they
//! fire and decides when it shows.

use crate::prelude::*;

use trpg_content::TipTrigger;
use trpg_core::{CastTarget, Event, Faction, Phase, Pos, TerrainRules, Unit, UnitId};

use super::mode::{Line, MenuEntry, Mode};
use super::{BattleScreen, WATER, art_list, doable, forecast, map_menu};
use crate::screen::Ctx;

impl BattleScreen {
    /// A player unit's level-up in `events` fires [`TipTrigger::FirstLevelUp`].
    pub(super) fn note_level_ups(&mut self, events: &[Event]) {
        let player_level_up = events.iter().any(|e| {
            matches!(e, Event::LeveledUp { unit, .. }
                if self.state.unit(*unit).is_some_and(|u| u.faction == Faction::Player))
        });
        if player_level_up {
            self.guide.fire(TipTrigger::FirstLevelUp);
        }
    }

    /// At the frame's start (1292): the queued tips' moment passes if the
    /// kind of mode changed ([`Guide::take_in_mode`]), then the tips and
    /// lessons the battle's state fired are queued.
    ///
    /// [`Guide::take_in_mode`]: super::guide::Guide::take_in_mode
    pub(super) fn take_in_tips(&mut self, ctx: &mut Ctx) {
        self.guide.take_in_mode(&self.mode);
        self.detect_tips(ctx);
        let battle = self.battle_id.as_deref();
        self.guide
            .absorb_lessons(ctx, battle, &self.state, &self.mode);
        self.detect_zoom_tip(ctx);
        self.guide.absorb_tips(ctx);
    }

    /// Fires the triggers that show in the battle's current state (the
    /// ones that come from a command's events are fired as it applies).
    fn detect_tips(&mut self, ctx: &Ctx) {
        for trigger in self.triggers_now() {
            self.guide.fire(trigger);
        }
        self.detect_wade_tip(ctx);
    }

    /// Fires [`TipTrigger::ZoomAvailable`] while the map, under a skin that
    /// can zoom, is being browsed with no lesson in play but one that waits
    /// for the turn to end (0466, 1304). After the lessons have absorbed
    /// this frame's triggers, so the first frame of a battle sees the ones
    /// it starts; the zoom lesson it starts is queued with the next frame's.
    fn detect_zoom_tip(&mut self, ctx: &Ctx) {
        let browsing = matches!(self.mode, Mode::Idle { .. });
        if browsing && !self.guide.lessons_busy() && ctx.game.map_skin.zoom().is_some() {
            self.guide.fire(TipTrigger::ZoomAvailable);
        }
    }

    /// Fires [`TipTrigger::FirstWadeInReach`] while the selected unit could
    /// step into a river (0483): `water` is the content's terrain, so it is
    /// looked up here.
    fn detect_wade_tip(&mut self, ctx: &Ctx) {
        let Mode::Selected(sel) = &self.mode else {
            return;
        };
        let water = ctx.game.content.terrain.display.id_of(WATER);
        if water.is_some_and(|w| doable::wades_in_reach(&self.state, sel, w)) {
            self.guide.fire(TipTrigger::FirstWadeInReach);
        }
    }

    /// What the frame did may start a lesson auto-end must wait for
    /// (every unit has acted: ending the turn), or leave one that can't be
    /// finished this turn: the triggers the state shows now go to the
    /// lessons alone, which take them in.
    pub(super) fn take_in_lessons(&mut self, ctx: &Ctx) {
        for trigger in self.triggers_now() {
            self.guide.fire_lesson(trigger);
        }
        let battle = self.battle_id.as_deref();
        self.guide
            .absorb_lessons(ctx, battle, &self.state, &self.mode);
    }

    /// The tip and lesson triggers the battle's current state shows.
    pub(super) fn triggers_now(&self) -> Vec<TipTrigger> {
        let mut fired = Vec::new();
        let state = &self.state;
        if state.turn() == 1 && state.phase() == Phase::Player {
            fired.push(TipTrigger::FirstBattleStart);
        }
        if state.phase() == Phase::Enemy {
            fired.push(TipTrigger::FirstEnemyPhase);
        }
        let low = |u: &Unit| u.faction == Faction::Player && u.hp > 0 && u.hp * 2 <= u.stats.hp;
        if state.units().iter().any(low) {
            fired.push(TipTrigger::FirstLowHp);
        }
        match &self.mode {
            Mode::Selected(sel) => {
                fired.push(TipTrigger::FirstUnitSelected);
                if self.has_skill(sel.unit) {
                    fired.push(TipTrigger::FirstSkillUnit);
                }
                if doable::reaches_enemy(state, sel) {
                    fired.push(TipTrigger::FirstEnemyInRange);
                }
            }
            Mode::Targeting(t) => {
                fired.push(TipTrigger::FirstForecast);
                if !forecast::kind_effects(state, t).is_empty() {
                    fired.push(TipTrigger::FirstEffective);
                }
                if t.can_swap() {
                    fired.push(TipTrigger::FirstWeaponSwap);
                }
            }
            Mode::Idle { .. } => {
                let on_enemy = self.hovered().is_some_and(|u| u.faction == Faction::Enemy);
                if on_enemy && self.threat.danger().is_none() {
                    fired.push(TipTrigger::DangerZoneAvailable);
                }
                if self.special_terrain(self.cursor.pos) {
                    fired.push(TipTrigger::FirstTerrain);
                }
            }
            Mode::ActionMenu { menu, arts, .. } => {
                if menu.items().iter().any(|i| i.value == MenuEntry::Magic) {
                    fired.push(TipTrigger::FirstCaster);
                }
                if art_list::can_use_art(arts) {
                    fired.push(TipTrigger::ArtsAvailable);
                }
            }
            Mode::List { menu, .. } => {
                let on_tile = |t: &CastTarget| matches!(t, CastTarget::Tile(_));
                let tile_spell = |line: &Line| match line {
                    Line::Spell(c) => c.targets.iter().any(on_tile),
                    _ => false,
                };
                if menu.items().iter().any(|i| tile_spell(&i.value)) {
                    fired.push(TipTrigger::TileSpellAvailable);
                }
            }
            _ => {}
        }
        fired.extend(self.lesson_triggers_now());
        fired
    }

    /// The triggers only lessons use (0457) that the battle's current
    /// state shows, besides the menus': every unit has acted, a unit has
    /// acted on turn 1 (1304), turn 2 began, the threat arrows show.
    pub(super) fn lesson_triggers_now(&self) -> Vec<TipTrigger> {
        let mut fired = Vec::new();
        let state = &self.state;
        if state.phase() != Phase::Player || state.outcome().is_some() {
            return fired;
        }
        let browsing = matches!(self.mode, Mode::Idle { .. });
        if self.no_unit_left() && browsing {
            fired.push(TipTrigger::NoUnitLeft);
        }
        let acted = |u: &Unit| u.faction == Faction::Player && u.acted;
        if state.turn() == 1 && browsing && state.units().iter().any(acted) {
            fired.push(TipTrigger::FirstUnitActed);
        }
        if state.turn() == 2 {
            fired.push(TipTrigger::SecondTurn);
        }
        if self.threat_view().is_some() {
            fired.push(TipTrigger::ThreatArrowsShown);
        }
        fired
    }

    /// Whether, in the player's phase of a battle still on, every unit of
    /// the player's has acted.
    fn no_unit_left(&self) -> bool {
        self.state.phase() == Phase::Player
            && self.state.outcome().is_none()
            && map_menu::ready_players(&self.state) == 0
    }

    /// Whether `unit` has a skill it can use now (a passive or an active).
    pub(super) fn has_skill(&self, unit: UnitId) -> bool {
        let state = &self.state;
        state
            .unit(unit)
            .is_some_and(|u| !u.usable_skills(state.classes(), state.skills()).is_empty())
    }

    /// Whether the terrain at `pos` is not plain ground: it gives a Def,
    /// Avoid or healing bonus, or costs a movement type other than 1 to
    /// enter (more, or impassable).
    pub(super) fn special_terrain(&self, pos: Pos) -> bool {
        let state = &self.state;
        let rules = state
            .map()
            .tiles
            .get(pos)
            .and_then(|&id| state.terrain().get(id));
        rules.is_some_and(not_plain)
    }
}

/// Whether terrain `t` is not plain ground: it gives a Def, Avoid or
/// healing bonus, or costs a movement type other than 1 to enter (more, or
/// impassable).
pub(super) fn not_plain(t: &TerrainRules) -> bool {
    t.defense != 0
        || t.avoid != 0
        || t.heal_percent != 0
        || t.move_cost.iter().any(|&c| c != Some(1))
}
