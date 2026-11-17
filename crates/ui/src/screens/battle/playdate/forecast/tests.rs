//! Where the Playdate's forecast goes and how its rows look, in cells and
//! pixels. What it says is tested on its view (`battle/forecast/tests.rs`).

use trpg_core::{Pos, StatValue};

use super::*;
use crate::map_view::playdate::tests::{SCREEN, braille, plains, shown};
use crate::screens::battle::forecast;
use crate::screens::battle::forecast::tests::targeting;
use crate::screens::battle::playdate::tests::ctx;
use crate::screens::battle::testing::skirmish;
use crate::screens::battle::view::{LeftView, SkillLineView};
use crate::screens::playdate::tests::{blank, c as ink, cell, find, row, rows};
use crate::words::Words;
use crate::{ConsoleSizes, ConsoleSpec};
use insta::assert_snapshot;

/// The lord's forecast against the brigand at `brigand_hp` (two strikes
/// of the lord's, one of the brigand's between them), changed by
/// `change`.
fn view(brigand_hp: StatValue, change: impl FnOnce(&mut ForecastView)) -> ForecastView {
    let state = skirmish(&ctx(), brigand_hp);
    let mut v = forecast::view((&state, Words::ENGLISH), &targeting(&state)).unwrap();
    change(&mut v);
    v
}

/// `v` painted over a plain map from its corner, and its window.
fn painted(v: &ForecastView) -> (GlyphBuffer, Rect) {
    let c = ctx();
    let scene = plains(&c);
    let mut buf = blank();
    paint(&c, &scene, v, SCREEN, &mut buf);
    (buf, rect(&c, &scene, v, SCREEN))
}

/// The forecast with its two units at `attacker` and `target`: where its
/// window goes.
fn placed(attacker: (i32, i32), target: (i32, i32)) -> Rect {
    let v = view(20, |v| {
        v.tiles = [
            Pos::new(attacker.0, attacker.1),
            Pos::new(target.0, target.1),
        ];
    });
    painted(&v).1
}

#[test]
fn the_forecast_goes_down_the_side_away_from_its_two_units() {
    // Both on the left half: the window on the right, from the top.
    let right = placed((7, 2), (8, 2));
    assert_eq!((right.x, right.y, right.w), (SCREEN.w - W, 0, W));
    // Both on the right: on the left.
    assert_eq!(placed((20, 2), (21, 3)).x, 0);
    // One under either side's window: away from the target.
    assert_eq!(placed((3, 2), (20, 2)).x, 0);
    assert_eq!(placed((20, 2), (3, 2)).x, SCREEN.w - W);
    // Each half has one: the attacker just left of the middle (its tile's
    // cells 23 and 24), the target right of it (25 and 26). Away from the
    // target.
    assert_eq!(placed((11, 2), (12, 2)).x, 0);
    assert_eq!(placed((12, 2), (11, 2)).x, SCREEN.w - W);
    // Below the window, a unit is covered by neither.
    let low = placed((3, 14), (20, 2));
    assert_eq!(low.x, 0, "{low:?}");
}

#[test]
fn the_rows_are_the_pcs_two_columns_strikes_and_totals() {
    let v = view(20, |_| {});
    let (buf, at) = painted(&v);
    // On the right: both units are on the left.
    assert_eq!(at, Rect::new(25, 0, W, 14));
    let shown: Vec<String> = (0..14).map(|y| row(&buf, y)).collect();
    assert_eq!(
        shown,
        [
            "╔═ Forecast ════════════╗",
            "║ Test Lord  Brigand    ║",
            "║ Iron S… 20 Iron A… 20 ║",
            "║ HP 19 → 11 HP 20 → 3  ║",
            "║ ██████▒▒▒▒ ██▒▒▒▒▒▒▒▒ ║",
            "║ Hit  96    Hit  65    ║",
            "║ Crit 3     Crit 0     ║",
            "╟───────────────────────╢",
            "║  8 dmg  → 1           ║",
            "║           2 ← 8 dmg   ║",
            "║  9 dmg  → 3           ║",
            "╟───────────────────────╢",
            "║ 17 ×2 Total   8 ×1    ║",
            "╚═══════════════════════╝",
        ]
    );
    // Black on the white inside.
    let name = cell(&buf, at.x + 2, 1);
    assert_eq!((name.fg, name.bg), (ink("text"), ink("panel_bg")));
}

