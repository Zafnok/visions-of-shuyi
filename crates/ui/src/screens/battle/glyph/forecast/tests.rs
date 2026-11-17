//! Where the forecast is painted and in which colours. What it says is
//! tested on its view (`battle/forecast/tests.rs`).

use super::*;
use crate::screen::tests::ctx;
use crate::screens::battle::forecast;
use crate::screens::battle::forecast::tests::targeting;
use crate::screens::battle::testing::skirmish;
use crate::screens::battle::view::{ForecastHp, TotalView};
use crate::words::Words;
use insta::assert_snapshot;

/// The lord's forecast against the brigand at `brigand_hp`, changed by
/// `change`.
fn view(brigand_hp: StatValue, change: impl FnOnce(&mut ForecastView)) -> ForecastView {
    let state = skirmish(&ctx(), brigand_hp);
    let mut v = forecast::view((&state, Words::ENGLISH), &targeting(&state)).unwrap();
    change(&mut v);
    v
}

fn render(v: &ForecastView) -> GlyphBuffer {
    let c = ctx();
    let blank = Cell::new(' ', Rgb::new(1, 2, 3), Rgb::new(1, 2, 3));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
    paint(&c, v, &mut buf);
    buf
}

fn text(buf: &GlyphBuffer, x: i32, y: i32, n: i32) -> String {
    (x..x + n).map(|x| buf.get(x, y).unwrap().glyph).collect()
}

/// The panel's row `y`, inside its border, trimmed.
fn row(buf: &GlyphBuffer, y: i32) -> String {
    text(buf, SIDE_PANEL.x + 1, y, SIDE_PANEL.w - 2)
        .trim()
        .to_owned()
}

#[test]
fn the_skull_is_one_overlay_per_run_of_pixels_inside_its_cell() {
    let red = Rgb::new(200, 0, 0);
    let runs = skull(3, 2, red);
    let pixels: i32 = runs.iter().map(|o| o.rect.w * o.rect.h).sum();
    let drawn = SKULL.iter().flat_map(|l| l.chars()).filter(|&c| c == 'X');
    assert_eq!(pixels, i32::try_from(drawn.count()).unwrap());
    let (cw, ch) = ConsoleSpec::PC.cell_px();
    for o in &runs {
        assert_eq!((o.color, o.layer, o.rect.h), (red, Layer::Over, 1));
        assert!(o.rect.x >= 3 * cw && o.rect.x + o.rect.w <= 4 * cw);
        assert!(o.rect.y >= 2 * ch && o.rect.y < 3 * ch);
    }
    // The top row: one run of 4 pixels, 2 in from the left.
    assert_eq!(runs[0].rect, Rect::new(3 * cw + 2, 2 * ch + 4, 4, 1));
    // A row with gaps: `X..XX..X` is three runs.
    let eyes: Vec<_> = runs.iter().filter(|o| o.rect.y == 2 * ch + 4 + 3).collect();
    assert_eq!(eyes.len(), 3);
}

#[test]
fn the_rows_of_a_forecast() {
    let buf = render(&view(20, |_| {}));
    // The title in the top border.
    assert!(text(&buf, SIDE_PANEL.x, SIDE_PANEL.y, 30).contains(" Forecast "));
    assert_eq!(row(&buf, SKILL_ROW), "");
    assert_eq!((NAME_ROW, WEAPON_ROW, UNDER_ROW, HP_ROW), (2, 3, 4, 4));
    assert_eq!(text(&buf, LEFT_X, NAME_ROW, 9), "Test Lord");
    assert_eq!(text(&buf, RIGHT_X, NAME_ROW, 7), "Brigand");
    // The uses after the weapon's name, not on a row of their own (0480).
    assert_eq!(text(&buf, LEFT_X, NAME_ROW + 1, 13), "Iron Sword 20");
    assert_eq!(text(&buf, RIGHT_X, NAME_ROW + 1, 11), "Iron Axe 20");
    assert_eq!(text(&buf, LEFT_X, HP_ROW, 2), "HP");
    assert_eq!(text(&buf, LEFT_X, HP_ROW + 2, 3), "Hit");
    assert_eq!(text(&buf, LEFT_X, HP_ROW + 3, 4), "Crit");
    assert_eq!(text(&buf, LEFT_X + 9, STRIKE_ROW, 1), "→");
    assert_eq!(text(&buf, NUMBER_X, STRIKE_ROW, 1), "1");
    assert_eq!(text(&buf, RIGHT_X, STRIKE_ROW + 1, 1), "←");
    assert_eq!(text(&buf, NUMBER_X, STRIKE_ROW + 1, 1), "2");
}

