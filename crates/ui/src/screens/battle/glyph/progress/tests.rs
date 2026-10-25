//! Where the EXP bar, level-up and class pages are painted. What they say
//! is tested on their view (`battle/progress/tests.rs`).

use super::*;
use crate::screen::tests::ctx;
use crate::screens::battle::view::StatGainView;
use crate::tips::view::FaceView;

fn painted(view: &ProgressView) -> GlyphBuffer {
    let c = ctx();
    let p = &c.palette;
    let blank = Cell::new(' ', p.get(UiColor::Text), p.get(UiColor::Black));
    let mut buf = GlyphBuffer::new(100, 32, blank);
    paint(&c, view, &mut buf);
    buf
}

/// Row `y` from column `x` to the right edge `right`, trimmed.
fn row(buf: &GlyphBuffer, x: i32, right: i32, y: i32) -> String {
    (x..right)
        .map(|x| buf.get(x, y).unwrap().glyph)
        .collect::<String>()
        .trim_end()
        .to_owned()
}

fn gains(face: Option<FaceView>, stats: Vec<StatGainView>) -> ProgressView {
    ProgressView::Gains(GainsView {
        face,
        placeholder: "portrait".into(),
        banner: "PROMOTED!".into(),
        name: "A name longer than twenty cells".into(),
        line: "Exile → Blade Heir".into(),
        stats,
    })
}

fn stat(name: &str, before: i32, gain: i32) -> StatGainView {
    StatGainView {
        name: name.into(),
        before,
        gain,
    }
}

#[test]
fn the_exp_bar_fills_a_cell_per_five_exp() {
    let c = ctx();
    let buf = painted(&ProgressView::Exp(ExpView {
        name: "Test Lord".into(),
        exp_name: "EXP".into(),
        exp: 45,
        max: 100,
    }));
    let y = EXP_BOX.y + 1;
    let x = EXP_BOX.x + 2;
    assert_eq!(row(&buf, x, x + 12, y), "Test Lord");
    assert_eq!(row(&buf, x + 13, x + 16, y), "EXP");
    let bar = row(&buf, x + 17, x + 17 + EXP_BAR_CELLS, y);
    assert_eq!(bar, format!("{}{}", "█".repeat(9), "░".repeat(11)));
    let fill = buf.get(x + 17, y).unwrap().fg;
    assert_eq!(fill, c.palette.get(UiColor::ExpBar));
    let right = EXP_BOX.x + EXP_BOX.w - 1;
    assert_eq!(row(&buf, right - 3, right - 1, y), "45");
    assert_eq!(buf.get(right, y).unwrap().glyph, '║');
    assert_eq!(buf.get(EXP_BOX.x, EXP_BOX.y + 2).unwrap().glyph, '╚');
}

#[test]
fn bars_fill_their_share_of_the_cells() {
    assert_eq!(bar_cells(0, 100), 0);
    assert_eq!(bar_cells(45, 100), 9);
    assert_eq!(bar_cells(99, 100), 19);
    assert_eq!(bar_cells(100, 100), EXP_BAR_CELLS);
    assert_eq!(bar_cells(150, 100), EXP_BAR_CELLS);
    assert_eq!(bar_cells(5, 0), 0);
}

#[test]
fn a_unit_without_a_face_gets_the_placeholder_box() {
    let buf = painted(&gains(None, vec![]));
    // Centred in the frame at (7, 6), 34 × 18.
    let label: String = (20..28).map(|x| buf.get(x, 15).unwrap().glyph).collect();
    assert_eq!(label, "portrait");
    assert_eq!(buf.get(19, 15).unwrap().glyph, ' ');
    // A face whose portrait is missing: the placeholder too.
    let lost = FaceView {
        portrait: "nobody".into(),
        expression: "happy".into(),
    };
    let buf = painted(&gains(Some(lost), vec![]));
    assert_eq!(buf.sprites(), [] as [shuyi_ui::frame::Sprite; 0]);
    let label: String = (20..28).map(|x| buf.get(x, 15).unwrap().glyph).collect();
    assert_eq!(label, "portrait");
}

