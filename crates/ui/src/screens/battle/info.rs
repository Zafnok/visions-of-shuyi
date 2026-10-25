//! The unit info screen (ticket 0405): everything about one unit, over the
//! map and side panel. A portrait (a placeholder until portraits exist,
//! 0703), then the unit's name, level, EXP, HP and stats as plain numbers
//! (`Str 7`; no class caps, ticket 0423), and its loadout, spells and
//! skills. Under the weapon ranks, the class's elemental affinities
//! (`Fire  Weak`, ticket 0410). [`view`] says what it shows; the battle
//! skin paints it (ADR-0054).

use crate::prelude::*;
use trpg_core::skill::effect_bonuses;
use trpg_core::{
    Affinity, BattleState, EffectSource, Element, StatKind, Stats, TimedEffect, Unit, UnitTags,
    WEAPON_SLOTS,
};

use super::art_list::art_name;
use super::art_list::durability_text;
use super::skills::{
    cost_text, effect_text, kind_name, skill_cost, skill_name, timed_text, until_text,
};
use super::view::{
    GearView, InfoEffectView, InfoSkillView, InfoSpellView, InfoStatView, InfoView, InfoWeaponView,
};
use crate::input::Action;
use crate::words::Words;

/// The stat's short name, as the design docs write it.
pub fn stat_name(words: Words<'_>, kind: StatKind) -> &str {
    words.text(match kind {
        StatKind::Hp => "stat.hp",
        StatKind::Str => "stat.str",
        StatKind::Mag => "stat.mag",
        StatKind::Dex => "stat.dex",
        StatKind::Spd => "stat.spd",
        StatKind::Def => "stat.def",
        StatKind::Res => "stat.res",
        StatKind::Mov => "stat.mov",
    })
}

/// The class tags that are on, by name: `Mounted Flying`.
pub fn tag_names(words: Words<'_>, tags: UnitTags) -> Vec<&str> {
    [
        (tags.mounted, "class.tag.mounted"),
        (tags.flying, "class.tag.flying"),
        (tags.armored, "class.tag.armored"),
    ]
    .iter()
    .filter(|(on, _)| *on)
    .map(|&(_, key)| words.text(key))
    .collect()
}

/// The name of an element (`Fire`).
fn element_name(words: Words<'_>, element: Element) -> &str {
    words.text(match element {
        Element::Fire => "battle.element.fire",
        Element::Ice => "battle.element.ice",
        Element::None => "battle.element.none",
    })
}

/// What a class makes of an element (`Weak`).
fn affinity_name(words: Words<'_>, affinity: Affinity) -> &str {
    words.text(match affinity {
        Affinity::Weak => "battle.affinity.weak",
        Affinity::Resist => "battle.affinity.resist",
        Affinity::Absorb => "battle.affinity.absorb",
    })
}

/// A gear bonus as text: its non-zero stats, `Def +2  Res +1`.
pub fn bonus_text(words: Words<'_>, bonus: &Stats) -> String {
    StatKind::ALL
        .iter()
        .filter(|&&k| bonus.get(k) != 0)
        .map(|&k| format!("{} +{}", stat_name(words, k), bonus.get(k)))
        .collect::<Vec<_>>()
        .join("  ")
}

/// A class and a level on one line: `Knight  Lv 3`.
pub fn class_level(words: Words<'_>, class: &str, level: impl core::fmt::Display) -> String {
    words.text_with(
        "battle.class_level",
        &[("class", &class), ("level", &level)],
    )
}

/// A range as text: `1` or `1-2`.
pub(super) fn range_text(min: u32, max: u32) -> String {
    if min == max {
        min.to_string()
    } else {
        format!("{min}-{max}")
    }
}