#[test]
fn a_side_reads_hp_now_and_after_hit_crit_and_its_uses() {
    let buf = render(&view(20, |v| {
        let side = &mut v.sides[0];
        side.hp = ForecastHp {
            now: 9,
            after: 2,
            max: 20,
        };
        (side.hit, side.crit) = (Some(85), Some(3));
        side.left = Some(LeftView::Count { left: 6, max: 10 });
        (v.sides[1].hit, v.sides[1].crit) = (None, None);
    }));
    let p = &ctx().palette;
    assert_eq!(text(&buf, LEFT_X, HP_ROW, 9), "HP  9 → 2");
    // The HP after in its HP colour, the arrow dimmed.
    let fg = |x, y| buf.get(x, y).unwrap().fg;
    assert_eq!(fg(LEFT_X + 8, HP_ROW), p.get(UiColor::HpLow));
    assert_eq!(fg(LEFT_X + 6, HP_ROW), p.get(UiColor::TextDim));
    assert_eq!(text(&buf, LEFT_X + 5, HP_ROW + 2, 3), "85 ");
    assert_eq!(text(&buf, LEFT_X + 5, HP_ROW + 3, 3), "3  ");
    assert_eq!(text(&buf, LEFT_X, NAME_ROW + 1, 13), "Iron Sword 6 ");
    assert_eq!(fg(LEFT_X + 11, NAME_ROW + 1), p.get(UiColor::Text));
    assert_eq!(fg(LEFT_X, NAME_ROW + 1), p.get(UiColor::TextDim));
    assert_eq!(text(&buf, RIGHT_X + 5, HP_ROW + 2, 3), "-- ");
    assert_eq!(text(&buf, RIGHT_X + 5, HP_ROW + 3, 3), "-- ");
    // A long name is cut with `…` to leave its uses room; one that just
    // fits is whole.
    let buf = render(&view(20, |v| v.sides[1].weapon = "Steel Sword".into()));
    assert_eq!(text(&buf, RIGHT_X, NAME_ROW + 1, 13), "Steel Swo… 20");
    let buf = render(&view(20, |v| v.sides[1].weapon = "Steel Axe X".into()));
    assert_eq!(text(&buf, RIGHT_X, NAME_ROW + 1, 13), "Steel Axe… 20");
    let buf = render(&view(20, |v| v.sides[1].weapon = "Steel Lance".into()));
    assert_eq!(text(&buf, RIGHT_X, NAME_ROW + 1, 13), "Steel Lan… 20");
    let buf = render(&view(20, |v| v.sides[1].weapon = "Silver Axe".into()));
    assert_eq!(text(&buf, RIGHT_X, NAME_ROW + 1, 13), "Silver Axe 20");
    // Nothing left to tell: the name alone.
    let buf = render(&view(20, |v| v.sides[0].left = None));
    assert_eq!(text(&buf, LEFT_X, NAME_ROW + 1, 13), "Iron Sword   ");
}

