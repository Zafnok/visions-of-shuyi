//! The guide speaks the box on screen (ticket 0476): a lesson's or a tip's
//! `text`, by its line id ([`trpg_content::voice::guide_line`]). A box
//! coming on screen asks for her clip; it going, or another taking its
//! place, stops it. With voices off, or no clip, nothing is asked for or
//! stopped (0238).

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
}

impl GuideVoice {
    /// Notes the line of the box on screen as the frame ends (`None` for
    /// no box), and starts or stops her voice when it changed.
    pub fn show(&mut self, line: Option<LineId>, ctx: &mut Ctx) {
        if line == self.on_screen {
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
