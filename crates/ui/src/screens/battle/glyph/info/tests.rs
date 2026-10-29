//! Where the info screen is painted. What it says is tested on its view
//! (`battle/turn_tests.rs`, `battle/view_tests.rs`).

use trpg_core::{Faction, UnitId};

use super::*;
use crate::color::Rgb;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::screen::tests::ctx;
use crate::screens::battle::info;
use crate::screens::battle::quick_battle;
use crate::screens::battle::view::{InfoEffectView, InfoSkillView, InfoSpellView};
use crate::words::Words;

/// The quick battle's lord's info screen, changed by `change`, painted.
fn painted(change: impl FnOnce(&mut InfoView)) -> GlyphBuffer {
    let c = ctx();
    let state = quick_battle(&c.game.content).unwrap();
    let lord = state.unit(UnitId(1)).unwrap();
    let mut view = info::view((&state, Words::ENGLISH), lord);
    change(&mut view);
    let blank = Cell::new(' ', Rgb::new(0, 0, 0), Rgb::new(0, 0, 0));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
    paint(&c, &view, &mut buf);
    buf
}

fn text(buf: &GlyphBuffer, x: i32, y: i32, n: i32) -> String {
    (x..x + n).map(|x| buf.get(x, y).unwrap().glyph).collect()
}

#[test]
fn the_columns_fit_inside_the_border() {
    const { assert!(PORTRAIT.x + PORTRAIT.w < MID_X) };
    assert!(RIGHT_X + i32::try_from(RIGHT_W).unwrap() < INFO.x + INFO.w - 1);
    const { assert!(ITEM_W + 2 <= RIGHT_W) };
    // The lists end above the bottom border (row 29).
    assert!(16 + 2 * i32::try_from(SKILL_MAX).unwrap() < INFO.h);
    assert!(18 + i32::try_from(SPELL_ROWS).unwrap() < INFO.h);
}

#[test]
fn the_middle_column_reads_name_class_exp_hp_and_stats() {
    let c = ctx();
    let buf = painted(|v| {
        v.name = "Ana".into();
        v.faction = Faction::Enemy;
        v.class_level = "Lord  Lv 3".into();
        v.exp = Some(42);
        (v.hp, v.max) = (5, 20);
        v.stats[0].value = 7;
        v.stats[1].changed = true;
    });
    assert_eq!(text(&buf, MID_X, 1, 3), "Ana");
    assert_eq!(buf.get(MID_X, 1).unwrap().fg, c.palette.get(UiColor::Enemy));
    assert_eq!(text(&buf, MID_X, 2, 10), "Lord  Lv 3");
    assert_eq!(text(&buf, MID_X, 3, 6), "EXP 42");
    assert_eq!(text(&buf, MID_X, 4, 8), "HP 5/20 ");
    // 5 of 20 HP: 3 of the bar's 10 cells.
    assert_eq!(text(&buf, MID_X + 11, 4, 10), "███░░░░░░░");
    assert_eq!(text(&buf, MID_X, 6, 5), "Stats");
    assert_eq!(text(&buf, MID_X, 7, 7), "Str   7");
    let changed = buf.get(MID_X + 6, 8).unwrap().fg;
    assert_eq!(changed, c.palette.get(UiColor::Effect));
    let plain = buf.get(MID_X + 6, 7).unwrap().fg;
    assert_eq!(plain, c.palette.get(UiColor::Text));
    // At the level cap: no EXP.
    let buf = painted(|v| v.exp = None);
    assert_eq!(text(&buf, MID_X, 3, 6), "EXP --");
    // The placeholder, centred in the portrait area.
    let label = text(
        &buf,
        PORTRAIT.x + 1,
        PORTRAIT.y + PORTRAIT.h / 2,
        PORTRAIT.w - 2,
    );
    assert_eq!(label, "       portrait       ");
}

#[test]
fn effects_and_skills_stop_where_their_rows_do() {
    let effect = |n| InfoEffectView {
        name: format!("Effect {n}"),
        mods: "Def +2".into(),
        until: "until your turn".into(),
    };
    let skill = |n, cost: Option<&str>| InfoSkillView {
        mark: if cost.is_some() { "A" } else { "P" }.into(),
        name: format!("Skill {n}"),
        cost: cost.map(str::to_owned),
        effect: "does a thing".into(),
    };
    let buf = painted(|v| {
        v.effects = (0..3).map(effect).collect();
        v.skills = vec![skill(0, Some("2/3"))];
        v.skills.extend((1..8).map(|n| skill(n, None)));
    });
    assert_eq!(text(&buf, EFFECT_X, 6, 7), "Effects");
    assert_eq!(text(&buf, EFFECT_X, 7, 8), "Effect 0");
    assert_eq!(text(&buf, EFFECT_X, 8, 6), "Def +2");
    assert_eq!(text(&buf, EFFECT_X, 10, 8), "Effect 1");
    // Two effects only.
    assert_eq!(text(&buf, EFFECT_X, 13, 8), "        ");
    assert_eq!(text(&buf, MID_X, 16, 9), "A Skill 0");
    assert_eq!(text(&buf, MID_X + 22, 16, 3), "2/3");
    assert_eq!(text(&buf, MID_X + 2, 17, 12), "does a thing");
    // Six skills, two rows each.
    assert_eq!(text(&buf, MID_X, 26, 9), "P Skill 5");
    assert_eq!(text(&buf, MID_X, 28, 9), "         ");
    let none = painted(|v| v.skills.clear());
    assert_eq!(text(&none, MID_X, 16, 2), "--");
}

