//! What the autobalance may change in a battle (ticket 0509,
//! `docs/design/playtest-bots.md`, *Autobalancing*): the engine's
//! [`Knob`]s over a battle file ([`TunedBattle`]). The search, the
//! distance to the bands and the pruning are the engine's
//! (`shuyi_core::autobalance_by`); the bots are never touched.
//!
//! Every battle kind has the "enemy numbers" set for now: the level, count,
//! class and weapon of each generic enemy the story doesn't name. Named
//! units, the map and the objective stay as authored. A random skirmish
//! gains terrain and layout knobs with its generator (0510).

use std::sync::Arc;

use shuyi_core::Knob;
use trpg_content::Content;
use trpg_content::battle::tune::{BattleText, Foe};
use trpg_core::{BattleDef, ItemDef, ItemId, Level, LoadoutDef, WeaponDef};

/// A battle file as the autobalance changes it, with the content it is
/// loaded against.
#[derive(Debug, Clone)]
pub struct TunedBattle {
    /// The file, its enemy list changed.
    pub text: BattleText,
    content: Arc<Content>,
}

impl TunedBattle {
    /// Battle file `text`, loaded against `content`.
    pub fn new(text: BattleText, content: Arc<Content>) -> Self {
        Self { text, content }
    }

    /// The battle as the content check loads it, or its errors.
    pub fn load(&self) -> Result<BattleDef, String> {
        self.text
            .load(&self.content.battle_refs())
            .map_err(|errors| {
                let lines: Vec<String> = errors.iter().map(ToString::to_string).collect();
                lines.join("\n")
            })
    }

    /// `self` with `change` made to the foes of `entry`: `None` if it has
    /// none left, or the battle no longer loads.
    fn with(&self, entry: usize, change: impl Fn(&Self, &mut Foe) -> Option<()>) -> Option<Self> {
        let mut out = self.clone();
        let mut any = false;
        for foe in out.text.foes.iter_mut().filter(|f| f.entry == entry) {
            change(self, foe)?;
            any = true;
        }
        (any && out.load().is_ok()).then_some(out)
    }

    /// `foe`'s level: its own, or its template's.
    fn level(&self, foe: &Foe) -> Level {
        let template = foe.template.as_ref();
        let generic = template.and_then(|t| self.content.characters.generics.get(t));
        foe.level.or(generic.map(|g| g.level)).unwrap_or(1)
    }

    /// `foe`'s level moved `step`, kept within the level cap.
    fn moved(&self, foe: &Foe, step: i32) -> Level {
        let cap = i64::from(self.content.classes.level_cap);
        let level = (i64::from(self.level(foe)) + i64::from(step)).clamp(1, cap);
        Level::try_from(level).unwrap_or(1)
    }

    /// `foe`'s loadout: its own, or its template's.
    fn loadout(&self, foe: &Foe) -> Option<LoadoutDef> {
        let template = foe.template.as_ref()?;
        let generic = self.content.characters.generics.get(template);
        foe.loadout.clone().or(generic.map(|g| g.loadout.clone()))
    }

    /// The weapon `step` tiers from `id` in its kind (by rank, then might),
    /// if there is one.
    fn tier(&self, id: &ItemId, step: i32) -> Option<ItemId> {
        let items = &self.content.items;
        let kind = items.weapon(id)?.kind;
        let mut tiers: Vec<(&ItemId, &WeaponDef)> = items
            .items
            .iter()
            .filter_map(|(id, item)| match item {
                ItemDef::Weapon(w) if w.kind == kind => Some((id, w)),
                _ => None,
            })
            .collect();
        tiers.sort_by_key(|(id, w)| (w.rank, w.might, w.price, *id));
        let at = tiers.iter().position(|(t, _)| *t == id)?;
        let to = at.checked_add_signed(isize::try_from(step).ok()?)?;
        tiers.get(to).map(|(id, _)| (*id).clone())
    }
}

