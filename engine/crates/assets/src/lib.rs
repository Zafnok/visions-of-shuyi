//! Shuyi Engine's assets: the bundle of files a game embeds, laid in layers
//! the game gives ([`bundle`]), errors that name a file, line and column
//! ([`error`]), RON files read with those errors ([`ron`]), and the `.map`
//! text format of a grid map ([`grid_map`]).
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// What `std`'s prelude gives for free, for the `no_std` build.
mod prelude {
    pub use alloc::borrow::ToOwned;
    pub use alloc::boxed::Box;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec::Vec;
    pub use alloc::{format, vec};
}

pub mod bundle;
pub mod error;
pub mod grid_map;
pub mod ron;

pub use bundle::{Bundle, Files, Layer};
pub use error::{ContentError, ContentErrors, line_of};
pub use grid_map::{GridMap, MapFeature, MapFormat, MapLook};
pub use ron::parse_ron;
