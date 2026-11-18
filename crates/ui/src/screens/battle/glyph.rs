//! The glyph look of the battle screen (ADR-0018, `look-and-feel.md`):
//! the map viewport on the left (painted by the map skin, ADR-0038), the
//! side panel in a box on the right (double-bordered while a unit is
//! selected), the message and help rows at the bottom, menus beside their
//! tile, and boxes and banners centred on the map. The forecast, the info
//! screen, the fight box, the EXP and level-up pages, and the rewind and
//! Guides screens over the battle have a module each. Everything about the look lives here; nothing here
//! decides what the screen shows. Another look is another `paint` of the
//! same view (ADR-0054).

pub mod fight_box;
pub mod forecast;
pub mod guides;
pub mod info;
pub mod progress;
pub mod rewind;
pub mod shop;

use crate::prelude::*;
use shuyi_ui::text::StyledText;
use trpg_core::Phase;

use super::banner::BannerKind;
use super::layout::{HELP_BAR, HELP_ROW, MAP_VIEW, SIDE_PANEL};
use super::units::{faction_color, paint_hp_bar};
use super::view::{
    BannerView, BattleMenuView, BattleView, DialogView, MenuHeader, MenuPlace, MessageKind,
    NotesView, ObjectiveLine, PanelCellView, PanelUnitView, PanelView,
};
use super::{PopupKind, objective, stage};
use crate::color::{Rgb, UiColor};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, PanelLook, Rect};
use crate::map_view::MapScene;
use crate::screen::Ctx;
use crate::screens::paint_debug_hint;
use crate::screens::playdate::wrap as wrap_words;
use crate::tips as tip_box;
use crate::tips::{LessonView, LessonsEndView};
use crate::widgets::menu::{self, MenuView};
use trpg_core::Outcome;

/// Length of the side panel's HP bar, in cells.
pub const HP_BAR_CELLS: i32 = 10;

/// The side panel's first text column: inside the border, one blank
/// column in.
pub const TEXT_X: i32 = SIDE_PANEL.x + 2;

/// The side panel's widest line of text, so nothing reaches the right
/// border.
pub const TEXT_W: usize = 26;

/// Row of the terrain name in the side panel; its bonuses follow.
pub const TERRAIN_ROW: i32 = SIDE_PANEL.y + 1;

/// Row of the unit's name in the side panel; the rest of the unit block
/// follows.
pub const UNIT_ROW: i32 = SIDE_PANEL.y + 5;

/// Side panel row of the party's gold: the last inside the border.
pub const GOLD_ROW: i32 = SIDE_PANEL.y + SIDE_PANEL.h - 2;

/// Column of the side panel's HP bar, after `HP 999/999 `.
pub const HP_BAR_X: i32 = TEXT_X + 11;

/// Row of a unit's first stats in the side panel, a blank row under its
/// block (0484).
pub const STATS_ROW: i32 = UNIT_ROW + 5;

/// Width of one name and number in the side panel (`Hit 104`).
pub const PANEL_CELL_W: usize = 7;

/// Columns from the first name and number of a row to the second.
pub const PANEL_CELL_GAP: i32 = 12;

/// Width of the weapon line: its uses end there (`Iron Sword        20/20`).
pub const PANEL_WEAPON_W: usize = 23;

/// Banner box width, in cells.
pub const BANNER_W: i32 = 24;

/// Banner box height: border, blank, title, second line, blank, border.
pub const BANNER_H: i32 = 6;

