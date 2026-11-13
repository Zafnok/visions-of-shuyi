//! A one-time tip over a screen outside battle (ticket 0874): the tip box
//! alone, over the screen that opened it, closed by Confirm or Cancel.
//! The screen asks for it with [`TipScreen::once`] the first time it
//! opens; the tip is then seen for the profile (ADR-0069), so it shows
//! once. The battle keeps its own queue of tips (`screens::battle::tips`).
//! The guide says the tip (ticket 0497): its line is preloaded when the box
//! is made, asked for on its first frame, and stopped when it closes.

use crate::prelude::*;
use trpg_content::{LineId, TipTrigger};

use super::{TipView, TipsSeen, glyph};
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::screens::battle::tips::TipState;

/// The tip box over the screen below.
#[derive(Debug, Clone)]
pub struct TipScreen {
    trigger: TipTrigger,
    /// The guide's line of the tip ([`trpg_content::voice::guide_line`]).
    line: LineId,
    /// Whether the first frame has run: she says the line only then.
    said: bool,
    /// Whether her clip was asked for: only then is there a voice to stop.
    speaking: bool,
}

impl TipScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "tip";

    /// The box of `trigger`'s tip, to push over the screen asking, if it
    /// should show: tips show in `ctx` ([`TipState::shown_in`]), the game
    /// has the tip, and the profile hasn't seen it. It is marked seen now.
    pub fn once(ctx: &mut Ctx, trigger: TipTrigger) -> Option<Self> {
        if !TipState::shown_in(ctx) {
            return None;
        }
        let tip = ctx.game.content.tips.for_trigger(trigger)?;
        let mut seen = TipsSeen::load(&ctx.storage);
        if seen.contains(&tip.id) {
            return None;
        }
        let id = tip.id.clone();
        let line = trpg_content::voice::guide_line(&tip.text);
        seen.mark(&id, &mut ctx.storage);
        ctx.preload_voices(core::slice::from_ref(&line));
        Some(Self {
            trigger,
            line,
            said: false,
            speaking: false,
        })
    }

    /// The tip it shows.
    pub fn trigger(&self) -> TipTrigger {
        self.trigger
    }

    /// The box, as plain data; `None` if the game has no such tip.
    pub fn view(&self, ctx: &Ctx) -> Option<TipView> {
        let tip = ctx.game.content.tips.for_trigger(self.trigger)?;
        Some(TipView::new(ctx, tip))
    }
}

impl Screen<Shuyi> for TipScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn actions(&self) -> Option<Vec<Action>> {
        Some(crate::screens::actions::TIP.to_vec())
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if input
            .actions
            .iter()
            .any(|a| matches!(a, Action::Confirm | Action::Cancel))
        {
            ctx.audio.menu(MenuSound::Select);
            if self.speaking {
                ctx.stop_voice();
            }
            return Transition::Pop;
        }
        if !core::mem::replace(&mut self.said, true) && ctx.has_voice(&self.line) {
            ctx.play_voice(&self.line);
            self.speaking = true;
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        if let Some(view) = self.view(ctx) {
            glyph::paint(ctx, &view, buf);
        }
    }

    fn is_overlay(&self) -> bool {
        true
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screen::tests::ctx;

    #[test]
    fn a_tip_shows_once_and_only_with_tips_on() {
        let mut c = ctx();
        assert!(TipScreen::once(&mut c, TipTrigger::FirstLoadGame).is_none());
        c.game.tips_enabled = true;
        let tip = TipScreen::once(&mut c, TipTrigger::FirstLoadGame).unwrap();
        assert_eq!(tip.trigger(), TipTrigger::FirstLoadGame);
        assert!(tip.is_overlay());
        assert!(TipScreen::once(&mut c, TipTrigger::FirstLoadGame).is_none());
        assert!(TipScreen::once(&mut c, TipTrigger::FirstProfileScreen).is_some());
    }

    #[test]
    fn confirm_or_cancel_closes_it() {
        let mut c = ctx();
        c.game.tips_enabled = true;
        let mut tip = TipScreen::once(&mut c, TipTrigger::FirstLoadGame).unwrap();
        crate::screens::actions::assert_declares(&tip, crate::screens::actions::TIP);
        let frame = |a: &[Action]| FrameInput::new(a.to_vec(), 0.0, vec![]);
        let mut step = |a: &[Action]| format!("{:?}", tip.update(&mut c, &frame(a)));
        assert_eq!(step(&[Action::CursorDown]), "None");
        assert_eq!(step(&[Action::Confirm]), "Pop");
        assert_eq!(step(&[Action::Cancel]), "Pop");
    }

    /// A context with tips on and a voice manifest holding a clip for
    /// every line the guide says.
    fn voiced() -> Ctx {
        let (mut c, _) = crate::screens::battle::guide_voice::tests::voiced();
        c.game.tips_enabled = true;
        c
    }

    /// The guide's line of `trigger`'s tip.
    fn line(c: &Ctx, trigger: TipTrigger) -> LineId {
        let tip = c.game.content.tips.for_trigger(trigger).unwrap();
        trpg_content::voice::guide_line(&tip.text)
    }

    fn voice_requests(c: &mut Ctx) -> Vec<crate::AudioRequest> {
        use crate::AudioRequest as R;
        c.audio
            .take()
            .into_iter()
            .filter(|r| {
                matches!(
                    r,
                    R::PlayVoice { .. } | R::StopVoice | R::PreloadVoices { .. }
                )
            })
            .collect()
    }

    /// Acceptance (0497): the box over the Load Game list preloads its
    /// line when made, asks for it once on its first frame, and stops it
    /// when Confirm closes it.
    #[test]
    fn the_tip_box_says_its_line_once_and_stops_it_on_close() {
        use crate::AudioRequest as R;
        use trpg_content::Variant;
        let mut c = voiced();
        let line = line(&c, TipTrigger::FirstLoadGame);
        let mut tip = TipScreen::once(&mut c, TipTrigger::FirstLoadGame).unwrap();
        assert_eq!(
            voice_requests(&mut c),
            [R::PreloadVoices {
                lines: vec![(line.clone(), Variant::None)]
            }]
        );
        let frame = |a: &[Action]| FrameInput::new(a.to_vec(), 0.0, vec![]);
        assert!(matches!(tip.update(&mut c, &frame(&[])), Transition::None));
        let play = R::PlayVoice {
            line,
            variant: Variant::None,
        };
        assert_eq!(voice_requests(&mut c), [play]);
        assert!(matches!(tip.update(&mut c, &frame(&[])), Transition::None));
        assert_eq!(voice_requests(&mut c), [], "asked for once");
        assert!(matches!(
            tip.update(&mut c, &frame(&[Action::Confirm])),
            Transition::Pop
        ));
        assert_eq!(voice_requests(&mut c), [R::StopVoice]);
    }

    /// Acceptance (0497): with voices off nothing is preloaded, asked for
    /// or stopped.
    #[test]
    fn with_voices_off_the_tip_box_asks_for_nothing() {
        let mut c = voiced();
        c.change_settings(|s| s.voices = false).unwrap();
        c.audio.take();
        let mut tip = TipScreen::once(&mut c, TipTrigger::FirstProfileScreen).unwrap();
        let frame = |a: &[Action]| FrameInput::new(a.to_vec(), 0.0, vec![]);
        tip.update(&mut c, &frame(&[]));
        tip.update(&mut c, &frame(&[Action::Cancel]));
        assert_eq!(voice_requests(&mut c), []);
    }
}
