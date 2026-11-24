//! A one-time tip over a screen outside battle (ticket 0874): the tip box
//! alone, over the screen that opened it, closed by Confirm or Cancel.
//! The screen asks for it with [`TipScreen::once`] the first time it
//! opens; the tip is then seen for the profile (ADR-0069), so it shows
//! once. The battle keeps its own queue of tips.
//! The guide says the tip (tickets 0497, 1291) through her
//! [`GuideVoice`], as in battle: on its first frame the box keeps its line
//! loaded and asks for it; closing it stops her and lets the line go. The
//! title keeps the lines of these tips loaded before their screens open.

use crate::prelude::*;
use trpg_content::{LineId, TipTrigger};

use super::voice::GuideVoice;
use super::{TipView, TipsSeen, glyph};
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};

/// The tips shown outside battle, by the screens that open them: the
/// Load Game list's and the Profile screen's (0874).
pub const OUT_OF_BATTLE: [TipTrigger; 2] =
    [TipTrigger::FirstLoadGame, TipTrigger::FirstProfileScreen];

/// The tip box over the screen below.
#[derive(Debug, Clone)]
pub struct TipScreen {
    trigger: TipTrigger,
    /// The guide's line of the tip ([`trpg_content::voice::guide_line`]).
    line: LineId,
    /// Her voice: the line kept loaded and said on the first frame.
    voice: GuideVoice,
}

impl TipScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "tip";

    /// The box of `trigger`'s tip, to push over the screen asking, if it
    /// should show: tips show in `ctx` ([`super::shown_in`]), the game
    /// has the tip, and the profile hasn't seen it. It is marked seen now.
    pub fn once(ctx: &mut Ctx, trigger: TipTrigger) -> Option<Self> {
        if !super::shown_in(ctx) {
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
        Some(Self {
            trigger,
            line,
            voice: GuideVoice::default(),
        })
    }

    /// The guide's lines of the tips of `triggers` that may still show in
    /// `ctx`: tips show, the game has them and the profile hasn't seen
    /// them. A screen that may open their boxes keeps them loaded (1291).
    pub fn coming_lines(ctx: &Ctx, triggers: &[TipTrigger]) -> Vec<LineId> {
        if !super::shown_in(ctx) {
            return Vec::new();
        }
        let seen = TipsSeen::load(&ctx.storage);
        let tips = triggers
            .iter()
            .filter_map(|&t| ctx.game.content.tips.for_trigger(t));
        tips.filter(|tip| !seen.contains(&tip.id))
            .map(|tip| trpg_content::voice::guide_line(&tip.text))
            .collect()
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
            self.voice.show(None, None, ctx);
            self.voice.leave(ctx);
            return Transition::Pop;
        }
        let line = self.line.clone();
        let keep = self.voice.keep_due(Some(&line), ctx);
        self.voice
            .show(Some(line.clone()), keep.then(|| vec![line]), ctx);
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

    /// Ticket 1310: a profile's seen tips from before triggers were names
    /// (the ids, one a line, as it saved them) still read: the tip seen
    /// then doesn't show again, the other does.
    #[test]
    fn a_profiles_seen_tips_from_before_still_read() {
        use crate::storage::Storage as _;
        let mut c = ctx();
        c.game.tips_enabled = true;
        let saved = c.game.content.tips.for_trigger(TipTrigger::FirstLoadGame);
        assert_eq!(saved.map(|t| t.id.as_str()), Some("export_save"));
        c.storage
            .write(crate::tips::TIPS_SEEN_KEY, "battle_start\nexport_save")
            .unwrap();
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
        let (mut c, _) = crate::tips::voice::testing::voiced();
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
                    R::PlayVoice { .. }
                        | R::StopVoice
                        | R::PreloadVoices { .. }
                        | R::KeepVoices { .. }
                )
            })
            .collect()
    }

    /// Acceptance (0497, 1291): the box over the Load Game list keeps its
    /// line loaded and asks for it once, on its first frame; Confirm
    /// stops it and lets the line go.
    #[test]
    fn the_tip_box_keeps_and_says_its_line_once_then_lets_it_go() {
        use crate::AudioRequest as R;
        use trpg_content::Variant;
        let mut c = voiced();
        let line = line(&c, TipTrigger::FirstLoadGame);
        let mut tip = TipScreen::once(&mut c, TipTrigger::FirstLoadGame).unwrap();
        assert_eq!(voice_requests(&mut c), [], "nothing until it is on screen");
        let frame = |a: &[Action]| FrameInput::new(a.to_vec(), 0.0, vec![]);
        assert!(matches!(tip.update(&mut c, &frame(&[])), Transition::None));
        let keep = |lines: Vec<LineId>| R::KeepVoices {
            lines: lines.into_iter().map(|l| (l, Variant::None)).collect(),
        };
        let play = R::play_voice(line.clone(), Variant::None);
        assert_eq!(voice_requests(&mut c), [keep(vec![line]), play]);
        assert!(matches!(tip.update(&mut c, &frame(&[])), Transition::None));
        assert_eq!(voice_requests(&mut c), [], "asked for once");
        assert!(matches!(
            tip.update(&mut c, &frame(&[Action::Confirm])),
            Transition::Pop
        ));
        assert_eq!(voice_requests(&mut c), [R::StopVoice, keep(vec![])]);
    }

    /// Acceptance (0497): with voices off nothing is kept, asked for or
    /// stopped.
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