/// Paints `view`: the map, then what is over it, then the bars.
pub fn paint(ctx: &Ctx, view: &BattleView, buf: &mut GlyphBuffer) {
    clear(ctx, buf);
    ctx.game.map_skin.paint(ctx, &view.scene, MAP_VIEW, buf);
    if let Some(m) = &view.menu
        && let MenuPlace::Beside(tile) = m.place
    {
        paint_menu_beside(ctx, view, m, tile, buf);
    }
    paint_popups(ctx, view, buf);
    if let Some(b) = &view.fight_box {
        // Combat animations are off: the plain box.
        fight_box::paint(ctx, b, buf);
    }
    if let Some(p) = &view.progress {
        progress::paint(ctx, p, buf);
    }
    if let Some(f) = &view.forecast {
        forecast::paint(ctx, f, buf);
    }
    if let Some(panel) = &view.panel {
        paint_panel(ctx, panel, buf);
    }
    if let Some(stage) = &view.stage {
        // Over the map and the side panel, which the plates replace.
        stage::glyph::paint(ctx, stage, buf);
    }
    paint_dialogs(ctx, view, buf);
    if let Some(lesson) = &view.lesson {
        paint_lesson(ctx, &view.scene, lesson, buf);
    }
    if let Some(end) = &view.lessons_end {
        paint_lessons_end(ctx, &view.scene, end, buf);
    }
    if let Some(tip) = &view.tip {
        tip_box::glyph::paint(ctx, tip, buf);
    }
    paint_bar(ctx, view, buf);
}

/// Whether `lesson`'s box goes against the bottom edge of the map view:
/// while the tile it must not hide is in the map view's top half, as the
/// map skin draws `scene` (0456). Else (or with that tile out of view)
/// against the top edge.
pub fn lesson_at_bottom(ctx: &Ctx, scene: &MapScene, lesson: &LessonView) -> bool {
    clear_at_bottom(ctx, scene, lesson.clear_of)
}

/// Whether a box that must not hide `tile` goes against the bottom edge.
fn clear_at_bottom(ctx: &Ctx, scene: &MapScene, tile: trpg_core::Pos) -> bool {
    let tile = ctx.game.map_skin.tile_cells(scene, MAP_VIEW, tile);
    tile.is_some_and(in_top_half)
}

/// Where `view`'s lesson box, or the box at the end of the lessons, is
/// painted, if one is up: for a test's snapshot of the box alone (1290).
pub fn lesson_box(ctx: &Ctx, view: &BattleView) -> Option<Rect> {
    let widths = crate::widgets::help::frame_widths(ctx);
    let scene = &view.scene;
    if let Some(lesson) = &view.lesson {
        let at_bottom = lesson_at_bottom(ctx, scene, lesson);
        return Some(tip_box::glyph::lesson_rect(lesson, at_bottom, &widths));
    }
    let end = view.lessons_end.as_ref()?;
    let at_bottom = clear_at_bottom(ctx, scene, end.clear_of);
    Some(tip_box::glyph::lessons_end_rect(end, at_bottom, &widths))
}

/// Paints the box at the end of the lessons over the map that shows `scene`,
/// clear of the cursor's tile.
pub fn paint_lessons_end(ctx: &Ctx, scene: &MapScene, end: &LessonsEndView, buf: &mut GlyphBuffer) {
    let at_bottom = clear_at_bottom(ctx, scene, end.clear_of);
    tip_box::glyph::paint_lessons_end(ctx, end, at_bottom, buf);
}

/// Paints `lesson`'s box over the map that shows `scene`, clear of the
/// cursor's tile.
pub fn paint_lesson(ctx: &Ctx, scene: &MapScene, lesson: &LessonView, buf: &mut GlyphBuffer) {
    let at_bottom = lesson_at_bottom(ctx, scene, lesson);
    tip_box::glyph::paint_lesson(ctx, lesson, at_bottom, buf);
}

/// Where `view`'s lesson box is painted, if it has one.
pub fn lesson_rect(ctx: &Ctx, view: &BattleView) -> Option<Rect> {
    let lesson = view.lesson.as_ref()?;
    let at_bottom = lesson_at_bottom(ctx, &view.scene, lesson);
    let widths = crate::widgets::help::frame_widths(ctx);
    Some(tip_box::glyph::lesson_rect(lesson, at_bottom, &widths))
}

/// Whether the cells `tile` are in the map view's top half: their middle
/// row is.
pub fn in_top_half(tile: Rect) -> bool {
    tile.y + tile.h / 2 < MAP_VIEW.y + MAP_VIEW.h / 2
}

