//! Moving looks and their stills (ADR-0131): every look the game ships
//! that moves by itself (a tile that loops through frames, an effect of
//! drifting particles) says the lowest graphics tier it plays at, and
//! names its still for each tier below, so a lesser target (the
//! Playdate, a GBA later) shows it standing.
//!
//! A tier is the engine's ([`GraphicsTier`], a fact of the target's
//! profile). The skins ask it through `ctx.platform.graphics` and draw
//! what [`Motion::drawn`] says. [`check`] reads every moving look and
//! reports a still left out, and lists each still that is still to do
//! ([`Still::todo`]): a missing picture is never a crash.
//!
//! In a data file, beside the look's frames:
//!
//! ```ron
//! motion: (
//!     // Plays at `Full` only, unless said: `lowest: Reduced`.
//!     stills: {
//!         Reduced: (show: FirstFrame),
//!         // No target is `Minimal` yet: a future GBA's (TODO(gba)).
//!         Minimal: (show: FirstFrame, todo: "a one-bit tile"),
//!     },
//! ),
//! ```
use crate::prelude::*;

use alloc::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

use crate::Content;

pub use shuyi_assets::target::GraphicsTier;

/// How a moving look shows on each graphics tier.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Motion {
    /// The lowest tier it moves at: [`GraphicsTier::Full`] unless said.
    #[serde(default)]
    pub lowest: GraphicsTier,
    /// What it shows on each tier below [`lowest`](Self::lowest): one for
    /// every such tier, [`check`] says.
    #[serde(default)]
    pub stills: BTreeMap<GraphicsTier, Still>,
}

/// A moving look's still, on one tier.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Still {
    /// What is drawn.
    pub show: Show,
    /// What is still to do for this tier, when the still drawn stands in
    /// for one not made yet (a lighter one-bit picture): [`check`] lists
    /// it. Written as the text alone (`todo: "a one-bit tile"`).
    #[serde(default, deserialize_with = "written")]
    pub todo: Option<String>,
}

/// A field that may be left out: `Some` of the text written.
fn written<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    String::deserialize(d).map(Some)
}

/// What a moving look shows standing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum Show {
    /// Its first frame (a tile's frame 0), held.
    FirstFrame,
    /// Nothing: the look is left out (an effect over the map).
    Nothing,
}

/// What a skin draws of a moving look on a tier ([`Motion::drawn`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drawn {
    /// The look, moving.
    Moving,
    /// Its still.
    Still(Show),
}

impl Motion {
    /// What a skin draws of the look at `tier`: the look moving at its
    /// [`lowest`](Self::lowest) tier and above, its still below. A still
    /// left out (which [`check`] reports) is its first frame.
    pub fn drawn(&self, tier: GraphicsTier) -> Drawn {
        if tier.plays(self.lowest) {
            return Drawn::Moving;
        }
        Drawn::Still(self.stills.get(&tier).map_or(Show::FirstFrame, |s| s.show))
    }
}

/// Every look `content` ships that moves by itself, named as a message
/// shows it, with its [`Motion`]: what [`check`] reads. None moves yet.
// TODO(1315): each tileset's rows that loop through frames.
// TODO(1316): each map effect of drifting particles.
pub fn moving_looks(_content: &Content) -> Vec<(String, &Motion)> {
    Vec::new()
}

/// What [`check`] found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StillsReport {
    /// A tier below a look's lowest with no still, or a still for a tier
    /// the look moves at: each an error.
    pub problems: Vec<String>,
    /// Each still to do, for a later ticket: never an error.
    pub todo: Vec<String>,
}

