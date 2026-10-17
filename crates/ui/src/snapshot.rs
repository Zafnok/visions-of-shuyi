//! The text form of a frame for snapshot tests (ADR-0007) is Shuyi
//! Engine's ([`shuyi_ui::frame::snapshot`], ticket 1108):
//! [`GlyphBuffer::to_snapshot`](crate::GlyphBuffer::to_snapshot). The
//! tests here hold it to this game's palette, whose names it prints.

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use crate::color::Rgb;
    use crate::color::UiColor;
    use crate::color::tests::game_palette;
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};

    #[test]
    fn sample_box() {
        let p = game_palette();
        let c = |u| p.get(u);
        let mut b = GlyphBuffer::new(
            ConsoleSpec::PC.cols / 4,
            ConsoleSpec::PC.rows / 4,
            Cell::new(' ', c(UiColor::Text), c(UiColor::Black)),
        );
        let panel = Rect::new(1, 1, 22, 6);
        b.fill_rect(panel, Cell::new(' ', c(UiColor::Text), c(UiColor::PanelBg)));
        b.draw_box(
            panel,
            BoxStyle::Double,
            c(UiColor::PanelBorderFocus),
            c(UiColor::PanelBg),
        );
        b.print(
            3,
            1,
            " Battle ",
            c(UiColor::TextHighlight),
            c(UiColor::PanelBg),
        );
        b.print(3, 3, "Aldo", c(UiColor::Player), c(UiColor::PanelBg));
        b.print_fg(8, 3, "vs", c(UiColor::TextDim));
        b.print(11, 3, "Brigand", c(UiColor::Enemy), c(UiColor::PanelBg));
        b.print(3, 4, "HP", c(UiColor::Text), c(UiColor::PanelBg));
        b.print(6, 4, "■■■■", c(UiColor::HpHigh), c(UiColor::PanelBg));
        b.blend_bg(Rect::new(3, 5, 4, 1), c(UiColor::MoveRange), 1.0);
        b.print_fg(3, 5, "move", c(UiColor::Text));
        b.blend_bg(Rect::new(20, 5, 2, 1), Rgb::new(255, 255, 255), 0.5);
        let snap = b.to_snapshot(&p);
        assert_eq!(snap, b.to_snapshot(&p), "deterministic");
        assert_snapshot!(snap);
    }
}