/// Paints the help line `help`, its keys as key caps, and the debug hint
/// `hint`, if any, at the right of the help row.
fn paint_help(ctx: &Ctx, help: &StyledText, hint: Option<&String>, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let colors = ctx
        .palette
        .text_colors_on(c(UiColor::TextDim), c(UiColor::Black));
    buf.print_styled(1, HELP_ROW, help, colors);
    if let Some(hint) = hint {
        paint_debug_hint(&ctx.palette, hint, buf, HELP_ROW);
    }
}

/// Clears the whole of `buf` to the blank black cell, for a screen that
/// paints all of it.
fn clear(ctx: &Ctx, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    buf.fill_rect(
        buf.bounds(),
        Cell::new(' ', c(UiColor::Text), c(UiColor::Black)),
    );
}

/// Where a menu of `(w, h)` cells goes beside the tile covering the cells
/// `tile`: one cell right of the tile (clear of the cursor's marks), or
/// left of it if it would run past the map view; its first item level with
/// the tile's top row, moved to fit the view. A menu too wide for either
/// side goes below the tile's rows (or above them), never over the tile;
/// only a menu that fits nowhere keeps the clamped place beside it
/// (ticket 0454).
pub fn menu_origin(tile: Rect, size: (i32, i32)) -> (i32, i32) {
    menu_origin_in(MAP_VIEW, tile, size)
}

/// [`menu_origin`] in the map area `area` (the Playdate's whole screen,
/// ticket 1038).
pub fn menu_origin_in(area: Rect, tile: Rect, (w, h): (i32, i32)) -> (i32, i32) {
    let right = tile.x + tile.w + 1;
    let (area_right, area_bottom) = (area.x + area.w, area.y + area.h);
    let my = (tile.y - 1).min(area_bottom - h).max(area.y);
    if right + w <= area_right {
        return (right, my);
    }
    let left = tile.x - 1 - w;
    if left >= area.x {
        return (left, my);
    }
    let x = tile.x.min(area_right - w).max(area.x);
    let below = tile.y + tile.h;
    if below + h <= area_bottom {
        return (x, below);
    }
    if tile.y - h >= area.y {
        return (x, tile.y - h);
    }
    (left.max(area.x), my)
}

/// The smallest rectangle holding `tile` and every rectangle of `clear`:
/// what a menu that keeps clear of an area skill's reach goes beside
/// (ticket 0492), with [`menu_origin_in`]'s rule, so the list sits just
/// past the reach's edge.
pub fn beside_all(tile: Rect, clear: impl IntoIterator<Item = Rect>) -> Rect {
    clear.into_iter().fold(tile, |a, r| {
        let (x, y) = (a.x.min(r.x), a.y.min(r.y));
        let right = (a.x + a.w).max(r.x + r.w);
        let bottom = (a.y + a.h).max(r.y + r.h);
        Rect::new(x, y, right - x, bottom - y)
    })
}

/// The cells beside which the map skin paints `m` in `area`: its tile's,
/// grown to hold the tiles it keeps clear of that the camera shows.
pub fn menu_anchor(
    tile_cells: impl Fn(trpg_core::Pos) -> Option<Rect>,
    m: &BattleMenuView,
    tile: trpg_core::Pos,
) -> Option<Rect> {
    let cells = tile_cells(tile)?;
    Some(beside_all(
        cells,
        m.clear_of.iter().filter_map(|&p| tile_cells(p)),
    ))
}

/// Paints the menu `m` beside where the map skin has `tile` of the scene
/// (and the reach it keeps clear of), with its header in the top border.
fn paint_menu_beside(
    ctx: &Ctx,
    view: &BattleView,
    m: &BattleMenuView,
    tile: trpg_core::Pos,
    buf: &mut GlyphBuffer,
) {
    let skin = &ctx.game.map_skin;
    let Some(cells) = menu_anchor(|p| skin.tile_cells(&view.scene, MAP_VIEW, p), m, tile) else {
        return;
    };
    let (x, y) = menu_origin(cells, menu::glyph::size(ctx, &m.menu));
    menu::glyph::paint(ctx, &m.menu, buf, x, y);
    if let Some(header) = &m.header {
        paint_header(ctx, header, buf, (x, y));
    }
}

