//! The Playdate's info screen: its pages, in cells. What it says is tested
//! on its view (`battle/turn_tests.rs`, `art_tests.rs`).

use trpg_content::Ink;
use trpg_core::UnitId;

use super::*;
use crate::input::Action;
use crate::map_view::playdate::tests::SCREEN;
use crate::screens::battle::info::{self as logic, turn_page};
use crate::screens::battle::playdate::tests::ctx;
use crate::screens::battle::testing::skirmish;
use crate::screens::battle::view::{InfoEffectView, InfoSpellView};
use crate::screens::playdate::tests::{blank, c as ink, cell, row};
use crate::words::Words;

/// The info screen of the skirmish's unit `id` on page `page`, changed by
/// `change`.
fn view(id: u32, page: u8, change: impl FnOnce(&mut InfoView)) -> InfoView {
    let state = skirmish(&ctx(), 20);
    let unit = state.unit(UnitId(id)).unwrap();
    let mut v = logic::view((&state, Words::ENGLISH), unit);
    v.page = page;
    change(&mut v);
    v
}

/// `v`'s rows, painted over the screen.
fn rows(v: &InfoView) -> Vec<String> {
    let mut buf = blank();
    paint(&ctx(), v, SCREEN, &mut buf);
    (0..15).map(|y| row(&buf, y)).collect()
}

/// `v` painted over the screen.
fn painted(v: &InfoView) -> GlyphBuffer {
    let mut buf = blank();
    paint(&ctx(), v, SCREEN, &mut buf);
    buf
}

/// A blank row inside the window.
const BLANK: &str = "║                                                ║";

#[test]
fn the_first_page_is_the_unit_its_stats_and_its_effects() {
    assert_eq!(
        rows(&view(1, 0, |_| {})),
        [
            "╔═ Test Lord ════════════════════════════════════╗",
            "║ Exile  Lv 1                                    ║",
            "║ HP 19/19   ██████████                          ║",
            "║ EXP 0                                          ║",
            "║ Mov 5  foot                                    ║",
            BLANK,
            "║ Stats                                          ║",
            "║ Str   6   Spd   7                              ║",
            "║ Mag   1   Def   4                              ║",
            "║ Dex   7   Res   3                              ║",
            BLANK,
            BLANK,
            BLANK,
            BLANK,
            "╚═══════════════════ ◀ 1/3 ▶ ════════════════════╝",
        ]
    );
    // A timed effect beside the stats, and the stat it raises the bar.
    let v = view(1, 0, |v| {
        v.stats[0].value = 8;
        v.stats[0].changed = true;
        v.effects = vec![InfoEffectView {
            name: "Rally".into(),
            mods: "Str +2".into(),
            until: "until your next turn".into(),
        }];
    });
    let shown = rows(&v);
    assert_eq!(
        shown[6],
        "║ Stats                   Effects                ║"
    );
    assert_eq!(
        shown[7],
        "║ Str   8   Spd   7       Rally                  ║"
    );
    assert_eq!(
        shown[8],
        "║ Mag   1   Def   4       Str +2                 ║"
    );
    assert_eq!(
        shown[9],
        "║ Dex   7   Res   3       until your next turn   ║"
    );
    let buf = painted(&v);
    let lit = cell(&buf, 8, 7);
    assert_eq!(
        (lit.glyph, lit.fg, lit.bg),
        ('8', ink(Ink::White), ink(Ink::Black))
    );
    assert_eq!(cell(&buf, 8, 8).bg, ink(Ink::White));
}