/// The knobs of `battle`'s kind, in the order the search tries them.
pub fn knobs(battle: &TunedBattle) -> Result<Vec<Box<dyn Knob<TunedBattle>>>, String> {
    let def = battle.load()?;
    // Every kind has the enemy numbers; 0510 adds the random skirmish's
    // terrain and layout.
    Ok(enemy_numbers(battle, &def))
}

/// The "enemy numbers" knobs: every unnamed enemy's level, then each one's
/// level, count, class and weapon.
fn enemy_numbers(battle: &TunedBattle, def: &BattleDef) -> Vec<Box<dyn Knob<TunedBattle>>> {
    let entries = battle.text.entries();
    let unnamed: Vec<(usize, String)> = entries
        .iter()
        .zip(&def.enemies)
        .enumerate()
        .filter(|(_, (e, _))| e.unnamed)
        .map(|(i, (e, unit))| (i, format!("{} at {},{}", unit.name, e.pos.0, e.pos.1)))
        .collect();
    let mut templates: Vec<String> = Vec::new();
    for t in entries.iter().filter_map(|e| e.template.clone()) {
        if !templates.contains(&t) {
            templates.push(t);
        }
    }
    let class_name = |t: &String| {
        let generic = battle.content.characters.generics.get(t);
        let class = generic.and_then(|g| battle.content.classes.get(&g.class));
        class.map_or_else(|| t.clone(), |c| c.name.clone())
    };
    let classes: Arc<[(String, String)]> = templates
        .iter()
        .map(|t| (t.clone(), class_name(t)))
        .collect();
    let mut knobs: Vec<Box<dyn Knob<TunedBattle>>> = vec![Box::new(AllLevels)];
    for (entry, label) in &unnamed {
        knobs.push(Box::new(Each::new(Change::Level, *entry, label)));
    }
    for (entry, label) in &unnamed {
        knobs.push(Box::new(Each::new(Change::Count, *entry, label)));
    }
    for (entry, label) in &unnamed {
        let own = entries[*entry].template.as_ref();
        let at = templates.iter().position(|t| Some(t) == own).unwrap_or(0);
        if templates.len() > 1 {
            let class = Change::Class(at, Arc::clone(&classes));
            knobs.push(Box::new(Each::new(class, *entry, label)));
        }
    }
    for (entry, label) in &unnamed {
        knobs.push(Box::new(Each::new(Change::Weapon, *entry, label)));
    }
    knobs
}

/// Every unnamed enemy's level, moved together: coarse, tried first.
struct AllLevels;

impl Knob<TunedBattle> for AllLevels {
    fn name(&self) -> &'static str {
        "unnamed enemies' level"
    }

    fn range(&self) -> std::ops::RangeInclusive<i32> {
        -10..=10
    }

    fn stride(&self) -> u32 {
        4
    }

    fn apply(&self, battle: &TunedBattle, step: i32) -> Option<TunedBattle> {
        let mut out = battle.clone();
        let entries = battle.text.entries();
        for foe in &mut out.text.foes {
            if entries[foe.entry].unnamed {
                foe.level = Some(battle.moved(foe, step));
            }
        }
        Some(out)
    }

    fn describe(&self, step: i32) -> String {
        format!("every unnamed enemy {step:+} levels")
    }
}

/// What an [`Each`] knob changes.
enum Change {
    /// Its level, -5 to +5.
    Level,
    /// -1 removes it, +1 adds a copy on a free tile within 2 of it.
    Count,
    /// Its class: another template of the battle's (index, then every
    /// template with its class's name, in the battle's order).
    Class(usize, Arc<[(String, String)]>),
    /// Its first weapon, a tier down or up in its kind.
    Weapon,
}

/// One change to one unnamed enemy (its entry in the file, and its copies).
struct Each {
    change: Change,
    entry: usize,
    name: String,
    /// "Brigand at 16,11".
    label: String,
}