/// Prints `header` in the top border of a menu at `(x, y)`: its name lit,
/// then the uses left, in the warning colour once none are.
fn paint_header(ctx: &Ctx, header: &MenuHeader, buf: &mut GlyphBuffer, (x, y): (i32, i32)) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let at = x + 2;
    // check-text: not a data name (the view's own)
    let name = format!(" {} ", header.name);
    let n = buf.print(at, y, &name, c(UiColor::TextHighlight), bg);
    if let Some(uses) = &header.uses {
        let color = if uses.spent {
            UiColor::HpLow
        } else {
            UiColor::Text
        };
        let text = format!("{} ", uses.text);
        buf.print(at + i32::from(n), y, &text, c(color), bg);
    }
}

/// The colour of a heal or burn number.
pub const fn popup_color(kind: PopupKind) -> UiColor {
    match kind {
        PopupKind::Heal => UiColor::HpHigh,
        PopupKind::Burn => UiColor::HpLow,
    }
}

/// Paints the heal and burn numbers over the units they tell of: over
/// where the map skin has their tiles, a row up.
fn paint_popups(ctx: &Ctx, view: &BattleView, buf: &mut GlyphBuffer) {
    for p in &view.popups {
        let fg = ctx.palette.get(popup_color(p.kind));
        let Some(tile) = ctx.game.map_skin.tile_cells(&view.scene, MAP_VIEW, p.pos) else {
            continue;
        };
        let y = (tile.y - 1).max(MAP_VIEW.y);
        let bg = ctx.palette.get(UiColor::Black);
        buf.print(tile.x, y, &p.text(), fg, bg);
    }
}

/// Prints `text` in `fg` from `(x, y)` in at most `width` cells, keeping
/// the cells' background. The glyph skins cut a long line hard, with no
/// `…` (the panel's and the forecast's tests), which is why this is not
/// the engine's `print_cut`.
pub fn print_clipped(buf: &mut GlyphBuffer, (x, y): (i32, i32), width: usize, text: &str, fg: Rgb) {
    let shown: String = text.chars().take(width).collect();
    buf.print_fg(x, y, &shown, fg);
}

/// Paints the side panel: its box, the terrain, then the unit.
fn paint_panel(ctx: &Ctx, panel: &PanelView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    // Double-line while a unit is selected (`look-and-feel.md`).
    let style = if panel.selected {
        BoxStyle::Double
    } else {
        BoxStyle::Single
    };
    let look = PanelLook::new(style, c(UiColor::Text), c(UiColor::PanelBg));
    buf.panel(SIDE_PANEL, &look.with_border(c(UiColor::PanelBorder)), None);
    if let Some(t) = &panel.terrain {
        // check-text: not a data name (the view's own)
        print_clipped(
            buf,
            (TEXT_X, TERRAIN_ROW),
            TEXT_W,
            &t.name,
            c(UiColor::TextHighlight),
        );
        print_clipped(
            buf,
            (TEXT_X, TERRAIN_ROW + 1),
            TEXT_W,
            &t.bonuses,
            c(UiColor::Text),
        );
        if let Some(heals) = &t.heals {
            print_clipped(
                buf,
                (TEXT_X, TERRAIN_ROW + 2),
                TEXT_W,
                heals,
                c(UiColor::Text),
            );
        }
    }
    if let Some(unit) = &panel.unit {
        paint_panel_unit(ctx, unit, buf);
    }
    print_clipped(
        buf,
        (TEXT_X, GOLD_ROW),
        TEXT_W,
        &panel.gold,
        c(UiColor::Text),
    );
}

