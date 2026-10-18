//! Debug screens (F2 in debug builds): Shuyi Engine's debug menu
//! (`shuyi_ui::debug`, ticket 1112) with this game's tools ([`menu`]). The
//! glyph sampler shows every font glyph and palette colour, for judging the
//! look (ticket 0011), with this game's sample panels; the portrait viewer
//! shows every portrait (ticket 0703); the test scene plays
//! `assets-test/dialogue/test.dlg` full-screen or over the screen the menu
//! was opened from (ticket 0704). The sprite test draws the test card as
//! sprite items (ticket 0231). The class-choice screen opens on a test
//! unit, to promote or reclass it (ticket 0603), until the between-battle
//! menus exist. "Map skin" switches how battle maps look between the glyph
//! skin and the test tileset (ticket 0433); the choice isn't saved. The
//! scene camera pans and zooms the test map as a backdrop behind a window
//! (ticket 0228, ADR-0048). "Voice test" says the test scene's lines that
//! have a voice clip, one per press (ticket 0238), until the dialogue
//! screen plays voices (0720). "Play test cinematic" plays
//! `assets-test/cinematics/test.ron` in a loop (ticket 0817, ADR-0060).
//! The scene preview isn't in the menu: `app` opens the game on it for
//! `--scene` (ticket 0723).

mod cinematic;
mod portrait_viewer;
mod scene_camera;
mod scene_preview;
mod sprite_test;

use crate::prelude::*;
pub use cinematic::{CinematicScreen, TEST_CINEMATIC};
pub use portrait_viewer::PortraitViewerScreen;
pub use scene_camera::{SceneCameraScreen, scene_camera};
pub use scene_preview::{ScenePreviewScreen, ScenePreviewView};
pub use shuyi_ui::debug::{DebugEntry, GlyphSamplerScreen};
pub use sprite_test::SpriteTestScreen;

use trpg_content::Present;

use crate::color::{Palette, UiColor};
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::map_view::{MapSkin, TEST_TILESET, next_skin};
use crate::screen::{Ctx, Shuyi, Transition};
use crate::screens::{ChangeKind, ClassChangeScreen, DialogueScreen, KeyBindingsScreen};

/// The debug menu for this game ([`shuyi_ui::debug::DebugMenuScreen`]).
pub type DebugMenuScreen = shuyi_ui::debug::DebugMenuScreen<Shuyi>;

/// Names of every screen the debug key does nothing on: the debug screens,
/// and the Key bindings screen (the Debug key is a key like any other while
/// it captures one, and is refused as reserved).
pub const SCREENS: [&str; 7] = [
    DebugMenuScreen::NAME,
    GlyphSamplerScreen::NAME,
    PortraitViewerScreen::NAME,
    KeyBindingsScreen::NAME,
    SpriteTestScreen::NAME,
    SceneCameraScreen::NAME,
    CinematicScreen::NAME,
];

/// The debug tools, in menu order, before "Map skin" ([`MAP_SKIN_TOOL`]),
/// whose label names the skin in use.
const TOOLS: [&str; 10] = [
    "Glyph sampler",
    "Portraits",
    "Play test scene",
    "Play test scene (overlay)",
    "Sprite test",
    "Class change: promote",
    "Class change: reclass",
    "Voice test",
    "Play test cinematic",
    "Scene camera",
];
/// Index of "Sprite test" in [`TOOLS`].
#[cfg(test)]
const SPRITE_TEST_TOOL: usize = 4;
/// Index of "Class change: promote" in [`TOOLS`].
const PROMOTE_TOOL: usize = 5;
/// Index of "Class change: reclass" in [`TOOLS`].
const RECLASS_TOOL: usize = 6;
/// Index of "Scene camera" in [`TOOLS`].
#[cfg(test)]
const SCENE_CAMERA_TOOL: usize = 9;
/// Index of "Play test cinematic" in [`TOOLS`].
#[cfg(test)]
const CINEMATIC_TOOL: usize = 8;
/// Index of "Voice test" in [`TOOLS`].
const VOICE_TEST_TOOL: usize = 7;
/// Index of "Map skin" in the menu: after [`TOOLS`].
#[cfg(test)]
const MAP_SKIN_TOOL: usize = TOOLS.len();
/// The scene the "Play test scene" tools play.
pub const TEST_SCENE: &str = "test";

