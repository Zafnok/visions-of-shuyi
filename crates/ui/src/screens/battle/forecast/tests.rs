//! What the forecast says. Where it is painted: `battle/glyph/forecast.rs`.

use trpg_core::{Pos, UnitId};

use super::*;
use crate::screen::tests::ctx;
use crate::screens::battle::attack::weapon_choices;
use crate::screens::battle::mode::Selection;
use crate::screens::battle::testing::skirmish;

/// The skirmish's lord at (7, 2) targeting the brigand (at
/// `brigand_hp`) with its iron sword.
pub(crate) fn targeting(state: &BattleState) -> Targeting {
    let mut sel = Selection::new(state, UnitId(1)).unwrap();
    sel.path = vec![sel.origin(), Pos::new(7, 2)];
    let choice = &weapon_choices(state, &sel)[0];
    let with = trpg_core::Equipped::Weapon(choice.slot);
    let plain = super::super::art_list::Technique::Attack;
    let mut t = Targeting::with(state, sel, with, plain, choice.targets.clone()).unwrap();
    t.cycle(true, state);
    assert_eq!(t.target(), UnitId(4));
    t
}

/// The forecast of `t`.
fn shown(state: &BattleState, t: &Targeting) -> ForecastView {
    view((state, Words::ENGLISH), t).expect("a forecast")
}

#[test]
fn strikes_read_as_damage_or_a_heal() {
    let s = PlannedStrike {
        by: Side::Attacker,
        damage: 8,
        healed: false,
        target_hp_after: 4,
        after_a_fall: false,
    };
    assert_eq!(strike_text(Words::ENGLISH, &s), " 8 dmg");
    assert_eq!(
        strike_text(Words::ENGLISH, &PlannedStrike { damage: 12, ..s }),
        "12 dmg"
    );
    assert_eq!(
        strike_text(Words::ENGLISH, &PlannedStrike { healed: true, ..s }),
        "+8 hp"
    );
}

#[test]
fn both_sides_their_numbers_strikes_and_totals() {
    let state = skirmish(&ctx(), 20);
    let t = targeting(&state);
    let v = shown(&state, &t);
    assert_eq!(
        (v.title.as_str(), v.hp_name.as_str(), v.total_name.as_str()),
        ("Forecast", "HP", "Total")
    );
    assert_eq!((v.hit_name.as_str(), v.crit_name.as_str()), ("Hit", "Crit"));
    let [lord, brigand] = &v.sides;
    assert_eq!(lord.name, "Test Lord");
    assert_eq!(lord.faction, trpg_core::Faction::Player);
    assert_eq!(brigand.name, "Brigand");
    assert_eq!(lord.left, Some(LeftView::Count { left: 20, max: 20 }));
    let numbers = t.preview.forecast;
    assert_eq!(
        (lord.hit, lord.crit),
        (Some(numbers.attacker.hit), Some(numbers.attacker.crit))
    );
    let counter = numbers.defender.expect("the brigand strikes back");
    assert_eq!(
        (brigand.hit, brigand.crit),
        (Some(counter.hit), Some(counter.crit))
    );
    let lord_hp = state.unit(UnitId(1)).unwrap();
    assert_eq!(
        lord.hp,
        ForecastHp {
            now: lord_hp.hp,
            after: t.preview.plan.attacker_hp,
            max: lord_hp.stats.hp,
        }
    );
    assert_eq!(brigand.hp.after, t.preview.plan.defender_hp);
    // Every strike in order, each side's added up.
    let plan = &t.preview.plan.strikes;
    assert_eq!(v.strikes.len(), plan.len());
    for (s, p) in v.strikes.iter().zip(plan) {
        assert_eq!(
            (s.by, s.kills, s.after_a_fall),
            (p.by, p.kills(), p.after_a_fall)
        );
        assert_eq!(s.text, strike_text(Words::ENGLISH, p));
    }
    let total = |by| {
        let mine: Vec<_> = plan.iter().filter(|s| s.by == by).collect();
        TotalView {
            damage: mine.iter().map(|s| s.damage).sum(),
            strikes: mine.len(),
        }
    };
    assert_eq!(lord.total, Some(total(Side::Attacker)));
    assert_eq!(brigand.total, Some(total(Side::Defender)));
    assert!(lord.total.unwrap().strikes > 0 && brigand.total.unwrap().strikes > 0);
    assert_eq!((v.no_counter, v.skill), (None, None));
    assert!(v.notes.is_empty());
}

#[test]
fn a_target_that_cant_strike_back_says_so_and_has_no_total() {
    let c = ctx();
    let state = skirmish(&c, 20);
    let mut t = targeting(&state);
    t.preview.forecast.defender = None;
    let v = shown(&state, &t);
    assert_eq!(v.no_counter.as_deref(), Some("no counter"));
    let brigand = &v.sides[1];
    assert_eq!(
        (brigand.hit, brigand.crit, brigand.total),
        (None, None, None)
    );
    assert_eq!(brigand.affinity, None);
    assert!(v.sides[0].total.is_some());
}