/// The unit block: name (faction colour), class and level, HP with a bar,
/// faction.
fn paint_panel_unit(ctx: &Ctx, unit: &PanelUnitView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let y = UNIT_ROW;
    // check-text: not a data name (the view's own)
    print_clipped(
        buf,
        (TEXT_X, y),
        TEXT_W,
        &unit.name,
        c(faction_color(unit.faction)),
    );
    print_clipped(
        buf,
        (TEXT_X, y + 1),
        TEXT_W,
        &unit.class_level,
        c(UiColor::Text),
    );
    let hp = format!("{} {}/{}", unit.hp_name, unit.hp, unit.max);
    print_clipped(buf, (TEXT_X, y + 2), TEXT_W, &hp, c(UiColor::Text));
    let bar = Rect::new(HP_BAR_X, y + 2, HP_BAR_CELLS, 1);
    let bg = c(UiColor::PanelBg);
    paint_hp_bar(buf, &ctx.palette, bar, (unit.hp, unit.max), bg);
    print_clipped(
        buf,
        (TEXT_X, y + 3),
        TEXT_W,
        &unit.faction_name,
        c(UiColor::TextDim),
    );
    // A blank row, the stats, a blank row, the weapon, the combat numbers.
    let below = paint_panel_cells(ctx, &unit.stats, buf, STATS_ROW);
    let weapon_row = below + 1;
    match &unit.weapon {
        Some(weapon) => paint_panel_weapon(ctx, weapon, buf, weapon_row),
        None => print_clipped(
            buf,
            (TEXT_X, weapon_row),
            TEXT_W,
            &unit.no_weapon,
            c(UiColor::Text),
        ),
    }
    paint_panel_cells(ctx, &unit.combat, buf, weapon_row + 1);
}

/// Paints `cells` two to a row from row `top`, the names dim and the
/// numbers lit (a timed effect's in its colour) and right-aligned; the row
/// after the last.
fn paint_panel_cells(ctx: &Ctx, cells: &[PanelCellView], buf: &mut GlyphBuffer, top: i32) -> i32 {
    let c = |u| ctx.palette.get(u);
    let mut y = top;
    for row in cells.chunks(2) {
        for (x, cell) in [TEXT_X, TEXT_X + PANEL_CELL_GAP].into_iter().zip(row) {
            // check-text: not a data name (the view's own)
            let name = &cell.name;
            print_clipped(buf, (x, y), PANEL_CELL_W, name, c(UiColor::TextDim));
            let name_w = name.chars().count();
            let value = format!(
                "{:>w$}",
                cell.value,
                w = PANEL_CELL_W.saturating_sub(name_w)
            );
            let color = if cell.changed {
                UiColor::Effect
            } else {
                UiColor::Text
            };
            let at = x + i32::try_from(name_w).unwrap_or(0);
            print_clipped(buf, (at, y), PANEL_CELL_W, &value, c(color));
        }
        y += 1;
    }
    y
}

/// Paints the weapon line: its name, then its uses right-aligned (in the
/// warning colour once none are left).
fn paint_panel_weapon(ctx: &Ctx, weapon: &MenuHeader, buf: &mut GlyphBuffer, y: i32) {
    let c = |u| ctx.palette.get(u);
    let uses = weapon.uses.as_ref();
    let uses_w = uses.map_or(0, |u| u.text.chars().count());
    let name_w = PANEL_WEAPON_W.saturating_sub(uses_w + 1).min(TEXT_W);
    // check-text: not a data name (the view's own)
    print_clipped(buf, (TEXT_X, y), name_w, &weapon.name, c(UiColor::Text));
    if let Some(uses) = uses {
        let color = if uses.spent {
            UiColor::HpLow
        } else {
            UiColor::Text
        };
        let at = TEXT_X + i32::try_from(PANEL_WEAPON_W - uses_w).unwrap_or(0);
        print_clipped(buf, (at, y), uses_w, &uses.text, c(color));
    }
}

