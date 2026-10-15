//! The arts menu of the attack flow (tickets 0414 and 0442,
//! `docs/design/combat-arts.md` → *The arts menu*): `Arts` in the action
//! menu lists the Combat Arts of the weapons the unit carries, then its
//! combat actives that swing a weapon, each with its cost and source. What
//! can't be used from there is dimmed with the reason. Every check is
//! [`BattleState::preview_attack`], so the menu never offers what the core
//! would refuse (ADR-0004). A spell active is a line of the spell list
//! instead (`Fire + Overcast`, [`super::magic`]).

use trpg_core::{
    ArtDef, ArtId, ArtNote, BattleState, CastTarget, CombatMods, CommandError, CostError, Equipped,
    Event, Faction, Pos, SkillCost, SkillId, StatKind, Unit, UnitAction, UnitId, UnitTag,
    WEAPON_SLOTS,
};

use super::attack::{spell_uses, weapon_durability};
use super::info::stat_name;
use super::skills::{effect_text, mods_text, skill_cost, skill_name, timed_text};
use super::view::{MenuHeader, UsesView};
use crate::widgets::menu::{Menu, MenuItem, SuffixKind};
use crate::words::Words;

/// What an attack is made with: the plain weapon, a Combat Art or a combat
/// active (one at most, `combat-arts.md`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Technique {
    /// A plain attack.
    #[default]
    Attack,
    /// A Combat Art.
    Art(ArtId),
    /// A combat active.
    Active(SkillId),
}

impl Technique {
    /// The attack on `target` with `with` (a weapon's slot, or an attack
    /// spell, which has no arts), using this.
    pub fn action(&self, target: UnitId, with: &Equipped) -> UnitAction {
        let (art, active) = match self {
            Technique::Attack => (None, None),
            Technique::Art(id) => (Some(id.clone()), None),
            Technique::Active(id) => (None, Some(id.clone())),
        };
        match with {
            Equipped::Weapon(slot) => UnitAction::Attack {
                target,
                slot: *slot,
                active,
                art,
            },
            Equipped::Spell(spell) => UnitAction::Cast {
                spell: spell.clone(),
                target: CastTarget::Unit(target),
                active,
            },
        }
    }

    /// Its name as the arts menu and the forecast show it (none for a
    /// plain attack).
    pub fn name(&self, state: &BattleState, words: Words<'_>) -> String {
        match self {
            Technique::Attack => String::new(),
            Technique::Art(id) => art_name(state, words, id),
            Technique::Active(id) => skill_name(state, words, id),
        }
    }

    /// What it does in a few words, as the menus describe the focused
    /// line: `plain attack`, the art's [`art_effect_text`] or the active's
    /// [`effect_text`] (empty if the table lacks it).
    pub fn effect_text(&self, state: &BattleState, words: Words<'_>) -> String {
        match self {
            Technique::Attack => words.text("battle.art.desc.attack").to_owned(),
            Technique::Art(id) => state
                .arts()
                .get(id)
                .map(|a| art_effect_text(words, a))
                .unwrap_or_default(),
            Technique::Active(id) => state
                .skills()
                .get(id)
                .map(|s| effect_text(words, s))
                .unwrap_or_default(),
        }
    }
}

/// The name of a class tag (`Mounted`).
fn tag_name(words: Words<'_>, tag: UnitTag) -> &str {
    words.text(match tag {
        UnitTag::Mounted => "class.tag.mounted",
        UnitTag::Flying => "class.tag.flying",
        UnitTag::Armored => "class.tag.armored",
    })
}