/// The debug menu the debug key opens, with this game's tools in menu
/// order, then "Map skin", which names the skin `ctx` uses.
pub fn menu(ctx: &Ctx) -> DebugMenuScreen {
    DebugMenuScreen::new(ctx, tools())
}

/// This game's tools, in menu order.
fn tools() -> Vec<DebugEntry<Shuyi>> {
    let entries = TOOLS.iter().enumerate().map(|(i, &label)| {
        let mut said = 0;
        DebugEntry::new(label, move |ctx: &mut Ctx| match i {
            0 => Transition::Push(Box::new(sampler(ctx))),
            1 => Transition::Push(Box::new(PortraitViewerScreen::new())),
            2 | 3 => play_test_scene(ctx, i == 3),
            4 => Transition::Push(Box::new(SpriteTestScreen)),
            PROMOTE_TOOL | RECLASS_TOOL => {
                let kind = if i == PROMOTE_TOOL {
                    ChangeKind::Promote
                } else {
                    ChangeKind::Reclass
                };
                ClassChangeScreen::demo(ctx, kind)
                    .map_or(Transition::None, |s| Transition::Push(Box::new(s)))
            }
            VOICE_TEST_TOOL => {
                say_next_voice(ctx, &mut said);
                Transition::None
            }
            8 => Transition::Push(Box::new(CinematicScreen::new(ctx))),
            _ => Transition::Push(Box::new(scene_camera(ctx))),
        })
    });
    let skin = DebugEntry::named(
        |ctx: &Ctx| format!("Map skin: {}", skin_label(ctx.game.map_skin.as_ref())),
        |ctx: &mut Ctx| {
            switch_skin(ctx);
            Transition::None
        },
    );
    entries.chain([skin]).collect()
}

/// "Play test scene": the test scene full-screen, or (`overlay`) in place
/// of the menu so it plays over the screen the menu was opened from (e.g.
/// the battle map). The viewer plays a scene with everyone there
/// (ADR-0055). Nothing without the test scene.
fn play_test_scene(ctx: &mut Ctx, overlay: bool) -> Transition {
    let Some(scene) = ctx.game.content.dialogue.get(TEST_SCENE).cloned() else {
        return Transition::None;
    };
    let lead = ctx.game.lead.clone();
    let everyone = &Present::Everyone;
    if overlay {
        let screen = DialogueScreen::told_overlay(ctx, &scene, lead, everyone);
        Transition::Replace(Box::new(screen))
    } else {
        let screen = DialogueScreen::told(ctx, &scene, lead, everyone);
        Transition::Push(Box::new(screen))
    }
}

/// "Voice test": says the next line of the test scene that has a voice
/// clip (`said` of them said so far), starting over after the last. Each
/// time round it first tells `app` the lines that are coming, as a scene
/// would.
fn say_next_voice(ctx: &mut Ctx, said: &mut usize) {
    let Some(scene) = ctx.game.content.dialogue.get(TEST_SCENE) else {
        return;
    };
    let lines: Vec<_> = scene.lines().iter().map(|l| l.id.clone()).collect();
    let voiced: Vec<_> = lines.into_iter().filter(|l| ctx.has_voice(l)).collect();
    if voiced.is_empty() {
        return;
    }
    let next = *said % voiced.len();
    if next == 0 {
        ctx.preload_voices(&voiced);
    }
    ctx.play_voice(&voiced[next]);
    *said += 1;
}

/// How the "Map skin" item names `skin`: its tileset's id ("test
/// tileset" for the test one), or its own name.
fn skin_label(skin: &dyn MapSkin) -> &str {
    match skin.tileset_id() {
        Some(TEST_TILESET) => "test tileset",
        Some(id) => id,
        None => skin.name(),
    }
}

/// Swaps `ctx`'s map skin for the next in the round: the glyph skin, each
/// tileset the content has, then the glyph skin again.
fn switch_skin(ctx: &mut Ctx) {
    ctx.set_map_skin(next_skin(&ctx.game.content, ctx.game.map_skin.as_ref()));
}