#[test]
fn a_broken_weapon_or_an_arts_cost_is_a_row_under_the_weapons() {
    let p = ctx().palette;
    let broken = render(&view(20, |v| {
        v.sides[1].left = Some(LeftView::Broken("(broken)".into()));
    }));
    assert_eq!(text(&broken, RIGHT_X, NAME_ROW + 1, 11), "Iron Axe   ");
    assert_eq!(text(&broken, RIGHT_X, UNDER_ROW, 8), "(broken)");
    let fg = broken.get(RIGHT_X, UNDER_ROW).unwrap().fg;
    assert_eq!(fg, p.get(UiColor::HpLow));
    // Both columns move down a row, and the strikes with them.
    assert_eq!(text(&broken, LEFT_X, HP_ROW + 1, 2), "HP");
    assert_eq!(text(&broken, NUMBER_X, STRIKE_ROW + 1, 1), "1");
    // An art's cost: the uses going down under the attacker's weapon, the
    // art's line only its name.
    let art = render(&view(20, |v| {
        v.skill = Some(SkillLineView {
            name: "Guard Break".into(),
            durability: Some((20, 16)),
        });
    }));
    assert_eq!(row(&art, SKILL_ROW), "Guard Break");
    assert_eq!(text(&art, LEFT_X, NAME_ROW + 1, 13), "Iron Sword   ");
    assert_eq!(text(&art, LEFT_X, UNDER_ROW, 7), "20 → 16");
    let fg = art.get(LEFT_X + 5, UNDER_ROW).unwrap().fg;
    assert_eq!(fg, p.get(UiColor::TextHighlight));
    // The target's uses stay after its weapon's name.
    assert_eq!(text(&art, RIGHT_X, NAME_ROW + 1, 11), "Iron Axe 20");
    assert_eq!(text(&art, RIGHT_X, UNDER_ROW, 8), "        ");
    assert_eq!(text(&art, LEFT_X, HP_ROW + 1, 2), "HP");
    // Without a cost: no row under the weapons.
    let active = render(&view(20, |v| {
        v.skill = Some(SkillLineView {
            name: "Keen Edge".into(),
            durability: None,
        });
    }));
    assert_eq!(row(&active, SKILL_ROW), "Keen Edge");
    assert_eq!(text(&active, LEFT_X, HP_ROW, 2), "HP");
}

#[test]
fn an_effective_strike_is_marked() {
    let c = ctx();
    let plain = render(&view(20, |_| {}));
    assert_eq!(text(&plain, LEFT_X + 6, STRIKE_ROW, 1), " ");
    let marked = render(&view(20, |v| {
        for s in v.strikes.iter_mut().filter(|s| s.by == Side::Attacker) {
            s.effective = true;
        }
    }));
    // `!` after both of the lord's strikes, in the highlight colour.
    for row in [STRIKE_ROW, STRIKE_ROW + 2] {
        let cell = marked.get(LEFT_X + 6, row).unwrap();
        assert_eq!(cell.glyph, '!');
        assert_eq!(cell.fg, c.palette.get(UiColor::TextHighlight));
    }
    assert_eq!(text(&marked, RIGHT_X + 8, STRIKE_ROW + 1, 1), " ");
}

#[test]
fn strikes_after_the_kill_are_dimmed_and_the_bars_shade_the_loss() {
    let c = ctx();
    let p = &c.palette;
    // The brigand at 3 HP falls to the first strike.
    let buf = render(&view(3, |_| {}));
    let fg = |x, y| buf.get(x, y).unwrap().fg;
    assert_eq!(fg(LEFT_X + 1, STRIKE_ROW), p.get(UiColor::Text));
    assert_eq!(fg(RIGHT_X + 3, STRIKE_ROW + 1), p.get(UiColor::TextDim));
    assert_eq!(fg(LEFT_X + 1, STRIKE_ROW + 2), p.get(UiColor::TextDim));
    // The skull on the first strike only.
    let skulls: Vec<i32> = buf
        .overlays()
        .iter()
        .filter(|o| o.color == p.get(UiColor::HpLow))
        .map(|o| o.rect.y / i32::from(ConsoleSpec::PC.cell_h))
        .collect();
    assert!(!skulls.is_empty() && skulls.iter().all(|&y| y == STRIKE_ROW));
    // The brigand at 3 of 20 falls: `→ 0` in `hp_low`. Its 3 HP round to
    // 2 cells of 12, all of them lost; the lord loses none of its 12.
    let after = buf.get(RIGHT_X + 8, HP_ROW).unwrap();
    assert_eq!((after.glyph, after.fg), ('0', p.get(UiColor::HpLow)));
    let bar = |x| text(&buf, x, HP_ROW + 1, HP_BAR_CELLS);
    assert_eq!(bar(LEFT_X), "████████████");
    assert_eq!(bar(RIGHT_X), "▒▒░░░░░░░░░░");
}

/// The cells of the lord's bar, its HP `now` and `after` of `max`, at
/// clock time `clock_s`.
fn lord_bar(now: StatValue, after: StatValue, max: StatValue, clock_s: f64) -> String {
    let v = view(20, |v| v.sides[0].hp = ForecastHp { now, after, max });
    let mut c = ctx();
    c.clock_s = clock_s;
    let blank = Cell::new(' ', Rgb::new(1, 2, 3), Rgb::new(1, 2, 3));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
    paint(&c, &v, &mut buf);
    text(&buf, LEFT_X, HP_ROW + 1, HP_BAR_CELLS)
}

