use std::sync::LazyLock;

use proptest::prelude::*;
use trpg_core::{Objective, Unit, UnitId};

use super::*;
use crate::testkit::content;

static CONTENT: LazyLock<Arc<Content>> = LazyLock::new(|| Arc::new(content().clone()));

/// Battle `id` as authored, with its knobs.
fn battle(id: &str) -> (TunedBattle, Vec<Box<dyn Knob<TunedBattle>>>) {
    let path = format!("battles/{id}.ron");
    let source = trpg_content::bundle::file(&path).unwrap_or_else(|| panic!("no {path}"));
    let text = BattleText::parse(&path, id, source).unwrap_or_else(|e| panic!("{e}"));
    let battle = TunedBattle::new(text, Arc::clone(&CONTENT));
    let knobs = knobs(&battle).unwrap_or_else(|e| panic!("{e}"));
    (battle, knobs)
}

fn knob<'a>(knobs: &'a [Box<dyn Knob<TunedBattle>>], name: &str) -> &'a dyn Knob<TunedBattle> {
    let found = knobs.iter().find(|k| k.name() == name);
    found.unwrap_or_else(|| panic!("no knob {name}")).as_ref()
}

fn levels(battle: &TunedBattle) -> Vec<Level> {
    let def = battle.load().unwrap_or_else(|e| panic!("{e}"));
    def.enemies.iter().map(|u| u.level).collect()
}

#[test]
fn the_enemy_numbers_come_in_order_unnamed_enemies_only() {
    let (_, knobs) = battle("quick");
    let names: Vec<&str> = knobs.iter().map(|k| k.name()).collect();
    // quick: three generic enemies and a stationary elemental, all
    // unnamed; three templates, so every one has a class knob.
    let at = ["Brigand at 8,2", "Brigand at 7,4", "Raider at 7,1"];
    assert_eq!(names[0], "unnamed enemies' level");
    assert_eq!(names[1..4], at.map(|a| format!("{a}: level"))[..]);
    assert_eq!(names.len(), 1 + 4 * 4);
    assert_eq!(knobs[0].stride(), 4);
    assert_eq!(knobs[0].range(), -10..=10);
    assert_eq!(knobs[1].range(), -5..=5);

    // ch01: the three with an `id` and the boss are named.
    let (ch01, knobs) = battle("ch01");
    let unnamed = ch01.text.entries().iter().filter(|e| e.unnamed).count();
    assert_eq!(unnamed, 10);
    assert!(knobs.iter().all(|k| !k.name().starts_with("Red Harl")));
    assert!(
        !knobs
            .iter()
            .any(|k| k.name().starts_with("Brigand at 10,10"))
    );
}

#[test]
fn the_level_knobs_move_levels_within_the_cap() {
    let (quick, knobs) = battle("quick");
    let before = levels(&quick);
    let up = knobs[0]
        .apply(&quick, 4)
        .unwrap_or_else(|| panic!("no apply"));
    let raised: Vec<Level> = before.iter().map(|l| l + 4).collect();
    assert_eq!(levels(&up), raised);
    let down = knobs[0]
        .apply(&quick, -10)
        .unwrap_or_else(|| panic!("no apply"));
    assert!(levels(&down).iter().all(|&l| l == 1));
    assert_eq!(knobs[0].describe(-4), "every unnamed enemy -4 levels");

    let one = knob(&knobs, "Brigand at 7,4: level");
    let up = one.apply(&quick, 2).unwrap_or_else(|| panic!("no apply"));
    assert_eq!(
        levels(&up),
        [before[0], before[1] + 2, before[2], before[3]]
    );
    assert_eq!(one.describe(2), "Brigand at 7,4 +2 levels");
}

#[test]
fn the_count_knob_removes_or_copies_until_no_tile_is_free() {
    let (quick, knobs) = battle("quick");
    let count = knob(&knobs, "Brigand at 7,4: count");
    let fewer = count
        .apply(&quick, -1)
        .unwrap_or_else(|| panic!("no apply"));
    assert_eq!(fewer.text.foes.len(), 3);
    assert!(!fewer.text.text().contains("pos: (7, 4)"));
    assert_eq!(count.describe(-1), "removed Brigand at 7,4");
    assert_eq!(count.describe(1), "one more beside Brigand at 7,4");

    let mut more = quick.clone();
    let mut added = 0;
    while let Some(next) = count.apply(&more, 1) {
        more = next;
        added += 1;
        assert!(added <= 12, "only 12 tiles lie within 2");
    }
    assert!(added > 0);
    let copies: Vec<&Foe> = more.text.foes.iter().filter(|f| f.copy).collect();
    assert_eq!(copies.len(), added);
    for copy in &copies {
        assert_eq!(copy.entry, 1);
        let (dx, dy) = (copy.pos.0 - 7, copy.pos.1 - 4);
        assert!((1..=2).contains(&(dx.abs() + dy.abs())), "{:?}", copy.pos);
    }
    // Each copy follows its original, in the same group.
    assert!(more.text.foes[1..=added].iter().all(|f| f.entry == 1));
}

