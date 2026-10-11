//! Skills in the battle UI (ticket 0412): the text that describes a skill
//! (its cost and one-line effect), and the `Skill` menu of non-combat
//! actives with the target mode of Shove (combat actives are chosen in the
//! attack flow's arts list, [`super::art_list`]). Legality is the core's: the attack forecast and
//! the commands refuse what a unit can't do, and this module only asks (a
//! command applied to a copy of the battle), so the player is never offered
//! a skill that would be refused (ADR-0004).

use trpg_core::{
    ActiveEffect, Area, BattleState, CombatMods, Command, CommandError, Condition, PassiveEffect,
    Phase, Pos, SkillCost, SkillDef, SkillId, SkillKind, StatKind, StatValue, TimedEffect,
    TimedMods, UnitAction, UnitId, WeaponKind, WeaponReq,
};

use super::art_list::reason_text;
use super::info::stat_name;
use super::mode::Selection;
use crate::color::UiColor;
use crate::widgets::menu::{Menu, MenuItem};
use crate::words::Words;

/// A skill's cost as text: `3 dur` (weapon durability), `+1 use` (one more
/// use of the spell) or, for a non-attack active, its uses left this battle
/// of its uses per battle, `2/3` (as a spell's uses are shown); `uses_left`
/// is only read for those.
pub fn cost_text(words: Words<'_>, cost: SkillCost, uses_left: u8) -> String {
    match cost {
        SkillCost::Durability(n) => words.text_with("battle.skill.cost.durability", &[("n", &n)]),
        SkillCost::ExtraSpellUse => words.text("battle.skill.cost.spell_use").to_owned(),
        SkillCost::Uses(max) => format!("{uses_left}/{max}"),
    }
}

/// The uses `unit` has left this battle of non-attack active `skill`.
pub fn uses_left(state: &BattleState, unit: UnitId, skill: &SkillId) -> u8 {
    state
        .unit(unit)
        .map_or(0, |u| u.skill_uses.uses_left(skill))
}

/// `+30 hit +10 crit`: the non-zero numbers and flags of `mods`.
pub fn mods_text(words: Words<'_>, mods: &CombatMods) -> String {
    let numbers = [
        (mods.hit, "battle.mods.hit"),
        (mods.crit, "battle.mods.crit"),
        (mods.might, "battle.mods.might"),
        (mods.avoid, "battle.mods.avoid"),
        (mods.attack_speed, "battle.mods.speed"),
        (mods.pierce, "battle.mods.pierce"),
    ];
    let mut parts: Vec<String> = numbers
        .iter()
        .filter(|(n, _)| *n != 0)
        .map(|(n, key)| words.text_with(key, &[("n", &format!("{n:+}"))]))
        .collect();
    if mods.extra_strikes > 0 {
        parts.push(words.text_with("battle.mods.extra_strikes", &[("n", &mods.extra_strikes)]));
    }
    let flags = [
        (mods.single_strike, "battle.mods.single_strike"),
        (mods.double_crit, "battle.mods.double_crit"),
        (mods.ignore_terrain, "battle.mods.ignore_terrain"),
        (mods.sword_followup.is_some(), "battle.mods.followup"),
    ];
    parts.extend(
        flags
            .iter()
            .filter(|(on, _)| *on)
            .map(|(_, key)| words.text(key).to_owned()),
    );
    parts.join(" ")
}

/// The name of a kind of weapon (`Sword`).
pub fn kind_name(words: Words<'_>, kind: WeaponKind) -> &str {
    words.text(match kind {
        WeaponKind::Sword => "battle.kind.sword",
        WeaponKind::Spear => "battle.kind.spear",
        WeaponKind::Axe => "battle.kind.axe",
        WeaponKind::Bow => "battle.kind.bow",
        WeaponKind::Gauntlet => "battle.kind.gauntlet",
    })
}

