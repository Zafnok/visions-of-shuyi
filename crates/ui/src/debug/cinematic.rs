//! "Play test cinematic" (debug builds, tickets 0817, 0818): plays
//! `assets-test/cinematics/test.ron` on the screen's own frame clock,
//! looping, until Cancel. For watching a cinematic file by eye.

use trpg_content::bundle::test_display_path;

use crate::audio::MenuSound;
use crate::cinematic::CinematicPlayer;
use crate::color::UiColor;
use crate::glyph_buffer::{Cell, GlyphBuffer};
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Transition};
use crate::screens::print_centred;
use crate::widgets::help::{help_line, key_name};

/// The cinematic played: `assets-test/cinematics/test.ron`.
pub const TEST_CINEMATIC: &str = "test";

/// Plays the test cinematic over and over; `Cancel` closes.
#[derive(Debug, Clone)]
pub struct CinematicScreen {
    /// `None` if the content has no test cinematic.
    player: Option<CinematicPlayer>,
    /// Seconds into the cinematic: below its length.
    time: f32,
}

impl CinematicScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "cinematic";

    /// The test cinematic at its start, its logo over the first of the
    /// title's backdrops, with the lead a game without a save has.
    pub fn new(ctx: &Ctx) -> Self {
        let backdrop = ctx.content.title.backdrops.first().copied();
        Self {
            player: CinematicPlayer::new(ctx, TEST_CINEMATIC, None)
                .map(|p| p.with_backdrop(backdrop)),
            time: 0.0,
        }
    }

    /// Seconds into the cinematic.
    pub fn time(&self) -> f32 {
        self.time
    }

    /// The player, if the content has the test cinematic.
    pub fn player(&self) -> Option<&CinematicPlayer> {
        self.player.as_ref()
    }
}

