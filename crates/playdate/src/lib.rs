//! The Playdate front end (ADR-0070): a second platform crate beside the
//! engine's `shuyi-app`. Like it, it owns the platform (the SDK's update callback,
//! the clock, files, the allocator) and holds no game or screen logic.
//!
//! `cargo xtask playdate` builds it as a static library with the `sdk`
//! feature and links it with `shim/shim.c` and the SDK's `setup.c` into
//! `pdex.elf` (the device) and `pdex.dll` (the Simulator). With `sdk` on
//! the crate is `no_std` and the platform glue in `src/pd/` is in;
//! without it only the plain parts below are, so the workspace's host
//! builds and tests never need the SDK.
//!
//! It runs the game (ticket 1033): `trpg_ui::Game::frame` once per SDK
//! update on the game's own stack, the frame drawn by the one-bit
//! renderer, the D-pad, A, B and the crank fed in as a controller (the
//! crank since ticket 1035, with the game's entries in the system menu),
//! and the game's storage on the Playdate's files. Its bundle is the
//! Playdate's, which leaves out what it does not use, and a loading card
//! is on the screen from the first frame while the data loads (ticket
//! 1036).
#![cfg_attr(feature = "sdk", no_std)]

extern crate alloc;

pub mod card;
pub mod heap;
pub mod menu;
pub mod pads;
pub mod report;
pub mod screen;
pub mod stack;
pub mod start;
pub mod storage;
pub mod text;

#[cfg(test)]
mod chapter_1;

#[cfg(feature = "sdk")]
mod pd;