/// An art's effect in a few words, built from its data: the number
/// changes first (`+10 hit`, `×3 vs Mounted`, `min range 1`), then what the
/// forecast's numbers don't show ([`note_text`], but `then strikes the
/// unit behind` for Line Pierce), apart by `; `.
pub fn art_effect_text(words: Words<'_>, art: &ArtDef) -> String {
    let e = &art.effect;
    let mut parts = vec![];
    if e.hit != 0 {
        let mods = CombatMods {
            hit: e.hit,
            ..CombatMods::default()
        };
        parts.push(mods_text(words, &mods));
    }
    if let Some((num, den)) = e.sword_followup {
        parts.push(words.text_with("battle.art.desc.followup", &[("num", &num), ("den", &den)]));
    }
    for &(tag, mult) in &e.effective {
        let tag = tag_name(words, tag);
        parts.push(words.text_with(
            "battle.art.desc.effective",
            &[("mult", &mult), ("tag", &tag)],
        ));
    }
    if let Some(n) = e.axe_min_damage {
        parts.push(words.text_with("battle.art.desc.axe_min", &[("n", &n)]));
    }
    if let Some(n) = e.min_range {
        parts.push(words.text_with("battle.art.desc.min_range", &[("n", &n)]));
    }
    parts.extend(e.notes().iter().map(|note| match note {
        ArtNote::Pierces => words.text("battle.art.desc.pierces").to_owned(),
        _ => note_text(words, note),
    }));
    parts.join("; ")
}

/// The name of art `id` (its id if the table lacks it).
pub fn art_name(state: &BattleState, words: Words<'_>, id: &ArtId) -> String {
    let art = state.arts().get(id);
    art.map_or(id.0.as_str(), |a| words.art(a)).to_owned()
}

/// One line of the arts menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtLine {
    /// What the line attacks with.
    pub technique: Technique,
    /// The weapon slots it can attack someone with from there, in slot
    /// order. Empty: the line is dimmed.
    pub slots: Vec<usize>,
    /// The slot of the weapon its forecast opens with: the equipped weapon
    /// if that is one of `slots`, else the first of them; for a dimmed
    /// line, the weapon whose cost or range stands in the way.
    pub slot: usize,
    /// What stands in the way, for a dimmed line: its cost
    /// ([`Fit::Cost`]: `broken`) or its range ([`Fit::Far`]: `out of
    /// range`). `None` if it can be chosen.
    pub blocked: Option<Fit>,
}

impl ArtLine {
    /// Whether the line can be chosen.
    pub fn usable(&self) -> bool {
        !self.slots.is_empty()
    }
}

/// Why a cost can't be paid, as a menu shows it.
pub fn reason_text(words: Words<'_>, error: &CostError) -> String {
    match error {
        CostError::WeaponBroken => words.text("battle.reason.broken").to_owned(),
        CostError::NotEnoughUses { left } => {
            words.text_with("battle.reason.uses_left", &[("left", left)])
        }
        CostError::NoUsesLeft => words.text("battle.reason.no_uses_left").to_owned(),
        CostError::NoWeapon | CostError::WrongSource => {
            words.text("battle.reason.cant_pay").to_owned()
        }
    }
}

/// Why a dimmed line can't be chosen, as a menu shows it after the line
/// (nothing for one that can).
pub fn blocked_text(words: Words<'_>, blocked: &Fit) -> String {
    match blocked {
        Fit::Cost(error) => reason_text(words, error),
        Fit::Far => words.text("battle.reason.out_of_range").to_owned(),
        Fit::Hits | Fit::No => String::new(),
    }
}

/// What stands between a technique with one weapon or spell and an attack
/// on someone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fit {
    /// Nothing: it can attack someone.
    Hits,
    /// Its cost can't be paid.
    Cost(CostError),
    /// Nobody is in its range.
    Far,
    /// It doesn't go with that weapon or spell.
    No,
}

/// How `technique` with `with` fits an attack by `unit` from `dest`: every
/// check is the core's ([`BattleState::preview_attack`] on each unit).
pub fn fit(
    state: &BattleState,
    unit: UnitId,
    dest: Pos,
    with: &Equipped,
    technique: &Technique,
) -> Fit {
    let mut found = Fit::No;
    for target in state.units() {
        match state.preview_attack(unit, dest, &technique.action(target.id, with)) {
            Ok(_) => return Fit::Hits,
            Err(
                CommandError::CannotPayArt { error, .. } | CommandError::CannotPay { error, .. },
            ) => found = Fit::Cost(error),
            // (The core checks the cost before the range: never both.)
            Err(CommandError::OutOfRange { .. }) => found = Fit::Far,
            Err(_) => {}
        }
    }
    found
}

