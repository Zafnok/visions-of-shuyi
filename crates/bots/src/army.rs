//! The reference army (ticket 0510): what "a lv 35 battle" means for the
//! bots (Nick, `docs/design/playtest-bots.md`, *Generating a skirmish to
//! order*). Who the player has recruited by a chapter, at that level, with
//! the ordinary gear of that point. An approximation, documented in
//! `docs/playtesting.md`, *Generating skirmishes*.

use trpg_content::chapter::{ChapterDef, roster_units};
use trpg_content::{Content, joins};
use trpg_core::{
    BasicStock, BotsArmy, Campaign, ChangeTables, CharacterId, ClassDef, ClassId, ClassRecord,
    ItemDef, ItemId, Level, SealKind, StatKind, StatValue, Stats, Stock, Tier, Unit, growth,
    promote,
};

/// The character level a unit in a class of each tier promotes at, from the
/// first (*the agent's starting rule*): `progression.md`'s pace, about 2
/// levels a battle, and mastery in about 6 battles at tier 1 and 10 at
/// tier 2. Nothing in the rules forces a promotion; this is when a player
/// who keeps up would have made it.
const PROMOTES_AT: [Level; 2] = [13, 33];

/// The class tier a player's unit has reached by `level`, and so the tier of
/// the enemies of a mixed skirmish at that level.
pub fn tier_at(level: Level) -> Tier {
    let promotions = PROMOTES_AT.iter().filter(|&&at| level >= at).count();
    Tier::try_from(promotions + 1).unwrap_or(1)
}

/// The story's chapters in order, from the New Game file's first along each
/// chapter's `next`.
pub fn story_chapters(content: &Content) -> Vec<&ChapterDef> {
    let mut chapters: Vec<&ChapterDef> = Vec::new();
    let mut at = Some(&content.new_game.first_chapter);
    while let Some(chapter) = at.and_then(|id| content.chapters.get(id)) {
        if chapters.iter().any(|c| c.id == chapter.id) {
            break;
        }
        chapters.push(chapter);
        at = chapter.next.as_ref();
    }
    chapters
}

/// Who the player has recruited when `chapter` is being played: the New
/// Game roster and everyone a scene recruits before its battle is won. A
/// chapter's victory scenes come after that, so they count from the next
/// chapter on. Whole story when `chapter` isn't one of its chapters.
pub fn roster_by(content: &Content, chapter: &str) -> Vec<CharacterId> {
    let mut roster = content.new_game.roster.clone();
    let mut recruit = |scenes: &mut dyn Iterator<Item = &String>| {
        let scenes = scenes.filter_map(|id| content.dialogue.get(id));
        for who in scenes.flat_map(joins) {
            if !roster.contains(&who) {
                roster.push(who);
            }
        }
    };
    for def in story_chapters(content) {
        recruit(&mut def.intro_scenes.iter());
        if def.id == chapter {
            break;
        }
        let battle = content.battles.get(&def.battle);
        let triggers = battle.iter().flat_map(|b| b.triggers.iter());
        recruit(&mut triggers.map(|t| &t.scene));
        recruit(&mut def.victory_scenes.iter());
    }
    roster
}

/// The units of `army`, each raised to its level with its gear.
pub fn reference_army(content: &Content, army: &BotsArmy) -> Vec<Unit> {
    let mut units = roster_units(content, &roster_by(content, &army.chapter));
    for unit in &mut units {
        raise(unit, content, army);
    }
    units
}

/// `campaign`'s roster as `army` has it. The units stay the campaign's own,
/// so the lead keeps the name the player gave.
pub fn field_army(campaign: &mut Campaign, content: &Content, army: &BotsArmy) {
    for unit in &mut campaign.roster {
        raise(unit, content, army);
    }
}

/// The pack of `army`: as many of the best consumable the basic shop sells
/// by its chapter as `cap` holds.
pub fn reference_pack(content: &Content, army: &BotsArmy, cap: usize) -> Vec<ItemId> {
    let best = on_sale(&content.basic_shop, shop_chapter(content, army))
        .into_iter()
        .filter_map(|id| match content.items.get(&id) {
            Some(ItemDef::Consumable(c)) => Some((c.price, id)),
            _ => None,
        })
        .max();
    best.map_or_else(Vec::new, |(_, id)| vec![id; cap])
}

