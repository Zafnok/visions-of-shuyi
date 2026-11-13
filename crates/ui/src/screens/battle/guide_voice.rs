//! The guide speaks the box on screen (ticket 0476): a lesson's or a tip's
//! `text`, by its line id ([`trpg_content::voice::guide_line`]). A box
//! coming on screen asks for her clip; it going, or another taking its
//! place, stops it. With voices off, or no clip, nothing is asked for or
//! stopped (0238).
//!
//! Her coming lines are kept loaded (0494, 0496): a clip not loaded 0.3 s
//! after it is asked for is skipped, and on the web it is fetched from the
//! host. So on the battle's first frame, and each time her box changes,
//! the screen hands the platform the lines the battle may still show
//! ([`Ctx::keep_voices`]), in any order: a tip fires the first time the
//! player does something, which the game can't foresee, so every one is
//! kept. The set is sent again only when it changed, or on the first frame
//! back after a screen over the battle, and an empty one when the battle
//! leaves the stack ([`GuideVoice::leave`]).

use crate::prelude::*;
use trpg_content::LineId;

use crate::screen::Ctx;

/// What the guide is saying on one battle screen.
#[derive(Debug, Clone, Default)]
pub struct GuideVoice {
    /// The line of the box on screen as the last frame ended, if any.
    on_screen: Option<LineId>,
    /// Whether her clip of that line was asked for: only then is there a
    /// voice to stop.
    speaking: bool,
    /// The last line she was asked to say: a box back on screen after a
    /// pause (a combat playing over an open lesson) doesn't say it again.
    said: Option<LineId>,
    /// The voiced lines last handed to [`Ctx::keep_voices`]: what the
    /// platform keeps loaded for her.
    kept: Vec<LineId>,
    /// Whether her set was sent since the battle came on top (its first
    /// frame, or back from a screen over it).
    sent: bool,
}

impl GuideVoice {
    /// Whether this frame should send the set of her coming lines, with
    /// `line` the line of the box on screen: on the first frame and
    /// whenever it changed, while voices are on.
    pub fn keep_due(&self, line: Option<&LineId>, ctx: &Ctx) -> bool {
        (!self.sent || line != self.on_screen.as_ref()) && ctx.settings().voices
    }

    /// Notes that a screen is over the battle: her set is sent again on
    /// the first frame back.
    pub fn covered(&mut self) {
        self.sent = false;
    }

    /// The battle leaves the stack: her lines are let go, if any was
    /// kept (an empty set sent when none was would only be noise).
    pub fn leave(&mut self, ctx: &mut Ctx) {
        if !core::mem::take(&mut self.kept).is_empty() {
            ctx.keep_voices(&[]);
        }
        self.sent = false;
    }