/// The techniques `u` may have for the weapon in `slot`: its arts (rank arts
/// lowest rank first, then the weapon's own), then the unit's combat
/// actives. Whether an active goes with that weapon is the core's to say
/// ([`fit`]).
fn techniques(state: &BattleState, u: &Unit, slot: usize) -> Vec<Technique> {
    let arts = u.arts_for(slot, state.classes(), state.items(), state.arts());
    let actives = u.usable_skills(state.classes(), state.skills());
    let arts = arts.into_iter().map(|a| Technique::Art(a.id.clone()));
    let actives = actives.into_iter().filter(|s| s.is_combat());
    arts.chain(actives.map(|s| Technique::Active(s.id.clone())))
        .collect()
}

/// The arts menu's lines for `unit` at `dest` (ticket 0442): the Combat
/// Arts of every weapon it carries, then its combat actives that swing a
/// weapon, each once. A line is dimmed with the reason when no weapon can
/// use it on anyone from there.
pub fn art_lines(state: &BattleState, unit: UnitId, dest: Pos) -> Vec<ArtLine> {
    let Some(u) = state.unit(unit) else {
        return vec![];
    };
    // Arts before actives, each in the order of the first weapon with it.
    let mut found: Vec<(Technique, Vec<(usize, Fit)>)> = vec![];
    for slot in 0..WEAPON_SLOTS {
        for technique in techniques(state, u, slot) {
            let how = fit(state, unit, dest, &Equipped::Weapon(slot), &technique);
            match found.iter_mut().find(|(t, _)| *t == technique) {
                Some((_, fits)) => fits.push((slot, how)),
                None => found.push((technique, vec![(slot, how)])),
            }
        }
    }
    found.sort_by_key(|(t, _)| matches!(t, Technique::Active(_)));
    let equipped = u.loadout.equipped_slot();
    let line = |(technique, fits): (Technique, Vec<(usize, Fit)>)| {
        let with = |want: fn(&Fit) -> bool| fits.iter().filter(move |(_, how)| want(how));
        let slots: Vec<usize> = with(|how| *how == Fit::Hits).map(|&(s, _)| s).collect();
        let (slot, blocked) = if let Some(&first) = slots.first() {
            let slot = equipped.filter(|e| slots.contains(e)).unwrap_or(first);
            (slot, None)
        } else {
            let cost = with(|how| matches!(how, Fit::Cost(_))).next();
            let (slot, how) = cost.or_else(|| with(|how| *how == Fit::Far).next())?;
            (*slot, Some(how.clone()))
        };
        Some(ArtLine {
            technique,
            slots,
            slot,
            blocked,
        })
    };
    found.into_iter().filter_map(line).collect()
}

/// Whether `Arts` is enabled: some line can be chosen.
pub fn can_use_art(lines: &[ArtLine]) -> bool {
    lines.iter().any(ArtLine::usable)
}

/// What a line costs: `−4 dur` or `+1 use` (empty for `Attack`). A cost in
/// the skill's own uses reads `−1 use`, though no combat active has one.
pub fn cost_label(state: &BattleState, words: Words<'_>, technique: &Technique) -> String {
    let cost = match technique {
        Technique::Attack => None,
        Technique::Art(id) => state.arts().get(id).map(ArtDef::skill_cost),
        Technique::Active(id) => state.skills().get(id).and_then(skill_cost),
    };
    match cost {
        Some(SkillCost::Durability(n)) => {
            words.text_with("battle.art.cost.durability", &[("n", &n)])
        }
        Some(SkillCost::ExtraSpellUse) => words.text("battle.art.cost.spell_use").to_owned(),
        Some(SkillCost::Uses(_)) => words.text("battle.art.cost.use").to_owned(),
        None => String::new(),
    }
}