#[test]
fn the_hp_that_would_be_lost_blinks_shaded_then_full() {
    assert_eq!(lord_bar(24, 12, 24, 0.0), "██████▒▒▒▒▒▒");
    assert_eq!(lord_bar(24, 12, 24, LOST_BLINK_S), "████████████");
    assert_eq!(
        lord_bar(24, 12, 24, 2.0 * LOST_BLINK_S + 0.1),
        "██████▒▒▒▒▒▒"
    );
    assert!(lost_shaded(0.0) && !lost_shaded(LOST_BLINK_S) && lost_shaded(f64::NAN));
    assert!(lost_shaded(LOST_BLINK_S * 0.99) && !lost_shaded(LOST_BLINK_S * 1.99));
    assert!(lost_shaded(f64::INFINITY) && lost_shaded(-3.0));
    // Full, the lost part is in the colour of the HP now.
    let v = view(20, |v| {
        v.sides[0].hp = ForecastHp {
            now: 24,
            after: 4,
            max: 24,
        };
    });
    let mut c = ctx();
    c.clock_s = LOST_BLINK_S;
    let blank = Cell::new(' ', Rgb::new(1, 2, 3), Rgb::new(1, 2, 3));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
    paint(&c, &v, &mut buf);
    let fg = buf.get(LEFT_X + 11, HP_ROW + 1).unwrap().fg;
    assert_eq!(fg, c.palette.get(UiColor::HpHigh));
}

#[test]
fn a_unit_left_with_hp_keeps_a_lit_cell() {
    // 1 HP of 28 rounds to no cell of 12; a unit that lives shows one.
    assert_eq!(lord_bar(28, 1, 28, 0.0), "█▒▒▒▒▒▒▒▒▒▒▒");
    assert_eq!(lord_bar(1, 1, 28, 0.0), "█░░░░░░░░░░░");
    // Only a fall shows none.
    assert_eq!(lord_bar(28, 0, 28, 0.0), "▒▒▒▒▒▒▒▒▒▒▒▒");
    assert_eq!(lord_bar(1, 0, 28, 0.0), "▒░░░░░░░░░░░");
}

#[test]
fn the_totals_go_under_a_rule_and_no_counter_beside_the_first_strike() {
    let buf = render(&view(20, |v| {
        v.strikes.truncate(1);
        v.sides[0].total = Some(TotalView {
            damage: 8,
            strikes: 1,
        });
        v.sides[1].total = None;
        v.no_counter = Some("no counter".into());
    }));
    assert_eq!(text(&buf, RIGHT_X + 2, STRIKE_ROW, 10), "no counter");
    assert!(row(&buf, STRIKE_ROW + 1).starts_with('─'));
    let totals = STRIKE_ROW + 2;
    assert_eq!(text(&buf, LEFT_X, totals, 4), " 8 ×");
    assert_eq!(text(&buf, NUMBER_X - 2, totals, 5), "Total");
    assert_eq!(text(&buf, RIGHT_X + 2, totals, 4), "    ");
}

#[test]
fn the_affinity_markers_go_under_the_crit() {
    let c = ctx();
    let buf = render(&view(20, |v| {
        v.sides[0].affinity = Some("(resist)".into());
        v.sides[1].affinity = Some("heals 9".into());
    }));
    assert_eq!(text(&buf, LEFT_X, AFFINITY_ROW, 8), "(resist)");
    assert_eq!(text(&buf, RIGHT_X, AFFINITY_ROW, 7), "heals 9");
    let fg = buf.get(LEFT_X, AFFINITY_ROW).unwrap().fg;
    assert_eq!(fg, c.palette.get(UiColor::TextHighlight));
    let plain = render(&view(20, |_| {}));
    assert_eq!(text(&plain, LEFT_X, AFFINITY_ROW, 8), "        ");
}

