//! The chapter card on the Playdate (ticket 0812): the PC's card in the
//! Playdate's inks and its cell font, the two rules, `Chapter 1` and the
//! name between them (its screen is too small for Cinzel's name: the
//! agent's starting rule).

use super::ChapterCardView;
use super::glyph::rule;
use crate::glyph_buffer::GlyphBuffer;
use crate::screen::Ctx;
use crate::screens::playdate::{centred, clear};

/// The rows from the middle one: the rules, the line and the name.
pub const ROWS: [i32; 4] = [-4, -2, 0, 2];
/// Each half of a rule, in cells.
pub const RULE_HALF: usize = 18;

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &ChapterCardView, buf: &mut GlyphBuffer) {
    clear(ctx, buf);
    let mid = i32::from(buf.height()) / 2;
    let [above, line, name, below] = ROWS.map(|dy| mid + dy);
    let rule = rule(RULE_HALF);
    centred(ctx, buf, above, &rule);
    centred(ctx, buf, below, &rule);
    centred(ctx, buf, line, &view.line);
    centred(ctx, buf, name, &view.heading);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::UiColor;
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::glyph_buffer::Cell;
    use crate::screen::tests::ctx;

    #[test]
    fn the_card_in_the_cell_font_between_its_rules() {
        let c = ctx();
        let ground = c.palette.get(UiColor::Black);
        let blank = Cell::new(' ', ground, ground);
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PLAYDATE, blank);
        paint(
            &c,
            &ChapterCardView::of_title("Chapter 4: The Hound's Country"),
            &mut buf,
        );
        let text_at = |y: i32| -> String {
            let cells = (0..i32::from(buf.width())).map(|x| buf.get(x, y).map_or(' ', |c| c.glyph));
            cells.collect::<String>().trim().to_owned()
        };
        // The Playdate's 15 rows, top to bottom: the rule, a blank row, the
        // line, a blank row, the name, a blank row, the rule, around the
        // middle row (7); the rest blank.
        let rule = rule(RULE_HALF);
        let mut want = vec![String::new(); usize::from(buf.height())];
        want[3].clone_from(&rule);
        want[5] = "Chapter 4".to_owned();
        want[7] = "The Hound's Country".to_owned();
        want[9].clone_from(&rule);
        let rows: Vec<String> = (0..i32::from(buf.height())).map(text_at).collect();
        assert_eq!(rows, want);
        assert_eq!(buf.sprites().len(), 0);
        assert!(rule.chars().count() <= usize::from(buf.width()));
    }
}