/// Where a line comes from: the art's rank (`E`, `D`), `weapon` for a
/// weapon's own art, `active` for a class active (empty for `Attack`).
pub fn source_label(state: &BattleState, words: Words<'_>, technique: &Technique) -> String {
    match technique {
        Technique::Attack => String::new(),
        Technique::Art(id) => match state.arts().get(id).and_then(|a| a.rank) {
            Some(rank) => format!("{rank:?}"),
            None => words.text("battle.art.source.weapon").to_owned(),
        },
        Technique::Active(_) => words.text("battle.art.source.active").to_owned(),
    }
}

/// Width of the name column.
const NAME_W: usize = 14;

/// The arts menu: `Guard Break     −4 dur  D`, a dimmed line with its
/// reason after it, focused on the first line that can be chosen. The
/// focused line's effect shows under the list (ticket 0429), dimmed lines
/// too: the focus stops on them.
pub fn arts_menu(state: &BattleState, words: Words<'_>, lines: &[ArtLine]) -> Menu {
    let items = lines
        .iter()
        .map(|c| {
            let name = c.technique.name(state, words);
            let cost = cost_label(state, words, &c.technique);
            let source = source_label(state, words, &c.technique);
            let label = format!("{name:<NAME_W$}  {cost:>6}  {source:<6}");
            let item = match &c.blocked {
                None => MenuItem::new(label),
                Some(why) => {
                    let why = format!(" {}", blocked_text(words, why));
                    MenuItem::disabled(label).with_suffix(why, SuffixKind::Warning)
                }
            };
            item.with_note(c.technique.effect_text(state, words))
        })
        .collect();
    Menu::new(items).browsing_disabled()
}

/// A weapon's durability as its lines show it: `20/20`, or `broken` at 0.
pub fn durability_text(words: Words<'_>, left: u32, max: u32) -> String {
    if left == 0 {
        words.text("battle.weapon.broken").to_owned()
    } else {
        format!("{left}/{max}")
    }
}