/// Paints the boxes over the map: the unit list, the objective page (the
/// notes under it), a question or the info screen, then the battle notes
/// box and the banner.
fn paint_dialogs(ctx: &Ctx, view: &BattleView, buf: &mut GlyphBuffer) {
    if let Some(m) = &view.menu
        && m.place == MenuPlace::Centred
    {
        paint_centred_menu(ctx, &m.menu, buf);
    }
    match &view.dialog {
        Some(DialogView::Objective(o)) => {
            let mut lines: Vec<_> = objective::padded(&o.lines)
                .into_iter()
                .chain(o.village.clone())
                .chain([o.turn.clone()])
                .map(|l| (l, UiColor::Text))
                .collect();
            if let Some(notes) = &o.notes {
                lines.push((String::new(), UiColor::Text));
                lines.push((notes.title.clone(), UiColor::TextDim));
                lines.extend(notes.lines.iter().map(|l| (l.clone(), UiColor::Text)));
            }
            let (_, h) = dialog_size(&o.title, &lines);
            let top = notes_top(ctx, view, h);
            draw_dialog_at(ctx, buf, &o.title, &lines, top);
        }
        Some(DialogView::Question(q)) => {
            let lines = [
                (q.question.clone(), UiColor::Text),
                (q.answers.clone(), UiColor::TextDim),
            ];
            draw_dialog(ctx, buf, "", &lines);
        }
        None => {}
    }
    if let Some(i) = &view.info {
        info::paint(ctx, i, buf);
    }
    if let Some(notes) = &view.notes {
        paint_notes(ctx, view, notes, buf);
    }
    if let Some(b) = &view.banner {
        paint_banner(ctx, b, buf);
    }
}

/// Paints the battle notes box, off the rows of the units they are about.
fn paint_notes(ctx: &Ctx, view: &BattleView, notes: &NotesView, buf: &mut GlyphBuffer) {
    let lines: Vec<_> = notes
        .lines
        .iter()
        .map(|l| (l.clone(), UiColor::Text))
        .collect();
    let (_, h) = dialog_size(&notes.title, &lines);
    let top = notes_top(ctx, view, h);
    draw_dialog_at(ctx, buf, &notes.title, &lines, top);
}

/// The top row of a box `h` rows tall listing the battle notes: off the
/// rows of the units they are about ([`box_top`]), where the map skin has
/// their tiles.
fn notes_top(ctx: &Ctx, view: &BattleView, h: i32) -> i32 {
    let rows: Vec<i32> = view
        .noted
        .iter()
        .filter_map(|&pos| ctx.game.map_skin.tile_cells(&view.scene, MAP_VIEW, pos))
        .map(|cells| cells.y)
        .collect();
    box_top(h, &rows)
}

/// The top row of a box `h` rows tall centred on the map view.
pub const fn centred_top(h: i32) -> i32 {
    MAP_VIEW.y + (MAP_VIEW.h - h) / 2
}

/// The top row of a box `h` rows tall that lists the notes: centred on the
/// map view, unless it would then cover one of `rows` (the screen rows of
/// the units the notes are about); then one row from the top of the view,
/// or one row from its bottom, whichever covers none of them. Centred if
/// every place covers one.
pub fn box_top(h: i32, rows: &[i32]) -> i32 {
    box_top_in(MAP_VIEW, h, rows)
}

/// [`box_top`] in the map area `area` (the Playdate's whole screen,
/// ticket 1038).
pub fn box_top_in(area: Rect, h: i32, rows: &[i32]) -> i32 {
    let centred = area.y + (area.h - h) / 2;
    let top = area.y + 1;
    let bottom = area.y + area.h - h - 1;
    let clear = |y: i32| !rows.iter().any(|&r| r >= y && r < y + h);
    [centred, top, bottom]
        .into_iter()
        .find(|&y| clear(y))
        .unwrap_or(centred)
}

/// The size (width, height) of the [`draw_dialog`] box of `lines` under
/// `title`, in cells.
fn dialog_size(title: &str, lines: &[(String, UiColor)]) -> (i32, i32) {
    let widest = lines
        .iter()
        .map(|(l, _)| l.chars().count())
        .chain(core::iter::once(title.chars().count() + 2))
        .max()
        .unwrap_or(0);
    let w = i32::try_from(widest).unwrap_or(0) + 4;
    let h = i32::try_from(lines.len()).unwrap_or(0) + 4;
    (w, h)
}

/// Draws a box of `lines` (text and colour) centred on the map view, one
/// blank column and row inside a double border; `title` over the top
/// border.
fn draw_dialog(ctx: &Ctx, buf: &mut GlyphBuffer, title: &str, lines: &[(String, UiColor)]) {
    let (_, h) = dialog_size(title, lines);
    draw_dialog_at(ctx, buf, title, lines, centred_top(h));
}