fn shop_chapter(content: &Content, army: &BotsArmy) -> u32 {
    let chapter = content.chapters.get(&army.chapter);
    chapter.and_then(|c| c.number).unwrap_or(1)
}

fn on_sale(shop: &BasicStock, chapter: u32) -> Vec<ItemId> {
    shop.shop(chapter).stock
}

/// `unit` at `army`'s level, promoted on the way, with its gear. Stats rise
/// by the average of each class's growths, so a level up's luck and its
/// minimum gains don't count.
fn raise(unit: &mut Unit, content: &Content, army: &BotsArmy) {
    let mut at = unit.level;
    while let Some(class) = content.classes.get(&unit.class) {
        let promotes = PROMOTES_AT.get(usize::from(class.tier).saturating_sub(1));
        let next = promotes
            .copied()
            .filter(|&p| p <= army.level)
            .and_then(|p| class.promotes_to.first().map(|to| (p, to.clone())));
        let until = next.as_ref().map_or(army.level, |(p, _)| *p).max(at);
        grow(unit, class, until.saturating_sub(at), content);
        at = until;
        let Some((_, to)) = next else {
            break;
        };
        if !advance(unit, class, &to, content) {
            break;
        }
    }
    unit.level = army.level;
    unit.exp = 0;
    let learned = unit
        .personal_spells
        .iter()
        .filter(|(l, _)| *l <= army.level);
    unit.learned
        .extend(learned.map(|(_, s)| s.clone()).collect::<Vec<_>>());
    gear(unit, content, shop_chapter(content, army));
    unit.hp = unit.stats.hp;
}

/// `levels` levels of `class`'s average growths for `unit`, within the hard
/// ceilings.
fn grow(unit: &mut Unit, class: &ClassDef, levels: Level, content: &Content) {
    let ceilings = &content.classes.hard_ceilings;
    let levels = StatValue::try_from(levels).unwrap_or(StatValue::MAX);
    let gain = |kind: StatKind, value: StatValue| {
        let points = StatValue::from(growth(unit, class, kind)).saturating_mul(levels) / 100;
        value.saturating_add(points).min(ceilings.get(kind))
    };
    let s = unit.stats;
    let stats = Stats {
        hp: gain(StatKind::Hp, s.hp),
        str: gain(StatKind::Str, s.str),
        mag: gain(StatKind::Mag, s.mag),
        dex: gain(StatKind::Dex, s.dex),
        spd: gain(StatKind::Spd, s.spd),
        def: gain(StatKind::Def, s.def),
        res: gain(StatKind::Res, s.res),
        ..s
    };
    unit.stats = stats;
}

/// `unit` promoted from `from` to `to` by the rules' own promotion, as a
/// player who mastered the class and bought the seal would: whether it
/// could.
fn advance(unit: &mut Unit, from: &ClassDef, to: &ClassId, content: &Content) -> bool {
    let Some(tier) = content.classes.get(to).map(|c| c.tier) else {
        return false;
    };
    let seal = content
        .items
        .items
        .iter()
        .find_map(|(id, item)| match item {
            ItemDef::Seal(s) if s.kind == SealKind::Tier(tier) => Some(id.clone()),
            _ => None,
        });
    let Some(seal) = seal else {
        return false;
    };
    let mut stock = Stock::default();
    stock.add(seal);
    unit.class_records.insert(
        from.id.clone(),
        ClassRecord {
            class_level: content.classes.class_level_cap,
            ..ClassRecord::UNLOCKED
        },
    );
    let tables = ChangeTables {
        classes: &content.classes,
        items: &content.items,
        spells: &content.spells,
    };
    promote(unit, to, &mut stock, &tables).is_ok()
}