/// What a menu shows in its top border: the name and durability of
/// `unit`'s weapon (`Iron Sword 20/20`) or the name and uses of its spell
/// (`Fire 6/10`); `None` if it has no such weapon or spell.
pub fn header(
    (state, words): (&BattleState, Words<'_>),
    (unit, with): (UnitId, &Equipped),
) -> Option<MenuHeader> {
    let (name, left, max) = match with {
        Equipped::Weapon(slot) => weapon_durability(state, words, unit, *slot),
        Equipped::Spell(spell) => spell_uses(state, words, unit, spell),
    }?;
    Some(MenuHeader {
        name,
        uses: Some(UsesView {
            text: durability_text(words, left, max),
            spent: left == 0,
        }),
    })
}

/// The name of the art or active a boss or a green unit used in `events`
/// (`combat-arts.md`: "the forecast and playback show its name"), for the
/// playback's banner. `before` holds the units as they were before. The
/// player chose its own, so its units get none.
pub fn playback_banner(
    state: &BattleState,
    words: Words<'_>,
    events: &[Event],
    before: &[Unit],
) -> Option<String> {
    let not_player = |id: &UnitId| {
        before
            .iter()
            .any(|u| u.id == *id && u.faction != Faction::Player)
    };
    events.iter().find_map(|e| match e {
        Event::ArtUsed { unit, art, .. } if not_player(unit) => Some(art_name(state, words, art)),
        Event::SkillUsed { unit, skill } if not_player(unit) => {
            Some(skill_name(state, words, skill))
        }
        _ => None,
    })
}

/// One of the art's non-number effects as text: `no counter`, `pierces`,
/// `pins: Mov −3`, `slows: Spd −3`, `stance: +20 avo`.
pub fn note_text(words: Words<'_>, note: &ArtNote) -> String {
    match note {
        ArtNote::NoCounter => words.text("battle.art.note.no_counter").to_owned(),
        ArtNote::Pierces => words.text("battle.art.note.pierces").to_owned(),
        ArtNote::Debuff(d) => {
            let key = match d.stat {
                StatKind::Mov => "battle.art.note.pins",
                StatKind::Spd => "battle.art.note.slows",
                _ => "battle.art.note.lowers",
            };
            let stat = stat_name(words, d.stat);
            words.text_with(key, &[("stat", &stat), ("amount", &d.amount)])
        }
        ArtNote::Stance(mods) => words.text_with(
            "battle.art.note.stance",
            &[("mods", &timed_text(words, mods))],
        ),
    }
}

#[cfg(test)]
mod tests {
    use trpg_core::{CombatMods, Debuff, TimedMods};

    use super::*;

    #[test]
    fn reasons_say_what_is_missing() {
        assert_eq!(
            reason_text(Words::ENGLISH, &CostError::WeaponBroken),
            "broken"
        );
        assert_eq!(
            reason_text(Words::ENGLISH, &CostError::NotEnoughUses { left: 1 }),
            "1 uses left"
        );
        assert_eq!(
            reason_text(Words::ENGLISH, &CostError::NoUsesLeft),
            "no uses left"
        );
        assert_eq!(
            reason_text(Words::ENGLISH, &CostError::NoWeapon),
            "can't pay"
        );
        assert_eq!(
            reason_text(Words::ENGLISH, &CostError::WrongSource),
            "can't pay"
        );
    }

    #[test]
    fn notes_read_as_the_design_writes_them() {
        assert_eq!(note_text(Words::ENGLISH, &ArtNote::NoCounter), "no counter");
        assert_eq!(note_text(Words::ENGLISH, &ArtNote::Pierces), "pierces");
        let pin = Debuff {
            stat: StatKind::Mov,
            amount: 3,
        };
        assert_eq!(
            note_text(Words::ENGLISH, &ArtNote::Debuff(pin)),
            "pins: Mov −3"
        );
        let slow = Debuff {
            stat: StatKind::Spd,
            ..pin
        };
        assert_eq!(
            note_text(Words::ENGLISH, &ArtNote::Debuff(slow)),
            "slows: Spd −3"
        );
        let other = Debuff {
            stat: StatKind::Def,
            ..pin
        };
        assert_eq!(
            note_text(Words::ENGLISH, &ArtNote::Debuff(other)),
            "lowers: Def −3"
        );
        let stance = TimedMods {
            stats: vec![],
            combat: CombatMods {
                avoid: 20,
                ..CombatMods::default()
            },
        };
        assert_eq!(
            note_text(Words::ENGLISH, &ArtNote::Stance(stance)),
            "stance: +20 avo"
        );
    }

    #[test]
    fn costs_read_as_what_a_line_spends() {
        let c = crate::screen::tests::ctx();
        let (map, units) = crate::screens::battle::testing::quick_units(&c);
        let rout = trpg_core::Objective::Rout { turn_limit: None };
        let state = crate::screens::battle::testing::battle_with(&c, map, units, rout);
        let active =
            |id: &str| cost_label(&state, Words::ENGLISH, &Technique::Active(SkillId::new(id)));
        assert_eq!(cost_label(&state, Words::ENGLISH, &Technique::Attack), "");
        assert_eq!(
            cost_label(
                &state,
                Words::ENGLISH,
                &Technique::Art(ArtId::new("guard_break"))
            ),
            "−4 dur"
        );
        assert_eq!(active("keen_edge"), "−3 dur");
        assert_eq!(active("overcast"), "+1 use");
        // No combat active costs its own uses; a skill that does reads so.
        assert_eq!(active("brace"), "−1 use");
        assert_eq!(active("nope"), "");
    }

    #[test]
    fn every_art_says_what_it_does() {
        let c = crate::screen::tests::ctx();
        let arts = &c.content.arts;
        for art in arts.arts.values() {
            let text = art_effect_text(Words::ENGLISH, art);
            assert!(!text.is_empty(), "{} has no text", art.id.0);
            assert!(text.chars().count() <= 40, "{}: {text}", art.id.0);
        }
        let text = |id: &str| art_effect_text(Words::ENGLISH, arts.get(&ArtId::new(id)).unwrap());
        assert_eq!(text("guard_break"), "+10 hit; no counter");
        assert_eq!(text("unhorse"), "+10 hit; ×3 vs Mounted");
        assert_eq!(text("armor_cleave"), "×2 vs Armored");
        assert_eq!(text("pinning_shot"), "pins: Mov −3");
        assert_eq!(text("pressure_point"), "slows: Spd −3");
        assert_eq!(text("flowing_cut"), "follow-up ×3/2");
        assert_eq!(text("crushing_swing"), "+20 hit; min damage 8");
        assert_eq!(text("close_shot"), "-10 hit; min range 1");
        assert_eq!(text("line_pierce"), "then strikes the unit behind");
        assert_eq!(text("sidestep"), "stance: +20 avo");
    }

    #[test]
    fn techniques_say_what_they_do() {
        let c = crate::screen::tests::ctx();
        let (map, units) = crate::screens::battle::testing::quick_units(&c);
        let rout = trpg_core::Objective::Rout { turn_limit: None };
        let state = crate::screens::battle::testing::battle_with(&c, map, units, rout);
        let text = |t: Technique| t.effect_text(&state, Words::ENGLISH);
        assert_eq!(text(Technique::Attack), "plain attack");
        assert_eq!(
            text(Technique::Art(ArtId::new("guard_break"))),
            "+10 hit; no counter"
        );
        assert_eq!(
            text(Technique::Active(SkillId::new("keen_edge"))),
            "Sword: +30 hit +10 crit"
        );
        // Neither in the tables.
        assert_eq!(text(Technique::Art(ArtId::new("nope"))), "");
        assert_eq!(text(Technique::Active(SkillId::new("nope"))), "");
    }

    #[test]
    fn durability_reads_left_of_max_or_broken() {
        assert_eq!(durability_text(Words::ENGLISH, 20, 20), "20/20");
        assert_eq!(durability_text(Words::ENGLISH, 3, 20), "3/20");
        assert_eq!(durability_text(Words::ENGLISH, 0, 20), "broken");
    }

    #[test]
    fn a_plain_attack_has_no_art_or_active() {
        let t = Technique::default();
        assert_eq!(t, Technique::Attack);
        assert_eq!(
            t.action(UnitId(4), &Equipped::Weapon(1)),
            UnitAction::Attack {
                target: UnitId(4),
                slot: 1,
                active: None,
                art: None,
            }
        );
        let art = Technique::Art(ArtId::new("guard_break"));
        assert!(matches!(
            art.action(UnitId(4), &Equipped::Weapon(0)),
            UnitAction::Attack {
                art: Some(_),
                active: None,
                ..
            }
        ));
        let active = Technique::Active(SkillId::new("keen_edge"));
        assert!(matches!(
            active.action(UnitId(4), &Equipped::Weapon(0)),
            UnitAction::Attack {
                art: None,
                active: Some(_),
                ..
            }
        ));
    }

    #[test]
    fn with_a_spell_the_attack_is_a_cast_at_the_unit() {
        let fire = Equipped::Spell(trpg_core::SpellId::new("fire"));
        let cast = |active| UnitAction::Cast {
            spell: trpg_core::SpellId::new("fire"),
            target: CastTarget::Unit(UnitId(4)),
            active,
        };
        assert_eq!(Technique::Attack.action(UnitId(4), &fire), cast(None));
        let overcast = SkillId::new("overcast");
        assert_eq!(
            Technique::Active(overcast.clone()).action(UnitId(4), &fire),
            cast(Some(overcast))
        );
    }
}