#[test]
fn the_class_knob_picks_among_the_battle_s_templates() {
    let (quick, knobs) = battle("quick");
    let class = knob(&knobs, "Brigand at 8,2: class");
    // test_brigand is the first of three templates.
    assert_eq!(class.range(), 0..=2);
    let raider = class.apply(&quick, 1).unwrap_or_else(|| panic!("no apply"));
    assert_eq!(raider.text.foes[0].template.as_deref(), Some("test_raider"));
    assert_eq!(levels(&raider)[0], levels(&quick)[0]);
    assert_eq!(class.describe(1), "Brigand at 8,2 → Raider");
    assert!(class.apply(&quick, 3).is_none());
    assert!(class.apply(&quick, -1).is_none());
}

#[test]
fn the_weapon_knob_moves_a_tier_in_the_same_kind() {
    let (quick, knobs) = battle("quick");
    let weapon = knob(&knobs, "Brigand at 8,2: weapon");
    let steel = weapon
        .apply(&quick, 1)
        .unwrap_or_else(|| panic!("no apply"));
    let loadout = steel.text.foes[0].loadout.clone();
    assert_eq!(
        loadout.map(|l| l.weapons),
        Some(vec![ItemId::new("steel_axe")])
    );
    assert!(steel.text.text().contains("weapons: [\"steel_axe\"]"));
    // The iron axe is the lowest axe and the steel axe the highest.
    assert!(weapon.apply(&quick, -1).is_none());
    assert!(weapon.apply(&steel, 1).is_none());
    assert_eq!(weapon.describe(1), "Brigand at 8,2: weapon a tier up");
}

/// What the knobs must never change: the player's slots, the named units
/// (all but their unit ids, which follow the list), the reinforcements,
/// the map, the objective and the story moments.
fn authored_parts(battle: &TunedBattle) -> String {
    let def = battle.load().unwrap_or_else(|e| panic!("{e}"));
    let entries = battle.text.entries();
    let unid = |u: &Unit| Unit {
        id: UnitId(0),
        ..u.clone()
    };
    let named: Vec<Unit> = def
        .enemies
        .iter()
        .zip(&battle.text.foes)
        .filter(|(_, f)| !entries[f.entry].unnamed)
        .map(|(u, _)| unid(u))
        .collect();
    let target = match def.objective {
        Objective::DefeatUnit { unit, .. } => {
            let all = def
                .enemies
                .iter()
                .chain(def.reinforcements.iter().map(|r| &r.unit));
            all.clone().find(|u| u.id == unit).map(unid)
        }
        _ => None,
    };
    let mut objective = def.objective;
    if let Objective::DefeatUnit { unit, .. } = &mut objective {
        *unit = UnitId(0);
    }
    let reinforcements: Vec<(u32, Unit)> = def
        .reinforcements
        .iter()
        .map(|r| (r.turn, unid(&r.unit)))
        .collect();
    let slots: Vec<String> = def
        .player_slots
        .iter()
        .map(|s| format!("{} {:?}", s.character.0, s.pos))
        .collect();
    let notes = format!("{:?}", def.battle_notes);
    let parts = (
        (named, target, objective, reinforcements),
        (
            slots,
            def.map,
            def.triggers,
            notes,
            def.kind,
            def.difficulty,
        ),
    );
    format!("{parts:#?}")
}

#[test]
fn autobalance_never_touches_named_units_map_or_objectives() {
    for id in ["ch01", "quick", "test", "test_hold", "test_escape"] {
        let (authored, knobs) = battle(id);
        let parts = authored_parts(&authored);
        for knob in &knobs {
            for step in knob.range() {
                if let Some(changed) = knob.apply(&authored, step) {
                    let name = knob.name();
                    assert_eq!(authored_parts(&changed), parts, "{id}: {name} at {step}");
                }
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    /// Any setting of the knobs within their ranges, applied in order (a
    /// knob that can't apply skipped, as the search does), loads.
    #[test]
    fn any_setting_of_the_knobs_passes_the_content_check(
        raw in proptest::collection::vec(-10i32..=10, 64),
    ) {
        let (mut battle, knobs) = battle("quick");
        for (knob, raw) in knobs.iter().zip(raw) {
            let range = knob.range();
            let step = raw.clamp(*range.start(), *range.end());
            if step != 0 && let Some(next) = knob.apply(&battle, step) {
                battle = next;
            }
        }
        prop_assert!(battle.load().is_ok(), "{:?}", battle.load());
    }
}
