//! The first-launch "Pick your layout" screen (`docs/design/controls.md`):
//! Shuyi Engine's (`shuyi_ui::screens::layout_picker`, ticket 1112) with
//! this game's legend ([`LEGEND`]), re-exported here under its old names.
//! The Options screen opens it again to switch layout
//! ([`LayoutPickerScreen::change`]). Its look is the engine's glyph skin
//! ([`glyph::paint`], ADR-0054).

pub use shuyi_ui::screens::layout_picker::*;

use crate::input::Action;
use crate::screen::{Ctx, Shuyi};

/// The layout picker for this game
/// ([`shuyi_ui::screens::layout_picker::LayoutPickerScreen`]).
pub type LayoutPickerScreen = shuyi_ui::screens::layout_picker::LayoutPickerScreen<Shuyi>;

/// The legend under each layout's keyboard: the keys of moving, choosing
/// and backing out, then the map's keys a new player meets first.
pub const LEGEND: [(LegendKeys<Action>, &str); 10] = [
    (LegendKeys::Cursor, "layout_picker.legend.move"),
    (
        LegendKeys::All(Action::Confirm),
        "layout_picker.legend.select",
    ),
    (LegendKeys::All(Action::Cancel), "layout_picker.legend.back"),
    (
        LegendKeys::First(Action::PrevUnit),
        "layout_picker.legend.prev_unit",
    ),
    (
        LegendKeys::First(Action::NextUnit),
        "layout_picker.legend.next_unit",
    ),
    (
        LegendKeys::First(Action::Info),
        "layout_picker.legend.unit_info",
    ),
    (
        LegendKeys::First(Action::DangerZone),
        "layout_picker.legend.danger_zone",
    ),
    (
        LegendKeys::First(Action::EndTurn),
        "layout_picker.legend.end_turn",
    ),
    (
        LegendKeys::First(Action::ToggleAutoEnd),
        "layout_picker.legend.auto_end",
    ),
    (
        LegendKeys::First(Action::Rewind),
        "layout_picker.legend.rewind",
    ),
];

/// The picker asked for on the first key press, with this game's legend
/// ([`LayoutPickerScreen::new`]).
pub fn first(ctx: &Ctx) -> LayoutPickerScreen {
    LayoutPickerScreen::new(ctx, &LEGEND)
}

/// The picker as Options opens it, to switch layout
/// ([`LayoutPickerScreen::change`]).
pub fn change(ctx: &Ctx) -> LayoutPickerScreen {
    LayoutPickerScreen::change(ctx, &LEGEND)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{LEFT_HANDED, RIGHT_HANDED};
    use crate::screen::ShuyiCtx;

    /// This game's legend, read from each of its layouts' keys.
    #[test]
    fn the_legend_comes_from_each_layout() {
        let c = Ctx::embedded().unwrap();
        let view = first(&c).view(&c);
        let legend = |i: usize| {
            view.layouts[i]
                .legend
                .iter()
                .map(|row| format!("{} {}", row.keys, row.what))
                .collect::<Vec<_>>()
        };
        assert_eq!(view.layouts[0].layout, RIGHT_HANDED);
        assert_eq!(
            legend(0),
            [
                "arrows move",
                "f select",
                "d/Escape back",
                "a prev unit",
                "s next unit",
                "e unit info",
                "w danger zone",
                "Space end turn",
                "Shift+Space auto-end",
                "r rewind",
            ]
        );
        assert_eq!(view.layouts[1].layout, LEFT_HANDED);
        assert_eq!(legend(1)[..3], ["wasd move", "j select", "k/Escape back"]);
        assert_eq!(view.layouts[0].name, "Right-handed");
        assert_eq!(label(&c, LEFT_HANDED), "Left-handed");
        assert!(change(&c).view(&c).help.contains("back"));
    }
}
