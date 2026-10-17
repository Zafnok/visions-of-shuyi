//! The Playdate front end (ADR-0070): a second platform crate beside
//! `trpg-app`. Like `app` it owns the platform (the SDK's update callback,
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
//! and the game's storage on the Playdate's files.
#![cfg_attr(feature = "sdk", no_std)]

extern crate alloc;

pub mod heap;
pub mod menu;
pub mod pads;
pub mod report;
pub mod screen;
pub mod stack;
pub mod storage;
pub mod text;

#[cfg(feature = "sdk")]
mod pd;