/// What the info screen of `unit` of `state` shows.
pub fn view((state, words): (&BattleState, Words<'_>), unit: &Unit) -> InfoView {
    let class = state.classes().get(&unit.class);
    let class_name = words.class_of(&unit.class, state.classes());
    let at_cap = unit.level >= state.classes().level_cap;
    let movement = class.and_then(|c| {
        state
            .terrain()
            .movement_types
            .get(usize::from(c.movement_type.0))
    });
    let tags = class.map(|c| c.tags).unwrap_or_default();
    let affinities = class.map(|c| c.affinities.as_slice()).unwrap_or_default();
    // Timed effects count: a raised or lowered stat is marked.
    let shown = effect_bonuses(&unit.effects).apply(unit.stats);
    let listed = StatKind::GROWABLE
        .into_iter()
        .filter(|&k| k != StatKind::Hp)
        .map(|kind| InfoStatView {
            name: stat_name(words, kind).to_owned(),
            value: shown.get(kind),
            changed: shown.get(kind) != unit.stats.get(kind),
        })
        .collect();
    let effects = unit
        .effects
        .iter()
        .map(|effect| InfoEffectView {
            name: effect_name(state, words, effect),
            mods: timed_text(words, &effect.mods),
            until: until_text(words, effect).to_owned(),
        })
        .collect();
    let text = |key| words.text(key).to_owned();
    InfoView {
        portrait: text("battle.portrait"),
        name: words.unit(unit).to_owned(),
        faction: unit.faction,
        class_level: class_level(words, class_name, unit.level),
        exp_name: text("battle.exp"),
        exp: (!at_cap).then_some(unit.exp),
        hp_name: stat_name(words, StatKind::Hp).to_owned(),
        hp: unit.hp,
        max: unit.stats.hp,
        stats_title: text("battle.info.stats"),
        stats: listed,
        effects_title: text("battle.info.effects"),
        effects,
        skills_title: text("battle.info.skills"),
        skills: skills(state, words, unit),
        mov_name: stat_name(words, StatKind::Mov).to_owned(),
        mov: unit.move_points(),
        movement: movement.cloned(),
        tags: tag_names(words, tags)
            .into_iter()
            .map(str::to_owned)
            .collect(),
        ranks_title: text("battle.info.weapon_ranks"),
        ranks: unit
            .weapon_ranks
            .iter()
            .map(|(kind, rank)| (kind_name(words, *kind).to_owned(), format!("{rank:?}")))
            .collect(),
        affinities_title: text("battle.info.affinities"),
        affinities: affinities
            .iter()
            .map(|&(element, affinity)| {
                let element = element_name(words, element).to_owned();
                (element, affinity_name(words, affinity).to_owned())
            })
            .collect(),
        weapons_title: text("battle.info.weapons"),
        equipped_mark: text("battle.info.mark.equipped"),
        weapons: weapons(state, words, unit),
        armour: GearView {
            title: text("battle.info.armour"),
            item: unit.loadout.armour.as_ref().and_then(|id| {
                let a = state.items().armour(id)?;
                let name = words.item(id, state.items()).to_owned();
                Some((name, bonus_text(words, &a.bonus)))
            }),
        },
        accessory: GearView {
            title: text("battle.info.accessory"),
            item: unit.loadout.accessory.as_ref().and_then(|id| {
                let a = state.items().accessory(id)?;
                let name = words.item(id, state.items()).to_owned();
                Some((name, bonus_text(words, &a.bonus)))
            }),
        },
        spells_title: text("battle.info.spells"),
        spells: spells(state, words, unit),
        page: 0,
    }
}

/// The info screen's page after `action` on page `page` of `pages`: Left
/// and Right turn the pages, wrapping, on a skin that shows the screen a
/// page at a time (the Playdate's, ticket 1039). `None` if `action`
/// turns no page or there is only the one.
pub fn turn_page(action: Action, page: u8, pages: u8) -> Option<u8> {
    let forward = match action {
        Action::CursorRight => true,
        Action::CursorLeft => false,
        _ => return None,
    };
    (pages > 1).then(|| {
        let page = page.min(pages - 1);
        if forward {
            (page + 1) % pages
        } else {
            page.checked_sub(1).unwrap_or(pages - 1)
        }
    })
}

/// What gave a timed effect: a skill's or an art's id, as a name.
fn effect_name(state: &BattleState, words: Words<'_>, effect: &TimedEffect) -> String {
    match &effect.source {
        EffectSource::Skill(id) => skill_name(state, words, id),
        EffectSource::Art(id) => art_name(state, words, id),
    }
}

/// The skills the unit can use: passive (always on) or active, the cost
/// of an active (for a non-attack active its uses left this battle,
/// `2/3`), and what each does.
fn skills(state: &BattleState, words: Words<'_>, unit: &Unit) -> Vec<InfoSkillView> {
    unit.usable_skills(state.classes(), state.skills())
        .into_iter()
        .map(|skill| {
            let cost = skill_cost(skill);
            let mark = words.text(if cost.is_some() {
                "battle.info.mark.active"
            } else {
                "battle.info.mark.passive"
            });
            let left = unit.skill_uses.uses_left(&skill.id);
            InfoSkillView {
                mark: mark.to_owned(),
                name: words.skill(skill).to_owned(),
                cost: cost.map(|cost| cost_text(words, cost, left)),
                effect: effect_text(words, skill),
            }
        })
        .collect()
}

/// The weapon slots: each weapon, whether it is equipped, its durability
/// and numbers.
fn weapons(state: &BattleState, words: Words<'_>, unit: &Unit) -> Vec<Option<InfoWeaponView>> {
    let items = state.items();
    let loadout = &unit.loadout;
    (0..WEAPON_SLOTS)
        .map(|slot| {
            let inst = loadout.weapon(slot)?;
            let def = items.weapon(&inst.def)?;
            let numbers = words.text_with(
                "battle.info.weapon_numbers",
                &[
                    ("might", &def.might),
                    ("hit", &def.hit),
                    ("crit", &def.crit),
                    ("range", &range_text(def.min_range, def.max_range)),
                    ("weight", &def.weight),
                ],
            );
            Some(InfoWeaponView {
                name: words.item(&inst.def, items).to_owned(),
                equipped: loadout.equipped_slot() == Some(slot),
                broken: inst
                    .is_broken()
                    .then(|| durability_text(words, 0, def.durability)),
                left: inst.durability_left,
                max: def.durability,
                numbers,
            })
        })
        .collect()
}

/// The spells the unit has learned, with their uses left.
fn spells(state: &BattleState, words: Words<'_>, unit: &Unit) -> Vec<InfoSpellView> {
    unit.learned
        .iter()
        .filter_map(|id| state.spells().get(id))
        .map(|spell| InfoSpellView {
            name: words.spell(spell).to_owned(),
            equipped: unit.loadout.equipped_spell() == Some(&spell.id),
            left: unit.spells.uses_left(&spell.id),
            uses: spell.uses,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_elements_and_affinities_are_named() {
        let w = Words::ENGLISH;
        let all = UnitTags {
            mounted: true,
            flying: true,
            armored: true,
        };
        assert_eq!(tag_names(w, all), ["Mounted", "Flying", "Armored"]);
        assert_eq!(tag_names(w, UnitTags::default()), Vec::<&str>::new());
        let flier = UnitTags {
            flying: true,
            ..UnitTags::default()
        };
        assert_eq!(tag_names(w, flier), ["Flying"]);
        assert_eq!(
            [Element::Fire, Element::Ice, Element::None].map(|e| element_name(w, e)),
            ["Fire", "Ice", "None"]
        );
        assert_eq!(
            [Affinity::Weak, Affinity::Resist, Affinity::Absorb].map(|a| affinity_name(w, a)),
            ["Weak", "Resist", "Absorb"]
        );
        assert_eq!(class_level(w, "Knight", 3), "Knight  Lv 3");
    }

    #[test]
    fn stat_names_bonuses_and_ranges() {
        assert_eq!(
            StatKind::ALL.map(|kind| stat_name(Words::ENGLISH, kind)),
            ["HP", "Str", "Mag", "Dex", "Spd", "Def", "Res", "Mov"]
        );
        let mut bonus = Stats::default();
        assert_eq!(bonus_text(Words::ENGLISH, &bonus), "");
        bonus.def = 2;
        bonus.res = 1;
        assert_eq!(bonus_text(Words::ENGLISH, &bonus), "Def +2  Res +1");
        assert_eq!(
            (range_text(1, 1), range_text(1, 2)),
            ("1".into(), "1-2".into())
        );
    }

    #[test]
    fn the_view_reads_the_unit_its_effects_and_its_loadout() {
        use trpg_core::{Phase, SkillId, TimedMods, UnitId};

        use crate::screen::tests::ctx;
        use crate::screens::battle::quick_battle;

        let c = ctx();
        let state = quick_battle(&c.game.content).unwrap();
        let mut lord = state.unit(UnitId(1)).unwrap().clone();
        let plain = view((&state, Words::ENGLISH), &lord);
        assert_eq!(
            (plain.name.as_str(), plain.hp, plain.max),
            ("Test Lord", lord.hp, lord.stats.hp)
        );
        assert_eq!(plain.exp, Some(lord.exp));
        assert_eq!(plain.mov, lord.move_points());
        assert_eq!(plain.stats.len(), 6);
        assert!(plain.stats.iter().all(|s| !s.changed), "{:?}", plain.stats);
        assert_eq!(
            plain.effects,
            [] as [crate::screens::battle::view::InfoEffectView; 0]
        );
        let equipped = lord.loadout.equipped_slot().unwrap();
        let weapons: Vec<bool> = plain.weapons.iter().flatten().map(|w| w.equipped).collect();
        assert_eq!(weapons.iter().filter(|&&e| e).count(), 1);
        assert!(plain.weapons[equipped].as_ref().is_some_and(|w| w.equipped));
        // A timed effect raises Def: shown raised and marked; the others
        // stay as they are.
        lord.effects.push(TimedEffect {
            source: EffectSource::Skill(SkillId::new("keen_edge")),
            mods: TimedMods {
                stats: vec![(StatKind::Def, 2)],
                ..TimedMods::default()
            },
            until: Phase::Player,
        });
        let raised = view((&state, Words::ENGLISH), &lord);
        let def = raised.stats.iter().find(|s| s.name == "Def").unwrap();
        assert_eq!((def.value, def.changed), (lord.stats.def + 2, true));
        let str_ = raised.stats.iter().find(|s| s.name == "Str").unwrap();
        assert_eq!((str_.value, str_.changed), (lord.stats.str, false));
        assert_eq!(raised.effects.len(), 1);
        assert_eq!(raised.effects[0].mods, "Def +2");
        // At the level cap: no EXP.
        lord.level = state.classes().level_cap;
        assert_eq!(view((&state, Words::ENGLISH), &lord).exp, None);
        // A broken weapon says so.
        if let Some(w) = lord.loadout.weapons[equipped].as_mut() {
            w.durability_left = 0;
        }
        let broken = view((&state, Words::ENGLISH), &lord);
        let shown = broken.weapons[equipped].as_ref().unwrap();
        assert_eq!((shown.broken.as_deref(), shown.left), (Some("broken"), 0));
    }
}