#[test]
fn a_face_is_drawn_in_the_frame() {
    let c = ctx();
    let art = &c.game.content.portraits["test_lord"];
    let face = FaceView {
        portrait: "test_lord".into(),
        expression: art.expressions[0].name.clone(),
    };
    let buf = painted(&gains(Some(face), vec![]));
    assert!(shows_face_of(&buf, &c, "test_lord"));
    let label: String = (20..28).map(|x| buf.get(x, 15).unwrap().glyph).collect();
    assert_ne!(label, "portrait");
}

#[test]
fn the_level_up_panel_rows_and_lit_gains() {
    let c = ctx();
    let stats = vec![stat("HP", 19, 6), stat("Str", 6, 3), stat("Mag", 1, 0)];
    let buf = painted(&gains(None, stats));
    let right = LEVEL_PANEL.x + LEVEL_PANEL.w - 1;
    let text = |y| row(&buf, LEVEL_TEXT_X, right, y);
    assert_eq!(text(LEVEL_BANNER_ROW), "PROMOTED!");
    assert_eq!(text(LEVEL_BANNER_ROW + 2), "A name longer than t");
    assert_eq!(text(LEVEL_BANNER_ROW + 3), "Exile → Blade Heir");
    assert_eq!(text(STAT_ROW), "HP   19 → 25  +6");
    assert_eq!(text(STAT_ROW + 2), "Mag   1");
    assert_eq!(text(STAT_ROW + 3), "");
    let fg = |x, y| buf.get(x, y).unwrap().fg;
    let p = &c.palette;
    // The gain lit, the rest of its row plain; a stat that stayed dim.
    assert_eq!(
        fg(LEVEL_TEXT_X + 14, STAT_ROW),
        p.get(UiColor::TextHighlight)
    );
    assert_eq!(fg(LEVEL_TEXT_X, STAT_ROW), p.get(UiColor::Text));
    assert_eq!(fg(LEVEL_TEXT_X, STAT_ROW + 2), p.get(UiColor::TextDim));
    assert_eq!(
        fg(LEVEL_TEXT_X, LEVEL_BANNER_ROW),
        p.get(UiColor::TextHighlight)
    );
    assert_eq!(
        fg(LEVEL_TEXT_X, LEVEL_BANNER_ROW + 2),
        p.get(UiColor::Player)
    );
}

#[test]
fn the_class_box_is_as_tall_as_its_lines() {
    let c = ctx();
    let page = |mastered: Option<&str>, lines: &[&str]| {
        ProgressView::Class(ClassProgressView {
            name: "Test Lord".into(),
            line: "Exile  CL 9 → 10".into(),
            mastered: mastered.map(str::to_owned),
            lines: lines.iter().map(|&l| l.to_owned()).collect(),
        })
    };
    let buf = painted(&page(None, &["Learned: Fire", "To stock: Leather Vest"]));
    let text = |buf: &GlyphBuffer, y| row(buf, BOX.x + 2, BOX.x + BOX.w - 1, y);
    assert_eq!(text(&buf, BOX.y + 1), "Test Lord    Exile  CL 9 → 10");
    assert_eq!(text(&buf, BOX.y + 2), "Learned: Fire");
    assert_eq!(text(&buf, BOX.y + 3), "To stock: Leather Vest");
    // Its bottom border is next.
    assert_eq!(buf.get(BOX.x, BOX.y + 4).unwrap().glyph, '╚');
    let buf = painted(&page(Some("CLASS MASTERED!"), &["Learned: Fire"]));
    assert_eq!(text(&buf, BOX.y + 2), "CLASS MASTERED!");
    let fg = buf.get(BOX.x + 2, BOX.y + 2).unwrap().fg;
    assert_eq!(fg, c.palette.get(UiColor::TextHighlight));
    assert_eq!(text(&buf, BOX.y + 3), "Learned: Fire");
    assert_eq!(buf.get(BOX.x, BOX.y + 4).unwrap().glyph, '╚');
}
