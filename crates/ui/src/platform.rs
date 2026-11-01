//! The facts of the target the game's own Playdate front end runs on
//! (ticket 1200). The engine's window backend reads its targets' profiles
//! itself; `crates/playdate` and the tools' Playdate start read this one.

use crate::prelude::*;

use shuyi_assets::target::TargetProfile;
use shuyi_ui::PlatformFacts;

/// The Playdate's profile.
const PLAYDATE_PROFILE: &str = include_str!("playdate_target.ron");

/// The Playdate's facts, from its profile.
pub fn playdate_facts() -> Result<PlatformFacts, String> {
    TargetProfile::from_source("playdate.ron", "playdate", PLAYDATE_PROFILE)
        .map(|profile| PlatformFacts::from_profile(&profile))
        .map_err(|errors| {
            let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
            errors.join("; ")
        })
}