impl Screen for CinematicScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if input.actions.contains(&Action::Cancel) {
            ctx.audio.menu(MenuSound::Cancel);
            return Transition::Pop;
        }
        if let Some(player) = &self.player
            && input.dt.is_finite()
        {
            #[allow(clippy::cast_precision_loss)] // a length is far below 2^24 ms
            let length = player.timeline().length_ms() as f32 / 1000.0;
            self.time = (self.time + input.dt.max(0.0)) % length;
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        let c = |u| ctx.palette.get(u);
        let (black, text, dim) = (c(UiColor::Black), c(UiColor::Text), c(UiColor::TextDim));
        if let Some(player) = &self.player {
            player.draw(ctx, self.time, 1.0, buf);
        } else {
            buf.fill_rect(buf.bounds(), Cell::new(' ', text, black));
            let path = format!("cinematics/{TEST_CINEMATIC}.ron");
            let missing = format!("No {}", test_display_path(&path));
            buf.print(2, 2, &missing, text, black);
        }
        let help = help_line(&[(Some(key_name(ctx.help_keys(), Action::Cancel)), "back")]);
        print_centred(buf, i32::from(buf.height()) - 1, &help, dim, black);
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cinematic::Frame;
    use crate::console::ConsoleSpec;
    use crate::glyph_buffer::Rect;
    use crate::screen::tests::ctx;

    fn frame(pressed: &[Action], dt: f32) -> FrameInput {
        FrameInput::new(pressed.to_vec(), dt, vec![])
    }

    fn draw(ctx: &Ctx, screen: &CinematicScreen) -> GlyphBuffer {
        let p = &ctx.palette;
        let stale = Cell::new('x', p.get(UiColor::Text), p.get(UiColor::PanelBg));
        let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
        screen.draw(ctx, &mut buf);
        buf
    }

    /// The bottom row of `buf` as text.
    fn bottom(buf: &GlyphBuffer) -> String {
        let y = i32::from(buf.height()) - 1;
        let row: String = (0..i32::from(buf.width()))
            .map(|x| buf.get(x, y).unwrap().glyph)
            .collect();
        row.trim().to_owned()
    }

    #[test]
    fn plays_on_its_own_clock_and_loops() {
        let mut ctx = ctx();
        let mut screen = CinematicScreen::new(&ctx);
        assert_eq!(screen.name(), "cinematic");
        assert_eq!(screen.time().to_bits(), 0.0_f32.to_bits());
        let player = screen.player().unwrap().clone();
        let shown =
            |ctx: &Ctx, s: &CinematicScreen| player.frame(ctx, s.time(), Rect::new(0, 0, 100, 32));
        assert!(matches!(shown(&ctx, &screen), Frame::MapPan { .. }));
        // Keys other than Cancel do nothing; time goes on.
        let stay = screen.update(&mut ctx, &frame(&[Action::Confirm], 2.5));
        assert!(matches!(stay, Transition::None));
        assert!((screen.time() - 2.5).abs() < 1e-4);
        screen.update(&mut ctx, &frame(&[], 4.0));
        assert!((screen.time() - 6.5).abs() < 1e-4);
        assert!(matches!(shown(&ctx, &screen), Frame::Talk { .. }));
        screen.update(&mut ctx, &frame(&[], 5.0));
        assert!(matches!(shown(&ctx, &screen), Frame::Characters { .. }));
        screen.update(&mut ctx, &frame(&[], 5.0));
        assert!((screen.time() - 16.5).abs() < 1e-4);
        assert_eq!(shown(&ctx, &screen), Frame::Logo { sparks: None });
        // Twenty seconds long: 16.5 + 4 is half a second into the next
        // round.
        screen.update(&mut ctx, &frame(&[], 4.0));
        assert!((screen.time() - 0.5).abs() < 1e-3, "{}", screen.time());
        assert!(matches!(shown(&ctx, &screen), Frame::MapPan { .. }));
        // A bad frame time moves nothing.
        for dt in [f32::NAN, f32::INFINITY, -1.0] {
            screen.update(&mut ctx, &frame(&[], dt));
            assert!((screen.time() - 0.5).abs() < 1e-3, "{dt}");
        }
        assert!(ctx.audio.take().is_empty());
    }

    #[test]
    fn draws_the_player_at_its_time_with_a_help_line() {
        let mut ctx = ctx();
        let mut screen = CinematicScreen::new(&ctx);
        screen.update(&mut ctx, &frame(&[], 3.0));
        let buf = draw(&ctx, &screen);
        let mut expected = draw(&ctx, &CinematicScreen::new(&ctx));
        assert_ne!(buf, expected);
        screen.player().unwrap().draw(&ctx, 3.0, 1.0, &mut expected);
        // All but the help line is the player's frame at full brightness.
        let last = i32::from(ConsoleSpec::PC.rows) - 1;
        for y in 0..last {
            for x in 0..i32::from(ConsoleSpec::PC.cols) {
                assert_eq!(buf.get(x, y), expected.get(x, y), "({x}, {y})");
            }
        }
        assert_eq!(buf.backdrop(), expected.backdrop());
        assert!(bottom(&buf).ends_with(" back"), "{}", bottom(&buf));
        assert!(!buf.get(50, last).unwrap().see_through);
        // The logo is over the title's first backdrop.
        let first = ctx.content.title.backdrops[0];
        screen.update(&mut ctx, &frame(&[], 14.0));
        let logo = draw(&ctx, &screen);
        assert!(logo.backdrop().is_none());
        assert!(logo.sprites().iter().any(|s| s.image == first));
    }

    #[test]
    fn cancel_closes_with_the_cancel_sound() {
        let mut ctx = ctx();
        let mut screen = CinematicScreen::new(&ctx);
        let pop = screen.update(&mut ctx, &frame(&[Action::Confirm, Action::Cancel], 1.0));
        assert!(matches!(pop, Transition::Pop));
        assert_eq!(screen.time().to_bits(), 0.0_f32.to_bits());
        let sounds = ctx.audio.take();
        let cues: Vec<_> = sounds.iter().map(|s| s.cue()).collect();
        assert_eq!(cues, [Some("menu_cancel")]);
    }

    #[test]
    fn says_so_when_the_cinematic_is_missing() {
        let mut ctx = ctx();
        ctx.content.cinematics.clear();
        let mut screen = CinematicScreen::new(&ctx);
        assert!(screen.player().is_none());
        screen.update(&mut ctx, &frame(&[], 1.0));
        assert_eq!(screen.time().to_bits(), 0.0_f32.to_bits());
        let buf = draw(&ctx, &screen);
        let row: String = (0..50).map(|x| buf.get(x, 2).unwrap().glyph).collect();
        assert_eq!(row.trim(), "No assets-test/cinematics/test.ron");
        assert!(bottom(&buf).ends_with(" back"));
        assert_eq!(buf.get(0, 0).unwrap().glyph, ' ');
    }
}
