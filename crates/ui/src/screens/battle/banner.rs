//! Phase and outcome banners (ticket 0405, `docs/design/turn-structure.md`):
//! `PLAYER PHASE` / `ENEMY PHASE` / `OTHER PHASE` with the turn beneath when
//! a phase starts, `VICTORY` / `DEFEAT` when the battle ends. The battle
//! screen's skin paints them.

use trpg_core::{Event, Outcome, Phase, Turn};

use super::view::BannerView;
use crate::words::Words;

/// How long a phase banner stays up unless Confirm closes it, in seconds.
/// *Tunable.*
pub const PHASE_BANNER_S: f32 = 1.0;

/// What a banner announces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BannerKind {
    /// A phase started.
    Phase {
        /// The phase.
        phase: Phase,
        /// The turn.
        turn: Turn,
    },
    /// The battle ended; stays until confirmed.
    Outcome(Outcome),
}

/// A banner on screen, and how long it has been up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Banner {
    /// What it announces.
    pub kind: BannerKind,
    /// Seconds shown so far.
    pub t: f32,
}

impl Banner {
    /// The banner for `event`, if it has one.
    pub fn for_event(event: &Event) -> Option<Self> {
        let kind = match *event {
            Event::PhaseStarted { turn, phase } => BannerKind::Phase { phase, turn },
            Event::BattleEnded { outcome } => BannerKind::Outcome(outcome),
            _ => return None,
        };
        Some(Self { kind, t: 0.0 })
    }

    /// Whether it has been up long enough to close by itself (outcome
    /// banners never do).
    pub fn expired(&self) -> bool {
        matches!(self.kind, BannerKind::Phase { .. }) && self.t >= PHASE_BANNER_S
    }

    /// The big line.
    pub fn title<'a>(&self, words: Words<'a>) -> &'a str {
        let key = match self.kind {
            BannerKind::Phase { phase, .. } => match phase {
                Phase::Player => "battle.banner.player_phase",
                Phase::Enemy => "battle.banner.enemy_phase",
                Phase::Other => "battle.banner.other_phase",
            },
            BannerKind::Outcome(Outcome::Victory) => "battle.banner.victory",
            BannerKind::Outcome(Outcome::Defeat) => "battle.banner.defeat",
        };
        words.text(key)
    }

    /// The line beneath: the turn for a phase (`Turn 2`, or `Turn 2/8`
    /// with a limit), nothing for an outcome.
    pub fn subtitle(&self, words: Words<'_>, limit: Option<Turn>) -> String {
        match self.kind {
            BannerKind::Phase { turn, .. } => match limit {
                Some(limit) => {
                    words.text_with("battle.turn_of", &[("turn", &turn), ("limit", &limit)])
                }
                None => words.text_with("battle.turn", &[("turn", &turn)]),
            },
            BannerKind::Outcome(_) => String::new(),
        }
    }

    /// What the banner shows; `limit` is the map's turn limit, if any.
    pub fn view(&self, words: Words<'_>, limit: Option<Turn>) -> BannerView {
        BannerView {
            kind: self.kind,
            title: self.title(words).to_owned(),
            subtitle: self.subtitle(words, limit),
        }
    }
}

#[cfg(test)]
mod tests {
    use trpg_core::UnitId;

    use super::*;

    fn phase(phase: Phase, turn: Turn) -> Banner {
        Banner {
            kind: BannerKind::Phase { phase, turn },
            t: 0.0,
        }
    }

    #[test]
    fn events_with_banners() {
        assert_eq!(
            Banner::for_event(&Event::PhaseStarted {
                turn: 2,
                phase: Phase::Enemy
            }),
            Some(phase(Phase::Enemy, 2))
        );
        let ended = Banner::for_event(&Event::BattleEnded {
            outcome: Outcome::Defeat,
        });
        assert_eq!(
            ended.map(|b| b.kind),
            Some(BannerKind::Outcome(Outcome::Defeat))
        );
        assert_eq!(
            Banner::for_event(&Event::UnitActed { unit: UnitId(1) }),
            None
        );
    }

    #[test]
    fn titles_and_turns() {
        let titles =
            [Phase::Player, Phase::Enemy, Phase::Other].map(|p| phase(p, 1).title(Words::ENGLISH));
        assert_eq!(titles, ["PLAYER PHASE", "ENEMY PHASE", "OTHER PHASE"]);
        assert_eq!(
            phase(Phase::Player, 3).subtitle(Words::ENGLISH, None),
            "Turn 3"
        );
        assert_eq!(
            phase(Phase::Player, 3).subtitle(Words::ENGLISH, Some(8)),
            "Turn 3/8"
        );
        let won = Banner {
            kind: BannerKind::Outcome(Outcome::Victory),
            t: 0.0,
        };
        assert_eq!(won.title(Words::ENGLISH), "VICTORY");
        assert_eq!(won.subtitle(Words::ENGLISH, Some(8)), "");
        assert_eq!(
            phase(Phase::Enemy, 2).view(Words::ENGLISH, Some(8)),
            BannerView {
                kind: BannerKind::Phase {
                    phase: Phase::Enemy,
                    turn: 2
                },
                title: "ENEMY PHASE".to_owned(),
                subtitle: "Turn 2/8".to_owned(),
            }
        );
    }

    #[test]
    fn only_phase_banners_expire() {
        let mut b = phase(Phase::Player, 1);
        b.t = PHASE_BANNER_S * 0.99;
        assert!(!b.expired());
        b.t = PHASE_BANNER_S;
        assert!(b.expired());
        let lost = Banner {
            kind: BannerKind::Outcome(Outcome::Defeat),
            t: 100.0,
        };
        assert!(!lost.expired());
        assert_eq!(lost.title(Words::ENGLISH), "DEFEAT");
    }
}