#[test]
fn the_kill_has_the_skull_and_the_strikes_after_it_are_grey() {
    // The brigand at 3 HP falls to the first strike.
    let v = view(3, |_| {});
    let (buf, at) = painted(&v);
    let x = at.x + 2;
    let skulls: Vec<(i32, i32)> = buf
        .overlays()
        .iter()
        .map(|o| (o.rect.x / 8, o.rect.y / 16))
        .collect();
    assert_ne!(skulls, [] as [(i32, i32); 0]);
    assert!(skulls.iter().all(|&s| s == (x + 7, 8)), "{skulls:?}");
    assert!(buf.overlays().iter().all(|o| o.color == ink("text")));
    // The brigand's counter and the lord's second strike, after the fall.
    assert_eq!(
        cell(&buf, at.x + 2 + RIGHT + 2, 9).fg,
        ink("window_text_off")
    );
    assert_eq!(cell(&buf, x, 10).fg, ink("window_text_off"));
    assert_eq!(cell(&buf, x, 8).fg, ink("text"));
    // The brigand's 3 of 20 HP: two cells of 10, all of them lost.
    assert_eq!(find(&buf, 3, "HP  3 → 0"), Some(x + RIGHT));
    assert_eq!(find(&buf, 4, "▒▒        "), Some(x + RIGHT));
    // In pixels: the skull is drawn black over the white inside.
    let frame = shown(&buf);
    let picture = braille(&frame);
    assert_ne!(picture, "");
    let (cw, ch) = ConsoleSpec::PLAYDATE.cell_px();
    let (sx, sy) = ((x + 7) * cw, 8 * ch);
    let inked = (sx..sx + cw)
        .flat_map(|px| (sy..sy + ch).map(move |py| (px, py)))
        .filter(|&(px, py)| !crate::onebit::is_white(&frame, px, py))
        .count();
    assert!(inked > 20, "{inked} pixels of skull");
}

#[test]
fn an_effective_strike_has_its_mark_and_a_skill_its_line() {
    let v = view(20, |v| {
        v.strikes[0].effective = true;
        v.skill = Some(SkillLineView {
            name: "Keen Edge".into(),
            durability: Some((20, 17)),
        });
    });
    let (buf, at) = painted(&v);
    let x = at.x + 2;
    assert_eq!(find(&buf, 1, "Keen Edge  "), Some(x));
    // The uses it spends under the lord's weapon, whose name is then
    // alone; the brigand's uses stay after its weapon's.
    assert_eq!(find(&buf, 2, &v.sides[0].name), Some(x));
    assert_eq!(find(&buf, 3, "Iron Sword Iron A… 20"), Some(x));
    assert_eq!(find(&buf, 4, "20 → 17"), Some(x));
    // Two rows more than the screen has: the rules go, so the first
    // strike is on row 9.
    assert!(rows(&buf).iter().all(|r| !r.contains('╟')));
    assert_eq!(cell(&buf, x + 6, 9).glyph, '!');
    assert_eq!(cell(&buf, x + 6, 11).glyph, ' ');
}

