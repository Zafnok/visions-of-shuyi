//! The scene preview (ticket 0723): the game opened straight on one
//! dialogue scene, for whoever writes scripts
//! (`docs/story/writers-guide.md`). `app` starts it for `--scene <id>`
//! ([`Game::start_on_scene`](crate::Game::start_on_scene)). The scene plays
//! with everyone there (ADR-0055), as in the debug menu's test scene; when
//! it ends this screen shows, and `Confirm` plays it again.
//!
//! What the screen says is a [`ScenePreviewView`]; [`paint`] draws it
//! (ADR-0054).

use trpg_content::{Present, Scene};

use crate::audio::MenuSound;
use crate::color::UiColor;
use crate::glyph_buffer::{Cell, GlyphBuffer};
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Transition};
use crate::screens::{DialogueScreen, print_centred};
use crate::widgets::help::{help_line, key_name};

/// Row of the "End of scene" line.
const END_ROW: i32 = 14;

/// What the screen under a previewed scene shows once it has ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenePreviewView {
    /// The line that names the scene.
    pub ended: String,
    /// The help line: the keys that play it again and quit.
    pub help: String,
}

/// Waits under a previewed scene: `Confirm` plays it again, `Cancel`
/// quits.
#[derive(Debug, Clone)]
pub struct ScenePreviewScreen {
    scene: Scene,
}

impl ScenePreviewScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "scene_preview";

    /// The preview of `scene`.
    pub fn new(scene: Scene) -> Self {
        Self { scene }
    }

    /// The scene from its first line, full-screen, with everyone there and
    /// `ctx`'s lead.
    pub fn play(&self, ctx: &Ctx) -> DialogueScreen {
        let (lead, names) = (ctx.lead.clone(), ctx.content.names.clone());
        DialogueScreen::new(&self.scene, lead, names, &Present::Everyone)
    }

    /// What the screen shows.
    pub fn view(&self, ctx: &Ctx) -> ScenePreviewView {
        let km = ctx.help_keys();
        ScenePreviewView {
            ended: format!("End of scene {}", self.scene.id),
            help: help_line(&[
                (Some(key_name(km, Action::Confirm)), "play again"),
                (Some(key_name(km, Action::Cancel)), "quit"),
            ]),
        }
    }
}

/// Draws `view` as glyphs.
pub fn paint(ctx: &Ctx, view: &ScenePreviewView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let black = c(UiColor::Black);
    buf.fill_rect(buf.bounds(), Cell::new(' ', c(UiColor::Text), black));
    print_centred(buf, END_ROW, &view.ended, c(UiColor::TextHighlight), black);
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, c(UiColor::TextDim), black);
}

impl Screen for ScenePreviewScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        for &action in &input.actions {
            match action {
                Action::Confirm => {
                    ctx.audio.menu(MenuSound::Select);
                    return Transition::Push(Box::new(self.play(ctx)));
                }
                Action::Cancel => return Transition::Quit,
                _ => {}
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug::TEST_SCENE;
    use crate::screen::tests::ctx;

    fn screen(ctx: &Ctx) -> ScenePreviewScreen {
        ScenePreviewScreen::new(ctx.content.dialogue.scenes[TEST_SCENE].clone())
    }

    fn update(s: &mut ScenePreviewScreen, ctx: &mut Ctx, actions: &[Action]) -> String {
        let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
        format!("{:?}", s.update(ctx, &input))
    }

    #[test]
    fn confirm_plays_the_scene_again_and_cancel_quits() {
        let mut ctx = ctx();
        let mut s = screen(&ctx);
        assert_eq!(s.name(), "scene_preview");
        assert_eq!(update(&mut s, &mut ctx, &[]), "None");
        assert_eq!(update(&mut s, &mut ctx, &[Action::CursorDown]), "None");
        assert_eq!(
            update(&mut s, &mut ctx, &[Action::CursorDown, Action::Confirm]),
            "Push(dialogue)"
        );
        assert_eq!(update(&mut s, &mut ctx, &[Action::Cancel]), "Quit");
        assert!(s.as_any().is_some());
    }

    #[test]
    fn the_scene_plays_with_everyone_there_and_the_lead_of_the_game() {
        let mut ctx = ctx();
        ctx.lead.gender = trpg_core::LeadGender::Female;
        let played = screen(&ctx).play(&ctx);
        let everyone = &Present::Everyone;
        let scene = &ctx.content.dialogue.scenes[TEST_SCENE];
        let (lead, names) = (ctx.lead.clone(), ctx.content.names.clone());
        let expected = DialogueScreen::new(scene, lead, names, everyone);
        assert_eq!(played.player(), expected.player());
        assert!(played.has_text());
    }

    #[test]
    fn the_view_names_the_scene_and_the_keys_of_the_keymap() {
        let ctx = ctx();
        let view = screen(&ctx).view(&ctx);
        assert_eq!(view.ended, "End of scene test");
        let km = ctx.help_keys();
        let (again, quit) = (key_name(km, Action::Confirm), key_name(km, Action::Cancel));
        assert_ne!(again, quit);
        let hints = [(Some(again), "play again"), (Some(quit), "quit")];
        assert_eq!(view.help, help_line(&hints));
        assert!(view.help.contains(" play again"), "{}", view.help);
        assert!(view.help.ends_with(" quit"), "{}", view.help);
    }

    #[test]
    fn the_view_is_painted_centred_with_the_help_on_the_bottom_row() {
        let ctx = ctx();
        let black = ctx.palette.get(UiColor::Black);
        let mut buf = GlyphBuffer::new(40, 20, Cell::new('x', black, black));
        let view = ScenePreviewView {
            ended: "ENDED".to_owned(),
            help: "HELP".to_owned(),
        };
        paint(&ctx, &view, &mut buf);
        let row = |y: i32| -> String {
            (0..40)
                .map(|x| buf.get(x, y).map_or('?', |c| c.glyph))
                .collect()
        };
        assert_eq!(row(END_ROW).trim(), "ENDED");
        assert_eq!(row(19).trim(), "HELP");
        assert_eq!(row(0).trim(), "");
        let hi = ctx.palette.get(UiColor::TextHighlight);
        let at = row(END_ROW).find('E').and_then(|x| i32::try_from(x).ok());
        assert_eq!(at.and_then(|x| buf.get(x, END_ROW)).map(|c| c.fg), Some(hi));
    }
}
