//! Where the forecast is painted and in which colours. What it says is
//! tested on its view (`battle/forecast/tests.rs`).

use super::*;
use crate::screen::tests::ctx;
use crate::screens::battle::forecast;
use crate::screens::battle::forecast::tests::targeting;
use crate::screens::battle::testing::skirmish;
use crate::screens::battle::view::{ForecastHp, TotalView};
use crate::words::Words;

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
    assert_eq!(text(&buf, LEFT_X, NAME_ROW, 9), "Test Lord");
    assert_eq!(text(&buf, RIGHT_X, NAME_ROW, 7), "Brigand");
    assert_eq!(text(&buf, LEFT_X, NAME_ROW + 1, 10), "Iron Sword");
    assert_eq!(text(&buf, RIGHT_X, NAME_ROW + 1, 8), "Iron Axe");
    assert_eq!(text(&buf, LEFT_X, BROKEN_ROW, 8), "20/20   ");
    assert_eq!(text(&buf, LEFT_X, HP_ROW + 1, 3), "Hit");
    assert_eq!(text(&buf, LEFT_X, HP_ROW + 2, 4), "Crit");
    assert_eq!(text(&buf, LEFT_X + 9, STRIKE_ROW, 1), "→");
    assert_eq!(text(&buf, NUMBER_X, STRIKE_ROW, 1), "1");
    assert_eq!(text(&buf, RIGHT_X, STRIKE_ROW + 1, 1), "←");
    assert_eq!(text(&buf, NUMBER_X, STRIKE_ROW + 1, 1), "2");
}

#[test]
fn a_side_reads_hp_hit_crit_and_what_is_left() {
    let buf = render(&view(20, |v| {
        let side = &mut v.sides[0];
        side.hp = ForecastHp {
            now: 9,
            after: 9,
            max: 20,
        };
        (side.hit, side.crit) = (Some(85), Some(3));
        side.left = Some(LeftView::Count { left: 6, max: 10 });
        v.sides[1].left = Some(LeftView::Broken("(broken)".into()));
        (v.sides[1].hit, v.sides[1].crit) = (None, None);
    }));
    assert_eq!(text(&buf, LEFT_X, HP_ROW, 5), "HP  9");
    assert_eq!(text(&buf, LEFT_X + 5, HP_ROW + 1, 3), "85 ");
    assert_eq!(text(&buf, LEFT_X + 5, HP_ROW + 2, 3), "3  ");
    assert_eq!(text(&buf, LEFT_X, BROKEN_ROW, 8), "6/10    ");
    // Two rows under the name, right under the weapon.
    assert_eq!(text(&buf, RIGHT_X, NAME_ROW + 2, 8), "(broken)");
    let p = &ctx().palette;
    let fg = buf.get(RIGHT_X, BROKEN_ROW).unwrap().fg;
    assert_eq!(fg, p.get(UiColor::HpLow));
    assert_eq!(text(&buf, RIGHT_X + 5, HP_ROW + 1, 3), "-- ");
    assert_eq!(text(&buf, RIGHT_X + 5, HP_ROW + 2, 3), "-- ");
    // Nothing left to tell: the row is blank.
    let buf = render(&view(20, |v| v.sides[0].left = None));
    assert_eq!(text(&buf, LEFT_X, BROKEN_ROW, 8), "        ");
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
    // The lord keeps all its HP (5 cells); the brigand's 3 of 20 HP
    // round to 1 cell, all of it lost.
    let bar = |x| text(&buf, x + 6, HP_ROW, HP_BAR_CELLS);
    assert_eq!(bar(LEFT_X), "█████");
    assert_eq!(bar(RIGHT_X), "▒░░░░");
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
fn the_skill_line_and_the_affinity_markers() {
    let c = ctx();
    let buf = render(&view(20, |v| {
        v.skill = Some(SkillLineView {
            name: "Guard Break".into(),
            durability: Some((20, 16)),
        });
        v.sides[0].affinity = Some("(resist)".into());
        v.sides[1].affinity = Some("heals 9".into());
    }));
    assert_eq!(row(&buf, SKILL_ROW), "Guard Break (20 → 16)");
    assert_eq!(text(&buf, LEFT_X, AFFINITY_ROW, 8), "(resist)");
    assert_eq!(text(&buf, RIGHT_X, AFFINITY_ROW, 7), "heals 9");
    let fg = buf.get(LEFT_X, AFFINITY_ROW).unwrap().fg;
    assert_eq!(fg, c.palette.get(UiColor::TextHighlight));
    // Without a durability cost: just the name.
    let buf = render(&view(20, |v| {
        v.skill = Some(SkillLineView {
            name: "Keen Edge".into(),
            durability: None,
        });
    }));
    assert_eq!(row(&buf, SKILL_ROW), "Keen Edge");
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