/// `unit`'s weapons and armour: the best the basic shop sells by `chapter`
/// that it can use. A kind the class fights with gets its best weapon, the
/// class's first kind first, up to its slots. A unit with nothing on sale
/// for it keeps what it has.
fn gear(unit: &mut Unit, content: &Content, chapter: u32) {
    let Some(class) = content.classes.get(&unit.class) else {
        return;
    };
    let items = &content.items;
    let sold = on_sale(&content.basic_shop, chapter);
    let weapons: Vec<ItemId> = class
        .weapons
        .iter()
        .filter_map(|proficiency| {
            let usable = sold.iter().filter_map(|id| {
                let w = items.weapon(id)?;
                (w.kind == proficiency.kind && unit.can_wield(w, class))
                    .then_some(((w.rank, w.might, w.price), id))
            });
            usable.max().map(|(_, id)| id.clone())
        })
        .take(usize::from(class.weapon_slots))
        .collect();
    if !weapons.is_empty() {
        unit.loadout.weapons = Default::default();
        for (slot, id) in unit.loadout.weapons.iter_mut().zip(&weapons) {
            *slot = items.new_weapon(id);
        }
    }
    let armour = sold.iter().filter_map(|id| match items.get(id) {
        Some(ItemDef::Armour(a)) if class.can_wear(a.weight_class) => Some((a.price, id)),
        _ => None,
    });
    if let Some((_, id)) = armour.max() {
        unit.loadout.armour = Some(id.clone());
    }
    unit.loadout.equipped = unit.default_equip(class, items, &content.spells);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn content() -> Content {
        trpg_content::load_embedded().unwrap_or_else(|e| panic!("{e}"))
    }

    fn army(chapter: &str, level: Level) -> BotsArmy {
        BotsArmy {
            chapter: chapter.to_owned(),
            level,
        }
    }

    fn ids(units: &[Unit]) -> Vec<String> {
        let characters = units.iter().filter_map(|u| u.character.as_ref());
        characters.map(|c| c.0.clone()).collect()
    }

    #[test]
    fn reference_army_levels_and_roster() {
        let content = content();
        let six = [
            "lead", "retainer", "sergeant", "poacher", "keeper", "heretic",
        ];
        for (chapter, level) in [("ch01", 7), ("ch02", 35)] {
            let units = reference_army(&content, &army(chapter, level));
            assert_eq!(ids(&units), six, "{chapter}");
            assert!(units.iter().all(|u| u.level == level), "{chapter}");
        }
    }

    #[test]
    fn a_recruit_counts_from_the_chapter_after_the_scene_that_recruits() {
        let content = content();
        // Joss joins in Chapter 2's victory scene.
        assert!(
            !roster_by(&content, "ch02")
                .iter()
                .any(|c| c.0 == "defector")
        );
        let whole_story = roster_by(&content, "no such chapter");
        assert!(whole_story.iter().any(|c| c.0 == "defector"));
    }

    #[test]
    fn levels_raise_stats_by_the_classes_average_growths() {
        let content = content();
        let low = reference_army(&content, &army("ch01", 2));
        let high = reference_army(&content, &army("ch01", 12));
        for (low, high) in low.iter().zip(&high) {
            let class = content.classes.get(&low.class).expect("its class");
            let want = low.stats.hp + i32::from(growth(low, class, StatKind::Hp)) * 10 / 100;
            assert!(high.stats.hp.abs_diff(want) <= 1, "{}", low.name);
            assert_eq!(high.hp, high.stats.hp);
        }
    }

    #[test]
    fn every_stat_grows_by_the_average_within_the_ceiling() {
        let content = content();
        let start = reference_army(&content, &army("ch01", 1));
        let unit = start.first().expect("a unit");
        let class = content.classes.get(&unit.class).expect("its class");
        let mut grown = unit.clone();
        grow(&mut grown, class, 20, &content);
        for kind in [
            StatKind::Hp,
            StatKind::Str,
            StatKind::Mag,
            StatKind::Dex,
            StatKind::Spd,
            StatKind::Def,
            StatKind::Res,
        ] {
            let gain = StatValue::from(growth(unit, class, kind)) * 20 / 100;
            assert!(gain > 0, "{kind:?} shows nothing here");
            assert_eq!(
                grown.stats.get(kind),
                unit.stats.get(kind) + gain,
                "{kind:?}"
            );
        }
        let mut capped = unit.clone();
        grow(&mut capped, class, 100_000, &content);
        assert_eq!(capped.stats.hp, content.classes.hard_ceilings.hp);
    }

    #[test]
    fn a_personal_spell_is_learned_from_its_level() {
        let content = content();
        let spell = trpg_core::SpellId::new("personal");
        let learned = |level| {
            let mut unit = reference_army(&content, &army("ch01", 1)).remove(0);
            unit.personal_spells = vec![(20, spell.clone())];
            raise(&mut unit, &content, &army("ch01", level));
            unit.learned.contains(&spell)
        };
        assert!(!learned(19));
        assert!(learned(20));
        assert!(learned(21));
    }

    #[test]
    fn a_promotion_that_can_t_happen_says_so() {
        let content = content();
        let mut unit = reference_army(&content, &army("ch01", 1)).remove(0);
        let class = content.classes.get(&unit.class).expect("its class").clone();
        let nowhere = trpg_core::ClassId("nowhere".to_owned());
        assert!(!advance(&mut unit, &class, &nowhere, &content));
        let next = class.promotes_to.first().expect("a promotion");
        assert!(advance(&mut unit, &class, next, &content));
        assert_eq!(&unit.class, next);
    }

    #[test]
    fn gear_fits_the_class_one_weapon_a_kind() {
        let content = content();
        let mut worn = 0;
        for level in [5, 20, 40] {
            for unit in reference_army(&content, &army("ch02", level)) {
                let class = content.classes.get(&unit.class).expect("its class");
                let weapons: Vec<_> = unit.loadout.weapons.iter().flatten().collect();
                let defs: Vec<_> = weapons
                    .iter()
                    .map(|w| content.items.weapon(&w.def).expect("a weapon"))
                    .collect();
                for def in &defs {
                    assert!(unit.can_wield(def, class), "{} at {level}", unit.name);
                }
                for (i, def) in defs.iter().enumerate() {
                    let same = defs[..i].iter().any(|d| d.kind == def.kind);
                    assert!(!same, "{} carries two of a kind", unit.name);
                }
                let Some(armour) = &unit.loadout.armour else {
                    panic!("{} at {level} wears nothing", unit.name);
                };
                let Some(ItemDef::Armour(a)) = content.items.get(armour) else {
                    panic!("{armour:?} isn't armour");
                };
                assert!(class.can_wear(a.weight_class), "{} at {level}", unit.name);
                worn += 1;
            }
        }
        assert!(worn > 0);
    }

    #[test]
    fn a_unit_with_no_armour_buys_some() {
        let content = content();
        let mut unit = reference_army(&content, &army("ch01", 1)).remove(0);
        unit.loadout.armour = None;
        gear(&mut unit, &content, 2);
        assert!(unit.loadout.armour.is_some());
    }

    #[test]
    fn units_promote_on_the_way_up_and_can_fight() {
        let content = content();
        for (level, tier) in [(12, 1), (13, 2), (32, 2), (33, 3), (99, 3)] {
            assert_eq!(tier_at(level), tier);
            for unit in reference_army(&content, &army("ch02", level)) {
                let class = content.classes.get(&unit.class).expect("its class");
                assert_eq!(class.tier, tier, "{} at {level}", unit.name);
                assert!(unit.loadout.equipped.is_some(), "{} at {level}", unit.name);
            }
        }
    }

    #[test]
    fn gear_is_what_the_shop_sells_by_the_chapter() {
        let content = content();
        let weapons = |chapter| {
            let units = reference_army(&content, &army(chapter, 5));
            let held: Vec<String> = units
                .iter()
                .flat_map(|u| u.loadout.weapons.iter().flatten())
                .map(|w| w.def.0.clone())
                .collect();
            held
        };
        assert!(weapons("ch02").iter().any(|w| w.starts_with("iron_")));
        assert!(weapons("ch02").iter().all(|w| !w.starts_with("steel_")));
    }

    #[test]
    fn the_pack_fills_with_the_best_consumable_on_sale() {
        let content = content();
        let pack = reference_pack(&content, &army("ch02", 5), 3);
        assert_eq!(pack, vec![ItemId::new("potion"); 3]);
        assert_eq!(reference_pack(&content, &army("ch01", 5), 3), []);
    }

    #[test]
    fn the_story_chapters_start_at_the_new_game_file() {
        let content = content();
        let chapters = story_chapters(&content);
        assert_eq!(chapters.first().map(|c| c.id.as_str()), Some("ch01"));
        assert!(chapters.iter().any(|c| c.id == "ch02"));
    }

    #[test]
    fn fielding_keeps_the_campaign_s_own_units() {
        let content = content();
        let def = content.battles.get("ch02").expect("Chapter 2's battle");
        let lead = trpg_core::LeadProfile::new("Wen", trpg_core::LeadGender::Female);
        let mut campaign =
            trpg_content::battle_campaign(&content, def, trpg_core::GameMode::Classic, lead);
        field_army(&mut campaign, &content, &army("ch02", 20));
        assert!(campaign.roster.iter().all(|u| u.level == 20));
        assert!(campaign.roster.iter().any(|u| u.name == "Wen"));
    }
}