impl Each {
    fn new(change: Change, entry: usize, label: &str) -> Self {
        let what = match change {
            Change::Level => "level",
            Change::Count => "count",
            Change::Class(..) => "class",
            Change::Weapon => "weapon",
        };
        Self {
            change,
            entry,
            name: format!("{label}: {what}"),
            label: label.to_owned(),
        }
    }

    /// A copy of the entry's last foe on the first free tile within 2 of
    /// the entry's own on which the battle still loads, after it in the list.
    fn add(&self, battle: &TunedBattle) -> Option<TunedBattle> {
        let def = battle.load().ok()?;
        let foes = &battle.text.foes;
        let last = foes.iter().rposition(|f| f.entry == self.entry)?;
        let (x, y) = battle.text.entries()[self.entry].pos;
        let starts = def
            .enemies
            .iter()
            .chain(def.reinforcements.iter().map(|r| &r.unit));
        let mut taken: Vec<(i32, i32)> = starts.map(|u| (u.pos.x, u.pos.y)).collect();
        taken.extend(def.player_slots.iter().map(|s| (s.pos.x, s.pos.y)));
        let near = (1..=2).flat_map(|d: i32| {
            (-d..=d).flat_map(move |dy| {
                let dx = d - dy.abs();
                [(x - dx, y + dy), (x + dx, y + dy)]
            })
        });
        near.filter(|at| !taken.contains(at)).find_map(|pos| {
            let mut out = battle.clone();
            let copy = Foe {
                copy: true,
                pos,
                ..foes[last].clone()
            };
            out.text.foes.insert(last + 1, copy);
            out.load().is_ok().then_some(out)
        })
    }
}

impl Knob<TunedBattle> for Each {
    fn name(&self) -> &str {
        &self.name
    }

    fn range(&self) -> std::ops::RangeInclusive<i32> {
        match &self.change {
            Change::Level => -5..=5,
            Change::Count | Change::Weapon => -1..=1,
            Change::Class(at, all) => {
                let at = i32::try_from(*at).unwrap_or(0);
                let last = i32::try_from(all.len()).unwrap_or(1) - 1;
                -at..=last - at
            }
        }
    }

    fn apply(&self, battle: &TunedBattle, step: i32) -> Option<TunedBattle> {
        match &self.change {
            Change::Level => battle.with(self.entry, |b, foe| {
                foe.level = Some(b.moved(foe, step));
                Some(())
            }),
            Change::Count if step > 0 => self.add(battle),
            Change::Count => {
                let mut out = battle.clone();
                out.text.foes.retain(|f| f.entry != self.entry);
                let gone = out.text.foes.len() < battle.text.foes.len();
                (gone && !out.text.foes.is_empty() && out.load().is_ok()).then_some(out)
            }
            Change::Class(at, all) => {
                let to = at.checked_add_signed(isize::try_from(step).ok()?)?;
                let (template, _) = all.get(to)?;
                battle.with(self.entry, |b, foe| {
                    foe.level = Some(b.level(foe));
                    foe.template = Some(template.clone());
                    foe.loadout = None;
                    Some(())
                })
            }
            Change::Weapon => battle.with(self.entry, |b, foe| {
                let mut loadout = b.loadout(foe)?;
                let first = loadout.weapons.first_mut()?;
                *first = b.tier(first, step)?;
                foe.loadout = Some(loadout);
                Some(())
            }),
        }
    }

    fn describe(&self, step: i32) -> String {
        let label = &self.label;
        match &self.change {
            Change::Level => format!("{label} {step:+} levels"),
            Change::Count if step > 0 => format!("one more beside {label}"),
            Change::Count => format!("removed {label}"),
            Change::Class(at, all) => {
                let to = at.checked_add_signed(isize::try_from(step).unwrap_or(0));
                let class = to.and_then(|to| all.get(to)).map_or("?", |(_, name)| name);
                format!("{label} → {class}")
            }
            Change::Weapon if step > 0 => format!("{label}: weapon a tier up"),
            Change::Weapon => format!("{label}: weapon a tier down"),
        }
    }
}

#[cfg(test)]
mod tests;