/// `Def +5 Res +5`: the stat bonuses of `stats`.
pub fn stats_text(words: Words<'_>, stats: &[(StatKind, StatValue)]) -> String {
    stats
        .iter()
        .map(|&(kind, n)| format!("{} {n:+}", stat_name(words, kind)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A timed effect's bonuses as text: its stats, then its combat numbers.
pub fn timed_text(words: Words<'_>, mods: &TimedMods) -> String {
    [
        stats_text(words, &mods.stats),
        mods_text(words, &mods.combat),
    ]
    .into_iter()
    .filter(|t| !t.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

/// When a passive's bonus applies, as a short tail (empty for always).
fn condition_text(words: Words<'_>, when: Condition) -> String {
    let kind = |key, kind| words.text_with(key, &[("kind", &kind_name(words, kind))]);
    match when {
        Condition::Always => String::new(),
        Condition::WeaponKindEquipped(k) => kind("battle.when.with_kind", k),
        Condition::NotOwnPhase => words.text("battle.when.attacked").to_owned(),
        Condition::HpAtMostHalf => words.text("battle.when.half_hp").to_owned(),
        Condition::MovedAtLeast(n) => words.text_with("battle.when.moved", &[("n", &n)]),
        Condition::AgainstWeaponKind(k) => kind("battle.when.against_kind", k),
    }
}

/// `text` and then `tail`, apart by a space unless the tail is empty.
fn then(text: String, tail: &str) -> String {
    if tail.is_empty() {
        text
    } else {
        format!("{text} {tail}")
    }
}

/// One passive effect as text, e.g. `+10 crit with Sword`.
fn passive_text(words: Words<'_>, effect: &PassiveEffect) -> String {
    match effect {
        PassiveEffect::StatWhile { stat, amount, when } => then(
            format!("{} {amount:+}", stat_name(words, *stat)),
            &condition_text(words, *when),
        ),
        PassiveEffect::CombatMod { mods, when } => {
            then(mods_text(words, mods), &condition_text(words, *when))
        }
        PassiveEffect::HealBonus(n) => {
            words.text_with("battle.passive.heal_bonus", &[("n", &format!("{n:+}"))])
        }
        PassiveEffect::SpellMight(n) => {
            words.text_with("battle.passive.spell_might", &[("n", &format!("{n:+}"))])
        }
        PassiveEffect::PostActionMove { tiles, when } => then(
            words.text_with("battle.passive.post_move", &[("tiles", tiles)]),
            &condition_text(words, *when),
        ),
        PassiveEffect::AllyAura { radius, mods } => words.text_with(
            "battle.passive.aura",
            &[("radius", radius), ("mods", &mods_text(words, mods))],
        ),
    }
}

/// An active's effect as text, e.g. `Sword: +30 hit +10 crit`.
fn active_text(words: Words<'_>, effect: &ActiveEffect) -> String {
    match effect {
        ActiveEffect::Strike {
            with,
            mods,
            range,
            stance,
            post_move,
            drain,
        } => {
            let mut parts = vec![mods_text(words, mods)];
            if *range > 0 {
                parts.push(words.text_with("battle.active.range", &[("range", range)]));
            }
            if let Some(stance) = stance {
                let mods = timed_text(words, &stance.mods);
                parts.push(words.text_with("battle.active.stance", &[("mods", &mods)]));
            }
            if *post_move > 0 {
                parts.push(words.text_with("battle.active.post_move", &[("tiles", post_move)]));
            }
            if *drain {
                parts.push(words.text("battle.active.drains").to_owned());
            }
            let body = parts
                .into_iter()
                .filter(|p| !p.is_empty())
                .collect::<Vec<_>>()
                .join(" ");
            match with {
                WeaponReq::Any => body,
                WeaponReq::Kind(kind) => format!("{}: {body}", kind_name(words, *kind)),
                WeaponReq::Spell => words.text_with("battle.active.spell", &[("body", &body)]),
            }
        }
        ActiveEffect::Buff { area, mods } => {
            let who = match area {
                Area::Own => words.text("battle.active.own").to_owned(),
                Area::Allies { radius } => {
                    words.text_with("battle.active.allies", &[("radius", radius)])
                }
            };
            format!("{who}: {}", timed_text(words, mods))
        }
        ActiveEffect::Heal { radius, power } => words.text_with(
            "battle.active.heal",
            &[("radius", radius), ("power", &format!("{power:+}"))],
        ),
        ActiveEffect::Push { collision } => {
            words.text_with("battle.active.push", &[("collision", collision)])
        }
    }
}

/// A skill's effect in a few words.
pub fn effect_text(words: Words<'_>, skill: &SkillDef) -> String {
    match &skill.kind {
        SkillKind::Passive(effects) => effects
            .iter()
            .map(|effect| passive_text(words, effect))
            .collect::<Vec<_>>()
            .join("; "),
        SkillKind::Active { effect, .. } => active_text(words, effect),
    }
}

/// An active's cost, `None` for a passive.
pub fn skill_cost(skill: &SkillDef) -> Option<SkillCost> {
    match skill.kind {
        SkillKind::Active { cost, .. } => Some(cost),
        SkillKind::Passive(_) => None,
    }
}

/// The name of skill `id` (its id if the table lacks it).
pub fn skill_name(state: &BattleState, words: Words<'_>, id: &SkillId) -> String {
    let skill = state.skills().get(id);
    skill.map_or(id.0.as_str(), |s| words.skill(s)).to_owned()
}

/// When a timed effect ends, e.g. `until Player phase`.
pub fn until_text<'a>(words: Words<'a>, effect: &TimedEffect) -> &'a str {
    words.text(match effect.until {
        Phase::Player => "battle.until.player",
        Phase::Enemy => "battle.until.enemy",
        Phase::Other => "battle.until.other",
    })
}

/// The non-combat actives `unit` knows, in the order of its skills.
fn known_non_combat(state: &BattleState, unit: UnitId) -> Vec<&SkillDef> {
    let Some(u) = state.unit(unit) else {
        return vec![];
    };
    u.usable_skills(state.classes(), state.skills())
        .into_iter()
        .filter(|s| s.is_active() && !s.is_combat())
        .collect()
}

/// Whether the action menu has a `Skill` entry: `unit` knows a non-combat
/// active (whether or not it can use it now).
pub fn has_skill_menu(state: &BattleState, unit: UnitId) -> bool {
    !known_non_combat(state, unit).is_empty()
}

/// One line of the `Skill` menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillChoice {
    /// The skill.
    pub skill: SkillId,
    /// Whether it is used on a chosen unit (Shove) rather than on no one
    /// in particular.
    pub needs_target: bool,
    /// Who it can be used on (empty when it needs no target).
    pub targets: Vec<UnitId>,
    /// Whether the core would accept it now; if not, the line is dimmed.
    pub usable: bool,
    /// Why it can't be paid for (`no uses left`), when that is what stops
    /// it: shown after its dimmed line.
    pub reason: Option<String>,
}

/// The command that uses `skill` from `sel`'s path end on `target`.
fn use_command(sel: &Selection, skill: &SkillId, target: Option<UnitId>) -> Command {
    Command::Act {
        unit: sel.unit,
        dest: sel.dest(),
        action: UnitAction::UseSkill {
            skill: skill.clone(),
            target,
        },
    }
}

/// Whether the core accepts `cmd` (applied to a copy).
fn accepted(state: &BattleState, cmd: &Command) -> bool {
    state.clone().apply(cmd).is_ok()
}

/// The `Skill` menu's lines for `sel`'s unit.
pub fn skill_choices(state: &BattleState, words: Words<'_>, sel: &Selection) -> Vec<SkillChoice> {
    known_non_combat(state, sel.unit)
        .into_iter()
        .map(|def| {
            let needs_target = matches!(
                def.kind,
                SkillKind::Active {
                    effect: ActiveEffect::Push { .. },
                    ..
                }
            );
            let (targets, usable) = if needs_target {
                let mut found: Vec<(Pos, UnitId)> = state
                    .units()
                    .iter()
                    .filter(|u| Pos::manhattan(sel.dest(), u.pos) == 1)
                    .filter(|u| accepted(state, &use_command(sel, &def.id, Some(u.id))))
                    .map(|u| (u.pos, u.id))
                    .collect();
                found.sort_by_key(|(p, _)| (p.y, p.x));
                let targets: Vec<UnitId> = found.into_iter().map(|(_, id)| id).collect();
                let usable = !targets.is_empty();
                (targets, usable)
            } else {
                let usable = accepted(state, &use_command(sel, &def.id, None));
                (vec![], usable)
            };
            // The cost is the core's first check, whoever the target is.
            let reason = match state.check(&use_command(sel, &def.id, None)) {
                Err(CommandError::CannotPay { error, .. }) => Some(reason_text(words, &error)),
                _ => None,
            };
            SkillChoice {
                skill: def.id.clone(),
                needs_target,
                targets,
                usable,
                reason,
            }
        })
        .collect()
}

/// Whether `Skill` is enabled: some skill can be used now.
pub fn can_use_skill(choices: &[SkillChoice]) -> bool {
    choices.iter().any(|c| c.usable)
}

/// The equipped weapon's `durability left/max`, as text.
fn equipped_durability(state: &BattleState, unit: UnitId) -> Option<(u32, u32)> {
    let u = state.unit(unit)?;
    let copy = u.loadout.weapon(u.loadout.equipped_slot()?)?;
    let def = state.items().weapon(&copy.def)?;
    Some((copy.durability_left, def.durability))
}

/// A line of the `Skill` menu: the name padded to `name_w`, then the cost
/// ([`cost_text`]), and for a cost in durability the equipped weapon's
/// durability `weapon` (left, max): `Brace     3/3`, or
/// `Brace   3 dur  Wpn 20/20`.
fn menu_line(
    words: Words<'_>,
    name: &str,
    name_w: usize,
    cost: Option<SkillCost>,
    uses_left: u8,
    weapon: Option<(u32, u32)>,
) -> String {
    let text = cost
        .map(|c| cost_text(words, c, uses_left))
        .unwrap_or_default();
    let weapon = match (cost, weapon) {
        (Some(SkillCost::Durability(_)), Some((left, max))) => {
            let weapon = words.text_with("battle.skill.weapon", &[("left", &left), ("max", &max)]);
            format!("  {weapon}")
        }
        _ => String::new(),
    };
    format!("{name:<name_w$}  {text:>6}{weapon}")
}

/// The `Skill` menu: each skill with its uses left this battle,
/// `Brace     3/3`. Unusable lines are dimmed, with the reason after one
/// that can't be paid for (`no uses left`).
pub fn skill_menu(
    state: &BattleState,
    words: Words<'_>,
    unit: UnitId,
    choices: &[SkillChoice],
) -> Menu {
    let name_w = choices
        .iter()
        .map(|c| skill_name(state, words, &c.skill).chars().count())
        .max()
        .unwrap_or(0);
    let weapon = equipped_durability(state, unit);
    let items = choices
        .iter()
        .map(|c| {
            let text = menu_line(
                words,
                &skill_name(state, words, &c.skill),
                name_w,
                state.skills().get(&c.skill).and_then(skill_cost),
                uses_left(state, unit, &c.skill),
                weapon,
            );
            match (&c.reason, c.usable) {
                (_, true) => MenuItem::new(text),
                (Some(why), false) => {
                    MenuItem::disabled(text).with_suffix(format!(" {why}"), UiColor::HpLow)
                }
                (None, false) => MenuItem::disabled(text),
            }
        })
        .collect();
    Menu::new(items)
}

/// Picking who a skill (Shove) is used on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillTargeting {
    /// The unit using the skill and its path (it stands at the path's end).
    pub sel: Selection,
    /// The `Skill` menu it was opened from (Cancel goes back to it).
    pub menu: Menu,
    /// The menu's lines.
    pub choices: Vec<SkillChoice>,
    /// Which line is being used.
    pub choice: usize,
    /// The target under the cursor, an index of the line's targets.
    pub index: usize,
}

impl SkillTargeting {
    /// Targeting for `choices[choice]`. `None` if it has no target.
    pub fn new(
        sel: Selection,
        menu: Menu,
        choices: Vec<SkillChoice>,
        choice: usize,
    ) -> Option<Self> {
        if choices.get(choice)?.targets.is_empty() {
            return None;
        }
        Some(Self {
            sel,
            menu,
            choices,
            choice,
            index: 0,
        })
    }

    /// The line being used.
    fn used(&self) -> &SkillChoice {
        // `choice` is valid and its targets are never empty ([`Self::new`]).
        &self.choices[self.choice]
    }

    /// The skill being used.
    pub fn skill(&self) -> &SkillId {
        &self.used().skill
    }

    /// The target under the cursor.
    pub fn target(&self) -> UnitId {
        self.used().targets[self.index]
    }

    /// Who the skill can be used on.
    pub fn targets(&self) -> &[UnitId] {
        &self.used().targets
    }

    /// Moves to the next target (or the previous one), wrapping.
    pub fn cycle(&mut self, forward: bool) {
        let n = self.used().targets.len();
        self.index = if forward {
            (self.index + 1) % n
        } else {
            (self.index + n - 1) % n
        };
    }

    /// The command that uses the skill.
    pub fn command(&self) -> Command {
        use_command(&self.sel, self.skill(), Some(self.target()))
    }

    /// The preview line, e.g. `Shove on Brigand (8 → 7 uses)`.
    pub fn preview(&self, state: &BattleState, words: Words<'_>) -> String {
        let name = skill_name(state, words, self.skill());
        let on = state.unit(self.target()).map_or(name.clone(), |u| {
            let unit = words.unit(u);
            words.text_with("battle.skill.on", &[("skill", &name), ("unit", &unit)])
        });
        let cost = cost_change(state, words, self.sel.unit, self.skill());
        format!("{on}{cost}")
    }
}

/// What paying for `skill` changes: ` (8 → 7 uses)` for its uses this
/// battle, ` (20 → 17)` for the equipped weapon's durability; empty if it
/// costs neither.
pub fn cost_change(state: &BattleState, words: Words<'_>, unit: UnitId, skill: &SkillId) -> String {
    let cost = state.skills().get(skill).and_then(skill_cost);
    match (cost, equipped_durability(state, unit)) {
        (Some(SkillCost::Uses(_)), _) => {
            let left = uses_left(state, unit, skill);
            let then = left.saturating_sub(1);
            let uses = words.text_with(
                "battle.skill.uses_change",
                &[("left", &left), ("then", &then)],
            );
            format!(" {uses}")
        }
        (Some(SkillCost::Durability(n)), Some((left, _))) => {
            format!(" ({left} → {})", left.saturating_sub(n))
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use trpg_core::SkillTable;

    use super::*;
    use crate::screen::tests::ctx;
    use crate::screens::battle::testing::{battle_with, quick_units};

    #[test]
    fn weapon_kinds_and_phases_are_named() {
        let w = Words::ENGLISH;
        let kinds = [
            WeaponKind::Sword,
            WeaponKind::Spear,
            WeaponKind::Axe,
            WeaponKind::Bow,
            WeaponKind::Gauntlet,
        ];
        assert_eq!(
            kinds.map(|k| kind_name(w, k)),
            ["Sword", "Spear", "Axe", "Bow", "Gauntlet"]
        );
        // As the data's own names for them.
        assert_eq!(
            kinds.map(|k| format!("{k:?}")),
            kinds.map(|k| kind_name(w, k))
        );
        let until = |until| {
            let effect = TimedEffect {
                source: trpg_core::EffectSource::Skill(SkillId::new("brace")),
                mods: TimedMods::default(),
                until,
            };
            until_text(w, &effect).to_owned()
        };
        assert_eq!(until(Phase::Player), "until Player phase");
        assert_eq!(until(Phase::Enemy), "until Enemy phase");
        assert_eq!(until(Phase::Other), "until Other phase");
        assert_eq!(
            condition_text(w, Condition::AgainstWeaponKind(WeaponKind::Bow)),
            "vs Bow"
        );
        assert_eq!(
            condition_text(w, Condition::WeaponKindEquipped(WeaponKind::Axe)),
            "with Axe"
        );
    }

    fn def<'a>(skills: &'a SkillTable, id: &str) -> &'a SkillDef {
        skills.get(&SkillId::new(id)).unwrap()
    }

    #[test]
    fn costs_read_as_durability_spell_uses_or_uses_left() {
        assert_eq!(
            cost_text(Words::ENGLISH, SkillCost::Durability(3), 9),
            "3 dur"
        );
        assert_eq!(
            cost_text(Words::ENGLISH, SkillCost::ExtraSpellUse, 9),
            "+1 use"
        );
        assert_eq!(cost_text(Words::ENGLISH, SkillCost::Uses(3), 2), "2/3");
        assert_eq!(cost_text(Words::ENGLISH, SkillCost::Uses(8), 0), "0/8");
    }

    #[test]
    fn menu_lines_show_uses_left_or_the_weapon_that_pays() {
        let line = |cost, weapon| menu_line(Words::ENGLISH, "Brace", 7, cost, 2, weapon);
        let uses = Some(SkillCost::Uses(3));
        assert_eq!(line(uses, Some((20, 20))), "Brace       2/3");
        assert_eq!(line(uses, None), "Brace       2/3");
        // A cost in durability names the weapon it is paid from.
        let dur = Some(SkillCost::Durability(3));
        assert_eq!(line(dur, Some((17, 20))), "Brace     3 dur  Wpn 17/20");
        assert_eq!(line(dur, None), "Brace     3 dur");
        assert_eq!(line(None, Some((17, 20))), "Brace          ");
    }

    #[test]
    fn effects_read_as_one_line() {
        let c = ctx();
        let skills = &c.content.skills;
        assert_eq!(
            effect_text(Words::ENGLISH, def(skills, "keen_edge")),
            "Sword: +30 hit +10 crit"
        );
        assert_eq!(
            effect_text(Words::ENGLISH, def(skills, "brace")),
            "self: Def +5 Res +5"
        );
        assert_eq!(
            effect_text(Words::ENGLISH, def(skills, "sword_focus_1")),
            "+10 crit with Sword"
        );
        assert_eq!(
            effect_text(Words::ENGLISH, def(skills, "steadfast_1")),
            "Def +2 when attacked"
        );
        assert_eq!(
            effect_text(Words::ENGLISH, def(skills, "shove")),
            "push 1 tile, 5 on impact"
        );
        assert_eq!(skill_cost(def(skills, "sword_focus_1")), None);
    }

    #[test]
    fn the_cost_change_shows_uses_or_durability() {
        let c = ctx();
        let (map, mut units) = quick_units(&c);
        for skill in ["shove", "keen_edge"] {
            assert!(units[0].learn_skill(&SkillId::new(skill), &c.content.skills));
        }
        let rout = trpg_core::Objective::Rout { turn_limit: None };
        let state = battle_with(&c, map, units, rout);
        let change =
            |skill: &str| cost_change(&state, Words::ENGLISH, UnitId(1), &SkillId::new(skill));
        assert_eq!(uses_left(&state, UnitId(1), &SkillId::new("shove")), 8);
        assert_eq!(change("shove"), " (8 → 7 uses)");
        // Keen Edge costs 3 of the equipped weapon's durability.
        let (left, _) = equipped_durability(&state, UnitId(1)).unwrap();
        assert_eq!(change("keen_edge"), format!(" ({left} → {})", left - 3));
        // A passive, a skill the table lacks and a unit that isn't there.
        assert_eq!(change("sword_focus_1"), "");
        assert_eq!(change("nope"), "");
        assert_eq!(uses_left(&state, UnitId(99), &SkillId::new("shove")), 0);
        assert_eq!(
            cost_change(&state, Words::ENGLISH, UnitId(99), &SkillId::new("shove")),
            " (0 → 0 uses)"
        );
        // The knight has no Shove: none of its uses.
        assert_eq!(uses_left(&state, UnitId(2), &SkillId::new("shove")), 0);
        assert_eq!(uses_left(&state, UnitId(2), &SkillId::new("brace")), 3);
    }

    #[test]
    fn range_and_moves_after_show_only_when_set() {
        let c = ctx();
        let skills = &c.content.skills;
        assert_eq!(
            effect_text(Words::ENGLISH, def(skills, "long_shot")),
            "Bow: +2 range"
        );
        assert_eq!(
            effect_text(Words::ENGLISH, def(skills, "vault")),
            "Bow: move 1 after"
        );
        assert!(!effect_text(Words::ENGLISH, def(skills, "keen_edge")).contains("range"));
        assert!(!effect_text(Words::ENGLISH, def(skills, "keen_edge")).contains("after"));
    }

    #[test]
    fn skill_targets_cycle_both_ways_and_wrap() {
        let c = ctx();
        let state = battle_with(
            &c,
            quick_units(&c).0,
            quick_units(&c).1,
            trpg_core::Objective::Rout { turn_limit: None },
        );
        let sel = Selection::new(&state, UnitId(1)).unwrap();
        let choice = SkillChoice {
            skill: SkillId::new("shove"),
            needs_target: true,
            targets: vec![UnitId(4), UnitId(5), UnitId(6)],
            usable: true,
            reason: None,
        };
        let menu = Menu::new(vec![]);
        let mut t = SkillTargeting::new(sel, menu, vec![choice], 0).unwrap();
        assert_eq!(t.targets(), [UnitId(4), UnitId(5), UnitId(6)]);
        let mut seen = vec![t.target()];
        for _ in 0..3 {
            t.cycle(true);
            seen.push(t.target());
        }
        assert_eq!(seen, [UnitId(4), UnitId(5), UnitId(6), UnitId(4)]);
        t.cycle(false);
        assert_eq!(t.target(), UnitId(6));
        t.cycle(false);
        assert_eq!(t.target(), UnitId(5));
        assert_eq!(
            t.command(),
            use_command(&t.sel, &SkillId::new("shove"), Some(UnitId(5)))
        );
    }

    #[test]
    fn every_skill_has_a_short_effect_line() {
        let c = ctx();
        for skill in c.content.skills.skills.values() {
            let text = effect_text(Words::ENGLISH, skill);
            assert!(!text.is_empty(), "{} has no text", skill.id.0);
        }
    }

    #[test]
    fn mods_and_stats_list_only_what_is_set() {
        assert_eq!(mods_text(Words::ENGLISH, &CombatMods::default()), "");
        let mods = CombatMods {
            avoid: -3,
            extra_strikes: 1,
            double_crit: true,
            ..CombatMods::default()
        };
        assert_eq!(
            mods_text(Words::ENGLISH, &mods),
            "-3 avo +1 strike double crit"
        );
        assert_eq!(
            stats_text(Words::ENGLISH, &[(StatKind::Def, 5), (StatKind::Res, -2)]),
            "Def +5 Res -2"
        );
    }
}