/// Every moving look of `looks`, each named as a message shows it
/// (`tileset "grass", terrain "water"`), checked for a still on each tier
/// below its lowest.
pub fn check<'a>(looks: impl IntoIterator<Item = (String, &'a Motion)>) -> StillsReport {
    let mut report = StillsReport::default();
    for (name, motion) in looks {
        for tier in GraphicsTier::ALL {
            let still = motion.stills.get(&tier);
            match (tier.plays(motion.lowest), still) {
                (false, None) => report.problems.push(format!(
                    "{name}: no still for {tier}, where it doesn't move (it moves from {})",
                    motion.lowest
                )),
                (true, Some(_)) => report.problems.push(format!(
                    "{name}: a still for {tier}, where it moves (from {})",
                    motion.lowest
                )),
                (
                    false,
                    Some(Still {
                        todo: Some(todo), ..
                    }),
                ) => {
                    report.todo.push(format!("{name}: {tier}: {todo}"));
                }
                _ => {}
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ron_loader::parse_ron;

    fn motion(source: &str) -> Motion {
        parse_ron("motion.ron", source).unwrap_or_else(|e| panic!("{e}"))
    }

    /// A tile row that loops, with the stills `stills` (as written).
    fn water(stills: &str) -> (String, Motion) {
        (
            "terrain \"water\"".to_owned(),
            motion(&format!("(stills: {{{stills}}})")),
        )
    }

    #[test]
    fn a_moving_tile_with_no_still_fails_the_check() {
        let (name, bare) = water("");
        let report = check([(name, &bare)]);
        assert_eq!(
            report.problems,
            [
                "terrain \"water\": no still for Minimal, where it doesn't move (it moves from Full)",
                "terrain \"water\": no still for Reduced, where it doesn't move (it moves from Full)",
            ]
        );
        assert_eq!(report.todo, Vec::<String>::new());
    }

    #[test]
    fn a_still_to_do_is_listed_not_failed() {
        let (name, todo) = water(
            "Reduced: (show: FirstFrame), \
             Minimal: (show: FirstFrame, todo: \"a one-bit water tile\")",
        );
        let report = check([(name, &todo)]);
        assert_eq!(report.problems, Vec::<String>::new());
        assert_eq!(
            report.todo,
            ["terrain \"water\": Minimal: a one-bit water tile"]
        );
    }

    #[test]
    fn a_look_that_moves_from_reduced_needs_a_still_for_minimal_alone() {
        let petals = motion("(lowest: Reduced, stills: {Minimal: (show: Nothing)})");
        assert_eq!(
            check([("petals".to_owned(), &petals)]),
            StillsReport::default()
        );
        let extra = motion(
            "(lowest: Reduced, stills: {Minimal: (show: Nothing), Reduced: (show: Nothing)})",
        );
        assert_eq!(
            check([("petals".to_owned(), &extra)]).problems,
            ["petals: a still for Reduced, where it moves (from Reduced)"]
        );
    }

    #[test]
    fn a_skin_draws_the_look_moving_from_its_lowest_tier_and_its_still_below() {
        use GraphicsTier::{Full, Minimal, Reduced};
        let (_, water) = water("Reduced: (show: FirstFrame), Minimal: (show: Nothing)");
        assert_eq!(water.drawn(Full), Drawn::Moving);
        assert_eq!(water.drawn(Reduced), Drawn::Still(Show::FirstFrame));
        assert_eq!(water.drawn(Minimal), Drawn::Still(Show::Nothing));
        // A still left out is drawn as the first frame, never a crash.
        let bare = motion("()");
        assert_eq!(bare.lowest, Full);
        assert_eq!(bare.drawn(Minimal), Drawn::Still(Show::FirstFrame));
        let cheap = motion("(lowest: Reduced)");
        assert_eq!(cheap.drawn(Reduced), Drawn::Moving);
    }

    #[test]
    fn a_wrong_tier_or_show_is_refused_with_where() {
        for source in [
            "(lowest: Low)",
            "(stills: {Reduced: (show: Frame0)})",
            "(speed: 2)",
        ] {
            let error = parse_ron::<Motion>("motion.ron", source).unwrap_err();
            assert!(error.to_string().starts_with("motion.ron:1:"), "{error}");
        }
    }
}