#[test]
fn an_effective_strike_is_marked_and_a_broken_weapon_says_so() {
    let c = ctx();
    let state = skirmish(&c, 20);
    let mut t = targeting(&state);
    let plain = shown(&state, &t);
    assert!(plain.strikes.iter().all(|s| !s.effective));
    t.preview.forecast.attacker.effective = true;
    t.preview.forecast.attacker.broken = true;
    let marked = shown(&state, &t);
    // Both of the lord's strikes; none of the brigand's.
    for s in &marked.strikes {
        assert_eq!(s.effective, s.by == Side::Attacker, "{s:?}");
    }
    assert_eq!(
        marked.sides[0].left,
        Some(LeftView::Broken("(broken)".into()))
    );
    assert_eq!(marked.sides[1].left, plain.sides[1].left);
}

#[test]
fn the_kill_and_the_strikes_after_it_are_flagged() {
    let c = ctx();
    // The brigand at 3 HP falls to the first strike.
    let state = skirmish(&c, 3);
    let v = shown(&state, &targeting(&state));
    let flags: Vec<_> = v
        .strikes
        .iter()
        .map(|s| (s.kills, s.after_a_fall))
        .collect();
    assert_eq!(flags[0], (true, false));
    assert!(flags[1..].iter().all(|&f| f == (false, true)), "{flags:?}");
    assert_eq!(v.sides[1].hp.after, 0);
}

#[test]
fn notes_skip_no_counter() {
    let c = ctx();
    let state = skirmish(&c, 20);
    let mut t = targeting(&state);
    t.preview.notes = vec![ArtNote::NoCounter, ArtNote::Pierces, ArtNote::Pierces];
    assert_eq!(shown(&state, &t).notes, ["pierces", "pierces"]);
}

#[test]
fn a_spell_or_nothing_equipped_names_the_counter_weapon_that_way() {
    let c = ctx();
    let state = skirmish(&c, 20);
    let brigand = &state.units()[3];
    assert_eq!(
        arms((&state, Words::ENGLISH), brigand, None),
        (String::new(), None)
    );
    // A spell it doesn't know has no uses left; a weapon has its
    // durability.
    let fire = Equipped::Spell(trpg_core::SpellId::new("fire"));
    assert_eq!(
        arms((&state, Words::ENGLISH), brigand, Some(&fire)),
        ("Fire".to_owned(), Some(Left::Uses(0, 10)))
    );
    assert_eq!(
        arms(
            (&state, Words::ENGLISH),
            brigand,
            brigand.loadout.equipped.as_ref()
        ),
        ("Iron Axe".to_owned(), Some(Left::Durability(20, 20)))
    );
    // An empty slot, or a spell missing from the table: nothing.
    let none = (String::new(), None);
    assert_eq!(
        arms(
            (&state, Words::ENGLISH),
            brigand,
            Some(&Equipped::Weapon(2))
        ),
        none
    );
    let lost = Equipped::Spell(trpg_core::SpellId::new("nope"));
    assert_eq!(arms((&state, Words::ENGLISH), brigand, Some(&lost)), none);
}

#[test]
fn affinity_markers_read_as_the_design_writes_them() {
    let c = ctx();
    let state = skirmish(&c, 20);
    let mut t = targeting(&state);
    let plain = t.preview.forecast.attacker;
    assert_eq!(affinity_text(Words::ENGLISH, &plain), None);
    let with = |affinity| SideForecast {
        affinity: Some(affinity),
        damage: 9,
        ..plain
    };
    assert_eq!(affinity_text(Words::ENGLISH, &with(Affinity::Weak)), None);
    assert_eq!(
        affinity_text(Words::ENGLISH, &with(Affinity::Resist)).as_deref(),
        Some("(resist)")
    );
    assert_eq!(
        affinity_text(Words::ENGLISH, &with(Affinity::Absorb)).as_deref(),
        Some("heals 9")
    );
    // On the striker's side; nothing without an affinity.
    assert_eq!(shown(&state, &t).sides[0].affinity, None);
    t.preview.forecast.attacker = with(Affinity::Resist);
    t.preview.forecast.defender = Some(with(Affinity::Absorb));
    let v = shown(&state, &t);
    assert_eq!(v.sides[0].affinity.as_deref(), Some("(resist)"));
    assert_eq!(v.sides[1].affinity.as_deref(), Some("heals 9"));
}

#[test]
fn a_spell_shows_its_uses_and_is_never_broken() {
    let c = ctx();
    let state = skirmish(&c, 20);
    let t = targeting(&state);
    let side = |left| {
        let column = Column {
            unit: &state.units()[0],
            weapon: "Fire".to_owned(),
            left,
            numbers: Some(t.preview.forecast.attacker),
            hp_after: 5,
            by: Side::Attacker,
        };
        side_view(Words::ENGLISH, &t.preview, column).left
    };
    let count = |left, max| Some(LeftView::Count { left, max });
    assert_eq!(side(Some(Left::Uses(6, 10))), count(6, 10));
    assert_eq!(side(Some(Left::Uses(0, 10))), count(0, 10));
    assert_eq!(side(Some(Left::Durability(3, 20))), count(3, 20));
    assert_eq!(
        side(Some(Left::Durability(0, 20))),
        Some(LeftView::Broken("(broken)".into()))
    );
    assert_eq!(side(None), None);
}