/// The glyph sampler of the font and palette in `ctx`, with this game's
/// sample panels, laid out for the PC console.
pub fn sampler(ctx: &Ctx) -> GlyphSamplerScreen {
    let glyphs: Vec<char> = ctx.game.content.font.glyphs.keys().copied().collect();
    GlyphSamplerScreen::new(ctx, ConsoleSpec::PC, &glyphs, sample_panels)
}

/// Palette size the layout must still fit, so adding colours doesn't clip the
/// demo panels again (tickets 0209, 0210).
#[cfg(test)]
const PALETTE_HEADROOM: usize = 60;

/// A buffer the size of the PC console (its layout is the PC's): the top
/// half shows every glyph in `glyphs` in a grid; the bottom half shows
/// every palette colour as a swatch plus its name, a line of sample text
/// and this game's two sample panels.
pub fn glyph_sampler(palette: &Palette, glyphs: &[char]) -> GlyphBuffer {
    shuyi_ui::debug::glyph::sampler(ConsoleSpec::PC, palette, glyphs, sample_panels)
}

/// A single-line info panel and a double-line (focused) panel from row `top`.
fn sample_panels(buf: &mut GlyphBuffer, palette: &Palette, top: i32) {
    let ui = |u| palette.get(u);
    let bg = ui(UiColor::PanelBg);
    let height = i32::from(buf.height()) - top;

    let panel = Rect::new(1, top, 40, height);
    buf.fill_rect(panel, Cell::new(' ', ui(UiColor::Text), bg));
    buf.draw_box(panel, BoxStyle::Single, ui(UiColor::PanelBorder), bg);
    buf.print(3, top, " Unit ", ui(UiColor::TextHighlight), bg);
    buf.print(3, top + 1, "Aldo", ui(UiColor::Player), bg);
    buf.print_fg(8, top + 1, "vs", ui(UiColor::TextDim));
    buf.print(11, top + 1, "Brigand", ui(UiColor::Enemy), bg);
    buf.print(3, top + 2, "HP", ui(UiColor::Text), bg);
    buf.print(6, top + 2, "■■■■■■", ui(UiColor::HpHigh), bg);
    buf.print(12, top + 2, "■■■", ui(UiColor::HpMid), bg);
    buf.print(15, top + 2, "■", ui(UiColor::HpLow), bg);
    buf.print(18, top + 2, "░▒▓█▀▄▌▐", ui(UiColor::ExpBar), bg);

    let focus = Rect::new(43, top, 56, height);
    buf.fill_rect(focus, Cell::new(' ', ui(UiColor::Text), bg));
    buf.draw_box(focus, BoxStyle::Double, ui(UiColor::PanelBorderFocus), bg);
    buf.print(45, top, " Ranges ", ui(UiColor::TextHighlight), bg);
    for (i, (label, tint)) in (0..).zip([
        ("move", UiColor::MoveRange),
        ("attack", UiColor::AttackRange),
        ("heal", UiColor::HealRange),
        ("danger", UiColor::DangerZone),
    ]) {
        let x = 45 + i * 13;
        buf.blend_bg(Rect::new(x, top + 1, 12, 1), ui(tint), 1.0);
        buf.print_fg(x, top + 1, &format!("{label:^12}"), ui(UiColor::Text));
    }
    // Two-cell tiles, as in ADR-0012's examples.
    let terrain = ["grass", "forest", "water", "mountain", "stone"];
    let glyphs = ["..", "♣♣", "≈≈", "^^", "▓▓"];
    for (i, (name, g)) in (0..).zip(terrain.iter().zip(glyphs)) {
        let fg = palette.lookup(name).unwrap_or(ui(UiColor::White));
        buf.print(45 + i * 3, top + 2, g, fg, bg);
    }
    buf.print(61, top + 2, "A·", ui(UiColor::Player), bg);
    buf.print(64, top + 2, "Bˇ", ui(UiColor::Enemy).scale(0.5), bg);
    buf.print(67, top + 2, "C!", ui(UiColor::Ally), bg);
    buf.print(70, top + 2, "D•", ui(UiColor::Neutral), bg);
    buf.print(73, top + 2, "[]", ui(UiColor::Cursor), bg);
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;
    use crate::color::tests::game_palette;
    use crate::input::Action;
    use crate::screen::FrameInput;
    use shuyi_ui::Screen;
    use shuyi_ui::debug::glyph::{
        GLYPHS_PER_ROW, SWATCH_COLUMNS, SWATCH_W, palette_top, panels_top, panels_top_for,
    };

    fn atlas_glyphs() -> Vec<char> {
        trpg_content::font::load()
            .unwrap()
            .glyphs
            .into_keys()
            .collect()
    }

    #[test]
    fn sampler_shows_every_glyph_and_colour() {
        let p = game_palette();
        let glyphs = atlas_glyphs();
        let b = glyph_sampler(&p, &glyphs);
        assert_eq!(
            (b.width(), b.height()),
            (ConsoleSpec::PC.cols, ConsoleSpec::PC.rows)
        );
        let bottom = palette_top(&glyphs);
        let row_end = 2 + 2 * i32::try_from(GLYPHS_PER_ROW).unwrap();
        let shown: String = (1..bottom)
            .flat_map(|y| (2..row_end).step_by(2).map(move |x| (x, y)))
            .map(|(x, y)| b.get(x, y).unwrap().glyph)
            .collect();
        let expected: String = glyphs.iter().collect();
        assert!(
            shown.starts_with(&expected),
            "top half must list every glyph in order"
        );
        for (name, rgb) in p.iter() {
            let found = (bottom..i32::from(ConsoleSpec::PC.rows)).any(|y| {
                (0..i32::from(ConsoleSpec::PC.cols)).any(|x| {
                    let cell = b.get(x, y).unwrap();
                    cell.glyph == '█' && cell.fg == rgb
                })
            });
            assert!(found, "no swatch for {name}");
        }
    }

    #[test]
    fn glyph_grid_fits_top_half() {
        let glyphs = atlas_glyphs();
        let rows = glyphs.len().div_ceil(GLYPHS_PER_ROW);
        assert!(i32::try_from(rows).unwrap() < palette_top(&glyphs));
        assert!(2 + 2 * GLYPHS_PER_ROW <= usize::from(ConsoleSpec::PC.cols));
        assert!(SWATCH_COLUMNS * SWATCH_W < i32::from(ConsoleSpec::PC.cols));
    }

    #[test]
    fn demo_panels_fit_below_the_embedded_palette() {
        let p = game_palette();
        let glyphs = atlas_glyphs();
        assert!(panels_top(&p, &glyphs) <= i32::from(ConsoleSpec::PC.rows) - 4);
    }

    #[test]
    fn demo_panels_keep_margin_for_a_larger_palette() {
        let p = game_palette();
        let glyphs = atlas_glyphs();
        assert!(p.iter().count() <= PALETTE_HEADROOM);
        assert!(panels_top_for(PALETTE_HEADROOM, &glyphs) <= i32::from(ConsoleSpec::PC.rows) - 4);
        // At least two spare rows for today's palette.
        assert!(panels_top(&p, &glyphs) <= i32::from(ConsoleSpec::PC.rows) - 6);
    }

    #[test]
    fn sampler_screen_draws_the_sampler_and_closes_on_cancel() {
        let mut ctx = crate::screen::tests::ctx();
        let mut screen = sampler(&ctx);
        assert_eq!(Screen::<Shuyi>::name(&screen), "glyph_sampler");
        let p = &ctx.palette;
        let mut buf = GlyphBuffer::with_spec(
            ConsoleSpec::PC,
            Cell::new('x', p.get(UiColor::Text), p.get(UiColor::Black)),
        );
        screen.draw(&ctx, &mut buf);
        assert_eq!(buf, glyph_sampler(&ctx.palette, &atlas_glyphs()));
        let frame = |a: &[Action]| FrameInput::new(a.to_vec(), 0.0, vec![]);
        let stay = screen.update(&mut ctx, &frame(&[Action::Confirm]));
        assert!(matches!(stay, Transition::None));
        let pop = screen.update(&mut ctx, &frame(&[Action::Confirm, Action::Cancel]));
        assert!(matches!(pop, Transition::Pop));
    }

    #[test]
    fn debug_menu_opens_each_tool() {
        use Action::{Cancel, Confirm, CursorDown, CursorUp};
        let mut ctx = crate::screen::tests::ctx();
        let mut menu = super::menu(&ctx);
        assert_eq!(menu.name(), "debug_menu");
        let mut outcome = |m: &mut DebugMenuScreen, a: &[Action]| {
            format!(
                "{:?}",
                m.update(&mut ctx, &FrameInput::new(a.to_vec(), 0.0, vec![]))
            )
        };
        assert_eq!(outcome(&mut menu, &[CursorUp]), "None");
        assert_eq!(
            outcome(&mut menu, &[CursorDown, Confirm]),
            "Push(glyph_sampler)"
        );
        assert_eq!(
            outcome(&mut menu, &[CursorDown, Confirm]),
            "Push(portrait_viewer)"
        );
        assert_eq!(outcome(&mut menu, &[CursorDown, Confirm]), "Push(dialogue)");
        assert_eq!(
            outcome(&mut menu, &[CursorDown, Confirm]),
            "Replace(dialogue)"
        );
        assert_eq!(
            outcome(&mut menu, &[CursorDown, Confirm]),
            "Push(sprite_test)"
        );
        assert_eq!(
            outcome(&mut menu, &[CursorDown, Confirm]),
            "Push(class_change)"
        );
        assert_eq!(
            outcome(&mut menu, &[CursorDown, Confirm]),
            "Push(class_change)"
        );
        // "Voice test" stays on the menu.
        assert_eq!(outcome(&mut menu, &[CursorDown, Confirm]), "None");
        assert_eq!(
            outcome(&mut menu, &[CursorDown, Confirm]),
            "Push(cinematic)"
        );
        assert_eq!(
            outcome(&mut menu, &[CursorDown, Confirm]),
            "Push(scene_camera)"
        );
        assert_eq!(outcome(&mut menu, &[Cancel, Confirm]), "Pop");
        // Without the test scene, its tools do nothing; nor do the class
        // change tools without their test character.
        ctx.game.content.dialogue.scenes.clear();
        ctx.game.content.characters.characters.clear();
        for downs in [2, 3, PROMOTE_TOOL, RECLASS_TOOL] {
            let mut menu = super::menu(&ctx);
            let mut a = vec![CursorDown; downs];
            a.push(Confirm);
            let frame = FrameInput::new(a, 0.0, vec![]);
            assert!(matches!(menu.update(&mut ctx, &frame), Transition::None));
        }
        assert_eq!(
            SCREENS,
            [
                "debug_menu",
                "glyph_sampler",
                "portrait_viewer",
                "key_bindings",
                "sprite_test",
                "scene_camera",
                "cinematic"
            ]
        );
        assert_eq!(TOOLS[SPRITE_TEST_TOOL], "Sprite test");
        assert_eq!(TOOLS[PROMOTE_TOOL], "Class change: promote");
        assert_eq!(TOOLS[RECLASS_TOOL], "Class change: reclass");
        assert_eq!(TOOLS[SCENE_CAMERA_TOOL], "Scene camera");
        assert_eq!(TOOLS[VOICE_TEST_TOOL], "Voice test");
        assert_eq!(TOOLS[CINEMATIC_TOOL], "Play test cinematic");
    }

    /// Ticket 0238: "Voice test" says the test scene's voiced lines in
    /// turn, announcing them each time round.
    #[test]
    fn voice_test_says_the_voiced_lines_of_the_test_scene_in_turn() {
        use crate::audio::AudioRequest;
        use trpg_content::Variant;
        let mut ctx = crate::screen::tests::ctx();
        let (manifest, lines) = crate::screen::tests::test_voices(&ctx, &[0, 2]);
        let mut menu = super::menu(&ctx);
        let choose = |ctx: &mut Ctx, menu: &mut DebugMenuScreen| {
            *menu = std::mem::replace(menu, super::menu(ctx)).focused(VOICE_TEST_TOOL);
            let frame = FrameInput::new(vec![Action::Confirm], 0.0, vec![]);
            assert!(matches!(menu.update(ctx, &frame), Transition::None));
            let voice = |r: &AudioRequest| !matches!(r, AudioRequest::PlaySound { .. });
            ctx.audio
                .take()
                .into_iter()
                .filter(voice)
                .collect::<Vec<_>>()
        };
        // No voices: only the menu's own sound.
        assert_eq!(choose(&mut ctx, &mut menu), []);
        ctx.set_voice_manifest(&manifest);
        let play = |i: usize| AudioRequest::PlayVoice {
            line: lines[i].clone(),
            variant: Variant::None,
        };
        let preload = AudioRequest::PreloadVoices {
            lines: lines.iter().map(|l| (l.clone(), Variant::None)).collect(),
        };
        assert_eq!(choose(&mut ctx, &mut menu), [preload.clone(), play(0)]);
        assert_eq!(choose(&mut ctx, &mut menu), [play(1)]);
        assert_eq!(choose(&mut ctx, &mut menu), [preload, play(0)]);
        // Without the test scene it does nothing.
        ctx.game.content.dialogue.scenes.clear();
        assert_eq!(choose(&mut ctx, &mut menu), []);
    }

    #[test]
    fn debug_menu_snapshot() {
        let ctx = crate::screen::tests::ctx();
        let p = &ctx.palette;
        let mut buf = GlyphBuffer::with_spec(
            ConsoleSpec::PC,
            Cell::new('x', p.get(UiColor::Text), p.get(UiColor::PanelBg)),
        );
        super::menu(&ctx).draw(&ctx, &mut buf);
        assert_snapshot!(buf.to_snapshot(p));
    }

    #[test]
    fn map_skin_switches_between_the_glyph_skin_and_the_test_tileset() {
        use Action::{Confirm, CursorDown, CursorUp};
        let mut ctx = crate::screen::tests::ctx();
        let mut menu = super::menu(&ctx);
        let label = |m: &DebugMenuScreen| m.menu().items()[MAP_SKIN_TOOL].label.clone();
        assert_eq!(label(&menu), "Map skin: glyph");
        // Up from the first tool wraps round to it.
        let frame = |a: &[Action]| FrameInput::new(a.to_vec(), 0.0, vec![]);
        let stay = menu.update(&mut ctx, &frame(&[CursorUp, Confirm]));
        assert!(matches!(stay, Transition::None));
        assert_eq!(ctx.game.map_skin.name(), "sprite");
        assert_eq!(label(&menu), "Map skin: test tileset");
        // Still on it: again for the tileset with layers, again for the
        // unit sheets on glyph terrain, and again for glyphs.
        assert_eq!(menu.menu().focus(), MAP_SKIN_TOOL);
        menu.update(&mut ctx, &frame(&[Confirm]));
        assert_eq!(ctx.game.map_skin.name(), "sprite");
        assert_eq!(label(&menu), "Map skin: test_auto");
        menu.update(&mut ctx, &frame(&[Confirm]));
        assert_eq!(ctx.game.map_skin.name(), "sprite_units");
        assert_eq!(label(&menu), "Map skin: test_units");
        menu.update(&mut ctx, &frame(&[Confirm]));
        assert_eq!(ctx.game.map_skin.name(), "glyph");
        assert_eq!(label(&menu), "Map skin: glyph");
        // A menu opened later names the skin in use.
        menu.update(&mut ctx, &frame(&[Confirm]));
        assert_eq!(label(&super::menu(&ctx)), "Map skin: test tileset");
        // Without any tileset, the glyph skin stays.
        ctx.game.content.tilesets.clear();
        menu.update(&mut ctx, &frame(&[Confirm]));
        assert_eq!(ctx.game.map_skin.name(), "glyph");
        menu.update(&mut ctx, &frame(&[Confirm]));
        assert_eq!(ctx.game.map_skin.name(), "glyph");
        assert_eq!(label(&menu), "Map skin: glyph");
        let down = menu.update(&mut ctx, &frame(&[CursorDown]));
        assert!(matches!(down, Transition::None));
    }

    #[test]
    fn sampler_snapshot() {
        let p = game_palette();
        let snap = glyph_sampler(&p, &atlas_glyphs()).to_snapshot(&p);
        assert_snapshot!(snap);
    }
}