#[test]
fn the_second_page_is_the_weapons_the_gear_and_the_ranks() {
    let shown = rows(&view(1, 1, |v| {
        v.affinities = vec![("Fire".into(), "Weak".into())];
        v.ranks.push(("Lance".into(), "E".into()));
    }));
    assert_eq!(
        shown,
        [
            "╔═ Test Lord ════════════════════════════════════╗",
            "║ Weapons                                        ║",
            "║ E Iron Sword                             20/20 ║",
            "║   Mt 5  Hit 90  Crit 0  Rng 1  Wt 2            ║",
            "║   Steel Sword                            25/25 ║",
            "║   Mt 8  Hit 75  Crit 0  Rng 1  Wt 4            ║",
            "║   --                                           ║",
            BLANK,
            BLANK,
            "║ Armor                   Accessory              ║",
            "║   Leather Vest            --                   ║",
            "║   Def +2  Res +2                               ║",
            "║ Weapon ranks            Affinities             ║",
            "║   Sword D  Lance E        Fire Weak            ║",
            "╚═══════════════════ ◀ 2/3 ▶ ════════════════════╝",
        ]
    );
}

#[test]
fn the_last_page_is_the_skills_then_the_spells_two_to_a_row_cut_at_the_frame() {
    let v = view(1, 2, |v| {
        let skill = v.skills[0].clone();
        v.skills = vec![skill; 3];
        v.spells = (1..=6)
            .map(|i| InfoSpellView {
                name: format!("Spell {i}"),
                equipped: i == 1,
                left: 2,
                uses: 3,
            })
            .collect();
    });
    let shown = rows(&v);
    assert_eq!(
        shown[1],
        "║ Skills                                         ║"
    );
    assert_eq!(
        shown[2],
        "║ A Inspire                                  2/2 ║"
    );
    assert_eq!(
        shown[3],
        "║   ally r2: +10 hit +10 avo                     ║"
    );
    assert_eq!(shown[8], BLANK);
    assert_eq!(
        shown[9],
        "║ Spells                                         ║"
    );
    assert_eq!(
        shown[10],
        "║ E Spell 1          2/3    Spell 2          2/3 ║"
    );
    assert_eq!(
        shown[12],
        "║   Spell 5          2/3    Spell 6          2/3 ║"
    );
    assert_eq!(shown[13], BLANK);
    // A fourth skill pushes the last spells past the frame: they are left
    // out, and nothing is painted over it.
    let more = view(1, 2, |w| {
        w.skills = vec![v.skills[0].clone(); 4];
        w.spells.clone_from(&v.spells);
    });
    let shown = rows(&more);
    assert_eq!(
        shown[13],
        "║   Spell 3          2/3    Spell 4          2/3 ║"
    );
    assert_eq!(
        shown[14],
        "╚═══════════════════ ◀ 3/3 ▶ ════════════════════╝"
    );
}

#[test]
fn an_enemy_has_its_own_name_in_the_frame_and_dashes_for_what_it_lacks() {
    let shown = rows(&view(4, 2, |_| {}));
    assert_eq!(
        shown[0],
        "╔═ Brigand ══════════════════════════════════════╗"
    );
    assert_eq!(
        shown[2],
        "║   --                                           ║"
    );
    assert_eq!(
        shown[4],
        "║ Spells                                         ║"
    );
}

#[test]
fn a_page_past_the_last_shows_the_last() {
    assert_eq!(rows(&view(1, 9, |_| {})), rows(&view(1, PAGES - 1, |_| {})));
}

#[test]
fn left_and_right_turn_the_pages_wrapping_and_one_page_never_turns() {
    assert_eq!(turn_page(Action::CursorRight, 0, 3), Some(1));
    assert_eq!(turn_page(Action::CursorRight, 2, 3), Some(0));
    assert_eq!(turn_page(Action::CursorLeft, 0, 3), Some(2));
    assert_eq!(turn_page(Action::CursorLeft, 2, 3), Some(1));
    assert_eq!(turn_page(Action::CursorDown, 0, 3), None);
    assert_eq!(turn_page(Action::CursorRight, 0, 1), None);
    // A page past the end turns from the last.
    assert_eq!(turn_page(Action::CursorLeft, 7, 3), Some(1));
}