#[test]
fn the_left_column_lists_mov_tags_ranks_and_affinities() {
    let buf = painted(|v| {
        v.mov = 5;
        v.movement = Some("foot".into());
        v.tags = vec!["Mounted".into(), "Flying".into()];
        v.ranks = vec![("Sword".into(), "D".into())];
        v.affinities = vec![("Ice".into(), "Absorb".into())];
    });
    let y = PORTRAIT.y + PORTRAIT.h + 1;
    assert_eq!(text(&buf, LEFT_X, y, 11), "Mov 5  foot");
    assert_eq!(text(&buf, LEFT_X, y + 1, 14), "Mounted Flying");
    assert_eq!(text(&buf, LEFT_X, y + 3, 12), "Weapon ranks");
    assert_eq!(text(&buf, LEFT_X, y + 4, 11), "Sword     D");
    assert_eq!(text(&buf, LEFT_X, y + 6, 10), "Affinities");
    assert_eq!(text(&buf, LEFT_X, y + 7, 16), "Ice       Absorb");
    // No affinities: no heading.
    let buf = painted(|v| {
        v.ranks = vec![("Sword".into(), "D".into())];
        v.affinities.clear();
    });
    assert_eq!(text(&buf, LEFT_X, y + 6, 10), "          ");
}

/// 1003: the supports under the affinities (or the ranks), cut at the
/// box's bottom; none, no heading.
#[test]
fn the_left_column_lists_the_supports_last() {
    let y = PORTRAIT.y + PORTRAIT.h + 1;
    let buf = painted(|v| {
        v.ranks = vec![("Sword".into(), "D".into())];
        v.affinities.clear();
        v.supports_title = "Supports".into();
        v.supports = vec![("Mira".into(), "B".into())];
    });
    assert_eq!(text(&buf, LEFT_X, y + 6, 8), "Supports");
    assert_eq!(text(&buf, LEFT_X, y + 7, 21), "Mira                B");
    let many = painted(|v| {
        v.supports_title = "Supports".into();
        v.supports = vec![("Mira".into(), "B".into()); 40];
    });
    let bottom = INFO.y + INFO.h - 1;
    assert_eq!(text(&many, LEFT_X, bottom, 4), "════");
    assert_eq!(text(&many, LEFT_X, bottom - 1, 4), "Mira");
    // Ranks to the box's last row: no room for the heading.
    let full = painted(|v| {
        v.affinities.clear();
        v.ranks = vec![("Sword".into(), "D".into()); 11];
        v.supports_title = "Supports".into();
        v.supports = vec![("Mira".into(), "B".into())];
    });
    assert_eq!(text(&full, LEFT_X, bottom, 4), "════");
    assert_eq!(text(&full, LEFT_X, bottom - 1, 5), "Sword");
    let none = painted(|v| {
        v.ranks = vec![("Sword".into(), "D".into())];
        v.affinities.clear();
        v.supports.clear();
    });
    assert_eq!(text(&none, LEFT_X, y + 6, 8), "        ");
}

#[test]
fn the_right_column_lists_weapons_gear_and_spells() {
    let c = ctx();
    let buf = painted(|v| {
        let w = v.weapons[0].as_mut().expect("a sword");
        w.broken = Some("broken".into());
        v.weapons[1] = None;
        v.spells = vec![InfoSpellView {
            name: "Fire".into(),
            equipped: true,
            left: 6,
            uses: 10,
        }];
    });
    assert_eq!(text(&buf, RIGHT_X, 1, 7), "Weapons");
    assert_eq!(text(&buf, RIGHT_X, 2, 12), "E Iron Sword");
    let right = RIGHT_X + i32::try_from(RIGHT_W).unwrap();
    assert_eq!(text(&buf, right - 6, 2, 6), "broken");
    let broken = buf.get(RIGHT_X + 2, 2).unwrap().fg;
    assert_eq!(broken, c.palette.get(UiColor::HpLow));
    assert_eq!(text(&buf, RIGHT_X + 2, 4, 2), "--");
    // Gear from row 9, four rows each, then the spells.
    assert_eq!(text(&buf, RIGHT_X, 9, 5), "Armor");
    assert_eq!(text(&buf, RIGHT_X, 13, 9), "Accessory");
    assert_eq!(text(&buf, RIGHT_X + 2, 14, 2), "--");
    assert_eq!(text(&buf, RIGHT_X, 17, 6), "Spells");
    assert_eq!(text(&buf, RIGHT_X, 18, 6), "E Fire");
    assert_eq!(text(&buf, right - 4, 18, 4), "6/10");
}