    /// Notes the line of the box on screen as the frame ends (`None` for
    /// no box), and starts or stops her voice when it changed. First
    /// keeps `coming` loaded (her lines the battle may still show, when
    /// [`keep_due`](Self::keep_due)): its lines with a clip, sent if they
    /// differ from the set sent last, or on the first frame on top with
    /// any.
    pub fn show(&mut self, line: Option<LineId>, coming: Option<Vec<LineId>>, ctx: &mut Ctx) {
        if let Some(mut coming) = coming {
            coming.retain(|l| ctx.has_voice(l));
            let resend = !self.sent && !coming.is_empty();
            if resend || coming != self.kept {
                ctx.keep_voices(&coming);
                self.kept = coming;
            }
            self.sent = true;
        }
        let changed = line != self.on_screen;
        if !changed {
            return;
        }
        if core::mem::take(&mut self.speaking) {
            ctx.stop_voice();
        }
        if let Some(line) = &line
            && self.said.as_ref() != Some(line)
            && ctx.has_voice(line)
        {
            ctx.play_voice(line);
            self.speaking = true;
            self.said = Some(line.clone());
        }
        self.on_screen = line;
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use core::fmt::Write as _;

    use crate::AudioRequest;
    use crate::screen::tests::ctx;

    /// A context whose voice manifest has a clip for every guide line
    /// (the tip box outside battle uses it too, 0497).
    pub(crate) fn voiced() -> (Ctx, Vec<LineId>) {
        let mut c = ctx();
        let content = &c.game.content;
        let scene = trpg_content::voice::guide_scene(&content.lessons, &content.tips);
        let lines = scene.lines();
        let mut clips = String::new();
        for l in &lines {
            let _ = writeln!(
                clips,
                "(line: \"{}\", spoken: \"{}\", voice: \"guide\",                  made_by: Generated(tool: \"t\", model: \"m\", date: \"2026-11-12\")),",
                l.id, l.text
            );
        }
        let ids = lines.iter().map(|l| l.id.clone()).collect();
        c.set_voice_manifest(&format!("(clips: [\n{clips}])"));
        assert_eq!(c.take_warnings(), [] as [String; 0]);
        c.audio.take();
        (c, ids)
    }

    fn keep(lines: &[LineId]) -> AudioRequest {
        AudioRequest::KeepVoices {
            lines: lines
                .iter()
                .map(|l| (l.clone(), trpg_content::Variant::None))
                .collect(),
        }
    }

    /// One frame's end as the battle screen calls it: `coming` is built
    /// only when sending the set is due.
    fn frame(v: &mut GuideVoice, line: Option<&LineId>, coming: &[LineId], c: &mut Ctx) {
        let coming = v.keep_due(line, c).then(|| coming.to_vec());
        v.show(line.cloned(), coming, c);
    }

    /// Acceptance (0494, 0496): the first frame keeps her coming lines
    /// loaded before any box shows; a box coming sends the new set before
    /// asking for its clip; an unchanged set or an unchanged box sends
    /// nothing.
    #[test]
    fn her_coming_lines_are_kept_first_and_at_each_change() {
        let (mut c, lines) = voiced();
        assert!(lines.len() >= 3);
        let mut v = GuideVoice::default();
        frame(&mut v, None, &lines, &mut c);
        assert_eq!(c.audio.take(), [keep(&lines)]);
        assert!(!v.keep_due(None, &c), "no change: no set built");
        frame(&mut v, None, &lines, &mut c);
        assert_eq!(c.audio.take(), []);
        // Her first box: the same list, so only her clip.
        frame(&mut v, Some(&lines[0]), &lines, &mut c);
        let play = |line: &LineId| AudioRequest::PlayVoice {
            line: line.clone(),
            variant: trpg_content::Variant::None,
        };
        assert_eq!(c.audio.take(), [play(&lines[0])]);
        // It goes and the next comes: the set without it, then her line.
        let rest = &lines[1..];
        frame(&mut v, Some(&lines[1]), rest, &mut c);
        assert_eq!(
            c.audio.take(),
            [keep(rest), AudioRequest::StopVoice, play(&lines[1])]
        );
    }

    /// After a screen over the battle, her set is sent again on the first
    /// frame back, even unchanged.
    #[test]
    fn her_lines_are_kept_again_after_a_screen_over_the_battle() {
        let (mut c, lines) = voiced();
        let mut v = GuideVoice::default();
        frame(&mut v, None, &lines, &mut c);
        assert_eq!(c.audio.take(), [keep(&lines)]);
        v.covered();
        frame(&mut v, None, &lines, &mut c);
        assert_eq!(c.audio.take(), [keep(&lines)]);
        frame(&mut v, None, &lines, &mut c);
        assert_eq!(c.audio.take(), []);
    }

    /// Acceptance (0496): leaving the battle lets her lines go, and a
    /// battle on screen again sends its set anew.
    #[test]
    fn leaving_lets_her_lines_go() {
        let (mut c, lines) = voiced();
        let mut v = GuideVoice::default();
        frame(&mut v, None, &lines, &mut c);
        c.audio.take();
        v.leave(&mut c);
        assert_eq!(c.audio.take(), [keep(&[])]);
        frame(&mut v, None, &lines, &mut c);
        assert_eq!(c.audio.take(), [keep(&lines)]);
    }

    /// Lines with no clip are left out of the set; with none, nothing is
    /// sent, on the first frame or on leaving.
    #[test]
    fn no_clip_keeps_nothing() {
        let (mut c, lines) = voiced();
        let mut v = GuideVoice::default();
        let unvoiced = [LineId::new("vix_00000000")];
        frame(&mut v, None, &unvoiced, &mut c);
        v.leave(&mut c);
        assert_eq!(c.audio.take(), []);
        let mixed = [unvoiced[0].clone(), lines[0].clone()];
        frame(&mut v, None, &mixed, &mut c);
        assert_eq!(c.audio.take(), [keep(&lines[..1])]);
    }

    /// With voices off, nothing is kept and no set is built.
    #[test]
    fn voices_off_keep_nothing() {
        let (mut c, lines) = voiced();
        c.change_settings(|s| s.voices = false).unwrap();
        c.audio.take();
        let mut v = GuideVoice::default();
        assert!(!v.keep_due(None, &c));
        frame(&mut v, None, &lines, &mut c);
        frame(&mut v, Some(&lines[0]), &lines, &mut c);
        assert_eq!(c.audio.take(), []);
    }
}