#[test]
fn a_broken_weapon_says_so_under_it_and_the_bar_blinks() {
    let v = view(20, |v| {
        v.sides[1].left = Some(LeftView::Broken("(broken)".into()));
    });
    let (buf, at) = painted(&v);
    let rx = at.x + 2 + RIGHT;
    assert_eq!(find(&buf, 2, "Iron Axe  "), Some(rx));
    assert_eq!(find(&buf, 3, "(broken)"), Some(rx));
    assert_eq!(find(&buf, 4, "HP 20 → 3"), Some(rx));
    // On the blink's other half the HP that would be lost is black too.
    let mut c = ctx();
    c.clock_s = crate::screens::battle::glyph::forecast::LOST_BLINK_S;
    let scene = plains(&c);
    let mut full = blank();
    paint(&c, &scene, &view(20, |_| {}), SCREEN, &mut full);
    assert_eq!(find(&full, 4, "██████████ ██████████"), Some(at.x + 2));
}

#[test]
fn an_hp_line_stays_in_its_column() {
    // No Act 1 unit has 100 HP, but a line that long is cut at its column,
    // leaving the strikes' numbers' column and the target's alone.
    let v = view(20, |v| {
        v.sides[0].hp = crate::screens::battle::view::ForecastHp {
            now: 100,
            after: 100,
            max: 100,
        };
    });
    let (buf, at) = painted(&v);
    let x = at.x + 2;
    assert_eq!(cell(&buf, x + NUMBER, 3).glyph, ' ');
    assert_eq!(find(&buf, 3, "HP 20 → 3"), Some(x + RIGHT));
}

#[test]
fn a_unit_left_with_hp_keeps_a_black_cell() {
    let v = view(20, |v| {
        v.sides[1].hp = crate::screens::battle::view::ForecastHp {
            now: 28,
            after: 1,
            max: 28,
        };
    });
    let (buf, at) = painted(&v);
    assert_eq!(find(&buf, 4, "█▒▒▒▒▒▒▒▒▒"), Some(at.x + 2 + RIGHT));
}

#[test]
fn a_target_that_cant_strike_back_says_so_beside_the_first_strike() {
    let v = view(20, |v| {
        v.strikes.retain(|s| s.by == Side::Attacker);
        v.sides[1].hit = None;
        v.sides[1].crit = None;
        v.sides[1].total = None;
        v.no_counter = Some("no counter".into());
    });
    let (buf, at) = painted(&v);
    let rx = at.x + 2 + RIGHT;
    assert_eq!(find(&buf, 8, "no counter"), Some(rx));
    assert_eq!(find(&buf, 5, "--"), Some(rx + 5));
}

#[test]
fn too_many_rows_for_the_screen_lose_the_rules_then_the_last_notes() {
    let notes = |n: usize| (0..n).map(|i| format!("note {i}")).collect::<Vec<_>>();
    // 12 rows and a note fit the 13 inside the frame.
    let v = view(20, |v| v.notes = notes(1));
    let (buf, at) = painted(&v);
    assert_eq!(at.h, 15);
    assert_eq!(cell(&buf, at.x, 7).glyph, '╟');
    assert_eq!(find(&buf, 13, "note 0"), Some(at.x + 2));
    // Five notes don't: the rules go, and the last notes with them.
    let v = view(20, |v| v.notes = notes(5));
    let (buf, at) = painted(&v);
    assert_eq!(at.h, 15);
    assert!(rows(&buf).iter().all(|r| !r.contains('╟')));
    assert_eq!(find(&buf, 10, "Total"), Some(at.x + 2 + NUMBER - 4));
    assert_eq!(find(&buf, 13, "note 2"), Some(at.x + 2));
    assert!(rows(&buf).iter().all(|r| !r.contains("note 3")));
}

#[test]
fn a_unit_an_art_strikes_is_a_row_after_the_totals() {
    let v = view(20, |v| {
        v.struck = vec![crate::screens::battle::view::StruckView {
            name: "Raider".into(),
            faction: trpg_core::Faction::Enemy,
            hit: 82,
            hp: (11, 4),
            effective: false,
        }];
    });
    let (buf, at) = painted(&v);
    assert_eq!(at.h, 15);
    assert_eq!(find(&buf, 12, "Total"), Some(at.x + 2 + NUMBER - 4));
    assert_eq!(find(&buf, 13, "Raider     82 11 → 4"), Some(at.x + 2));
}

