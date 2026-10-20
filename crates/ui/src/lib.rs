//! Glyph buffer, screens and input handling, independent of macroquad. See
//! ADR-0004, and `crates/ui/README.md` for how screens fit together.
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// What `std`'s prelude gives for free, for the `no_std` build.
mod prelude {
    pub use alloc::borrow::ToOwned;
    pub use alloc::boxed::Box;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec::Vec;
    pub use alloc::{format, vec};

    #[cfg(not(feature = "std"))]
    pub use shuyi_ui::float::{Float, Round};

    // The lookups every screen makes on the context.
    pub use crate::screen::ShuyiCtx as _;
    // A menu label's text (until the game's menus hold text keys, 1162).
    pub use crate::widgets::menu::LabelText as _;
}

pub mod audio;
pub mod cinema;
pub mod cinematic;
pub mod color;
pub mod console;
pub mod debug;
pub mod dialogue;
pub mod flow;
pub mod game;

pub mod glyph_buffer;
#[cfg(any(test, feature = "harness"))]
pub mod harness;
pub mod input;
pub mod map_view;
pub mod onebit;
pub mod portrait;
pub mod profiles;
pub mod save;
pub mod screen;
pub mod screens;
pub mod settings;
pub mod snapshot;
pub mod storage;
pub mod tips;
pub mod widgets;
pub mod words;

pub use audio::{AudioQueue, AudioRequest, MusicClock, MusicCommand, MusicState};
pub use color::{Palette, Rgb, UiColor};
pub use console::{ConsoleSizes, ConsoleSpec};
pub use game::{FrameOutput, Game, RawInputEvent};
pub use glyph_buffer::{
    Backdrop, BoxStyle, Cell, GlyphBuffer, Item, Layer, Overlay, Paint, PxRect, Rect, Sprite,
};
pub use screen::{
    Ctx, FrameInput, KeyPrompt, LoadError, Screen, ScreenStack, Shuyi, ShuyiCtx, Transition,
};
pub use settings::{Settings, ShuyiSettings};
pub use shuyi_ui::{GameHooks, ScreenHook};
pub use storage::{MemoryStorage, Storage, StorageError};
pub use trpg_content::ImageId;
pub use words::{Language, Words};