/// [`draw_dialog`], with the box's top border on row `top` instead of
/// centred top to bottom.
fn draw_dialog_at(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    title: &str,
    lines: &[(String, UiColor)],
    top: i32,
) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let (w, h) = dialog_size(title, lines);
    let rect = Rect::new(MAP_VIEW.x + (MAP_VIEW.w - w) / 2, top, w, h);
    let look = PanelLook::new(BoxStyle::Double, c(UiColor::Text), bg)
        .with_border(c(UiColor::PanelBorder))
        .with_title(c(UiColor::TextHighlight));
    buf.panel(rect, &look, Some(title).filter(|t| !t.is_empty()));
    for (y, (line, color)) in (rect.y + 2..).zip(lines) {
        buf.print(rect.x + 2, y, line, c(*color), bg);
    }
}

/// Paints `menu` centred on the map view.
fn paint_centred_menu(ctx: &Ctx, menu: &MenuView, buf: &mut GlyphBuffer) {
    let (w, h) = menu::glyph::size(ctx, menu);
    let x = MAP_VIEW.x + (MAP_VIEW.w - w) / 2;
    let y = MAP_VIEW.y + (MAP_VIEW.h - h) / 2;
    menu::glyph::paint(ctx, menu, buf, x, y);
}

/// The colour of a banner announcing `kind`: the phase's faction's, or
/// lit for a victory and the warning colour for a defeat.
pub const fn banner_color(kind: BannerKind) -> UiColor {
    match kind {
        BannerKind::Phase { phase, .. } => match phase {
            Phase::Player => UiColor::Player,
            Phase::Enemy => UiColor::Enemy,
            Phase::Other => UiColor::Ally,
        },
        BannerKind::Outcome(Outcome::Victory) => UiColor::TextHighlight,
        BannerKind::Outcome(Outcome::Defeat) => UiColor::HpLow,
        BannerKind::Objective => UiColor::Player,
    }
}

/// Paints the banner centred on the map view, in a double box of its
/// colour.
fn paint_banner(ctx: &Ctx, banner: &BannerView, buf: &mut GlyphBuffer) {
    if banner.kind == BannerKind::Objective {
        paint_objective_banner(ctx, banner, buf);
        return;
    }
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let rect = Rect::new(
        MAP_VIEW.x + (MAP_VIEW.w - BANNER_W) / 2,
        MAP_VIEW.y + (MAP_VIEW.h - BANNER_H) / 2,
        BANNER_W,
        BANNER_H,
    );
    let color = banner_color(banner.kind);
    let look = PanelLook::new(BoxStyle::Double, c(UiColor::Text), bg);
    buf.panel(rect, &look.with_border(c(color)), None);
    let row = |dy| Rect::new(rect.x, rect.y + dy, rect.w, 1);
    buf.print_centred(row(2), &banner.title, c(color));
    buf.print_centred(row(3), &banner.subtitle, c(UiColor::Text));
}

/// The objective banner's size for `lines` under `title`: the lines,
/// their labels in a column ([`objective::label_width`]), a border and two
/// blank columns each side; the title, a blank row, the lines and a blank
/// row between borders.
pub fn objective_banner_size(title: &str, lines: &[ObjectiveLine]) -> (i32, i32) {
    let cells = |s: &str| i32::try_from(s.chars().count()).unwrap_or(0);
    let w = objective::padded(lines)
        .iter()
        .map(|l| cells(l))
        .chain([cells(title)])
        .max()
        .unwrap_or(0);
    let rows = i32::try_from(lines.len()).unwrap_or(0);
    (w + 6, rows + 6)
}