#[test]
fn notes_stop_above_the_bottom_border() {
    let c = ctx();
    let blank = Cell::new(' ', Rgb::new(1, 2, 3), Rgb::new(1, 2, 3));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
    let border = SIDE_PANEL.y + SIDE_PANEL.h - 1;
    let notes = ["pierces".to_owned(), "pierces".to_owned()];
    paint_notes(&c, &notes, border - 1, &mut buf);
    // One row is left for the two notes.
    assert_eq!(text(&buf, LEFT_X, border - 1, 7), "pierces");
    assert_eq!(text(&buf, LEFT_X, border, 7), "       ");
    assert_eq!(text(&buf, LEFT_X, border + 1, 7), "       ");
    let fg = buf.get(LEFT_X, border - 1).unwrap().fg;
    assert_eq!(fg, c.palette.get(UiColor::Effect));
    // After a forecast's totals, a row down.
    let buf = render(&view(20, |v| v.notes = vec!["pierces".into()]));
    let totals = STRIKE_ROW + i32::try_from(view(20, |_| {}).strikes.len()).unwrap() + 1;
    assert_eq!(text(&buf, LEFT_X, totals + 2, 7), "pierces");
}

#[test]
fn a_unit_an_art_strikes_is_a_row_under_the_totals() {
    let row = |effective| crate::screens::battle::view::StruckView {
        name: "Raider".into(),
        faction: trpg_core::Faction::Enemy,
        hit: 82,
        hp: (11, 4),
        effective,
    };
    let totals = STRIKE_ROW + i32::try_from(view(20, |_| {}).strikes.len()).unwrap() + 1;
    let buf = render(&view(20, |v| {
        v.struck = vec![row(false), row(true)];
        v.notes = vec!["pierces".into()];
    }));
    assert_eq!(
        text(&buf, LEFT_X, totals + 2, 25),
        "Raider    Hit  82  11 → 4"
    );
    // `!` after the hit of an effective strike only.
    assert_eq!(text(&buf, LEFT_X + 17, totals + 2, 1), " ");
    assert_eq!(text(&buf, LEFT_X + 17, totals + 3, 1), "!");
    let fg = buf.get(LEFT_X + 17, totals + 3).unwrap().fg;
    assert_eq!(fg, ctx().palette.get(UiColor::TextHighlight));
    // The notes after a blank row.
    assert_eq!(text(&buf, LEFT_X, totals + 5, 7), "pierces");
}

/// The lord's forecast with a Steel Sword of 25 uses, its short name
/// `short` (ticket 0498).
fn steel(short: Option<&str>) -> ForecastView {
    view(20, |v| {
        v.sides[0].weapon = "Steel Sword".into();
        v.sides[0].short = short.map(Into::into);
        v.sides[0].left = Some(LeftView::Count { left: 25, max: 25 });
    })
}

#[test]
fn a_weapon_shows_its_name_or_its_short_name_or_is_cut() {
    // The PC's 13 cells leave 10 for the name before ` 25`.
    let v = steel(Some("Stl Sword"));
    assert_eq!(weapon_name(&v.sides[0], Some(25), COLUMN_W), "Stl Sword");
    // With no uses after it, the whole name fits.
    assert_eq!(weapon_name(&v.sides[0], None, COLUMN_W), "Steel Sword");
    // A short name too wide too (the Playdate's 10 cells) is cut itself.
    assert_eq!(weapon_name(&v.sides[0], Some(25), 10), "Stl Sw…");
    // No short name: the name is cut.
    let bare = steel(None);
    assert_eq!(
        weapon_name(&bare.sides[0], Some(25), COLUMN_W),
        "Steel Swo…"
    );
    // A name that fits keeps it, short name or not.
    let iron = view(20, |v| v.sides[0].short = Some("Irn Swd".into()));
    assert_eq!(
        weapon_name(&iron.sides[0], Some(20), COLUMN_W),
        "Iron Sword"
    );
}

#[test]
fn the_columns_width_is_the_one_the_content_checks_short_names_by() {
    assert_eq!(COLUMN_W, trpg_content::item::WEAPON_LINE_CELLS);
}

#[test]
fn steel_sword_forecast_snapshot() {
    let buf = render(&steel(Some("Stl Sword")));
    assert_eq!(text(&buf, LEFT_X, WEAPON_ROW, 13), "Stl Sword 25 ");
    let rows: Vec<String> = (SIDE_PANEL.y..SIDE_PANEL.y + 12)
        .map(|y| row(&buf, y))
        .collect();
    assert_snapshot!(rows.join(
        "
"
    ));
}