#[test]
fn two_rects_overlap_only_where_they_share_a_cell() {
    let a = Rect::new(2, 2, 3, 3);
    assert!(overlap(a, Rect::new(4, 4, 2, 2)));
    assert!(overlap(a, a));
    // Touching edges share no cell, on any side.
    for b in [
        Rect::new(5, 2, 2, 3),
        Rect::new(0, 2, 2, 3),
        Rect::new(2, 5, 3, 2),
        Rect::new(2, 0, 3, 2),
    ] {
        assert!(!overlap(a, b) && !overlap(b, a), "{b:?}");
    }
}

#[test]
fn a_target_below_the_window_counts_as_clear_of_it() {
    // The target on the left half but under the window's bottom row (the
    // window is 14 rows), the attacker on the right: the left window
    // covers neither, so it goes there, though the target is on the left.
    let at = placed((20, 2), (3, 14));
    assert_eq!((at.x, at.h), (0, 14));
}

#[test]
fn the_affinity_line_and_a_counters_kill_skull_sit_in_the_targets_column() {
    let v = view(20, |v| {
        v.sides[1].affinity = Some("(resist)".into());
        v.sides[0].affinity = Some("heals 9".into());
        // The brigand's counter kills the lord.
        v.strikes[1].kills = true;
    });
    let (buf, at) = painted(&v);
    let (x, rx) = (at.x + 2, at.x + 2 + RIGHT);
    // Under Crit, a row each side.
    assert_eq!(find(&buf, 7, "heals 9"), Some(x));
    assert_eq!(find(&buf, 7, "(resist)"), Some(rx));
    // The strikes a row lower: the counter on row 10, its skull after
    // its damage.
    let skulls: Vec<(i32, i32)> = buf
        .overlays()
        .iter()
        .map(|o| (o.rect.x / 8, o.rect.y / 16))
        .collect();
    assert_ne!(skulls, [] as [(i32, i32); 0]);
    assert!(skulls.iter().all(|&s| s == (rx + 2 + 7, 10)), "{skulls:?}");
}

#[test]
fn the_rules_stay_inside_the_window() {
    // On the left: the cell right of the frame is the map's, not a rule.
    let v = view(20, |v| {
        v.tiles = [Pos::new(20, 2), Pos::new(21, 2)];
    });
    let (buf, at) = painted(&v);
    assert_eq!(at.x, 0);
    assert_eq!(cell(&buf, at.x + at.w - 1, 7).glyph, '╢');
    assert_ne!(cell(&buf, at.x + at.w, 7).glyph, '─');
}

/// Ticket 0489: an effective spear strike, its `!` and the note that says
/// why.
#[test]
fn effective_spear_forecast_snapshot() {
    let v = view(20, |v| {
        v.strikes[0].effective = true;
        v.notes = vec!["Spear ×2 vs Mounted".into()];
    });
    let (buf, _) = painted(&v);
    assert!(rows(&buf).iter().any(|r| r.contains("Spear ×2 vs Mounted")));
    assert_snapshot!(rows(&buf).join(
        "
"
    ));
}

#[test]
fn steel_sword_forecast_snapshot() {
    // Its short name (0498) is too wide for the Playdate's 10 cells with
    // its uses, so it is the short name that is cut.
    let v = view(20, |v| {
        v.sides[0].weapon = "Steel Sword".into();
        v.sides[0].short = Some("Stl Sword".into());
        v.sides[0].left = Some(LeftView::Count { left: 25, max: 25 });
    });
    let (buf, _) = painted(&v);
    assert!(rows(&buf).iter().any(|r| r.contains("Stl Sw… 25")));
    assert_snapshot!(rows(&buf).join(
        "
"
    ));
}