/// Paints the objective banner (0481) centred on the map view: the title
/// in the banner's colour, then each line, its label highlighted.
fn paint_objective_banner(ctx: &Ctx, banner: &BannerView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let (w, h) = objective_banner_size(&banner.title, &banner.lines);
    let rect = Rect::new(
        MAP_VIEW.x + (MAP_VIEW.w - w) / 2,
        MAP_VIEW.y + (MAP_VIEW.h - h) / 2,
        w,
        h,
    );
    let color = banner_color(banner.kind);
    let look = PanelLook::new(BoxStyle::Double, c(UiColor::Text), bg);
    buf.panel(rect, &look.with_border(c(color)), None);
    buf.print_centred(
        Rect::new(rect.x, rect.y + 2, rect.w, 1),
        &banner.title,
        c(color),
    );
    let indent = i32::try_from(objective::label_width(&banner.lines)).unwrap_or(0);
    for (y, l) in (rect.y + 4..).zip(&banner.lines) {
        buf.print(rect.x + 3, y, &l.label, c(UiColor::TextHighlight), bg);
        buf.print(rect.x + 3 + indent, y, &l.text, c(UiColor::Text), bg);
    }
}

/// Paints the bar under the map and panel (0445): the toggles' row, the
/// danger-zone hint from where the help starts and auto-end above the
/// help's end turn, or a combat's or a preview's line at its left; then
/// the help row. A note or a warning is painted over the map just above
/// the bar.
fn paint_bar(ctx: &Ctx, view: &BattleView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(HELP_BAR, Cell::new(' ', c(UiColor::Text), black));
    if let Some(t) = &view.toggles {
        let colors = || ctx.palette.text_colors_on(c(UiColor::TextDim), black);
        buf.print_styled(1, HELP_BAR.y, &t.left, colors());
        if let Some((auto_end, column)) = &t.auto_end {
            let x = 1 + i32::try_from(*column).unwrap_or(0);
            buf.print_styled(x, HELP_BAR.y, auto_end, colors());
        }
    }
    match &view.message {
        Some(m) if m.kind == MessageKind::Said => {
            buf.print(1, HELP_BAR.y, &m.text, c(UiColor::Text), black);
        }
        Some(m) if m.kind == MessageKind::Toast => paint_note(ctx, &m.text, buf),
        Some(m) => paint_warning(ctx, &m.text, buf),
        None => {}
    }
    paint_help(ctx, &view.help, view.debug_hint.as_ref(), buf);
}

/// Paints a note on a shaded strip across the map's bottom rows, just
/// above the bar: one row, more if it has to wrap to the map's width.
fn paint_note(ctx: &Ctx, text: &str, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (fg, bg) = (c(UiColor::TextHighlight), c(UiColor::PanelBg));
    let lines = wrap_words(text, MESSAGE_W);
    let rows = i32::try_from(lines.len()).unwrap_or(1);
    let top = HELP_BAR.y - rows;
    buf.fill_rect(
        Rect::new(MAP_VIEW.x, top, MAP_VIEW.w, rows),
        Cell::new(' ', fg, bg),
    );
    for (y, line) in (top..).zip(&lines) {
        buf.print(MAP_VIEW.x + 1, y, line, fg, bg);
    }
}

/// Paints a warning in a box at the map's bottom left, just above the
/// bar, wrapped to fit inside the map.
fn paint_warning(ctx: &Ctx, text: &str, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (fg, bg) = (c(UiColor::TextHighlight), c(UiColor::PanelBg));
    let lines = wrap_words(text, WARNING_W);
    let cells = |s: &String| i32::try_from(s.chars().count()).unwrap_or(0);
    let w = lines.iter().map(cells).max().unwrap_or(0) + 4;
    let h = i32::try_from(lines.len()).unwrap_or(1) + 2;
    let rect = Rect::new(MAP_VIEW.x, HELP_BAR.y - h, w, h);
    let look = PanelLook::new(BoxStyle::Single, fg, bg);
    buf.panel(rect, &look.with_border(fg), None);
    for (y, line) in (rect.y + 1..).zip(&lines) {
        buf.print(rect.x + 2, y, line, fg, bg);
    }
}

/// The widest line of a note: the map's width less a blank column each
/// side.
const MESSAGE_W: usize = MAP_VIEW.w.unsigned_abs() as usize - 2;

/// The widest line of a warning: the map's width less the box's border
/// and a blank column each side.
const WARNING_W: usize = MAP_VIEW.w.unsigned_abs() as usize - 4;

#[cfg(test)]
pub(crate) mod tests;
