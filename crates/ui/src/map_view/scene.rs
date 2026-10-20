//! What is on the visible battle map, as plain data (ADR-0038): the
//! tactics kit's scene (`shuyi_ui::grid::scene`, ticket 1115) of this
//! game's units ([`UnitView`]), with its ranges ([`range`]) and its
//! terrains' names ([`TerrainLooks`]). No colours, glyphs, cells or pixels:
//! a [`MapSkin`](super::MapSkin) turns a scene into those.

use crate::prelude::*;
use core::fmt::Write as _;

use shuyi_ui::grid::scene::MapUnit;
use shuyi_ui::grid::skin::{TerrainGlyphs, Terrains};
use trpg_content::terrain::TerrainDisplayTable;
use trpg_core::skill::TimedMods;
use trpg_core::{CharacterId, ClassId, Faction, Pos, StatValue, TerrainId, Unit, UnitId};

pub use shuyi_ui::grid::scene::{
    CursorStyle, CursorView, RangeLayer, RangeName, ThreatView, TileView,
};
pub use shuyi_ui::grid::walk::{Facing, STANDING_FRAME};

/// The visible part of a battle map: the kit's scene of this game's units.
pub type MapScene = shuyi_ui::grid::MapScene<UnitView>;

/// The ranges shown on the battle map: the kit's range names, each its
/// colour's, so a skin tints it with that colour, and the scene's text
/// marks it with its first letter (`d`, `m`, `a`, `h`).
pub mod range {
    use shuyi_ui::grid::scene::RangeName;

    use crate::color::UiColor;

    /// Tiles the enemy could attack this turn (the danger zone).
    pub const DANGER: RangeName = RangeName(UiColor::DangerZone.name());
    /// Tiles a unit can move to.
    pub const MOVE: RangeName = RangeName(UiColor::MoveRange.name());
    /// Tiles a unit can attack, or the targets of an attack or a skill.
    pub const ATTACK: RangeName = RangeName(UiColor::AttackRange.name());
    /// The targets of a heal.
    pub const HEAL: RangeName = RangeName(UiColor::HealRange.name());

    /// Every range.
    pub const ALL: [RangeName; 4] = [DANGER, MOVE, ATTACK, HEAL];
}

/// This game's terrains, as the kit's skins and the scene's text look them
/// up: their string ids, glyphs and colour names.
#[derive(Debug, Clone, Copy)]
pub struct TerrainLooks<'a>(pub &'a TerrainDisplayTable);

impl Terrains for TerrainLooks<'_> {
    fn name(&self, id: TerrainId) -> Option<&str> {
        self.0.get(id).map(|t| t.id.as_str())
    }

    fn glyphs(&self, id: TerrainId) -> Option<TerrainGlyphs<'_>> {
        self.0.get(id).map(|t| TerrainGlyphs {
            glyphs: t.glyphs,
            fg: &t.fg,
            bg: &t.bg,
        })
    }
}

/// The kinds of timed effect a unit is under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct UnitEffects {
    /// One that helps it: a stance, a buff.
    pub bonus: bool,
    /// One that hinders it: a debuff.
    pub penalty: bool,
}

/// A unit as shown on the map.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitView {
    /// Which unit.
    pub id: UnitId,
    /// The tile it is shown on (along its path while it walks).
    pub pos: Pos,
    /// Its side.
    pub faction: Faction,
    /// Its two-letter map label.
    pub label: String,
    /// Its class.
    pub class: ClassId,
    /// The named character, or `None` for a generic unit.
    pub character: Option<CharacterId>,
    /// Whether it has acted this phase.
    pub acted: bool,
    /// Current and max HP.
    pub hp: (StatValue, StatValue),
    /// The kinds of timed effect (0412) it is under.
    pub effects: UnitEffects,
    /// How far it has fallen (0404): `0` = standing … `1` = gone.
    pub fade: f32,
    /// Whether it is picked out right now: a battle note on screen is
    /// about it (0411). It blinks, so this goes on and off.
    pub highlight: bool,
    /// Which way it is turned: the way it walks, else towards the camera.
    pub facing: Facing,
    /// Its walking frame: `0` one foot forward, `1` standing
    /// ([`STANDING_FRAME`]), `2` the other foot forward. It changes as the
    /// unit steps on the spot (one that can still act) or walks.
    pub frame: u8,
    /// How far it is from [`pos`](Self::pos) towards the next tile of its
    /// walk, in tiles across and down, each in `-1.0..=1.0`; `(0, 0)` when
    /// it isn't walking.
    pub offset: (f32, f32),
    /// How strongly it flashes from a blow that just landed on it (a
    /// cinematic's strike, 0827): `1` as it lands, fading to `0`.
    pub flash: f32,
    /// Whether its HP bar and effect marks show: not for someone in a
    /// conversation's opening, who isn't fighting (0734).
    pub marks: bool,
}

