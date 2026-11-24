//! What the process tool shares with the game's tool program, `trpg-tools`
//! (ADR-0129): the layered view of the game's files on disk, which
//! `check-leaks` reads here and `check-script` and `voice` read there, and
//! the names of the commands `cargo xtask` runs there.

pub mod content_layer;

/// The commands of the game's tool program, `trpg-tools`, that `cargo xtask`
/// still takes by their old names and runs there (ADR-0129). A test in
/// `trpg-tools` keeps this list its commands.
pub const FORWARDED: &[&str] = &[
    // Shuyi's tools, with the game's (ticket 1117).
    "shuyi",
    "check-keys",
    "check-text",
    "lang-status",
    "font-atlas",
    "frame-png",
    "web",
    "private-assets",
    // The engine's skill pack, embedded in `shuyi-cli` (ADR-0097).
    "skill-pack",
    // The generators.
    "icon",
    "sfx",
    "test-tileset",
    "lettering",
    "title-art",
    "voice-test-clips",
    "readme-pictures",
    // The importers.
    "map-sprite-import",
    "battler-import",
    "tileset-import",
    "portrait-import",
    "story-import",
    // The bots, the scripts and the voice.
    "playtest",
    "bot-wins",
    "skirmish",
    "lines",
    "check-script",
    "voice",
    // The Playdate.
    "playdate",
    "playdate-images",
];
