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
//! So far (ticket 1031) it loads the game's data on the game's own stack
//! and shows how long that took and how much of the stack it used. The
//! game itself runs from ticket 1033.
#![cfg_attr(feature = "sdk", no_std)]

extern crate alloc;

pub mod heap;
pub mod report;
pub mod stack;
pub mod text;

#[cfg(feature = "sdk")]
mod pd;
