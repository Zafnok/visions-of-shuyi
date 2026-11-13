//! The guide speaks the box on screen (ticket 0476): a lesson's or a tip's
//! `text`, by its line id ([`trpg_content::voice::guide_line`]). A box
//! coming on screen asks for her clip; it going, or another taking its
//! place, stops it. With voices off, or no clip, nothing is asked for or
//! stopped (0238).
//!
//! Her coming lines are preloaded (0494): a clip not loaded 0.3 s after it
//! is asked for is skipped, and on the web it is fetched from the host. So
//! on the battle's first frame, and each time her box changes, the screen
//! hands the platform the lines the battle may still show, likeliest
//! first; it keeps the first few loaded while the player reads the box on
//! screen. The list is sent again only when it changed, or after a screen
//! over the battle (a scene) sent its own.

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
    /// The lines last handed to [`Ctx::preload_voices`]; `None` before
    /// the first frame.
    preloaded: Option<Vec<LineId>>,
}

impl GuideVoice {
    /// Whether this frame should preload her coming lines, with `line`
    /// the line of the box on screen: on the first frame and whenever it
    /// changed, while voices are on.
    pub fn preload_due(&self, line: Option<&LineId>, ctx: &Ctx) -> bool {
        (self.preloaded.is_none() || line != self.on_screen.as_ref()) && ctx.settings().voices
    }

    /// Notes that a screen is over the battle: a scene there preloads its
    /// own lines in place of hers, so her list is sent again on the first
    /// frame back.
    pub fn covered(&mut self) {
        self.preloaded = None;
    }

    /// Notes the line of the box on screen as the frame ends (`None` for
    /// no box), and starts or stops her voice when it changed. First
    /// preloads `coming` (her lines the battle may still show, likeliest
    /// first, when [`preload_due`](Self::preload_due)) if it differs from
    /// the list sent last.
    pub fn show(&mut self, line: Option<LineId>, coming: Option<Vec<LineId>>, ctx: &mut Ctx) {
        if let Some(coming) = coming
            && self.preloaded.as_ref() != Some(&coming)
        {
            ctx.preload_voices(&coming);
            self.preloaded = Some(coming);
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
mod tests {
    use super::*;
    use core::fmt::Write as _;

    use crate::AudioRequest;
    use crate::screen::tests::ctx;

    /// A context whose voice manifest has a clip for every guide line.
    fn voiced() -> (Ctx, Vec<LineId>) {
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

    fn preload(lines: &[LineId]) -> AudioRequest {
        AudioRequest::PreloadVoices {
            lines: lines
                .iter()
                .map(|l| (l.clone(), trpg_content::Variant::None))
                .collect(),
        }
    }

    /// One frame's end as the battle screen calls it: `coming` is built
    /// only when a preload is due.
    fn frame(v: &mut GuideVoice, line: Option<&LineId>, coming: &[LineId], c: &mut Ctx) {
        let coming = v.preload_due(line, c).then(|| coming.to_vec());
        v.show(line.cloned(), coming, c);
    }

    /// Acceptance (0494): the first frame preloads her coming lines before
    /// any box shows; a box coming preloads the new list before asking for
    /// its clip; an unchanged list or an unchanged box sends nothing.
    #[test]
    fn her_coming_lines_are_preloaded_first_and_at_each_change() {
        let (mut c, lines) = voiced();
        assert!(lines.len() >= 3);
        let mut v = GuideVoice::default();
        frame(&mut v, None, &lines, &mut c);
        assert_eq!(c.audio.take(), [preload(&lines)]);
        assert!(!v.preload_due(None, &c), "no change: no list built");
        frame(&mut v, None, &lines, &mut c);
        assert_eq!(c.audio.take(), []);
        // Her first box: the same list, so only her clip.
        frame(&mut v, Some(&lines[0]), &lines, &mut c);
        let play = |line: &LineId| AudioRequest::PlayVoice {
            line: line.clone(),
            variant: trpg_content::Variant::None,
        };
        assert_eq!(c.audio.take(), [play(&lines[0])]);
        // It goes and the next comes: the list without it, then her line.
        let rest = &lines[1..];
        frame(&mut v, Some(&lines[1]), rest, &mut c);
        assert_eq!(
            c.audio.take(),
            [preload(rest), AudioRequest::StopVoice, play(&lines[1])]
        );
    }

    /// A screen over the battle preloads its own lines: hers are sent
    /// again on the first frame back, even unchanged.
    #[test]
    fn her_lines_are_preloaded_again_after_a_screen_over_the_battle() {
        let (mut c, lines) = voiced();
        let mut v = GuideVoice::default();
        frame(&mut v, None, &lines, &mut c);
        assert_eq!(c.audio.take(), [preload(&lines)]);
        v.covered();
        frame(&mut v, None, &lines, &mut c);
        assert_eq!(c.audio.take(), [preload(&lines)]);
        frame(&mut v, None, &lines, &mut c);
        assert_eq!(c.audio.take(), []);
    }

    /// With voices off, nothing is preloaded and no list is built.
    #[test]
    fn voices_off_preload_nothing() {
        let (mut c, lines) = voiced();
        c.change_settings(|s| s.voices = false).unwrap();
        c.audio.take();
        let mut v = GuideVoice::default();
        assert!(!v.preload_due(None, &c));
        frame(&mut v, None, &lines, &mut c);
        frame(&mut v, Some(&lines[0]), &lines, &mut c);
        assert_eq!(c.audio.take(), []);
    }
}
