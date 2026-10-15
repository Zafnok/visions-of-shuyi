//! What the dialogue log remembers (ticket 0729, `docs/design/look-and-feel.md`,
//! *Dialogue log*): every line of the scene the player has seen, as the
//! scene was on it, and which one is on screen. Pure: the dialogue screen
//! tells it what happens; jumping hands back the scene to show.

use trpg_content::MusicLine;

use crate::dialogue::DialoguePlayer;

/// One line the player has seen: a text box, or a reply they picked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    /// The scene as it was there: on that text box, or (for a reply) at the
    /// choice, before the reply was picked.
    pub player: DialoguePlayer,
    /// The reply picked, for a reply (its index in the choice's options).
    pub reply: Option<usize>,
    /// The music playing there (`None` if the scene hadn't set any yet).
    pub music: Option<MusicLine>,
}

/// The lines seen in this scene, oldest first, and which one is on screen.
///
/// After a jump back, the lines seen past the one on screen stay (the log
/// shows them dimmer, and the player can jump forward to them), except
/// from the first reply on: a reply is asked again. So no reply is ever
/// past the line on screen, and reading on from it shows the lines already
/// kept, in order, until the last.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct History {
    seen: Vec<Seen>,
    /// Index in `seen` of the line on screen (0 while there is none).
    at: usize,
}

impl History {
    /// Nothing seen yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Every line seen, oldest first.
    pub fn seen(&self) -> &[Seen] {
        &self.seen
    }

    /// Whether nothing has been seen.
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }

    /// The index of the line on screen in [`seen`](Self::seen).
    pub fn at(&self) -> usize {
        self.at
    }

    /// The index of the furthest line seen (0 while there is none).
    pub fn latest(&self) -> usize {
        self.seen.len().saturating_sub(1)
    }

    /// `player` shows a new text box, with `music` playing: the next line
    /// already kept (reading on after a jump back), else a new one.
    pub fn reached(&mut self, player: &DialoguePlayer, music: Option<MusicLine>) {
        if self.at + 1 < self.seen.len() {
            self.at += 1;
            return;
        }
        self.push(Seen {
            player: player.clone(),
            reply: None,
            music,
        });
    }

    /// The player picks reply `option` of the choice open in `player`
    /// (before it is played), with `music` playing. The lines kept past the
    /// one on screen all come before the choice (a reply is never past
    /// it), so they stay: the reply follows the last of them.
    pub fn replied(&mut self, player: &DialoguePlayer, option: usize, music: Option<MusicLine>) {
        self.at = self.latest();
        self.push(Seen {
            player: player.clone(),
            reply: Some(option),
            music,
        });
    }

    /// Goes to line `i`: it is the line on screen, and the lines from the
    /// first reply past it on are forgotten (the reply is asked again).
    /// Going to a reply forgets it too, and the line it answered is on
    /// screen, with its choice open. Returns what to show, or `None` (and
    /// changes nothing) if there is no line `i`.
    pub fn jump(&mut self, i: usize) -> Option<Seen> {
        let target = self.seen.get(i)?.clone();
        let first_reply = (i..self.seen.len()).find(|&j| self.seen[j].reply.is_some());
        if let Some(r) = first_reply {
            self.seen.truncate(r);
        }
        self.at = i.min(self.latest());
        Some(target)
    }

    fn push(&mut self, seen: Seen) {
        self.seen.push(seen);
        self.at = self.latest();
    }
}

#[cfg(test)]
mod tests;
