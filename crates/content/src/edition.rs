//! The file that says a bundle is the Chapter 1 demo's (ticket 0914,
//! ADR-0118): `build.rs` puts it in the demo's table, and
//! [`crate::bundle::is_demo`] reads it. Shared by both, so the build and
//! the game can't disagree on it.

/// The file the demo's bundle holds that no other build has: its
/// edition's name.
pub const EDITION_FILE: &str = "edition.txt";

/// The demo edition's name, the text of [`EDITION_FILE`].
pub const DEMO: &str = "demo";