impl UnitView {
    /// `unit` standing on its own tile.
    pub fn of(unit: &Unit) -> Self {
        Self {
            id: unit.id,
            pos: unit.pos,
            faction: unit.faction,
            label: unit.map_label.clone(),
            class: unit.class.clone(),
            character: unit.character.clone(),
            acted: unit.acted,
            hp: (unit.hp, unit.stats.hp),
            effects: UnitEffects {
                bonus: unit.effects.iter().any(|e| !hinders(&e.mods)),
                penalty: unit.effects.iter().any(|e| hinders(&e.mods)),
            },
            fade: 0.0,
            highlight: false,
            facing: Facing::Down,
            frame: STANDING_FRAME,
            offset: (0.0, 0.0),
            flash: 0.0,
            marks: true,
        }
    }

    /// Whether it is between two tiles of a walk.
    pub fn between_tiles(&self) -> bool {
        self.offset != (0.0, 0.0)
    }

    /// Whether it is under any timed effect.
    pub const fn has_effect(&self) -> bool {
        self.effects.bonus || self.effects.penalty
    }

    /// This unit shown on `pos` instead.
    #[must_use]
    pub fn at(self, pos: Pos) -> Self {
        Self { pos, ..self }
    }

    /// This unit `fade` of the way through falling.
    #[must_use]
    pub fn fading(self, fade: f32) -> Self {
        Self { fade, ..self }
    }

    /// This unit picked out, or not.
    #[must_use]
    pub fn highlighted(self, highlight: bool) -> Self {
        Self { highlight, ..self }
    }
}

impl MapUnit for UnitView {
    type Id = UnitId;

    fn id(&self) -> UnitId {
        self.id
    }

    fn pos(&self) -> Pos {
        self.pos
    }

    fn text(&self) -> String {
        unit_text(self)
    }
}

/// Whether a timed effect of `mods` hinders the unit it is on (a debuff:
/// Pinning Shot's Mov −3) rather than helps it (a stance: Sidestep's avoid
/// +20): whether it lowers any number. One that both raises and lowers
/// counts as hindering.
pub fn hinders(mods: &TimedMods) -> bool {
    let c = &mods.combat;
    let combat = [c.hit, c.crit, c.might, c.avoid, c.attack_speed, c.pierce];
    mods.stats.iter().any(|&(_, amount)| amount < 0)
        || combat.iter().any(|&n| n < 0)
        || c.single_strike
}

/// A unit's line in [`MapScene::to_text`](shuyi_ui::grid::MapScene::to_text):
/// `#4 Br enemy brigand (7,3) hp 20/30 acted bonus penalty fade=0.25`.
/// A unit of a named character has it in brackets after its class,
/// `bonus` or `penalty` (or both) if it is under a timed effect, and
/// `highlight` if it is picked out. A unit on its walk also has the way it
/// faces (`facing=right`; none when it faces the camera), how far it is
/// towards the next tile (`offset=(0.50,0.00)`) and its walking frame
/// (`frame=0`). The frame of a unit stepping on the spot isn't shown: it
/// goes by the clock alone.
// check-text: not player text (the scene as text, for tests)
fn unit_text(u: &UnitView) -> String {
    let faction = match u.faction {
        Faction::Player => "player",
        Faction::Enemy => "enemy",
        Faction::Ally => "ally",
        Faction::Neutral => "neutral",
    };
    let mut text = format!("#{} {} {faction} {}", u.id.0, u.label, u.class.0);
    if let Some(c) = &u.character {
        let _ = write!(text, " [{}]", c.0);
    }
    let _ = write!(text, " ({},{}) hp {}/{}", u.pos.x, u.pos.y, u.hp.0, u.hp.1);
    if u.acted {
        text.push_str(" acted");
    }
    if u.effects.bonus {
        text.push_str(" bonus");
    }
    if u.effects.penalty {
        text.push_str(" penalty");
    }
    if u.fade > 0.0 {
        let _ = write!(text, " fade={:.2}", u.fade);
    }
    if u.highlight {
        text.push_str(" highlight");
    }
    let walking = u.facing != Facing::Down || u.between_tiles();
    if u.facing != Facing::Down {
        let _ = write!(text, " facing={}", u.facing.name());
    }
    if u.between_tiles() {
        let _ = write!(text, " offset=({:.2},{:.2})", u.offset.0, u.offset.1);
    }
    if walking && u.frame != STANDING_FRAME {
        let _ = write!(text, " frame={}", u.frame);
    }
    if u.flash > 0.0 {
        let _ = write!(text, " flash={:.2}", u.flash);
    }
    if !u.marks {
        text.push_str(" no-marks");
    }
    text
}

#[cfg(test)]
mod tests;
